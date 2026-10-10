# SwiftUI app and shared Rust core boundary

Status: Read-only app overview with system widgets and app groups implemented and running locally; user visual acceptance pending. Actions X1–X4 approved 2026-10-08 (O1 = Swift app quit, O2 = `--port`) and implemented on `feat/native-desktop`; user acceptance pending.
Started: 2026-10-07

## Goal and approved decisions

Use a native SwiftUI macOS app while preserving the Rust `tiny` CLI and its
shared `tiny-core`. Swift owns presentation and macOS window/access integration;
Rust owns process inspection/actions and cleanup computation/actions. Each
operation has one implementation in `tiny-core`, called by the CLI and linked
in-process into the app through UniFFI-generated Swift bindings.
The user approved the Swift/Rust division on 2026-10-07 and reconfirmed
**Processes + Clean** as the first native demo scope. Prefer SwiftUI, AppKit
when necessary, and Swift Charts; add no third-party UI library without a
demonstrated need.

One shared engine keeps safety rules, providers and revalidation identical in
the CLI and the app, unlike separate app and CLI codebases. The user approved
replacing the earlier `tiny desktop --stdio` worker protocol with in-process
UniFFI after the spike recorded below. Later on 2026-10-07 the user deferred
showing equivalent `tiny ...` commands in the GUI for the MVP: no command bar,
copy action or command string is shown. The shared core keeps this addable later.

The [MVP spec](2026-10-06-processes-clean-mvp.md)
continues to own the product requirements and PC-P, PC-C, PC-D acceptance
criteria. This spec owns only the native/Rust boundary. The existing
[MVP plan](../plans/2026-10-06-processes-clean-mvp.md) owns execution.
PR #1's broader Tauri implementation is reusable source, not a scope expansion.
Existing SQLite/quarantine data must not be deleted or migrated by the demo.

## Native demo

Provide a separately identified native `.app`, clearly marked as sample data,
with Processes and Clean. On 2026-10-07 the user rejected the default-control
sidebar/inspector/focus layouts as generic and chose a dark bento-grid style:
Apple-widget tiles with one primary value each, colored rings/sparklines with
semantic color (safe green, review orange), glass/material depth, and a bottom
dock for navigation, the review action and the sample data state. Simulate search/sorting, process details,
separate quit/force-quit confirmations, cleanup path selection/preview,
cancellation, and stale/unavailable data. No real signals, filesystem mutations,
background scanning, or permission changes occur in the demo. Production UI
integration requires the user's rendered-flow acceptance.

## First live native slice (approved 2026-10-08)

The first runnable SwiftUI app reads real processes on this Mac and lets the
user inspect the accepted bento-style interface. Process termination and all
cleanup actions are deferred. This slice may ship before T3c/T3d/T4; it reuses
the existing process core and UniFFI adapter. No new CLI contract is needed.

- **R1:** Given the installed development app, when launched directly, then it
  lists real local processes (name, PID, CPU, memory and owner) without a dev
  server, worker or separate CLI installation.
- **R2:** Given the live list, when searching, sorting, selecting, refreshing or
  pausing automatic refresh, then the interface stays responsive and details
  belong to the selected PID/start-time identity. Late results cannot replace
  a newer selection; exited/reused PIDs show an explicit unavailable state.
- **R3:** Given unmeasured CPU, inaccessible memory/ports, or a query error,
  then the UI distinguishes unavailable data from zero and empty results;
  retained data is visibly stale/paused. CPU is per-core and may exceed 100%.
- **R4:** Given this first slice, when interacting with any control, then no
  process signal, cleanup execution or permission change occurs. The UI labels
  the read-only scope; deferred actions are not actionable.
- **R5:** Given the native bundle, when comparing against an owned temporary
  process and rendering the live SwiftUI surface, then the process appears and
  disappears correctly, the signature verifies, and the user can review the
  actual app. Automated evidence does not replace user visual acceptance.

The existing dark bento direction and native `.app` review surface remain
approved. The original scratch demo is absent as of 2026-10-08; an earlier bento source
was recovered from session history as a visual reference. The live integration
still requires its own rendered-flow acceptance.

## App overview and grouping (2026-10-08)

The user requested CPU/RAM widgets and an app-oriented list with recognizable
app icons, replacing the flat process-first overview. Keep the native dark
bento surface and read-only boundary. System & Background is collapsed by
default for processes without an identifiable app. This implementation default
was stated in the conversation; the optional alternative is a filter-based view.

- **G1:** Given a valid live sample, the top two widgets show whole-machine CPU
  utilization (0–100%) and RAM used/total plus percentage. They use system metrics,
  never summed process percentages or summed resident memory as whole-machine
  usage. Warm-up, unavailable, paused and stale states stay explicit.
- **G2:** Given several processes belonging to one installed app, the main list
  shows one stable app row with its local app icon, name, process count and
  aggregated CPU/resident memory. App association uses bundle/executable metadata
  and bounded parent relationships, not name similarity. Independent child apps
  keep their own group; unresolved processes remain available separately.
- **G3:** Given incomplete member measurements, app totals visibly state partial
  availability rather than presenting missing data as zero. App CPU retains the
  current per-core convention (can exceed 100%); whole-machine widget CPU is
  explicitly labelled separately. Resident-memory sums are not a substitute for
  system RAM usage and may count shared pages more than once.
- **G4:** Given a selected app, its inspector shows the member processes and lets
  the user select a member for the existing PID/start-time-safe detail view.
  Search matches app or member metadata; sort uses app aggregate resources.
  Refresh preserves app selection across member churn; disappeared groups and
  reused process identities cannot display another entity's details.
- **G5:** Given missing local icons or app metadata, show a native fallback icon
  and retain inspectable process data. No network icon lookup, new dependency,
  process action, cleanup action, or permission change is introduced.
- **G6:** Given the new app surface, native checks cover grouping, aggregate
  completeness, lifecycle/selection and system metric semantics; a real-data
  native render and installed bundle are supplied for user visual acceptance.

## Process and cleanup actions (approved 2026-10-08)

On 2026-10-08 the user approved five actions for the native app and CLI:
Quit, Force Quit, Clean to Trash, Free port, and Reveal/Copy. This
reactivates T3c, T3d, T4 and the action-capable part of T5/T6 under the
existing PC-P, PC-C, PC-D and B criteria, and supersedes R4's read-only rule
for these five actions only. Suspend/resume, priority changes, app-scoped
cleanup, uninstall and login items remain deferred.

- **X1 — Quit / Force Quit:** PC-P3/PC-P4 and the process-action rules in the
  FFI boundary below apply unchanged. App-level quit semantics are open (O1).
- **X2 — Free port:** Given a TCP port, when the user looks up its listener,
  then the core returns each visible owning process with PID/start-time
  identity, and Quit/Force Quit go through the X1 path with its refusals and
  revalidation. A non-root `lsof` cannot see other users' sockets, so an
  invisible owner, another user's owner, or a failed probe is reported as
  unknown or not actionable, never as "port is free". Several owners are all
  listed; nothing is signalled automatically.
- **X3 — Reveal / Copy:** Given a selected app or process, the user can reveal
  its bundle or executable in Finder and copy its PID or path. These are
  Swift-only, non-mutating actions. A missing or inaccessible path disables the
  action with a reason instead of revealing something else.
- **X4 — Action surface:** The READ ONLY label is replaced only once actions are
  wired. Confirmation text names the target and the consequence (PC-D2). Layout
  of action controls passes the frontend gate with the user before Swift work.
  Layout chosen by the user on 2026-10-08: Quit and a `⋯` menu (Reveal, Copy)
  on the selected app in the inspector, Quit/Force Quit…/Reveal/Copy PID on a
  selected member process, and the same items in the row context menu; native
  macOS alerts naming target and consequence, with Force Quit confirmed
  separately as a destructive action; a bento **Ports** tile listing the
  current user's visible listeners (owner, Quit/Force Quit, visibility caveat);
  and an Activity ⇄ Clean switch in the bottom dock, where Clean shows category
  tiles (green safe, orange review, report-only marked) leading to a per-path
  review sheet, Move to Trash and a per-item report. Each rendered flow still
  needs the user's visual acceptance.

Decisions (user chose the recommended options on 2026-10-08):

- **O1 — App-level Quit, decided (a):** each app row groups several processes, and signalling
  all members is a tree signal that PC-P3 forbids. Options: (a) Quit on the app
  row uses `NSRunningApplication.terminate()` in Swift, verified on 2026-10-08
  to need no Automation prompt (evidence in the plan's ui lane part 1), plus per-PID Force Quit through Rust; this moves one action into Swift
  and amends this boundary. (b) Rust sends an `osascript` quit by bundle ID,
  which triggers an Automation prompt per target app. (c) Per-PID Quit only, in
  the inspector. Chosen: (a). App-row Quit is the one Swift-owned action;
  per-PID Quit/Force Quit stay in Rust.
- **O2 — Free port in the CLI, decided:** additive `tiny processes --port <PORT>`
  filters the list to visible owners (PC-P5 parity); quitting stays by PID.

## In-process FFI boundary

A dedicated Rust adapter crate depends on `tiny-core` and exports a narrow API
with UniFFI proc macros. The app links it as a static library; there is no
worker subprocess, daemon, or network listener at the boundary. Existing CLI
commands keep calling `tiny-core` directly and do not depend on the adapter.
`tiny-core` itself still runs bounded system tools (for example `pgrep`, `lsof`,
`osascript`); failures and timeouts are typed errors, never empty results. A
Finder-launched app has a minimal `PATH`, so the app resolves tools to absolute
paths through `PATH` and the standard Homebrew prefixes (the CLI keeps `PATH`
only). A provider whose tool is missing is reported as unavailable rather than
silently omitted.

- Swift sees only generated adapter records, enums, objects, and errors. Adapter
  records mirror the needed `tiny-core` data; `tiny-core` gains no UniFFI
  dependency, and Rust internal types are not exposed to Swift.
- Paths cross the boundary as strings, and tuples become named records.
  Generated bindings are build output and are never edited by hand.
- Errors are typed adapter enums with stable cases plus diagnostic detail.
  User-facing copy and guidance (including FDA/Automation explanations) live in
  Swift. Swift never maps an error, panic, or missing result to empty success.
- Long operations are blocking Rust calls run off the main actor. Progress uses
  a foreign callback trait; callbacks arrive on a background thread, so Swift
  hops to the main actor before touching UI state. Progress belongs to the call
  that started it.
- Cancellation uses an adapter-owned token object passed to the operation.
  Rust checks it between work units and before every mutation and returns
  accurate completed/failed/skipped results.
- An adapter-owned session object gates overlapping scans and mutations: busy
  operations are rejected instead of queued or merged. Short read-only process
  queries (list/detail) do not take the gate, so the UI can refresh during a
  scan.
- Planned operations: process list, process detail, process terminate
  (graceful or force, each confirmed separately), cleanup discover, cleanup
  preview, cleanup execute, and cancel. Mutation calls require
  explicit confirmation. Process list/detail/terminate are also exposed through
  a new additive `tiny processes` CLI command backed by the same core (PC-P5).
- Discovery isolates provider failures: a denied or failing category becomes a
  typed per-category error/unavailable entry while other categories still return.
- Preview IDs and selected-path fingerprints live in the session object. Execute
  accepts a preview ID, not new arbitrary paths. Expired, consumed, modified,
  or lost previews fail without mutation. An app restart discards previews; the
  UI must require rediscovery/review and never automatically replay actions.
- Process actions revalidate PID/start time and ownership immediately before
  signaling and report permission denial, an already-exited target, and a target
  still running after the signal as distinct outcomes. Cleanup retains the PRD's
  Trash-only desktop policy, provider/path safety checks, and per-item reporting
  (moved, failed, skipped with reason, not attempted); no permanent-delete or
  quarantine fallback. Per-item revalidation and cancellation checks happen in
  the core immediately before each item, not once per batch. Desktop execution
  only moves individually listed paths to Trash; providers whose cleanup is a
  tool command rather than a per-path move (for example `docker system prune`)
  are report-only on the desktop.
- Trash keeps the existing Finder (`osascript`) route so Finder's Put Back works.
  In the app this requires Automation (Apple Events) permission for Finder; a
  denial is a typed error with guidance and never falls back to deletion.
- Unwinding Rust panics surface as a failed call, so the adapter build must keep
  `panic = "unwind"`. A panic during a mutation is an unknown outcome, not
  success or an invitation to retry; the operation gate is released and the
  preview stays consumed. Aborts and memory faults terminate the app, so the app
  persists a small in-flight marker before each mutation and clears it on any
  return. A marker present at launch shows an unknown-outcome notice telling the
  user to inspect the target/Trash before repeating; nothing is replayed.
- In-process code runs under the app's own TCC identity. Verify Full Disk Access
  and Finder Automation behavior for the bundled app on macOS. Development builds
  use the separate bundle ID `com.mxrsv.tiny.dev` so their permission state never
  mixes with the installed `/Applications/Tiny.app`. SwiftUI does not grant
  permissions or bypass TCC; inaccessible data remains explicit.

Names and module locations are a concrete implementation proposal, not a
shipped API. The app and adapter are built and released together; adapter API
changes regenerate bindings in the same change.

## Spike evidence (2026-10-07)

A scratch-only spike linked PR #1's `tiny-core` through UniFFI 0.32 and compared
it with a minimal JSON-lines worker. Both returned identical discovery results.
UniFFI delivered typed Swift errors and converted an unwinding panic into a Swift
error while the process stayed alive. Bindings compiled under Swift 6 language
mode without concurrency errors. The stdio worker needed hand-mapped string error
codes, and the convenient Foundation pipe readers delayed the first frame by
0.6–1.25 s, while a raw `read(2)` loop received it in about 5 ms. Not tested:
Xcode/SwiftPM build integration, aborts, cancellation, and FDA attribution.

## Boundary acceptance criteria

- **B1:** Given existing CLI usage, when commands run after integration, then
  names, flags, confirmation rules, and existing scan JSON remain compatible.
- **B2:** Given valid/invalid adapter calls, when Swift invokes them, then each
  call returns a typed result or typed error, progress reaches only its caller,
  and panics surface as failures rather than empty success.
- **B3:** Given a scan/action running, when cancellation, overlapping calls,
  panics, or app termination occur, then the UI remains responsive, outcomes are
  accurate, no mutation is blindly retried or performed without a valid
  confirmed preview, and a relaunch after termination mid-mutation shows the
  unknown-outcome notice.
- **B4:** Given the native bundle, when launched from Finder without a shell or
  dev server, then it runs with its linked engine, displays unavailable tools and
  denied Full Disk Access/Automation honestly, and supports the accepted keyboard
  and confirmation flows.
- **B5:** Given temporary process/file fixtures, when executing CLI and native
  flows, then identity changes, symlinks, stale previews, denied access, Trash
  failures, and partial outcomes retain the PRD's safety invariants.
- **B6:** Given the sample-only prototype, when users exercise its controls,
  then no real process/file action or permission change occurs, and the user
  selects/accepts the layout before production integration.
