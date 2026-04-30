# Plan: `tiny clean`

## Summary

Add a new `tiny clean` command that performs real cleanup, while keeping `tiny scan`
as the read-only file discovery command. The cleanup flow should feel similar to
`tiny uninstall`: discover candidates, show a clear report, let the user select
categories interactively, print the final plan, then ask for an action before any
files are touched.

The v1 scope is intentionally narrow: developer caches and obvious recoverable
data, no malware scanning, no app updates, no cloud cleanup, no automatic
deletion of personal files, and no app leftover sweeping (that already lives in
`tiny uninstall`).

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
- Do not surface personal files from `Documents`, `Desktop`, or `Downloads` in
  any form. The intended workflow is `tiny scan` to inspect, then manual
  `rm`/`mv` by the user.
- Do not sweep app leftovers (bundle-id-keyed `~/Library` debris) in v1. That
  logic already lives in `tiny uninstall` and is owned by that command.
- Do not touch external volume Trash (`/Volumes/*/.Trashes`) in v1. Only
  `~/.Trash`.

## User Experience

### Commands

```bash
tiny clean
tiny clean --dry-run
tiny clean --category user-logs -y
tiny clean --category trash --hard
tiny clean --include-review
tiny clean --include-destructive
```

### Default interactive flow

1. Discover cleanup candidates from the safe-by-default provider set.
2. Display categories with size, item count, and risk badge.
3. Open a checkbox picker so the user can select categories.
4. Print the cleanup plan with concrete paths (top 20 per category, summary
   for the rest).
5. Ask what action to take:
   - Move to Trash (recoverable)
   - Dry-run (no changes)
   - Hard delete (not recoverable)
   - Cancel
6. Execute only after the user chooses a non-dry-run action.

### Example picker

```text
[ ] User logs               120 MB   safe
[ ] Xcode DerivedData       9.8 GB   safe
```

When `--include-review` is set, additional rows appear (unchecked):

```text
[ ] User caches             4.2 GB   review
[ ] Xcode Archives          2.1 GB   review
[ ] Cargo cache             1.3 GB   review
[ ] npm cache               850 MB   review
```

When `--include-destructive` is set:

```text
[ ] Trash                  18.0 GB   destructive
```

## Public CLI Interface

Add a new `CleanOpts` struct and `Commands::Clean(CleanOpts)` variant.

Options:

```text
--dry-run
    Show the report and exit without prompting for cleanup. Mutually exclusive
    with -y/--yes.

-y, --yes
    Skip the picker and the action menu. Requires --category to be set, so the
    scope is always explicit. Without --hard, executes Move to Trash.

--hard
    Use permanent deletion instead of Move to Trash. Always requires either an
    interactive Y/n confirm (when -y is not set) or the env var
    TINY_CONFIRM_HARD=1 (when -y is set).

--category <id>
    Restrict discovery and selection to the named category. May be repeated:
    `--category user-logs --category xcode-derived`. Validated against the
    canonical id list below; unknown ids exit non-zero with the list printed.

--include-review
    Show review-risk categories in the picker (unchecked, with a "review"
    badge). Ignored if --category is set.

--include-destructive
    Show destructive categories such as Trash in the picker (unchecked, with
    a "destructive" badge). Ignored if --category is set.
```

### Flag interaction matrix

- `--category <id>` always overrides risk gating. Naming the category counts
  as explicit consent; `--include-review` / `--include-destructive` are
  ignored when `--category` is set.
- `--include-review` and `--include-destructive` only apply when `--category`
  is not set. They expand the default picker; they do not pre-check anything.
- `--dry-run` and `-y` are mutually exclusive (clap `conflicts_with`).
- `--hard` requires either an interactive confirm (no `-y`) or
  `TINY_CONFIRM_HARD=1` (with `-y`). Without that, exit non-zero.
- `--category trash --hard` is the only sanctioned way to empty Trash via
  this command.
- `--include-destructive --hard -y` (no `--category`) is refused outright,
  even with `TINY_CONFIRM_HARD=1`. To run a destructive category
  non-interactively the user must name it via `--category`.

### Default behavior summary

- `tiny clean` discovers and shows only safe categories.
- `tiny clean --dry-run` never prompts and never removes anything.
- `tiny clean -y` errors out, since `-y` without `--category` has no
  well-defined scope. Print: `error: --yes requires --category to specify scope`.
- `tiny clean --category <id>` runs interactively for that category.
- `tiny clean --category <id> -y` runs non-interactively, Move to Trash.
- `tiny clean --category <id> --hard -y` runs non-interactively, permanent
  delete; requires `TINY_CONFIRM_HARD=1`.

## Cleanup Categories

Each category has a stable canonical id used for `--category`, picker labels,
and tests.

### Safe (visible by default)

| id              | Label             | Path(s)                                 | Notes                                                                                                             |
| --------------- | ----------------- | --------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `user-logs`     | User logs         | `~/Library/Logs/*`                      | Skip-on-running-app does not apply; logs are append-only and not held by running processes for read-after-delete. |
| `xcode-derived` | Xcode DerivedData | `~/Library/Developer/Xcode/DerivedData` | Skip if Xcode is running.                                                                                         |

### Review (hidden until `--include-review` or `--category`)

| id                    | Label                   | Path(s)                                                                                         | Notes                                                                                                                                                    |
| --------------------- | ----------------------- | ----------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `user-caches`         | User caches             | `~/Library/Caches/*`                                                                            | Demoted to Review (see B1). Skip subdirs whose bundle id matches a running process.                                                                      |
| `xcode-archives`      | Xcode Archives          | `~/Library/Developer/Xcode/Archives`                                                            | User may want these for crash symbolication. Skip if Xcode is running.                                                                                   |
| `xcode-devicesupport` | Xcode iOS DeviceSupport | `~/Library/Developer/Xcode/iOS DeviceSupport`                                                   | Re-downloaded on next device attach. Skip if Xcode is running.                                                                                           |
| `cargo`               | Cargo cache             | `~/.cargo/registry/cache`, `~/.cargo/registry/src`, `~/.cargo/git/db`, `~/.cargo/git/checkouts` | Pinned subdirs only. Never touch `~/.cargo/bin`, `~/.cargo/config.toml`, `~/.cargo/credentials*`, or `~/.rustup`. Skip if `cargo` or `rustc` is running. |
| `npm`                 | npm cache               | `npm config get cache` (runtime query)                                                          | Skip provider entirely if `npm` is not on PATH.                                                                                                          |
| `pnpm`                | pnpm store              | `pnpm store path` (runtime query)                                                               | Skip provider entirely if `pnpm` is not on PATH. Skip if `pnpm` or `node` running with pnpm context — heuristic, just check for `pnpm` process.          |
| `yarn`                | yarn cache              | `yarn cache dir` (runtime query)                                                                | Skip provider entirely if `yarn` is not on PATH.                                                                                                         |

### Destructive (hidden until `--include-destructive` or `--category`)

| id      | Label      | Path(s)    | Notes                                                                                                                                                                                                                     |
| ------- | ---------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `trash` | User Trash | `~/.Trash` | Only accepts `ExecAction::EmptyTrash` (mapped from UI `Hard delete`). Rejects `Trash` and `HardDelete`. See B3. Implementation uses `osascript -e 'tell application "Finder" to empty trash'`, not file-by-file deletion. |

### Excluded from v1

- Old/large file candidates from `Downloads`, `Desktop`, `Documents` —
  duplicates `tiny scan` and creates a one-keypress path to deleting personal
  files. Use `tiny scan` instead.
- App leftovers by bundle id — already covered by `tiny uninstall`. Will be
  considered in a later version after a shared helper refactor.

## Architecture

Use a provider-based structure so each cleanup source is isolated.

Suggested layout:

```text
src/commands/clean/
  mod.rs          // run(opts), top-level orchestration
  cli_validate.rs // flag matrix validation (matrix above)
  types.rs        // CleanItem, CleanAction, RiskLevel, ExecReport
  fs_safe.rs      // symlink-safe size + delete helpers
  process.rs      // running-app detection (pgrep wrapper)
  discover.rs     // build provider list from CLI options
  picker.rs       // dialoguer MultiSelect + Select for action
  report.rs       // print plan, format columns, truncation
  execute.rs      // dispatches per-provider execute
  providers/
    mod.rs        // CleanProvider trait, registry
    user_logs.rs
    user_caches.rs
    xcode.rs      // DerivedData, Archives, DeviceSupport
    dev_caches.rs // cargo, npm, pnpm, yarn
    trash.rs
```

Core model:

```rust
pub trait CleanProvider {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn risk(&self) -> RiskLevel;

    /// Returns the running process name to check before discover/execute.
    /// None means no app gating.
    fn requires_app_quit(&self) -> Option<&'static str> { None }

    /// Returns true if the provider should not even be listed (e.g. CLI not
    /// installed). Different from "discovered nothing".
    fn available(&self) -> bool { true }

    fn discover(&self) -> anyhow::Result<Vec<CleanItem>>;

    /// Default: per-item delete via the action. Override for trash (empty
    /// trash) or any provider with non-trivial semantics.
    fn execute(&self, items: &[CleanItem], action: ExecAction)
        -> anyhow::Result<ExecReport>;
}

pub enum RiskLevel { Safe, Review, Destructive }

pub struct CleanItem {
    pub category_id: String,
    pub category_label: String,
    pub path: PathBuf,
    pub size: u64,
    pub risk: RiskLevel,
}

/// What reaches a provider's execute(). Three semantics, distinct on
/// purpose so providers can't accidentally conflate them:
/// - `Trash`: move each item to ~/.Trash via Finder.
/// - `HardDelete`: file-by-file recursive permanent removal.
/// - `EmptyTrash`: invoke Finder's "empty trash" — only Trash provider
///   accepts this, and it accepts only this.
pub enum ExecAction { Trash, HardDelete, EmptyTrash }

/// What the user sees in the action menu. The orchestrator (`execute.rs`)
/// maps `HardDelete` → `ExecAction::EmptyTrash` for the Trash provider, and
/// → `ExecAction::HardDelete` for everyone else.
pub enum CleanAction { Trash, HardDelete, DryRun, Cancel }

pub struct ExecReport {
    pub removed_paths: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
    pub skipped_running_app: Option<String>, // app name if skipped
}
```

Command flow:

1. Validate CLI flag combinations (matrix above). Reject early.
2. Build provider list from CLI options. Skip unavailable providers.
3. For each provider, if `requires_app_quit` matches a running process, skip
   discovery and remember the skip reason for the final report.
4. Discover items from remaining providers.
5. Group by category and calculate size totals.
6. If `--dry-run`, print summary and detailed report, exit.
7. If `-y` and `--category`: build plan from named categories, print plan,
   skip picker, skip action menu, execute action implied by `--hard`.
8. Otherwise: show picker, then build plan from selected categories, print
   plan, show action menu, execute.
9. If Trash category is in the plan but the chosen `CleanAction` is `Trash`,
   abort with a clear error: "trash category requires --hard".
10. Map `CleanAction` to per-provider `ExecAction`:
    - `CleanAction::Trash` → `ExecAction::Trash` for every provider. Trash
      provider rejects this in step 9 above.
    - `CleanAction::HardDelete` → `ExecAction::EmptyTrash` for the Trash
      provider, `ExecAction::HardDelete` for everyone else.
11. Execute via per-provider `execute()`. Each provider validates the
    incoming `ExecAction` and rejects unsupported variants (Trash provider
    rejects everything except `EmptyTrash`; other providers reject
    `EmptyTrash`). Continue on per-item failure; summarize at end.

## Safety Rules

- Never remove anything before a plan is printed.
- Prefer Move to Trash by default.
- Require explicit `--hard` for permanent deletion.
- `--hard` requires either interactive confirm (Y/n, default n) or
  `TINY_CONFIRM_HARD=1` if combined with `-y`.
- Destructive categories are hidden from the default picker. Reaching them
  requires either `--include-destructive` (interactive only) or
  `--category <id>`.
- Skip paths that do not exist.
- Symlink-safe FS only. See "Symlink-safe FS helpers" below.
- Refuse system paths in v1.
- Do not surface personal files from `Downloads`/`Desktop`/`Documents`.
- Skip categories whose `requires_app_quit` matches a running process. Report
  the skip in the final summary with a "Quit <App> and rerun" hint.
- If one item fails to remove, continue with the rest and report failures at
  the end.

### Symlink-safe FS helpers (`fs_safe.rs`)

`uninstall.rs::dir_size` uses `is_dir()` which follows symlinks. We will not
copy that pattern. The helpers in `fs_safe.rs` must:

- Use `fs::symlink_metadata()` for every metadata read.
- Provide `walk_no_follow(root)` that yields entries without descending into
  symlinks. Symlink entries themselves are yielded, but their target is not
  walked.
- Provide `dir_size_safe(root)` built on `walk_no_follow`.
- Provide `remove_recursive_safe(root)` that walks with no-follow, removes
  files and symlinks via `fs::remove_file`, then `fs::remove_dir` for
  directories bottom-up. Never use `fs::remove_dir_all`.
- Add a regression test: create a symlink inside a temp tree pointing at a
  separate temp tree; run discovery + execute with `Hard`; assert the
  separate tree is untouched.

### Running-app detection (`process.rs`)

- Single helper: `is_running(process_name: &str) -> bool` using
  `pgrep -x <name>` (exit code 0 = running). Wrap in a trait or take a
  callable so tests can inject.
- Process names per provider:
  - `xcode-derived`, `xcode-archives`, `xcode-devicesupport` → `Xcode`
  - `cargo` → check both `cargo` and `rustc`
  - `npm` → `npm`
  - `pnpm` → `pnpm`
  - `yarn` → `yarn`
- For `user-caches`: do **not** gate the whole provider on a single process.
  Instead, during discovery, list the immediate children of `~/Library/Caches`
  and skip any subdirectory whose name (typically a bundle id) matches a
  currently running app via `lsof` or `launchctl list`. v1 may use a simpler
  heuristic: check `pgrep -f <last-segment-of-bundle-id>`. Document this is
  best-effort, not airtight.

## README Updates

Update README to describe the distinction:

- `tiny scan`: read-only report for old and large files.
- `tiny clean`: interactive cleanup with selectable categories and recoverable
  default behavior.

Include examples and a note:

```bash
tiny clean
tiny clean --dry-run
tiny clean --include-review
tiny clean --category user-logs
tiny clean --category trash --hard
```

> Note: even Move to Trash is destructive if macOS "Empty Trash automatically"
> (System Settings → General → Storage) is on. Use `--dry-run` first or
> disable that setting.

## Test Plan

Automated tests:

- Provider discovery returns empty results when target paths do not exist.
- Size aggregation by category is correct.
- Risk filtering: default picker shows only Safe.
- `--include-review` adds Review categories to picker.
- `--include-destructive` adds Destructive categories to picker.
- `--dry-run` never invokes execute paths.
- `-y` without `--category` exits non-zero.
- `-y --category <id>` runs without picker or action menu.
- `--hard` without `-y` and without confirm exits non-zero.
- `--hard -y` without `TINY_CONFIRM_HARD=1` exits non-zero.
- `--include-destructive --hard -y` (no `--category`) exits non-zero.
- Trash provider rejects `ExecAction::Trash` and `ExecAction::HardDelete`;
  accepts only `ExecAction::EmptyTrash`.
- Non-trash providers reject `ExecAction::EmptyTrash`.
- Action mapping in `execute.rs`: UI `HardDelete` maps to
  `ExecAction::EmptyTrash` for the Trash provider and `ExecAction::HardDelete`
  for every other provider.
- `--category` with unknown id exits non-zero with the valid id list printed.
- `--category` may be repeated.
- Cargo provider only includes pinned subdirs; never includes `bin/`,
  `config.toml`, `credentials*`, or any `~/.rustup` path.
- `walk_no_follow` does not descend into symlinks; `remove_recursive_safe`
  does not delete symlink targets (regression test described above).
- Running-app gating: when injected `is_running` returns true for `Xcode`,
  the Xcode providers are skipped at discovery and reported.
- Missing paths are skipped without failing the whole command.

Manual checks:

```bash
cargo test
cargo run -- --help
cargo run -- clean --dry-run
cargo run -- clean --category user-logs --dry-run
cargo run -- clean --include-review --dry-run
cargo run -- clean --include-destructive --dry-run
cargo run -- clean
TINY_CONFIRM_HARD=1 cargo run -- clean --category trash --hard -y
```

## Rollout Plan

1. Add CLI type and command dispatch (`CleanOpts`, `Commands::Clean`).
2. Add clean module skeleton, core types, `fs_safe`, `process`.
3. Add CLI flag-matrix validation with error tests.
4. Implement Safe providers first (`user-logs`, `xcode-derived`).
5. Add report and interactive picker.
6. Add execution with Trash and hard delete (with all gates from B2).
7. Implement Review providers behind `--include-review` / `--category`.
8. Implement Trash provider with `--include-destructive` / `--category trash`.
9. Update README.
10. Run tests and manually verify the CLI help/output.

## Open Decisions

(No outstanding open decisions for v1; the original three were resolved in the
final plan above.)

- Resolved: review/destructive are hidden by default, surfaced only via
  `--include-*` or `--category`.
- Resolved: `--category` accepts multiple values via repeat (`Append`).
- Resolved: app leftovers are deferred from v1; `tiny uninstall` keeps that
  responsibility.

## Changelog

- v1 plan after review: applied 3 blockers (B1 demote user caches to Review;
  B2 add `TINY_CONFIRM_HARD` gate for `--hard -y`; B3 trash provider only
  accepts Hard action and uses Finder empty-trash) and 6 high-severity fixes
  (H1 `-y` requires `--category`; H2 dropped Downloads/Desktop/Documents
  category; H3 pinned exact cargo paths and runtime-query for npm/pnpm/yarn;
  H4 dedicated symlink-safe FS helpers; H5 pinned flag interaction matrix;
  H6 running-app skip rule).
- Post-review tweak: split `ExecAction::EmptyTrash` out from `HardDelete` so
  Trash provider and file-by-file providers can't conflate semantics. UI
  action menu unchanged; mapping happens in `execute.rs`.
