import Foundation
@testable import TinyEngine

struct EngineTests {
    static func run() async throws {
        try await liveProcessLifecycleAndWarmup()
        try await wrongStartTimeIsRejected()
        try unavailableAndWarmupAreNotZero()
        try postProbeIdentityIsRevalidated()
        print("PASS: 4 engine checks (live lifecycle, identity, unavailable/warmup, post-probe revalidation)")
    }

    private static func liveProcessLifecycleAndWarmup() async throws {
        let child = Process()
        child.executableURL = URL(fileURLWithPath: "/bin/sleep")
        child.arguments = ["30"]
        try child.run()
        defer { if child.isRunning { child.terminate(); child.waitUntilExit() } }
        let engine = Engine()
        let first = try await engine.list()
        try check(!first.cpuMeasured, "first CPU sample unmeasured")
        try check(first.systemUsage.cpuPercent == nil && first.systemUsage.cpuWarmingUp, "system CPU warmup explicit")
        try check((first.systemUsage.memoryTotalBytes ?? 0) > 0, "system RAM total measured")
        try check((first.systemUsage.memoryUsedBytes ?? UInt64.max) <= (first.systemUsage.memoryTotalBytes ?? 0), "system used RAM bounded")
        try check(first.processes.allSatisfy { $0.cpuPercent == nil }, "no fake zero CPU during warmup")
        guard let spawned = first.processes.first(where: { $0.pid == UInt32(child.processIdentifier) }) else {
            throw CheckFailure(message: "owned child must appear")
        }
        try check(spawned.name == "sleep" && spawned.isCurrentUser, "owned child identity")
        let detail = try await engine.detail(spawned.identity)
        try check(detail.process.identity == spawned.identity, "detail identity")
        try check(detail.ports != nil || detail.portsError != nil, "ports have value or explicit failure")
        let measured = try await engine.list()
        try check(measured.systemUsage.cpuPercent.map { (0...100).contains($0) } == true, "whole-machine CPU 0...100 measured")
        try check(!measured.systemUsage.cpuWarmingUp, "system warmup completed")
        let sampled = measured.processes.first { $0.identity == spawned.identity }
        try check(sampled?.cpuMeasured == true && sampled?.cpuPercent != nil, "third spaced sample measured")
        child.terminate()
        child.waitUntilExit()
        let after = try await engine.list()
        try check(!after.processes.contains { $0.identity == spawned.identity }, "exited child removed")
        do {
            _ = try await engine.detail(spawned.identity)
            throw CheckFailure(message: "exited detail must throw")
        } catch is FfiError { /* The process disappeared, not an empty successful detail. */ }
    }

    private static func wrongStartTimeIsRejected() async throws {
        let engine = Engine()
        let list = try await engine.list()
        guard let own = list.processes.first(where: { $0.pid == UInt32(ProcessInfo.processInfo.processIdentifier) }) else {
            throw CheckFailure(message: "own process must appear")
        }
        let wrong = ProcessIdentity(pid: own.pid, startTime: own.startTime + 1)
        do {
            _ = try await engine.detail(wrong)
            throw CheckFailure(message: "reused PID identity must throw")
        } catch is EngineError { /* Expected identity mismatch. */ }
    }

    private static func postProbeIdentityIsRevalidated() throws {
        let row = process(memory: 4096, cpu: 0)
        let matching = FfiProcessSnapshot(processes: [row], sampledAt: 100, cpuMeasured: true, systemUsage: FfiSystemUsage(cpuPercent: nil, cpuWarmingUp: true, memoryUsedBytes: nil, memoryTotalBytes: nil))
        try Engine.requireCurrent(row.identity, in: matching)
        for snapshot in [FfiProcessSnapshot(processes: [], sampledAt: 100, cpuMeasured: true, systemUsage: FfiSystemUsage(cpuPercent: nil, cpuWarmingUp: true, memoryUsedBytes: nil, memoryTotalBytes: nil)), matching] {
            let expected = snapshot.processes.isEmpty ? row.identity :
                ProcessIdentity(pid: row.pid, startTime: row.startTime + 1)
            do {
                try Engine.requireCurrent(expected, in: snapshot)
                throw CheckFailure(message: "post-probe exit/PID reuse must fail")
            } catch is EngineError { /* Probe result must not survive loss of its process identity. */ }
        }
    }

    private static func unavailableAndWarmupAreNotZero() throws {
        try check(process(memory: nil, cpu: nil).memoryText == "Unavailable", "missing memory")
        try check(process(memory: nil, cpu: nil).cpuText == "Unavailable", "restricted CPU")
        try check(process(memory: 4096, cpu: nil).cpuText == "Warming up", "CPU warmup")
        try check(process(memory: 4096, cpu: 0).cpuText == "0.0%", "real zero CPU")
        try check(process(memory: 4096, cpu: 135).cpuText == "135.0%", "per-core CPU over 100")
    }

    private static func process(memory: UInt64?, cpu: Float?) -> FfiProcessInfo {
        FfiProcessInfo(pid: 1, name: "fixture", user: nil, isCurrentUser: false,
            parentPid: nil, startTime: 10, cpuPercent: cpu, cpuMeasured: cpu != nil, memoryBytes: memory, executablePath: nil)
    }
}

struct CheckFailure: Error, CustomStringConvertible {
    let message: String
    var description: String { message }
}

func check(_ condition: @autoclosure () -> Bool, _ message: String) throws {
    guard condition() else { throw CheckFailure(message: message) }
}
