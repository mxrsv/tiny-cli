# Processes + Clean desktop MVP

Record: Active task plan
Started: 2026-10-06
Documentation checkout: `/Users/kyantran/Documents/Development/Personal/tiny-cli` (`main`)
Implementation checkout: `/Users/kyantran/.codex/worktrees/tiny-pr-1/tiny-cli`, branch `feat/native-desktop` created on 2026-10-07 from PR #1 commit `fb07b4441b261543754ba2f4f4099a69a5f39e0d`

## Requirements reference and delivery scope

The [PRD Product Scope](../../_bmad-output/planning-artifacts/prd.md#product-scope)
owns PC-P1–PC-P5, PC-C1–PC-C6, and PC-D1–PC-D3. The user reconfirmed
Processes + Clean on 2026-10-07. The approved
[native/CLI boundary spec](../specs/2026-10-07-swiftui-cli-boundary.md)
owns B1–B6 and the SwiftUI + retained Rust CLI direction. The
[architecture](../../_bmad-output/planning-artifacts/architecture.md)
records the replacement of the earlier Tauri shell choice. Do not duplicate
product requirements here or expand scope to PR #1's Smart Scan/Space Lens/undo.

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
  - **T3a — workspace and adapter skeleton:**
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
  - **T3a′ — link smoke:** build `tiny-ffi` as a release staticlib, generate Swift
    bindings and link one export from a scratch `swiftc` program.
    - Linker flags come from
      `cargo rustc -p tiny-ffi --release -- --print native-static-libs`.
    - This proves the link early; it is a scratch check, not a deliverable.
  - **T3b — read-only process data:**
    - First move the bounded, fallible command runner into shared core: it times
      out, returns typed errors on spawn failure, non-zero exit or timeout, and
      lets tests inject a runner. T4a reuses it.
    - Then implement:
      - sampling (fresh vs first/unavailable CPU sample). A one-shot caller
        such as the CLI takes two refreshes separated by
        `sysinfo::MINIMUM_CPU_UPDATE_INTERVAL`, as `sys.rs` already does, so
        CPU is measured rather than unavailable;
      - details and parent/child tree (exited parent → unavailable);
      - listening ports via a bounded `lsof` probe with explicit errors.
  - **T3c — process actions:**
    - Send signals with `libc::kill`. Make `libc` a direct `tiny-core`
      dependency. Map `std::io::Error::last_os_error()`:
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

As of 2026-10-07:
- **Approved direction:** SwiftUI + a shared Rust core linked in-process through
  UniFFI, the retained CLI, Processes + Clean, and the accepted bento demo
  without a CLI command display.
- **Branch:** `feat/native-desktop` exists in the PR worktree at `fb07b44`, with
  the uncommitted local `Cargo.lock` adjustment.
- **Implementation:** no production adapter, process or native code exists yet.
- **Not authorized:** push, merge and release.

Keep the canonical requirements and this plan in the primary checkout. Do not
interpret PR #1's broader feature set as accepted MVP scope. The accepted
sample-only demo is the visual reference for T5c; retire its scratch source and
bundle after the production UI implements it and the user accepts it.

Next: T3a on `feat/native-desktop`. The
`tiny processes` syntax in T3d was approved on 2026-10-07.
Document actual checks/results here; do not create another plan.
