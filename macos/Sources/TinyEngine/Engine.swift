import Foundation

public struct ProcessIdentity: Hashable, Sendable {
    public let pid: UInt32
    public let startTime: UInt64

    public init(pid: UInt32, startTime: UInt64) {
        self.pid = pid
        self.startTime = startTime
    }
}

public extension FfiProcessInfo {
    var identity: ProcessIdentity { ProcessIdentity(pid: pid, startTime: startTime) }
    var cpuText: String {
        guard let cpuPercent, cpuMeasured else {
            return memoryBytes == nil ? "Unavailable" : "Warming up"
        }
        return String(format: "%.1f%%", cpuPercent)
    }
    var memoryText: String {
        guard let memoryBytes else { return "Unavailable" }
        return ByteCountFormatter.string(fromByteCount: Int64(clamping: memoryBytes), countStyle: .memory)
    }
}

public enum EngineError: LocalizedError {
    case identityChanged
    public var errorDescription: String? { "This process exited or its PID was reused. Select a current process." }
}

/// Serial actor keeps blocking FFI calls off the main actor. No detached tasks or worker processes.
public actor Engine {
    private var session: TinySession?
    private var lastSample: TimeInterval = 0
    private static let minimumSampleInterval: TimeInterval = 0.25

    public init() {}

    public func list() throws -> FfiProcessSnapshot {
        try Task.checkCancellation()
        defer { lastSample = ProcessInfo.processInfo.systemUptime }
        return try readySession().processList()
    }

    public func detail(_ identity: ProcessIdentity) throws -> FfiProcessDetail {
        try Task.checkCancellation()
        defer { lastSample = ProcessInfo.processInfo.systemUptime }
        let result = try readySession().processDetail(pid: identity.pid)
        guard result.process.identity == identity else { throw EngineError.identityChanged }
        // lsof runs after the first snapshot. Revalidate after the probe so its
        // ports cannot be attached to a process that exited or was replaced.
        lastSample = ProcessInfo.processInfo.systemUptime
        try Self.requireCurrent(identity, in: list())
        return result
    }

    static func requireCurrent(_ identity: ProcessIdentity, in snapshot: FfiProcessSnapshot) throws {
        guard snapshot.processes.contains(where: { $0.identity == identity }) else {
            throw EngineError.identityChanged
        }
    }

    private func readySession() -> TinySession {
        // sysinfo's CPU warm-up needs three spaced samples, including detail queries.
        // A synchronous bounded wait preserves actor serialization without reentrancy.
        let elapsed = ProcessInfo.processInfo.systemUptime - lastSample
        if elapsed < Self.minimumSampleInterval {
            Thread.sleep(forTimeInterval: Self.minimumSampleInterval - elapsed)
        }
        if let session { return session }
        let created = TinySession()
        session = created
        return created
    }
}

extension FfiProcessInfo: Identifiable {
    public var id: ProcessIdentity { identity }
}
