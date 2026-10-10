> Global standard: ~/.claude v2026-10-04.

# tiny-cli

macOS-focused Rust CLI (binary `tiny`) for system info, read-only file scans,
cache cleanup, a focus timer and app uninstalls. Stack: Rust 2021 single binary
crate, `clap` 4.5, `anyhow`, `serde`, `sysinfo` 0.32, `dialoguer` 0.11; tests
use `assert_cmd` + `predicates`. Usage lives in [docs/user/](docs/user/); the
documentation index is [docs/README.md](docs/README.md).

## Branches and worktrees

- `main` holds this single-crate CLI. The Rust workspace and the SwiftUI desktop app live on
  `feat/clean-trust` (worktree `../tiny-cli-worktrees/clean-trust`), whose `AGENTS.md` and
  docs describe that layout. Until it lands, edit shared docs there; `main` takes only doc
  fixes for its own code and is merged into that branch, never cherry-picked.
- Branches live in worktrees at `../tiny-cli-worktrees/<slug>`.

## Common commands

| Command                                      | Purpose                                 |
| -------------------------------------------- | --------------------------------------- |
| `cargo build`                                | debug build of `tiny`                   |
| `cargo test --all-targets`                   | inline unit tests + `tests/` smoke      |
| `cargo clippy --all-targets -- -D warnings`  | lint gate                               |
| `cargo run -- --help`                        | CLI help                                |
| `cargo run -- clean --dry-run`               | safe manual check of the cleanup plan   |

There is no CI on `main`. What a green run does and does not prove is in
[development](docs/operations/development.md#what-a-green-run-proves).

## Directory structure

```
src/
├─ cli.rs                    # every clap definition; one *Opts struct per command
├─ main.rs                   # dispatch: Commands variant → commands::<x>::run(opts)
└─ commands/
   ├─ clean/                 # interactive cleanup: discover → picker → execute → report
   │  ├─ fs_safe.rs          # symlink-safe walk/size/remove — clean's only delete path
   │  ├─ types.rs            # ExecAction vs CleanAction, ExecReport
   │  └─ providers/          # one CleanProvider per file; registry in mod.rs
   ├─ scan/                  # read-only scan of Downloads/Desktop/Documents
   └─ sys.rs, focus.rs, uninstall.rs
tests/clean_smoke.rs         # assert_cmd end-to-end checks, disk-independent
docs/
├─ user/, internals/, operations/   # living docs by reader; index in docs/README.md
└─ specs/, plans/                    # requirements / execution records (D0/D4)
```

Locations that differ from `~/.claude/templates/project-structure.md`:

- Rust CLI is not a listed project type → follow the existing `snake_case`
  module layout above.
- Unit tests → inline `#[cfg(test)] mod tests` in the file under test;
  integration tests → `tests/`.

## Documentation

- Read the [design language](docs/DESIGN-LANGUAGE.md) before changing UI; keep its approved
  primary colour.
- `docs/user/` has one page per command family, in the voice of the shipped CLI;
  `README.md` keeps only the command table and links.
- `docs/internals/` holds decisions, cross-module constraints and traps; every page under
  `internals/` and `operations/` opens with the "For maintainers" callout. The
  [glossary](docs/internals/glossary.md) owns the vocabulary: use its terms in code and docs.
- No file catalogs, control-flow narration or appended PR summaries. There is no release
  page until tiny has a release process.

## Repo-specific rules (R-rules — only the delta from the global standard)

- **R1.** A new cleanup category implements `CleanProvider` and is registered in
  all three of `all_providers`, `known_category_ids` and `category_family` in
  [providers/mod.rs](src/commands/clean/providers/mod.rs). `category_family`
  panics on an unknown id. See
  [the provider registry](docs/internals/clean.md#the-provider-registry).
- **R2.** In `clean`, every metadata read uses `symlink_metadata` and every delete
  goes through [fs_safe.rs](src/commands/clean/fs_safe.rs)
  (`remove_recursive_safe`) or [`move_to_trash`](src/commands/clean/providers/mod.rs); never call `fs::remove_*` or
  `is_dir()` on discovered paths.
- **R3.** Keep `ExecAction::{Trash, HardDelete, EmptyTrash}` distinct from the
  menu-level `CleanAction` in [types.rs](src/commands/clean/types.rs).
  [`map_action`](src/commands/clean/execute.rs) maps menu Hard delete to
  `EmptyTrash` for the `trash` provider, which rejects every other action. See
  [mapping actions](docs/internals/clean.md#mapping-actions).
- **R4.** Honour `requires_app_quit()`: discovery skips a category whose owning
  app is running. Do not bypass it to "clean more".
- **R5.** `scan` stays strictly read-only. In `clean`, personal files appear only in the
  Review-risk categories `downloads-old`, `screenshots-old` and `mail-attachments`.
- **R6.** Tests and manual runs never delete real user data: use `--dry-run`, or
  temp directories created by the test. `--hard -y` with `TINY_CONFIRM_HARD=1`
  is a real permanent delete.

## Known traps

Read [docs/internals/traps.md](docs/internals/traps.md) before touching deletion code. For
desktop work, switch to the `feat/clean-trust` worktree first.

## Language

- Docs/comments: English (README, CLI help, code comments, specs and plans).
- Commit messages: conventional commits with a scope, description in Vietnamese
  (e.g. `docs(desktop): ...`). PR titles and bodies in Vietnamese.
