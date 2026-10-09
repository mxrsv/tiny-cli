---
date: 2026-10-10
status: APPROVED (layout option pending)
---

# Spec: Clean categories — families, an honest overview and new categories

**Date**: 2026-10-10 | **Status**: requirements approved 2026-10-10; layout option pending
the mock review | **Plan**: written once the layout is chosen | **Builds on**:
[clean trust spec](2026-10-09-clean-trust-evidence.md)

## 1. Context

The Clean screen on `feat/clean-trust` shows every category as a tile in three trust groups
(UI1 of the clean trust spec). On real data (`/tmp/tiny-clean-trust-shots/real/clean-all.png`,
2026-10-09):

- "Rebuilt automatically" is one wall of 14 tiles; the eye cannot tell where the space is.
- There is no total: only the selected bytes in the bottom bar.
- Tiles reading "Zero KB", "at least Zero KB" or "0 of 0 items selected" take a full tile.
- Categories whose tool or folder is missing disappear without a trace.

A multi-agent review on 2026-10-10 (competitors, macOS disk landscape, core audit, UX
critique; raw research kept in the session scratchpad) produced the requirements below.

## 2. User decisions (2026-10-10)

- **Families**: split categories into families by where the data comes from, so the screen
  reads at a glance.
- **Layout**: chosen after a mock of every option is reviewed. Options:
  - **A** — families as sub-sections inside the trust groups, sorted by size (keeps UI1).
  - **A + bar** — A with a storage bar split by family under the headline.
  - **B** — families as the outer sections, trust as a badge on each tile (changes UI1).
  - **C** — summary cards per family that open a family's tiles (the deferred Smart Scan
    epic).
  - No sidebar: rejected on 2026-10-07 in the SwiftUI boundary spec on `main`
    (`docs/specs/2026-10-07-swiftui-cli-boundary.md`).
- **Safety first**: SR4–SR7 of the clean trust spec land before any category is split out of
  `user-caches`.
- **AC4 of the clean trust spec** is approved in the same round as this layout.

## 3. Requirements

### Families

- **FM1.** Families: Apps & browsers, Developer tools, System & logs, Your files, Leftovers;
  AI models once its providers exist. One per-id table in tiny-core holds family, trust
  section, `comes_back` and the desktop decision, with a test that covers every id.
  AGENTS.md R1 is updated in the same change.
- **FM2.** Family and section reach Swift through additive FFI fields.
- **FM3.** No checkbox on a family or trust group. Category toggles keep their rules;
  review-risk and private-looking items are never pre-selected.
- **FM4.** Families and the categories inside them are sorted by size. A category under
  1 MB renders as a compact row, not a tile.

### Overview

- **OV1.** A headline "≈X GB you can move to the Trash": the deduplicated sum over every
  movable candidate, prefixed "≈" because sizes are apparent sizes. It never says "free up".
  Report-only categories are excluded and named.
- **OV2.** The dock shows bytes instead of the category count.
- **OV3.** A ledger accounts for every checked category: skipped because an app is running
  (with "Quit <app> & scan again", which asks first and says the selection resets), not on
  this Mac, read partially, nothing found.
- **OV4.** Every tile, report-only included, has a read-only Inspect list of its paths.
- **OV5.** If the chosen layout has a storage bar: clicking a segment scrolls to the family
  and never selects; its colours differ from the SAFE and REVIEW colours; VoiceOver reads an
  equivalent list.

### New categories

Each is its own provider registered under R1, with fixture tests in temp dirs (R6). A path
split out of `user-caches` is excluded from it through roots the new provider declares, so
nothing is counted or selected twice.

| Category | Paths | Comes back | Desktop |
| --- | --- | --- | --- |
| `browser-automation` | `~/Library/Caches/{ms-playwright,ms-playwright-mcp,Cypress}`, `~/.cache/{puppeteer,selenium}` | browser install command | Trash |
| `swift-packages` | `~/Library/Caches/org.swift.swiftpm`, `~/Library/org.swift.swiftpm`, CocoaPods and Carthage caches | re-resolved | Trash, gated on Xcode |
| `js-tool-caches` | `~/Library/Caches/{electron,node-gyp,deno}`, `~/.bun/install/cache` | re-downloaded | Trash |
| `homebrew-cache` | `~/Library/Caches/Homebrew` | re-downloaded | Trash; old kegs report-only (`brew cleanup`) |
| `browser-profiles` | `Cache` and `Code Cache` inside each Chrome, Brave, Edge and Arc profile | browser recreates | Trash, gated on the browser; never cookies, history, Local Storage or IndexedDB |
| `electron-app-caches` | allow-listed cache folders under `~/Library/Application Support/<App>/` | app recreates | Trash; never `globalStorage` |
| `device-updates` | `~/Library/iTunes/{iPhone,iPad} Software Updates/*.ipsw` | re-downloaded | Trash per file |
| `macos-installers` | `/Applications/Install macOS *.app` | App Store | Trash; refused with a reason when admin rights are needed |
| `ai-models` | `~/.ollama/models`, `~/.lmstudio/models`, `~/.cache/huggingface/hub` | slow re-download | size only at first; Ollama report-only (`ollama rm`) |

- **NC1.** Not providers, because no safe per-path action exists: iOS device backups, VM
  disks (`Docker.raw`, OrbStack), Messages attachments. They may appear in an information
  panel only.

## 4. Acceptance criteria

- **AC1** The mock review picks a layout; it is recorded in §2.
- **AC2** The per-id table test fails for an id missing family, section, `comes_back` or
  desktop decision.
- **AC3** A fixture with overlapping candidates shows the headline equals the deduplicated
  sum.
- **AC4** Real-data scan screenshots of the chosen layout are approved by the user (with the
  clean trust AC4); Move to Trash is never pressed.
- **AC5** Each new provider has temp-dir fixture tests, and no path is offered by two
  providers.
- **AC6** `cargo test -p tiny-ffi -p tiny-core --all-targets --locked`, clippy `-D warnings`
  and `scripts/test-native.sh` pass.

## 5. Out of scope

- Smart Scan and Health score, Space Lens, undo and history, schedules.
- An ignore list shared by the CLI and the app (needs its own spec).
- Duplicates, large files and iCloud eviction in Clean (R5); stripping languages or
  architectures from apps (breaks code signatures).
- Showing CLI commands on the desktop (deferred by the SwiftUI boundary spec).
