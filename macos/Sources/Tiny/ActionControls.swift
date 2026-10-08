import AppKit
import SwiftUI
import TinyEngine

/// Reveal / Copy (X3): Swift-only and non-mutating. A missing path disables the item.
enum Finder {
    static func existing(_ path: String?) -> String? {
        guard let path, path.hasPrefix("/"), FileManager.default.fileExists(atPath: path) else { return nil }
        return path
    }
    static func reveal(_ path: String) { NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: path)]) }
    static func copy(_ text: String) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(text, forType: .string)
    }
}

/// Menu items shared by the row context menus, the inspector `⋯` menus and the Ports tile.
@MainActor enum ActionItems {
    @ViewBuilder static func app(_ group: AppGroup, actions: ActionState) -> some View {
        switch AppQuit.resolve(group) {
        case .ready(let target) where !actions.isRunning:
            Button("Quit \(group.name)") { actions.request(app: target) }
        case .ready:
            unavailable(ActionCopy.busy.text)
        case .unavailable(let reason):
            unavailable(reason)
        }
        Divider()
        bundleItems(group)
    }

    /// Reveal and Copy Path for an app bundle, also used by the inspector `⋯` menu.
    @ViewBuilder static func bundleItems(_ group: AppGroup) -> some View {
        reveal(group.bundlePath, title: "Reveal in Finder")
        Button("Copy Path") { group.bundlePath.map(Finder.copy) }.disabled(group.bundlePath == nil)
    }

    @ViewBuilder static func process(_ process: FfiProcessInfo, actions: ActionState) -> some View {
        if let reason = ActionCopy.eligibility(process) ?? (actions.isRunning ? ActionCopy.busy.text : nil) {
            unavailable(reason)
        } else {
            Button("Quit") { actions.request(process, kind: .graceful) }
            Button("Force Quit…") { actions.request(process, kind: .force) }
        }
        Divider()
        reveal(process.executablePath, title: "Reveal Executable in Finder")
        Button("Copy PID") { Finder.copy(String(process.pid)) }
    }

    @ViewBuilder static func group(_ group: AppGroup, actions: ActionState) -> some View {
        if group.isApplication { app(group, actions: actions) }
        else if let member = group.members.first { process(member, actions: actions) }
    }

    private static func unavailable(_ reason: String) -> some View {
        Button("Quit unavailable — \(reason)") {}.disabled(true)
    }

    @ViewBuilder private static func reveal(_ path: String?, title: String) -> some View {
        if let path = Finder.existing(path) {
            Button(title) { Finder.reveal(path) }
        } else {
            Button("\(title) (path unavailable)") {}.disabled(true)
        }
    }
}

/// Quit plus a `⋯` menu for the selected app (O1a).
struct AppActionBar: View {
    let group: AppGroup
    let actions: ActionState

    var body: some View {
        let resolution = AppQuit.resolve(group)
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                Button("Quit") { if case let .ready(target) = resolution { actions.request(app: target) } }
                    .disabled(actions.isRunning || resolution.reason != nil)
                    .help(resolution.reason ?? "Ask \(group.name) to quit")
                Menu { ActionItems.bundleItems(group) } label: { Image(systemName: "ellipsis") }
                    .menuStyle(.borderlessButton).menuIndicator(.hidden).fixedSize()
                    .accessibilityLabel("More actions for \(group.name)")
            }.controlSize(.small)
            if let reason = resolution.reason { Text(reason).font(.caption2).foregroundStyle(.secondary) }
        }
    }
}

/// Quit, Force Quit…, Reveal and Copy PID for the selected member process.
/// Reveal and Copy stay available when Quit is blocked.
struct ProcessActionBar: View {
    let process: FfiProcessInfo
    let actions: ActionState

    var body: some View {
        let blocked = ActionCopy.eligibility(process)
        let quitDisabled = blocked != nil || actions.isRunning
        let path = Finder.existing(process.executablePath)
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 6) {
                Button("Quit") { actions.request(process, kind: .graceful) }.disabled(quitDisabled)
                Button("Force Quit…") { actions.request(process, kind: .force) }.disabled(quitDisabled)
                Spacer(minLength: 0)
                Button { path.map(Finder.reveal) } label: { Image(systemName: "folder") }
                    .disabled(path == nil).help(path == nil ? "Executable path unavailable" : "Reveal executable in Finder")
                    .accessibilityLabel("Reveal executable in Finder")
                Button { Finder.copy(String(process.pid)) } label: { Image(systemName: "doc.on.doc") }
                    .help("Copy PID \(process.pid)").accessibilityLabel("Copy PID")
            }.controlSize(.small)
            if let blocked { Text(blocked).font(.caption2).foregroundStyle(.secondary) }
        }
    }
}

/// Result of the last action. Unknown outcomes stay until dismissed.
struct NoticeBanner: View {
    let notice: ActionNotice
    let dismiss: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: symbol).foregroundStyle(tint).accessibilityHidden(true)
            Text(notice.text).font(.callout).textSelection(.enabled).fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 8)
            Button("Dismiss", action: dismiss).buttonStyle(.borderless)
        }
        .padding(.horizontal, 16).padding(.vertical, 10)
        .background(tint.opacity(0.10), in: RoundedRectangle(cornerRadius: 14))
        .overlay(RoundedRectangle(cornerRadius: 14).strokeBorder(tint.opacity(0.28)))
        .accessibilityElement(children: .combine)
    }

    private var tint: Color {
        switch notice.tone {
        case .success: return Theme.accent
        case .info: return .cyan
        case .warning, .unknown: return .orange
        case .failure: return .red
        }
    }

    private var symbol: String {
        switch notice.tone {
        case .success: return "checkmark.circle.fill"
        case .info: return "info.circle.fill"
        case .warning: return "exclamationmark.triangle.fill"
        case .failure: return "xmark.octagon.fill"
        case .unknown: return "questionmark.diamond.fill"
        }
    }
}
