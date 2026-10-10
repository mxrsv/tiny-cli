# Development

> For maintainers. Using tiny? See [docs/user/](../user/).

How to build, run, test and check the repository. Architecture is in
[`../internals/`](../internals/overview.md).

## Prerequisites

- macOS with the Xcode Command Line Tools; Xcode itself is not needed. The app targets
  macOS 14 or later and is built and tested on Apple Silicon.
- Rust stable through `rustup`, 1.91 or newer: [`Cargo.lock`](../../Cargo.lock) pins
  UniFFI's `cargo-platform` 0.3.3, which requires it. The test and lint gates pass `--locked`.
- Swift 6, from the Command Line Tools.

## Commands

| Command                                                          | What it does                                                    |
| ---------------------------------------------------------------- | --------------------------------------------------------------- |
| `cargo fmt --all --check`                                        | Format gate                                                     |
| `cargo test --workspace --all-targets --locked`                  | Rust unit tests plus the CLI smoke tests                        |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Lint gate                                                       |
| [`scripts/test-native.sh`](../../scripts/test-native.sh)         | Builds the bridge, then runs the Swift checks                   |
| `cargo run -p tiny -- <command> [flags]`                         | Run the CLI without installing, e.g. `cargo run -p tiny -- sys` |
| `cargo install --path crates/tiny`                               | Install the checkout as `tiny`                                  |
| [`scripts/build-tiny-ffi.sh`](../../scripts/build-tiny-ffi.sh)   | Build the static library and regenerate the Swift bindings      |
| [`scripts/build-native-app.sh`](../../scripts/build-native-app.sh) | Build, sign and verify `macos/.build/Tiny Dev.app`            |

Run `build-tiny-ffi.sh` once in a fresh worktree and after any `tiny-ffi` API change: the
bindings are generated and not committed. `build-native-app.sh` fails if the bridge's debug
hooks reach any bundle, or smoke hooks reach the default one; `--smoke-hooks` adds the smoke
hooks.

The app binary has its own checks:

```bash
APP="macos/.build/Tiny Dev.app/Contents/MacOS/Tiny"
"$APP" --smoke-test                       # its own `sleep` child appears, then exits
"$APP" --snapshot /tmp/tiny.png [scene]   # render one screen to a PNG
"$APP" --quit-test-app /tmp/Name.app      # --smoke-hooks build only
```

- `--smoke-test` lists real processes, reads one detail and the whole-machine usage, and
  ends only the child it started.
- `--snapshot` needs an absolute `.png` path and no Screen Recording permission. Scenes
  `member` (default), `app`, `notice` and `minimum` show live processes; `clean`,
  `clean-all` and `review` run a real read-only scan, and `review` also requests a real
  preview; `review-fixture` and `report` use fixture data. No scene confirms or runs an
  action.
- `--quit-test-app` launches that disposable app and quits only the instance it launched.

## Tests

- **Rust unit tests** sit in a `#[cfg(test)] mod tests` block inside the file they test.
  External tools and running apps are injected through mocks; see the
  [clean pipeline](../internals/clean.md).
- **CLI smoke tests** in [`crates/tiny/tests/`](../../crates/tiny/tests/) drive the built
  binary with `assert_cmd`. `clean` runs only with `--dry-run` or input that fails
  validation; `processes` signals only children the test spawned. The `--port` test needs
  `/usr/sbin/lsof`, and skips when `python3` is missing.
- **Swift checks** in `test-native.sh` compile the real state and engine sources into a
  plain executable, because the Command Line Tools ship neither XCTest nor Swift Testing;
  `swift test` is not the entry point. Cleanup harnesses use `FakeCleanEngine`, so no check
  moves a file. A new Swift source must be added to the script's list.
- A test that creates files uses its own temporary directory and removes it.

## What a green run proves

[CI](../../.github/workflows/ci.yml) runs on every push and pull request: `fmt`, `test` and
`clippy` on Ubuntu, `test` and `clippy` on macOS. That covers the Rust side of `tiny-ffi`, but
CI never generates the Swift bindings, compiles Swift or runs `test-native.sh`; run that
locally before merging Swift or FFI changes.

The `cargo` gates prove flag validation, registry consistency, action mapping, symlink-safe
removal on temporary trees, provider parsing against recorded command output, and process
listing and refusals against disposable children. `test-native.sh` proves the Swift state
and copy against the real bridge and fakes. Neither proves that a provider finds the right
paths on a real Mac, that Finder accepts a Trash request, that a real `tmutil`, `docker`,
`simctl` or `go` behaves as mocked, or that the app looks right. Visual acceptance is manual:
build the app, render `--snapshot` scenes, and look at them.

## Manual checks

```bash
cargo run -p tiny -- clean --dry-run --include-review --include-destructive
cargo run -p tiny -- clean --category user-logs        # interactive, defaults to Trash
cargo run -p tiny -- uninstall SomeApp --dry-run
```

Never pass `--hard` against your real home folder to test a change. In the app, Scan and
Review… change nothing, but Move to Trash moves real files.
