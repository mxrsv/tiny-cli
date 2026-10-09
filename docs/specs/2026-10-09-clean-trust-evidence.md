---
date: 2026-10-09
status: APPROVED
---

# Spec: Clean screen that earns trust — correct selection rules, per-path evidence

**Date**: 2026-10-09 | **Status**: APPROVED 2026-10-09 | **Plan**: [2026-10-09-clean-trust-evidence](../plans/2026-10-09-clean-trust-evidence.md)

## 1. Context

A real-data audit on 2026-10-09 of the SwiftUI Clean screen found that the *execution*
layer is sound (re-check, fingerprints, overlap merge, protected paths), but several
*selection rules* offered paths that should not be offered or labelled them wrongly:

- `ios-simulators` listed `Devices/device_set.plist`, the CoreSimulator registry.
- `python-caches` treated any folder named `env`/`venv`/`.venv` next to a manifest as a
  virtualenv, and listed 1521 `__pycache__` folders inside venvs as separate items.
- `vscode` had no running-app gate.
- `downloads-old` offered recovery codes, a signing certificate and `env.txt`, and judged
  age by mtime only (a PDF opened 11 days earlier was "old").
- `user-caches` guessed the owning app from the last bundle-id segment
  (`com.microsoft.VSCode` → `VSCode`, while the process is `Code`) and offered
  `com.apple.*` daemon caches such as `CloudKit` and `com.apple.HomeKit`.
- `rust-targets` / `node-modules` / venvs called a project "idle" from manifest mtime:
  `tiny-cli/target` was flagged one day after its last commit.

The first three, plus Finder dotfiles in Downloads, are fixed on `feat/native-desktop`
(uncommitted at the time of writing). This spec covers the rest.

**Goal**: a user looking at any offered path can see *why* it is offered and *how it comes
back*, and nothing is offered on a rule that is known to misfire.

## 2. User decisions (2026-10-09)

- **Approach A**: fix the selection rules first, then show per-path evidence in the existing
  card layout; group by trust; hide empty categories. No full redesign.
- **Downloads**: stays movable to Trash. Files that look sensitive are flagged and never
  selected by a category-level toggle. (This relaxes `main`'s AGENTS.md R5 for Downloads;
  R5 is updated when the change reaches `main`.)
- **iOS Simulator devices**: only devices `xcrun simctl` reports as unavailable; refuse while
  Xcode or Simulator runs.
- **Idle signal**: idle only when the manifest AND the project's git activity are older than
  the threshold.
- **User caches**: resolve the owning app from the bundle id; drop `com.apple.*` entries.
- **Demo surface**: the real app on the user's machine, scan only; Move to Trash is never
  pressed during review.

## 3. Requirements

### Selection rules (tiny-core)

- **SR1 Idle projects.** A `target/`, `node_modules/` or venv is idle only when its manifest
  mtime is older than `idle_days` AND, when the project is inside a git work tree, the last
  commit is older than `idle_days` AND the work tree has no uncommitted changes. If git
  cannot answer within the tool timeout, the item is not offered.
- **SR2 Downloads age.** A Downloads file is old only when both mtime and its Spotlight
  last-opened date (when present) are older than `idle_days`.
- **SR3 Sensitive Downloads.** A Downloads file is flagged sensitive when its name or
  extension matches a fixed list (recovery/backup codes, keys, certificates, `.env`,
  password vaults, VPN profiles). Flagged files are still offered, never selected by the
  category toggle, and must be ticked one by one.
- **SR4 User caches.** `com.apple.*` entries are not offered. For other entries the owning
  app is resolved from the bundle id to its executable name; the running-app gate uses that
  name. When no app is found the item is offered with "owning app unknown".
- **SR5 Simulator devices.** Only devices reported unavailable by `xcrun simctl list devices
  -j` are offered; `CoreSimulator/Caches` entries stay as they are. Both are refused while
  `Xcode` or `Simulator` runs.

### Evidence contract (tiny-core → tiny-ffi → Swift)

- **EV1.** Every candidate carries a list of typed evidence facts (not free text), for
  example: manifest modified on, last commit on, work tree clean, venv marker found, last
  opened on, owning app and its state, sensitive name. Swift renders the copy.
- **EV2.** Every category carries a short "how it comes back" fact (rebuild command, app
  re-downloads, not recoverable except from Trash).
- **EV3.** The FFI change is additive; the CLI output is unchanged unless stated.

### Clean screen (SwiftUI)

- **UI1.** Tiles are grouped: "Rebuilt automatically", "Your files" (Downloads, Mail
  attachments, Screenshots), "Report only". Empty categories collapse into one line
  ("Checked, nothing found: …").
- **UI2.** Each review row shows its evidence under the path; a sensitive row shows a
  warning and is excluded from category select-all.
- **UI3.** Each tile shows its "how it comes back" line.
- **UI4.** Existing guarantees stay: review items are never pre-selected (PC-C3), the
  confirmation names the Trash and Put Back, the report lists skipped items with reasons.

## 4. Acceptance criteria

- **AC1** SR1–SR5 each have fixture tests in temp dirs that fail without the rule.
- **AC2** On the user's machine (scan only): `tiny-cli/target` and `spacevibe-bench/node_modules`
  are not offered; recovery-code files in Downloads show as sensitive and stay unticked
  after ticking the Downloads tile; no `com.apple.*` path appears under User caches;
  `com.microsoft.VSCode` is skipped while VS Code runs.
- **AC3** Every offered candidate in the real-data scan has at least one evidence fact.
- **AC4** Screenshots of the grouped screen and of the review sheet with evidence are approved
  by the user.
- **AC5** `cargo test -p tiny-ffi -p tiny-core --all-targets --locked`, clippy `-D warnings`
  and `scripts/test-native.sh` pass.

## 5. Out of scope

- Full trust-first redesign, activity log, permanent deletion from the desktop.
- Porting to `main`'s CLI-only tree (raised separately).

## 6. Open decision

- **Simulator device removal on the desktop.** Removing a device correctly is
  `xcrun simctl delete <udid>`, a permanent tool command, not a move to Trash. Desktop
  policy keeps tool commands report-only (like Docker). Proposed: split devices into their
  own category, report-only on the desktop with the `simctl` command shown; the CLI runs it.
