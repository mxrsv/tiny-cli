import Foundation
import TinyEngine

struct AppDescriptor: Equatable, Sendable {
    let bundlePath: String
    let name: String
}

struct AppGroup: Identifiable {
    let id: String
    let name: String
    let bundlePath: String?
    let members: [FfiProcessInfo]
    var isApplication: Bool { bundlePath != nil }
    var measuredCPU: [Double] {
        members.compactMap { row in
            guard row.cpuMeasured, let value = row.cpuPercent, value.isFinite, value >= 0 else { return nil }
            return Double(value)
        }
    }
    var cpuPercent: Double? { measuredCPU.isEmpty ? nil : measuredCPU.reduce(0, +) }
    var measuredMemory: [UInt64] { members.compactMap(\.memoryBytes) }
    var memoryBytes: UInt64? {
        guard !measuredMemory.isEmpty else { return nil }
        var total: UInt64 = 0
        for bytes in measuredMemory {
            let addition = total.addingReportingOverflow(bytes)
            guard !addition.overflow else { return nil }
            total = addition.partialValue
        }
        return total
    }
    var cpuText: String {
        guard let cpuPercent else { return measuredMemory.isEmpty ? "Unavailable" : "Warming up" }
        return String(format: "%.1f%%", cpuPercent) + (measuredCPU.count < members.count ? " · partial" : "")
    }
    var memoryText: String {
        guard let memoryBytes else { return "Unavailable" }
        return ByteCountFormatter.string(fromByteCount: Int64(clamping: memoryBytes), countStyle: .memory)
            + (measuredMemory.count < members.count ? " · partial" : "")
    }
    func matches(_ query: String) -> Bool {
        query.isEmpty || name.localizedCaseInsensitiveContains(query) || members.contains {
            $0.name.localizedCaseInsensitiveContains(query) || String($0.pid).contains(query)
                || ($0.user?.localizedCaseInsensitiveContains(query) ?? false)
        }
    }
}

enum AppGroups {
    private static let maximumParentDepth = 64

    /// Lexical bundle boundary only: nested helper bundles belong to their outer application.
    static func bundlePath(for executable: String?) -> String? {
        guard let executable, executable.hasPrefix("/") else { return nil }
        let components = URL(fileURLWithPath: executable).standardizedFileURL.pathComponents
        guard let end = components.firstIndex(where: { $0.lowercased().hasSuffix(".app") }) else { return nil }
        return NSString.path(withComponents: Array(components.prefix(through: end)))
    }

    static func make(_ rows: [FfiProcessInfo], catalog: [ProcessIdentity: AppDescriptor] = [:]) -> [AppGroup] {
        let byPID = Dictionary(rows.map { ($0.pid, $0) }, uniquingKeysWith: { first, _ in first })
        let seeds = Dictionary(uniqueKeysWithValues: rows.compactMap { row -> (ProcessIdentity, AppDescriptor)? in
            if let path = bundlePath(for: row.executablePath) {
                let metadata = catalog[row.identity]
                let name: String
                if let metadata, metadata.bundlePath == path { name = metadata.name }
                else { name = URL(fileURLWithPath: path).deletingPathExtension().lastPathComponent }
                return (row.identity, AppDescriptor(bundlePath: path, name: name))
            }
            return catalog[row.identity].map { (row.identity, $0) }
        })
        let assignments = rows.map { row in (row, resolve(row, seeds: seeds, byPID: byPID)) }
        let grouped = Dictionary(grouping: assignments) { row, app in
            app.map { "app:\($0.bundlePath)" } ?? "process:\(row.pid):\(row.startTime)"
        }
        return grouped.map { id, entries in
            let sorted = entries.sorted { $0.0.pid < $1.0.pid }
            let app = sorted.compactMap(\.1).first
            return AppGroup(id: id, name: app?.name ?? sorted[0].0.name,
                            bundlePath: app?.bundlePath, members: sorted.map(\.0))
        }
    }

    private static func resolve(_ process: FfiProcessInfo, seeds: [ProcessIdentity: AppDescriptor],
                                byPID: [UInt32: FfiProcessInfo]) -> AppDescriptor? {
        var current = process
        var visited = Set<ProcessIdentity>()
        for _ in 0..<maximumParentDepth {
            guard visited.insert(current.identity).inserted else { return nil }
            if let ownApp = seeds[current.identity] { return ownApp }
            guard let parentPID = current.parentPid, let parent = byPID[parentPID],
                  let owner = current.user, owner == parent.user,
                  current.startTime > 0, parent.startTime > 0, parent.startTime <= current.startTime else { return nil }
            current = parent
        }
        return nil
    }
}
