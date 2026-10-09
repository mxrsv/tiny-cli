# Architecture overview

> For maintainers. Using tiny? See [docs/user/](../user/).

tiny is one Rust binary crate. [`src/cli.rs`](../../src/cli.rs) holds every clap
definition, [`src/main.rs`](../../src/main.rs) matches the subcommand and calls
`commands::<name>::run(opts)`, and each command owns its module under
[`src/commands/`](../../src/commands/). The only shared code is byte and duration formatting
in [`src/util.rs`](../../src/util.rs). Commands do not call each other.

## Adding a command

Add a variant to `Commands` with its own `*Opts` struct in `cli.rs`, one match arm in
`main.rs`, and a module that exposes `run`. Validate flag combinations before doing any I/O,
as [`clean`](../../src/commands/clean/cli_validate.rs) does, so a bad invocation fails before
discovery.

## Read-only versus destructive

`sys`, `scan` and `focus` never remove anything; `scan` is read-only by contract and must
stay that way. `clean` and `uninstall` are the two commands that remove data, and they share
one shape: discover, print a plan, ask for an action, act.

- **Move to Trash** goes through Finder with `osascript`, passing the path as an argument
  rather than interpolating it into AppleScript, so Put Back works and quoting cannot break.
  `clean` and `uninstall` each carry a copy of this helper.
- **Hard delete** in `clean` goes through
  [`remove_recursive_safe`](../../src/commands/clean/fs_safe.rs), which removes bottom-up and
  never follows a symlink. `uninstall` has its own `hard_remove`: it unlinks a symlink and
  otherwise calls `fs::remove_dir_all` — see [traps](traps.md).
- **Confirmation** is a separate prompt for a permanent action chosen interactively. With
  `-y` there is no prompt: `clean --hard -y` demands `TINY_CONFIRM_HARD=1` instead, and
  `uninstall --hard -y` demands nothing.

The clean pipeline, its provider registry and its action mapping are in
[clean.md](clean.md).

## Platform

Most commands assume macOS: Finder for Trash, `pgrep`, `mdls`, `tmutil`, `defaults` and
`/Applications`. `sys` uses `sysinfo`, `scan` only walks directories and `focus` writes a
JSON file under `$HOME`, so those three are the portable ones. There is no `cfg(target_os)` gating; nothing stops a build elsewhere.

## Desktop migration

A native macOS app is planned, not shipped. The current direction is the
[SwiftUI and shared Rust core spec](../specs/2026-10-07-swiftui-cli-boundary.md): a
`tiny-core` crate holds the operations, the CLI and a SwiftUI app both call it, the app
through UniFFI. That work lives on branch `feat/native-desktop`, not on `main`.

[`_bmad-output/planning-artifacts/architecture.md`](../../_bmad-output/planning-artifacts/architecture.md)
and [`_bmad-output/project-context.md`](../../_bmad-output/project-context.md) still describe
the earlier Tauri + React plan; where they disagree with the spec, the spec wins.
