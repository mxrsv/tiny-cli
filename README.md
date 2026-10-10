# tiny-cli

A small, practical Rust CLI for everyday performance and productivity utilities.
The binary is named `tiny`. The project is intentionally minimal so it stays
easy to read while it grows.

## Goals

- Keep dependencies tight (`clap`, `anyhow`, `serde`, `sysinfo`, `dialoguer`).
- Separate CLI parsing (`src/cli.rs`) from command execution (`src/commands/`).
- Provide useful, real-world commands that are safe by default.

## Build & Run

```bash
cargo build
cargo run -- --help
cargo install --path .     # put `tiny` on your PATH
```

## Commands

| Command          | What it does                                                     | Guide |
| ---------------- | ---------------------------------------------------------------- | ----- |
| `tiny sys`       | OS, host, uptime, CPU, memory and per-disk usage                 | [Getting started](docs/user/getting-started.md#sys--system-information) |
| `tiny scan`      | Report large, old and duplicate files; strictly read-only        | [Scanning files](docs/user/scan.md) |
| `tiny clean`     | Interactive cleanup of caches and recoverable data (macOS)       | [Cleaning caches](docs/user/clean.md) |
| `tiny uninstall` | Remove apps and their `~/Library` leftovers (macOS)              | [Uninstalling apps](docs/user/uninstall.md) |
| `tiny focus`     | Focus timer that logs finished sessions                          | [Getting started](docs/user/getting-started.md#focus--focus-timer) |

`clean` and `uninstall` always show a plan first, default to Move to Trash and confirm Hard
delete; only `-y --hard` skips that confirmation, and `clean` then also needs
`TINY_CONFIRM_HARD=1`.

## Documentation

The [documentation index](docs/README.md) splits pages by reader: using tiny
(`docs/user/`), how it is built (`docs/internals/`) and maintaining it
(`docs/operations/`).

## Roadmap

- `doctor` — check common dev environment issues (PATH, dotfiles, tools).
- `today` — daily summary combining focus log and calendar-style notes.
