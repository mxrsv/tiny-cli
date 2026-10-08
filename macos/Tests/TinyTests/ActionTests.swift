import AppKit
import TinyEngine

@MainActor enum ActionTests {
    static func run() async throws {
        try outcomeAndRefusalCopy()
        try await cancelSendsNothingAndForceConfirmsSeparately()
        try await busyIsReportedAndNothingQueued()
        try await markerSetClearedAndDetectedAtLaunch()
        try listenersStates()
        try appQuitResolution()
        try await appQuitStillRunningNeverEscalates()
        try await alertOrderKeysAndSingleAction()
        print("PASS: 8 action checks (copy, confirmation, busy, marker, listeners, app resolution, app quit, alert)")
    }

    private static func outcomeAndRefusalCopy() throws {
        let row = process(77, name: "node")
        let outcomes: [FfiTerminateOutcome] = [.exited, .stillRunning, .alreadyExited, .permissionDenied, .identityChanged,
                                               .refused(reason: .protected(name: "Dock"))]
        for kind in [FfiTerminateKind.graceful, .force] {
            let texts = outcomes.map { ActionCopy.notice($0, process: row, kind: kind).text }
            try check(Set(texts).count == outcomes.count, "every outcome has distinct copy")
            try check(texts.allSatisfy { $0.contains("77") }, "outcome copy names the PID")
        }
        try check(ActionCopy.notice(.exited, process: row, kind: .graceful).tone == .success, "exit is success")
        let still = ActionCopy.notice(.stillRunning, process: row, kind: .graceful)
        try check(still.tone == .warning && still.text.contains("Force Quit…"), "still running suggests, never sends, force")
        try check(ActionCopy.notice(.permissionDenied, process: row, kind: .force).tone == .failure, "denied is failure")
        let refusals: [FfiRefusal] = [.ownProcess, .ownParent, .systemProcess, .otherUser, .protected(name: "Finder"), .identityUncertain]
        try check(Set(refusals.map(ActionCopy.refusal)).count == refusals.count, "every refusal has distinct copy")
        try check(ActionCopy.refusal(.protected(name: "Finder")).contains("Finder"), "protected names the process")
        for reason in refusals {
            try check(ActionCopy.eligibility(process(50, refusal: reason)) == ActionCopy.refusal(reason), "Rust refusal blocks Quit up front")
        }
        try check(ActionCopy.eligibility(row) == nil, "own ordinary process eligible")
        let force = ActionCopy.request(row, kind: .force)
        try check(force.isDestructive && force.title.contains("node") && force.title.contains("77")
                  && force.message.contains("SIGKILL"), "force confirmation names target and consequence")
        let quit = ActionCopy.request(row, kind: .graceful)
        try check(!quit.isDestructive && quit.title.contains("“node” (PID 77)") && quit.message.contains("SIGTERM"), "quit confirmation")
        try check(ActionCopy.notice(FfiError.Operation(detail: "kill failed"), summary: "Quit x").text.contains("kill failed"), "typed error detail")
    }

    private static func cancelSendsNothingAndForceConfirmsSeparately() async throws {
        let recorder = Recorder(result: .stillRunning)
        let (actions, marker, cleanup) = makeActions(recorder)
        defer { cleanup() }
        let row = process(88)
        actions.request(row, kind: .graceful)
        try check(actions.pending?.target == .process(row, .graceful), "quit opens a confirmation, sends nothing")
        actions.cancel()
        try check(actions.pending == nil && recorder.calls.isEmpty && marker.pending.isEmpty, "cancel sends nothing")
        actions.request(row, kind: .graceful)
        guard let quit = actions.pending else { throw CheckFailure(message: "quit request pending") }
        await actions.confirm(quit)?.value
        try check(recorder.calls.map(\.1) == [.graceful] && recorder.calls[0].0 == FfiTerminateTarget(row), "one graceful signal to the confirmed target")
        try check(actions.notice?.tone == .warning && actions.pending == nil, "still running reported, nothing chained")
        try check(actions.confirm(quit) == nil && recorder.calls.count == 1, "a confirmed request cannot run twice")
        actions.request(row, kind: .force)
        guard let force = actions.pending, force.isDestructive, force.id != quit.id else {
            throw CheckFailure(message: "force needs its own destructive confirmation")
        }
        actions.cancel()
        try check(actions.confirm(force) == nil && recorder.calls.count == 1, "cancelled force sends nothing")
        actions.request(row, kind: .force)
        await actions.pending.flatMap { actions.confirm($0) }?.value
        try check(recorder.calls.map(\.1) == [.graceful, .force], "force only after its own confirmation")
    }

    private static func busyIsReportedAndNothingQueued() async throws {
        let recorder = Recorder(result: .exited, error: FfiError.Busy)
        let (actions, _, cleanup) = makeActions(recorder)
        defer { cleanup() }
        actions.request(process(5), kind: .graceful)
        await actions.pending.flatMap { actions.confirm($0) }?.value
        try check(actions.notice == ActionCopy.busy && actions.notice?.text.contains("Another operation is running") == true, "Busy copy")
        try check(recorder.calls.count == 1 && !actions.isRunning, "Busy is not retried")

        let slow = Recorder(result: .exited, delay: .milliseconds(200))
        let (second, _, cleanupSecond) = makeActions(slow)
        defer { cleanupSecond() }
        second.request(process(6), kind: .graceful)
        let task = second.pending.flatMap { second.confirm($0) }
        try check(second.isRunning, "running state set before the call")
        second.request(process(7), kind: .force)
        try check(second.pending == nil && second.notice == ActionCopy.busy, "request during an action is rejected, not queued")
        await task?.value
        try check(slow.calls.map(\.0.pid) == [6], "only the confirmed action ran")
        let app = AppQuitTarget(name: "Editor", bundlePath: "/Applications/Editor.app", pid: 40, launchDate: Date())
        second.request(process(8), kind: .graceful)
        let quitting = second.pending.flatMap { second.confirm($0) }
        second.request(app: app)
        try check(second.pending == nil && second.notice == ActionCopy.busy, "app Quit shares the running guard")
        await quitting?.value
    }

    private static func markerSetClearedAndDetectedAtLaunch() async throws {
        let recorder = Recorder(result: .exited)
        let (actions, marker, cleanup) = makeActions(recorder)
        defer { cleanup() }
        let other = UUID() // Another tracked operation, e.g. a future cleanup.
        try marker.begin(other, summary: "Move 3 items to Trash")
        recorder.onCall = { recorder.markersDuringCall = marker.pending.map(\.summary) }
        actions.request(process(9, name: "worker"), kind: .force)
        await actions.pending.flatMap { actions.confirm($0) }?.value
        try check(recorder.markersDuringCall.contains { $0.contains("worker") }, "marker durable before the call")
        try check(marker.pending.map(\.id) == [other], "a finished Quit clears only its own marker")

        let panic = Recorder(result: .exited, error: CocoaError(.featureUnsupported))
        let (crashed, panicMarker, cleanupPanic) = makeActions(panic)
        defer { cleanupPanic() }
        crashed.request(process(10, name: "worker"), kind: .graceful)
        await crashed.pending.flatMap { crashed.confirm($0) }?.value
        try check(crashed.notice?.tone == .unknown && panicMarker.pending.isEmpty, "panic is an unknown outcome, marker cleared")

        try panicMarker.begin(UUID(), summary: "Force Quit “worker” (PID 10)")
        let relaunched = ActionState(terminate: { _, _ in .exited }, quitApp: { _ in .exited }, marker: panicMarker)
        try check(relaunched.notice?.tone == .unknown && relaunched.notice?.text.contains("PID 10") == true, "marker at launch shows notice")
        try check(panicMarker.pending.count == 1, "launch does not clear before the user sees it")
        let later = UUID()
        try panicMarker.begin(later, summary: "a newer action")
        relaunched.dismissNotice()
        try check(panicMarker.pending.map(\.id) == [later] && relaunched.notice == nil, "dismiss clears only the launch markers")

        let unwritable = ActionState(terminate: recorder.terminate, quitApp: { _ in .exited },
                                     marker: InFlightMarker(directory: URL(fileURLWithPath: "/dev/null/markers")))
        let before = recorder.calls.count
        unwritable.request(process(11), kind: .graceful)
        try check(unwritable.pending.flatMap { unwritable.confirm($0) } == nil && recorder.calls.count == before
                  && unwritable.notice?.tone == .failure && !unwritable.isRunning, "no durable marker, nothing sent")
        try markerSurvivesAbort()
    }

    /// A child writes a marker and then aborts; the parent must still find it.
    private static func markerSurvivesAbort() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("tiny-abort-\(UUID().uuidString)")
        defer { try? FileManager.default.removeItem(at: directory) }
        let child = Process()
        child.executableURL = Bundle.main.executableURL
        child.arguments = [markerAbortFlag, directory.path]
        try child.run()
        child.waitUntilExit()
        try check(child.terminationReason == .uncaughtSignal && child.terminationStatus == SIGABRT, "child aborted")
        try check(InFlightMarker(directory: directory).pending.map(\.summary) == ["abort test"], "marker survives abort()")
    }

    static let markerAbortFlag = "--marker-then-abort"

    /// Child side of `markerSurvivesAbort`.
    static func markerThenAbort(_ path: String) -> Never {
        do { try InFlightMarker(directory: URL(fileURLWithPath: path)).begin(UUID(), summary: "abort test") } catch { exit(2) }
        abort()
    }

    private static func listenersStates() throws {
        let caveat = "Only listeners visible to the current user are shown."
        try check(PortsDisplay.make(listeners: nil, error: nil) == .loading, "loading before first probe")
        try check(PortsDisplay.make(listeners: FfiListeners(listeners: [], visibilityCaveat: caveat, sampledAt: 1), error: nil)
                  == .empty(caveat: caveat), "empty list keeps caveat, not 'no ports in use'")
        let row = process(30, name: "node")
        let listeners = [
            FfiListener(port: 3000, address: "127.0.0.1", pid: 30, process: row, refusal: nil, actionable: true),
            FfiListener(port: 3000, address: "::1", pid: 30, process: row, refusal: nil, actionable: true),
            FfiListener(port: 5000, address: "*", pid: 31, process: process(31), refusal: .identityUncertain, actionable: false),
            FfiListener(port: 6000, address: "*", pid: 32, process: nil, refusal: nil, actionable: false)]
        guard case let .rows(rows, shown) = PortsDisplay.make(listeners: FfiListeners(listeners: listeners, visibilityCaveat: caveat, sampledAt: 1), error: nil) else {
            throw CheckFailure(message: "rows expected")
        }
        try check(shown == caveat && rows.count == 3, "caveat shown; IPv4/IPv6 of one owner share a row")
        try check(rows[0].addresses == ["127.0.0.1", "::1"] && rows[0].blockedReason == nil, "actionable owner")
        try check(rows[1].blockedReason == ActionCopy.refusal(.identityUncertain), "IdentityUncertain not actionable")
        try check(rows[2].blockedReason?.contains("Not in the current") == true, "owner missing from sample not actionable")
        let state = AppState()
        state.applyListeners(FfiListeners(listeners: [], visibilityCaveat: caveat, sampledAt: 1))
        state.recordListenersError(FfiError.Operation(detail: "lsof timed out"))
        guard case let .failed(message) = state.ports, message.contains("lsof timed out") else {
            throw CheckFailure(message: "probe error is an error state, not an empty list")
        }
        state.stop()
    }

    private static func appQuitResolution() throws {
        let launch = Date(timeIntervalSince1970: 10.4)
        let path = "/Applications/Editor.app"
        let group = AppGroup(id: "app:\(path)", name: "Editor", bundlePath: path, members: [process(40), process(41)])
        let outer = AppCandidate(pid: 40, bundleURL: URL(fileURLWithPath: path), launchDate: launch)
        let ready = AppQuit.resolve(group, candidates: [outer])
        try check(ready == .ready(AppQuitTarget(name: "Editor", bundlePath: path, pid: 40, launchDate: launch)), "one matching app")
        try check(AppQuit.resolve(group, candidates: []).reason?.contains("does not list") == true, "no running app disables Quit")
        let relaunched = AppCandidate(pid: 40, bundleURL: URL(fileURLWithPath: path), launchDate: Date(timeIntervalSince1970: 50))
        try check(AppQuit.resolve(group, candidates: [relaunched]).reason?.contains("launch time") == true, "launch time must match the member")
        let helper = AppCandidate(pid: 41, bundleURL: URL(fileURLWithPath: path + "/Contents/Helpers/Helper.app"), launchDate: launch)
        try check(AppQuit.resolve(group, candidates: [helper]).reason != nil, "nested helper is not the app")
        let second = AppCandidate(pid: 41, bundleURL: URL(fileURLWithPath: path), launchDate: launch)
        try check(AppQuit.resolve(group, candidates: [outer, second]).reason?.contains("Several") == true, "ambiguous instances")
        let finderPath = "/System/Library/CoreServices/Finder.app"
        let finder = AppGroup(id: "app:\(finderPath)", name: "Finder", bundlePath: finderPath,
                              members: [process(60, name: "Finder", refusal: .protected(name: "Finder"))])
        let finderApp = AppCandidate(pid: 60, bundleURL: URL(fileURLWithPath: finderPath), launchDate: launch)
        try check(AppQuit.resolve(finder, candidates: [finderApp]) == .unavailable(ActionCopy.refusal(.protected(name: "Finder"))),
                  "protected app with a launch date stays disabled with Rust's reason")
        let own = AppGroup(id: "app:\(path)", name: "Tiny Dev", bundlePath: path, members: [process(40, refusal: .ownProcess)])
        try check(AppQuit.resolve(own, candidates: [outer]) == .unavailable(ActionCopy.refusal(.ownProcess)), "Tiny cannot quit itself")
        let background = AppGroup(id: "process:1", name: "tool", bundlePath: nil, members: [process(42)])
        try check(AppQuit.resolve(background, candidates: []).reason != nil, "background groups have no app Quit")
    }

    private static func appQuitStillRunningNeverEscalates() async throws {
        let recorder = Recorder(result: .exited)
        var quits: [AppQuitTarget] = []
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("tiny-checks-\(UUID().uuidString)")
        defer { try? FileManager.default.removeItem(at: directory) }
        let actions = ActionState(terminate: recorder.terminate, quitApp: { quits.append($0); return .stillRunning },
                                  marker: InFlightMarker(directory: directory))
        let target = AppQuitTarget(name: "Editor", bundlePath: "/Applications/Editor.app", pid: 40, launchDate: Date())
        actions.request(app: target)
        try check(actions.pending?.title == "Quit “Editor”?" && actions.pending?.isDestructive == false, "app quit confirmation")
        await actions.pending.flatMap { actions.confirm($0) }?.value
        try check(quits == [target] && recorder.calls.isEmpty, "one app quit request; no process signal")
        try check(actions.notice?.text.contains("save dialog") == true, "still open is reported honestly")
    }

    private static func alertOrderKeysAndSingleAction() async throws {
        let recorder = Recorder(result: .exited)
        let (actions, _, cleanup) = makeActions(recorder)
        defer { cleanup() }
        let row = process(91, name: "node")
        actions.request(row, kind: .force)
        guard let force = actions.pending else { throw CheckFailure(message: "force pending") }
        let alert = ConfirmationAlert.make(force)
        try check(alert.messageText == force.title && alert.informativeText == force.message, "alert names target and consequence")
        try check(alert.buttons.map(\.title) == ["Cancel", "Force Quit"], "Cancel first, action second")
        try check(alert.buttons.map(\.keyEquivalent) == ["\r", ""], "Return is Cancel; the action has no key equivalent")
        try check(alert.buttons[1].hasDestructiveAction && !alert.buttons[0].hasDestructiveAction, "Force Quit styled destructive")
        try check(!ConfirmationAlert.make(ActionCopy.request(row, kind: .graceful)).buttons[1].hasDestructiveAction, "Quit not destructive")
        let escape = NSEvent.keyEvent(with: .keyDown, location: .zero, modifierFlags: [], timestamp: 0, windowNumber: 0, context: nil,
                                      characters: "\u{1b}", charactersIgnoringModifiers: "\u{1b}", isARepeat: false, keyCode: 53)
        let returnKey = NSEvent.keyEvent(with: .keyDown, location: .zero, modifierFlags: [], timestamp: 0, windowNumber: 0, context: nil,
                                         characters: "\r", charactersIgnoringModifiers: "\r", isARepeat: false, keyCode: 36)
        try check(escape.map(ConfirmationAlert.isEscape) == true && returnKey.map(ConfirmationAlert.isEscape) == false, "Escape detected")
        ConfirmationAlert.finish(.alertFirstButtonReturn, request: force, actions: actions)
        try check(actions.pending == nil && recorder.calls.isEmpty, "Cancel/Escape response sends nothing")
        try check(ConfirmationAlert.finish(.alertSecondButtonReturn, request: force, actions: actions) == nil, "cancelled request cannot run later")
        actions.request(row, kind: .force)
        guard let again = actions.pending else { throw CheckFailure(message: "force pending again") }
        await ConfirmationAlert.finish(.alertSecondButtonReturn, request: again, actions: actions)?.value
        await ConfirmationAlert.finish(.alertSecondButtonReturn, request: again, actions: actions)?.value
        try check(recorder.calls.map(\.1) == [.force], "action response runs exactly once")
    }

    private static func makeActions(_ recorder: Recorder) -> (ActionState, InFlightMarker, () -> Void) {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("tiny-checks-\(UUID().uuidString)")
        let marker = InFlightMarker(directory: directory)
        let actions = ActionState(terminate: recorder.terminate, quitApp: { _ in .exited }, marker: marker)
        return (actions, marker, { try? FileManager.default.removeItem(at: directory) })
    }

    private static func process(_ pid: UInt32, name: String = "sample", refusal: FfiRefusal? = nil) -> FfiProcessInfo {
        FfiProcessInfo(pid: pid, name: name, user: "alice", isCurrentUser: refusal != .otherUser, parentPid: nil,
                       startTime: 10, cpuPercent: 1, cpuMeasured: true, memoryBytes: 1024, executablePath: nil, refusal: refusal)
    }
}

/// Fake `processTerminate`: records every call and never signals anything.
@MainActor final class Recorder {
    var calls: [(FfiTerminateTarget, FfiTerminateKind)] = []
    var markersDuringCall: [String] = []
    var onCall: () -> Void = {}
    let result: FfiTerminateOutcome
    let error: (any Error)?
    let delay: Duration?

    init(result: FfiTerminateOutcome, error: (any Error)? = nil, delay: Duration? = nil) {
        self.result = result
        self.error = error
        self.delay = delay
    }

    var terminate: ActionState.Terminate {
        { target, kind in try await self.record(target, kind) }
    }

    private func record(_ target: FfiTerminateTarget, _ kind: FfiTerminateKind) async throws -> FfiTerminateOutcome {
        calls.append((target, kind))
        onCall()
        if let delay { try await Task.sleep(for: delay) }
        if let error { throw error }
        return result
    }
}
