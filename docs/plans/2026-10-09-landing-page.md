# Landing page for the tiny Mac app

Record: Active task plan
Started: 2026-10-09
Spec: [Landing page](../specs/2026-10-08-landing-page.md) owns the sections,
style, stack, CTAs, typography standard and LP1–LP6. This plan owns the phase
order, the tasks and the review evidence.

## Phases

Each phase ends with the user approving it by eye (LP6). A passing build does
not count.

| Phase | Review surface | Status |
| ----- | -------------- | ------ |
| 0 Direction | Three static hero specimens in `site/specimens/` | Awaiting choice |
| 1 Tokens | Next.js scaffold in `site/`, token file, type lint gate (LP2) | Not started |
| 2 Composition | Greybox of every section on the demo route | Not started |
| 3 Treatment | Glass, colour and type at ≤734 px and ≥1069 px, on a Mac and in one Inter render (LP1, LP4) | Not started |
| 4 Motion | Recording of the pinned hero story, with and without reduced motion (LP5) | Not started |

## Phase 0: direction

Built 2026-10-09. The specimens share one token file (the spec's type steps,
the design language's colour and motion tokens), a CSS MacBook with a notch,
and a stand-in for the app's Clean screen. Open the HTML files directly in
Safari; they need no build.

- **A · Quiet stage.** Closest to Apple's macOS page: centred headline on
  white, the MacBook alone on a soft blue floor, glass held back for the
  Features tiles.
- **B · Widget orbit (recommended).** Three glass widgets around the MacBook:
  free disk, memory pressure, ready for the Trash. They map onto the three
  steps of the pinned story (processes, scan, clean), so each can light up as
  its step plays. Busiest of the three; widgets stack under the device on
  phones.
- **C · Open engine.** Split hero with the MacBook bleeding off the right edge
  and a glass strip naming the current step and its equivalent `tiny` command.
  Leads with transparency but pulls the CLI closer to the pitch than the spec
  wants.

Proposed satellite references beside the Apple macOS anchor, not yet reviewed
in depth: cleanshot.com and culturedcode.com/things for light, calm Mac-app
marketing with product tiles; raycast.com only if C wins.

Findings that carry into Phase 1 whichever direction wins:

- **The primary cannot carry text on white.** `#91AEC9` is 2.31:1 on white,
  so it fills buttons and chips with `#1D1D1F` text (7.29:1). Links and small
  accents use the supporting tone `#3E5B78` (7.06:1 on white). Secondary text
  `#555C66` holds 5.3:1 or better on every light ground used.
- **Glass needs something behind it.** On a plain white page the widget
  material reads as a grey card; B tints the page with a pale field of the
  primary so blur and edge highlight are visible. LP4 contrast is measured
  against that field.
- **The device screen is unreadable on phones.** At 390 px the app UI inside
  the MacBook is about a quarter of its size. The phone fallback for the
  pinned story should crop to the app window rather than show the whole
  device.
- **Waitlist form:** the specimens' form sends nothing and says so on submit.
  The backend stays an open decision in the spec.

Pricing does not appear in a hero; its draft state is a Phase 2 concern.

### Tasks

- [x] Build the three specimens on the spec's type steps and the design
  language's tokens.
- [x] Check each at 1440 px and 390 px for horizontal overflow.
- [ ] User picks a direction and the satellite references (records the
  decision in the spec, then removes `site/specimens/`).
