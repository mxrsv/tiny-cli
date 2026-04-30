# Plan: `tiny clean`

## Summary

Add a new `tiny clean` command that performs real cleanup, while keeping `tiny scan`
as the read-only file discovery command. The cleanup flow should feel similar to
`tiny uninstall`: discover candidates, show a clear report, let the user select
categories interactively, print the final plan, then ask for an action before any
files are touched.

The v1 scope is intentionally broad enough to feel like a lightweight
CleanMyMac-style cleaner, but the default behavior must stay conservative:
recoverable cleanup via Trash, no malware scanning, no app updates, no cloud
cleanup, and no automatic deletion of personal files.

## Goals

- Provide a real cleanup command named `tiny clean`.
- Keep cleanup transparent: always show what will be removed before execution.
- Use an interactive multi-select UI like `tiny uninstall`.
- Group cleanup candidates by category and risk level.
- Default to Move to Trash for recoverability.
- Keep the architecture provider-based so future cleanup categories do not turn
  the command into one large file.

## Non-goals

- Do not replace `tiny scan`; `scan` remains read-only.
- Do not implement malware detection or security scanning.
- Do not implement performance boosting, background process management, or app
  update management.
- Do not clean system-level paths such as `/System`, `/Library`, or `/private`
  in v1.
- Do not automatically clean personal files from `Documents`, `Desktop`, or
  `Downloads`; those can only appear as review candidates.

## User Experience

### Commands

```bash
tiny clean
tiny clean --dry-run
tiny clean -y
tiny clean --hard -y
tiny clean --category caches
tiny clean --include-review
tiny clean --include-destructive
```

### Default interactive flow

1. Discover cleanup candidates.
2. Display categories with size, item count, and risk.
3. Open a checkbox picker so the user can select categories.
4. Print the cleanup plan with concrete paths.
5. Ask what action to take:
   - Move to Trash (recoverable)
   - Dry-run (no changes)
   - Hard delete (not recoverable)
   - Cancel
6. Execute only after the user chooses a non-dry-run action.

### Example picker

```text
[ ] User caches             4.2 GB   safe
[ ] User logs               120 MB   safe
[ ] Xcode DerivedData       9.8 GB   safe
[ ] Cargo cache             1.3 GB   review
[ ] npm cache               850 MB   review
[ ] Trash                  18.0 GB   destructive
```

## Public CLI Interface

Add a new `CleanOpts` struct and `Commands::Clean(CleanOpts)` variant.

Suggested options:

```text
--dry-run
    Show the report and exit without prompting for cleanup.

-y, --yes
    Skip the action menu and execute the default action.

--hard
    Use permanent deletion instead of Move to Trash.

--category <category>
    Restrict discovery and selection to one cleanup category.

--include-review
    Include review-risk categories in the default picker list.

--include-destructive
    Include destructive categories such as Trash in the picker list.
```

Default behavior:

- `tiny clean` includes safe categories by default.
- Review and destructive categories are visible only when explicitly requested,
  unless the implementation chooses to show them unchecked with clear labels.
- `tiny clean -y` moves selected/default safe items to Trash.
- `tiny clean --hard -y` permanently deletes selected/default items.
- `tiny clean --dry-run` never prompts and never removes anything.

## Cleanup Categories

### Safe

These are expected to be regenerable and can be shown by default:

- User caches: `~/Library/Caches/*`
- User logs: `~/Library/Logs/*`
- Xcode DerivedData: `~/Library/Developer/Xcode/DerivedData`

### Review

These can free meaningful space but may slow down future builds, installs, or
tooling startup:

- Xcode Archives: `~/Library/Developer/Xcode/Archives`
- Xcode iOS DeviceSupport: `~/Library/Developer/Xcode/iOS DeviceSupport`
- Cargo cache: selected subdirectories under `~/.cargo`
- npm cache: detected via standard npm cache location
- pnpm store: detected via standard pnpm store location
- yarn cache: detected via standard yarn cache location
- Old or large file candidates from `Downloads`, `Desktop`, and `Documents`

### Destructive

These should require explicit opt-in:

- User Trash: `~/.Trash`
- App leftovers discovered by bundle id, potentially sharing logic with
  `tiny uninstall`

## Architecture

Use a provider-based structure so each cleanup source is isolated.

Suggested layout:

```text
src/commands/clean/
  mod.rs
  types.rs
  discover.rs
  picker.rs
  report.rs
  execute.rs
  providers/
    mod.rs
    user_cache.rs
    user_logs.rs
    xcode.rs
    dev_cache.rs
    trash.rs
```

Core model:

```rust
trait CleanProvider {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn risk(&self) -> RiskLevel;
    fn discover(&self) -> Result<Vec<CleanItem>>;
}

enum RiskLevel {
    Safe,
    Review,
    Destructive,
}

struct CleanItem {
    category_id: String,
    category_label: String,
    path: PathBuf,
    size: u64,
    risk: RiskLevel,
}

enum CleanAction {
    Trash,
    HardDelete,
    DryRun,
    Cancel,
}
```

Command flow:

1. Build provider list from CLI options.
2. Discover items from providers.
3. Group by category and calculate size totals.
4. If `--dry-run`, print summary and detailed report, then exit.
5. If interactive, show category picker.
6. Build a plan from selected categories.
7. Print the concrete paths and total size.
8. Decide action.
9. Execute via Trash or permanent deletion.

## Safety Rules

- Never remove anything before a plan is printed.
- Prefer Move to Trash by default.
- Require explicit `--hard` for permanent deletion.
- Do not select destructive categories by default.
- Skip paths that do not exist.
- Do not follow symlinks into unexpected locations when calculating or deleting.
- Refuse system paths in v1.
- Keep personal file candidates as review-only and never selected by default.
- If one item fails to remove, continue with the rest and report failures at the
  end.

## README Updates

Update README to describe the distinction:

- `tiny scan`: read-only report for old and large files.
- `tiny clean`: interactive cleanup with selectable categories and recoverable
  default behavior.

Include examples:

```bash
tiny clean
tiny clean --dry-run
tiny clean --include-review
tiny clean --category caches
```

## Test Plan

Automated tests:

- Provider discovery returns empty results when target paths do not exist.
- Size aggregation by category is correct.
- Risk filtering includes safe categories by default.
- `--include-review` includes review categories.
- `--include-destructive` includes destructive categories.
- `--dry-run` never executes removal.
- `--hard -y` maps to permanent deletion action.
- Missing paths are skipped without failing the whole command.

Manual checks:

```bash
cargo test
cargo run -- --help
cargo run -- clean --dry-run
cargo run -- clean --category caches --dry-run
cargo run -- clean --include-review --dry-run
cargo run -- clean
```

## Rollout Plan

1. Add CLI type and command dispatch.
2. Add clean module skeleton and core types.
3. Implement safe providers first.
4. Add report and interactive picker.
5. Add execution with Trash and hard delete.
6. Add review and destructive providers behind explicit options.
7. Update README.
8. Run tests and manually verify the CLI help/output.

## Open Decisions

- Whether review/destructive categories should be hidden by default or shown
  unchecked with warnings.
- Whether `--category` should accept only one category or allow multiple values.
- Whether app leftovers should reuse `uninstall` internals immediately or wait
  for a shared helper refactor.

## Review Findings — must resolve before implementation

The items below were raised in plan review. They must be addressed (either fixed
in this plan or explicitly accepted with rationale) before any code lands.

### Blockers

#### B1. Wholesale `~/Library/Caches/*` deletion can corrupt running apps

The plan currently lists `User caches: ~/Library/Caches/*` as a Safe category
that is visible and selected by default. In practice that directory holds live
state for running apps (Mail offline cache, browser session, CloudKit, IM
caches, signed sessions in `HTTPStorages`/`WebKit`). Deleting blindly while an
app is open can crash the app, log the user out, or lose unsynced data.

Fix:

- Demote `User caches` to Review for v1, or
- Keep it Safe only if the provider skips any subdirectory whose bundle id
  matches a currently running process. Implement with `pgrep`/`launchctl` or
  `lsof +D`. Skipped subdirs must be reported with a clear "quit X and rerun"
  hint.
- Either way, document in this plan which cache subdirectories are considered
  safe to clear unconditionally (e.g. `Homebrew`, `pip`, `Yarn`, framework dev
  caches) versus those that need the running-app check.

#### B2. `--hard -y` with no extra gate is a one-line data loss command

`tiny clean` operates on a much larger blast radius than `tiny uninstall`
(potentially many categories × many files). The current plan inherits the
uninstall pattern where `-y` skips the hard-delete confirm. A single
`tiny clean --hard -y` could permanently destroy 30+ GB across cache, Xcode
DerivedData, Trash, and leftovers with no recovery path.

Fix:

- `--hard` without `-y` → require an interactive confirm (Y/n, default n),
  matching `uninstall.rs`.
- `--hard -y` → require an additional explicit gate. Either an env var
  `TINY_CONFIRM_HARD=1` or a flag such as `--i-know-what-im-doing`. Without
  the gate, refuse and exit non-zero.
- `--include-destructive --hard -y` → refuse outright with a clear error,
  even if the gate above is set, unless `--category` pins the destructive
  category explicitly.

#### B3. Trash provider with default "Move to Trash" action is a silent no-op

The Destructive category `User Trash: ~/.Trash` cannot be moved to Trash —
the operation is meaningless and will either silently succeed or fail with
an opaque osascript error, leaving the user believing Trash was emptied.

Fix:

- Trash provider must only accept `Hard` (empty trash) as its execute action.
- When the user selects Trash in the picker but the chosen action is Trash,
  either downgrade that single category to Hard with a printed warning, or
  abort and ask the user to choose a different action.
- Implementation must use
  `osascript -e 'tell application "Finder" to empty trash'`, not file-by-file
  deletion (slow, can hit SIP/permission errors).

### High-severity issues

#### H1. `tiny clean -y` does not define what "default selection" means

The text "moves selected/default safe items to Trash" admits at least three
interpretations: all safe categories, nothing, or only the category passed via
`--category`. This must be pinned before any code is written.

Fix: `-y` without `--category` exits with a non-zero status and the message
`error: --yes requires --category to specify scope`. `-y --category <id>`
runs the named category(ies) only. This mirrors how `tiny uninstall` requires
either a `name` argument or the picker.

#### H2. "Old/large files in Downloads/Desktop/Documents" duplicates `tiny scan`

That category overlaps with `tiny scan` (read-only, file-level) and creates
two sources of truth for thresholds. More importantly, surfacing it inside a
`MultiSelect` picker means a single space-bar press toggles thousands of
personal files for deletion — the exact failure mode the plan tries to avoid.

Fix:

- Remove the category from v1.
- Update Non-goals to state that `tiny clean` does not surface personal files
  from `Downloads`, `Desktop`, or `Documents`. The intended workflow is
  `tiny scan` to inspect, then manual `rm`/`mv` by the user.
- Revisit in a later version as a dedicated interactive flow if needed.

#### H3. `~/.cargo` "selected subdirectories" is too vague — risks deleting toolchain

`~/.cargo` contains both `bin/` (rustup toolchain binaries) and registry/git
caches. The plan does not pin which subdirectories are in scope. The same
gap applies to npm/pnpm/yarn, where hardcoded paths break for users with
custom prefixes.

Fix: pin exact paths in this plan.

- Cargo, safe to clean: `~/.cargo/registry/cache`, `~/.cargo/registry/src`,
  `~/.cargo/git/db`, `~/.cargo/git/checkouts`.
- Cargo, never touch: `~/.cargo/bin`, `~/.cargo/config.toml`,
  `~/.cargo/credentials*`. Do not touch `~/.rustup` at all.
- npm: query `npm config get cache` at runtime; do not assume `~/.npm`.
- pnpm: query `pnpm store path`.
- yarn: query `yarn cache dir`.
- If the corresponding CLI is not installed, skip the provider silently.

#### H4. Symlink-safety is not enforced at the code level

The Safety Rules mention not following symlinks, but the existing
`dir_size` helper in `uninstall.rs` uses `is_dir()`, which follows symlinks.
`fs::remove_dir_all` also walks into symlinked directories and can delete
content on the symlink target.

Fix: add a "Symlink-safe FS helpers" subsection that mandates:

- All metadata reads use `symlink_metadata()`.
- Replace `fs::remove_dir_all` with a custom walker that refuses to descend
  into symlinks.
- Add a test case that creates a symlink inside a discovered category,
  runs discovery and execution, and asserts the symlink target is untouched.

#### H5. Interaction between `--category`, `--include-review`, `--include-destructive` is undefined

Without an explicit rule, combinations like `--category trash` (with no
`--include-destructive`) or `--category caches --include-destructive` have no
clear behavior, and downstream code will encode whichever interpretation
happens first.

Fix: pin the matrix.

- `--category <id>` always overrides risk gating. The user has named the
  category explicitly, so neither `--include-review` nor `--include-destructive`
  is required.
- `--include-review` and `--include-destructive` apply only when `--category`
  is not set; they expand the default picker.
- `--category` may be repeated (`--category caches --category logs`); use
  clap `action = Append`. Avoid comma-separated values.
- Validate category ids at the CLI layer; on unknown id, print the list of
  valid ids and exit non-zero.

#### H6. No check for running apps or running builds before deletion

Deleting Xcode DerivedData mid-build, `cargo registry/src` mid-`cargo build`,
or an app's cache while the app is open can corrupt state in subtle ways.
Safety Rules currently say nothing about process state.

Fix: add to Safety Rules.

- Before discovering or executing a category tied to a specific app
  (Xcode, Cargo, Slack, etc.), check for running processes via
  `pgrep -x <name>` or equivalent.
- On a match, skip that category at execution time and log
  "<App> is running; skipping <category>. Quit <App> and rerun."
- Heuristic accuracy is acceptable; the goal is to prevent the obvious
  in-flight cases, not to be airtight.
