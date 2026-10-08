import AppKit
import Observation
import TinyEngine

@MainActor @Observable
final class AppState {
    enum Sort: String, CaseIterable {
        case cpu = "CPU", memory = "Memory", name = "Name"
    }
    var query = ""
    var sort: Sort = .cpu
    var paused = false
    var showBackground = false
    private(set) var groups: [AppGroup] = []
    private(set) var groupSelection: String?
    private(set) var systemUsage: FfiSystemUsage?
    private(set) var processes: [FfiProcessInfo] = []
    private(set) var sampledAt: Date?
    private(set) var listError: String?
    private(set) var selection: ProcessIdentity?
    private(set) var detail: FfiProcessDetail?
    private(set) var detailError: String?
    private(set) var detailSampledAt: Date?
    private(set) var refreshing = false
    private(set) var detailLoading = false
    private(set) var listeners: FfiListeners?
    private(set) var listenersError: String?
    let actions: ActionState
    @ObservationIgnored private let catalog = AppCatalog()
    @ObservationIgnored private let engine: Engine
    @ObservationIgnored private var generation = 0
    @ObservationIgnored private var pendingRefresh = false
    @ObservationIgnored private var refreshTask: Task<Void, Never>?

    init(engine: Engine = Engine(), markerDirectory: URL = InFlightMarker.defaultDirectory) {
        self.engine = engine
        actions = ActionState(terminate: { try await engine.terminate($0, kind: $1) },
                              quitApp: { await AppQuit.quit($0) }, marker: InFlightMarker(directory: markerDirectory))
        actions.onFinish = { [weak self] in self?.requestRefresh() }
    }

    var selectedGroup: AppGroup? { groups.first { $0.id == groupSelection } }
    var appCount: Int { groups.filter(\.isApplication).count }
    var backgroundCount: Int { groups.filter { !$0.isApplication }.count }
    var visibleGroups: [AppGroup] {
        let search = query.trimmingCharacters(in: .whitespacesAndNewlines)
        let matches = groups.filter { $0.matches(search) }
        let apps = Self.sortedGroups(matches.filter(\.isApplication), sort: sort)
        let background = Self.sortedGroups(matches.filter { !$0.isApplication }, sort: sort)
        return apps + (showBackground || !search.isEmpty ? background : [])
    }
    func icon(for group: AppGroup) -> NSImage { catalog.icon(for: group) }
    var ports: PortsDisplay { PortsDisplay.make(listeners: listeners, error: listenersError) }

    static func sortedGroups(_ groups: [AppGroup], sort: Sort) -> [AppGroup] {
        groups.sorted { a, b in
            switch sort {
            case .cpu:
                if a.cpuPercent != b.cpuPercent { return (a.cpuPercent ?? -1) > (b.cpuPercent ?? -1) }
            case .memory:
                switch (a.memoryBytes, b.memoryBytes) {
                case let (.some(a), .some(b)) where a != b: return a > b
                case (.some, .none): return true
                case (.none, .some): return false
                default: break
                }
            case .name:
                let order = a.name.localizedCaseInsensitiveCompare(b.name)
                if order != .orderedSame { return order == .orderedAscending }
            }
            return a.id < b.id
        }
    }
    var selectedProcess: FfiProcessInfo? { selectedGroup?.members.first { $0.identity == selection } }
    var status: String {
        if listError != nil { return sampledAt == nil ? "Unavailable" : "Stale" }
        if paused { return "Paused" }
        return sampledAt == nil ? "Connecting" : "Live"
    }

    func selectGroup(_ id: String?) {
        guard id != groupSelection else { return }
        groupSelection = id
        selection = nil
        generation += 1
        detail = nil
        detailError = nil
        detailSampledAt = nil
        detailLoading = false
    }

    func select(_ identity: ProcessIdentity?) {
        guard identity != selection else { return }
        selection = identity
        if let identity, let group = groups.first(where: { $0.members.contains { $0.identity == identity } }) {
            groupSelection = group.id
        }
        generation += 1
        detail = nil
        detailError = nil
        detailSampledAt = nil
        detailLoading = identity != nil
        requestRefresh()
    }

    func requestRefresh() {
        guard !refreshing else { pendingRefresh = true; return }
        refreshing = true
        refreshTask = Task { [weak self] in await self?.refresh() }
    }

    func run() async {
        requestRefresh()
        do {
            while !Task.isCancelled {
                try await Task.sleep(for: .seconds(2))
                if !paused && !refreshing { requestRefresh() }
            }
        } catch is CancellationError {
            stop()
        } catch {
            listError = error.localizedDescription
            stop()
        }
    }

    func stop() {
        detailLoading = false
        generation += 1
        pendingRefresh = false
        refreshTask?.cancel()
    }

    func waitForRefresh() async { await refreshTask?.value }

    private func refresh() async {
        defer {
            refreshing = false
            refreshTask = nil
            if pendingRefresh && !Task.isCancelled {
                pendingRefresh = false
                requestRefresh()
            }
        }
        do {
            let result = try await engine.list()
            try Task.checkCancellation()
            apply(result)
        } catch is CancellationError { return }
        catch { recordListError(error) }
        if !Task.isCancelled, let selection { await refreshDetail(selection, generation: generation) }
        else { detailLoading = false }
        guard !Task.isCancelled else { return }
        do {
            let result = try await engine.listeners()
            try Task.checkCancellation()
            applyListeners(result)
        } catch is CancellationError { return }
        catch { recordListenersError(error) }
    }

    func applyListeners(_ result: FfiListeners) {
        listeners = result
        listenersError = nil
    }

    /// A failed probe is an error state, never an empty list.
    func recordListenersError(_ error: any Error) {
        listeners = nil
        listenersError = "Cannot read listening ports. \(ActionCopy.describe(error))"
    }

    func recordListError(_ error: any Error) {
        listError = "Cannot refresh processes. \(error.localizedDescription)"
    }

    func apply(_ snapshot: FfiProcessSnapshot) {
        processes = snapshot.processes
        systemUsage = snapshot.systemUsage
        groups = AppGroups.make(processes, catalog: catalog.descriptors(for: processes))
        catalog.retainIcons(for: groups)
        sampledAt = Date(timeIntervalSince1970: TimeInterval(snapshot.sampledAt))
        listError = nil
        if let groupSelection, !groups.contains(where: { $0.id == groupSelection }) {
            generation += 1
            selection = nil
            detail = nil
            detailSampledAt = nil
            detailLoading = false
            detailError = "This app is no longer in the current sample. Select a running app."
        }
        if selection == nil, selectedGroup != nil { detailError = nil }
        if let selection, selectedGroup?.members.contains(where: { $0.identity == selection }) != true {
            generation += 1
            self.selection = nil
            detail = nil
            detailSampledAt = nil
            detailLoading = false
            detailError = "This process exited, changed app, or its PID was reused. Select a current member."
        }
    }

    private func refreshDetail(_ identity: ProcessIdentity, generation request: Int) async {
        guard processes.contains(where: { $0.identity == identity }),
              selectedGroup?.members.contains(where: { $0.identity == identity }) == true else { return }
        detailLoading = true
        do {
            let result = try await engine.detail(identity)
            try Task.checkCancellation()
            acceptDetail(result, for: identity, generation: request)
        } catch is CancellationError { return }
        catch {
            guard selection == identity && generation == request else { return }
            // Preserve last-good detail only for the same identity, visibly marked stale.
            detailError = error.localizedDescription
            detailLoading = false
        }
    }

    func acceptDetail(_ result: FfiProcessDetail, for identity: ProcessIdentity, generation request: Int) {
        guard generation == request, selection == identity, result.process.identity == identity,
              processes.contains(where: { $0.identity == identity }),
              selectedGroup?.members.contains(where: { $0.identity == identity }) == true else { return }
        detail = result
        detailSampledAt = Date()
        detailError = nil
        detailLoading = false
    }
}
