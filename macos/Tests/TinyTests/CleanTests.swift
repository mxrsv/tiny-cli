import Foundation
import TinyEngine

/// Cleanup screen logic against a fake engine. The real `cleanExecute` is never called.
@MainActor enum CleanTests {
    static func run() async throws {
        try await selectionRules()
        try await previewConfirmExecuteRunsOnce()
        try reportMapping()
        try await errorStatesRequireRescan()
        try await previewErrorsShowAndDoNotRepeat()
        try await executeRefusalsShowInSheet()
        try await rescanOrSelectionChangeVoidsConfirmation()
        try await markerAroundExecute()
        try await progressOwnership()
        try await realDiscoveryIsReadOnlyAndScoped()
        try groupingCollapsesEmptyCategories()
        try await sensitiveItemsAreNeverSelectedByCategory()
        try evidenceCopy()
        print("PASS: 13 clean checks (selection, confirm/execute once, report, errors, preview errors, refusals, stale confirmation, marker, progress, real discovery, grouping, sensitive selection, evidence copy)")
    }

    private static func selectionRules() async throws {
        let (clean, _, _, cleanup) = make()
        defer { cleanup() }
        await clean.scan()?.value
        try check(clean.phase == .ready, "scan ready")
        try check(clean.selection == ["safe-1"], "only safe items of safe Move-to-Trash categories start selected")
        try check(!clean.selection.contains("safe-parent"), "a safe folder holding review items is not preselected (PC-C3)")
        try check(!CleanState.contains("/a/b", "/a/bc") && CleanState.contains("/a/b", "/a/b/c") && CleanState.contains("/a/b", "/a/b"),
                  "ancestor test is per path component")
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
        try await untickedChildMovesWithTickedParent()
        try await categoryCheckboxIsTriState()
    }

    /// M2: mixed state; a click clears any selection, and a safe category never adds review items.
    private static func categoryCheckboxIsTriState() async throws {
        let (clean, _, _, cleanup) = make()
        defer { cleanup() }
        await clean.scan()?.value
        let state = { (id: String) in CleanState.checkState(clean.categories.first { $0.id == id }!, selection: clean.selection) }
        try check(state("safe") == .mixed && state("review") == .off, "preselected safe category is mixed")
        clean.toggle(category: "safe")
        try check(state("safe") == .off && !clean.selection.contains("safe-1"), "clicking a category with any selection clears it")
        clean.toggle(category: "safe")
        try check(clean.selection == ["safe-1"], "a safe category adds only safe items that carry no review item")
        clean.toggle(candidate: "safe-review")
        clean.toggle(candidate: "safe-parent")
        try check(state("safe") == .on, "ticking the rest one by one makes it on")
        clean.toggle(category: "review")
        try check(state("review") == .on, "a review category selects its review items when clicked")
    }

    /// M1: a child under a ticked parent cannot be kept by unticking it; counts are deduplicated.
    private static func untickedChildMovesWithTickedParent() async throws {
        let (clean, engine, _, cleanup) = make()
        defer { cleanup() }
        await engine.useCategories([category("nested", candidates: [candidate("parent", path: "/d/p"), candidate("child", path: "/d/p/c")])])
        await clean.scan()?.value
        try check(clean.selection == ["parent", "child"] && clean.effectiveSelection.map(\.id) == ["parent"],
                  "a parent and its selected child count once")
        clean.toggle(candidate: "child")
        await clean.requestPreview()?.value
        while clean.previewLoading { await Task.yield() }
        try check(clean.coveredBy["child"] == "/d/p", "an unticked child is shown as moving with its parent")
        try check(clean.preview?.items.first?.covers.first { $0.candidateId == "child" }?.selected == false, "Rust reports it unselected")
        try check(CleanCopy.movesWith("/Users/alice/Library/Caches").hasPrefix("Moves with Caches"), "moves-with copy")
        try await siblingNamesCountOnce()
        try await blockedCopyMatchesSelectability()
    }

    /// N1: "com.x.y" sorts before "com.x/z" as a string; the child must still count once.
    private static func siblingNamesCountOnce() async throws {
        let (clean, engine, _, cleanup) = make()
        defer { cleanup() }
        await engine.useCategories([category("names", candidates: [
            candidate("x", path: "/c/com.x"), candidate("xy", path: "/c/com.x.y"), candidate("xz", path: "/c/com.x/z")])])
        await clean.scan()?.value
        try check(Set(clean.effectiveSelection.map(\.id)) == ["x", "xy"], "a child after a sibling with a longer name counts once")
    }

    /// N2: "select them too" only when the blocked items can be selected; blocked items are not counted.
    private static func blockedCopyMatchesSelectability() async throws {
        let (clean, _, _, cleanup) = make()
        defer { cleanup() }
        await clean.scan()?.value
        let selectable = FfiPreviewExclusion(candidateId: "safe-parent", path: "/p", reason: .coversUnselectedReview(candidateIds: ["review-1"]))
        let reportOnly = FfiPreviewExclusion(candidateId: "safe-parent", path: "/p", reason: .coversUnselectedReview(candidateIds: ["review-1", "report-1"]))
        try check(clean.exclusionText(selectable).contains("Select it too"), "selectable review items can be selected")
        try check(clean.exclusionText(reportOnly) == CleanCopy.unmovableInside, "report-only items cannot be selected, so keep the folder")
        clean.toggle(candidate: "safe-parent")
        try check(clean.effectiveSelection.map(\.id) == ["safe-parent"], "parent counted before a preview")
        await clean.requestPreview()?.value
        while clean.previewLoading { await Task.yield() }
        try check(clean.preview?.excluded.first?.blocksReview == true, "the fake preview blocks it like Rust")
        try check(clean.effectiveSelection.map(\.id) == ["safe-1"], "an item the preview left out is not counted")
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
                                              (.UntrustedHome(detail: "HOME does not match the account home"), true, "HOME folder")] {
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
        await engine.failDiscover(with: .UntrustedHome(detail: "HOME does not match the account home"))
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

    /// M3: preview errors keep Rust's detail, name the operation, and do not repeat.
    private static func previewErrorsShowAndDoNotRepeat() async throws {
        let (clean, engine, _, cleanup) = make()
        defer { cleanup() }
        await clean.scan()?.value
        await engine.failPreview(with: .InvalidInput(detail: "/x is report-only on the desktop"))
        await clean.requestPreview()?.value
        try check(clean.error?.message.hasPrefix("Preview failed: /x is report-only") == true, "preview error keeps Rust's detail")
        try check(!clean.canUpdatePreview && clean.requestPreview() == nil, "the same selection cannot repeat the error")
        await engine.failPreview(with: nil)
        clean.toggle(candidate: "review-1")
        try check(clean.error == nil && clean.canUpdatePreview, "changing the selection clears it and allows a new preview")
        try check(CleanCopy.error(CocoaError(.featureUnsupported), during: .preview).message.hasPrefix("Preview failed"),
                  "an unexpected preview error does not say scan")
    }

    /// M4: refusals to start show in the sheet and execute nothing.
    private static func executeRefusalsShowInSheet() async throws {
        let (clean, engine, _, cleanup) = make()
        defer { cleanup() }
        await clean.scan()?.value
        await clean.requestPreview()?.value
        guard let preview = clean.preview else { throw CheckFailure(message: "preview") }
        let quit = UUID()
        _ = clean.actions.beginTracked(quit, summary: "Quit “sleep” (PID 7)")
        try check(clean.requestMove() == nil && clean.error?.message == ActionCopy.busy.text, "requestMove checks the shared guard first")
        clean.actions.endTracked(quit)
        _ = clean.requestMove()
        _ = clean.actions.beginTracked(quit, summary: "Quit “sleep” (PID 7)")
        try check(clean.finishConfirmation(true, previewId: preview.previewId) == nil && clean.error != nil, "busy guard at execute shows in the sheet")
        clean.actions.endTracked(quit)
        _ = clean.requestMove()
        clean.confirmationUnavailable(preview.previewId)
        try check(clean.error?.message == ActionCopy.confirmationUnavailable.text && clean.pendingConfirmation == nil,
                  "no window for the confirmation is shown in the sheet")
        let executed = await engine.executed
        try check(executed.isEmpty, "nothing executed")
    }

    /// L1: a rescan or a selection change voids an open confirmation.
    private static func rescanOrSelectionChangeVoidsConfirmation() async throws {
        let (clean, engine, _, cleanup) = make()
        defer { cleanup() }
        await clean.scan()?.value
        await clean.requestPreview()?.value
        guard let first = clean.preview, clean.requestMove() != nil else { throw CheckFailure(message: "preview") }
        clean.toggle(candidate: "review-1")
        try check(clean.preview == nil && clean.pendingConfirmation == nil, "a selection change drops the preview")
        try check(clean.finishConfirmation(true, previewId: first.previewId) == nil, "selection changed between confirm and execute")
        clean.toggle(candidate: "review-1")
        await clean.requestPreview()?.value
        guard let second = clean.preview, clean.requestMove() != nil else { throw CheckFailure(message: "preview again") }
        clean.reviewing = true
        try check(clean.scan() == nil, "no rescan while the review sheet is open")
        clean.reviewing = false
        let rescan = clean.scan()
        try check(clean.preview == nil && clean.pendingConfirmation == nil, "rescanning drops the preview and the confirmation")
        try check(clean.finishConfirmation(true, previewId: second.previewId) == nil, "an old confirmation cannot run after a rescan")
        await rescan?.value
        let executed = await engine.executed
        try check(executed.isEmpty, "nothing executed")
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
        let notice = panicking.actions.notice
        try check(notice?.tone == .unknown && notice?.text.contains("Check the Trash") == true
                  && notice?.text.contains("still running") == false, "unknown outcome tells the user to check the Trash")
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
        // Independent of `CleanState.contains`: compare standardized path components.
        let components = { (path: String) in URL(fileURLWithPath: path).standardizedFileURL.pathComponents }
        let candidates = found.categories.flatMap(\.candidates)
        let reviewPaths = candidates.filter { $0.risk != .safe }.map { components($0.path) }
        for item in candidates where preselected.contains(item.id) {
            let ancestor = components(item.path)
            try check(!reviewPaths.contains { $0.starts(with: ancestor) }, "preselected \(item.path) carries no review item")
        }
    }

    // MARK: Trust evidence (UI1-UI3)

    /// UI1: three groups in order; scanned empty movable categories collapse into one line; skipped ones keep tiles.
    private static func groupingCollapsesEmptyCategories() throws {
        let item = { candidate("c-\(UUID().uuidString)") }
        let categories = [
            category("cargo", candidates: [item()], comesBack: .rebuild(command: "cargo build")),
            category("downloads", risk: .review, candidates: [item()], comesBack: .trashOnly),
            category("docker", desktop: .reportOnly(reason: .notPerPathTrash), candidates: [item()], comesBack: nil),
            category("yarn", comesBack: .redownload),
            category("mail", risk: .review, comesBack: .trashOnly),
            category("vscode", status: .appRunning(app: "Code")),
            category("broken", status: .failed(detail: "boom")),
            category("unknown", candidates: [item()], comesBack: nil),
            category("gone", candidates: [item()], comesBack: .notRecoverable)]
        let layout = CleanState.layout(categories)
        try check(layout.sections.map(\.group) == [.rebuilt, .yourFiles, .reportOnly], "groups appear in the agreed order")
        try check(layout.sections[0].categories.map(\.id) == ["cargo", "vscode", "broken"], "rebuilt group keeps skipped categories as tiles")
        try check(layout.sections[1].categories.map(\.id) == ["downloads", "unknown", "gone"],
                  "trashOnly, notRecoverable and unknown comesBack are never called rebuilt")
        try check(layout.sections[2].categories.map(\.id) == ["docker"], "report-only group")
        try check(layout.checkedEmpty.map(\.id) == ["yarn", "mail"], "scanned, movable, empty categories collapse")
        try check(CleanCopy.checkedEmpty(["Yarn cache", "Mail attachments"]) == "Checked, nothing found: Yarn cache, Mail attachments", "collapsed line copy")
        try check(CleanState.layout([category("only-empty")]).sections.isEmpty, "all-empty scan has no tiles")
        try check(CleanCopy.comesBack(.rebuild(command: "cargo build")) == "Comes back with “cargo build”"
                    && CleanCopy.comesBack(.redownload) == "Downloaded again when needed"
                    && CleanCopy.comesBack(.appRecreates) == "The app recreates it"
                    && CleanCopy.comesBack(.trashOnly) == "Only the Trash can bring these back"
                    && CleanCopy.comesBack(.notRecoverable) == "Permanent: cannot be restored", "comes-back copy")
    }

    /// Private-looking items are never selected by preselection or the tile toggle, but can be ticked one by one.
    private static func sensitiveItemsAreNeverSelectedByCategory() async throws {
        let codes = candidate("codes", evidence: [.modified(at: 0), .sensitive(reason: "recovery codes")])
        let (clean, engine, _, cleanup) = make()
        defer { cleanup() }
        await engine.useCategories([
            category("downloads", risk: .review, candidates: [candidate("a"), codes, candidate("b")], comesBack: .trashOnly),
            category("vault", risk: .review, candidates: [candidate("only-private", evidence: [.sensitive(reason: "key")])], comesBack: .trashOnly),
            category("safe-dl", candidates: [candidate("s1"), candidate("s2", evidence: [.sensitive(reason: "certificate")])], comesBack: .trashOnly)])
        await clean.scan()?.value
        let state = { (id: String) in CleanState.checkState(clean.categories.first { $0.id == id }!, selection: clean.selection) }
        try check(clean.selection == ["s1"], "a safe category never preselects a private-looking item")
        clean.toggle(category: "downloads")
        try check(clean.selection == ["s1", "a", "b"], "the tile toggle skips private-looking items")
        try check(state("downloads") == .mixed, "tile is mixed while a private item stays unticked")
        clean.toggle(candidate: "codes")
        try check(clean.selection.contains("codes") && state("downloads") == .on, "ticked one by one, the tile becomes on")
        clean.toggle(category: "downloads")
        try check(state("downloads") == .off && clean.selection == ["s1"], "a click clears everything including a ticked private item")
        clean.toggle(category: "vault")
        try check(state("vault") == .off && !clean.selection.contains("only-private"), "a category of only private items selects nothing")
        clean.toggle(category: "safe-dl")
        clean.toggle(category: "safe-dl")
        try check(clean.selection == ["s1"], "a safe tile click never adds the private-looking item")
    }

    private static func evidenceCopy() throws {
        let utc = TimeZone(identifier: "UTC")!
        let may4 = Int64(1_777_896_000) // 2026-05-04 12:00 UTC
        try check(CleanCopy.date(may4, in: utc) == "4 May 2026", "abbreviated date, no time")
        let facts: [FfiEvidence] = [.manifestModified(file: "Cargo.toml", at: may4), .lastCommit(at: may4), .workTreeClean]
        try check(CleanCopy.evidenceLine(facts, in: utc) == "Cargo.toml changed 4 May 2026 · Last commit 4 May 2026 · No uncommitted changes", "git evidence line")
        try check(CleanCopy.evidenceLine([.modified(at: may4), .lastOpened(at: may4)], in: utc) == "Modified 4 May 2026 · Last opened 4 May 2026", "download evidence line")
        try check(CleanCopy.evidenceLine([.owningApp(name: "Code")]) == "Owner: Code (not running)", "owning app")
        try check(CleanCopy.evidenceLine([.owningAppUnknown]) == "Owner app unknown", "owner unknown")
        try check(CleanCopy.evidenceLine([.venvMarker]) == "Has pyvenv.cfg" && CleanCopy.evidenceLine([.notGitRepo]) == "Not in a git repository", "marker facts")
        try check(CleanCopy.evidenceLine([.sensitive(reason: "recovery codes")]) == nil, "a sensitive fact is a warning, not part of the line")
        let private1 = candidate("p", evidence: [.modified(at: may4), .sensitive(reason: "recovery codes")])
        try check(CleanCopy.sensitiveReason(private1) == "Looks private: recovery codes" && CleanState.isSensitive(private1), "sensitive warning")
        try check(CleanCopy.sensitiveReason(candidate("q")) == nil && !CleanState.isSensitive(candidate("q")), "no warning without a sensitive fact")
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
                         status: FfiCategoryStatus = .found, candidates: [FfiCleanCandidate] = [],
                         comesBack: FfiComesBack? = .appRecreates) -> FfiCleanCategory {
        FfiCleanCategory(id: id, label: id, inclusionReason: "fixture", family: "dev", risk: risk, status: status,
                         desktopAction: desktop, candidates: candidates, totalBytes: candidates.reduce(0) { $0 + $1.sizeBytes },
                         unreadable: [], refused: [], comesBack: comesBack)
    }

    nonisolated static func candidate(_ id: String, risk: FfiRisk = .safe, size: UInt64 = 100,
                                      path: String? = nil, evidence: [FfiEvidence] = []) -> FfiCleanCandidate {
        FfiCleanCandidate(id: id, path: path ?? "/Users/alice/Library/Caches/\(id)", sizeBytes: size, unreadableEntries: 0, risk: risk, evidence: evidence)
    }
}

/// Scripted cleanup engine for checks; it never touches the filesystem.
actor FakeCleanEngine: CleanEngine {
    private(set) var executed: [String] = []
    private(set) var markersDuringExecute: [String] = []
    private var executeError: FfiError?
    private var discoverError: FfiError?
    private var previewError: FfiError?
    private var categories: [FfiCleanCategory]?
    private var panics = false
    private var marker: InFlightMarker?

    func failExecute(with error: FfiError) { executeError = error }
    func failDiscover(with error: FfiError?) { discoverError = error }
    func failPreview(with error: FfiError?) { previewError = error }
    func useCategories(_ categories: [FfiCleanCategory]) { self.categories = categories }
    func panicExecute() { panics = true }
    func observeMarker(_ marker: InFlightMarker) { self.marker = marker }

    func cleanDiscover(_ options: FfiCleanOptions, token: CancellationToken,
                       progress: any ProgressListener) async throws -> FfiDiscovery {
        if let discoverError { throw discoverError }
        progress.onProgress(progress: FfiProgress(operation: "scan", completed: 1, total: 1, message: "fixture"))
        typealias T = CleanTests
        if let categories { return FfiDiscovery(discoveryId: "d1", categories: categories) }
        return FfiDiscovery(discoveryId: "d1", categories: [
            T.category("safe", candidates: [T.candidate("safe-1"), T.candidate("safe-review", risk: .review),
                                            T.candidate("safe-parent", path: "/Users/alice/Library/Caches")]),
            T.category("review", risk: .review, candidates: [T.candidate("review-1", risk: .review), T.candidate("review-2", risk: .review)]),
            T.category("report", desktop: .reportOnly(reason: .notPerPathTrash), candidates: [T.candidate("report-1")]),
            T.category("running", status: .appRunning(app: "Xcode"))])
    }

    func cleanPreview(_ candidateIds: [String]) async throws -> FfiPreview {
        if let previewError { throw previewError }
        let all = try await cleanDiscover(CleanState.options, token: CancellationToken(), progress: ProgressRelay { _ in })
            .categories.flatMap(\.candidates)
        let selected = candidateIds.compactMap { id in all.first { $0.id == id } }
        // Like Rust: an item carrying an unselected non-safe candidate is left out (PC-C3).
        let blocked = selected.compactMap { item -> FfiPreviewExclusion? in
            let review = all.filter { $0.id != item.id && $0.risk != .safe && !candidateIds.contains($0.id)
                                      && CleanState.contains(item.path, $0.path) }.map(\.id)
            return review.isEmpty ? nil : FfiPreviewExclusion(candidateId: item.id, path: item.path,
                                                               reason: .coversUnselectedReview(candidateIds: review))
        }
        let items = selected.filter { item in !blocked.contains { $0.candidateId == item.id } }.map { item in
            FfiPreviewItem(candidateId: item.id, categoryId: "safe", path: item.path, sizeBytes: item.sizeBytes, risk: item.risk,
                           covers: all.filter { $0.id != item.id && CleanState.contains(item.path, $0.path) }.map {
                               FfiCoveredCandidate(candidateId: $0.id, path: $0.path, risk: $0.risk, selected: candidateIds.contains($0.id))
                           })
        }
        return FfiPreview(previewId: "p\(candidateIds.count)", items: items, excluded: blocked,
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
