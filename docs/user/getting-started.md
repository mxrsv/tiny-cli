# Getting started

`tiny` is a small command-line tool for everyday upkeep of a Mac: what the machine is doing,
which files are taking space, which caches can go, and which apps left data behind. Every
command that can remove something shows you a plan first and defaults to Move to Trash.

## Install

tiny is built from source. You need a Rust toolchain (`rustup` installs one).

```bash
git clone https://github.com/mxrsv/tiny-cli
cd tiny-cli
cargo install --path .     # puts `tiny` on your PATH (~/.cargo/bin)
```

To try it without installing, replace `tiny` with `cargo run --` in every example.

`sys`, `scan` and `focus` only read your system or write their own log. `clean` and
`uninstall` move things to the Trash through Finder, so they need macOS; the first run may
ask you to allow your terminal to control Finder.

## Commands

| Command          | What it does                                                    |
| ---------------- | --------------------------------------------------------------- |
| `tiny sys`       | OS, host, uptime, CPU, memory and per-disk usage                |
| `tiny scan`      | Large and old files in your folders, never deletes — see [scan](scan.md) |
| `tiny clean`     | Interactive cleanup of caches and leftovers — see [clean](clean.md) |
| `tiny uninstall` | Remove an app and its `~/Library` data — see [uninstall](uninstall.md) |
| `tiny focus`     | A focus timer that logs finished sessions                       |

`tiny --help` and `tiny <command> --help` list every flag.

## `sys` — system information

```bash
tiny sys
```

Prints one snapshot and exits. Nothing is written anywhere.

## `focus` — focus timer

```bash
tiny focus                          # 25 minutes
tiny focus --minutes 50 --label "deep work"
```

Shows a progress bar until the session ends, then appends it to
`~/.tiny-cli/focus-sessions.json`. Stopping early with Ctrl-C records nothing.

If that file is not valid JSON, tiny refuses to overwrite it: it copies the file to
`focus-sessions.json.bak` and asks you to fix or delete the original.
