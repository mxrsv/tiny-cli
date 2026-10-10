# Landing page for the tiny Mac app

Status: Requirements approved 2026-10-08; Phase 0 specimens ready for the direction choice ([plan](../plans/2026-10-09-landing-page.md)).
Started: 2026-10-08

## Goal and approved decisions

Build a marketing site for the native Mac app. The Rust `tiny` CLI and
`tiny-core` are presented as the engine behind the app, not as the product the
page sells. The user approved this scope on 2026-10-08 after an intent
interview.

- **Sections:** Hero, Features, Pricing and a Docs quick-start on the landing
  page, plus a separate `/docs` page covering installation, Full Disk Access,
  Clean, Processes and the CLI.
- **Style:** native, in the manner of Apple's macOS product page: light
  background, Apple-widget tiles with glass/material depth. Motion is lighter
  than Apple's own pages, with one exception approved on 2026-10-09: the hero
  product sits on a MacBook with a notch, and on wide screens it pins while its
  display plays three steps (processes → scan → clean to the Trash) as the user
  scrolls. Phones and reduced-motion users get the same three steps as a static
  sequence.
- **Pricing:** Free + Pro, open-core, Pro billed as a subscription. This
  replaces the earlier "free, no subscription" position. The Pro feature list,
  Free tier limits and prices are not decided; the page shows them as drafts
  until they are.
- **Stack and location:** Next.js in `site/` of this repository, with GSAP and
  Lenis for restrained motion. three.js is added only if a reviewed focal needs
  it.
- **Calls to action:** primary "Get early access" (email waitlist) while the
  app is unreleased; secondary "View source on GitHub". The user delegated this
  choice on 2026-10-08; the page sells the app, so the engine/CLI link stays
  secondary.
- **Copy:** English.
- **Review surfaces:** three throwaway static HTML hero specimens for the
  direction choice, then a demo route inside `site/` (excluded from production)
  for typography, glass, composition and motion review.

Feature copy must match what the app actually does at publish time. The
[native boundary spec](2026-10-07-swiftui-cli-boundary.md) and
[MVP spec](2026-10-06-processes-clean-mvp.md)
describe planned behavior, not shipped behavior; planned capabilities are
labelled as such.

**Out of scope:** app and CLI code, real payments or subscription handling,
deployment and domain, localization.

## Typography standard

The contract below is locked; the values are drafts until the user approves
them by eye in Phase 3. Values come from a teardown of Apple's production CSS
(apple.com/macos and apple.com/macbook-pro, fetched 2026-10-08), because the
page uses the same system font on Apple devices.

### Families

- **Sans:** `-apple-system, BlinkMacSystemFont`, then self-hosted Inter, then
  `sans-serif`. Apple devices render SF Pro (with automatic Text/Display optical
  sizes); other platforms render Inter. SF Pro is never self-hosted: its license
  does not permit webfont use.
- **Mono (CLI commands only):** `ui-monospace, "SF Mono"`, then self-hosted
  JetBrains Mono, then `monospace`.
- No other families.

### Named steps

Only these steps exist. Each step bundles size, leading, weight and tracking.
Responsive changes are discrete at Apple's breakpoints (≤1068 px and ≤734 px).
Cells give size in px / unitless leading.

| Step       | ≥1069 px    | 735–1068 px | ≤734 px     | Weight | Use                                 |
| ---------- | ----------- | ----------- | ----------- | ------ | ----------------------------------- |
| `display`  | 80 / 1.05   | 64 / 1.0625 | 48 / 1.0835 | 600    | Hero headline, once per page        |
| `headline` | 48 / 1.0835 | 40 / 1.1    | 32 / 1.125  | 600    | Section headlines                   |
| `title`    | 28 / 1.1429 | 24 / 1.1667 | 21 / 1.1905 | 600    | Widget, card and plan titles        |
| `lead`     | 21 / 1.381  | 19 / 1.4211 | 17 / 1.4706 | 600    | Eyebrows and intro copy             |
| `body`     | 17 / 1.4706 | 17 / 1.4706 | 17 / 1.4706 | 400    | Paragraphs                          |
| `callout`  | 14 / 1.4286 | 14 / 1.4286 | 14 / 1.4286 | 400    | Widget labels, navigation, captions |
| `footnote` | 12 / 1.3334 | 12 / 1.3334 | 12 / 1.3334 | 400    | Legal text, pricing footnotes       |
| `code`     | 14 / 1.4286 | 14 / 1.4286 | 14 / 1.4286 | 400    | CLI commands in the mono family     |

Tracking follows the size actually rendered (SF Pro switches from Text to
Display optics near 20 px, so the sign flips):

| Size (px) | 80       | 64       | 48       | 40  | 32      | 28      | 24      | 21      | 19      | 17       | 14       | 12      |
| --------- | -------- | -------- | -------- | --- | ------- | ------- | ------- | ------- | ------- | -------- | -------- | ------- |
| Tracking  | -0.015em | -0.009em | -0.003em | 0   | 0.004em | 0.007em | 0.009em | 0.011em | 0.012em | -0.022em | -0.016em | -0.01em |

`code` uses tracking 0. These tracking values are calibrated for SF; Inter may
need its own overrides, decided in the Treatment review (LP6). A need outside
these bundles becomes a new reviewed step, not a local override. Numerals in
widgets and pricing use tabular figures (`font-variant-numeric: tabular-nums`)
on an existing step.

### Enforcement

- One token file defines the families and steps; components apply type only
  through named steps (classes or a text primitive).
- The framework's default type scale is disabled, so `text-sm`-style defaults
  do not exist.
- A lint gate in `site/` fails on raw `font-size`, `line-height`,
  `letter-spacing`, `font-weight` or `font-family` values, and on arbitrary
  type utilities, anywhere outside the token file.
- Only weights 400 and 600 are allowed.

## Acceptance criteria

- **LP1:** Given macOS or iOS in Safari and Chrome, when the page renders, then
  text uses SF Pro and Inter is not downloaded. Given Windows or Android, then
  Inter renders instead of Segoe UI or Roboto. CLI commands render in the mono
  stack, and the resolved mono family is checked in Safari and Chrome on macOS.
- **LP2:** Given any component, when the type lint gate runs, then every type
  property resolves to a named step or family, and a raw value outside the
  token file fails the gate.
- **LP3:** Given the landing page and `/docs`, when read, then every section
  listed above exists with real copy (no placeholder text). Feature claims match
  the app at publish time, planned items are labelled, and undecided pricing is
  visibly marked as a draft.
- **LP4:** Given widget tiles on glass, when rendered over the background
  behind them, then text meets WCAG AA contrast at every breakpoint.
- **LP5:** Given `prefers-reduced-motion: reduce`, when the page loads and
  scrolls, then non-essential motion is disabled and no content depends on an
  animation finishing.
- **LP6:** Given the review flow, when each phase completes, then the user has
  approved it by eye: direction specimen, tokens, composition greybox,
  treatment at ≤734 px and ≥1069 px (on an Apple device and in one non-Apple
  render using Inter), and a motion recording. A passing build does not count
  as approval.

## Open decisions

- Waitlist backend (the demo form sends nothing).
- Pro feature list, Free tier limits and prices.
- Satellite references beside the Apple macOS anchor; picked in Phase 0.
