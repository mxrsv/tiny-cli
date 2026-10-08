#if TINY_SMOKE_HOOKS
import AppKit

/// Development-only checks, compiled in by `scripts/build-native-app.sh --smoke-hooks`.
extension Main {
    @MainActor static func runSmokeHook(_ arguments: [String]) async throws -> Bool {
        guard arguments.count == 3 && arguments[1] == "--quit-test-app" else { return false }
        try await quitTestApp(arguments[2])
        return true
    }

    /// O1(a) check: launches a disposable app from /tmp, then quits only that
    /// instance through the production Quit path. Refuses an already-running app.
    @MainActor fileprivate static func quitTestApp(_ path: String) async throws {
        let url = URL(fileURLWithPath: path).resolvingSymlinksInPath().standardizedFileURL
        // Foundation reports /private/tmp as /tmp after resolving symlinks.
        guard url.path.hasPrefix("/tmp/") || url.path.hasPrefix("/private/tmp/"), url.pathExtension == "app",
              let bundleID = Bundle(url: url)?.bundleIdentifier else {
            throw failure(6, "Expected an .app bundle under /tmp")
        }
        guard NSRunningApplication.runningApplications(withBundleIdentifier: bundleID).isEmpty else {
            throw failure(7, "\(bundleID) is already running; only an instance launched here may be quit")
        }
        let configuration = NSWorkspace.OpenConfiguration()
        configuration.activates = false
        configuration.createsNewApplicationInstance = true
        let started = Date()
        let app = try await NSWorkspace.shared.openApplication(at: url, configuration: configuration)
        do {
            guard app.bundleURL?.resolvingSymlinksInPath().standardizedFileURL.path == url.path,
                  let bundlePath = app.bundleURL?.standardizedFileURL.path,
                  let launchDate = app.launchDate, launchDate.timeIntervalSince(started) > -1 else {
                throw failure(8, "The launched app does not match \(url.path)")
            }
            let deadline = ContinuousClock.now + .seconds(10)
            while !app.isFinishedLaunching && ContinuousClock.now < deadline { try await Task.sleep(for: .milliseconds(100)) }
            let outcome = await AppQuit.quit(AppQuitTarget(name: app.localizedName ?? bundleID, bundlePath: bundlePath,
                                                           pid: app.processIdentifier, launchDate: launchDate))
            guard outcome == .exited else { throw failure(9, "Quit outcome \(outcome) for PID \(app.processIdentifier)") }
        } catch {
            await discard(app, bundleID: bundleID) // Any failed guard: clean up the instance launched above.
            throw error
        }
        print("PASS: launched \(bundleID) PID \(app.processIdentifier) from \(url.path); NSRunningApplication.terminate() → exited")
    }

    fileprivate static func failure(_ code: Int, _ message: String) -> NSError {
        NSError(domain: "Tiny", code: code, userInfo: [NSLocalizedDescriptionKey: message])
    }

    /// No instance of `bundleID` ran before the launch, so one carrying it is ours.
    @MainActor private static func discard(_ app: NSRunningApplication, bundleID: String) async {
        guard app.bundleIdentifier == bundleID, !app.isTerminated else { return }
        app.terminate()
        let deadline = ContinuousClock.now + .seconds(2)
        while !app.isTerminated && ContinuousClock.now < deadline { try? await Task.sleep(for: .milliseconds(100)) }
        if !app.isTerminated { app.forceTerminate() }
    }
}
#else
extension Main {
    @MainActor static func runSmokeHook(_ arguments: [String]) async throws -> Bool { false }
}
#endif
