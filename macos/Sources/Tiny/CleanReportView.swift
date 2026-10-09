import SwiftUI
import TinyEngine

/// Per-item result of one Move to Trash run. Moved bytes are not freed space.
struct CleanReportView: View {
    let report: FfiExecReport

    var body: some View {
        let sections = ReportSections(report)
        Tile {
            VStack(alignment: .leading, spacing: 14) {
                HStack(spacing: 14) {
                    if report.movedCount > 0 { MovedToTrashMark() }
                    Text(CleanCopy.summary(report)).font(.system(size: 22, weight: .semibold, design: .rounded)).monospacedDigit()
                }
                Text("Items in the Trash still use disk space until the Trash is emptied. Put Back in Finder restores them.")
                    .font(.callout).foregroundStyle(.secondary)
                if let stopped = CleanCopy.stopped(report.stopped) {
                    Label(stopped, systemImage: report.stopped == .automationDenied ? "lock.shield" : "stop.circle")
                        .font(.callout).foregroundStyle(.orange).fixedSize(horizontal: false, vertical: true)
                        .padding(12).frame(maxWidth: .infinity, alignment: .leading)
                        .background(.orange.opacity(0.10), in: RoundedRectangle(cornerRadius: 12))
                }
                ScrollView {
                    VStack(alignment: .leading, spacing: 16) {
                        section("MOVED TO TRASH", sections.moved, color: Theme.accent)
                        section("FAILED", sections.failed, color: .red)
                        section("SKIPPED", sections.skipped, color: .orange)
                        section("NOT ATTEMPTED", sections.notAttempted, color: .secondary)
                    }.frame(maxWidth: .infinity, alignment: .leading)
                }
                Text("Nothing is retried automatically. Scan again to review what is left.")
                    .font(.caption).foregroundStyle(.secondary)
            }
        }
    }

    @ViewBuilder private func section(_ title: String, _ rows: [FfiItemResult], color: Color) -> some View {
        if !rows.isEmpty {
            VStack(alignment: .leading, spacing: 6) {
                Text("\(title) · \(rows.count)").font(.system(size: 10, weight: .semibold)).tracking(0.8).foregroundStyle(color)
                ForEach(rows, id: \.candidateId) { result in
                    HStack(alignment: .firstTextBaseline, spacing: 10) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(result.path).font(.system(size: 12, design: .monospaced)).lineLimit(1).truncationMode(.middle)
                                .textSelection(.enabled)
                            if result.outcome != .movedToTrash {
                                Text(CleanCopy.outcome(result.outcome)).font(.caption).foregroundStyle(.secondary)
                                    .fixedSize(horizontal: false, vertical: true)
                            }
                        }
                        Spacer(minLength: 8)
                        Text(CleanCopy.bytes(result.sizeBytes)).monospacedDigit().foregroundStyle(.secondary)
                    }
                    .padding(8).background(.white.opacity(0.03), in: RoundedRectangle(cornerRadius: 8))
                }
            }
        }
    }
}
