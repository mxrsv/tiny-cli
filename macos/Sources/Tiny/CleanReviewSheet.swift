import SwiftUI
import TinyEngine

/// Per-path review before the final confirmation. Unchecking excludes a path;
/// the preview is rebuilt by Rust from candidate IDs, never from paths.
struct CleanReviewSheet: View {
    let clean: CleanState
    let actions: ActionState
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Review items to move to the Trash").font(.system(size: 20, weight: .semibold, design: .rounded))
            Text("Uncheck anything you want to keep. Sizes were measured during the scan.")
                .font(.callout).foregroundStyle(.secondary)
            List {
                ForEach(reviewedCategories, id: \.id) { category in
                    Section(category.label) {
                        ForEach(category.candidates, id: \.id) { row($0) }
                    }
                }
            }
            .listStyle(.inset)
            .frame(minHeight: 220)
            previewSection
            HStack {
                Button("Cancel") { clean.reviewing = false }.keyboardShortcut(.cancelAction)
                Spacer()
                Button(moveTitle) { confirm() }
                    .buttonStyle(.borderedProminent)
                    .disabled(clean.preview == nil || clean.previewLoading || clean.preview?.items.isEmpty == true)
            }
        }
        .padding(22)
        .frame(width: 680, height: 600)
    }

    /// Categories with a selected item or an item that moves with one; their
    /// other items stay listed unchecked.
    private var reviewedCategories: [FfiCleanCategory] {
        let covered = clean.coveredBy
        return clean.categories.filter { category in
            category.candidates.contains { clean.selection.contains($0.id) || covered[$0.id] != nil }
        }
    }

    private func row(_ candidate: FfiCleanCandidate) -> some View {
        let parent = clean.coveredBy[candidate.id]
        return HStack(spacing: 10) {
            Toggle(candidate.path, isOn: Binding(get: { parent != nil || clean.selection.contains(candidate.id) },
                                                 set: { _ in clean.toggle(candidate: candidate.id) }))
                .toggleStyle(.checkbox).labelsHidden().disabled(parent != nil)
            VStack(alignment: .leading, spacing: 2) {
                Text(candidate.path).font(.system(size: 12, design: .monospaced)).lineLimit(1).truncationMode(.middle)
                    .help(candidate.path)
                if let parent {
                    Text(CleanCopy.movesWith(parent)).font(.caption).foregroundStyle(.secondary)
                } else if let excluded = clean.preview?.excluded.first(where: { $0.candidateId == candidate.id && $0.blocksReview }) {
                    Text(CleanCopy.exclusion(excluded.reason)).font(.caption).foregroundStyle(.orange)
                }
            }
            if candidate.risk != .safe { Pill(text: "REVIEW", tint: .orange) }
            Spacer(minLength: 8)
            Text((candidate.unreadableEntries > 0 ? "≥ " : "") + CleanCopy.bytes(candidate.sizeBytes))
                .monospacedDigit().foregroundStyle(.secondary)
        }
    }

    @ViewBuilder private var previewSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            if let error = clean.error {
                Label(error.message, systemImage: "exclamationmark.triangle.fill").font(.callout).foregroundStyle(.orange)
                    .fixedSize(horizontal: false, vertical: true).textSelection(.enabled)
            }
            if clean.previewLoading {
                HStack { ProgressView().controlSize(.small); Text("Checking the selection…").foregroundStyle(.secondary) }
            } else if let preview = clean.preview {
                Text("\(CleanCopy.items(preview.items.count)) · \(CleanCopy.bytes(preview.bytesSelected)) selected")
                    .font(.headline).monospacedDigit()
                if let expires = clean.previewExpiresAt {
                    Text("This review expires at \(expires.formatted(date: .omitted, time: .shortened)); after that, scan again.")
                        .font(.caption).foregroundStyle(.secondary)
                }
                ForEach(preview.excluded.filter(\.blocksReview).prefix(Self.excludedShown), id: \.candidateId) { excluded in
                    Label("\(excluded.path): \(CleanCopy.exclusion(excluded.reason))", systemImage: "exclamationmark.triangle")
                        .font(.caption).foregroundStyle(.orange).lineLimit(2).truncationMode(.middle).help(excluded.path)
                }
                let blocked = preview.excluded.filter(\.blocksReview).count
                if blocked > Self.excludedShown {
                    Text("\(blocked - Self.excludedShown) more folders are not moved because they contain unselected review items.")
                        .font(.caption).foregroundStyle(.orange)
                }
                let merged = preview.excluded.count - blocked
                if merged > 0 {
                    Text(CleanCopy.merged(merged)).font(.caption).foregroundStyle(.secondary)
                }
            } else {
                HStack {
                    Text(clean.error == nil ? "The selection changed." : "Change the selection to try again.").foregroundStyle(.secondary)
                    Button("Update Preview") { clean.requestPreview() }.disabled(!clean.canUpdatePreview)
                }
            }
        }
        .padding(12).frame(maxWidth: .infinity, alignment: .leading)
        .background(.white.opacity(0.04), in: RoundedRectangle(cornerRadius: 12))
    }

    private static let excludedShown = 3

    private var moveTitle: String {
        guard let preview = clean.preview else { return "Move to Trash…" }
        return "Move \(CleanCopy.items(preview.items.count)) to Trash…"
    }

    private func confirm() {
        guard let copy = clean.requestMove(), let previewId = clean.pendingConfirmation else { return }
        guard let window = ConfirmationAlert.hostWindow() else {
            clean.confirmationUnavailable(previewId)
            return
        }
        ConfirmationAlert.present(copy, on: window) { response in
            clean.finishConfirmation(ConfirmationAlert.confirms(response), previewId: previewId)
        }
    }
}
