# Design language

The visual guidance for tiny: the approved primary colour, room for each surface to evolve,
and shared motion rules. The landing type scale lives in the
[landing spec](specs/2026-10-08-landing-page.md#typography-standard).

## Primary colour and UI flexibility

Approved on 2026-10-10. This is design guidance, not a claim that the palette is already
integrated throughout the app.

- **Primary colour: smoky blue `#91AEC9`.** Use this as tiny's brand accent rather than
  choosing a new primary colour for each screen. Changing the primary colour requires
  an explicit user decision.
- Keep colours consistent within a surface and text, icons and controls readable.
  Verify contrast for the actual foreground/background pairs. Status and risk must also
  have a label or symbol; colour alone must not carry their meaning.
- Supporting colours are adjustable: backgrounds, text, borders, selection fills and
  semantic colours may evolve to suit the surface and preserve contrast. The Clean
  preview's supporting hex values are reference choices, not mandatory tokens.
- Layout, navigation placement, information hierarchy, grouping, density and component
  arrangement remain flexible. The Clean preview is a colour study, not a required
  template for other screens.
- Icon treatments are flexible too. Palette rendering for category icons, monochrome
  symbols for small actions and squircle containers for prominent accents are the
  current direction, not fixed assignments. Choose by context and keep equivalent roles
  visually consistent within each surface.

Evaluate future UI changes against their task and content. Follow the project's existing
visual review gate; approval of this primary colour does not pre-approve future layouts.

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
