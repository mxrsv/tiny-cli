import SwiftUI
import TinyEngine

/// Bottom dock: the Activity ⇄ Clean switch, status and the screen's controls.
struct Dock: View {
    @Bindable var state: AppState

    var body: some View {
        HStack(spacing: 12) {
            Picker("Screen", selection: $state.screen) {
                ForEach(AppState.Screen.allCases, id: \.self) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented).labelsHidden().frame(width: 170)
            .accessibilityLabel("Switch between Activity and Clean")
            if state.screen == .clean { cleanStatus } else { activityStatus }
            Spacer()
            if let running = state.actions.running {
                ProgressView().controlSize(.small).accessibilityHidden(true)
                Text("\(running.summary)…").lineLimit(1).foregroundStyle(.secondary)
            } else {
                Text("Every action asks first").foregroundStyle(.secondary)
            }
            if state.screen == .clean { cleanControls } else { activityControls }
        }
        .font(.system(size: 12))
        .buttonStyle(.bordered)
        .padding(.horizontal, 18).padding(.vertical, 12)
        .background(.ultraThinMaterial, in: Capsule())
        .overlay(Capsule().strokeBorder(.white.opacity(0.10)))
    }

    @ViewBuilder private var activityStatus: some View {
        Circle().fill(state.status == "Live" ? Theme.accent : .orange).frame(width: 7, height: 7)
        Text(state.status).fontWeight(.semibold)
        if let sampled = state.sampledAt {
            Text("Sample \(sampled.formatted(date: .omitted, time: .standard))").foregroundStyle(.secondary)
        }
        if state.refreshing { ProgressView().controlSize(.small).accessibilityLabel("Refreshing processes") }
    }

    @ViewBuilder private var activityControls: some View {
        Button {
            state.paused.toggle()
            if !state.paused { state.requestRefresh() }
        } label: {
            Label(state.paused ? "Resume" : "Pause", systemImage: state.paused ? "play.fill" : "pause.fill")
        }.help("Pause or resume automatic refresh every 2 seconds")
        Button { state.requestRefresh(includingPorts: true) } label: { Label("Refresh", systemImage: "arrow.clockwise") }
            .keyboardShortcut("r", modifiers: .command).disabled(state.refreshing)
    }

    @ViewBuilder private var cleanStatus: some View {
        let clean = state.clean
        Circle().fill(clean.error == nil ? Theme.accent : .orange).frame(width: 7, height: 7)
        Text(cleanStatusText).fontWeight(.semibold)
    }

    private var cleanStatusText: String {
        switch state.clean.phase {
        case .idle: return "Not scanned"
        case .scanning: return "Scanning"
        case .ready: return "\(state.clean.categories.count) categories"
        case .executing: return "Moving to Trash"
        case .report: return "Report"
        }
    }

    @ViewBuilder private var cleanControls: some View {
        let clean = state.clean
        if clean.isBusy {
            Button(clean.phase == .executing ? "Stop" : "Cancel Scan") { clean.cancel() }
                .keyboardShortcut(.cancelAction)
        } else {
            Button { clean.scan() } label: {
                Label(clean.discovery == nil ? "Scan" : "Scan Again", systemImage: "magnifyingglass")
            }.keyboardShortcut("r", modifiers: .command)
        }
    }
}
