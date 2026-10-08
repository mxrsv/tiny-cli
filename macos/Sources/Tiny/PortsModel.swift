import Foundation
import TinyEngine

/// One owner of one port; its IPv4/IPv6 sockets share a row.
struct PortRow: Identifiable, Equatable {
    let port: UInt16
    let pid: UInt32
    let addresses: [String]
    let process: FfiProcessInfo?
    /// `nil` only when Quit / Force Quit may be offered.
    let blockedReason: String?
    var id: String { "\(port):\(pid)" }
    var ownerName: String { process?.name ?? "PID \(pid)" }
}

/// What the Ports tile shows. An empty list is never "no ports in use".
enum PortsDisplay: Equatable {
    case loading
    case failed(String)
    case empty(caveat: String)
    case rows([PortRow], caveat: String)

    static func make(listeners: FfiListeners?, error: String?) -> PortsDisplay {
        if let error { return .failed(error) }
        guard let listeners else { return .loading }
        let rows = rows(listeners.listeners)
        return rows.isEmpty ? .empty(caveat: listeners.visibilityCaveat) : .rows(rows, caveat: listeners.visibilityCaveat)
    }

    static func rows(_ listeners: [FfiListener]) -> [PortRow] {
        var order: [String] = []
        var grouped: [String: [FfiListener]] = [:]
        for listener in listeners {
            let key = "\(listener.port):\(listener.pid)"
            if grouped[key] == nil { order.append(key) }
            grouped[key, default: []].append(listener)
        }
        return order.compactMap { key in
            guard let entries = grouped[key], let first = entries.first else { return nil }
            return PortRow(port: first.port, pid: first.pid, addresses: entries.map(\.address),
                           process: first.process, blockedReason: blockedReason(first))
        }
    }

    static func blockedReason(_ listener: FfiListener) -> String? {
        if let refusal = listener.refusal { return ActionCopy.refusal(refusal) }
        if listener.process == nil { return "Not in the current process sample. Refresh to look again." }
        return listener.actionable ? nil : "Tiny cannot act on this owner."
    }
}
