# Development

> For maintainers. Using tiny? See [docs/user/](../user/).

How to build, run, test and check the repository. Architecture is in
[`../internals/`](../internals/overview.md).

## Prerequisites

A stable Rust toolchain through `rustup`, on macOS. Dependencies are declared in
[`Cargo.toml`](../../Cargo.toml); there is no committed lock file.

## Commands

| Command                                     | What it does                                       |
| ------------------------------------------- | -------------------------------------------------- |
| `cargo build`                               | Debug build of `tiny` in `target/debug/`           |
| `cargo run -- <command> [flags]`            | Run without installing, e.g. `cargo run -- sys`    |
| `cargo test --all-targets`                  | Inline unit tests plus `tests/clean_smoke.rs`      |
| `cargo clippy --all-targets -- -D warnings` | The lint gate                                      |
| `cargo install --path .`                    | Install the current checkout as `tiny`             |

## Tests

- **Unit tests** sit in a `#[cfg(test)] mod tests` block inside the file they test. External
  processes and running apps are injected through `MockRunner` and `MockChecker`; see
  [clean.md](../internals/clean.md#test-seams).
- **Integration tests** in [`tests/`](../../tests/) drive the built binary with `assert_cmd`.
  They may only use `--dry-run` or inputs that fail validation.
- A test that creates files uses its own temporary directory and removes it.

## What a green run proves

`cargo test --all-targets` proves flag validation, registry consistency, action mapping,
symlink-safe removal on temporary trees and provider parsing against recorded command
output. It does not prove that any provider finds the right paths on a real Mac, that Finder
accepts a Trash request, or that a real `tmutil`, `docker` or `go` behaves as mocked. Check
those by hand with `--dry-run`, then with a single `--category` you can afford to lose.

## Manual checks

```bash
cargo run -- clean --dry-run --include-review --include-destructive
cargo run -- clean --category user-logs        # interactive, defaults to Trash
cargo run -- uninstall SomeApp --dry-run
```

Never pass `--hard` against your real home folder to test a change.
