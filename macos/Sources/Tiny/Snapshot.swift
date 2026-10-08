import AppKit
import SwiftUI
import TinyEngine

/// In-process rendering of the app's own views with real data, without Screen
/// Recording. Scenes only select, scan (read-only) or preview; nothing is
/// confirmed or executed, and the marker directory is throwaway.
extension Main {
    enum SnapshotScene: String { case member, app, notice, minimum, clean, cleanAll = "clean-all", review, report }

    @MainActor static func snapshot(to url: URL, scene: SnapshotScene) async throws {
        NSApplication.shared.setActivationPolicy(.prohibited)
        let markers = FileManager.default.temporaryDirectory.appendingPathComponent("tiny-snapshot-\(UUID().uuidString)")
        defer { try? FileManager.default.removeItem(at: markers) }
        if scene == .notice { try InFlightMarker(directory: markers).begin(UUID(), summary: "Force Quit “sleep” (PID 4242)") }
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
