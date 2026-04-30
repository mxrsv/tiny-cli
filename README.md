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

### `clean` — scan common folders, report only

```bash
cargo run -- clean
cargo run -- clean --min-size-mb 100 --older-than-days 30
```

Scans `~/Downloads`, `~/Desktop`, and `~/Documents`. Reports the largest files
and the oldest files that exceed the thresholds. **No files are deleted in
v1** — this command is read-only.

Options:

- `--min-size-mb` (default `100`): report files at or above this size.
- `--older-than-days` (default `90`): report files older than this many days.

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
