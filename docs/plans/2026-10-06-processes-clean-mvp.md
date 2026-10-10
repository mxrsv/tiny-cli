# Processes + Clean desktop MVP

Record: Active task plan
Started: 2026-10-06
Documentation checkout: `/Users/kyantran/Documents/Development/Personal/tiny-cli` (`main`)
Implementation checkout: `/Users/kyantran/.codex/worktrees/tiny-pr-1/tiny-cli`, branch `feat/native-desktop` created on 2026-10-07 from PR #1 commit `fb07b4441b261543754ba2f4f4099a69a5f39e0d`; work continues on `feat/clean-trust` (`/Users/kyantran/Documents/Development/Personal/tiny-cli-worktrees/clean-trust`) since 2026-10-09

## Requirements reference and delivery scope

The [MVP spec](../specs/2026-10-06-processes-clean-mvp.md)
owns PC-P1–PC-P5, PC-C1–PC-C6, and PC-D1–PC-D3. The user reconfirmed
Processes + Clean on 2026-10-07. The approved
[native/CLI boundary spec](../specs/2026-10-07-swiftui-cli-boundary.md)
owns B1–B6 and the SwiftUI + retained Rust CLI direction, which replaced the
earlier Tauri shell choice. Do not duplicate
product requirements here or expand scope to PR #1's Smart Scan/Space Lens/undo.

## Current follow-up: actions 1–5 with parallel lanes (2026-10-08)

Requirements: [X1–X4, open O1/O2](../specs/2026-10-07-swiftui-cli-boundary.md#process-and-cleanup-actions-approved-2026-10-08),
plus the existing PC-P/PC-C/PC-D and B criteria. Task detail stays in T3c, T3d,
T4 and T5 below; this section only assigns them to waves and lanes.

**Checkout strategy.** Code cannot land on `main`: it is the old single crate
without the `tiny-core`/`tiny-ffi` workspace. Several agents in one checkout
would contend for the same `target/` lock, break each other's builds and mix
path-scoped commits. Each lane therefore gets its own worktree and short-lived
branch from `feat/native-desktop`; the orchestrator merges lanes back into
`feat/native-desktop` in wave order. Lanes need no `Agent` worktree isolation:
that option branches from the primary checkout (`main`), so lane worktrees are
created manually. Fallback if P0 is not approved: run the same tasks
sequentially in the single PR worktree.

| Lane | Branch | Worktree |
| --- | --- | --- |
| proc | `feat/native-desktop-proc` | `/Users/kyantran/.codex/worktrees/tiny-actions-proc` |
| clean | `feat/native-desktop-clean` | `/Users/kyantran/.codex/worktrees/tiny-actions-clean` |
| ui | `feat/native-desktop-ui` | `/Users/kyantran/.codex/worktrees/tiny-actions-ui` |

**Wave 0 — orchestrator, sequential (blocks all lanes).**
- [x] **P0 — commit the native slice.** `macos/`, `scripts/` and `AGENTS.md` are
  untracked and A1–A4/N1–N4 are uncommitted on `feat/native-desktop`; lanes
  branched before this commit would lack the SwiftUI app. `.gitignore` already
  covers `macos/.build/`, `macos/Generated/` and the copied `tiny_ffi.swift`.
  Done 2026-10-08: `91e63ea` (N1–N4 and A1–A4 share the same files, so one
  commit) and `1e5cc34` (`AGENTS.md`).
- [x] **P1 — contract freeze, committed on `feat/native-desktop`.** Names are
  proposals; shapes are binding for the lanes:
  - core `error.rs` and `FfiError` gain every new case once: refused (with a
    reason: self/parent, PID 0/1, protected, other user), identity changed,
    permission denied, already exited, preview invalid (expired, consumed,
    modified), `AutomationDenied`, `TrashFailed`;
  - signatures for core `processes::terminate`, `processes::port_owners`, the
    checked runner tool-resolution API owned by T4a, and the FFI session methods
    for terminate, port owners, clean discover/preview/execute; bodies are
    `todo!()`-free stubs returning `Unsupported`;
  - `crates/tiny-ffi/src/lib.rs` declares `pub mod clean;` with an empty module;
  - terminate takes the session gate as the spec requires, so a mutation during
    a clean scan returns `Busy`; Swift shows that state rather than queueing;
  - no lane adds a dependency; `Cargo.lock` is tracked on this branch.
  - Done 2026-10-08 as `b8db1a9`, reduced to the shared files only: core
    `Error::AutomationDenied`, `FfiError::{AutomationDenied, PreviewInvalid}`
    and an empty `tiny-ffi::clean`. Process refusals, identity change,
    permission denial and exit states are a `terminate` outcome enum owned by
    the proc lane, not errors. Port lookup keeps the absolute `LSOF` path, so
    the proc lane does not depend on T4a's runner changes.
- [x] **P2 — create the proc and clean worktrees/branches** from the P1
  commit, and record O1/O2 answers in the spec before lane prompts are sent.
  The ui worktree is created at the start of Wave 2 from `feat/native-desktop`
  after the proc merge; before Wave 3 the orchestrator merges the updated
  `feat/native-desktop` into the ui branch. Only the orchestrator moves lane
  bases.

**Wave 1 — parallel.**
- [x] **proc lane:** T3c terminate; X2 `port_owners` through the existing
  `lsof` runner; FFI wrappers; T3d `tiny processes` list/show/quit (and
  `--port` if O2 = CLI). Files: `crates/tiny-core/src/processes/`,
  `crates/tiny-ffi/src/processes.rs`, `crates/tiny/src/{cli.rs,main.rs,render/}`,
  new `crates/tiny/tests/processes_smoke.rs`. Gates: T3d's verify list.
  If O1 = (b), this lane also owns the Rust `osascript` app quit by bundle ID.
  The X2 non-visibility rule is checked with a fixture runner, and the
  other-user limit is recorded if no real invisible listener is available.
  Done 2026-10-08 and fast-forwarded into `feat/native-desktop` at `e037ede`
  (`3f34a04`..`e037ede`, including the `listeners` API for the Ports tile and
  five review fixes: pure PID-guard test, real+effective UID rule everywhere,
  errno mapping tests, no-`--yes` smoke tests, `IdentityUncertain` for owners
  started after the lsof probe). Gates on the merged branch: fmt/clippy clean;
  tests 162 core, 12 FFI, 19 CLI unit, 11 processes_smoke, 3 clean_smoke,
  1 scan_smoke passed; `scripts/test-native.sh` 15 passed. Residual risks:
  EPERM and other-user socket invisibility verified by fixtures only; 1 s
  start-time granularity; a GUI app slower than 2 s reports StillRunning.
- [x] **clean lane:** T4a → T4b → T4c in order, one commit per sub-task. Files:
  `crates/tiny-core/src/clean/`, `crates/tiny-core/src/runner.rs`,
  `crates/tiny-ffi/src/{clean.rs,session.rs}`. Gates: T4's verify lists. This
  is the critical path. 2026-10-08: `48dbdf3`, `f0c9922`, `ebedc61`,
  `afd6352`, `dab95df`, plus `bfd146b`, which adds a per-category
  `inclusion_reason` for PC-C1. Lane gates: 183 core, 24 FFI, 19 CLI and
  3 clean_smoke tests passed; `clean --help` and `clean --dry-run` match
  `b8db1a9`. `cargo check -p tiny-desktop` is not verified because the disk
  was full. Review fixes `74b05f5`..`861bc58`: app-orphans report-only on the
  desktop (UnreliableMatch); union of gates over the merged overlaps and over
  overlapping unselected candidates; literal `pgrep -x --`; desktop-only
  discovery refusal of `/`, `$HOME`, its ancestors and tool-printed roots
  outside `$HOME` (CLI output unchanged, execute-time ProtectedPath kept);
  NotOnHomeVolume; trusted home via `getpwuid_r`; trailing error-code parse;
  cancel before validate and before the move; probe-first validation;
  AutomationDenied returned as `Ok(report)`; preview TTL on both clocks.
  The re-review confirmed all of them. Merged with `--no-ff` as `5f2dc15`. Gates on
  the merged branch: fmt/clippy clean; tests 211 core, 35 FFI, 19 CLI,
  11 processes_smoke, 3 clean_smoke, 1 scan_smoke passed;
  `scripts/test-native.sh` 24 passed; `cargo check -p tiny-desktop --locked`
  finished cleanly (src-tauri still compiles). Out of scope, raised with the user: the
  CLI `app-orphans` provider (also on `main`) misclassifies installed apps'
  data such as VS Code `Code`.
- [x] **orchestrator, in parallel:** frontend gate with the user for X4
  (layout recorded in the spec on 2026-10-08; the Ports tile added a
  current-user listener list API to the proc lane): action
  placement on app rows and inspector, Quit/Force Quit confirmations, Free port
  lookup, Reveal/Copy, the Clean destination and its review sheet.

**Wave 2 — after the proc lane merges and the layout is approved.**
- [x] **ui lane, part 1:** regenerate bindings; wire Quit/Force Quit (per O1),
  Free port and Reveal/Copy; T5c in-flight marker and unknown-outcome notice;
  extend `scripts/test-native.sh`. If O1 = (a), verify from the signed
  Tiny Dev.app that `NSRunningApplication.terminate()` quits a disposable test
  app bundle without an Automation prompt, never a real user app. Files: `macos/Sources/Tiny/`,
  `macos/Sources/TinyEngine/Engine.swift`, `macos/Tests/`.
  - 2026-10-08 progress: commits `05f8fa6`, `91ed987`, `c9ca6b5` on
    `feat/native-desktop-ui`. O1(a) evidence: the signed Tiny Dev.app was run
    via `open -n -W ... --args --quit-test-app /tmp/tiny-o1/TinyO1Disposable.app`
    (minimal ad-hoc-signed NSApplication, bundle ID
    `com.mxrsv.tiny.o1-disposable`). Two runs printed `PASS: ...
    NSRunningApplication.terminate() → exited` (PIDs 39396, 56043). `log show`
    had zero `kTCCServiceAppleEvents` entries for `com.mxrsv.tiny.dev` and no
    -1743. A launch-disclosure tccd line also appeared when the disposable app
    was opened without Tiny, so `terminate()` did not cause it. Whether that
    disclosure was visible on screen was not observed.
    `scripts/test-native.sh`: 23 passed. Review verdict fix-then-merge; fixes
    landed in `02765cf`..`d2d74f7`: NSAlert sheet with Cancel as default
    (Return/Esc) and a notice when no sheet can be shown; Rust `refusal` on
    `FfiProcessInfo` gating app and member Quit; `--quit-test-app` compiled
    only with `--smoke-hooks` (O1 re-run PASS on that path); fsync'd
    per-operation marker files proven to survive SIGKILL; Ports probed every
    10 s; layout fixes. Fast-forwarded into `feat/native-desktop` at
    `d2d74f7`. Gates there: fmt/clippy clean; tests 162 core, 13 FFI, 19 CLI,
    11 processes_smoke, 3 clean_smoke, 1 scan_smoke passed;
    `scripts/test-native.sh` 24 passed. Not exercised end-to-end: real
    Return/Esc on the sheet (user acceptance); Safari cryptex path hypothesis.
- [x] **clean lane** continues T4b/T4c (done; see the clean lane bullet).

**Wave 3 — after the clean lane merges.**
- [x] **ui lane, part 2:** Clean destination and review sheet, progress,
  cancellation, per-item report, Automation-denied guidance; T5d bundle checks
  (`NSAppleEventsUsageDescription`, signing, debug-hook absence).
  Done 2026-10-08 in `252889d`..`9cde827`, after two review rounds. Fixes
  included: PC-C3 enforced in the Rust preview (`CoversUnselectedReview`, also
  refused in `consume_preview`) and in Swift preselection; per-item `covers`
  with locked child rows and deduplicated counts (PC-C2); tri-state category
  checkboxes; preview and execute errors shown in the sheet; rescan
  invalidating the preview; a typed `FfiError::UntrustedHome`. Execute is only
  exercised through `FakeCleanEngine`.

**Wave 4 — orchestrator.**
- [x] `review-change` per lane diff before each merge; after the last merge,
  the T6 gates on `feat/native-desktop`, install `Tiny Dev.app`, README update
  for `tiny processes`, and user acceptance of rendered flows (PC-D3).
  2026-10-08: ui fast-forwarded to `9cde827`; README/AGENTS updated in
  `80249c8`. Final gates on `feat/native-desktop` `9cde827`: fmt/clippy clean;
  tests 211 core, 38 FFI, 19 CLI, 11 processes_smoke, 3 clean_smoke,
  1 scan_smoke passed; `scripts/test-native.sh` 34 passed. Final bundle
  `macos/.build/Tiny Dev.app`: `codesign --verify --deep --strict` valid,
  `NSAppleEventsUsageDescription` present, `--quit-test-app` absent from the
  default build, installed-style `--smoke-test` PASS on 748 real processes.
  The bundle was NOT copied to `/Applications/Tiny Dev.app`, because that copy
  was running (PID 51765). Lane worktrees removed; their branches are kept.
  Remaining: user acceptance (T6, PC-D3) of Quit/Force Quit alerts (Return/Esc),
  the Ports tile, the mixed category checkbox, and a real Move to Trash on
  disposable files, including the first Finder Automation prompt and a denial.

**Lane prompt rules.** Each prompt includes: the absolute worktree path and
`cd <worktree> &&` on every command, because the shell resets to `main`; a
check that `git rev-parse --abbrev-ref HEAD` is the lane branch before each
commit; `git commit -- <paths>` with a conventional commit and a Vietnamese
description; no push, merge or rebase onto other lanes; per-package `-p` gates
with `--locked`; process tests signal only a disposable child they spawned;
clean tests use the injected fake `Trash` and temp fixtures, never Finder,
`osascript` or real user data (R6); generated bindings are not edited or
committed by Rust lanes. Lanes edit only their listed files; a needed change
elsewhere is reported to the orchestrator, not made.

Merge-conflict hotspots: `FfiError` and core `error.rs` (frozen in P1),
`tiny-ffi/src/lib.rs`, `session.rs` (clean lane only), README and this plan
(orchestrator only).

## Current follow-up: system widgets and app groups (2026-10-08)

Requirements: [G1–G6](../specs/2026-10-07-swiftui-cli-boundary.md#app-overview-and-grouping-2026-10-08).
Use the existing dirty native checkout; preserve all first-slice work. No new
branch or duplicate plan. AppKit supplies local app identity/icon metadata;
Swift groups rows for presentation; Rust remains the sole process/system sampler.

- [x] **A1 — system sampling and boundary.** Extend existing
  `crates/tiny-core/src/processes/snapshot.rs` to refresh whole-machine CPU/RAM
  on the retained sampler and include system usage in the snapshot. Update
  fixture construction in `crates/tiny-core/src/processes/mod.rs`. Extend
  `crates/tiny-ffi/src/processes.rs` with corresponding fields and executable
  path metadata already collected by core. Preserve existing CLI behavior.
- [x] **A2 — app presentation model.** Add
  `macos/Sources/Tiny/AppGroups.swift` for pure group assignment/aggregation and
  `macos/Sources/Tiny/AppCatalog.swift` for native running-app identity/icon
  resolution. Match outer app bundle and bounded same-owner ancestry, keep
  independent app boundaries, break cycles and retain unresolved processes.
  Cache local icons by app identity. Extend `AppState.swift` with stable group
  selection and existing child-process detail guards; no additional sampler.
- [x] **A3 — native widgets and grouped list.** Update `ProcessesView.swift`,
  `ProcessDetailView.swift`, `Theme.swift` and `TinyApp.swift` as needed: two
  truthful system widgets, app rows with icons/totals/counts, member drill-down,
  collapsed background section, current search/sort/pause/refresh controls.
  Extend in-process live snapshot mode to render this actual surface.
  - [x] A3a: whole-system CPU/RAM widgets.
  - [x] A3b: app rows, icons and collapsed background.
  - [x] A3c: group/member inspector and stable interaction.
- [x] **A4 — regression and delivery.** Extend existing Swift native check files,
  add `macos/Tests/TinyTests/AppGroupTests.swift`, and register new source/tests
  in `scripts/test-native.sh`. Cover nested helpers, independent child apps,
  unknown paths, cycles, partial CPU/RAM, stable group/member selection and live
  system values. Run the affected Rust tests/clippy/fmt and native harness;
  rebuild/sign, review, render, then update only the installed Tiny Dev.app.
  Update existing README and this record. User interaction acceptance is separate.

Plan review: no blockers; the grouped UI step now has three subchecks for
widgets, app list and inspector. One worker owns all implementation files.

Verification: `cargo test -p tiny-core -p tiny-ffi --all-targets --locked`,
`cargo clippy -p tiny-core -p tiny-ffi --all-targets --locked -- -D warnings`,
`cargo fmt --all --check`, `scripts/test-native.sh`,
`scripts/build-native-app.sh`, installed `Tiny --smoke-test` and `--snapshot`.
New files are in the native source/test locations already established by the
first slice and are referenced by SwiftPM or the native test script.

- [ ] **A5 — user acceptance:** inspect live widgets, app icons/groups, sorting,
  app/member selection, background expansion and pause/refresh in the installed
  app. This supersedes the first flat-list surface's pending visual review.

2026-10-08 follow-up verification:
- `cargo test -p tiny-core -p tiny-ffi --all-targets --locked`: 144 core + 8 FFI
  tests passed. Clippy (`-D warnings`), format and diff checks passed.
- `scripts/test-native.sh`: 15 checks passed (4 engine, 5 grouped state,
  6 grouping/metadata/membership checks). Old raw-list-only search tests were
  replaced with checks of the actual grouped list path.
- Review lanes (blind, edge-case, acceptance) completed. Fixed executable-path
  cache staleness exposed by app grouping: owned shell-to-sleep exec retained the
  same PID/start time; before the fix the test saw `Some("bash")` instead of
  `Some("sleep")` after 3.16 s; with `UpdateKind::Always` it passed (0.02 s latest).
  Also cleared the detail spinner on stop and foreign member details on regroup.
- Final release build after a numeric PID display correction: `Build complete!
  (4.92s)`. Native logic tests preceded this UI-only formatting change; the final
  installed render confirms PID text has no locale thousands separators.
- Replaced only the session-owned `/Applications/Tiny Dev.app` after checking
  the installed hash and stopping its verified same-user executable. Signature
  printed `valid on disk` and `satisfies its Designated Requirement`.
  Installed binary matches build SHA256
  `950e0bb0f8935f62f5ddf61417084f26c3e6c73c00e40882ffc780c2c0d72016`.
- Installed smoke test: `PASS: 636 real processes`; own child appeared/exited,
  detail matched its PID; global CPU 56.52327%, RAM 17845010432/25769803776 bytes.
  `sysctl -n hw.memsize` returned 25769803776, matching the denominator exactly.
- Final installed snapshot rendered 625 real processes / 52 app groups at
  `/tmp/tiny-app-groups.NGsg7L/apps-installed.png`, inspected at original size.
  Whole-system widgets, local app logos, grouped member counts and member
  inspector are present. No Computer Use or desktop screen capture performed.
- Generated/bundled test-hook occurrence checks returned zero; README/AGENTS
  links resolve. User hands-on acceptance remains A5; no action integration.

## Current slice: live read-only SwiftUI (2026-10-08)

Scope: [R1–R5 in the boundary spec](../specs/2026-10-07-swiftui-cli-boundary.md#first-live-native-slice-approved-2026-10-08).
Base revision: `7414007`, existing implementation checkout/branch above.
This slice advances the read-only part of T5 ahead of T3c/T3d/T4. Those tasks
remain pending. Native source does not exist yet; the scratch demo is gone, but the original
bento source was recovered from the 2026-10-07 session Write record for reference.

- [x] **N1 — native build and bridge.** New `scripts/build-tiny-ffi.sh`,
  `scripts/build-native-app.sh`, `macos/Package.swift`,
  `macos/Sources/TinyEngine/Engine.swift`; generated bindings/header/modulemap
  and static library are ignored build outputs. Use SwiftPM with a local C
  system-library target and the Rust static library because this Mac has only
  Command Line Tools, not Xcode's XCFramework packager. Discover native linker
  libraries with `cargo rustc -- --print native-static-libs`. Reuse the existing
  adapter, make list errors throwable if needed for UniFFI panic propagation.
  Blocking list/detail work stays off the main actor, with serialized sampling.
  Generate with `cargo run -p tiny-ffi --features bindgen --bin uniffi-bindgen
  -- generate --library target/release/libtiny_ffi.a --language swift --out-dir
  <ignored-output>`. SwiftPM system target `tiny_ffiFFI` owns the generated header
  and `module.modulemap`; `TinyEngine` depends on it and links the static archive
  with CoreFoundation, IOKit, objc, iconv, System, c and m (verified rustc output).
- [x] **N2 — live native interface.** New Swift files under
  `macos/Sources/Tiny/{TinyApp.swift,AppState.swift,ProcessesView.swift,ProcessDetailView.swift,Theme.swift}` for app entry, process state/list/detail and bento styling.
  Retain PID/start-time identity, discard obsolete detail responses, avoid
  overlapping refreshes, show warming-up/unavailable/stale states. Native
  controls support search, sorting, selection, pause and manual refresh.
  Render only read-only functionality; no Clean or signal operation is wired.
- [x] **N3 — tests and local bundle.** New Swift tests in
  `macos/Tests/TinyEngineTests/EngineTests.swift` and `macos/Tests/TinyTests/ProcessStateTests.swift`, run through new `scripts/test-native.sh`; cover real own/temporary process lifecycle,
  unavailable data, search/sort and selection response identity as appropriate.
  Build `Tiny Dev.app` with bundle ID `com.mxrsv.tiny.dev`, ad-hoc sign and verify.
  Install separately from `/Applications/Tiny.app` and launch it. Support an
  in-process render to a scratch PNG without screen capture or Computer Use.
- [x] **N4 — review and evidence.** Run Rust affected tests/clippy/fmt, Swift
  build/tests, bundle signature checks and inspect a live-data render. Update
  implementation README/AGENTS and this plan. Commands: `cargo test -p tiny-ffi -p tiny-core --all-targets --locked`,
  `cargo clippy -p tiny-ffi -p tiny-core --all-targets --locked -- -D warnings`,
  `cargo fmt --all --check`, `scripts/build-tiny-ffi.sh`,
  `scripts/test-native.sh`, `scripts/build-native-app.sh`, and
  `codesign --verify --deep --strict <bundle>`. User visual acceptance remains
  separate. No push, merge, release or real process termination is authorized.

New files use the existing planned macOS/script locations and are referenced
from SwiftPM, build scripts or README; temporary evidence lives in scratch.
Independent plan review (2026-10-08, iteration 2): no blockers after adding
explicit native/test paths and verification commands. Grouped N1/N2 subtasks
remain a tracking suggestion, with one worker owning the complete native slice.

The implementation checkout received a minimal delta-only AGENTS.md (absent
at the start of this slice). Tests only terminate child processes they create.

First-slice hands-on review is superseded by A5 for the current app-oriented
surface. Automated state checks and offscreen renders do not prove that gate.

2026-10-08 verification for N1–N4 (implementation base `7414007`, uncommitted):
- Environment: macOS 27.2 (26B5086k), arm64, Swift 6.3.2, rustc 1.99.0.
- `cargo test -p tiny-ffi -p tiny-core --all-targets --locked`: core 141 passed,
  FFI 8 passed; zero failures. `cargo clippy` with the same packages/targets and
  `-D warnings` passed; `cargo fmt --all --check` passed.
- `scripts/test-native.sh`: 4 engine checks + 5 state checks = 9 passed.
  Includes real owned-child lifecycle, warm-up, identity mismatch, post-probe
  identity validation, unavailable values, filtering/sorting, obsolete response
  rejection, cancellation and stale-data retention.
- `scripts/build-native-app.sh`: release build succeeded, final build 3.87 s;
  ad-hoc signature verified. Conflicting `CARGO_TARGET_DIR` and
  `CARGO_BUILD_TARGET` were tested: scripts use the explicit native target/output.
- Three review lanes completed (blind, edge-case, acceptance). Fixed stale Rust
  artifacts in the native test script, Cargo output/target overrides, and
  process identity revalidation after the port probe. Acceptance reviewer
  rechecked the identity fix; no remaining confirmed findings in this slice.
- Installed `/Applications/Tiny Dev.app` and launched through `open`; observed
  running executable PID 70042. Installed bundle verification printed
  `valid on disk` and `satisfies its Designated Requirement`.
- Installed binary, launched with cwd `/tmp`, `--smoke-test` printed:
  `PASS: 626 real processes; own PID 71462; owned child appeared and exited;
  detail PID 71462; first CPU unmeasured true`.
- Installed `--snapshot` rendered 626 real processes to
  `/tmp/tiny-live-check.InBOOE/processes-installed.png`. Inspected at original
  resolution: bento tiles, measured CPU, full column headers, inspector and
  controls visible. This is an in-process render, not a desktop screenshot.
- Generated Swift `debugpanic` occurrences: 0; bundled `debug_panic` strings: 0.
  README/AGENTS/spec/plan relative links resolve; `git diff --check` passed.
- Runtime checks did not exercise interactive keyboard/mouse flows. Those and
  user visual acceptance remain pending; no Computer Use was performed.

Toolchain adaptation: `swift test` cannot run on this Command Line Tools-only
installation because neither XCTest nor Swift Testing is available. Native
checks use a stdlib-only executable harness compiled against the same generated
engine and AppState source; failures exit nonzero. This avoids requiring Xcode
or an additional dependency for the local read-only slice.

## Approach

Reuse PR #1's extracted `crates/tiny-core` and `crates/tiny`, not the older
single-crate paths on `main`. Preserve the separate Tauri development app as a
reference until the native app is accepted; do not delete it in the demo slice.
Existing SQLite/quarantine data is outside this migration's mutation scope.

Build a sample-only SwiftUI `.app` first in the approved dark bento-grid style
recorded in the boundary spec. Tiles, rings, sparklines and the navigation dock use
SwiftUI/AppKit only; no third-party UI library. No real process/file action
occurs before the user accepts the rendered design and flows.

The GUI shows no equivalent `tiny ...` command in this MVP (user decision,
2026-10-07); no command-string builder is planned.

For production, propose `macos/Package.swift`, `macos/Sources/Tiny/` for the app,
and `macos/Sources/TinyEngine/` for the generated bindings plus a thin Swift
wrapper, because no Swift module exists in the repo. Swift files follow Swift's
PascalCase convention. A new adapter crate `crates/tiny-ffi/` depends on
`tiny-core` and exports the UniFFI API. Propose `scripts/build-tiny-ffi.sh` to
build its static library, generate bindings and package an xcframework, and
`scripts/build-native-app.sh` to build and bundle the app; these are new paths,
not claims of existing functionality. New process-domain files belong under
`crates/tiny-core/src/processes/`.

The app links `tiny-ffi` in-process as the boundary spec requires. A session
object retains trusted previews in memory; Swift sees only generated adapter
types. Keep work off the main actor, hop progress callbacks to the main actor,
and gate overlapping operations in the session object. Cancellation is
cooperative, including checks before individual mutations; never blindly replay
a cleanup after a failure. Native FDA and Finder Automation behavior for the bundled
app are required checks, not assumed fixes from SwiftUI. Development bundles use
bundle ID `com.mxrsv.tiny.dev`.

Reuse Rust process/safety primitives where verified. Revalidate process identity
and cleanup path/provider/running-app boundaries at mutation time. Keep Trash-only
desktop behavior distinct from the advanced CLI. Existing Tauri services are
reference for guards, not a reason to import SQLite/quarantine into this MVP.

## Execution environment

The earlier user request for Codex Cloud development remains historical context;
this work runs locally. Documentation commits land on `main` in the primary
checkout. T3–T6 code commits land on `feat/native-desktop` in the PR worktree
(conventional commits, one per completed slice, `git commit -- <paths>`). No
cloud dispatch, push, merge, release, or replacement of the installed
`/Applications/Tiny.app` is authorized yet. Keep native tests on this Mac; a
cloud/Linux check cannot establish FDA, Automation, Trash, ports, app
lifecycle, keyboard behavior, or visual acceptance.

The primary checkout contains pre-existing dirty planning files. Update their
owning scope pointers selectively, preserve unrelated changes and `error.log`,
and do not copy the whole dirty primary checkout into the PR worktree. The PR
worktree also contains an uncommitted local `Cargo.lock` adjustment from the
native run; T3a folds it into the first lock update and commits it. No core extraction task is marked accepted solely from inspection.

## Tasks

- [x] **T0: Record approved scope and establish the original baseline**
  - Completed survey on 2026-10-06; evidence retained below.
  - On 2026-10-07, update the existing PRD/architecture/project-context pointers
    and this plan; add the boundary spec without copying product requirements.
  - Verify: documentation links, `git diff --check`, independent plan review.

- [x] **T1: Validate the extracted core and existing CLI baseline**
  - Completed 2026-10-07 at PR checkout `fb07b44`; evidence below.
  - Existing source files: PR checkout `Cargo.toml`, `crates/tiny-core/src`,
    `crates/tiny/src/{cli.rs,main.rs,render}`, `crates/tiny/tests`.
  - Preserve current CLI names/flags, registry IDs/families, scan JSON and
    interactive confirmation behavior; investigate failures before editing.
  - Verify: `cargo test -p tiny-core -p tiny --all-targets --locked`,
    `cargo clippy -p tiny-core -p tiny --all-targets --locked -- -D warnings`,
    `cargo run -p tiny -- --help` in the PR checkout. Covers B1/PC-C6.

- [x] **T2: Build and accept the native interaction demo**
  - Accepted by the user on 2026-10-07 (bento-grid demo, rendered on this Mac).
  - Independent of T1; uses in-memory mock data only.
  - Scratch source: `/tmp/tiny-swiftui-demo.Bf8DXt/TinyPreview.swift`;
    artifact: `/tmp/tiny-swiftui-demo.Bf8DXt/Tiny Preview.app`.
  - 2026-10-07: the default-control sidebar/inspector/focus comparison was
    rejected as generic; the demo was rebuilt in the approved bento-grid style.
  - Validate search/sort, keyboard row selection, details, separate
    quit/force-quit dialogs, cleanup tile selection/preview/cancel, the
    navigation dock (no CLI command display, 2026-10-07), and stale/unavailable states.
  - Verify: `swiftc -parse-as-library -swift-version 6 -target arm64-apple-macosx14.0 TinyPreview.swift -o tiny-preview`,
    `./tiny-preview --snapshot <scratch-dir>` (in-process render, no Screen
    Recording access), bundle signature verification, launch evidence, and user
    native visual acceptance. Build checks do not establish rendered correctness.
  - The demo targets macOS 14 for `onKeyPress` and numeric text transitions; the
    production minimum macOS version is not decided by this demo.
  - No Computer Use/screen capture without current-task authorization.
    Covers B6 and the interaction-design portion of PC-D2/PC-D3.

- [ ] **T3: Add the FFI adapter, process domain and `tiny processes` CLI**
  - Depends on T1. Branch `feat/native-desktop`.
  - Existing files: root `Cargo.toml` (`members`, `default-members`), `Cargo.lock`,
    `crates/tiny-core/{Cargo.toml,src/lib.rs}`, `crates/tiny-core/src/clean/runner.rs`,
    `crates/tiny/src/{cli.rs,main.rs}`, `crates/tiny/src/render/`, `crates/tiny/tests/`.
  - New files: `crates/tiny-ffi/{Cargo.toml,src/lib.rs,src/session.rs,src/processes.rs,
    src/bin/uniffi-bindgen.rs}`, `crates/tiny-core/src/processes/{mod.rs,snapshot.rs,actions.rs}`,
    `crates/tiny/src/render/processes.rs`, `crates/tiny/tests/processes_smoke.rs`.
    Attach every new module to the existing entry points.
  - **T3a — workspace and adapter skeleton:** done 2026-10-07 (`e6f1712` on
    `feat/native-desktop`). The T3a gate is a lock-free `AtomicBool`;
    mutex-poison handling arrived with the sampler mutex in T3b.
    - Add `crates/tiny-ffi` to `members` and `default-members`.
    - `crate-type = ["staticlib", "lib"]`; the `lib` type serves Rust tests and
      the bindgen binary.
    - Run one unlocked `cargo build -p tiny-ffi` to update `Cargo.lock`. The
      resulting lock, including the pre-existing local adjustment, is committed
      with T3a; every later gate uses `--locked`.
    - Add `#[cfg(not(panic = "unwind"))] compile_error!` in `tiny-ffi`.
    - Implement:
      - the session object;
      - typed errors;
      - the cancellation token;
      - the progress callback trait;
      - an RAII operation-gate guard that releases on unwind and treats mutex
        poisoning as a typed error.
    - A `debug_panic` export exists only behind a `test-hooks` cargo feature.
    - Verify: Rust tests for typed errors, overlap rejection, cancellation, and
      gate release after a panic caught with `std::panic::catch_unwind` around a
      session method. Re-run the T1 gates, because the lock changed.
  - **T3a′ — link smoke:** done 2026-10-07 (scratch, evidence below). build `tiny-ffi` as a release staticlib, generate Swift
    bindings and link one export from a scratch `swiftc` program.
    - Linker flags come from
      `cargo rustc -p tiny-ffi --release -- --print native-static-libs`.
    - This proves the link early; it is a scratch check, not a deliverable.
  - **T3b — read-only process data:** done 2026-10-07 (`3f1146a` runner,
    `7414007` processes + adapter). Decisions made during implementation:
    - `libc` became a direct `tiny-core` dependency in T3b (for `getuid`)
      instead of T3c.
    - Read-only process queries bypass the operation gate, and only scans and
      mutations take it (spec updated).
    - The sampler mutex recovers from poisoning, because it holds no invariant
      a panic could break.
    - The runner moved to `crate::runner`; `clean::runner` re-exports it until
      T4a migrates the providers.
    - First move the bounded, fallible command runner into shared core: it times
      out, returns typed errors on spawn failure, non-zero exit or timeout, and
      lets tests inject a runner. T4a reuses it.
    - Then implement:
      - sampling (fresh vs first/unavailable CPU sample). A one-shot caller
        such as the CLI takes three refreshes separated by
        `sysinfo::MINIMUM_CPU_UPDATE_INTERVAL` (about 0.4 s). On macOS,
        `sysinfo` reports a new process's CPU as 0 until its third refresh, so
        "measured" is tracked per process (pid + start time);
      - details and parent/child tree (exited parent → unavailable);
      - listening ports via a bounded `lsof` probe with explicit errors.
  - **T3c — process actions:**
    - Send signals with `libc::kill` (`libc` is already a direct dependency
      since T3b). Map `std::io::Error::last_os_error()`:
      - EPERM → permission denied;
      - ESRCH → already exited.
    - One core entry `terminate(kind: graceful | force)`: graceful = SIGTERM,
      force = SIGKILL. Each kind is confirmed separately by the caller; both
      share the same revalidation. The adapter exposes the same entry.
    - Revalidate PID + start time + UID immediately before signaling.
    - After the signal, re-sample and report "still running" separately.
    - Refuse:
      - self and own parent;
      - PID 0/1 (launchd);
      - non-current-UID processes;
      - a named protected list (`WindowServer`, `loginwindow`, `Dock`,
        `SystemUIServer`, `Finder`).
    - No automatic SIGKILL, tree signals or elevation.
    - Accepted residual risk: `sysinfo` start time has 1 s granularity, so a
      small check→signal window remains. Mitigate by also comparing executable
      path/name.
    - Tests use a disposable child only.
  - **T3d — `tiny processes` CLI (additive, PC-P5):**
    - Public syntax approved by the user on 2026-10-07:
      - `tiny processes [--sort cpu|memory] [--limit N] [--json]` lists processes
        (default sort `cpu`);
      - `tiny processes show <PID> [--json]` shows detail, parent/children and
        ports;
      - `tiny processes quit <PID> [--force] [--yes]`: graceful by default, and
        `--force` is the separately confirmed SIGKILL.
    - Confirmation mirrors `clean --hard`:
      - `quit` prompts unless `--yes`;
      - `--force --yes` additionally requires `TINY_CONFIRM_FORCE=1`, like
        `TINY_CONFIRM_HARD`;
      - refusals (self, PID 1, protected, other UID) exit non-zero with the
        reason.
    - JSON fields (camelCase, as in `scan --json`): `pid`, `name`, `user`,
      `isCurrentUser`, `parentPid`, `startTime`, `cpuPercent` (null when not
      measured), `cpuMeasured`, `memoryBytes`, `sampledAt`. `show` adds
      `children` and `ports` (null plus `portsError` when the probe fails).
    - Dispatched in `main.rs`, rendered by `render/processes.rs`; existing
      commands and flags are unchanged (B1). Same core calls as the adapter.
    - `processes_smoke.rs` covers:
      - the list and `--json` with `cpuMeasured: true`;
      - `show` of a disposable child;
      - `quit` of a disposable child;
      - refused PID 1 and self;
      - `--force --yes` refused without `TINY_CONFIRM_FORCE`.
  - Verify:
    - `cargo test -p tiny-core -p tiny -p tiny-ffi --all-targets --locked`;
    - `cargo clippy -p tiny-core -p tiny -p tiny-ffi --all-targets --locked -- -D warnings`;
    - `cargo run -p tiny --locked -- processes --help`;
    - an exact `--help` diff against `main` for `clean`, `scan`, `sys`, `focus`
      and `uninstall`. At top level, the only allowed difference is the added
      `processes` line.
  - Covers B1–B3/B5 and PC-P1–PC-P5.

- [ ] **T4: Add cleanup discovery, trusted preview and recoverable execution**
  - Depends on T3 (session, token, gate, shared runner).
  - Existing files: `crates/tiny-core/src/clean/{discover.rs,execute.rs,fs_safe.rs,types.rs,process.rs,runner.rs}`,
    `crates/tiny-core/src/clean/providers/mod.rs` and providers using its helpers,
    and `crates/tiny/src/render/clean` regression tests.
  - New adapter module: `crates/tiny-ffi/src/clean.rs` with mirror records and
    focused tests; keep presentation out of shared core, without importing Tauri app state.
  - **T4a — checked discovery/cancellation:** add checked, cancellable core
    entry points rather than recovering lost errors in the adapter.
    - Add an operation context with the cancellation flag, threaded through
      discovery loops and provider work, not just between categories.
    - Add checked directory/size walks and a fallible running-process probe that
      uses the T3b runner.
    - Isolate provider failures: a failing `provider.discover()` becomes a typed
      per-category error entry, and other categories still return.
    - Report "tool not found" (for example docker/go absent from `PATH`) as
      unavailable instead of silently skipping.
    - The shared runner resolves a tool once to an absolute path and spawns
      that path. Lookup policy is a parameter:
      - CLI: `PATH` only, so CLI behavior is unchanged (B1);
      - app: `PATH`, then `/opt/homebrew/bin` and `/usr/local/bin`.

      Test with an injected `PATH` lacking the tool and a fixture prefix
      containing it.
    - Keep the compatibility wrappers and the existing CLI output; test that
      output stays compatible.
    - Checkpoints:
      1. checked size/directory walks and their tests;
      2. provider/discovery integration with error isolation.

      Run focused `tiny-core` tests after each checkpoint and the CLI regression
      gates after integration.
    - Verify with injected fixtures: denied reads, failed running-app probes,
      unavailable size data, cancellation mid-walk, a child-command timeout, one
      failing provider alongside healthy ones, and a missing tool.
  - **T4b — trusted preview:**
    - Store discovered candidates, previews, path fingerprints and the
      `CleanOptions` used at discovery in the session object.
    - Verify selection boundaries and forged/expired/replaced paths.
  - **T4c — checked execution (modifies `execute.rs`, `types.rs`, `providers/mod.rs`):**
    - Desktop eligibility is an explicit, deny-by-default opt-in on
      `CleanProvider` (for example `fn desktop_trash_paths(&self) -> bool { false }`).
      It is set to true only for providers whose cleanup moves each listed path
      to Trash (`execute_per_item`). The checked execute validates each item and
      then calls the `Trash` trait on that path directly; it never calls
      `provider.execute`, so no provider-specific command runs from the desktop.
    - Non-opted-in providers are report-only on the desktop, with a reason.
      Today that includes `docker`, whose execute runs a system-wide
      `docker system prune -af --volumes` even for Trash, and all Destructive
      categories.
    - Tests:
      - a docker preview is refused for execution;
      - every ID in `known_category_ids()` has a deliberate opt-in value.
    - Add a checked core execute entry taking the operation context and a
      per-item validator: fingerprint, `symlink_metadata`, root containment,
      running-app check, then the cancel check, run immediately before each item.
      Root containment includes each provider's discovery roots and
      provider-specific guards such as `is_safe_cargo_path`. These are enforced
      checks; today `is_safe_cargo_path` is only a `debug_assert!`.
    - It returns per-item results: moved, failed, skipped (with reason), not
      attempted. Keep the existing `execute` as a compatibility wrapper.
    - Introduce a `Trash` trait in core:
      - the default implementation is the current Finder/`osascript` route;
      - tests inject a fake;
      - typed `AutomationDenied` (Apple Events error -1743) and `TrashFailed`
        errors.
    - The adapter requires a confirmed preview ID and marks the preview consumed
      atomically before the first mutation, so a panic cannot enable a replay.
    - Refuse an item when a required safety probe fails. A Trash failure
      preserves the source. Cancellation
      reports completed work without claiming freed space.
    - Verify via temporary fixtures and the injected Trash:
      - empty selection and cancellation;
      - path overlap, symlinks and replacement;
      - forged, expired and consumed previews;
      - partial results;
      - Automation denial and Trash errors;
      - a panic mid-execute (preview stays consumed, gate released);
      - a docker preview refused for execution.

      Re-run the T1 gates and `cargo test -p tiny-ffi --locked`.
  - Covers B2/B3/B5 and PC-C1–PC-C6.

- [ ] **T5: Integrate the accepted UI and bundle the native app**
  - Depends on T2 visual acceptance, T3 and T4.
  - Proposed new SwiftPM executable target `Tiny`: `macos/Package.swift`,
    `macos/Sources/Tiny/{TinyApp.swift,ProcessesView.swift,CleanView.swift,
    CleanupPreviewSheet.swift,AppState.swift}`. Proposed local library target
    `TinyEngine`: generated `tiny_ffi.swift` plus `macos/Sources/TinyEngine/Engine.swift`,
    which runs blocking calls off the main actor and delivers progress on it.
    It exists to test the real bindings without launching the SwiftUI app, not
    to add an interchangeable service framework.
  - Existing file: `.gitignore` (add the generated paths below).
  - **T5a — build integration:** `scripts/build-tiny-ffi.sh`:
    - builds the arm64 release staticlib (arm64-only is a stated local limit);
    - runs bindgen in `--library` mode;
    - packages `macos/Frameworks/TinyFFI.xcframework` with headers and the
      generated modulemap renamed to `module.modulemap`, so the module name
      `tiny_ffiFFI` matches what the generated Swift imports;
    - copies `tiny_ffi.swift` into `macos/Sources/TinyEngine/`.

    `Package.swift` declares the xcframework as a `binaryTarget`, with
    `linkerSettings` for the frameworks/libraries reported by
    `--print native-static-libs` (CoreFoundation, IOKit, libobjc and others
    reported). A `--test-hooks` option builds the `test-hooks` feature for tests
    only. The generated Swift file and `macos/Frameworks/` are git-ignored.
    Verify: `scripts/build-tiny-ffi.sh`, then `swift build --package-path macos --product Tiny`.
    `swift test` starts in T5b, which creates the first test target.
  - **T5b — Swift engine tests:** add SwiftPM test target `TinyEngineTests`:
    `macos/Tests/TinyEngineTests/{EngineErrorTests.swift,EngineOperationTests.swift}`.
    Built with `test-hooks` in the release profile the app ships. Exercise:
    - typed errors;
    - `debug_panic` → panic-as-failure → unknown-outcome mapping;
    - progress delivered on the main actor;
    - cancellation and overlap rejection;
    - preview invalidation through a fresh session.

    Temporary fixtures only. Verify:
    `scripts/build-tiny-ffi.sh --test-hooks && swift test --package-path macos`.
  - **T5c — UI integration:** bind the accepted bento layout to `TinyEngine`.
    Implement:
    - focus/keyboard and confirmations;
    - denied/partial/stale/unavailable-tool states;
    - an Automation-denied state with guidance;
    - operation ownership, without duplicating Rust decisions in Swift.

    Persist an in-flight marker in `UserDefaults` before each mutation and clear
    it on any return. A marker at launch, or a `rustPanic` during execute, shows
    the unknown-outcome notice. No CLI command display (MVP decision).
  - **T5d — packaging/platform checks:** new `scripts/build-native-app.sh` always
    rebuilds the FFI without `test-hooks`, then bundles the `Tiny` app.
    - Bundle ID `com.mxrsv.tiny.dev`; `NSAppleEventsUsageDescription` in
      Info.plist; and, if hardened runtime is enabled, the
      `com.apple.security.automation.apple-events` entitlement.
    - Sign with a stable Apple Development identity when one is available.
      Otherwise sign ad hoc and accept that FDA/Automation grants reset after
      each rebuild.
    - Copy the protected-folder FDA probe logic from
      `src-tauri/src/permissions.rs` into `tiny-core` as an enum-only
      granted/required/unknown status. Explanation strings live in Swift.
      `src-tauri` stays unchanged.
    - Do not delete Tauri data or reset TCC; no extra DB is needed.
  - Verify:
    - `cargo build -p tiny -p tiny-ffi --locked`;
    - `scripts/build-native-app.sh`;
    - `codesign --verify --deep --strict <bundle>`;
    - after the build, `grep -ci debugpanic macos/Sources/TinyEngine/tiny_ffi.swift`
      prints `0`, and `strings <bundle>/Contents/MacOS/Tiny | grep -i debug_panic`
      prints nothing. `nm` alone can miss stripped symbols.
  - On the Mac (temporary fixtures only):
    - Finder launch;
    - FDA and Automation grant/deny flows;
    - provider availability compared between the Finder launch and the CLI;
    - kill the app mid-execute, relaunch, and confirm the unknown-outcome notice
      with no replay;
    - failure/cancel flows.

    Review adapter input validation and mutation paths before real-fixture
    native actions. Covers B2–B5 and PC-D1–PC-D3.

- [ ] **T6: Accept native flows and update shipped-behavior documentation**
  - Depends on T5. Existing files: `README.md` (document `tiny processes`),
    `CHANGELOG.md` if present, this plan and the boundary spec; update repo rules
    only for actual delivered locations/commands (D12).
  - Run real mutation checks only against disposable child/file fixtures;
    inspect user-approved rendered output and access-denied/unavailable states.
  - After relevant final changes, run:
    - `cargo fmt --all --check`;
    - the T3 Rust gates;
    - `cargo test -p tiny-ffi --locked`;
    - `scripts/build-tiny-ffi.sh --test-hooks && swift test --package-path macos`;
    - `scripts/build-native-app.sh`, the bundle signature checks and the
      `debug_panic` symbol-absence check.
  - Retire the prototype after its accepted layout has been implemented; remove
    only this task's experimental files. Retiring the old Tauri shell is a later
    concrete, reviewable step, not an automatic delete during the demo.
  - Developer ID/notarization, public distribution and broader roadmap stay
    outside this development slice. Complete only when all relevant PC/B gates
    and user visual/native acceptance are met.

## Verification evidence
- 2026-10-07 (T3b, `feat/native-desktop` `3f1146a` + `7414007`, `rustc` 1.99.0):
  - **Tests:** `cargo test -p tiny-core -p tiny -p tiny-ffi --all-targets --locked`
    exited 0: `tiny-core` 141, `tiny` 19, `clean_smoke` 3, `scan_smoke` 1 and
    `tiny-ffi` 8 passed; 0 failed. They cover:
    - the runner: exit code, stdout/stderr, timeout kill, missing binary,
      non-UTF-8, and the 4 KB stderr cap;
    - lsof parsing (IPv4/IPv6/wildcard, exit 1 empty means no ports, stderr
      or timeout means an error);
    - detail: exited parent, children, other-user skip with zero probe calls,
      unknown PID;
    - the unavailable-not-zero rule and sorting;
    - a disposable busy `/usr/bin/yes` child: unmeasured first, more than 1 %
      CPU once measured, gone after kill;
    - in the adapter: CPU measured only on the third list, and reads that
      bypass the gate.
  - **Clippy and format:** `-D warnings` exited 0 for the three crates and for
    `tiny-ffi --features test-hooks,bindgen`; `cargo fmt --all --check`
    passed.
  - **CLI baseline:** `--help` for the top level and the 5 subcommands is
    identical to `main`.
  - **lsof check:** a manual run against an `nc` listener printed
    `p…/f3/n127.0.0.1:54329` and exit 0; a process without a listener and PID 1
    both exit 1 with no output, which confirms the parsing rules and why the
    probe is skipped for other users.
- 2026-10-07 (T3b Swift link smoke, scratch `/tmp/claude-501/t3b-link`):
  - **CPU bug found:** a Swift 6 program on the release staticlib first showed
    every CPU as 0.0 after two samples, even for its own busy thread. The root
    cause is in `sysinfo` 0.32.1 `macos/process.rs:236-261`: a new process's
    CPU times are recorded on its second refresh and computed from the third.
  - **After the fix:** the third `processList()` showed `cpuMeasured=true`, the
    own busy thread at about 71 %, the busiest other process at about 133 %
    (per-core), 65 of 707 processes non-zero, and 232 other-user processes
    unreadable (`memoryBytes == nil`).
- 2026-10-07 (T3a, `feat/native-desktop` `e6f1712`, `rustc` 1.99.0):
  - **Lock:** one unlocked `cargo build -p tiny-ffi` updated `Cargo.lock`. It
    was committed together with the earlier local adjustment.
  - **Tests:** `cargo test -p tiny-core -p tiny -p tiny-ffi --all-targets --locked`
    exited 0: `tiny-core` 123, `tiny` 19, `clean_smoke` 3, `scan_smoke` 1 and
    `tiny-ffi` 5 passed; 0 failed. The `tiny-ffi` tests cover typed error
    mapping, progress conversion, overlap rejection, gate release after a
    caught panic, and cancellation.
  - **Clippy:** `-D warnings` exited 0 for the three crates, and for `tiny-ffi`
    with `--features test-hooks,bindgen`.
  - **Format:** `cargo fmt -p tiny-ffi --check` passed.
  - **CLI baseline:** `--help` for the top level and `clean`, `scan`, `sys`,
    `focus` and `uninstall` is identical to `main`.
- 2026-10-07 (T3a′ link smoke, scratch `/tmp/claude-501/t3a-link`):
  - **Flags:** `--print native-static-libs` reported `-framework CoreFoundation
    -lobjc -framework IOKit -liconv -lSystem -lc -lm`.
  - **Bindings:** bindgen `--library` mode works on `libtiny_ffi.a` (no cdylib
    needed).
  - **Shipped-style build:** a Swift 6 program linked the release staticlib with
    those flags, called `TinySession`/`CancellationToken`, and exited 0. The
    generated Swift has 0 `debugPanic` matches and `strings` finds 0
    `debug_panic`.
  - **`test-hooks` build:** `strings` finds 2 matches, a positive control for the
    T5d gate. `debugPanic` surfaced as `rustPanic("debug_panic test hook")`, and
    the session reported `busy=false` afterwards.
- 2026-10-07: plan review of T3–T5, iteration 1/3, returned `EXECUTABLE: Partial`
  with 6 HIGH, 8 MEDIUM and 6 LOW findings: lock/workspace gates, missing
  PC-P5 CLI, errno-less signaling, Finder Automation/Trash seam, core-level
  per-item checks, the post-abort notice, panic/gate handling, runner ordering,
  packaging, PATH, provider isolation, signing and commit destination. The user
  decided:
  - hide the CLI command from the GUI for the MVP;
  - keep Finder/`osascript` Trash;
  - dev bundle ID `com.mxrsv.tiny.dev`;
  - code on `feat/native-desktop`.

  T3–T6 were revised accordingly.
- 2026-10-07: plan review iteration 2/3: all 20 iteration-1 findings verified
  fixed, plus 10 new findings (1 HIGH, 4 MEDIUM, 5 LOW): docker prune behind
  desktop Trash, test-hooks build order, T5a `swift test` order, Homebrew
  resolution, the `tiny processes` contract and force gate, `--help` diff, FDA
  probe placement, CLI CPU sampling, the force action in core, and PRD FR39
  annotations. All 10 were revised.
- 2026-10-07: plan review iteration 3/3 returned `EXECUTABLE: Yes`, no
  blockers. All 10 iteration-2 findings were verified fixed; the trend over
  three iterations was 20 → 10 → 4 findings and HIGH 6 → 1 → 0. The remaining
  findings (1 MEDIUM, 3 LOW) were applied: deny-by-default desktop eligibility
  opt-in with enforced provider path guards, a stronger test-hooks absence
  gate, error-copy ownership in Swift, and removal of a duplicate line.
- 2026-10-07: demo command bar removed per the MVP decision; Swift 6 build
  clean, `--snapshot` PNGs inspected, bundle re-signed (`codesign --verify
  --strict` passed) and relaunched.
- 2026-10-07 (T1, PR checkout `fb07b44`, local `Cargo.lock` change kept,
  `rustc` 1.90.0): `cargo test -p tiny-core -p tiny --all-targets --locked`
  exited 0: `tiny-core` 123 passed, `tiny` unit 19 passed, `clean_smoke` 3
  passed (164.29 s), `scan_smoke` 1 passed, 0 failed. `cargo clippy -p tiny-core
  -p tiny --all-targets --locked -- -D warnings` exited 0. `--help` output for
  the top level and `clean`, `scan`, `sys`, `focus`, `uninstall` is identical
  between `main` and the PR checkout (B1/PC-C6 baseline).
- 2026-10-07: `rustup update stable` moved the toolchain from `rustc` 1.90.0 to
  1.99.0, satisfying UniFFI 0.32's `rustc` 1.91 requirement without pinning
  `cargo-platform`. T1 re-run on 1.99.0: clippy `-D warnings` exited 0; tests
  exited 0 with the same counts (123 + 19 + 3 + 1 passed, 0 failed).
- 2026-10-07: bento-grid demo rebuilt; Swift 6 build with a macOS 14 target
  produced no errors or warnings. `--snapshot` rendered Processes, Clean,
  Clean-stale and Unavailable PNGs that were inspected for layout defects (fixed:
  double row highlight, inconsistent number locale, tile overflow). Bundle
  re-signed ad hoc (`codesign --verify --strict` passed) and relaunched. The
  user accepted the rendered demo on 2026-10-07.
- 2026-10-07: UniFFI vs stdio spike in scratch `/tmp/tiny-spike` (never wired
  into the repo; deleted after the decision). UniFFI 0.32 build
  needed `cargo-platform` pinned to 0.3.1 under `rustc` 1.90. Both Swift clients
  compiled with `-swift-version 6`. FFI: `groups=2 items=149 bytes=105743890
  progress=5 firstProgress=0.0003–0.0008s`; typed `InvalidInput` error; panic
  surfaced as `UniffiInternalError rustPanic("spike panic")` with the process
  alive. Stdio: same results, first frame 0.64–1.25 s through Foundation pipe
  readers versus 0.0045 s with raw `read(2)`. Build integration, aborts,
  cancellation and FDA were not tested. The user approved switching to UniFFI.
- (Superseded by the UniFFI revision) 2026-10-07: independent `plan-reviewer` review, iteration 3/3, returned
  `EXECUTABLE: Yes`, no outstanding findings. Earlier missing native test targets
  and checked/cancellable core API details were resolved, with explicit work
  checkpoints added. This is plan evidence, not an implementation test result.
- (Superseded by the bento demo entry above) 2026-10-07: native prototype compiled with Swift 6 and a macOS 13 deployment
  target; `swiftc -parse-as-library -swift-version 6 -target arm64-apple-macosx13.0
  TinyPreview.swift -o tiny-preview` exited 0. Bundle signature check returned
  `valid on disk` / `satisfies its Designated Requirement`; native launch succeeded.
  The sample-only app remains open at the T2 scratch artifact path. Rendered
  appearance/interaction have not been inspected through Computer Use, and user
  visual acceptance is pending. No production CLI protocol is implemented.
- 2026-10-07: active plan/spec relative links, the Product Scope anchor and
  whitespace checks passed; `git diff --check` exited 0 in the primary checkout.
- 2026-10-07: earlier local PR baseline checks in this conversation passed:
  frontend build, native Tauri build/run and `cargo test -p tiny-desktop --lib
  --locked` (`8 passed; 0 failed`). These do not establish T1's full core/CLI
  regression gates or acceptance of the broader Tauri product scope.

- 2026-10-06: `cargo test --all-targets` exited 0 on the existing CLI:
  `140 passed; 0 failed` unit tests and `3 passed; 0 failed` smoke tests.
  The all-category dry-run smoke test took 100.89 seconds for the integration
  suite on this machine; this is not a desktop performance measurement.
- 2026-10-06: source inspection found 31 registered cleanup category IDs.
- 2026-10-06: `cargo clippy --all-targets -- -D warnings` exited 0:
  `Finished dev profile ... in 1.32s`.
- 2026-10-06: documentation validation confirmed both plan links and their
  anchors, seven scope-amendment links, and all 14 current acceptance criteria.
  `git diff --check` exited 0; the new plan has no trailing whitespace.
- 2026-10-06: installed `codex cloud exec --help` confirms submission support;
  `codex cloud list --json --limit 20` succeeded with zero tasks and no cursor.
  Environment selection and cloud task submission have not occurred.
- At that 2026-10-06 checkpoint, core extraction, process commands, demo,
  desktop build, and native acceptance had not been performed.


## Handoff

As of 2026-10-09:
- Current app overview A1–A4 implemented, verified, installed and launched.
  A5 user hands-on/visual acceptance remains pending. Broader plan stays active.
- Implementation checkout: `/Users/kyantran/.codex/worktrees/tiny-pr-1/tiny-cli`,
  branch `feat/native-desktop`, head `80249c8`; all work is committed locally.
- Installed app: `/Applications/Tiny Dev.app`, `com.mxrsv.tiny.dev`, ad-hoc signed,
  now contains the action-capable build from `80249c8`. The earlier read-only
  instance was stopped and the rebuilt app installed and launched on 2026-10-09
  (PID 81493). Installed and build executable SHA-256 values match.
- 2026-10-09 delivery checks: `scripts/test-native.sh` printed
  `PASS: 34 total native checks`; `scripts/build-native-app.sh` succeeded;
  installed `codesign --verify --deep --strict --verbose=2` printed `valid on disk`
  and `satisfies its Designated Requirement`; installed `--smoke-test` passed
  with 798 real processes and an owned child appearing/exiting. App/member
  in-process renders were inspected at `/tmp/tiny-ui-delivery-2026-10-09/`:
  app Quit and member Quit/Force Quit controls are visible. No Computer Use or
  live confirmation interaction was performed; user visual/flow acceptance
  remains pending. No application source changes were needed for delivery.
- The existing `/Applications/Tiny.app` and Tauri source/data remain untouched.
  Canonical spec/plan remain in the primary checkout; preserve unrelated edits.
- Rebuild with `scripts/build-native-app.sh`, check native behavior with
  `scripts/test-native.sh`; this CLT installation lacks XCTest/Swift Testing.
- 2026-10-08: actions 1–5 (spec X1–X4, O1 = Swift app quit, O2 = `--port`) are
  implemented and merged on `feat/native-desktop` at `80249c8` (code `9cde827`).
  All automated gates pass (see Wave 4). Nothing has been pushed or released.
  Next: the user reviews the now-open `/Applications/Tiny Dev.app` and accepts
  the flows listed in Wave 4. After that, mark T3–T6 accepted. Separate follow-up the
  user must decide on: the CLI `app-orphans` misclassification (also on `main`).
