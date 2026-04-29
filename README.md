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

## Roadmap

- `files` — richer filesystem analytics (duplicates, by-extension breakdowns).
- `doctor` — check common dev environment issues (PATH, dotfiles, tools).
- `today` — daily summary combining focus log and calendar-style notes.
