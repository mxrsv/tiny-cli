> Global standard: ~/.claude v2026-10-04.

# tiny-cli

macOS-focused Rust workspace: the shared engine [`tiny-core`](crates/tiny-core/src/lib.rs),
the `tiny` [CLI](crates/tiny/src/main.rs), and a native [SwiftUI app](macos/Sources/Tiny/TinyApp.swift)
that links the engine in-process through [UniFFI](crates/tiny-ffi/src/lib.rs). Usage lives in
[docs/user/](docs/user/); the documentation index is [docs/README.md](docs/README.md).

## Branches and worktrees

- `feat/clean-trust` holds the workspace and the desktop app; `main` still holds the older
  single-crate CLI. Until this branch lands, edit docs here; `main` takes only doc fixes for
  its own code and is merged into this branch, never cherry-picked.
- Branches live in worktrees at `../tiny-cli-worktrees/<slug>`. A fresh worktree runs
  `scripts/build-tiny-ffi.sh` before `swift build`: the Swift bindings are generated and
  gitignored.

## Common commands

| Command                                                         | Purpose                                     |
| --------------------------------------------------------------- | ------------------------------------------- |
| `cargo fmt --all --check`                                       | format gate                                 |
| `cargo test --workspace --all-targets --locked`                 | unit tests + `crates/tiny/tests/` smoke     |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | lint gate                                   |
| [`scripts/test-native.sh`](scripts/test-native.sh)              | bridge + Foundation-only Swift checks       |
| [`scripts/build-native-app.sh`](scripts/build-native-app.sh)    | ad-hoc signed `macos/.build/Tiny Dev.app`   |
| `cargo run -p tiny -- clean --dry-run`                          | safe manual check of the cleanup plan       |

CI runs the three `cargo` gates; `test-native.sh` runs only locally. What a green run does
and does not prove is in [development](docs/operations/development.md#what-a-green-run-proves).

## Directory structure

```
crates/
├─ tiny-core/src/            # engine: no CLI, no UI, typed errors (thiserror)
│  ├─ clean/                 # providers/, fs_safe.rs, finder_trash.rs, checked_execute.rs
│  ├─ processes/             # sampling, ports, terminate
│  └─ scan/, uninstall.rs, sys.rs, focus.rs
├─ tiny/src/                 # cli.rs (clap) + render/<command>; tests/ smoke
└─ tiny-ffi/src/             # UniFFI records and calls for Swift
macos/Sources/               # Tiny (SwiftUI views and state), TinyEngine (engine actor)
scripts/                     # bridge, native checks, app bundle
docs/                        # user/, internals/, operations/, specs/, plans/
```

Locations that differ from `~/.claude/templates/project-structure.md`:

- Rust follows the `snake_case` module layout above; Swift files are `PascalCase`, and
  SwiftPM lives in [macos/Package.swift](macos/Package.swift).
- Rust unit tests → inline `#[cfg(test)] mod tests`; CLI end-to-end tests →
  `crates/tiny/tests/`; Swift checks → [test-native.sh](scripts/test-native.sh), which lists
  every source it compiles, so a new Swift file must be added there.

## Documentation

- Read the [design language](docs/DESIGN-LANGUAGE.md) before changing UI; keep its approved
  primary colour.
- `docs/user/` has one page per command family plus the desktop app, in the voice of the
  shipped product; `README.md` keeps only the command table and links.
- `docs/internals/` holds decisions, cross-module constraints and traps; every page under
  `internals/` and `operations/` opens with the "For maintainers" callout. The
  [glossary](docs/internals/glossary.md) owns the vocabulary: use its terms in code, docs and
  UI copy.
- No file catalogs, control-flow narration or appended PR summaries. There is no release
  page until tiny has a release process.

## Repo-specific rules (R-rules — only the delta from the global standard)

- **R1.** A new cleanup category implements `CleanProvider` and is registered in
  `all_providers_with`, `known_category_ids`, `category_family` and `comes_back` in
  [providers/mod.rs](crates/tiny-core/src/clean/providers/mod.rs); `category_family` panics on
  an unknown id. The desktop may move its items only if `desktop_trash_paths()` returns
  true. See [the provider registry](docs/internals/clean.md#the-provider-registry).
- **R2.** In `clean`, every metadata read on a discovered path uses `symlink_metadata`, and
  every delete goes through [fs_safe.rs](crates/tiny-core/src/clean/fs_safe.rs)
  (`remove_recursive_safe`) or Finder: [`FinderTrash`](crates/tiny-core/src/clean/finder_trash.rs)
  in the app, [`move_to_trash`](crates/tiny-core/src/clean/providers/mod.rs) in the CLI. Never
  call `fs::remove_*` or `is_dir()` on discovered paths.
- **R3.** Keep `ExecAction::{Trash, HardDelete, EmptyTrash}` distinct from the menu-level
  `CleanAction` in [types.rs](crates/tiny-core/src/clean/types.rs).
  [`map_action`](crates/tiny-core/src/clean/execute.rs) maps menu Hard delete to `EmptyTrash`
  for the `trash` provider, which rejects every other action. See
  [mapping actions](docs/internals/clean.md#mapping-actions).
- **R4.** Honour `quit_apps()` (default: `requires_app_quit()`) and `item_apps()`: discovery
  skips a category whose owning app runs, and the desktop re-checks each path before acting.
  Do not bypass them to "clean more".
- **R5.** `scan` stays strictly read-only. In `clean`, personal files appear only in the
  Review categories `downloads-old`, `screenshots-old` and `mail-attachments`; the desktop
  flags files that look sensitive and never selects them with a category toggle
  ([decision](docs/specs/2026-10-09-clean-trust-evidence.md#2-user-decisions-2026-10-09)).
- **R6.** Tests and manual runs never delete real user data: use `--dry-run` or temp
  directories created by the test. Swift checks, snapshots and smoke flags never call the
  real `cleanExecute` (it moves real files through Finder); checks use `FakeCleanEngine`,
  and real `cleanDiscover`/`cleanPreview` are read-only. `--hard -y` with
  `TINY_CONFIRM_HARD=1` is a real permanent delete.

## Desktop conventions

- Swift owns presentation and macOS integration; every process and cleanup operation has
  one implementation in `tiny-core`
  ([boundary spec](docs/specs/2026-10-07-swiftui-cli-boundary.md)). Never edit generated
  bindings; regenerate with `scripts/build-tiny-ffi.sh` after any `tiny-ffi` API change, and
  never enable `test-hooks` in the app.
- The [engine actor](macos/Sources/TinyEngine/Engine.swift) serializes sampling off the main
  actor. Keep PID + start-time identity through list and detail. Whole-machine usage comes
  from the [system sampler](crates/tiny-core/src/processes/snapshot.rs), never summed
  app/process values.
- App-row Quit is the one Swift-owned action ([AppQuit](macos/Sources/Tiny/AppQuit.swift)),
  gated by the Rust `refusal`. Per-PID Quit/Force Quit and Move to Trash go through Rust and
  its session gate (`Busy`).
- Every Swift-side mutation claims the single [ActionState](macos/Sources/Tiny/ProcessActions.swift)
  guard and writes an [in-flight marker](macos/Sources/Tiny/InFlightMarker.swift) before
  acting; nothing is replayed at launch.
- Review-risk items are never moved unless explicitly selected (PC-C3 in the
  [MVP spec](docs/specs/2026-10-06-processes-clean-mvp.md#clean-acceptance-criteria)); the
  [FFI preview](crates/tiny-ffi/src/clean.rs) and Swift both enforce it.
- [build-native-app.sh](scripts/build-native-app.sh) fails if smoke or debug hooks reach the
  default bundle; `--smoke-hooks` builds them. Hardened runtime is off, so no apple-events
  entitlement exists.
- [App groups](macos/Sources/Tiny/AppGroups.swift) use bundle paths and conservative
  same-owner ancestry; nested helper metadata must not replace an outer app's identity.

## Known traps

Read [docs/internals/traps.md](docs/internals/traps.md) before touching deletion code, the
FFI bridge or the native build.

## Language

- Docs/comments: English (README, CLI help, code comments, UI copy, specs and plans).
- Commit messages: conventional commits with a scope, description in Vietnamese
  (e.g. `docs(desktop): ...`). PR titles and bodies in Vietnamese.
