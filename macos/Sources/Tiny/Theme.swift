import SwiftUI

enum Theme {
    static let tile = Color(white: 0.085)
    static let raised = Color(white: 0.15)
    static let accent = Color(red: 0.48, green: 0.91, blue: 0.63)
    static let radius: CGFloat = 22
    static let gap: CGFloat = 14
    static func number(_ size: CGFloat) -> Font {
        .system(size: size, weight: .semibold, design: .rounded).monospacedDigit()
    }
}

struct Tile<Content: View>: View {
    @ViewBuilder var content: Content
    var body: some View {
        content.padding(20)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .background(Theme.tile, in: RoundedRectangle(cornerRadius: Theme.radius))
            .overlay(RoundedRectangle(cornerRadius: Theme.radius).strokeBorder(.white.opacity(0.08)))
            .shadow(color: .black.opacity(0.28), radius: 18, y: 10)
    }
}

struct Pill: View {
    let text: String
    var tint: Color = Theme.accent
    var body: some View {
        Text(text).font(.system(size: 11, weight: .semibold))
            .padding(.horizontal, 10).padding(.vertical, 5)
            .background(tint.opacity(0.12), in: Capsule()).foregroundStyle(tint)
    }
}

struct Backdrop: View {
    var body: some View {
        ZStack {
            Color(red: 0.035, green: 0.04, blue: 0.045)
            RadialGradient(colors: [Theme.accent.opacity(0.09), .clear], center: .topLeading,
                           startRadius: 0, endRadius: 650)
            RadialGradient(colors: [Color.blue.opacity(0.10), .clear], center: .bottomTrailing,
                           startRadius: 0, endRadius: 700)
        }.ignoresSafeArea()
    }
}

struct UsageWidget: View {
    let title: String
    let value: String
    let subtitle: String
    let fraction: Double?
    let icon: String
    let tint: Color
    let state: String

    var body: some View {
        Tile {
            HStack(spacing: 24) {
                VStack(alignment: .leading, spacing: 9) {
                    Label(title, systemImage: icon).font(.system(size: 11, weight: .semibold))
                        .tracking(1).foregroundStyle(.secondary)
                    Text(value).font(Theme.number(42)).contentTransition(.numericText())
                    Text(subtitle).font(.system(size: 12)).foregroundStyle(.secondary)
                }
                Spacer(minLength: 0)
                VStack(spacing: 10) {
                    ZStack {
                        Circle().stroke(tint.opacity(0.13), lineWidth: 7)
                        if let fraction {
                            Circle().trim(from: 0, to: min(1, max(0, fraction)))
                                .stroke(tint, style: StrokeStyle(lineWidth: 7, lineCap: .round))
                                .rotationEffect(.degrees(-90))
                        }
                        Image(systemName: icon).font(.system(size: 23)).foregroundStyle(tint)
                    }.frame(width: 68, height: 68).accessibilityHidden(true)
                    Text(state).font(.system(size: 10, weight: .medium)).foregroundStyle(.secondary)
                }
            }
        }
    }
}
