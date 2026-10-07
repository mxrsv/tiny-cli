# SwiftUI app and shared Rust core boundary

Status: Approved direction; native visual acceptance and production integration pending.
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

On 2026-10-07 the user kept the PRD's "every GUI action shows the equivalent
`tiny ...` command" differentiator, which requires one shared engine rather than
separate app and CLI implementations. The user then approved replacing the
earlier `tiny desktop --stdio` worker protocol with in-process UniFFI after the
spike recorded below.

The [PRD Product Scope](../../_bmad-output/planning-artifacts/prd.md#product-scope)
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
dock that always shows the equivalent `tiny ...` command. Simulate search/sorting, process details,
separate quit/force-quit confirmations, cleanup path selection/preview,
cancellation, and stale/unavailable data. No real signals, filesystem mutations,
background scanning, or permission changes occur in the demo. Production UI
integration requires the user's rendered-flow acceptance.

## In-process FFI boundary

A dedicated Rust adapter crate depends on `tiny-core` and exports a narrow API
with UniFFI proc macros. The app links it as a static library; no subprocess,
shell, PATH lookup, daemon, or network listener is involved. Existing CLI
commands keep calling `tiny-core` directly and do not depend on the adapter.

- Swift sees only generated adapter records, enums, objects, and errors. Adapter
  records mirror the needed `tiny-core` data; `tiny-core` gains no UniFFI
  dependency, and Rust internal types are not exposed to Swift.
- Paths cross the boundary as strings, and tuples become named records.
  Generated bindings are build output and are never edited by hand.
- Errors are typed adapter enums with stable cases and user-facing messages.
  Swift never maps an error, panic, or missing result to empty success.
- Long operations are blocking Rust calls run off the main actor. Progress uses
  a foreign callback trait; callbacks arrive on a background thread, so Swift
  hops to the main actor before touching UI state. Progress belongs to the call
  that started it.
- Cancellation uses an adapter-owned token object passed to the operation.
  Rust checks it between work units and before every mutation and returns
  accurate completed/failed/skipped results.
- An adapter-owned session object gates overlapping operations: busy operations
  are rejected instead of queued or merged.
- Planned operations: process list, process detail, process terminate, cleanup
  discover, cleanup preview, cleanup execute, and cancel. Mutation calls require
  explicit confirmation.
- Preview IDs and selected-path fingerprints live in the session object. Execute
  accepts a preview ID, not new arbitrary paths. Expired, consumed, modified,
  or lost previews fail without mutation. An app restart discards previews; the
  UI must require rediscovery/review and never automatically replay actions.
- Process actions revalidate PID/start time and ownership immediately before
  signaling. Cleanup retains the PRD's Trash-only desktop policy, provider/path
  safety checks, and per-item reporting; no permanent-delete/quarantine fallback.
- Unwinding Rust panics surface as a failed call, so the adapter build must keep
  `panic = "unwind"`. A panic during a mutation is an unknown outcome, not
  success or an invitation to retry. Aborts and memory faults terminate the app;
  after relaunch the user sees no replay and is told to inspect the target/Trash
  before repeating.
- In-process code runs under the app's own TCC identity. Verify Full Disk Access
  behavior for the bundled app on macOS. SwiftUI does not grant permissions or
  bypass TCC; inaccessible data remains explicit.

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
  accurate, and no mutation is blindly retried or performed without a valid
  confirmed preview.
- **B4:** Given the native bundle, when launched from Finder without a shell or
  dev server, then it runs with its linked engine, displays unavailable/denied
  access honestly, and supports the accepted keyboard and confirmation flows.
- **B5:** Given temporary process/file fixtures, when executing CLI and native
  flows, then identity changes, symlinks, stale previews, denied access, Trash
  failures, and partial outcomes retain the PRD's safety invariants.
- **B6:** Given the sample-only prototype, when users exercise its controls,
  then no real process/file action or permission change occurs, and the user
  selects/accepts the layout before production integration.
