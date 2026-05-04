# tiny-cli

A small, practical Rust CLI for everyday performance and productivity utilities.
The binary is named `tiny`. The project is intentionally minimal so it stays
easy to read while it grows.

## Goals

- Keep dependencies tight (`clap`, `anyhow`, `serde`, `sysinfo`).
- Separate CLI parsing (`src/cli.rs`) from command execution (`src/commands/`).
- Provide useful, real-world commands that are safe by default.

## Build & Run

```bash
cargo build
cargo run -- --help
```

## Commands

### `sys` — system information

```bash
cargo run -- sys
```

Reports OS, host, uptime, CPU count and model, memory usage, and per-disk
usage.

### `scan` — scan common folders, report only

```bash
cargo run -- scan
cargo run -- scan --min-size-mb 100 --older-than-days 30
```

Scans `~/Downloads`, `~/Desktop`, and `~/Documents`. Reports the largest files
and the oldest files that exceed the thresholds. **`scan` is strictly
read-only** — it never deletes anything. For cleanup, see `clean` below.

Options:

- `--min-size-mb` (default `100`): report files at or above this size.
- `--older-than-days` (default `90`): report files older than this many days.

### `clean` — interactive cleanup of caches and recoverable data (macOS)

```bash
tiny clean                              # picker → plan → action menu
tiny clean --dry-run                    # show plan and exit, no prompt
tiny clean --include-review             # also list review-risk caches
tiny clean --include-destructive        # also list Trash + tmutil snapshots
tiny clean --review-paths               # drill-down picker per path before action
tiny clean --idle-days 60               # raise idle threshold for project caches
tiny clean --category node-modules      # interactive, scoped to one category
tiny clean --category trash --hard      # the only sanctioned way to empty Trash
TINY_CONFIRM_HARD=1 tiny clean --category cargo --hard -y   # non-interactive permanent delete
```

Categories are grouped into three families in the picker (31 total):

**Dev caches (17)** — `cargo`, `npm`, `pnpm`, `yarn`, `node-modules`,
`python-caches`, `rust-targets`, `gradle-maven`, `jetbrains`, `vscode`,
`ios-simulators`, `android-sdk`, `go-cache`, `docker`, `xcode-derived`,
`xcode-archives`, `xcode-devicesupport`.

**User storage (6)** — `downloads-old`, `screenshots-old`,
`mail-attachments`, `streaming-caches`, `chat-caches`, `browser-caches`.

**System leftovers (8)** — `user-logs`, `user-caches`, `trash`,
`quarantine`, `crash-reports`, `app-orphans`, `time-machine-local`,
`font-quicklook-caches`.

The default picker shows only **safe** categories. Add `--include-review`
for developer caches and other items that may want manual review. Add
`--include-destructive` to surface `trash` and `time-machine-local`
(snapshots are not recoverable). Use `--category <id>` to target a single
category — repeating the flag is fine (`--category user-logs --category
xcode-derived`).

**Flags that affect discovery:**

- `--idle-days N` (default `30`): only flag project caches whose project
  manifest hasn't been touched in the last `N` days. Applies to
  `node-modules`, `python-caches`, `rust-targets`, `android-sdk`,
  `downloads-old`, `screenshots-old`. `N` must be `> 0`.
- `--review-paths`: after the family picker, opens a per-path drill-down
  so you can deselect individual paths before the action runs. Also
  available mid-flow as the **Review paths** entry in the action menu.

**Safety model:**

- Default action is **Move to Trash** (recoverable).
- `--hard` is permanent; without `-y` it requires a Y/n confirmation, with
  `-y` it requires `TINY_CONFIRM_HARD=1` in the environment.
- `--yes` requires `--category` so the scope is always explicit.
- The Trash provider is wired so only `Hard delete` reaches it (mapped to
  Finder's empty-trash); `Move to Trash` for the trash category is rejected.
- Discovery skips a category whose owning app is currently running
  (e.g. `xcode-derived` when Xcode is open).
- All filesystem walks are symlink-safe — symlinked directories are never
  followed, so we cannot delete data outside the listed paths.

> Note: even Move to Trash is destructive if macOS "Empty Trash automatically"
> (System Settings → General → Storage) is on. Use `--dry-run` first or
> disable that setting.

`scan` vs `clean` at a glance: `scan` is read-only and surfaces personal
files in `~/Downloads` / `~/Desktop` / `~/Documents` so you can review and
decide. `clean` is interactive deletion of developer caches and recoverable
data. They are intentionally separate — personal files never appear in
`clean`.

### `focus` — local focus timer

```bash
cargo run -- focus --minutes 25
cargo run -- focus --minutes 50 --label "deep work"
```

Runs a synchronous timer with a simple progress bar. When the session ends,
the entry is appended to `~/.tiny-cli/focus-sessions.json` so you have a
record of completed sessions.

### `uninstall` — remove apps and their `~/Library` leftovers (macOS)

```bash
tiny uninstall                       # picker → report → action menu
tiny uninstall AltTab                # report → action menu
tiny uninstall AltTab --dry-run      # report only, no prompt
tiny uninstall AltTab -y             # skip menu, move to Trash immediately
tiny uninstall AltTab --hard -y      # skip menu, rm -rf (advanced)
tiny uninstall AltTab --shallow      # only /Applications/<Name>.app
tiny uninstall AltTab --leftovers-only  # only ~/Library cleanup
```

After the report, an action menu lets you choose:

- **Move to Trash (recoverable)** — default, safe
- **Dry-run (no changes)**
- **Hard delete (NOT recoverable)** — extra confirm prompt before touching anything
- **Cancel**

**Safety:**

- Default action highlights "Move to Trash" — you must press Enter, nothing auto-runs.
- Trash is recoverable; hard delete requires a separate `[y/N]` confirmation.
- Refuses system apps (`com.apple.*`) and Homebrew casks (use `brew uninstall --cask`; pass `--force` to override).

**Picker sorts:** `--sort=last-used` (default), `size`, `name`.

## Roadmap

- `files` — richer filesystem analytics (duplicates, by-extension breakdowns).
- `doctor` — check common dev environment issues (PATH, dotfiles, tools).
- `today` — daily summary combining focus log and calendar-style notes.
