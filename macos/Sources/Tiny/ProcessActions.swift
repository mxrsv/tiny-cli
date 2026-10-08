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

/// The one tracked mutation running now: a Quit, Force Quit, app Quit or cleanup.
struct RunningAction: Equatable {
    let id: UUID
    let summary: String
}

struct ActionNotice: Equatable {
    enum Tone { case success, info, warning, failure, unknown }
    let tone: Tone
    let text: String
}

/// User-facing copy for every action, outcome and refusal. Rust decides; Swift explains.
enum ActionCopy {
    static let busy = ActionNotice(tone: .warning,
        text: "Another operation is running. Nothing was sent; try again when it finishes.")

    static let confirmationUnavailable = ActionNotice(tone: .failure,
        text: "Couldn't show the confirmation; nothing was sent.")

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

    /// Rust's refusal from the last sample; `processTerminate` still re-checks it.
    static func eligibility(_ process: FfiProcessInfo) -> String? { process.refusal.map(refusal) }

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

    static func interrupted(_ summaries: [String]) -> ActionNotice {
        ActionNotice(tone: .unknown, text: "Tiny stopped during: \(summaries.joined(separator: "; ")). Its result is unknown. Check the target before trying again; nothing is repeated automatically.")
    }

    static func markerFailed(_ summary: String, _ error: any Error) -> ActionNotice {
        ActionNotice(tone: .failure, text: "\(summary) was not started: Tiny could not record it safely. \(error.localizedDescription)")
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
        case .UntrustedHome(let detail): return "Tiny's HOME folder cannot be trusted (\(detail))."
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
    private(set) var running: RunningAction?
    private(set) var notice: ActionNotice?
    @ObservationIgnored var onFinish: @MainActor () -> Void = {}
    @ObservationIgnored private let terminate: Terminate
    @ObservationIgnored private let quitApp: QuitApp
    @ObservationIgnored private let marker: InFlightMarker
    /// Markers found at launch; dismissing the notice clears only these.
    @ObservationIgnored private var interrupted: [UUID] = []

    init(terminate: @escaping Terminate, quitApp: @escaping QuitApp, marker: InFlightMarker) {
        self.terminate = terminate
        self.quitApp = quitApp
        self.marker = marker
        let leftover = marker.pending
        if !leftover.isEmpty {
            interrupted = leftover.map(\.id)
            notice = ActionCopy.interrupted(leftover.map(\.summary))
        }
    }

    var isRunning: Bool { running != nil }

    func request(_ process: FfiProcessInfo, kind: FfiTerminateKind) {
        open(ActionCopy.request(process, kind: kind))
    }

    func request(app: AppQuitTarget) { open(ActionCopy.request(app: app)) }

    func cancel() { pending = nil }

    /// The confirmation sheet could not be shown, so the request is dropped visibly.
    func confirmationUnavailable() {
        pending = nil
        notice = ActionCopy.confirmationUnavailable
    }

    func dismissNotice() {
        notice = nil
        interrupted.forEach(marker.clear)
        interrupted = []
    }

    /// Only the request currently shown can run, once. App Quit and process
    /// actions share `running`. The marker is durable before the call starts, so
    /// a crash mid-call is detected at next launch; without it nothing is sent.
    @discardableResult
    func confirm(_ request: ActionRequest) -> Task<Void, Never>? {
        guard pending?.id == request.id else { return nil }
        pending = nil
        guard running == nil else { notice = ActionCopy.busy; return nil }
        do { try marker.begin(request.id, summary: request.summary) } catch {
            notice = ActionCopy.markerFailed(request.summary, error)
            return nil
        }
        running = RunningAction(id: request.id, summary: request.summary)
        return Task { await perform(request) }
    }

    /// Claims the shared guard for a mutation owned elsewhere (cleanup), with the
    /// same durable marker. Returns false, with a notice, when it must not start.
    func beginTracked(_ id: UUID, summary: String) -> Bool {
        guard running == nil else { notice = ActionCopy.busy; return false }
        do { try marker.begin(id, summary: summary) } catch {
            notice = ActionCopy.markerFailed(summary, error)
            return false
        }
        running = RunningAction(id: id, summary: summary)
        return true
    }

    /// Releases a guard taken by `beginTracked`, on any return.
    func endTracked(_ id: UUID) {
        guard running?.id == id else { return }
        marker.clear(id)
        running = nil
    }

    func post(_ notice: ActionNotice) { self.notice = notice }

    private func open(_ request: ActionRequest) {
        guard running == nil else { notice = ActionCopy.busy; return }
        pending = request
    }

    private func perform(_ request: ActionRequest) async {
        defer {
            marker.clear(request.id)
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
