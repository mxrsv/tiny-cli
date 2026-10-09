import Foundation
import TinyEngine

/// A cleanup error the screen shows; `needsRescan` disables review until a new scan.
struct CleanError: Equatable {
    let message: String
    let needsRescan: Bool
}

/// User-facing copy for cleanup. Rust decides what is safe; Swift explains it.
enum CleanCopy {
    static func bytes(_ value: UInt64) -> String {
        ByteCountFormatter.string(fromByteCount: Int64(clamping: value), countStyle: .file)
    }

    static func items(_ count: Int) -> String { "\(count) \(count == 1 ? "item" : "items")" }

    static func reportOnly(_ reason: FfiReportOnlyReason) -> String {
        switch reason {
        case .destructive: return "Permanent deletion stays in the CLI."
        case .notPerPathTrash: return "Cleanup is a tool command (e.g. Docker prune, Time Machine snapshots), so it stays in the CLI."
        case .unreliableMatch: return "Folder-name matching is unreliable here; review these in Finder."
        }
    }

    static func status(_ status: FfiCategoryStatus) -> String? {
        switch status {
        case .found: return nil
        case .appRunning(let app): return "Not scanned: \(app) is running. Quit it and scan again."
        case .unavailable(let reason): return "Unavailable: \(reason)"
        case .failed(let detail): return "Scan failed: \(detail)"
        }
    }

    // MARK: Groups and evidence

    static func groupTitle(_ group: CleanState.Group) -> String {
        switch group {
        case .rebuilt: return "Rebuilt automatically"
        case .yourFiles: return "Your files"
        case .reportOnly: return "Report only"
        }
    }

    static func groupNote(_ group: CleanState.Group) -> String {
        switch group {
        case .rebuilt: return "Caches and build output that tools or apps create again when needed."
        case .yourFiles: return "Only the Trash can bring these back. Check each path before you move it."
        case .reportOnly: return "Tiny shows these but never moves them."
        }
    }

    static func checkedEmpty(_ labels: [String]) -> String {
        "Checked, nothing found: " + labels.joined(separator: ", ")
    }

    static func comesBack(_ fact: FfiComesBack) -> String {
        switch fact {
        case .rebuild(let command): return "Comes back with “\(command)”"
        case .redownload: return "Downloaded again when needed"
        case .appRecreates: return "The app recreates it"
        case .trashOnly: return "Only the Trash can bring these back"
        case .notRecoverable: return "Permanent: cannot be restored"
        }
    }

    /// Abbreviated date without time, e.g. "4 May 2026"; fixed locale so it reads the same everywhere.
    static func date(_ unixSeconds: Int64, in timeZone: TimeZone = .current) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.calendar = Calendar(identifier: .gregorian)
        formatter.timeZone = timeZone
        formatter.dateFormat = "d MMM yyyy"
        return formatter.string(from: Date(timeIntervalSince1970: TimeInterval(unixSeconds)))
    }

    /// One fact as a short phrase. A `sensitive` fact is a warning, shown by `sensitiveReason`.
    static func evidence(_ fact: FfiEvidence, in timeZone: TimeZone = .current) -> String? {
        switch fact {
        case .modified(let at): return "Modified \(date(at, in: timeZone))"
        case .manifestModified(let file, let at): return "\(file) changed \(date(at, in: timeZone))"
        case .lastCommit(let at): return "Last commit \(date(at, in: timeZone))"
        case .workTreeClean: return "No uncommitted changes"
        case .notGitRepo: return "Not in a git repository"
        case .venvMarker: return "Has pyvenv.cfg"
        case .lastOpened(let at): return "Last opened \(date(at, in: timeZone))"
        case .owningApp(let name): return "Owner: \(name) (not running)"
        case .owningAppUnknown: return "Owner app unknown"
        case .sensitive: return nil
        }
    }

    /// The compact secondary line under a review row; nil when there is nothing to say.
    static func evidenceLine(_ facts: [FfiEvidence], in timeZone: TimeZone = .current) -> String? {
        let parts = facts.compactMap { evidence($0, in: timeZone) }
        return parts.isEmpty ? nil : parts.joined(separator: " · ")
    }

    static func sensitiveReason(_ candidate: FfiCleanCandidate) -> String? {
        for case .sensitive(let reason) in candidate.evidence { return "Looks private: \(reason)" }
        return nil
    }

    static func privateNote(_ count: Int) -> String {
        "\(count) \(count == 1 ? "path looks" : "paths look") private and \(count == 1 ? "is" : "are") never selected with this tile. Tick \(count == 1 ? "it" : "them") one by one in Review."
    }

    static func isLowerBound(_ category: FfiCleanCategory) -> Bool {
        !category.unreadable.isEmpty || category.candidates.contains { $0.unreadableEntries > 0 }
    }

    static func size(_ category: FfiCleanCategory) -> String {
        (isLowerBound(category) ? "at least " : "") + bytes(category.totalBytes)
    }

    static func refused(_ category: FfiCleanCategory) -> String? {
        guard let first = category.refused.first else { return nil }
        let count = category.refused.count
        return "\(count) \(count == 1 ? "path" : "paths") never offered: \(first.reason)"
    }

    /// Final confirmation for a preview. Cancel stays the default in `ConfirmationAlert`.
    static func confirmation(_ preview: FfiPreview) -> ConfirmationCopy {
        ConfirmationCopy(
            title: "Move \(items(preview.items.count)) (\(bytes(preview.bytesSelected))) to Trash?",
            message: "They go to the Trash, and you can put them back from Finder unless “Empty Trash automatically” is on in Finder settings. Tiny re-checks each item first and skips anything that changed.",
            confirmLabel: "Move to Trash", isDestructive: false)
    }

    static let unmovableInside = "Not moved: it contains items Tiny only reports or cannot move. Keep this folder."

    static func movesWith(_ parent: String) -> String {
        "Moves with \(URL(fileURLWithPath: parent).lastPathComponent). Untick that folder to keep this one."
    }

    static func merged(_ count: Int) -> String {
        "\(count) selected \(count == 1 ? "path is" : "paths are") the same as, or inside, another selected item; each moves once and is counted once."
    }

    static func exclusion(_ reason: FfiExclusionReason) -> String {
        switch reason {
        case .duplicate: return "Same path as another selected item; moved once."
        case .insideSelected: return "Inside another selected folder; moves with it."
        case .coversUnselectedReview(let ids):
            return "Not moved: it contains \(ids.count) review \(ids.count == 1 ? "item" : "items") you did not select. Select \(ids.count == 1 ? "it" : "them") too, or keep this folder."
        }
    }

    static func error(_ error: any Error, during operation: CleanState.Operation) -> CleanError {
        let failed = operation == .scan ? "Scan failed" : operation == .preview ? "Preview failed" : "Cleanup failed"
        guard let error = error as? FfiError else {
            return CleanError(message: "\(failed) unexpectedly. Nothing was moved. \(error.localizedDescription)", needsRescan: false)
        }
        switch error {
        case .Busy:
            return CleanError(message: "Another operation is running, so nothing was \(operation == .scan ? "scanned" : "moved"). Scan again when it finishes.",
                              needsRescan: operation != .scan)
        case .Cancelled where operation == .scan:
            return CleanError(message: "Scan cancelled. Nothing was changed.", needsRescan: false)
        case .PreviewInvalid(let detail):
            return CleanError(message: "This review is no longer valid (\(detail)). Nothing was moved; scan again.", needsRescan: true)
        case .UntrustedHome(let detail):
            return CleanError(message: "Tiny's HOME folder cannot be trusted (\(detail)), so cleanup is turned off. Nothing was scanned or moved. Open Tiny from Finder and try again.",
                              needsRescan: true)
        case .InvalidInput(let detail):
            // Rust names the item, e.g. "X is report-only" or "X could not be inspected".
            return CleanError(message: "\(failed): \(detail). Change the selection; nothing was moved.", needsRescan: false)
        default:
            return CleanError(message: "\(failed). Nothing was moved. \(ActionCopy.describe(error))", needsRescan: operation == .execute)
        }
    }

    /// A panic or bridge failure during Move to Trash: items may or may not have moved.
    static func unknownOutcome(_ summary: String) -> ActionNotice {
        ActionNotice(tone: .unknown, text: "Tiny could not confirm the result of: \(summary). Check the Trash and the paths you reviewed before scanning again; nothing is repeated automatically.")
    }

    static let expired = CleanError(message: "This review expired after 15 minutes. Nothing was moved; scan again.", needsRescan: true)

    // MARK: Report

    static func summary(_ report: FfiExecReport) -> String {
        "Moved to Trash: \(bytes(report.bytesMovedToTrash)) (\(items(Int(report.movedCount)))) of \(bytes(report.bytesSelected)) selected"
    }

    static func stopped(_ reason: FfiStopReason?) -> String? {
        switch reason {
        case nil: return nil
        case .cancelled?: return "Stopped because you cancelled. Items after that point were not attempted."
        case .automationDenied?: return automationGuidance
        }
    }

    static let automationGuidance = "macOS did not let Tiny control Finder, so the remaining items were not moved and nothing was deleted. To allow it, open System Settings → Privacy & Security → Automation, turn on Finder under Tiny Dev, then scan again."

    static func outcome(_ outcome: FfiItemOutcome) -> String {
        switch outcome {
        case .movedToTrash:
            return "Moved to Trash"
        case let .failed(reason, detail, stillPresent):
            let cause = reason == .automationDenied ? "Finder Automation was denied" : "Finder could not move it: \(detail)"
            return cause + (stillPresent ? ". It is still in place." : ". It is no longer at its path; check the Trash.")
        case let .skipped(reason, detail):
            return skip(reason) + (detail.map { " (\($0))" } ?? "")
        case .notAttempted:
            return "Not attempted: the run stopped before this item."
        }
    }

    static func skip(_ reason: FfiSkipReason) -> String {
        switch reason {
        case .reportOnly: return "Skipped: this category is report-only"
        case .unknownCategory: return "Skipped: unknown category"
        case .missing: return "Skipped: it no longer exists"
        case .symlink: return "Skipped: it became a symbolic link"
        case .changed: return "Skipped: it changed since the scan"
        case .outsideScanRoots: return "Skipped: it is outside the scanned folders"
        case .protectedPath: return "Skipped: protected location"
        case .notOnHomeVolume: return "Skipped: not on your home volume, where Trash could delete it permanently"
        case .providerGuard: return "Skipped: the category's safety rule refused it"
        case .appRunning: return "Skipped: its app is running"
        case .safetyCheckUnavailable: return "Skipped: a safety check could not run"
        }
    }
}

extension FfiPreviewExclusion {
    /// Left out because it would carry unselected review items (PC-C3).
    var blocksReview: Bool {
        if case .coversUnselectedReview = reason { return true }
        return false
    }
}

/// Report rows split by outcome, in preview order.
struct ReportSections: Equatable {
    var moved: [FfiItemResult] = []
    var failed: [FfiItemResult] = []
    var skipped: [FfiItemResult] = []
    var notAttempted: [FfiItemResult] = []

    init(_ report: FfiExecReport) {
        for result in report.results {
            switch result.outcome {
            case .movedToTrash: moved.append(result)
            case .failed: failed.append(result)
            case .skipped: skipped.append(result)
            case .notAttempted: notAttempted.append(result)
            }
        }
    }
}
