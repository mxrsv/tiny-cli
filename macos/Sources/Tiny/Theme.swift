import SwiftUI

enum Theme {
    static let tile = Color(white: 0.085)
    static let raised = Color(white: 0.15)
    static let accent = Color(red: 0.48, green: 0.91, blue: 0.63)
    static let radius: CGFloat = 22
    static let gap: CGFloat = 14
    static let danger = Color(red: 1, green: 0.45, blue: 0.42)
    static func number(_ size: CGFloat) -> Font {
        .system(size: size, weight: .semibold, design: .rounded).monospacedDigit()
    }

    /// Motion tokens from docs/DESIGN-LANGUAGE.md.
    enum Motion {
        static let quick = Animation.timingCurve(0.22, 0.8, 0.24, 1, duration: 0.2)
    }
}

/// The app's button variants. Menus, alerts and pickers stay native.
struct TinyButtonStyle: ButtonStyle {
    enum Variant { case primary, secondary, destructive, ghost }
    var variant: Variant
    /// Square, label-less button with the same height as a text button.
    var icon = false

    func makeBody(configuration: Configuration) -> some View {
        Surface(configuration: configuration, variant: variant, icon: icon)
    }

    private struct Surface: View {
        let configuration: Configuration
        let variant: Variant
        let icon: Bool
        @Environment(\.isEnabled) private var enabled
        @Environment(\.controlSize) private var controlSize
        @Environment(\.accessibilityReduceMotion) private var reduceMotion
        @State private var hovering = false

        var body: some View {
            let small = controlSize == .small || controlSize == .mini
            let height: CGFloat = small ? 26 : 30
            configuration.label
                .font(.system(size: small ? 11.5 : 12.5, weight: .semibold))
                .lineLimit(1)
                .foregroundStyle(foreground)
                .padding(.horizontal, icon ? 0 : (small ? 11 : 14))
                .frame(width: icon ? height : nil, height: height)
                .background(fill, in: Capsule())
                .overlay(Capsule().strokeBorder(.white.opacity(variant == .secondary ? 0.07 : 0)))
                .contentShape(Capsule())
                .opacity(enabled ? 1 : 0.4)
                .scaleEffect(configuration.isPressed && !reduceMotion ? 0.96 : 1)
                .onHover { hovering = $0 && enabled }
                .animation(Theme.Motion.quick, value: configuration.isPressed)
                .animation(Theme.Motion.quick, value: hovering)
        }

        private var foreground: Color {
            switch variant {
            case .primary: return .black.opacity(0.85)
            case .secondary: return .primary
            case .destructive: return Theme.danger
            case .ghost: return hovering ? .primary : .secondary
            }
        }

        private var fill: Color {
            let lift = (hovering ? 1 : 0) + (configuration.isPressed ? 1 : 0)
            switch variant {
            case .primary: return Theme.accent.opacity(1 - Double(lift) * 0.12)
            case .secondary: return .white.opacity(0.08 + Double(lift) * 0.05)
            case .destructive: return Theme.danger.opacity(0.13 + Double(lift) * 0.07)
            case .ghost: return .white.opacity(Double(lift) * 0.07)
            }
        }
    }
}

extension ButtonStyle where Self == TinyButtonStyle {
    static func tiny(_ variant: TinyButtonStyle.Variant, icon: Bool = false) -> TinyButtonStyle {
        TinyButtonStyle(variant: variant, icon: icon)
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
                        .tracking(1).foregroundStyle(.secondary).lineLimit(1)
                    Text(value).font(Theme.number(42)).lineLimit(1).minimumScaleFactor(0.6)
                        .contentTransition(.numericText())
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
