import AppKit
import TinyEngine

/// macOS supplies presentation metadata; Rust remains the only resource sampler.
@MainActor final class AppCatalog {
    private var icons: [String: NSImage] = [:]
    private var names: [String: String] = [:]

    func descriptors(for processes: [FfiProcessInfo]) -> [ProcessIdentity: AppDescriptor] {
        let byPID = Dictionary(processes.map { ($0.pid, $0) }, uniquingKeysWith: { first, _ in first })
        var resolved: [ProcessIdentity: AppDescriptor] = [:]
        for app in NSWorkspace.shared.runningApplications {
            guard app.processIdentifier > 0, let process = byPID[UInt32(app.processIdentifier)],
                  Self.matches(process, launchDate: app.launchDate),
                  let url = app.bundleURL, let path = AppGroups.bundlePath(for: url.path),
                  let descriptor = Self.descriptor(bundleURL: url, localizedName: app.localizedName, knownName: names[path]) else { continue }
            let isOuterBundle = url.standardizedFileURL.path == path
            let name = descriptor.name
            if isOuterBundle {
                names[path] = name
                if icons[path] == nil, let icon = app.icon { icons[path] = icon }
            }
            resolved[process.identity] = AppDescriptor(bundlePath: path, name: name)
        }
        for process in processes {
            guard let path = AppGroups.bundlePath(for: process.executablePath) else { continue }
            let name = names[path] ?? URL(fileURLWithPath: path).deletingPathExtension().lastPathComponent
            resolved[process.identity] = AppDescriptor(bundlePath: path, name: name)
        }
        return resolved
    }

    static func descriptor(bundleURL: URL, localizedName: String?, knownName: String?) -> AppDescriptor? {
        guard bundleURL.isFileURL, let path = AppGroups.bundlePath(for: bundleURL.path) else { return nil }
        let fallback = URL(fileURLWithPath: path).deletingPathExtension().lastPathComponent
        let name = bundleURL.standardizedFileURL.path == path ? (localizedName ?? fallback) : (knownName ?? fallback)
        return AppDescriptor(bundlePath: path, name: name)
    }

    static func matches(_ process: FfiProcessInfo, launchDate: Date?) -> Bool {
        guard process.startTime > 0, let launchDate else { return false }
        // Rust start times have one-second resolution; require the same epoch second.
        return launchDate.timeIntervalSince1970 >= Double(process.startTime)
            && launchDate.timeIntervalSince1970 < Double(process.startTime) + 1
    }

    func retainIcons(for groups: [AppGroup]) {
        let paths = Set(groups.compactMap(\.bundlePath))
        icons = icons.filter { paths.contains($0.key) }
        names = names.filter { paths.contains($0.key) }
        for path in paths where icons[path] == nil {
            icons[path] = NSWorkspace.shared.icon(forFile: path)
        }
    }

    func icon(for group: AppGroup) -> NSImage {
        if let path = group.bundlePath, let image = icons[path] { return image }
        return NSImage(systemSymbolName: group.isApplication ? "app" : "terminal", accessibilityDescription: nil)
            ?? NSImage(size: NSSize(width: 32, height: 32))
    }
}
