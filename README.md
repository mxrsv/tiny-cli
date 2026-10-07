# tiny-cli

A local macOS desktop app and Rust CLI for everyday performance and productivity.
The desktop uses Tauri 2 + React 19; its engine is shared with the `tiny` CLI.
All filesystem operations stay on your machine.

## Goals

- Keep the engine independent of the GUI and async runtime.
- Separate CLI parsing/presentation (`crates/tiny`) from computation (`crates/tiny-core`).
- Provide useful, real-world commands that are safe by default.

## Build & Run

### macOS desktop

Requirements: macOS 13+, Node.js 22+, Rust stable, Xcode Command Line Tools
(`xcode-select --install`).

```bash
npm ci
npm run tauri dev
```

To create an ad-hoc signed `.app` and `.dmg`:

```bash
npm run tauri build
```

Output is under `target/release/bundle/`. Developer ID signing and notarization
are deferred; the current bundle is for local development.

The desktop includes Smart Scan and an explained Health Score, cleanup with
three risk lanes and path preview, Space Lens treemap/sunburst, live activity,
scan history, a storage forecast, undo history and background scan preferences.
It checks Full Disk Access at startup and guides you to System Settings when
macOS denies access. The `unknown` state means access could not be verified;
results may omit protected locations.

Closing the window hides Tiny to its menubar tray. Use **Quit Tiny** in the tray
to exit. Scheduled scans run while the app remains open, and optional launch at
login keeps the tray available. Rules send notifications and never delete files.
Settings use the Tauri store; history and cleanup journals use SQLite under the
app data directory (normally `~/Library/Application Support/com.mxrsv.tiny/`).
The most recent 120 scan snapshots are retained.

Cleanup always needs a preview and explicit confirmation. Files first go to
macOS Trash (restore with Finder **Put Back**); if Trash refuses the move, Tiny
tries its own quarantine on the same volume (restore from **Undo & recovery**).
If neither move works, the source is preserved and the failure is shown. Preview
plans expire after ten minutes and cannot be replayed. Restoration never
overwrites an existing path. Quarantine entries have a 30-day retention marker
and are kept at least that long; this version does not purge them automatically.
Finder controls its own Trash retention.

The GUI keeps Docker prune, Trash emptying and Time Machine snapshot deletion
report-only because they cannot be undone. Health ranking uses risk and size;
the score is a heuristic indicator, not a hardware diagnosis or an AI service.
Space Lens is read-only and reports skipped paths and limits. It retains the
200 largest entries per folder and expands deeper branches on demand; it scans
to a maximum depth of 64. The whole-disk estimate can differ from macOS Storage
because of APFS snapshots/shared blocks and inaccessible folders.

### Browser preview

```bash
npm run dev
```

Open `http://localhost:1420`. Outside Tauri this is an explicitly labelled
read-only demo with sample data. It does not access your files; cleanup, restore
and preferences writes are disabled. The production frontend builds with
`npm run build`.

### CLI

```bash
cargo build -p tiny
cargo run -p tiny -- --help
```

CLI source/presentation is in `crates/tiny/src/`; computation and providers live
in `crates/tiny-core/src/`. `src/` is the React frontend, and `src-tauri/` owns
native IPC, permissions, persistence, background work and recoverable moves.

### Validation

```bash
npm run build
npm run test
cargo fmt --all --check
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

On macOS build the frontend before Cargo checks: the Tauri context embeds
`dist/`. Linux checks cover core, CLI and portable app services; the native
Tauri shell is macOS-only. GitHub Actions also has a macOS test/build job.

For native smoke testing: launch Tiny, verify the FDA denied/unknown/granted
states, run Smart Scan, preview a disposable cache path and confirm the move,
restore it via Finder or Tiny, and verify the tray survives closing the window.
Check notification permissions, launch at login, scheduled scan settings and a
read-only Space Lens scan on both a directory and `/`.

See `docs/plans/2026-10-06-tiny-desktop.md` for implementation scope and limits.

## Commands

### `sys` — system information

```bash
cargo run -p tiny -- sys
```

Reports OS, host, uptime, CPU count and model, memory usage, and per-disk
usage.

### `scan` — scan common folders, report only

```bash
cargo run -p tiny -- scan
cargo run -p tiny -- scan --min-size-mb 100 --older-than-days 30
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
cargo run -p tiny -- focus --minutes 25
cargo run -p tiny -- focus --minutes 50 --label "deep work"
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
