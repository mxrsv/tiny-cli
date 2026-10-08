import Foundation
import Observation
import TinyEngine

/// What the user is asked to confirm. Targets are copied when the dialog opens,
/// so a refresh cannot change what gets signalled.
struct ActionRequest: Identifiable, Equatable {
    enum Target: Equatable {
        case process(FfiProcessInfo, FfiTerminateKind)
        case app(AppQuitTarget)
    }
    let id = UUID()
    let target: Target
    let title: String
    let message: String
    let confirmLabel: String
    let isDestructive: Bool
    /// Stored in the in-flight marker and shown if the outcome is unknown.
    let summary: String
}

struct ActionNotice: Equatable {
    enum Tone { case success, info, warning, failure, unknown }
    let tone: Tone
    let text: String
}

/// Persisted before each mutation and cleared on any return. One left behind at
/// launch means Tiny stopped mid-action, so the outcome is unknown.
struct InFlightMarker {
    static let key = "inFlightAction"
    let defaults: UserDefaults
    var pending: String? { defaults.string(forKey: Self.key) }
    func begin(_ summary: String) { defaults.set(summary, forKey: Self.key) }
    func clear() { defaults.removeObject(forKey: Self.key) }
}

/// User-facing copy for every action, outcome and refusal. Rust decides; Swift explains.
enum ActionCopy {
    static let busy = ActionNotice(tone: .warning,
        text: "Another operation is running. Nothing was sent; try again when it finishes.")

    static func label(_ process: FfiProcessInfo) -> String { "“\(process.name)” (PID \(process.pid))" }

    static func request(_ process: FfiProcessInfo, kind: FfiTerminateKind) -> ActionRequest {
        let target = label(process)
        switch kind {
        case .graceful:
            return ActionRequest(target: .process(process, kind), title: "Quit \(target)?",
                message: "Tiny asks it to quit (SIGTERM). It can save and exit, or ignore the request. Unsaved work in this process may be lost.",
                confirmLabel: "Quit", isDestructive: false, summary: "Quit \(target)")
        case .force:
            return ActionRequest(target: .process(process, kind), title: "Force Quit \(target)?",
                message: "The process is stopped immediately (SIGKILL) and cannot save. Any unsaved work in it is lost.",
                confirmLabel: "Force Quit", isDestructive: true, summary: "Force Quit \(target)")
        }
    }

    static func request(app: AppQuitTarget) -> ActionRequest {
        ActionRequest(target: .app(app), title: "Quit “\(app.name)”?",
            message: "\(app.name) is asked to quit, like choosing Quit from its menu. It may ask you to save documents first. Tiny never force quits it automatically.",
            confirmLabel: "Quit", isDestructive: false, summary: "Quit app “\(app.name)”")
    }

    static func refusal(_ reason: FfiRefusal) -> String {
        switch reason {
        case .ownProcess: return "This is Tiny itself."
        case .ownParent: return "This process launched Tiny."
        case .systemProcess: return "This is a core system process (PID 0 or 1)."
        case .otherUser: return "It belongs to another user. Tiny only acts on your own processes."
        case .protected(let name): return "\(name) keeps your login session running, so Tiny protects it."
        case .identityUncertain: return "It started after the port lookup, so it may not own this port. Refresh to check again."
        }
    }

    /// What Swift already knows before asking Rust. Protected names stay a Rust decision.
    static func eligibility(_ process: FfiProcessInfo,
                            ownPID: UInt32 = UInt32(ProcessInfo.processInfo.processIdentifier),
                            parentPID: UInt32 = UInt32(getppid())) -> String? {
        if process.pid <= 1 { return refusal(.systemProcess) }
        if process.pid == ownPID { return refusal(.ownProcess) }
        if process.pid == parentPID { return refusal(.ownParent) }
        if !process.isCurrentUser { return refusal(.otherUser) }
        return nil
    }

    static func notice(_ outcome: FfiTerminateOutcome, process: FfiProcessInfo, kind: FfiTerminateKind) -> ActionNotice {
        let target = label(process)
        let verb = kind == .force ? "Force Quit" : "Quit"
        switch outcome {
        case .exited:
            return ActionNotice(tone: .success, text: "\(verb): \(target) exited.")
        case .stillRunning where kind == .graceful:
            return ActionNotice(tone: .warning, text: "\(target) is still running 2 seconds after Quit. It may be saving or ignoring the request. Use Force Quit… only if losing unsaved work is acceptable.")
        case .stillRunning:
            return ActionNotice(tone: .warning, text: "\(target) is still running after Force Quit. Refresh to check it again.")
        case .alreadyExited:
            return ActionNotice(tone: .info, text: "\(target) had already exited. Nothing was sent.")
        case .permissionDenied:
            return ActionNotice(tone: .failure, text: "macOS denied \(verb) for \(target). Tiny does not request elevated rights.")
        case .identityChanged:
            return ActionNotice(tone: .warning, text: "PID \(process.pid) now belongs to a different process, so nothing was sent. Select the current process and try again.")
        case .refused(let reason):
            return ActionNotice(tone: .failure, text: "Tiny did not \(verb.lowercased()) \(target). \(refusal(reason))")
        }
    }

    static func notice(_ outcome: AppQuitOutcome, app: AppQuitTarget) -> ActionNotice {
        let name = "“\(app.name)”"
        switch outcome {
        case .exited:
            return ActionNotice(tone: .success, text: "\(name) quit.")
        case .stillRunning:
            return ActionNotice(tone: .warning, text: "\(name) is still open after \(AppQuit.settleSeconds) seconds. It may be showing a save dialog or declined to quit. Tiny never force quits an app automatically.")
        case .alreadyExited:
            return ActionNotice(tone: .info, text: "\(name) had already quit. Nothing was sent.")
        case .identityChanged:
            return ActionNotice(tone: .warning, text: "\(name) was relaunched or replaced, so nothing was sent. Select it again and retry.")
        case .notSent:
            return ActionNotice(tone: .failure, text: "macOS did not accept the quit request for \(name). Nothing was sent.")
        }
    }

    /// Typed adapter errors: the call failed before or instead of signalling.
    static func notice(_ error: FfiError, summary: String) -> ActionNotice {
        if case .Busy = error { return busy }
        return ActionNotice(tone: .failure, text: "\(summary) failed. \(describe(error))")
    }

    static func unknownOutcome(_ summary: String) -> ActionNotice {
        ActionNotice(tone: .unknown, text: "Tiny could not confirm the result of: \(summary). Check whether the target is still running before trying again.")
    }

    static func interrupted(_ summary: String) -> ActionNotice {
        ActionNotice(tone: .unknown, text: "Tiny stopped during: \(summary). Its result is unknown. Check the target before trying again; nothing is repeated automatically.")
    }

    static func describe(_ error: any Error) -> String {
        guard let error = error as? FfiError else { return error.localizedDescription }
        switch error {
        case .Busy: return busy.text
        case .Cancelled: return "The operation was cancelled."
        case .InvalidInput(let detail): return "Tiny rejected the request: \(detail)"
        case .Operation(let detail), .Io(let detail): return detail
        case .Unsupported(let detail): return "Not supported: \(detail)"
        case .AutomationDenied(let detail): return "macOS denied Automation access. \(detail)"
        case .PreviewInvalid(let detail): return "The preview is no longer valid. \(detail)"
        }
    }
}

/// Confirmation and execution of Quit / Force Quit. Nothing is queued: a request
/// while another action runs is rejected with the busy notice.
@MainActor @Observable
final class ActionState {
    typealias Terminate = @Sendable (FfiTerminateTarget, FfiTerminateKind) async throws -> FfiTerminateOutcome
    typealias QuitApp = @MainActor (AppQuitTarget) async -> AppQuitOutcome

    private(set) var pending: ActionRequest?
    private(set) var running: ActionRequest?
    private(set) var notice: ActionNotice?
    @ObservationIgnored var onFinish: @MainActor () -> Void = {}
    @ObservationIgnored private let terminate: Terminate
    @ObservationIgnored private let quitApp: QuitApp
    @ObservationIgnored private let marker: InFlightMarker

    init(terminate: @escaping Terminate, quitApp: @escaping QuitApp, marker: InFlightMarker) {
        self.terminate = terminate
        self.quitApp = quitApp
        self.marker = marker
        if let interrupted = marker.pending { notice = ActionCopy.interrupted(interrupted) }
    }

    var isRunning: Bool { running != nil }

    func request(_ process: FfiProcessInfo, kind: FfiTerminateKind) {
        open(ActionCopy.request(process, kind: kind))
    }

    func request(app: AppQuitTarget) { open(ActionCopy.request(app: app)) }

    func cancel() { pending = nil }

    func dismissNotice() {
        notice = nil
        if running == nil { marker.clear() }
    }

    /// Only the request currently shown can run, once. The marker is written
    /// before the call starts so a crash mid-call is detected at next launch.
    @discardableResult
    func confirm(_ request: ActionRequest) -> Task<Void, Never>? {
        guard pending?.id == request.id else { return nil }
        pending = nil
        guard running == nil else { notice = ActionCopy.busy; return nil }
        running = request
        marker.begin(request.summary)
        return Task { await perform(request) }
    }

    private func open(_ request: ActionRequest) {
        guard running == nil else { notice = ActionCopy.busy; return }
        pending = request
    }

    private func perform(_ request: ActionRequest) async {
        defer {
            marker.clear()
            running = nil
            onFinish()
        }
        switch request.target {
        case let .process(process, kind):
            do {
                notice = ActionCopy.notice(try await terminate(FfiTerminateTarget(process), kind), process: process, kind: kind)
            } catch let error as FfiError {
                notice = ActionCopy.notice(error, summary: request.summary)
            } catch {
                // A Rust panic or bridge failure: the signal may or may not have been sent.
                notice = ActionCopy.unknownOutcome(request.summary)
            }
        case let .app(app):
            notice = ActionCopy.notice(await quitApp(app), app: app)
        }
    }
}
