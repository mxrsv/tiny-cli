import AppKit
import SwiftUI
import TinyEngine

struct TinyNativeApp: App {
    @State private var state = AppState()
    var body: some Scene {
        Window("Tiny Dev", id: "processes") {
            ProcessesView(state: state)
        }
        .windowStyle(.hiddenTitleBar)
        .defaultSize(width: 1240, height: 850)
        .commands { CommandGroup(replacing: .newItem) {} }
    }
}

@main enum Main {
    @MainActor static func main() async {
        let arguments = CommandLine.arguments
        do {
            if arguments.count == 2 && arguments[1] == "--smoke-test" {
                try await smokeTest()
            } else if arguments.count == 3 && arguments[1] == "--snapshot" {
                guard arguments[2].hasPrefix("/"), arguments[2].hasSuffix(".png") else {
                    throw CocoaError(.fileWriteInvalidFileName)
                }
                try await snapshot(to: URL(fileURLWithPath: arguments[2]))
            } else if arguments.count == 1 || arguments.dropFirst().allSatisfy({ $0.hasPrefix("-psn_") }) {
                NSApplication.shared.setActivationPolicy(.regular)
                TinyNativeApp.main()
            } else {
                throw NSError(domain: "Tiny", code: 1, userInfo: [NSLocalizedDescriptionKey:
                    "Usage: Tiny [--smoke-test | --snapshot /absolute/path.png]"])
            }
        } catch {
            FileHandle.standardError.write(Data("Tiny: \(error.localizedDescription)\n".utf8))
            exit(1)
        }
    }

    private static func smokeTest() async throws {
        let engine = Engine()
        let child = Process()
        child.executableURL = URL(fileURLWithPath: "/bin/sleep")
        child.arguments = ["30"]
        try child.run()
        defer { if child.isRunning { child.terminate(); child.waitUntilExit() } }
        let first = try await engine.list()
        let ownPID = UInt32(ProcessInfo.processInfo.processIdentifier)
        guard let own = first.processes.first(where: { $0.pid == ownPID }),
              first.processes.contains(where: { $0.pid == UInt32(child.processIdentifier) }) else {
            throw NSError(domain: "Tiny", code: 2, userInfo: [NSLocalizedDescriptionKey: "Own process or test child missing"])
        }
        let detail = try await engine.detail(own.identity)
        child.terminate()
        child.waitUntilExit()
        let after = try await engine.list()
        guard !after.processes.contains(where: { $0.pid == UInt32(child.processIdentifier) }) else {
            throw NSError(domain: "Tiny", code: 3, userInfo: [NSLocalizedDescriptionKey: "Exited test child still present"])
        }
        guard let systemCPU = after.systemUsage.cpuPercent, (0...100).contains(systemCPU),
              let used = after.systemUsage.memoryUsedBytes, let total = after.systemUsage.memoryTotalBytes,
              total > 0, used <= total else {
            throw NSError(domain: "Tiny", code: 5, userInfo: [NSLocalizedDescriptionKey: "Invalid whole-machine usage sample"])
        }
        print("PASS: \(first.processes.count) real processes; own PID \(ownPID); owned child appeared and exited; detail PID \(detail.process.pid); first CPU unmeasured \(!first.cpuMeasured)")
        print("System CPU \(systemCPU)% (all cores); RAM \(used)/\(total) bytes")
    }

    @MainActor private static func snapshot(to url: URL) async throws {
        NSApplication.shared.setActivationPolicy(.prohibited)
        let state = AppState()
        state.requestRefresh()
        await state.waitForRefresh()
        guard state.listError == nil, !state.processes.isEmpty else {
            throw NSError(domain: "Tiny", code: 4, userInfo: [NSLocalizedDescriptionKey: state.listError ?? "No process data"])
        }
        if let own = state.processes.first(where: { $0.pid == UInt32(ProcessInfo.processInfo.processIdentifier) }) {
            state.select(own.identity)
            await state.waitForRefresh()
        }
        state.requestRefresh()
        await state.waitForRefresh()
        let size = NSSize(width: 1240, height: 850)
        let host = NSHostingView(rootView: ProcessesView(state: state, startsPolling: false)
            .frame(width: size.width, height: size.height))
        let window = NSWindow(contentRect: NSRect(origin: .zero, size: size), styleMask: [.borderless],
                              backing: .buffered, defer: false)
        window.appearance = NSAppearance(named: .darkAqua)
        window.contentView = host
        host.layoutSubtreeIfNeeded()
        try await Task.sleep(for: .seconds(1))
        try withExtendedLifetime(window) { try writeSnapshot(host: host, to: url) }
        print("Rendered \(state.processes.count) live processes to \(url.path)")
    }

    @MainActor private static func writeSnapshot(host: NSView, to url: URL) throws {
        host.layoutSubtreeIfNeeded()
        guard let bitmap = host.bitmapImageRepForCachingDisplay(in: host.bounds) else { throw CocoaError(.fileWriteUnknown) }
        host.cacheDisplay(in: host.bounds, to: bitmap)
        guard let png = bitmap.representation(using: .png, properties: [:]) else { throw CocoaError(.fileWriteUnknown) }
        try png.write(to: url, options: .atomic)
    }
}
