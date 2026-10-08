import AppKit
import SwiftUI
import TinyEngine

struct TinyNativeApp: App {
    @State private var state = AppState()
    var body: some Scene {
        Window("Tiny Dev", id: "processes") {
            ProcessesView(state: state)
        }
        .windowStyle(.hiddenTitleBar)
        .defaultSize(width: 1240, height: 850)
        .commands { CommandGroup(replacing: .newItem) {} }
    }
}

@main enum Main {
    @MainActor static func main() async {
        let arguments = CommandLine.arguments
        do {
            if arguments.count == 2 && arguments[1] == "--smoke-test" {
                try await smokeTest()
            } else if (3...4).contains(arguments.count) && arguments[1] == "--snapshot" {
                guard arguments[2].hasPrefix("/"), arguments[2].hasSuffix(".png"),
                      let scene = SnapshotScene(rawValue: arguments.count == 4 ? arguments[3] : "member") else {
                    throw CocoaError(.fileWriteInvalidFileName)
                }
                try await snapshot(to: URL(fileURLWithPath: arguments[2]), scene: scene)
            } else if arguments.count == 3 && arguments[1] == "--quit-test-app" {
                try await quitTestApp(arguments[2])
            } else if arguments.count == 1 || arguments.dropFirst().allSatisfy({ $0.hasPrefix("-psn_") }) {
                NSApplication.shared.setActivationPolicy(.regular)
                TinyNativeApp.main()
            } else {
                throw NSError(domain: "Tiny", code: 1, userInfo: [NSLocalizedDescriptionKey:
                    "Usage: Tiny [--smoke-test | --snapshot /absolute/path.png [member|app|notice|minimum] | --quit-test-app /tmp/Name.app]"])
            }
        } catch {
            FileHandle.standardError.write(Data("Tiny: \(error.localizedDescription)\n".utf8))
            exit(1)
        }
    }

    private static func smokeTest() async throws {
        let engine = Engine()
        let child = Process()
        child.executableURL = URL(fileURLWithPath: "/bin/sleep")
        child.arguments = ["30"]
        try child.run()
        defer { if child.isRunning { child.terminate(); child.waitUntilExit() } }
        let first = try await engine.list()
        let ownPID = UInt32(ProcessInfo.processInfo.processIdentifier)
        guard let own = first.processes.first(where: { $0.pid == ownPID }),
              first.processes.contains(where: { $0.pid == UInt32(child.processIdentifier) }) else {
            throw NSError(domain: "Tiny", code: 2, userInfo: [NSLocalizedDescriptionKey: "Own process or test child missing"])
        }
        let detail = try await engine.detail(own.identity)
        child.terminate()
        child.waitUntilExit()
        let after = try await engine.list()
        guard !after.processes.contains(where: { $0.pid == UInt32(child.processIdentifier) }) else {
            throw NSError(domain: "Tiny", code: 3, userInfo: [NSLocalizedDescriptionKey: "Exited test child still present"])
        }
        guard let systemCPU = after.systemUsage.cpuPercent, (0...100).contains(systemCPU),
              let used = after.systemUsage.memoryUsedBytes, let total = after.systemUsage.memoryTotalBytes,
              total > 0, used <= total else {
            throw NSError(domain: "Tiny", code: 5, userInfo: [NSLocalizedDescriptionKey: "Invalid whole-machine usage sample"])
        }
        print("PASS: \(first.processes.count) real processes; own PID \(ownPID); owned child appeared and exited; detail PID \(detail.process.pid); first CPU unmeasured \(!first.cpuMeasured)")
        print("System CPU \(systemCPU)% (all cores); RAM \(used)/\(total) bytes")
    }

    enum SnapshotScene: String { case member, app, notice, minimum }

    /// Renders the app's own view with real data. Scenes only select or stage
    /// copy; nothing is confirmed, and the defaults suite is throwaway.
    @MainActor private static func snapshot(to url: URL, scene: SnapshotScene) async throws {
        NSApplication.shared.setActivationPolicy(.prohibited)
        let suite = "com.mxrsv.tiny.dev.snapshot.\(UUID().uuidString)"
        guard let defaults = UserDefaults(suiteName: suite) else { throw CocoaError(.fileWriteUnknown) }
        defer { defaults.removePersistentDomain(forName: suite) }
        if scene == .notice { InFlightMarker(defaults: defaults).begin("Force Quit “sleep” (PID 4242)") }
        let child = Process()
        child.executableURL = URL(fileURLWithPath: "/bin/sleep")
        child.arguments = ["60"]
        try child.run()
        defer { if child.isRunning { child.terminate(); child.waitUntilExit() } }
        let state = AppState(defaults: defaults)
        state.requestRefresh()
        await state.waitForRefresh()
        guard state.listError == nil, !state.processes.isEmpty else {
            throw NSError(domain: "Tiny", code: 4, userInfo: [NSLocalizedDescriptionKey: state.listError ?? "No process data"])
        }
        let ownPID = UInt32(ProcessInfo.processInfo.processIdentifier)
        if scene == .app, let app = state.visibleGroups.first(where: { group in
            group.isApplication && !group.members.contains { $0.pid == ownPID }
        }) {
            state.selectGroup(app.id)
        } else if let owned = state.processes.first(where: { $0.pid == UInt32(child.processIdentifier) }) {
            state.select(owned.identity)
            await state.waitForRefresh()
        }
        state.requestRefresh()
        await state.waitForRefresh()
        let size = scene == .minimum ? NSSize(width: 1060, height: 740) : NSSize(width: 1240, height: 850)
        let host = NSHostingView(rootView: ProcessesView(state: state, startsPolling: false)
            .frame(width: size.width, height: size.height))
        let window = NSWindow(contentRect: NSRect(origin: .zero, size: size), styleMask: [.borderless],
                              backing: .buffered, defer: false)
        window.appearance = NSAppearance(named: .darkAqua)
        window.contentView = host
        host.layoutSubtreeIfNeeded()
        try await Task.sleep(for: .seconds(1))
        try withExtendedLifetime(window) { try writeSnapshot(host: host, to: url) }
        print("Rendered \(state.processes.count) live processes to \(url.path)")
    }

    /// O1(a) check: launches a disposable app from /tmp, then quits only that
    /// instance through the production Quit path. Refuses an already-running app.
    @MainActor private static func quitTestApp(_ path: String) async throws {
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
        guard app.bundleURL?.resolvingSymlinksInPath().standardizedFileURL.path == url.path,
              let bundlePath = app.bundleURL?.standardizedFileURL.path,
              let launchDate = app.launchDate, launchDate.timeIntervalSince(started) > -1 else {
            throw failure(8, "The launched app does not match \(url.path)")
        }
        let deadline = ContinuousClock.now + .seconds(10)
        while !app.isFinishedLaunching && ContinuousClock.now < deadline { try await Task.sleep(for: .milliseconds(100)) }
        let outcome = await AppQuit.quit(AppQuitTarget(name: app.localizedName ?? bundleID, bundlePath: bundlePath,
                                                       pid: app.processIdentifier, launchDate: launchDate))
        guard outcome == .exited else {
            app.forceTerminate() // Cleanup of the instance launched above only.
            throw failure(9, "Quit outcome \(outcome) for PID \(app.processIdentifier)")
        }
        print("PASS: launched \(bundleID) PID \(app.processIdentifier) from \(url.path); NSRunningApplication.terminate() → exited")
    }

    private static func failure(_ code: Int, _ message: String) -> NSError {
        NSError(domain: "Tiny", code: code, userInfo: [NSLocalizedDescriptionKey: message])
    }

    @MainActor private static func writeSnapshot(host: NSView, to url: URL) throws {
        host.layoutSubtreeIfNeeded()
        guard let bitmap = host.bitmapImageRepForCachingDisplay(in: host.bounds) else { throw CocoaError(.fileWriteUnknown) }
        host.cacheDisplay(in: host.bounds, to: bitmap)
        guard let png = bitmap.representation(using: .png, properties: [:]) else { throw CocoaError(.fileWriteUnknown) }
        try png.write(to: url, options: .atomic)
    }
}
