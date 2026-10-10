# Docs and agent instructions restructure

Record: Active task plan
Started: 2026-10-10
Checkouts: `main` at `/Users/kyantran/Documents/Development/Personal/tiny-cli`;
`feat/clean-trust` at `/Users/kyantran/Documents/Development/Personal/tiny-cli-worktrees/clean-trust`
(this plan lives here only).

## Scope

No spec; the requirements were agreed in conversation on 2026-10-10:

- Remove the BMad workflow completely: skills, `_bmad/` config and `_bmad-output/`.
  Requirements still cited move into `docs/specs/`.
- Keep docs short but complete, and linked: `AGENTS.md` → [docs index](../README.md) →
  tier pages; each R-rule links its internals section; each spec and plan link both ways.
- One rule set and one vocabulary: `AGENTS.md` follows the same skeleton on both
  branches, the [glossary](../internals/glossary.md) owns the terms, and living docs
  describe the code on the branch they sit on.

Out of scope: the Theme accent and Lottie in `f18de6e`, CI changes, stale
`feat/clean-trust-*` worktrees, pushing.

## Tasks

- [x] **A1 — main: remove BMad workflow.** `4fa0971`: skills and `_bmad/`.
- [x] **A2 — main: remove `_bmad-output/`, migrate MVP requirements.** `3cfa9df`:
  [MVP spec](../specs/2026-10-06-processes-clean-mvp.md) holds PC-P/PC-C/PC-D; links
  repointed; desktop pointer moved to `feat/clean-trust`.
- [x] **B1 — merge `main` into `feat/clean-trust`.** `af772d3`: code conflicts take the
  branch (`crates/`) side; `AGENTS.md` and `README.md` kept from the branch for B2/B4.
- [x] **B2 — `AGENTS.md` for the workspace.** `fdaaf05`. Port R1–R6 to `crates/` paths, one gate
  matching CI, native bridge steps, worktree convention, language section, links into
  internals.
- [x] **B3 — internals.** `b8d9b8f`. Overview, clean pipeline, glossary and traps describe the
  workspace, FFI and Swift app; native traps extracted from the MVP plan.
- [x] **B4 — user docs, operations and README.** `f53b80d`. README back to the short command table;
  desktop and `processes` usage move to `docs/user/`; development page gains the
  native gates.
- [x] **B5 — records.** `3cc92eb`. Active specs and plans link both ways; stale checkouts and
  handoffs corrected; frozen records untouched.
- [x] **B6 — main `AGENTS.md` skeleton.** `cd10f27`, plus `303b5d3` for two drifted
  lines in main's `docs/user/clean.md`; merged forward in `a88a8cb` keeping the branch side. Same sections as the branch, with the
  branch-ownership line.

## Verification

- `~/.claude/scripts/docs-anchors.sh` and `docs-compliance.sh` on both checkouts.
- A relative-link check over active specs and plans.
- Stale-token grep: `src/commands` (branch only), `feat/native-desktop` as a pointer,
  `_bmad`, `Cargo.lock is gitignored`.
- After B1: `cargo fmt --all --check`, `cargo test --workspace --all-targets --locked`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `scripts/test-native.sh`.

## Progress

- 2026-10-10: A1, A2 on `main`; B1 merged locally. Nothing pushed.
- 2026-10-10: B2–B6 done. After B1: `cargo fmt --all --check` clean; workspace tests all
  ok (273 core, `clean_smoke` 171 s); clippy `-D warnings` clean; `test-native.sh` 34/34
  PASS. After B6: `docs-anchors.sh` exit 0 and `docs-compliance.sh` pass on both
  checkouts; 187 relative links in living docs and active records, 0 broken; stale-token
  grep empty.
- 2026-10-10: install, app build and command table each have one home (getting-started,
  desktop page, README); README 290 → 40 lines.

## Handoff

- Remaining: push both branches (user decision). Draft PRs #4 and #5 will conflict on
  `AGENTS.md` R1 and `README.md`; PR #2 (main) touches `docs/user/clean.md` and
  `docs/internals/clean.md`, which main merges forward with the branch side.
- Open decisions: freeze `plans/2026-05-04-tiny-clean-advanced.md` as superseded (its
  §1 item 4 family-grouped summary never shipped) and mark `plans/2026-04-30-tiny-clean.md`
  historical (shipped in `0995524`…`ba8ef9b`).
