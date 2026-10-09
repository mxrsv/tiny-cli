import AppKit
import SwiftUI
import TinyEngine

/// In-process rendering of the app's own views with real data, without Screen
/// Recording. Scenes only select, scan (read-only) or preview; nothing is
/// confirmed or executed, and the marker directory is throwaway.
extension Main {
    enum SnapshotScene: String { case member, app, notice, minimum, clean, cleanAll = "clean-all", review, reviewFixture = "review-fixture", report }

    @MainActor static func snapshot(to url: URL, scene: SnapshotScene) async throws {
        NSApplication.shared.setActivationPolicy(.prohibited)
        let markers = FileManager.default.temporaryDirectory.appendingPathComponent("tiny-snapshot-\(UUID().uuidString)")
        defer { try? FileManager.default.removeItem(at: markers) }
        if scene == .notice { try InFlightMarker(directory: markers).begin(UUID(), summary: "Force Quit “sleep” (PID 4242)") }
        if scene == .reviewFixture {
            try await renderFixtureReview(markers: markers, to: url)
            return
        }
        if scene == .report {
            try await render(CleanReportView(report: sampleReport()).padding(26).background(Backdrop()),
                             size: NSSize(width: 1188, height: 1180), to: url)
            print("Rendered a sample report with every outcome to \(url.path)")
            return
        }
        let child = Process()
        child.executableURL = URL(fileURLWithPath: "/bin/sleep")
        child.arguments = ["60"]
        try child.run()
        defer { if child.isRunning { child.terminate(); child.waitUntilExit() } }
        let state = AppState(markerDirectory: markers)
        state.requestRefresh()
        await state.waitForRefresh()
        guard state.listError == nil, !state.processes.isEmpty else {
            throw NSError(domain: "Tiny", code: 4, userInfo: [NSLocalizedDescriptionKey: state.listError ?? "No process data"])
        }
        if scene == .clean || scene == .cleanAll || scene == .review {
            try await renderClean(state, scene: scene, to: url)
            return
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
        try await render(ProcessesView(state: state, startsPolling: false), size: size, to: url)
        print("Rendered \(state.processes.count) live processes to \(url.path)")
    }

    /// Fixture data (`DisplayCleanEngine`) through the real review sheet: nothing is scanned or moved.
    @MainActor private static func renderFixtureReview(markers: URL, to url: URL) async throws {
        let actions = ActionState(terminate: { _, _ in .alreadyExited }, quitApp: { _ in .alreadyExited },
                                  marker: InFlightMarker(directory: markers))
        let clean = CleanState(engine: DisplayCleanEngine(), actions: actions)
        await clean.scan()?.value
        clean.toggle(candidate: "jetbrains")
        clean.toggle(candidate: "b-ips")
        for id in ["rust-a", "node-a", "venv-a", "cache-old", "dl-installer", "dl-recovery"] { clean.toggle(candidate: id) }
        await clean.requestPreview()?.value
        try await render(CleanReviewSheet(clean: clean, actions: actions).background(Theme.tile),
                         size: NSSize(width: 680, height: 600), to: url)
        print("Rendered the review sheet with fixture data to \(url.path)")
    }

    /// Real, read-only discovery; the review scene also asks Rust for a real preview.
    @MainActor private static func renderClean(_ state: AppState, scene: SnapshotScene, to url: URL) async throws {
        state.screen = .clean
        await state.clean.scan()?.value
        guard state.clean.phase == .ready else {
            throw NSError(domain: "Tiny", code: 10, userInfo: [NSLocalizedDescriptionKey: state.clean.error?.message ?? "Scan failed"])
        }
        guard scene == .review else {
            // `clean-all` is tall enough to show every tile, including report-only ones.
            let height: CGFloat = scene == .cleanAll ? 2400 : 850
            try await render(ProcessesView(state: state, startsPolling: false), size: NSSize(width: 1240, height: height), to: url)
            print("Rendered \(state.clean.categories.count) cleanup categories to \(url.path)")
            return
        }
        // Stage, never run: tick a safe folder that holds a review item, if real data has one,
        // so the sheet shows Rust's PC-C3 exclusion next to "moves with" rows, plus the first
        // item of each movable category so its evidence shows.
        let selectable = state.clean.categories.filter(CleanState.isSelectable)
        let all = selectable.flatMap(\.candidates)
        if let carrier = all.first(where: { item in
            item.risk == .safe && all.contains { $0.risk != .safe && $0.id != item.id && CleanState.contains(item.path, $0.path) }
        }) {
            state.clean.toggle(candidate: carrier.id)
        }
        for first in selectable.compactMap(\.candidates.first) where !state.clean.selection.contains(first.id) {
            state.clean.toggle(candidate: first.id)
        }
        await state.clean.requestPreview()?.value
        try await render(CleanReviewSheet(clean: state.clean, actions: state.actions).background(Theme.tile),
                         size: NSSize(width: 680, height: 600), to: url)
        print("Rendered a preview of \(state.clean.preview?.items.count ?? 0) items to \(url.path)")
    }

    @MainActor private static func render(_ view: some View, size: NSSize, to url: URL) async throws {
        let host = NSHostingView(rootView: view.frame(width: size.width, height: size.height).preferredColorScheme(.dark))
        let window = NSWindow(contentRect: NSRect(origin: .zero, size: size), styleMask: [.borderless],
                              backing: .buffered, defer: false)
        window.appearance = NSAppearance(named: .darkAqua)
        window.contentView = host
        host.layoutSubtreeIfNeeded()
        try await Task.sleep(for: .seconds(1))
        try withExtendedLifetime(window) {
            host.layoutSubtreeIfNeeded()
            guard let bitmap = host.bitmapImageRepForCachingDisplay(in: host.bounds) else { throw CocoaError(.fileWriteUnknown) }
            host.cacheDisplay(in: host.bounds, to: bitmap)
            guard let png = bitmap.representation(using: .png, properties: [:]) else { throw CocoaError(.fileWriteUnknown) }
            try png.write(to: url, options: .atomic)
        }
    }

    /// Display-only data covering every outcome; no cleanup call produces it.
    private static func sampleReport() -> FfiExecReport {
        let outcomes: [FfiItemOutcome] = [
            .movedToTrash, .movedToTrash,
            .failed(reason: .trashFailed, detail: "Finder error -8013", sourceStillPresent: true),
            .failed(reason: .automationDenied, detail: "Not authorized to send Apple events to Finder.", sourceStillPresent: true),
            .skipped(reason: .changed, detail: nil),
            .skipped(reason: .appRunning, detail: "Xcode"),
            .skipped(reason: .notOnHomeVolume, detail: nil),
            .notAttempted]
        let results = outcomes.enumerated().map { index, outcome in
            FfiItemResult(candidateId: "c\(index)", categoryId: "xcode-derived-data",
                          path: "/Users/you/Library/Developer/Xcode/DerivedData/Project-\(index)-abcdef",
                          sizeBytes: UInt64(index + 1) * 120_000_000, outcome: outcome)
        }
        return FfiExecReport(results: results, bytesSelected: 4_320_000_000, bytesMovedToTrash: 360_000_000, movedCount: 2,
                             failedCount: 2, skippedCount: 3, notAttemptedCount: 1, stopped: .automationDenied)
    }
}

/// Display-only data for the `clean`, `review` and `review-fixture` scenes: every group, the
/// collapsed empty line, several evidence kinds, a private-looking file, a folder that moves
/// with its parent and a folder Rust leaves out because it holds an unselected review item.
/// It never executes anything.
private struct DisplayCleanEngine: CleanEngine {
    private static let home = "/Users/you"
    private static let logs = "\(home)/Library/Logs"

    /// 12:00 UTC, so the abbreviated date reads the same in every time zone.
    private static func day(_ year: Int, _ month: Int, _ day: Int) -> Int64 {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: "UTC")!
        return Int64(calendar.date(from: DateComponents(year: year, month: month, day: day, hour: 12))!.timeIntervalSince1970)
    }

    private static func item(_ id: String, _ path: String, _ size: UInt64, _ risk: FfiRisk = .safe,
                             _ evidence: [FfiEvidence]) -> FfiCleanCandidate {
        FfiCleanCandidate(id: id, path: path, sizeBytes: size, unreadableEntries: 0, risk: risk, evidence: evidence)
    }

    private static func category(_ id: String, _ label: String, _ reason: String, _ risk: FfiRisk, _ comesBack: FfiComesBack,
                                _ items: [FfiCleanCandidate], status: FfiCategoryStatus = .found,
                                desktop: FfiDesktopAction = .moveToTrash) -> FfiCleanCategory {
        FfiCleanCategory(id: id, label: label, inclusionReason: reason, family: "dev", risk: risk, status: status,
                         desktopAction: desktop, candidates: items, totalBytes: items.reduce(0) { $0 + $1.sizeBytes },
                         unreadable: [], refused: [], comesBack: comesBack)
    }

    private static func categories() -> [FfiCleanCategory] {
        let seen: [FfiEvidence] = [.modified(at: day(2026, 9, 30))]
        let log = { (id: String, path: String, size: UInt64, risk: FfiRisk) in item(id, "\(logs)/\(path)", size, risk, seen) }
        let dev = "\(home)/Developer", downloads = "\(home)/Downloads", caches = "\(home)/Library/Caches"
        return [
            category("user-logs", "User logs", "Sample", .safe, .appRecreates, [
                log("jetbrains", "JetBrains", 120_000_000, .safe), log("diagnostic", "DiagnosticReports", 40_000_000, .safe),
                log("claude", "Claude", 41_600_000, .safe)]),
            category("crash-reports", "Crash reports", "Sample", .safe, .appRecreates, [
                log("a-ips", "DiagnosticReports/a.ips", 20_000_000, .safe), log("b-ips", "DiagnosticReports/b.ips", 20_000_000, .safe)]),
            category("jetbrains-logs", "JetBrains logs", "Sample", .review, .appRecreates, [
                log("idea", "JetBrains/IntelliJIdea2026.2", 120_000_000, .review)]),
            category("rust-targets", "Rust build output", "Cargo target folders in projects idle for 30+ days.", .safe,
                     .rebuild(command: "cargo build"), [
                        item("rust-a", "\(dev)/old-parser/target", 2_100_000_000, .safe,
                             [.manifestModified(file: "Cargo.toml", at: day(2026, 3, 2)), .lastCommit(at: day(2026, 3, 9)), .workTreeClean]),
                        item("rust-b", "\(dev)/scratch-cli/target", 640_000_000, .safe,
                             [.manifestModified(file: "Cargo.toml", at: day(2026, 2, 11)), .notGitRepo])]),
            category("node-modules", "node_modules", "Installed packages in projects idle for 30+ days.", .safe,
                     .rebuild(command: "npm install"), [
                        item("node-a", "\(dev)/landing-v1/node_modules", 880_000_000, .safe,
                             [.manifestModified(file: "package.json", at: day(2026, 4, 18)), .lastCommit(at: day(2026, 4, 20)), .workTreeClean])]),
            category("python-venvs", "Python virtualenvs", "Virtualenvs in projects idle for 30+ days.", .review,
                     .rebuild(command: "python -m venv .venv"), [
                        item("venv-a", "\(dev)/ml-notes/.venv", 1_400_000_000, .review,
                             [.manifestModified(file: "requirements.txt", at: day(2026, 1, 14)), .venvMarker, .notGitRepo])]),
            category("user-caches", "App caches", "Per-app cache folders in ~/Library/Caches.", .safe, .appRecreates, [
                item("cache-code", "\(caches)/com.microsoft.VSCode", 310_000_000, .safe, seen + [.owningApp(name: "Code")]),
                item("cache-old", "\(caches)/com.example.OldTool", 54_000_000, .safe, seen + [.owningAppUnknown])]),
            category("vscode", "VS Code caches", "Editor caches that VS Code recreates.", .safe, .appRecreates, [],
                     status: .appRunning(app: "Code")),
            category("yarn-cache", "Yarn cache", "Yarn's package download cache.", .safe, .redownload, []),
            category("downloads-old", "Old Downloads", "Files in Downloads not modified or opened for 30+ days.", .review, .trashOnly, [
                item("dl-installer", "\(downloads)/Xcode_26.xip", 3_200_000_000, .review,
                     [.modified(at: day(2026, 4, 9)), .lastOpened(at: day(2026, 5, 2))]),
                item("dl-invoice", "\(downloads)/invoice-march.pdf", 480_000, .review, [.modified(at: day(2026, 3, 31))]),
                item("dl-recovery", "\(downloads)/recovery-codes.txt", 4_096, .review,
                     [.modified(at: day(2026, 6, 12)), .sensitive(reason: "recovery or backup codes")]),
                item("dl-env", "\(downloads)/env.txt", 1_024, .review,
                     [.modified(at: day(2026, 7, 3)), .sensitive(reason: "environment file with secrets")])]),
            category("mail-attachments", "Mail attachments", "Attachments Mail saved to disk.", .review, .trashOnly, []),
            category("docker", "Docker data", "Docker images and volumes.", .review, .notRecoverable, [
                item("docker-a", "\(home)/Library/Containers/com.docker.docker/Data/vms", 12_000_000_000, .review, seen)],
                     desktop: .reportOnly(reason: .notPerPathTrash))]
    }

    func cleanDiscover(_ options: FfiCleanOptions, token: CancellationToken,
                       progress: any ProgressListener) async throws -> FfiDiscovery {
        FfiDiscovery(discoveryId: "sample", categories: Self.categories())
    }

    /// Mirrors Rust's preview rules for the fixture: a path inside another selected one moves
    /// with it, and a folder holding an unselected review item is left out.
    func cleanPreview(_ candidateIds: [String]) async throws -> FfiPreview {
        let all = Self.categories().flatMap { category in category.candidates.map { (category.id, $0) } }
        let selected = all.filter { candidateIds.contains($0.1.id) }
        var items: [FfiPreviewItem] = [], excluded: [FfiPreviewExclusion] = []
        for (categoryId, candidate) in selected.sorted(by: { $0.1.path < $1.1.path }) {
            if let parent = selected.first(where: { $0.1.id != candidate.id && CleanState.contains($0.1.path, candidate.path) && $0.1.path != candidate.path }) {
                excluded.append(FfiPreviewExclusion(candidateId: candidate.id, path: candidate.path,
                                                    reason: .insideSelected(parentCandidateId: parent.1.id)))
                continue
            }
            let inside = all.filter { $0.1.id != candidate.id && CleanState.contains(candidate.path, $0.1.path) }
            let unselectedReview = inside.filter { $0.1.risk != .safe && !candidateIds.contains($0.1.id) }
            if !unselectedReview.isEmpty {
                excluded.append(FfiPreviewExclusion(candidateId: candidate.id, path: candidate.path,
                                                    reason: .coversUnselectedReview(candidateIds: unselectedReview.map(\.1.id))))
                continue
            }
            items.append(FfiPreviewItem(candidateId: candidate.id, categoryId: categoryId, path: candidate.path,
                                        sizeBytes: candidate.sizeBytes, risk: candidate.risk,
                                        covers: inside.map { FfiCoveredCandidate(candidateId: $0.1.id, path: $0.1.path, risk: $0.1.risk,
                                                                                selected: candidateIds.contains($0.1.id)) }))
        }
        return FfiPreview(previewId: "sample", items: items, excluded: excluded,
                          bytesSelected: items.reduce(0) { $0 + $1.sizeBytes }, expiresInSeconds: 900)
    }

    func cleanExecute(_ previewId: String, token: CancellationToken,
                      progress: any ProgressListener) async throws -> FfiExecReport {
        throw FfiError.Unsupported(detail: "the snapshot engine never moves anything")
    }
}
