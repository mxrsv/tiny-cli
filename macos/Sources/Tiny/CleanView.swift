import SwiftUI
import TinyEngine

/// The Clean destination: category tiles, selection and the path into review.
struct CleanView: View {
    @Bindable var clean: CleanState
    let actions: ActionState

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.gap) {
            if let error = clean.error {
                Label(error.message, systemImage: "exclamationmark.triangle.fill")
                    .font(.callout).foregroundStyle(.orange).textSelection(.enabled)
                    .fixedSize(horizontal: false, vertical: true)
                    .padding(12).frame(maxWidth: .infinity, alignment: .leading)
                    .background(.orange.opacity(0.10), in: RoundedRectangle(cornerRadius: 14))
            }
            content.frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        }
        .sheet(isPresented: $clean.reviewing) { CleanReviewSheet(clean: clean, actions: actions) }
    }

    @ViewBuilder private var content: some View {
        switch clean.phase {
        case .idle:
            placeholder("sparkles", "Find caches you can move to the Trash",
                        "Tiny scans developer caches, logs and leftovers, read-only. Nothing moves until you review and confirm.")
        case .scanning:
            progressTile("Scanning cleanup categories…")
        case .executing:
            progressTile("Moving items to the Trash…")
        case .report:
            if let report = clean.report { CleanReportView(report: report) }
        case .ready:
            if clean.categories.isEmpty {
                placeholder("checkmark.circle", "No cleanup categories found", "Scan again later; nothing was changed.")
            } else {
                VStack(spacing: Theme.gap) {
                    tiles
                    selectionBar
                }
            }
        }
    }

    private var tiles: some View {
        ScrollView {
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 300), spacing: Theme.gap)], spacing: Theme.gap) {
                ForEach(clean.categories, id: \.id) { CleanTile(category: $0, clean: clean) }
            }.padding(.bottom, 4)
        }
    }

    private var selectionBar: some View {
        HStack(spacing: 12) {
            Image(systemName: "trash").foregroundStyle(Theme.accent)
            Text("\(CleanCopy.items(clean.effectiveSelection.count)) selected · \(CleanCopy.bytes(clean.selectedBytes)) measured at scan")
                .monospacedDigit()
            Text("Review-risk items are never selected for you.").font(.caption).foregroundStyle(.secondary)
            Spacer()
            Button("Review…") { clean.openReview() }
                .buttonStyle(.borderedProminent).disabled(!clean.canReview)
                .help(clean.canReview ? "Review each path before anything moves" : "Select items, or scan again")
        }
        .padding(.horizontal, 18).padding(.vertical, 12)
        .background(Theme.tile, in: RoundedRectangle(cornerRadius: 16))
        .overlay(RoundedRectangle(cornerRadius: 16).strokeBorder(.white.opacity(0.08)))
    }

    private func progressTile(_ title: String) -> some View {
        Tile {
            VStack(alignment: .leading, spacing: 14) {
                Text(title).font(.system(size: 20, weight: .semibold, design: .rounded))
                if let progress = clean.progress, let total = progress.total, total > 0 {
                    ProgressView(value: Double(progress.completed), total: Double(total))
                    Text("\(progress.completed) of \(total) · \(progress.message)").font(.caption).foregroundStyle(.secondary)
                        .lineLimit(1).truncationMode(.middle)
                } else {
                    ProgressView().controlSize(.small)
                    Text(clean.progress?.message ?? "Starting…").font(.caption).foregroundStyle(.secondary)
                }
                Button(clean.phase == .executing ? "Stop" : "Cancel Scan") { clean.cancel() }
                Text(clean.phase == .executing ? "Stopping finishes the current item; the rest are not attempted."
                     : "Cancelling changes nothing.").font(.caption).foregroundStyle(.secondary)
            }.frame(maxWidth: 520, alignment: .leading)
        }.frame(maxHeight: 240)
    }

    private func placeholder(_ symbol: String, _ title: String, _ detail: String) -> some View {
        Tile {
            VStack(spacing: 14) {
                Image(systemName: symbol).font(.system(size: 34)).foregroundStyle(Theme.accent)
                Text(title).font(.system(size: 22, weight: .semibold, design: .rounded))
                Text(detail).foregroundStyle(.secondary).multilineTextAlignment(.center).frame(maxWidth: 460)
                Button { clean.scan() } label: { Label("Scan", systemImage: "magnifyingglass") }
                    .buttonStyle(.borderedProminent).disabled(clean.isBusy)
            }.frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }
}

/// One category: risk ring, size, why it is included, and what Tiny may do with it.
struct CleanTile: View {
    let category: FfiCleanCategory
    let clean: CleanState

    var body: some View {
        let selectable = CleanState.isSelectable(category)
        let selected = category.candidates.filter { clean.selection.contains($0.id) }.count
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .top, spacing: 12) {
                ring
                VStack(alignment: .leading, spacing: 3) {
                    Text(category.label).font(.system(size: 15, weight: .semibold)).lineLimit(2)
                    Text(riskText).font(.system(size: 10, weight: .semibold)).tracking(0.6).foregroundStyle(tint)
                }
                Spacer(minLength: 4)
                if selectable {
                    Toggle("Select \(category.label)", isOn: Binding(
                        get: { selected == category.candidates.count }, set: { _ in clean.toggle(category: category.id) }))
                        .toggleStyle(.checkbox).labelsHidden()
                }
            }
            Text(category.status == .found ? CleanCopy.size(category) : "—").font(Theme.number(24))
            Text(category.inclusionReason).font(.caption).foregroundStyle(.secondary).lineLimit(3)
                .fixedSize(horizontal: false, vertical: true)
            footer(selectable: selectable, selected: selected)
        }
        .padding(16)
        .frame(maxWidth: .infinity, minHeight: 190, alignment: .topLeading)
        .background(Theme.tile, in: RoundedRectangle(cornerRadius: 18))
        .overlay(RoundedRectangle(cornerRadius: 18).strokeBorder(selected > 0 ? tint.opacity(0.45) : .white.opacity(0.08)))
        .opacity(category.desktopAction == .moveToTrash && category.status == .found ? 1 : 0.72)
        .accessibilityElement(children: .contain)
    }

    @ViewBuilder private func footer(selectable: Bool, selected: Int) -> some View {
        if case let .reportOnly(reason) = category.desktopAction {
            note("Report only. " + CleanCopy.reportOnly(reason), symbol: "eye", color: .secondary)
        }
        if let status = CleanCopy.status(category.status) {
            note(status, symbol: "exclamationmark.circle", color: .orange)
        } else if category.desktopAction != .moveToTrash {
            EmptyView()
        } else if category.candidates.isEmpty {
            note("Nothing found", symbol: "checkmark.circle", color: .secondary)
        } else {
            Text("\(selected) of \(CleanCopy.items(category.candidates.count)) selected").font(.caption).foregroundStyle(.secondary)
        }
        if let refused = CleanCopy.refused(category) { note(refused, symbol: "lock", color: .secondary) }
        if CleanCopy.isLowerBound(category) {
            note("Some entries could not be read, so the size is a lower bound.", symbol: "questionmark.folder", color: .secondary)
        }
    }

    private func note(_ text: String, symbol: String, color: Color) -> some View {
        Label(text, systemImage: symbol).font(.caption).foregroundStyle(color).lineLimit(3)
            .fixedSize(horizontal: false, vertical: true)
    }

    private var ring: some View {
        ZStack {
            Circle().stroke(tint.opacity(0.9), lineWidth: 4)
            Image(systemName: category.desktopAction == .moveToTrash ? "trash" : "eye").font(.system(size: 13)).foregroundStyle(tint)
        }.frame(width: 34, height: 34).accessibilityHidden(true)
    }

    private var tint: Color {
        guard category.desktopAction == .moveToTrash else { return .gray }
        return category.risk == .safe ? Theme.accent : .orange
    }

    private var riskText: String {
        let risk = category.risk == .safe ? "SAFE" : category.risk == .review ? "REVIEW" : "DESTRUCTIVE"
        return category.desktopAction == .moveToTrash ? risk : "\(risk) · REPORT ONLY"
    }
}
