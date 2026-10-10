# Known traps

> For maintainers. Using tiny? See [docs/user/](../user/).

Things that are not obvious from reading one file.

## Deletion

- **`uninstall --hard -y` asks nothing.** `clean` and `processes quit --force` demand an
  environment variable with `-y`; `uninstall` deletes permanently on the flags alone
  ([`uninstall.rs`](../../crates/tiny/src/render/uninstall.rs)).
- **An `uninstall` plan stops at its first failing path**, leaving that app partly removed;
  later apps still run ([`execute_plan`](../../crates/tiny-core/src/uninstall.rs)).
- **Move to Trash is not always recoverable.** With "Empty Trash automatically" on, Finder
  deletes it later; on a volume without a Trash it may delete at once, so the app skips paths
  off the home volume ([`checked_execute.rs`](../../crates/tiny-core/src/clean/checked_execute.rs)).
- **Some categories ignore Trash.** `docker`, `docker-volumes`, `time-machine-local` and
  `simulator-devices` run their tool for both CLI actions. The CLI's warning lists those ids by
  hand in [`render/clean/mod.rs`](../../crates/tiny/src/render/clean/mod.rs); add new
  tool-backed categories there.
- **`category_family` panics** on an unknown id. Register a new category in all four registry
  functions ([clean.md](clean.md#the-provider-registry)).

## Running locally

- **Real runs touch real data.** Only `--dry-run` and validation errors are side-effect free, so
  [`clean_smoke.rs`](../../crates/tiny/tests/clean_smoke.rs) is limited to those.
- **The dry-run smoke test is slow on a populated home folder**: it walks every category on the
  real disk (101–164 s on a developer Mac in the
  [MVP plan](../plans/2026-10-06-processes-clean-mvp.md), about 16 s on the CI macOS runner).
- **The first Trash action prompts for Automation access** to Finder. After a denial the CLI
  fails every path and carries on, while the app recognises error -1743 and stops with
  `AutomationDenied` ([`finder_trash.rs`](../../crates/tiny-core/src/clean/finder_trash.rs)).
  Grant access in System Settings → Privacy & Security → Automation.
- **The port smoke test needs `/usr/sbin/lsof`.** It skips without `python3` but fails without
  `lsof` at that path ([`processes_smoke.rs`](../../crates/tiny/tests/processes_smoke.rs)), so
  the Linux job in [CI](../../.github/workflows/ci.yml) fails.

## Native build

- **A fresh worktree cannot `swift build`**: bindings are [gitignored](../../.gitignore) output
  of [`build-tiny-ffi.sh`](../../scripts/build-tiny-ffi.sh); rerun it after any `tiny-ffi` change.
- **The app links `target/<host>/release/libtiny_ffi.a`.** The script passes `--target-dir` and
  `--target` so `CARGO_TARGET_DIR` or `CARGO_BUILD_TARGET` cannot redirect it; a plain
  `cargo build --release` output is never linked.
- **SwiftPM links Rust through a system-library target**, not an XCFramework, because Command
  Line Tools have no XCFramework packager ([MVP plan](../plans/2026-10-06-processes-clean-mvp.md)).
  The linker lists in [`Package.swift`](../../macos/Package.swift) and
  [`test-native.sh`](../../scripts/test-native.sh) come from
  `cargo rustc -p tiny-ffi --release -- --print native-static-libs`; update both together.
- **Building needs `rustc` 1.91 or newer**, though no toolchain file or `rust-version` says so:
  the locked `cargo-platform` 0.3.3, pulled in by UniFFI, requires it, and every gate runs
  `--locked` ([`Cargo.lock`](../../Cargo.lock)).
- **`swift test` runs nothing**: no test target, and Command Line Tools lack XCTest;
  `test-native.sh` compiles a hard-coded file list, so add new Swift files there.
- **Finder Automation needs no entitlement only while hardened runtime is off** in the ad-hoc
  signed `com.mxrsv.tiny.dev` bundle ([`build-native-app.sh`](../../scripts/build-native-app.sh));
  with it on, add `com.apple.security.automation.apple-events`.

## Repository

- **`main` is a different codebase**: the single-crate CLI under `src/`, merged into this branch
  ([AGENTS.md](../../AGENTS.md#branches-and-worktrees)). A code fix made there lands in `src/`
  and must be ported to `crates/` by hand.
- **The CLI's clean and scan modules are mostly shims**: files such as
  [`render/clean/types.rs`](../../crates/tiny/src/render/clean/types.rs) only re-export
  `tiny-core`, where the logic belongs.
