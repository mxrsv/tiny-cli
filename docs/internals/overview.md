# Architecture overview

> For maintainers. Using tiny? See [docs/user/](../user/).

tiny is a Cargo workspace of three crates plus a SwiftUI app. The
[boundary spec](../specs/2026-10-07-swiftui-cli-boundary.md) sets the split: every operation has
one implementation in Rust, and each front end only parses, presents and integrates.

| Part                                             | Owns                                                                                                                                 | Must not                                   |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------ |
| [`tiny-core`](../../crates/tiny-core/src/lib.rs) | Every operation (clean, processes, scan, uninstall, sys, focus), with the typed [`Error`](../../crates/tiny-core/src/error.rs)       | Parse flags, print, prompt or use `anyhow` |
| [`tiny`](../../crates/tiny/src/main.rs)          | The CLI: clap in [`cli.rs`](../../crates/tiny/src/cli.rs), prompts and output in [`render/<command>`](../../crates/tiny/src/render/mod.rs) | Implement an operation                     |
| [`tiny-ffi`](../../crates/tiny-ffi/src/lib.rs)   | UniFFI records, the typed `FfiError` and the per-app [`TinySession`](../../crates/tiny-ffi/src/session.rs)                            | Let a `tiny-core` type cross into Swift    |
| [`macos/`](../../macos/Package.swift)            | SwiftUI views and state, the [`Engine`](../../macos/Sources/TinyEngine/Engine.swift) actor, macOS integration such as app Quit       | Reimplement a process or cleanup operation |

## Adding a command

Put the operation in `tiny-core`, with serde options in
[`options.rs`](../../crates/tiny-core/src/options.rs) when both front ends pass them. In `tiny`,
add a `Commands` variant with its own `*Opts` struct and `From` conversion in `cli.rs`, one match
arm in `main.rs`, and a `render/<command>` module whose `run` calls core and prints. Validate
flag combinations before any I/O, as
[`cli_validate.rs`](../../crates/tiny/src/render/clean/cli_validate.rs) does; core re-checks
what the app can send too. To reach the app, add a `#[uniffi::export]` method on `TinySession`
returning `Ffi*` records, claim the session gate (`begin()`) if it scans or mutates, regenerate
the bindings with [`build-tiny-ffi.sh`](../../scripts/build-tiny-ffi.sh), and call it through
the `Engine` actor.

## Read-only versus destructive

`sys`, `scan`, `focus` and process listing never remove or signal anything; `scan` is read-only
by contract. `clean` and `uninstall` remove data and `processes quit` sends signals; each shows
what it will do, asks, then acts.

- **Move to Trash** asks Finder through `osascript` with the path as an argument, never
  interpolated, so Put Back works and quoting cannot break. There are three copies: the app's
  [`FinderTrash`](../../crates/tiny-core/src/clean/finder_trash.rs), the CLI clean's
  [`move_to_trash`](../../crates/tiny-core/src/clean/providers/mod.rs) and an inline one in
  [`uninstall.rs`](../../crates/tiny-core/src/uninstall.rs).
- **Hard delete** exists only in the CLI. `clean` and `uninstall` remove through
  [`remove_recursive_safe`](../../crates/tiny-core/src/clean/fs_safe.rs), bottom-up and never
  following a symlink; tool-backed categories run their tool instead.
- **Confirmation** is a separate prompt for a permanent action chosen interactively. With `-y`
  there is none: `clean --hard -y` demands `TINY_CONFIRM_HARD=1`, `processes quit --force -y`
  demands `TINY_CONFIRM_FORCE=1` ([`processes.rs`](../../crates/tiny/src/render/processes.rs)),
  and `uninstall --hard -y` demands nothing
  ([`uninstall.rs`](../../crates/tiny/src/render/uninstall.rs)).
- **Signals** go only to a process whose identity still matches a fresh lookup, with no
  escalation or tree kill ([`terminate.rs`](../../crates/tiny-core/src/processes/terminate.rs)).

The clean pipeline, its registry and the desktop's checked Trash are in [clean.md](clean.md).

## Platform

`clean`, `uninstall` and port lookups assume macOS tools: Finder, `pgrep`, `mdls`, `tmutil`,
`xcrun`, `lsof`, `git`. Code the app reaches spawns them by absolute path or through
`ToolLookup::app`, because a Finder-launched app gets a minimal `PATH`
([`runner.rs`](../../crates/tiny-core/src/runner.rs)). There is no `cfg(target_os)` gating: the
workspace builds on Linux and fails at run time wherever it shells out to a macOS tool.

## Desktop app

This branch has a SwiftUI app with two [screens](../../macos/Sources/Tiny/AppState.swift):
Activity (processes grouped by app, listening ports) and Clean (scan, review sheet, Move to
Trash). It links `tiny-core` in-process through
`tiny-ffi` and is built locally as the ad-hoc signed `Tiny Dev.app` by
[`build-native-app.sh`](../../scripts/build-native-app.sh); there is no release. `main` still
holds the older single-crate CLI without the app. Scope is the
[MVP spec](../specs/2026-10-06-processes-clean-mvp.md); the Clean screen's evidence and trust
sections follow the [clean trust spec](../specs/2026-10-09-clean-trust-evidence.md).
