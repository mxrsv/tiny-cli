import SwiftUI
import TinyEngine

struct ProcessDetailView: View {
    let state: AppState

    var body: some View {
        Tile {
            ScrollViewReader { scroller in ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    HStack {
                        Text("INSPECTOR").font(.system(size: 10, weight: .semibold)).tracking(1).foregroundStyle(.secondary)
                        Spacer()
                        if state.detailLoading { ProgressView().controlSize(.small) }
                    }
                    if let group = state.selectedGroup {
                        appIdentity(group)
                        if group.isApplication { AppActionBar(group: group, actions: state.actions) }
                        memberList(group)
                        if let process = state.detail?.process ?? state.selectedProcess {
                            identity(process).id(Self.processAnchor)
                            ProcessActionBar(process: process, actions: state.actions)
                            if let error = state.detailError {
                                Label(state.detail == nil ? "Unavailable" : "Stale detail", systemImage: "exclamationmark.triangle")
                                    .foregroundStyle(.orange)
                                Text(error).font(.caption).foregroundStyle(.secondary).textSelection(.enabled)
                            }
                            measurements(process)
                            if let detail = state.detail { relationships(detail) }
                            else if state.detailError == nil {
                                Text("Reading parent, children and listening ports…").font(.caption).foregroundStyle(.secondary)
                            }
                        } else {
                            Text(state.detailError ?? "Choose a member process to inspect its details.")
                                .font(.caption).foregroundStyle(.secondary)
                        }
                    } else {
                        empty
                    }
                }.frame(maxWidth: .infinity, alignment: .leading)
            }
            // A large app pushes the selected process below the fold; bring its actions into view.
            .task(id: state.selection) {
                if state.selection != nil { scroller.scrollTo(Self.processAnchor, anchor: .top) }
            } }
        }
    }

    private static let processAnchor = "selected-process"

    private func appIdentity(_ group: AppGroup) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Image(nsImage: state.icon(for: group)).resizable().interpolation(.high)
                .frame(width: 56, height: 56).accessibilityHidden(true)
            Text(group.name).font(.system(size: 22, weight: .semibold, design: .rounded))
                .textSelection(.enabled).fixedSize(horizontal: false, vertical: true)
            Text("\(group.members.count) \(group.members.count == 1 ? "process" : "processes") · \(group.cpuText) CPU")
                .font(.caption).foregroundStyle(.secondary)
            Text("\(group.memoryText) resident memory").font(.caption).foregroundStyle(.secondary)
        }
    }

    private func memberList(_ group: AppGroup) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("MEMBER PROCESSES").font(.system(size: 10, weight: .semibold))
                .tracking(0.6).foregroundStyle(.secondary)
            ScrollView {
                LazyVStack(spacing: 4) {
                    ForEach(group.members) { member in
                        Button { state.select(member.identity) } label: {
                            HStack {
                                VStack(alignment: .leading, spacing: 3) {
                                    Text(member.name).lineLimit(1)
                                    Text("PID \(String(member.pid)) · \(member.cpuText)").font(.caption2).foregroundStyle(.secondary)
                                }
                                Spacer(minLength: 0)
                                if state.selection == member.identity { Image(systemName: "checkmark").foregroundStyle(Theme.accent) }
                            }.font(.caption).padding(8).contentShape(Rectangle())
                                .background(state.selection == member.identity ? Theme.accent.opacity(0.10) : .white.opacity(0.03),
                                            in: RoundedRectangle(cornerRadius: 8))
                        }.buttonStyle(.plain).help("Inspect \(member.name), PID \(member.pid)")
                            .contextMenu { ActionItems.process(member, actions: state.actions) }
                    }
                }
            }.frame(height: min(CGFloat(group.members.count) * 48, 172))
        }
    }

    private func identity(_ process: FfiProcessInfo) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Divider()
            Text(process.name).font(.system(size: 16, weight: .semibold)).textSelection(.enabled)
            HStack { Pill(text: "PID \(process.pid)"); Text(process.user ?? "Owner unavailable").foregroundStyle(.secondary) }
                .font(.caption)
            if process.startTime > 0 {
                Text("Started \(Date(timeIntervalSince1970: TimeInterval(process.startTime)).formatted(date: .abbreviated, time: .shortened))")
                    .font(.caption2).foregroundStyle(.secondary)
            } else { Text("Start time unavailable").font(.caption2).foregroundStyle(.secondary) }
        }
    }

    private func measurements(_ process: FfiProcessInfo) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Divider()
            pair("CPU", process.cpuText)
            pair("Resident memory", process.memoryText)
            Text("CPU is measured per core. 100% means one full core; multi-core work can exceed it.")
                .font(.caption2).foregroundStyle(.secondary)
        }
    }

    private func relationships(_ detail: FfiProcessDetail) -> some View {
        VStack(alignment: .leading, spacing: 14) {
            Divider()
            group("PARENT", text: parentText(detail.parent))
            group("CHILDREN · \(detail.children.count)", text: detail.children.isEmpty ? "No children in this sample" :
                    detail.children.sorted { $0.pid < $1.pid }.map { "\($0.name) · \($0.pid)" }.joined(separator: "\n"))
            group("LISTENING TCP PORTS", text: portsText(detail))
            if let sampled = state.detailSampledAt {
                Text("Detail sample \(sampled.formatted(date: .omitted, time: .standard))")
                    .font(.caption2).foregroundStyle(.secondary)
            }
        }
    }

    private var empty: some View {
        VStack(alignment: .leading, spacing: 14) {
            Image(systemName: state.groupSelection == nil ? "cursorarrow.click" : "exclamationmark.circle")
                .font(.system(size: 32)).foregroundStyle(.secondary).padding(.top, 45)
            Text(state.groupSelection == nil ? "Take a closer look" : "App unavailable")
                .font(.system(size: 22, weight: .semibold, design: .rounded))
            Text(state.detailError ?? "Select an app to see its member processes, then inspect a process for parent, children and listening ports.")
                .foregroundStyle(.secondary).font(.callout)
            Text("Quit and Force Quit always ask first. Nothing is stopped automatically.")
                .foregroundStyle(Theme.accent).font(.caption).padding(.top, 6)
        }
    }

    private func pair(_ name: String, _ value: String) -> some View {
        HStack { Text(name).foregroundStyle(.secondary); Spacer(); Text(value).monospacedDigit() }.font(.callout)
    }

    private func group(_ title: String, text: String) -> some View {
        VStack(alignment: .leading, spacing: 7) {
            Text(title).font(.system(size: 10, weight: .semibold)).tracking(0.5).foregroundStyle(.secondary)
            Text(text).font(.callout).textSelection(.enabled).fixedSize(horizontal: false, vertical: true)
        }
    }

    private func parentText(_ parent: FfiParentState) -> String {
        switch parent {
        case .none: return "No parent"
        case .running(let pid, let name): return "\(name) · \(pid)"
        case .exited(let pid): return "PID \(pid) · exited"
        }
    }

    private func portsText(_ detail: FfiProcessDetail) -> String {
        guard let ports = detail.ports else { return "Unavailable\n\(detail.portsError ?? "Could not inspect listening ports.")" }
        return ports.isEmpty ? "No listening TCP ports" : ports.map { "\($0.address):\($0.port)" }.joined(separator: "\n")
    }
}
