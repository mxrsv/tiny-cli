# Processes + Clean desktop MVP

Status: Approved 2026-10-06; reconfirmed with the SwiftUI direction 2026-10-07.
Started: 2026-10-06
Plan: [2026-10-06-processes-clean-mvp](../plans/2026-10-06-processes-clean-mvp.md)

Migrated on 2026-10-10 from the Product Scope section of the BMad PRD, which was
removed with the BMad workflow; the full PRD stays readable with
`git show 4fa0971:_bmad-output/planning-artifacts/prd.md`. Its personas, growth
roadmap and Tauri-era functional requirements are history, not requirements.

## Goal

The first desktop release helps a Mac user or developer find what consumes
resources and review cleanup candidates before acting. It has two destinations,
Processes and Clean. The [native/CLI boundary spec](2026-10-07-swiftui-cli-boundary.md)
owns the stack: a SwiftUI app linking the shared Rust `tiny-core` through UniFFI,
with the Rust CLI kept. Exact layout and styling need an interactive demo and the
user's visual acceptance before production integration.

## Processes acceptance criteria

- **PC-P1:** Show process name, PID, CPU, memory, ownership and sample time;
  support search and CPU/memory sorting. Missing or stale measurements are
  explicit, never displayed as freshly measured zeroes.
- **PC-P2:** Show parent/child relationships and listening ports when macOS
  permits collection. Exited parents, permissions and failed probes produce an
  explicit unavailable state rather than an empty-success claim.
- **PC-P3:** Let the user request termination of an eligible process with
  confirmation. Use SIGTERM first; force termination is a separate, explicitly
  confirmed action. Do not automatically escalate to SIGKILL or signal a tree.
- **PC-P4:** Revalidate PID and process start time at the action boundary.
  Refuse changed identities, self, protected/system processes and targets
  outside the permitted current-user scope. Report permission denial and a
  process that remains alive accurately.
- **PC-P5:** Expose the same process data and action semantics through the CLI
  (additive `tiny processes`) and the desktop via the shared core. Fixture tests
  and a disposable child process cover termination; validation never targets
  real user applications.

## Clean acceptance criteria

- **PC-C1:** Reuse the registered cleanup providers and risk groups. Display each
  candidate's category, path, measured size, reason for inclusion and known
  skip/permission limitations.
- **PC-C2:** Provide category selection and per-path review/exclusion before
  confirmation. Empty selection and cancellation cause no filesystem change.
- **PC-C3:** Use macOS Trash for the default desktop action. Review-risk items
  require explicit selection. Permanent deletion, emptying the Trash and
  nonrecoverable provider operations stay in the advanced CLI flow for this MVP;
  never fall back silently to permanent deletion.
- **PC-C4:** Revalidate selected paths, provider boundaries and relevant
  running-app checks immediately before acting. An unavailable safety check
  refuses the affected action and explains why.
- **PC-C5:** Report successful, failed and skipped items individually. Selected
  bytes, bytes moved to Trash and measured free disk space are different
  quantities; never claim that moving to Trash freed that many bytes.
- **PC-C6:** Preserve the existing CLI commands and flags. Shared core APIs return
  structured data; presentation and interactive prompts stay in the CLI or
  desktop adapter.

## Desktop acceptance criteria

- **PC-D1:** Keep sampling and cleanup off the UI thread; provide loading,
  cancellation, partial-result and error states. Overlapping scans or actions
  must not cause duplicate mutations or use another operation's result.
- **PC-D2:** Support keyboard navigation, visible focus, readable contrast and
  confirmation text that identifies the target and consequence.
- **PC-D3:** Release acceptance requires the user to approve rendered output and
  the Processes/Clean flows on macOS. Build or test success alone is not visual
  or native-app acceptance.

## Deferred

Smart Scan/Health Score, Space Lens, app-managed quarantine and undo, GUI
uninstall and startup management, menubar monitoring, scheduled or automatic
cleanup, forecasting, focus integration, maintenance/protection, supervised
process restart, platforms other than macOS, public distribution automation,
and showing the equivalent `tiny …` command (with a copy button and a visibility
toggle) beside GUI actions, deferred 2026-10-07. Existing CLI capabilities stay
available. Future work must reactivate a deferred item explicitly.
