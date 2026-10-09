# Plan: Clean trust — selection rules and per-path evidence

**Spec**: [2026-10-09-clean-trust-evidence](../specs/2026-10-09-clean-trust-evidence.md) |
**Checkout**: `~/.codex/worktrees/tiny-pr-1/tiny-cli`, branch `feat/native-desktop` |
**Status**: approved 2026-10-09; T5 unblocked and T8–T10 added 2026-10-10

## Tasks

| # | Task | Covers | Check |
|---|------|--------|-------|
| T0 | Commit the audit fixes already in the tree (simulator plist, `pyvenv.cfg`, venv `__pycache__`, VS Code gate, Downloads dotfiles) | context | done: 216 + 38 tests, clippy, fmt, before/after dry run |
| T1 | Typed evidence on `CleanItem` + category "comes back" fact; expose through `FfiCleanCandidate` / `FfiCleanCategory`; regenerate bindings | EV1–EV3 | core + ffi tests; CLI dry run unchanged |
| T2 | Git-aware idle check shared by rust-targets, node-modules, python venvs (`git log -1 --format=%ct`, `git status --porcelain` through the bounded runner, MockRunner in tests) | SR1, AC1 | fixtures: fresh commit, dirty tree, no repo, git timeout |
| T3 | Downloads: last-opened date via one batched `mdls` call; sensitive-name list | SR2, SR3 | fixtures + MockRunner |
| T4 | User caches: drop `com.apple.*`; bundle id → executable via app bundle `Info.plist` lookup | SR4 | fixtures; real `com.microsoft.VSCode` → `Code` |
| T5 | New destructive `simulator-devices` from `simctl list devices -j` (unavailable only), CLI `simctl delete <udid>` per item, desktop report-only; `ios-simulators` keeps `Caches` only; gate Xcode + Simulator | SR5 | MockRunner JSON fixtures |
| T6 | Swift: grouped tiles, collapsed empty line, evidence rows, sensitive rows excluded from tile toggle, "comes back" line | UI1–UI4 | `scripts/test-native.sh`; CleanTests for toggle rule |
| T7 | Real-data scan screenshots for review; never press Move to Trash | AC2–AC4 | user approval, with the clean-categories layout |
| T8 | Docker: `docker` = images + build cache with per-type prune; destructive `docker-volumes`; `comes_back` and desktop table updated | SR6 | MockRunner records the commands run |
| T9 | App orphans: CLI `execute` refuses every item with a reason | SR7 | temp-dir item survives Trash and Hard delete |
| T10 | User caches: vendor-folder gate table; Apple folders without the prefix dropped | SR4 | fixtures + MockChecker with `Google Chrome` running |

One commit per task; T1 lands before T2–T6 use it.

## Waves (parallel build)

- **Wave 0, serial:** T0, then T1 as a contract on `feat/clean-trust`: `Evidence` and
  `ComesBack` types, `ComesBack` for every provider, runner-taking constructors for the
  idle and Downloads providers, an empty `project_activity` module, the FFI records.
  After this no wave-1 task needs `providers/mod.rs` or `types.rs`.
- **Wave 1, parallel, one worktree + branch + cloned `target/` each:**
  A = T2 (`project_activity.rs`, `rust_targets.rs`, `node_modules.rs`, `python_caches.rs`),
  B = T3 (`downloads_old.rs`), C = T4 (`user_caches.rs`, one test in `mod.rs`),
  E = T6 (`CleanView`, `CleanReviewSheet`, `CleanCopy`, `CleanState`, `CleanTests`).
- **Wave 2, serial:** merge A/B/C/E into `feat/clean-trust`, full gate, T7 real-data
  scan. T5 joins once the simulator decision is made. Merging into `feat/native-desktop`
  needs the user's go-ahead: another session edits that checkout.

## Progress

- 2026-10-09: audit and T0 changes made, not committed. Spec drafted.
- 2026-10-09: T0 committed on `feat/native-desktop` (`c5e313b`), docs (`9979650`).
  Wave 0 contract on `feat/clean-trust` (`1a28d55`): core/ffi 217 + 38 tests,
  clippy (incl. `tiny`), fmt, `test-native.sh` 34 checks pass. Wave 1 started in
  worktrees under `~/Documents/Development/Personal/tiny-cli-worktrees/`
  (`idle-git`, `downloads`, `user-caches`, `swift-ui`), each with an APFS clone of
  `target/` (`cp -c -R`, ~5 s); a cold `cargo test -p tiny-core` there took 7.8 s.

- 2026-10-09: wave 1 merged into `feat/clean-trust`: T3 `f83c912`, T2 `ef7b326`,
  T4 `31c19d8` + `0efd64c` (ShipIt helpers via parent bundle id, mdfind in chunks of 50),
  T6 `3c03c2c`. Integration fixes: checked discovery keeps empty categories so the
  "Checked, nothing found" line lists them; a tile with unreadable or refused paths is
  not collapsed; `clean`/`review` snapshot scenes use real read-only discovery again,
  `review-fixture` shows the fixture evidence; inclusion reasons match the new rules.
  Gate: core/ffi 255 + 38 tests, clippy incl. `tiny`, fmt, `test-native.sh` 34 checks.
  T7 real-data scan (scan only): `tiny-cli/target` and every node_modules not offered,
  no `com.apple.*` under User caches, VS Code categories skipped while `Code` runs,
  Downloads 58 items with 5 flagged private; screenshots in `/tmp/tiny-clean-trust-shots/real/`.

- 2026-10-10: decisions in spec §2a. The Docker and app-orphans fixes landed on `main`
  first (`1baeb67`, `8b4842a`: 145 + 3 tests pass; clippy fails only on six pre-existing
  `sort_by`/division lints in `focus.rs`, `scan/` and `uninstall.rs`). T5, T8–T10 started
  on `feat/clean-trust`.

## Handoff

- Open: whether a dirty work tree with only old edits still blocks the idle rule (SR1
  follow-up).
- AC4 pending: approved in one round with the layout from the
  [clean categories spec](../specs/2026-10-10-clean-categories.md). Merge into `feat/native-desktop` needs the
  user's go-ahead; another session has uncommitted Clean UI changes there.
