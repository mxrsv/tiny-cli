import Lottie
import SwiftUI

/// One-shot mark for a run that moved at least one item to the Trash. Follows the
/// motion rules in docs/DESIGN-LANGUAGE.md: it plays once, rests on its last frame,
/// and stays still under Reduce Motion. The summary text beside it carries the meaning.
struct MovedToTrashMark: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        Group {
            if let animation = Self.animation {
                LottieView(animation: animation)
                    // Main-thread engine: the in-process --snapshot renderer captures its
                    // frames; the Core Animation engine's fills come out black there.
                    .configuration(LottieConfiguration(renderingEngine: .mainThread))
                    .playbackMode(reduceMotion
                        ? .paused(at: .progress(1))
                        : .playing(.fromProgress(0, toProgress: 1, loopMode: .playOnce)))
            } else {
                Image(systemName: "trash").font(.system(size: 30)).foregroundStyle(Theme.accent)
            }
        }
        .frame(width: 56, height: 56)
        .accessibilityHidden(true)
    }

    private static let animation: LottieAnimation? = {
        let animation = LottieAnimation.named("moved-to-trash", bundle: resources)
        if animation == nil { assertionFailure("moved-to-trash.json is missing from the resource bundle") }
        return animation
    }()

    /// SwiftPM's resource bundle. Inside Tiny.app, build-native-app.sh copies it into
    /// Contents/Resources, where the generated `Bundle.module` does not look.
    private static let resources: Bundle = Bundle.main.resourceURL
        .map { $0.appendingPathComponent("TinyNative_Tiny.bundle") }
        .flatMap { Bundle(url: $0) } ?? .module
}
