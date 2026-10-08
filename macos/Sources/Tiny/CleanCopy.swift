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

    static func exclusion(_ reason: FfiExclusionReason) -> String {
        switch reason {
        case .duplicate: return "Same path as another selected item; moved once."
        case .insideSelected: return "Inside another selected folder; moves with it."
        }
    }

    static func error(_ error: any Error, during operation: CleanState.Operation) -> CleanError {
        guard let error = error as? FfiError else {
            return CleanError(message: "Scan failed unexpectedly. Nothing was changed. \(error.localizedDescription)", needsRescan: false)
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
        case .InvalidInput:
            return CleanError(message: "Select at least one item that can be moved to the Trash.", needsRescan: false)
        default:
            return CleanError(message: "Cleanup failed. Nothing was moved. \(ActionCopy.describe(error))", needsRescan: operation != .scan)
        }
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
