# Known traps

> For maintainers. Using tiny? See [docs/user/](../user/).

Things that are not obvious from reading one file.

## Deletion

- **`uninstall` does not use `fs_safe`.** It has its own sizing and removal in
  [`uninstall.rs`](../../src/commands/uninstall.rs). Removal unlinks a symlinked leftover
  rather than following it, but sizing checks each leftover with `is_dir()`, so a leftover
  that is itself a symlink to a folder is reported with the target's size. Changes to
  [`fs_safe.rs`](../../src/commands/clean/fs_safe.rs) do not reach `uninstall`.
- **`uninstall --hard -y` asks nothing.** `clean` requires `TINY_CONFIRM_HARD=1` for the
  same combination; `uninstall` deletes permanently on the flags alone.
- **An `uninstall` plan stops at its first failing path**, leaving that app partly removed;
  later apps still run.
- **Move to Trash is not always recoverable.** With macOS "Empty Trash automatically" on,
  Finder deletes it later. Exercise destructive commands with `--dry-run`.
- **`time-machine-local` treats Trash as permanent.** Snapshots have no Trash, so both
  actions run `tmutil deletelocalsnapshots`.
- **`category_family` panics** on an id it does not know. Register a new category in all
  three registry functions — see [clean.md](clean.md#the-provider-registry).

## Running locally

- **Real runs touch real data.** Discovery reads your actual caches and app list; only
  `--dry-run` and the validation errors are side-effect free. The smoke tests in
  [`tests/clean_smoke.rs`](../../tests/clean_smoke.rs) are limited to those for this reason.
- **The dry-run smoke test is slow** (about 100 seconds) because it walks every category on
  the real machine with `--include-review --include-destructive`.
- **The first Trash action prompts for Automation access** to Finder. If it was denied
  earlier, `osascript` fails for every path until access is granted in System Settings →
  Privacy & Security → Automation.

## Repository

- **`Cargo.lock` is gitignored** while tiny is a single binary crate; do not commit it from
  `main`.
- **The desktop planning documents disagree.** `_bmad-output/` describes Tauri + React; the
  [2026-10-07 spec](../specs/2026-10-07-swiftui-cli-boundary.md) chose SwiftUI + UniFFI.
  Neither app exists on `main`; inspect branch `feat/native-desktop` before resuming. See
  [overview](overview.md#desktop-migration).
