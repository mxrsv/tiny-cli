import Foundation

/// One file per running action, written atomically and fsync'd before the action
/// starts, so it survives an abort. A file left at launch means Tiny stopped
/// mid-action and that action's outcome is unknown.
struct InFlightMarker {
    struct Entry: Equatable {
        let id: UUID
        let summary: String
    }

    private static let suffix = ".marker"
    let directory: URL

    static var defaultDirectory: URL {
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        return support.appendingPathComponent(Bundle.main.bundleIdentifier ?? "Tiny", isDirectory: true)
            .appendingPathComponent("in-flight", isDirectory: true)
    }

    var pending: [Entry] {
        let names = (try? FileManager.default.contentsOfDirectory(atPath: directory.path)) ?? []
        return names.sorted().compactMap { name in
            guard name.hasSuffix(Self.suffix), let id = UUID(uuidString: String(name.dropLast(Self.suffix.count))) else { return nil }
            let data = FileManager.default.contents(atPath: file(id).path)
            return Entry(id: id, summary: data.flatMap { String(data: $0, encoding: .utf8) } ?? "an earlier action")
        }
    }

    /// Throws when the marker cannot be made durable; the caller must not act then.
    func begin(_ id: UUID, summary: String) throws {
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let temporary = directory.appendingPathComponent(".\(id.uuidString).tmp")
        guard FileManager.default.createFile(atPath: temporary.path, contents: nil) else { throw CocoaError(.fileWriteUnknown) }
        let handle = try FileHandle(forWritingTo: temporary)
        try handle.write(contentsOf: Data(summary.utf8))
        try handle.synchronize()
        try handle.close()
        guard rename(temporary.path, file(id).path) == 0 else { throw Self.posixError() }
        let descriptor = open(directory.path, O_RDONLY)
        guard descriptor >= 0 else { throw Self.posixError() }
        defer { close(descriptor) }
        guard fsync(descriptor) == 0 else { throw Self.posixError() }
    }

    /// Removes only this action's marker, never another operation's.
    func clear(_ id: UUID) {
        // A marker that cannot be removed only causes a cautious notice at next launch.
        try? FileManager.default.removeItem(at: file(id))
    }

    private func file(_ id: UUID) -> URL { directory.appendingPathComponent(id.uuidString + Self.suffix) }

    private static func posixError() -> POSIXError { POSIXError(POSIXErrorCode(rawValue: errno) ?? .EIO) }
}
