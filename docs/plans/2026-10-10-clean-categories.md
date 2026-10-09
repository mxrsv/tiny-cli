# Plan: Clean categories — families, overview, new categories

**Spec**: [2026-10-10-clean-categories](../specs/2026-10-10-clean-categories.md) |
**Checkout**: `~/Documents/Development/Personal/tiny-cli-worktrees/clean-trust`, branch
`feat/clean-trust` (main-only follow-ups name `~/Documents/Development/Personal/tiny-cli`) |
**Status**: waiting on the layout decision (C0); nothing implemented yet

## Tasks

| # | Task | Covers | Check |
|---|------|--------|-------|
| C0 | Record the chosen layout (A, A + bar, B or C) in spec §2 | AC1 | user choice |
| C1 | One per-id table in tiny-core (family, trust section, `comes_back`, desktop decision) with a registration test; additive FFI `family` + `section`; regenerate bindings; update AGENTS.md R1 | FM1, FM2, AC2 | core/ffi tests fail for an id missing a field |
| C2 | Swift layout for the chosen option: family sub-sections, size sort, categories under 1 MB as rows, no family or group checkbox | FM3, FM4 | `scripts/test-native.sh`, CleanTests |
| C3 | Headline "≈X GB you can move to the Trash" (deduplicated over movable candidates, report-only excluded and named); dock shows bytes | OV1, OV2, AC3 | fixture with overlapping candidates |
| C4 | Ledger: skipped because running (with "Quit <app> & scan again"), not on this Mac, read partially, nothing found | OV3 | CleanTests per outcome |
| C5 | Read-only Inspect list on every tile, report-only included | OV4 | CleanTests |
| C6 | Family storage bar, only if C0 picks A + bar or B | OV5 | VoiceOver list, click never selects |
| C7 | Real-data scan screenshots of the new layout; never press Move to Trash | AC4 (+ clean trust AC4) | user approval |
| C8 | Providers batch 1: `browser-automation`, `swift-packages`, `js-tool-caches`, `homebrew-cache`; split paths excluded from `user-caches` through declared roots | NC, AC5 | temp-dir fixtures; no path offered twice |
| C9 | Providers batch 2: `browser-profiles`, `electron-app-caches`, `device-updates`, `macos-installers`, `ai-models` (size only) | NC, AC5 | temp-dir fixtures; browser and app gates |

C1 lands before C2–C6. C8–C9 follow C7.

## Progress

- 2026-10-10: multi-agent review (9 agents: competitors, disk landscape, core audit, UX
  critique, three proposers, critic, synthesis) and a Sonnet mock of the four layouts.
  Both live only in the session scratchpad `/tmp/tiny-clean-ideas-2026-10-10/`
  (`synthesis.md`, `mock/index.html`, `mock/option-*.png`); `/tmp` is not durable, the
  decisions are in the spec. Spec written (`f7f302f`).
- 2026-10-10: the safety work the spec requires first is done and recorded in the
  [clean trust plan](2026-10-09-clean-trust-evidence.md): `main` `1baeb67`, `8b4842a`;
  `feat/clean-trust` `c91e953`, `8463af1`, `978e531`, `f4338de`, `0631800`. Pushed to
  `origin` with `main`, `feat/native-desktop` and `feat/clean-trust`.
- 2026-10-10: C1 done on branch `claude/category-taxonomy-581gt9` (draft PR into
  `feat/clean-trust`): `CATEGORIES` table, `TrustSection`, FFI `family` ids renamed and
  `section` added with a Swift default of `nil`, so existing Swift call sites build. AI
  models is not a family yet (FM1: once its providers exist). Trash and `app-orphans` are
  Leftovers; `user-caches` is Apps & browsers.

## Handoff

- **Next**: the user picks a layout (recommended: A + bar, the only option that shows where
  the space is without changing UI1); then C0 and C1.
- **Waiting on the user**:
  - AC4 approval, in one round with the clean trust AC4 (C7).
  - Go-ahead to merge `feat/clean-trust` into `feat/native-desktop`; that checkout
    (`~/.codex/worktrees/tiny-pr-1/tiny-cli`) has another session's uncommitted Clean UI
    changes.
  - Full Disk Access for `com.mxrsv.tiny.dev`, to learn whether Mail, Chat and Browser show
    "at least Zero KB" only for lack of access.
- **Uncommitted on `main`**: `docs/user/clean.md` documents `docker-volumes` and the
  report-only `app-orphans`, but the file belongs to another session's uncommitted docs
  restructure (`docs/`, `AGENTS.md`, `README.md`, `src/commands/clean/cli_validate.rs`
  comment, staged `.planning/` rename), so it was not committed.
- **Known issues, not in scope, need a go-ahead**:
  - `main` `src/commands/clean/providers/user_caches.rs` still gates by the last name
    segment (`Google` is offered while Chrome runs); the fix exists only on
    `feat/clean-trust`.
  - `browser_caches.rs` looks for Chrome's cache under `Application Support`; the real
    cache is `~/Library/Caches/Google/Chrome/<Profile>/Cache` (both trees).
  - R2 drift: `is_file()` follows symlinks in `rust_targets.rs`, `node_modules.rs`,
    `python_caches.rs` (both trees).
  - `main` clippy fails on six pre-existing lints in `focus.rs`, `scan/`, `uninstall.rs`.
  - `app-orphans` stays report-only until ownership is matched per bundle id;
    `health.rs` counts it as reclaimable.
  - The CLI drops categories whose discovery failed without saying so.
  - Open items in the clean trust plan's handoff (Docker `Size` vs `Reclaimable`,
    placeholder evidence for AC3, `simulator-devices` untested against a real `simctl`,
    SR1 dirty-tree follow-up).
