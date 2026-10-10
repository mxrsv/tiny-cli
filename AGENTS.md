> Global standard: ~/.claude v2026-10-04.

# tiny-cli

macOS-focused Rust CLI (binary `tiny`) for system info, read-only file scans,
cache cleanup, a focus timer and app uninstalls. Stack: Rust 2021 single binary
crate, `clap` 4.5, `anyhow`, `serde`, `sysinfo` 0.32, `dialoguer` 0.11; tests
use `assert_cmd` + `predicates`. Usage lives in [README.md](README.md).

## Common commands

| Command                                      | Purpose                                 |
| -------------------------------------------- | --------------------------------------- |
| `cargo build`                                | debug build of `tiny`                   |
| `cargo test --all-targets`                   | inline unit tests + `tests/` smoke      |
| `cargo clippy --all-targets -- -D warnings`  | lint gate                               |
| `cargo run -- --help`                        | CLI help                                |
| `cargo run -- clean --dry-run`               | safe manual check of the cleanup plan   |

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
_bmad-output/                # historical BMad artifacts (PRD, epics); read-only
```

Locations that differ from `~/.claude/templates/project-structure.md`:

- Rust CLI is not a listed project type → follow the existing `snake_case`
  module layout above.
- Unit tests → inline `#[cfg(test)] mod tests` in the file under test;
  integration tests → `tests/`.
- Desktop PRD, architecture and epics stay in `_bmad-output/planning-artifacts/` as
  read-only history; the BMad workflow was removed on 2026-10-10. New requirements go
  in `docs/specs/`.

## Documentation

Most code changes need no documentation change; agents and maintainers can read the code.
The index is [docs/README.md](docs/README.md).

- Before changing UI, read the [design language](docs/DESIGN-LANGUAGE.md). Preserve its
  approved primary colour; layout, information arrangement and icon treatments remain
  flexible as described there.

- `docs/internals/` holds architectural decisions and their reasons, constraints that span
  modules, and traps hard to discover from the source. Before adding a paragraph, ask what a
  maintainer would get wrong without it.
- `docs/user/` helps users accomplish tasks, one page per command family, in the voice of the
  shipped CLI, with no implementation detail. Update it when a flag or behaviour changes; the
  root README keeps only the command table and links.
- `docs/operations/` is the maintainer runbook. There is no release page until tiny has a
  release process. Every page under `internals/` and `operations/` opens with the
  "For maintainers" callout.
- Do not write file catalogs, field or method enumerations, control-flow narration, or
  appended PR summaries. When a documented decision changes, rewrite or remove the text.
- Specs in `docs/specs/` own requirements and acceptance criteria. Plans in `docs/plans/`
  link to specs and own technical tasks, progress, verification and handoff (D0/D4).
  Retain completed records as history. Small work needs neither document. Reviews stay
  in chat or an explicitly requested PR; raw research artifacts stay in scratchpad.

## Repo-specific rules (R-rules — only the delta from the global standard)

- **R1.** A new cleanup category implements `CleanProvider` and is registered in
  all three of `all_providers`, `known_category_ids` and `category_family` in
  [providers/mod.rs](src/commands/clean/providers/mod.rs). `category_family`
  panics on an unknown id.
- **R2.** In `clean`, every metadata read uses `symlink_metadata` and every delete
  goes through [fs_safe.rs](src/commands/clean/fs_safe.rs)
  (`remove_recursive_safe`) or [`move_to_trash`](src/commands/clean/providers/mod.rs); never call `fs::remove_*` or
  `is_dir()` on discovered paths.
- **R3.** Keep `ExecAction::{Trash, HardDelete, EmptyTrash}` distinct from the
  menu-level `CleanAction` in [types.rs](src/commands/clean/types.rs).
  [`map_action`](src/commands/clean/execute.rs) maps menu Hard delete to
  `EmptyTrash` for the `trash` provider, which rejects every other action.
- **R4.** Honour `requires_app_quit()`: discovery skips a category whose owning
  app is running. Do not bypass it to "clean more".
- **R5.** `scan` stays strictly read-only, and personal files never appear in
  `clean`.
- **R6.** Tests and manual runs never delete real user data: use `--dry-run`, or
  temp directories created by the test. `--hard -y` with `TINY_CONFIRM_HARD=1`
  is a real permanent delete.

## Known traps

Read [docs/internals/traps.md](docs/internals/traps.md) before touching deletion code or
resuming desktop work. The desktop app is not on `main`: inspect branch
`feat/native-desktop` (`git worktree list`) first.

## Language

- Docs/comments: English (README, CLI help, code comments, specs and plans).
- Commit messages: conventional commits with a scope, description in Vietnamese
  (e.g. `docs(desktop): ...`). PR titles and bodies in Vietnamese.
