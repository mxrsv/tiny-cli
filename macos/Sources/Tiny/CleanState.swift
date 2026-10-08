import Foundation
import Observation
import TinyEngine

/// The cleanup calls the screen needs; `Engine` in the app, a fake in checks.
protocol CleanEngine: Sendable {
    func cleanDiscover(_ options: FfiCleanOptions, token: CancellationToken,
                       progress: any ProgressListener) async throws -> FfiDiscovery
    func cleanPreview(_ candidateIds: [String]) async throws -> FfiPreview
    func cleanExecute(_ previewId: String, token: CancellationToken,
                      progress: any ProgressListener) async throws -> FfiExecReport
}

extension Engine: CleanEngine {}

/// Rust calls back on a background thread; `deliver` hops to the main actor.
final class ProgressRelay: ProgressListener {
    private let deliver: @Sendable (FfiProgress) -> Void
    init(_ deliver: @escaping @Sendable (FfiProgress) -> Void) { self.deliver = deliver }
    func onProgress(progress: FfiProgress) { deliver(progress) }
}

/// Scan → select → review/preview → confirm → Move to Trash → per-item report.
/// Nothing is replayed: an invalid, expired or busy preview requires a new scan.
@MainActor @Observable
final class CleanState {
    enum Phase: Equatable { case idle, scanning, ready, executing, report }
    enum Operation: Equatable { case scan, preview, execute }

    /// Same scope as the CLI default: safe and review, never destructive.
    static let options = FfiCleanOptions(categories: [], includeReview: true, includeDestructive: false, idleDays: 30)

    private(set) var phase: Phase = .idle
    private(set) var discovery: FfiDiscovery?
    private(set) var selection: Set<String> = []
    private(set) var progress: FfiProgress?
    private(set) var error: CleanError?
    private(set) var preview: FfiPreview?
    private(set) var previewExpiresAt: Date?
    private(set) var previewLoading = false
    /// Selection whose preview failed; asking again would only repeat the error.
    private(set) var previewFailedFor: [String]?
    private(set) var report: FfiExecReport?
    /// Preview ID shown in the open confirmation; it can be confirmed once.
    private(set) var pendingConfirmation: String?
    var reviewing = false
    /// Progress is accepted only from the call that started this operation.
    private(set) var operation: UUID?
    @ObservationIgnored private var token: CancellationToken?
    @ObservationIgnored private let engine: any CleanEngine
    @ObservationIgnored private let actions: ActionState
    @ObservationIgnored private let now: () -> Date

    init(engine: any CleanEngine, actions: ActionState, now: @escaping () -> Date = Date.init) {
        self.engine = engine
        self.actions = actions
        self.now = now
    }

    var categories: [FfiCleanCategory] { discovery?.categories ?? [] }
    var isBusy: Bool { phase == .scanning || phase == .executing }
    var selectedCandidates: [FfiCleanCandidate] {
        categories.flatMap(\.candidates).filter { selection.contains($0.id) }
    }
    /// Selected items with duplicates and paths inside another selected item
    /// removed, so overlaps are counted once (as Rust's preview does).
    var effectiveSelection: [FfiCleanCandidate] {
        var kept: [FfiCleanCandidate] = []
        for candidate in selectedCandidates.sorted(by: { $0.path < $1.path }) {
            if let last = kept.last, Self.contains(last.path, candidate.path) { continue }
            kept.append(candidate)
        }
        return kept
    }
    var selectedBytes: UInt64 { effectiveSelection.reduce(0) { $0.saturatingAdd($1.sizeBytes) } }
    /// Candidate ID → path of the preview item that moves it along.
    var coveredBy: [String: String] {
        var map: [String: String] = [:]
        for item in preview?.items ?? [] {
            for covered in item.covers { map[covered.candidateId] = item.path }
        }
        return map
    }
    var canReview: Bool { phase == .ready && !selection.isEmpty && error?.needsRescan != true }
    var canUpdatePreview: Bool { canReview && !previewLoading && previewFailedFor != selection.sorted() }

    static func isSelectable(_ category: FfiCleanCategory) -> Bool {
        category.desktopAction == .moveToTrash && category.status == .found
            && category.risk != .destructive && !category.candidates.isEmpty
    }

    /// Only safe Move-to-Trash categories start selected, and only safe items that
    /// would not carry a review item along (the same path or an ancestor of one).
    static func preselected(_ discovery: FfiDiscovery) -> Set<String> {
        let review = discovery.categories.flatMap(\.candidates).filter { $0.risk != .safe }.map(\.path)
        return Set(discovery.categories.filter { isSelectable($0) && $0.risk == .safe }
            .flatMap(\.candidates)
            .filter { candidate in candidate.risk == .safe && !review.contains { contains(candidate.path, $0) } }
            .map(\.id))
    }

    /// Whether moving `ancestor` moves `path`: the same path or one inside it.
    nonisolated static func contains(_ ancestor: String, _ path: String) -> Bool {
        path == ancestor || path.hasPrefix(ancestor.hasSuffix("/") ? ancestor : ancestor + "/")
    }

    // MARK: Scan

    @discardableResult
    func scan() -> Task<Void, Never>? {
        guard !isBusy else { return nil }
        let id = start(.scanning)
        discovery = nil
        selection = []
        report = nil
        guard let token else { return nil }
        let relay = relay(for: id)
        return Task {
            do {
                let found = try await engine.cleanDiscover(Self.options, token: token, progress: relay)
                guard operation == id else { return }
                discovery = found
                selection = Self.preselected(found)
                finish(.ready)
            } catch {
                guard operation == id else { return }
                self.error = CleanCopy.error(error, during: .scan)
                finish(.idle)
            }
        }
    }

    func cancel() { token?.cancel() }

    // MARK: Selection

    enum CheckState: Equatable { case off, mixed, on }

    static func checkState(_ category: FfiCleanCategory, selection: Set<String>) -> CheckState {
        let selected = category.candidates.filter { selection.contains($0.id) }.count
        return selected == 0 ? .off : selected == category.candidates.count ? .on : .mixed
    }

    /// Any selected item clears the category. Otherwise a safe category adds only
    /// its safe items that carry no review item; review items are ticked one by
    /// one, or by clicking a review category.
    func toggle(category id: String) {
        guard phase == .ready, let category = categories.first(where: { $0.id == id }), Self.isSelectable(category) else { return }
        let ids = Set(category.candidates.map(\.id))
        if !ids.isDisjoint(with: selection) {
            selection.subtract(ids)
        } else if category.risk == .safe, let discovery {
            selection.formUnion(Self.preselected(discovery).intersection(ids))
        } else {
            selection.formUnion(ids)
        }
        invalidatePreview()
    }

    func toggle(candidate id: String) {
        guard phase == .ready, categories.contains(where: { Self.isSelectable($0) && $0.candidates.contains { $0.id == id } }) else { return }
        if selection.remove(id) == nil { selection.insert(id) }
        invalidatePreview()
    }

    // MARK: Review and preview

    func openReview() {
        guard canReview else { return }
        reviewing = true
        requestPreview()
    }

    @discardableResult
    func requestPreview() -> Task<Void, Never>? {
        guard canUpdatePreview else { return nil }
        let ids = selection.sorted()
        previewLoading = true
        previewFailedFor = nil
        return Task {
            defer { previewLoading = false }
            do {
                let result = try await engine.cleanPreview(ids)
                guard selection.sorted() == ids, phase == .ready else { return }
                preview = result
                previewExpiresAt = now().addingTimeInterval(TimeInterval(result.expiresInSeconds))
                if error?.needsRescan == false { error = nil }
            } catch {
                previewFailedFor = ids
                self.error = CleanCopy.error(error, during: .preview)
            }
        }
    }

    /// Opens the final confirmation for the current preview, or explains why not.
    func requestMove() -> ConfirmationCopy? {
        guard let preview, !preview.items.isEmpty, !isBusy else { return nil }
        if let expires = previewExpiresAt, now() >= expires {
            requireRescan(CleanCopy.expired)
            return nil
        }
        pendingConfirmation = preview.previewId
        return CleanCopy.confirmation(preview)
    }

    /// Maps the confirmation once: only a confirmed, still-pending preview executes.
    @discardableResult
    func finishConfirmation(_ confirmed: Bool, previewId: String) -> Task<Void, Never>? {
        guard pendingConfirmation == previewId else { return nil }
        pendingConfirmation = nil
        guard confirmed, let preview, preview.previewId == previewId else { return nil }
        if let expires = previewExpiresAt, now() >= expires {
            requireRescan(CleanCopy.expired)
            return nil
        }
        return execute(preview)
    }

    // MARK: Execute

    private func execute(_ preview: FfiPreview) -> Task<Void, Never>? {
        let summary = "Move \(CleanCopy.items(preview.items.count)) (\(CleanCopy.bytes(preview.bytesSelected))) to Trash"
        let marker = UUID()
        guard actions.beginTracked(marker, summary: summary) else { return nil }
        let id = start(.executing)
        reviewing = false
        // The preview is consumed by this call whatever happens; never offer it again.
        self.preview = nil
        previewExpiresAt = nil
        guard let token else { actions.endTracked(marker); return nil }
        let relay = relay(for: id)
        return Task {
            defer { actions.endTracked(marker) }
            do {
                report = try await engine.cleanExecute(preview.previewId, token: token, progress: relay)
                finish(.report)
            } catch let error as FfiError {
                requireRescan(CleanCopy.error(error, during: .execute))
            } catch {
                // A Rust panic or bridge failure: items may or may not have moved.
                actions.post(ActionCopy.unknownOutcome(summary))
                requireRescan(CleanError(message: "Tiny could not confirm what was moved. Check the Trash before scanning again.", needsRescan: true))
            }
        }
    }

    // MARK: Progress

    func receive(_ update: FfiProgress, for id: UUID) {
        guard operation == id else { return }
        if let current = progress, current.operation == update.operation, update.completed < current.completed { return }
        progress = update
    }

    private func relay(for id: UUID) -> ProgressRelay {
        ProgressRelay { update in Task { @MainActor [weak self] in self?.receive(update, for: id) } }
    }

    private func start(_ next: Phase) -> UUID {
        let id = UUID()
        operation = id
        token = CancellationToken()
        progress = nil
        error = nil
        phase = next
        return id
    }

    private func finish(_ next: Phase) {
        phase = next
        operation = nil
        token = nil
        progress = nil
    }

    /// The selection changed: drop the preview, and any error a new preview may fix.
    private func invalidatePreview() {
        preview = nil
        previewExpiresAt = nil
        if error?.needsRescan == false { error = nil }
    }

    private func requireRescan(_ error: CleanError) {
        invalidatePreview()
        self.error = error
        reviewing = false
        finish(discovery == nil ? .idle : .ready)
    }
}

private extension UInt64 {
    func saturatingAdd(_ other: UInt64) -> UInt64 {
        let (sum, overflow) = addingReportingOverflow(other)
        return overflow ? .max : sum
    }
}
