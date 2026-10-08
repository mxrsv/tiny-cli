import AppKit
import TinyEngine

/// The one Swift-owned action (O1a): ask a running macOS app to quit, like ⌘Q.
/// Member processes are never signalled as a tree.
struct AppQuitTarget: Equatable {
    let name: String
    let bundlePath: String
    let pid: Int32
    let launchDate: Date
}

enum AppQuitOutcome: Equatable { case exited, stillRunning, alreadyExited, identityChanged, notSent }

/// A running application as AppKit reports it, decoupled for tests.
struct AppCandidate {
    let pid: Int32
    let bundleURL: URL?
    let launchDate: Date?
}

@MainActor enum AppQuit {
    enum Resolution: Equatable {
        case ready(AppQuitTarget)
        case unavailable(String)

        var reason: String? {
            if case let .unavailable(reason) = self { return reason }
            return nil
        }
    }

    nonisolated static let settleSeconds = 5
    private static let pollInterval: Duration = .milliseconds(100)

    static func resolve(_ group: AppGroup) -> Resolution {
        resolve(group, candidates: NSWorkspace.shared.runningApplications.map {
            AppCandidate(pid: $0.processIdentifier, bundleURL: $0.bundleURL, launchDate: $0.launchDate)
        })
    }

    /// Exactly one outer-bundle app whose PID and launch time match a member of the
    /// group, and which Rust would not refuse (protected, system, own, other user).
    static func resolve(_ group: AppGroup, candidates: [AppCandidate]) -> Resolution {
        guard let bundlePath = group.bundlePath else { return .unavailable("Not a macOS app bundle.") }
        let members = Dictionary(group.members.map { ($0.pid, $0) }, uniquingKeysWith: { first, _ in first })
        let owned = candidates.compactMap { candidate -> (AppCandidate, FfiProcessInfo)? in
            guard let url = candidate.bundleURL, url.isFileURL, url.standardizedFileURL.path == bundlePath,
                  let pid = UInt32(exactly: candidate.pid), let member = members[pid] else { return nil }
            return (candidate, member)
        }
        if let refusal = owned.lazy.compactMap(\.1.refusal).first {
            return .unavailable(ActionCopy.refusal(refusal))
        }
        let matches = owned.filter { AppCatalog.matches($0.1, launchDate: $0.0.launchDate) }.map(\.0)
        guard matches.count == 1, let app = matches.first, let launchDate = app.launchDate else {
            if matches.count > 1 { return .unavailable("Several copies of this app are running. Quit their processes individually.") }
            return .unavailable(owned.isEmpty
                ? "macOS does not list this bundle as a running app; it may be a helper or tool. Quit its processes individually."
                : "macOS did not report a matching launch time, so Tiny cannot confirm which app to quit. Quit its processes individually.")
        }
        return .ready(AppQuitTarget(name: group.name, bundlePath: bundlePath, pid: app.pid, launchDate: launchDate))
    }

    /// Sends one quit request and waits a bounded time. Never escalates to force.
    static func quit(_ target: AppQuitTarget) async -> AppQuitOutcome {
        guard let app = NSRunningApplication(processIdentifier: target.pid), !app.isTerminated else { return .alreadyExited }
        guard app.bundleURL?.standardizedFileURL.path == target.bundlePath, app.launchDate == target.launchDate else {
            return .identityChanged
        }
        guard app.terminate() else { return .notSent }
        let deadline = ContinuousClock.now + .seconds(settleSeconds)
        while !app.isTerminated {
            guard ContinuousClock.now < deadline else { return .stillRunning }
            do { try await Task.sleep(for: pollInterval) } catch { return app.isTerminated ? .exited : .stillRunning }
        }
        return .exited
    }
}
