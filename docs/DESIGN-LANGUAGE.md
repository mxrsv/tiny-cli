# Design language

The visual guidance for tiny: the approved colour system, room for each surface to evolve,
and shared motion rules. The landing type scale lives in the
[landing spec](specs/2026-10-08-landing-page.md#typography-standard).

## Colour

Approved on 2026-10-11, replacing the smoky blue `#91AEC9` approved on 2026-10-10. This is
design guidance, not a claim that the palette is already integrated in the app or on the
landing page. Changing the primary colour requires an explicit user decision.

### Brand ramp: clear blue

One OKLCH ramp at hue 250 with chroma up to 0.11. Take brand colours from these steps
instead of picking new hex values per element.

| 50 | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `#F1F8FF` | `#DDEDFF` | `#BADBFF` | `#97C6F7` | `#73ADE7` | `#5590CC` | `#3D75AC` | `#2A5B8A` | `#1C4267` | `#0B2843` |

- **Primary is step 600 `#3D75AC`.** It carries white text at 4.8:1, so it is the lightest
  step for filled buttons and links on light backgrounds.
- Step 500 fills progress, step 100 is the quiet tint for tracks, and step 900 is text on
  brand tints.
- Smoky blue `#91AEC9` survives only as a tint inside gradients. Its chroma is too low to
  pair as a flat UI colour.

### Where colour goes

- Colour appears in one or two large moments per surface, such as the landing hero glow and
  the final call to action. Everything else is neutral: canvas white or `#F5F5F5`, text
  `#0A0A0A`, secondary text `#6B6B6B` and hairlines in 8% black.
- On the landing page the main call to action is dark ink, not brand blue. In the app,
  primary buttons use step 600, following the macOS accent convention.
- Status, actions and progress never share one tint. A success tag uses a green ramp at the
  same chroma (fill `#DBF2DF`, text `#2B663B`) plus a check mark. A secondary button is white
  with a step 300 outline and step 700 text.
- Verify contrast for the actual foreground/background pairs. Status and risk must also
  have a label or symbol; colour alone must not carry their meaning.
- Supporting colours stay adjustable within these rules: backgrounds, borders, selection
  fills and semantic colours may evolve to suit the surface and preserve contrast.

### Headlines and references

- No serif display face. The landing hero pairs one headline line with a glass chip that
  shows a real command, such as `tiny clean --dry-run` from the
  [clean guide](user/clean.md).
- Learn principles from reference apps such as Supaste, CoolDock and Letra, but do not copy
  what identifies them: an italic serif second headline line, a dark-to-white sky gradient
  under a centred Apple-logo eyebrow, a floating black navigation pill, rolling-hills
  wallpaper behind every screenshot, or their exact blues.

## UI flexibility

- Layout, navigation placement, information hierarchy, grouping, density and component
  arrangement remain flexible. The Clean preview is a colour study, not a required
  template for other screens.
- Icon treatments are flexible too. Palette rendering for category icons, monochrome
  symbols for small actions and squircle containers for prominent accents are the
  current direction, not fixed assignments. Choose by context and keep equivalent roles
  visually consistent within each surface.

Evaluate future UI changes against their task and content. Follow the project's existing
visual review gate; approval of this colour system does not pre-approve future layouts.

## Motion

Motion is quiet and smooth, and every animation on either surface speaks the same language.
A user should never have to watch an animation to understand what happened.

### Principles

- **Motion explains a change of state.** It shows that something started, finished or moved
  (a scan completes, items go to the Trash). Nothing moves only to decorate.
- **One moving focus at a time.** At most one Lottie animation is on screen, and a Lottie never
  plays on the same element as an SF Symbol effect.
- **Loops only while work is running.** A loop stops the moment the work it represents ends.
- **No bounce, overshoot, rotation or particles.**
- **The landing page's pinned hero story is the one approved set-piece**
  ([landing spec](specs/2026-10-08-landing-page.md#goal-and-approved-decisions)). It still uses
  the tokens below.

### Tokens

Both surfaces use the same values: SwiftUI code as `Theme.Motion` constants (added with the
first SwiftUI-driven animation), Lottie files as their keyframe easing and timing, and the
landing page as CSS custom properties. A value outside this table needs a design review, not a local
override.

| Token      | Value                             | Use                                              |
| ---------- | --------------------------------- | ------------------------------------------------ |
| `ease`     | `cubic-bezier(0.22, 0.8, 0.24, 1)` | The only curve. SwiftUI: `.timingCurve(0.22, 0.8, 0.24, 1, duration:)` |
| `quick`    | 0.2 s                             | Hover, press, toggle                             |
| `base`     | 0.35 s                            | Content or state swap                            |
| `slow`     | 0.6 s                             | A panel entering, a completion moment            |
| `distance` | ≤ 16 pt                           | Largest translate                                |
| `scale`    | 0.96 – 1                          | The only scale range                             |
| `stagger`  | 60 – 80 ms, at most 4 elements    | Lists that appear together                       |

Nothing runs longer than 0.8 s except a progress loop. The default transition is a fade
combined with a short rise of at most `distance`.

### Lottie

- Colours come from the app theme; stroke weight matches SF Symbols at the same size.
- A one-shot animation lasts at most 1.2 s. A loop has a cycle of at least 1.5 s and low
  contrast.
- The text next to an animation carries the meaning, so the animation is hidden from
  assistive technology.

### Reduced motion

When the system asks for reduced motion, movement becomes a crossfade of at most `base`, and
a Lottie animation stays still on its final frame. No information may depend on an animation
finishing.

### Review

An animation is accepted only after it has been watched on a recording or a running build,
including with reduced motion turned on. A passing build does not count.
