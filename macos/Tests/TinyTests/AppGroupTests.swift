import Foundation
import TinyEngine

@MainActor enum AppGroupTests {
    static func run() throws {
        try bundleAndAncestryBoundaries()
        try unknownOwnersCyclesAndConservation()
        try partialAggregatesAndZero()
        try nativeMetadataPriorityAndIdentity()
        try groupSelectionSurvivesMemberChurn()
        try movedMemberCannotKeepForeignDetail()
        print("PASS: 6 app-group checks (boundaries, conservation, partial totals, metadata, churn, regrouping)")
    }

    private static func bundleAndAncestryBoundaries() throws {
        let rows = [row(1, path: "/Applications/Browser.app/Contents/MacOS/Browser"),
                    row(2, parent: 1, path: "/Applications/Browser.app/Contents/Frameworks/Helper.app/Contents/MacOS/Helper"),
                    row(3, parent: 2, path: "/usr/bin/tool"),
                    row(4, parent: 1, path: "/Applications/Editor.app/Contents/MacOS/Editor"),
                    row(5, path: "/Users/alice/Browser.app/Contents/MacOS/Browser")]
        let groups = AppGroups.make(rows)
        let browser = groups.first { $0.bundlePath == "/Applications/Browser.app" }
        try check(browser?.members.map(\.pid) == [1, 2, 3], "nested helpers and command descendants group with outer app")
        try check(groups.count == 3, "independent child apps and same-named different paths remain separate")
        try check(groups.first { $0.bundlePath == "/Applications/Editor.app" }?.members.map(\.pid) == [4], "child app overrides parent")
        try check(AppGroups.bundlePath(for: "/tmp/not.app-data/worker") == nil, "app-like names do not seed groups")
        try check(AppGroups.bundlePath(for: "relative.app/tool") == nil, "relative paths rejected")
    }

    private static func unknownOwnersCyclesAndConservation() throws {
        let rows = [row(1, path: "/Applications/Editor.app/Contents/MacOS/Editor"),
                    row(2, parent: 1, user: nil), row(3, parent: 1, user: "bob"),
                    row(4, parent: 1, start: 1), row(5, parent: 6), row(6, parent: 5),
                    row(7, parent: 999), row(8, parent: 1, start: 0)]
        let groups = AppGroups.make(rows)
        try check(groups.count == rows.count, "unknown owner, cross-owner, invalid time and cycle remain unresolved")
        let identities = groups.flatMap(\.members).map(\.identity)
        try check(identities.count == rows.count && Set(identities) == Set(rows.map(\.identity)), "every process appears exactly once")
    }

    private static func partialAggregatesAndZero() throws {
        let members = [row(1, cpu: 120, memory: 1000), row(2, cpu: 0, memory: 2000), row(3, cpu: nil, memory: nil)]
        let group = AppGroup(id: "app", name: "Example", bundlePath: "/Example.app", members: members)
        try check(group.cpuPercent == 120 && group.cpuText.contains("partial"), "per-core CPU not clamped; partial labeled")
        try check(group.memoryBytes == 3000 && group.memoryText.contains("partial"), "available resident sum labeled partial")
        let missing = AppGroup(id: "missing", name: "Missing", bundlePath: nil, members: [row(4, cpu: nil, memory: nil)])
        try check(missing.cpuPercent == nil && missing.memoryBytes == nil && missing.cpuText == "Unavailable", "all unavailable not zero")
        let zero = AppGroup(id: "zero", name: "Zero", bundlePath: nil, members: [row(5, cpu: 0, memory: 0)])
        try check(zero.cpuPercent == 0 && zero.memoryBytes == 0 && !zero.cpuText.contains("partial"), "measured zero retained")
    }

    private static func nativeMetadataPriorityAndIdentity() throws {
        let root = URL(fileURLWithPath: "/Applications/Browser.app")
        let helper = root.appendingPathComponent("Contents/Helpers/Helper.app")
        try check(AppCatalog.descriptor(bundleURL: root, localizedName: "Browser Localized", knownName: nil)?.name == "Browser Localized", "root localized metadata")
        try check(AppCatalog.descriptor(bundleURL: helper, localizedName: "Wrong Helper", knownName: "Browser Localized")?.name == "Browser Localized", "helper cannot overwrite root name")
        try check(AppCatalog.descriptor(bundleURL: helper, localizedName: "Wrong Helper", knownName: nil)?.name == "Browser", "helper fallback uses outer name")
        try check(AppCatalog.matches(row(1), launchDate: Date(timeIntervalSince1970: 10.5)), "same identity launch second")
        try check(!AppCatalog.matches(row(1), launchDate: Date(timeIntervalSince1970: 11)), "later PID identity rejected")
        try check(!AppCatalog.matches(row(1), launchDate: nil), "unknown launch identity not trusted")
    }

    private static func groupSelectionSurvivesMemberChurn() throws {
        let state = AppState()
        let root = row(1, path: "/Applications/Editor.app/Contents/MacOS/Editor")
        let helper = row(2, parent: 1, path: "/Applications/Editor.app/Contents/Helpers/Helper")
        state.apply(snapshot([root, helper]))
        let groupID = state.groups[0].id
        state.selectGroup(groupID)
        state.select(helper.identity)
        state.stop()
        state.apply(snapshot([root, row(3, parent: 1)]))
        try check(state.groupSelection == groupID && state.selectedGroup?.members.count == 2, "group selection stable across membership churn")
        try check(state.selection == nil && state.detail == nil, "exited member detail cleared")
        state.apply(snapshot([]))
        try check(state.selectedGroup == nil && state.detail == nil && state.detailError != nil, "disappeared group explicit")
    }

    private static func movedMemberCannotKeepForeignDetail() throws {
        let state = AppState()
        let root = row(1, path: "/Applications/Editor.app/Contents/MacOS/Editor")
        let member = row(2, parent: 1)
        state.apply(snapshot([root, member]))
        state.select(member.identity)
        state.acceptDetail(FfiProcessDetail(process: member, parent: .none, children: [], ports: [], portsError: nil),
                           for: member.identity, generation: 1)
        try check(state.detail != nil, "initial member detail accepted")
        state.stop()
        state.apply(snapshot([root, row(2, parent: nil)]))
        try check(state.selectedGroup?.members.map(\.pid) == [1], "old app survives regrouping")
        try check(state.selection == nil && state.detail == nil && !state.detailLoading, "foreign member/detail discarded")
    }

    private static func row(_ pid: UInt32, parent: UInt32? = nil, start: UInt64 = 10, user: String? = "alice",
                            path: String? = nil, cpu: Float? = 20, memory: UInt64? = 1024) -> FfiProcessInfo {
        FfiProcessInfo(pid: pid, name: "process-\(pid)", user: user, isCurrentUser: user == "alice", parentPid: parent,
                       startTime: start, cpuPercent: cpu, cpuMeasured: cpu != nil, memoryBytes: memory, executablePath: path, refusal: nil)
    }
    private static func snapshot(_ rows: [FfiProcessInfo]) -> FfiProcessSnapshot {
        FfiProcessSnapshot(processes: rows, sampledAt: 100, cpuMeasured: true,
            systemUsage: FfiSystemUsage(cpuPercent: 25, cpuWarmingUp: false, memoryUsedBytes: 1000, memoryTotalBytes: 4000))
    }
}
