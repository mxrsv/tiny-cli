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
            let actions = ActionState(terminate: { _, _ in .alreadyExited }, quitApp: { _ in .alreadyExited },
                                      marker: InFlightMarker(directory: markers))
            let clean = CleanState(engine: DisplayCleanEngine(), actions: actions)
            await clean.scan()?.value
            clean.toggle(candidate: "jetbrains")
            clean.toggle(candidate: "b-ips")
            await clean.requestPreview()?.value
            try await render(CleanReviewSheet(clean: clean, actions: actions).background(Theme.tile),
                             size: NSSize(width: 680, height: 600), to: url)
            print("Rendered the review sheet with fixture data to \(url.path)")
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
        // so the sheet shows Rust's PC-C3 exclusion next to "moves with" rows.
        let all = state.clean.categories.filter(CleanState.isSelectable).flatMap(\.candidates)
        if let carrier = all.first(where: { item in
            item.risk == .safe && all.contains { $0.risk != .safe && $0.id != item.id && CleanState.contains(item.path, $0.path) }
        }) {
            state.clean.toggle(candidate: carrier.id)
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

/// Display-only data for the `review-fixture` scene: a folder that moves with its
/// parent, and a folder Rust leaves out because it holds an unselected review item.
/// It never executes anything.
private struct DisplayCleanEngine: CleanEngine {
    private static let logs = "/Users/you/Library/Logs"

    func cleanDiscover(_ options: FfiCleanOptions, token: CancellationToken,
                       progress: any ProgressListener) async throws -> FfiDiscovery {
        func item(_ id: String, _ path: String, _ size: UInt64, _ risk: FfiRisk = .safe) -> FfiCleanCandidate {
            FfiCleanCandidate(id: id, path: "\(Self.logs)/\(path)", sizeBytes: size, unreadableEntries: 0, risk: risk)
        }
        func category(_ id: String, _ label: String, _ risk: FfiRisk, _ items: [FfiCleanCandidate]) -> FfiCleanCategory {
            FfiCleanCategory(id: id, label: label, inclusionReason: "Sample", family: "user-storage", risk: risk, status: .found,
                             desktopAction: .moveToTrash, candidates: items, totalBytes: items.reduce(0) { $0 + $1.sizeBytes },
                             unreadable: [], refused: [])
        }
        return FfiDiscovery(discoveryId: "sample", categories: [
            category("user-logs", "User logs", .safe, [item("jetbrains", "JetBrains", 120_000_000),
                                                       item("diagnostic", "DiagnosticReports", 40_000_000),
                                                       item("claude", "Claude", 41_600_000)]),
            category("crash-reports", "Crash reports", .safe, [item("a-ips", "DiagnosticReports/a.ips", 20_000_000),
                                                               item("b-ips", "DiagnosticReports/b.ips", 20_000_000)]),
            category("jetbrains-logs", "JetBrains logs", .review, [item("idea", "JetBrains/IntelliJIdea2026.2", 120_000_000, .review)])])
    }

    func cleanPreview(_ candidateIds: [String]) async throws -> FfiPreview {
        let diagnostic = "\(Self.logs)/DiagnosticReports"
        func covered(_ id: String, _ name: String, _ selected: Bool) -> FfiCoveredCandidate {
            FfiCoveredCandidate(candidateId: id, path: "\(diagnostic)/\(name)", risk: .safe, selected: selected)
        }
        return FfiPreview(previewId: "sample", items: [
            FfiPreviewItem(candidateId: "claude", categoryId: "user-logs", path: "\(Self.logs)/Claude", sizeBytes: 41_600_000,
                           risk: .safe, covers: []),
            FfiPreviewItem(candidateId: "diagnostic", categoryId: "user-logs", path: diagnostic, sizeBytes: 40_000_000,
                           risk: .safe, covers: [covered("a-ips", "a.ips", true), covered("b-ips", "b.ips", false)])],
            excluded: [
                FfiPreviewExclusion(candidateId: "jetbrains", path: "\(Self.logs)/JetBrains",
                                    reason: .coversUnselectedReview(candidateIds: ["idea"])),
                FfiPreviewExclusion(candidateId: "a-ips", path: "\(diagnostic)/a.ips",
                                    reason: .insideSelected(parentCandidateId: "diagnostic"))],
            bytesSelected: 81_600_000, expiresInSeconds: 900)
    }

    func cleanExecute(_ previewId: String, token: CancellationToken,
                      progress: any ProgressListener) async throws -> FfiExecReport {
        throw FfiError.Unsupported(detail: "the snapshot engine never moves anything")
    }
}
