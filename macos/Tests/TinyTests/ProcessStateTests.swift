import Foundation
import TinyEngine

@main @MainActor enum ProcessStateTests {
    static func main() async {
        let arguments = CommandLine.arguments
        if arguments.count == 3 && arguments[1] == ActionTests.markerKillFlag { ActionTests.markerThenKill(arguments[2]) }
        do {
            try await EngineTests.run()
            try searchAndSortKeepUnavailableLast()
            try refreshRetainsSelectionAndRejectsPidReuse()
            try obsoleteDetailNeverReplacesNewSelection()
            try errorsKeepLastGoodSampleAndReportStale()
            try cancelledResponseCannotInstallDetail()
            print("PASS: 5 process state checks")
            try AppGroupTests.run()
            try await ActionTests.run()
            try await CleanTests.run()
            print("PASS: 31 total native checks")
        } catch {
            FileHandle.standardError.write(Data("FAIL: \(error)\n".utf8))
            exit(1)
        }
    }

    private static func searchAndSortKeepUnavailableLast() throws {
        let rows = [process(3, name: "Safari", user: "alice", cpu: 25, memory: 4096),
                    process(1, name: "helper", user: "root", cpu: nil, memory: nil),
                    process(2, name: "worker", user: "alice", cpu: 150, memory: 0)]
        let state = AppState()
        state.apply(snapshot(rows))
        state.query = "SAF"
        try check(state.visibleGroups.flatMap(\.members).map(\.pid) == [3], "group member case-insensitive search")
        state.query = "  alice  "
        try check(state.visibleGroups.flatMap(\.members).map(\.pid) == [2, 3], "group owner search and CPU sort")
        state.query = "1"
        try check(state.visibleGroups.flatMap(\.members).map(\.pid) == [1], "group member PID search")
        state.query = ""
        try check(state.visibleGroups.isEmpty, "background hidden by default")
        state.showBackground = true
        state.sort = .memory
        try check(state.visibleGroups.flatMap(\.members).map(\.pid) == [3, 2, 1], "group memory nil after zero")
        state.sort = .name
        try check(state.visibleGroups.flatMap(\.members).map(\.pid) == [1, 3, 2], "group name sort")
    }

    private static func refreshRetainsSelectionAndRejectsPidReuse() throws {
        let state = AppState()
        let old = process(42)
        state.apply(snapshot([old]))
        state.select(old.identity)
        state.stop()
        state.apply(snapshot([old, process(1)]))
        try check(state.selection == old.identity && state.selectedProcess?.pid == 42, "stable selection")
        state.apply(snapshot([process(42, start: 99)]))
        try check(state.selection == nil && state.selectedProcess == nil, "reused PID not selected")
        try check(state.detail == nil && state.detailError?.contains("no longer") == true, "explicit exited state")
    }

    private static func obsoleteDetailNeverReplacesNewSelection() throws {
        let state = AppState()
        let first = process(10), next = process(20)
        state.apply(snapshot([first, next]))
        state.select(first.identity) // generation 1
        state.select(next.identity) // generation 2
        state.acceptDetail(detail(first), for: first.identity, generation: 1)
        try check(state.detail == nil, "late previous selection discarded")
        state.acceptDetail(detail(next), for: next.identity, generation: 2)
        try check(state.detail?.process.identity == next.identity, "current detail accepted")
        state.acceptDetail(detail(process(20, start: 99)), for: next.identity, generation: 2)
        try check(state.detail?.process.identity == next.identity, "wrong returned identity discarded")
        state.stop()
    }

    private static func errorsKeepLastGoodSampleAndReportStale() throws {
        let state = AppState()
        state.recordListError(CocoaError(.fileReadNoPermission))
        try check(state.status == "Unavailable", "first failure unavailable")
        state.apply(snapshot([process(5)]))
        state.recordListError(CocoaError(.fileReadNoPermission))
        try check(state.status == "Stale" && state.processes.map(\.pid) == [5], "failure retains last-good data")
        state.apply(snapshot([process(6)]))
        try check(state.status == "Live" && state.listError == nil, "successful retry clears stale")
        state.paused = true
        try check(state.status == "Paused" && state.processes.map(\.pid) == [6], "pause retains snapshot")
    }

    private static func cancelledResponseCannotInstallDetail() throws {
        let state = AppState()
        let row = process(10)
        state.apply(snapshot([row]))
        state.select(row.identity)
        state.stop()
        state.acceptDetail(detail(row), for: row.identity, generation: 1)
        try check(state.detail == nil, "cancelled response discarded")
        try check(!state.detailLoading, "cancellation clears detail spinner")
    }

    private static func process(_ pid: UInt32, start: UInt64 = 10, name: String = "sample", user: String = "alice",
                                cpu: Float? = 10, memory: UInt64? = 1024) -> FfiProcessInfo {
        FfiProcessInfo(pid: pid, name: name, user: user, isCurrentUser: true, parentPid: nil,
                       startTime: start, cpuPercent: cpu, cpuMeasured: cpu != nil, memoryBytes: memory, executablePath: nil, refusal: nil)
    }
    private static func snapshot(_ rows: [FfiProcessInfo]) -> FfiProcessSnapshot {
        FfiProcessSnapshot(processes: rows, sampledAt: 100, cpuMeasured: true, systemUsage: FfiSystemUsage(cpuPercent: nil, cpuWarmingUp: true, memoryUsedBytes: nil, memoryTotalBytes: nil))
    }
    private static func detail(_ row: FfiProcessInfo) -> FfiProcessDetail {
        FfiProcessDetail(process: row, parent: .none, children: [], ports: nil, portsError: "Unavailable")
    }
}
