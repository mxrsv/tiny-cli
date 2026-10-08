import SwiftUI
import TinyEngine

struct ProcessesView: View {
    @Bindable var state: AppState
    var startsPolling = true

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            header
            if let notice = state.actions.notice {
                NoticeBanner(notice: notice) { state.actions.dismissNotice() }
            }
            if state.screen == .clean {
                CleanView(clean: state.clean, actions: state.actions)
            } else {
                activity
            }
            Dock(state: state)
        }
        .padding(26)
        .frame(minWidth: 1060, minHeight: 740)
        .background(Backdrop())
        .preferredColorScheme(.dark)
        .tint(Theme.accent)
        .onChange(of: state.actions.pending?.id) { _, id in
            guard id != nil, let request = state.actions.pending else { return }
            guard let window = ConfirmationAlert.hostWindow() else {
                state.actions.confirmationUnavailable()
                return
            }
            ConfirmationAlert.present(request, on: window, actions: state.actions)
        }
        .task { if startsPolling { await state.run() } }
    }

    @ViewBuilder private var activity: some View {
        HStack(spacing: Theme.gap) {
            UsageWidget(title: "CPU · SYSTEM", value: systemCPUText,
                subtitle: "All cores · 0–100%", fraction: state.systemUsage?.cpuPercent.map { Double($0) / 100 },
                icon: "cpu", tint: Theme.accent, state: state.status).frame(width: Self.widgetWidth)
            UsageWidget(title: "RAM · SYSTEM", value: systemRAMText,
                subtitle: systemRAMSubtitle, fraction: systemRAMFraction,
                icon: "memorychip", tint: .cyan, state: state.status).frame(width: Self.widgetWidth)
            PortsTile(state: state)
        }.frame(height: 176)
        HStack(alignment: .top, spacing: Theme.gap) {
            processList.frame(maxWidth: .infinity, maxHeight: .infinity)
            ProcessDetailView(state: state).frame(width: 306)
        }
    }

    private static let widgetWidth: CGFloat = 300

    private var header: some View {
        HStack(alignment: .center, spacing: 12) {
            Image(systemName: state.screen == .clean ? "sparkles" : "waveform.path.ecg")
                .font(.system(size: 24, weight: .medium)).foregroundStyle(Theme.accent)
                .frame(width: 52, height: 52)
                .background(Theme.accent.opacity(0.10), in: RoundedRectangle(cornerRadius: 16))
            VStack(alignment: .leading, spacing: 4) {
                Text(state.screen == .clean ? "Clean" : "Apps & activity").font(.system(size: 30, weight: .semibold, design: .rounded))
                Text(state.screen == .clean ? "Caches and leftovers you can move to the Trash." : "Your apps, and the work behind them.")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Text("tiny").font(.system(size: 23, weight: .bold, design: .rounded)).foregroundStyle(.secondary)
            Pill(text: "ASKS BEFORE ACTING")
        }
    }

    private var systemCPUText: String {
        state.systemUsage?.cpuPercent.map { String(format: "%.1f%%", $0) } ??
            (state.sampledAt == nil ? "—" : (state.systemUsage?.cpuWarmingUp == true ? "Warming up" : "Unavailable"))
    }
    private var systemRAMFraction: Double? {
        guard let used = state.systemUsage?.memoryUsedBytes,
              let total = state.systemUsage?.memoryTotalBytes, total > 0 else { return nil }
        return Double(used) / Double(total)
    }
    private var systemRAMText: String {
        systemRAMFraction.map { String(format: "%.1f%%", $0 * 100) } ?? "Unavailable"
    }
    private var systemRAMSubtitle: String {
        guard let used = state.systemUsage?.memoryUsedBytes,
              let total = state.systemUsage?.memoryTotalBytes else { return "System memory is unavailable" }
        let usedText = ByteCountFormatter.string(fromByteCount: Int64(clamping: used), countStyle: .memory)
        let totalText = ByteCountFormatter.string(fromByteCount: Int64(clamping: total), countStyle: .memory)
        return "\(usedText) used of \(totalText)"
    }

    private var processList: some View {
        Tile {
            VStack(alignment: .leading, spacing: 14) {
                listToolbar
                if let error = state.listError {
                    Label(error, systemImage: "exclamationmark.triangle")
                        .font(.caption).foregroundStyle(.orange).textSelection(.enabled)
                }
                table
                HStack {
                    Button {
                        state.showBackground.toggle()
                    } label: {
                        Label("System & Background · \(state.backgroundCount)",
                              systemImage: state.showBackground ? "chevron.down" : "chevron.right")
                    }.buttonStyle(.plain).foregroundStyle(.secondary)
                    Spacer()
                    Text("\(state.appCount) apps · \(state.processes.count) processes")
                }.font(.system(size: 11)).foregroundStyle(.secondary)
                Text("App CPU is per core and can exceed 100%. App RAM sums resident memory; shared pages may count twice.")
                    .font(.system(size: 10)).foregroundStyle(.secondary)
            }
        }
    }

    private var listToolbar: some View {
        HStack(spacing: 12) {
            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
                TextField("Search app, process, PID or owner", text: $state.query).textFieldStyle(.plain)
                    .accessibilityLabel("Search apps and their member processes")
                if !state.query.isEmpty {
                    Button { state.query = "" } label: { Image(systemName: "xmark.circle.fill") }
                        .buttonStyle(.plain).accessibilityLabel("Clear search")
                }
            }.padding(10).background(.white.opacity(0.045), in: RoundedRectangle(cornerRadius: 10))
            Picker("Sort", selection: $state.sort) {
                ForEach(AppState.Sort.allCases, id: \.self) { Text($0.rawValue).tag($0) }
            }.frame(width: 140)
        }
    }

    private var table: some View {
        Table(state.visibleGroups, selection: Binding(get: { state.groupSelection }, set: { state.selectGroup($0) })) {
            TableColumn("Application") { group in
                HStack(spacing: 10) {
                    Image(nsImage: state.icon(for: group)).resizable().interpolation(.high)
                        .frame(width: 28, height: 28).accessibilityHidden(true)
                    VStack(alignment: .leading, spacing: 2) {
                        Text(group.name).fontWeight(.medium).lineLimit(1)
                        if !group.isApplication { Text("Background · PID \(group.members.first?.pid ?? 0)")
                            .font(.system(size: 10)).foregroundStyle(.secondary) }
                    }
                }.padding(.vertical, 4).help(group.name)
            }.width(min: 170, ideal: 260)
            TableColumn("Processes") { Text(String($0.members.count)).monospacedDigit().foregroundStyle(.secondary) }
                .width(72)
            TableColumn("CPU") { group in
                Text(group.cpuText).monospacedDigit()
                    .help("Per-core CPU · \(group.measuredCPU.count) of \(group.members.count) processes measured")
            }.width(128)
            TableColumn("Resident RAM") { group in
                Text(group.memoryText).monospacedDigit()
                    .help("Resident memory · \(group.measuredMemory.count) of \(group.members.count) processes measured")
            }.width(152)
        }
        .contextMenu(forSelectionType: String.self) { ids in
            if let id = ids.first, let group = state.groups.first(where: { $0.id == id }) {
                ActionItems.group(group, actions: state.actions)
            }
        }
        .tableStyle(.inset(alternatesRowBackgrounds: true))
        .scrollContentBackground(.hidden)
        .overlay {
            if state.visibleGroups.isEmpty {
                ContentUnavailableView(state.sampledAt == nil ? "Reading processes…" : "No matching apps",
                    systemImage: state.sampledAt == nil ? "waveform.path.ecg" : "magnifyingglass",
                    description: Text(state.query.isEmpty ? "Refresh to read the current process list." : "Try another app, process, PID or owner."))
            }
        }
    }
}
