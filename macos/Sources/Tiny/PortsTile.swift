import SwiftUI
import TinyEngine

/// Current user's visible TCP listeners. The caveat is always shown with the list.
struct PortsTile: View {
    let state: AppState

    var body: some View {
        Tile {
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Label("PORTS · LISTENING", systemImage: "network").font(.system(size: 11, weight: .semibold))
                        .tracking(1).foregroundStyle(.secondary)
                    Spacer()
                    if case let .rows(rows, _) = state.ports {
                        Text("\(rows.count) visible").font(.system(size: 10, weight: .medium)).foregroundStyle(.secondary)
                    }
                }
                content
            }
        }
    }

    @ViewBuilder private var content: some View {
        switch state.ports {
        case .loading:
            Text("Reading listening ports…").font(.callout).foregroundStyle(.secondary)
            Spacer(minLength: 0)
        case .failed(let message):
            Label(message, systemImage: "exclamationmark.triangle").font(.caption).foregroundStyle(.orange)
                .textSelection(.enabled).fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 0)
        case .empty(let caveat):
            Text("No visible listeners").font(.callout.weight(.medium))
            Spacer(minLength: 0)
            caveatText(caveat)
        case let .rows(rows, caveat):
            ScrollView {
                LazyVStack(spacing: 3) { ForEach(rows) { row(for: $0) } }.padding(.bottom, 10)
            }
            // Fade the bottom edge so a partly visible row reads as "scroll for more".
            .mask {
                VStack(spacing: 0) {
                    Color.black
                    LinearGradient(colors: [.black, .clear], startPoint: .top, endPoint: .bottom).frame(height: 18)
                }
            }
            caveatText(caveat)
        }
    }

    private func row(for row: PortRow) -> some View {
        HStack(spacing: 10) {
            Text(":\(String(row.port))").font(.system(size: 13, weight: .semibold, design: .monospaced))
                .frame(width: 58, alignment: .leading)
            VStack(alignment: .leading, spacing: 1) {
                Text("\(row.ownerName) · PID \(String(row.pid))").font(.caption).lineLimit(1)
                Text(row.blockedReason ?? row.addresses.joined(separator: ", "))
                    .font(.system(size: 10)).foregroundStyle(.secondary).lineLimit(1)
            }
            Spacer(minLength: 4)
            if let process = row.process, row.blockedReason == nil {
                Menu { items(process) } label: { Image(systemName: "ellipsis.circle") }
                    .menuStyle(.borderlessButton).menuIndicator(.hidden).fixedSize()
                    .accessibilityLabel("Actions for \(row.ownerName), PID \(row.pid), port \(row.port)")
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 4)
        .background(.white.opacity(0.03), in: RoundedRectangle(cornerRadius: 8))
        .opacity(row.blockedReason == nil ? 1 : 0.55)
        .help(row.blockedReason.map { "\(row.addresses.joined(separator: ", ")) · \($0)" } ?? row.addresses.joined(separator: ", "))
        .contextMenu { if let process = row.process, row.blockedReason == nil { items(process) } }
    }

    @ViewBuilder private func items(_ process: FfiProcessInfo) -> some View {
        Button("Inspect") { state.select(process.identity) }
        Divider()
        ActionItems.process(process, actions: state.actions)
    }

    private func caveatText(_ caveat: String) -> some View {
        Label(caveat, systemImage: "eye.slash").font(.system(size: 10)).foregroundStyle(.secondary)
            .fixedSize(horizontal: false, vertical: true)
    }
}
