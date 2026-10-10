# tiny-cli

Small, practical tools for everyday upkeep of a Mac: the `tiny` command-line tool and the
Tiny desktop app. Both run on one Rust engine, show you what they found before changing
anything, and keep everything on your machine.

## Build & Run

Requirements: macOS and Rust 1.91 or newer (`rustup`). The desktop app also needs macOS 14
or later, Swift 6 and the Xcode Command Line Tools; it is built and tested on Apple Silicon.

### CLI

```bash
cargo build -p tiny
cargo run -p tiny -- --help
cargo install --path crates/tiny     # put `tiny` on your PATH
```

### Desktop app

```bash
scripts/build-native-app.sh
open "macos/.build/Tiny Dev.app"
```

The script builds and signs `Tiny Dev.app` for this Mac without installing it; copy it into
`/Applications` to keep it.

## Commands

| Command          | What it does                                                      | Guide |
| ---------------- | ----------------------------------------------------------------- | ----- |
| `tiny sys`       | OS, host, uptime, CPU, memory and per-disk usage                  | [Getting started](docs/user/getting-started.md#sys--system-information) |
| `tiny scan`      | Report large, old and duplicate files; strictly read-only         | [Scanning files](docs/user/scan.md) |
| `tiny clean`     | Interactive cleanup of caches and leftovers (macOS)               | [Cleaning caches](docs/user/clean.md) |
| `tiny uninstall` | Remove apps and their `~/Library` leftovers (macOS)               | [Uninstalling apps](docs/user/uninstall.md) |
| `tiny processes` | List, inspect and quit your processes; find who listens on a port | [Processes](docs/user/processes.md) |
| `tiny focus`     | Focus timer that logs finished sessions                           | [Getting started](docs/user/getting-started.md#focus--focus-timer) |
| Desktop app      | Apps & activity and Clean in one window                           | [Desktop app](docs/user/desktop.md) |

`clean` and `uninstall` show a plan first, default to Move to Trash and confirm Hard delete;
only `-y --hard` skips that confirmation, and `clean` then also needs `TINY_CONFIRM_HARD=1`.
Docker, Time Machine and simulator-device cleanup is permanent whichever action you pick.
The desktop app never deletes permanently: it only moves items to the Trash.

## Documentation

The [documentation index](docs/README.md) splits pages by reader: using tiny
(`docs/user/`), how it is built (`docs/internals/`) and maintaining it
(`docs/operations/`).

## Roadmap

- `doctor` — check common dev environment issues (PATH, dotfiles, tools).
- `today` — daily summary combining focus log and calendar-style notes.
