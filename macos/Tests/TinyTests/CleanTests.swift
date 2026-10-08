import Foundation
import TinyEngine

/// Cleanup screen logic against a fake engine. The real `cleanExecute` is never called.
@MainActor enum CleanTests {
    static func run() async throws {
        try await selectionRules()
        try await previewConfirmExecuteRunsOnce()
        try reportMapping()
        try await errorStatesRequireRescan()
        try await markerAroundExecute()
        try await progressOwnership()
        try await realDiscoveryIsReadOnlyAndScoped()
        print("PASS: 7 clean checks (selection, confirm/execute once, report, errors, marker, progress, real discovery)")
    }

    private static func selectionRules() async throws {
        let (clean, _, _, cleanup) = make()
        defer { cleanup() }
        await clean.scan()?.value
        try check(clean.phase == .ready, "scan ready")
        try check(clean.selection == ["safe-1"], "only safe items of safe Move-to-Trash categories start selected")
        clean.toggle(category: "report")
        clean.toggle(candidate: "report-1")
        try check(!clean.selection.contains("report-1"), "report-only cannot be selected")
        clean.toggle(category: "running")
        try check(clean.selection == ["safe-1"], "a category whose app is running cannot be selected")
        clean.toggle(category: "review")
        try check(clean.selection.isSuperset(of: ["review-1", "review-2"]), "review items need an explicit selection")
        clean.toggle(candidate: "review-2")
        try check(!clean.selection.contains("review-2"), "per-path exclusion")
        try check(!CleanState.isSelectable(category("report", desktop: .reportOnly(reason: .notPerPathTrash))), "report-only rule")
    }

    private static func previewConfirmExecuteRunsOnce() async throws {
        let (clean, engine, _, cleanup) = make()
        defer { cleanup() }
        await clean.scan()?.value
        clean.openReview()
        try check(clean.reviewing, "review sheet opens")
        await clean.requestPreview()?.value
        while clean.previewLoading { await Task.yield() }
        guard let preview = clean.preview, let copy = clean.requestMove() else { throw CheckFailure(message: "preview and confirmation") }
        try check(copy.title == "Move 1 item (\(CleanCopy.bytes(100))) to Trash?" && copy.message.contains("Empty Trash automatically"),
                  "confirmation names count, size and the Trash consequence")
        try check(ConfirmationAlert.make(copy).buttons.map(\.title) == ["Cancel", "Move to Trash"], "Cancel is the default button")
        try check(clean.finishConfirmation(false, previewId: preview.previewId) == nil, "cancel sends nothing")
        try check(clean.finishConfirmation(true, previewId: preview.previewId) == nil, "a cancelled confirmation cannot execute later")
        _ = clean.requestMove()
        await clean.finishConfirmation(true, previewId: preview.previewId)?.value
        try check(clean.finishConfirmation(true, previewId: preview.previewId) == nil, "a confirmed preview cannot run again")
        let executed = await engine.executed
        try check(executed == [preview.previewId], "execute runs exactly once with the preview ID")
        try check(clean.phase == .report && clean.preview == nil && !clean.reviewing, "report shown; preview consumed")
    }

    private static func reportMapping() throws {
        let report = FakeCleanEngine.everyOutcome(stopped: .automationDenied)
        let sections = ReportSections(report)
        try check(sections.moved.count == 1 && sections.failed.count == 2 && sections.skipped.count == 2
                  && sections.notAttempted.count == 1, "outcomes split into sections")
        try check(CleanCopy.summary(report).hasPrefix("Moved to Trash: ") && !CleanCopy.summary(report).lowercased().contains("freed"),
                  "moved bytes never called freed")
        let guidance = CleanCopy.stopped(.automationDenied) ?? ""
        try check(guidance.contains("Privacy & Security → Automation") && guidance.contains("nothing was deleted"), "Automation guidance")
        try check(CleanCopy.stopped(.cancelled)?.contains("not attempted") == true && CleanCopy.stopped(nil) == nil, "stopped copy")
        let texts = report.results.map { CleanCopy.outcome($0.outcome) }
        try check(texts[1].contains("still in place") && texts[2].contains("check the Trash"), "failure says where the item is")
        try check(texts[3].contains("changed since the scan") && texts[4].contains("(Xcode)"), "skip reasons and details")
        try check(texts[5].contains("Not attempted"), "not attempted copy")
        try check(CleanCopy.error(FfiError.Cancelled, during: .execute).message.hasPrefix("Cleanup failed")
                  && CleanCopy.error(FfiError.Cancelled, during: .execute).needsRescan, "execute never reports a scan cancellation")
    }

    private static func errorStatesRequireRescan() async throws {
        for (error, needsRescan, phrase) in [(FfiError.Busy, true, "Another operation"),
                                              (.PreviewInvalid(detail: "preview already used"), true, "no longer valid"),
                                              (.Operation(detail: "HOME does not match the account home"), true, "HOME folder")] {
            let (clean, engine, _, cleanup) = make()
            defer { cleanup() }
            await clean.scan()?.value
            await engine.failExecute(with: error)
            await clean.requestPreview()?.value
            while clean.previewLoading { await Task.yield() }
            guard let preview = clean.preview, clean.requestMove() != nil else { throw CheckFailure(message: "preview") }
            await clean.finishConfirmation(true, previewId: preview.previewId)?.value
            try check(clean.error?.needsRescan == needsRescan && clean.error?.message.contains(phrase) == true, "error copy: \(phrase)")
            try check(!clean.canReview && clean.preview == nil, "review disabled until a new scan")
        }
        let (clean, engine, _, cleanup) = make()
        defer { cleanup() }
        await engine.failDiscover(with: .Operation(detail: "HOME does not match the account home"))
        await clean.scan()?.value
        try check(clean.phase == .idle && clean.error?.message.contains("HOME folder") == true, "HOME mismatch on scan")
        await engine.failDiscover(with: nil)
        await clean.scan()?.value
        try check(clean.phase == .ready && clean.error == nil, "a new scan clears the error")
        var clock = Date()
        let (expiring, _, _, cleanupExpiring) = make(now: { clock })
        defer { cleanupExpiring() }
        await expiring.scan()?.value
        await expiring.requestPreview()?.value
        while expiring.previewLoading { await Task.yield() }
        clock = clock.addingTimeInterval(16 * 60)
        try check(expiring.requestMove() == nil && expiring.error == CleanCopy.expired, "an expired preview requires a rescan")
    }

    private static func markerAroundExecute() async throws {
        let (clean, engine, marker, cleanup) = make()
        defer { cleanup() }
        await clean.scan()?.value
        await engine.observeMarker(marker)
        await clean.requestPreview()?.value
        while clean.previewLoading { await Task.yield() }
        guard let preview = clean.preview, clean.requestMove() != nil else { throw CheckFailure(message: "preview") }
        await clean.finishConfirmation(true, previewId: preview.previewId)?.value
        let during = await engine.markersDuringExecute
        try check(during.contains { $0.contains("to Trash") }, "marker durable before execute")
        try check(marker.pending.isEmpty, "marker cleared after execute")

        let (panicking, failing, panicMarker, cleanupPanic) = make()
        defer { cleanupPanic() }
        await panicking.scan()?.value
        await failing.panicExecute()
        await panicking.requestPreview()?.value
        while panicking.previewLoading { await Task.yield() }
        guard let second = panicking.preview, panicking.requestMove() != nil else { throw CheckFailure(message: "preview") }
        await panicking.finishConfirmation(true, previewId: second.previewId)?.value
        try check(panicking.error?.needsRescan == true && panicMarker.pending.isEmpty, "non-FfiError is an unknown outcome; marker cleared")
    }

    private static func progressOwnership() async throws {
        let (clean, _, _, cleanup) = make()
        defer { cleanup() }
        let task = clean.scan()
        guard let current = clean.operation else { throw CheckFailure(message: "operation id") }
        clean.receive(FfiProgress(operation: "scan", completed: 1, total: 4, message: "old"), for: UUID())
        try check(clean.progress == nil, "progress from another call is ignored")
        clean.receive(FfiProgress(operation: "scan", completed: 2, total: 4, message: "now"), for: current)
        clean.receive(FfiProgress(operation: "scan", completed: 1, total: 4, message: "late"), for: current)
        try check(clean.progress?.message == "now", "progress belongs to its call and never goes backwards")
        await task?.value
        clean.receive(FfiProgress(operation: "scan", completed: 3, total: 4, message: "after"), for: current)
        try check(clean.progress == nil, "a finished call cannot report progress")
    }

    /// Read-only: the real discovery with the CLI's default scope.
    private static func realDiscoveryIsReadOnlyAndScoped() async throws {
        try check(CleanState.options.includeReview && !CleanState.options.includeDestructive && CleanState.options.idleDays == 30,
                  "discovery scope matches the CLI default")
        let found = try await Engine().cleanDiscover(CleanState.options, token: CancellationToken(), progress: ProgressRelay { _ in })
        try check(!found.categories.isEmpty, "real discovery returns categories")
        try check(found.categories.allSatisfy { $0.risk != .destructive }, "no destructive categories")
        let preselected = CleanState.preselected(found)
        let review = Set(found.categories.flatMap(\.candidates).filter { $0.risk != .safe }.map(\.id))
        try check(preselected.isDisjoint(with: review), "no review candidate is preselected in real data")
    }

    // MARK: Fixtures

    static func make(now: @escaping () -> Date = Date.init) -> (CleanState, FakeCleanEngine, InFlightMarker, () -> Void) {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("tiny-clean-\(UUID().uuidString)")
        let marker = InFlightMarker(directory: directory)
        let actions = ActionState(terminate: { _, _ in .exited }, quitApp: { _ in .exited }, marker: marker)
        let engine = FakeCleanEngine()
        return (CleanState(engine: engine, actions: actions, now: now), engine, marker,
                { try? FileManager.default.removeItem(at: directory) })
    }

    nonisolated static func category(_ id: String, risk: FfiRisk = .safe, desktop: FfiDesktopAction = .moveToTrash,
                         status: FfiCategoryStatus = .found, candidates: [FfiCleanCandidate] = []) -> FfiCleanCategory {
        FfiCleanCategory(id: id, label: id, inclusionReason: "fixture", family: "dev", risk: risk, status: status,
                         desktopAction: desktop, candidates: candidates, totalBytes: candidates.reduce(0) { $0 + $1.sizeBytes },
                         unreadable: [], refused: [])
    }

    nonisolated static func candidate(_ id: String, risk: FfiRisk = .safe, size: UInt64 = 100) -> FfiCleanCandidate {
        FfiCleanCandidate(id: id, path: "/Users/alice/Library/Caches/\(id)", sizeBytes: size, unreadableEntries: 0, risk: risk)
    }
}

/// Scripted cleanup engine for checks; it never touches the filesystem.
actor FakeCleanEngine: CleanEngine {
    private(set) var executed: [String] = []
    private(set) var markersDuringExecute: [String] = []
    private var executeError: FfiError?
    private var discoverError: FfiError?
    private var panics = false
    private var marker: InFlightMarker?

    func failExecute(with error: FfiError) { executeError = error }
    func failDiscover(with error: FfiError?) { discoverError = error }
    func panicExecute() { panics = true }
    func observeMarker(_ marker: InFlightMarker) { self.marker = marker }

    func cleanDiscover(_ options: FfiCleanOptions, token: CancellationToken,
                       progress: any ProgressListener) async throws -> FfiDiscovery {
        if let discoverError { throw discoverError }
        progress.onProgress(progress: FfiProgress(operation: "scan", completed: 1, total: 1, message: "fixture"))
        typealias T = CleanTests
        return FfiDiscovery(discoveryId: "d1", categories: [
            T.category("safe", candidates: [T.candidate("safe-1"), T.candidate("safe-review", risk: .review)]),
            T.category("review", risk: .review, candidates: [T.candidate("review-1", risk: .review), T.candidate("review-2", risk: .review)]),
            T.category("report", desktop: .reportOnly(reason: .notPerPathTrash), candidates: [T.candidate("report-1")]),
            T.category("running", status: .appRunning(app: "Xcode"))])
    }

    func cleanPreview(_ candidateIds: [String]) async throws -> FfiPreview {
        let items = candidateIds.map { FfiPreviewItem(candidateId: $0, categoryId: "safe", path: "/tmp/\($0)", sizeBytes: 100, risk: .safe) }
        return FfiPreview(previewId: "p\(candidateIds.count)", items: items, excluded: [],
                          bytesSelected: UInt64(items.count) * 100, expiresInSeconds: 900)
    }

    func cleanExecute(_ previewId: String, token: CancellationToken,
                      progress: any ProgressListener) async throws -> FfiExecReport {
        executed.append(previewId)
        markersDuringExecute = marker?.pending.map(\.summary) ?? []
        if panics { throw CocoaError(.featureUnsupported) }
        if let executeError { throw executeError }
        return Self.everyOutcome(stopped: nil)
    }

    nonisolated static func everyOutcome(stopped: FfiStopReason?) -> FfiExecReport {
        let outcomes: [FfiItemOutcome] = [
            .movedToTrash,
            .failed(reason: .trashFailed, detail: "Finder error -8013", sourceStillPresent: true),
            .failed(reason: .automationDenied, detail: "not authorized", sourceStillPresent: false),
            .skipped(reason: .changed, detail: nil),
            .skipped(reason: .appRunning, detail: "Xcode"),
            .notAttempted]
        let results = outcomes.enumerated().map { index, outcome in
            FfiItemResult(candidateId: "c\(index)", categoryId: "xcode-derived-data",
                          path: "/Users/alice/Library/Developer/Xcode/DerivedData/Project-\(index)", sizeBytes: 1_000_000, outcome: outcome)
        }
        return FfiExecReport(results: results, bytesSelected: 6_000_000, bytesMovedToTrash: 1_000_000, movedCount: 1,
                             failedCount: 2, skippedCount: 2, notAttemptedCount: 1, stopped: stopped)
    }
}
