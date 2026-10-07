# Processes + Clean desktop MVP

Record: Active task plan
Started: 2026-10-06
Documentation checkout: `/Users/kyantran/Documents/Development/Personal/tiny-cli` (`main`, `3c7e8a4`)
Implementation source checkout: `/Users/kyantran/.codex/worktrees/tiny-pr-1/tiny-cli` (detached HEAD, `fb07b4441b261543754ba2f4f4099a69a5f39e0d`, PR #1)

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
recorded in the boundary spec. Tiles, rings, sparklines and the command dock use
SwiftUI/AppKit only; no third-party UI library. No real process/file action
occurs before the user accepts the rendered design and flows.

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
a cleanup after a failure. Native FDA behavior for the bundled app is a required
check, not an assumed fix from SwiftUI.

Reuse Rust process/safety primitives where verified. Revalidate process identity
and cleanup path/provider/running-app boundaries at mutation time. Keep Trash-only
desktop behavior distinct from the advanced CLI. Existing Tauri services are
reference for guards, not a reason to import SQLite/quarantine into this MVP.

## Execution environment

The earlier user request for Codex Cloud development remains historical context;
this conversation approved and ran PR #1 locally and now chooses a native demo.
No cloud dispatch, new branch, commit, push, merge, release, or replacement of the
installed `/Applications/Tiny.app` is part of this planning/demo slice. Keep
native tests on this Mac; a cloud/Linux check cannot establish FDA, Trash, ports,
app lifecycle, keyboard behavior, or visual acceptance.

The primary checkout contains pre-existing dirty planning files. Update their
owning scope pointers selectively, preserve unrelated changes and `error.log`,
and do not copy the whole dirty primary checkout into the PR worktree. The PR
worktree also contains the local `Cargo.lock` compatibility adjustment from the
native run. No core extraction task is marked accepted solely from inspection.

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
    quit/force-quit dialogs, cleanup tile selection/preview/cancel, the command
    dock, and stale/unavailable states.
  - Verify: `swiftc -parse-as-library -swift-version 6 -target arm64-apple-macosx14.0 TinyPreview.swift -o tiny-preview`,
    `./tiny-preview --snapshot <scratch-dir>` (in-process render, no Screen
    Recording access), bundle signature verification, launch evidence, and user
    native visual acceptance. Build checks do not establish rendered correctness.
  - The demo targets macOS 14 for `onKeyPress` and numeric text transitions; the
    production minimum macOS version is not decided by this demo.
  - No Computer Use/screen capture without current-task authorization.
    Covers B6 and the interaction-design portion of PC-D2/PC-D3.

- [ ] **T3: Add the FFI adapter and process domain**
  - Depends on T1 and the concrete boundary/structure approach presented here.
  - Existing files: root `Cargo.toml` workspace members, `crates/tiny-core/src/lib.rs`.
  - New files: `crates/tiny-ffi/{Cargo.toml,src/lib.rs,src/session.rs,src/processes.rs,
    src/bin/uniffi-bindgen.rs}`,
    `crates/tiny-core/src/processes/{mod.rs,snapshot.rs,actions.rs}` and focused
    adjacent/CLI tests. Attach every new module to the existing entry points.
    UniFFI 0.32 needs `rustc` ≥ 1.91; the local toolchain is 1.99.0 since
    2026-10-07, so no `cargo-platform` pin is needed.
  - **T3a — adapter first:** implement the session object, typed errors, the
    cancellation token, progress callback trait and the operation gate. Verify
    with Rust tests calling the exported functions: typed errors, unwinding
    panic as failure (`panic = "unwind"` kept), overlap rejection and
    cancellation, before exposing any process mutation.
  - **T3b — read-only process data:** implement sampling, details/tree and bounded
    port probes; verify freshness, permissions, exited parents and unavailable
    first CPU samples before adding actions.
  - **T3c — process actions:** add confirmed signaling with fresh ownership and
    PID/start-time checks; verify only with disposable child processes.
  - Verify installed `sysinfo` support for CPU sampling, UID, PID/start time,
    parents and signals; macOS port probes must be bounded with explicit errors.
  - Revalidate ownership/identity; no automatic SIGKILL, tree signals or elevation.
  - Verify adapter errors, overlap and cancellation with Rust tests; process
    action tests use a disposable child only. Run the T1 checks plus
    `cargo test -p tiny-ffi --locked` with `--locked`. Covers B1–B3/B5 and PC-P1–PC-P5.

- [ ] **T4: Add cleanup discovery, trusted preview and recoverable execution**
  - Depends on T3; shares its session, cancellation token and operation gate.
  - Existing files: `crates/tiny-core/src/clean/{discover.rs,execute.rs,fs_safe.rs,types.rs}`,
    registry/providers and `crates/tiny/src/render/clean` regression tests.
  - New adapter module: `crates/tiny-ffi/src/clean.rs` with mirror records and
    focused tests; keep presentation out of shared core, without importing Tauri app state.
  - **T4a — checked discovery/cancellation:** add checked, cancellable core
    entry points rather than attempting to recover lost errors in the adapter.
    Existing affected files include `clean/providers/mod.rs`,
    `clean/process.rs`, `clean/runner.rs`, `clean/fs_safe.rs`, `clean/discover.rs`
    and providers using those helpers. Add a small operation context with a
    cancellation flag, checked directory/size walks, a fallible running-process
    probe and bounded child-command collection. Thread it through discovery
    loops/provider work, not just between categories. Keep compatibility
    wrappers/signatures and existing CLI formatting; test output compatibility.
    Verify denied reads, failed running-app probes, unavailable size data,
    cancellation mid-walk and child-command timeout using injected fixtures.
    Execute this slice through explicit checkpoints: first the fallible process
    probe/bounded runner and their tests; then checked size/directory walks and
    tests; finally provider/discovery integration. Run focused `tiny-core` tests
    after each checkpoint and CLI regression gates after integration.
  - **T4b — trusted preview:** store discovered candidates/previews/fingerprints
    in the session object; verify selection boundaries and forged/expired/replaced paths.
  - **T4c — execution/results:** require a confirmed preview ID, reject replay,
    revalidate paths/providers/running apps and report partial outcomes. Refuse
    an item when a required safety probe fails. Check cancellation before each
    mutation and never label an unreadable walk as empty success.
  - Keep destructive categories report-only; Trash failure preserves source,
    and cancellation reports already-completed work without claiming freed space.
  - Verify via temporary fixtures and an injected Trash adapter: empty selection,
    cancellation, path overlap, symlinks/replacement, forged/expired/consumed
    previews, partial results, panics during execution and Trash errors.
    Re-run T1 checks and `cargo test -p tiny-ffi --locked`.
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
  - **T5a — build integration first:** `scripts/build-tiny-ffi.sh` builds the
    arm64 static library, generates bindings and packages
    `macos/Frameworks/TinyFFI.xcframework` as a SwiftPM `binaryTarget`; generated
    files and the xcframework stay out of git. This untested coupling is the main
    remaining FFI risk, so prove it before UI binding.
    Verify: `scripts/build-tiny-ffi.sh` then `swift build --package-path macos --product Tiny`.
  - **T5b — Swift engine tests:** add SwiftPM test target `TinyEngineTests` with
    `macos/Tests/TinyEngineTests/{EngineErrorTests.swift,EngineOperationTests.swift}`.
    Exercise typed errors, panic-as-failure, progress on the main actor,
    cancellation, overlap rejection and preview invalidation through a fresh
    session, using temporary fixtures only.
    Verify: `swift test --package-path macos`.
  - **T5c — UI integration:** bind the accepted Processes/Clean layout to
    `TinyEngine`; implement focus/keyboard, confirmations, denied/partial/stale
    states and operation ownership without duplicating Rust decisions in Swift.
  - **T5d — packaging/platform checks:** new `scripts/build-native-app.sh`
    runs the FFI build and bundles the signed `Tiny` app. Verify Finder launch
    without a terminal/dev server and FDA behavior for the app identity.
  - Use the existing bundle ID/app-data location only after accounting for
    retained Tauri data. Do not delete data or reset TCC; no extra DB is needed.
  - Verify: `cargo build -p tiny -p tiny-ffi --locked`, `swift build --package-path macos --product Tiny`,
    `scripts/build-native-app.sh`, `codesign --verify --deep --strict <bundle>`.
  - Verify FDA behavior, launch/relaunch after termination and failure/cancel
    flows on Mac. Review adapter input validation and mutation paths before
    real-fixture native actions. Covers B2–B5 and PC-D1–PC-D3.

- [ ] **T6: Accept native flows and update shipped-behavior documentation**
  - Depends on T5. Existing files: `README.md`, this plan and boundary spec;
    update repo rules only for actual delivered locations/commands.
  - Run real mutation checks only against disposable child/file fixtures;
    inspect user-approved rendered output and access-denied/unavailable states.
  - Run `cargo fmt --all --check`, T1 regression checks, native contract tests,
    `swift test --package-path macos`, app package build and bundle signature checks
    after relevant final changes.
  - Retire the prototype after its accepted layout has been implemented; remove
    only this task's experimental files. Retiring the old Tauri shell is a later
    concrete, reviewable step, not an automatic delete during the demo.
  - Developer ID/notarization, public distribution and broader roadmap stay
    outside this development slice. Complete only when all relevant PC/B gates
    and user visual/native acceptance are met.

## Verification evidence
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
- 2026-10-07: independent `plan-reviewer` review, iteration 3/3, returned
  `EXECUTABLE: Yes`, no outstanding findings. Earlier missing native test targets
  and checked/cancellable core API details were resolved, with explicit work
  checkpoints added. This is plan evidence, not an implementation test result.
- 2026-10-07: native prototype compiled with Swift 6 and a macOS 13 deployment
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

As of 2026-10-07, the user approved SwiftUI + a shared Rust core linked
in-process through UniFFI (replacing the stdio worker after the spike), kept the
CLI, and selected Processes + Clean for the first native demo. Production
native/adapter code has not been implemented. The proposed native module/adapter approach is recorded
above for review; the user requested a docs commit on `main` on 2026-10-07; no branch/push/merge/release.

Keep the canonical requirements and this existing plan in the primary checkout;
the PR worktree is the inspected extracted-core source baseline. Preserve the
primary dirty documents and the PR worktree's local Cargo.lock change. Do not
interpret PR #1's broader feature set as accepted MVP scope.

The accepted sample-only bento demo is the visual reference for T5c; no real
operations are wired. Retire its scratch source and bundle after the production
UI implements and the user accepts it.

Independent plan review (`EXECUTABLE: Yes`) predates the UniFFI revision of T3–T5;
re-review those tasks before starting T3.
Next: re-review T3–T5, then start T3 on `rustc` 1.99.0.
Document actual checks/results here; do not create another plan.
