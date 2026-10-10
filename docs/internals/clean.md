# Clean pipeline

> For maintainers. Using tiny? See [docs/user/clean.md](../user/clean.md).

`tiny clean` runs in a fixed order from [`clean/mod.rs`](../../src/commands/clean/mod.rs):
validate flags, select providers, discover, print the summary, pick categories, print the
plan, optionally drill into paths, choose an action, execute, report. Terms are defined in
the [glossary](glossary.md).

## Validation first

[`cli_validate.rs`](../../src/commands/clean/cli_validate.rs) refuses bad combinations
before any filesystem access: unknown `--category` ids, `-y` without `--category`,
`--include-destructive --hard -y` without `--category`, `--idle-days 0`, and `--hard -y`
without `TINY_CONFIRM_HARD=1`. The environment check is injected as a boolean
(`validate_with_env`) so tests never set process environment.

The original flag matrix and its reasoning are in the
[`tiny clean` plan](../plans/2026-04-30-tiny-clean.md).

## The provider registry

A category exists only when it appears in all three of `all_providers`,
`known_category_ids` and `category_family` in
[`providers/mod.rs`](../../src/commands/clean/providers/mod.rs):

- `all_providers` builds the provider objects and passes tunables such as `idle_days`.
- `known_category_ids` is what `--category` validation accepts.
- `category_family` is the single source of family assignment and panics on an unknown id.
  The `every_known_category_has_family` test catches a missing arm.

No single test compares `all_providers` against `known_category_ids`. Most provider files
instead assert that their own `ID` is listed; give a new provider the same test.

Providers reuse the helpers in the same file: `top_level_entries` and `root_as_item` for
discovery, `execute_per_item` for the common Trash and hard-delete path, `dev_search_roots`
and `is_idle` for project caches.

## Selection and discovery

[`discover.rs`](../../src/commands/clean/discover.rs) picks providers by risk unless
`--category` is given, in which case naming a category is treated as consent and the
`--include-*` flags are ignored. For each selected provider it skips those whose
`available()` is false (the tool is not installed) and those whose gating app is running,
then keeps only providers that found items.

## Mapping actions

[`execute.rs`](../../src/commands/clean/execute.rs) turns the menu choice into one exec action
per provider through `map_action`:

| Menu choice | `trash` provider            | every other provider |
| ----------- | --------------------------- | -------------------- |
| Trash       | refused before execution    | `ExecAction::Trash`  |
| Hard delete | `ExecAction::EmptyTrash`    | `ExecAction::HardDelete` |

The trash provider rejects `Trash` and `HardDelete`; `execute_per_item` rejects
`EmptyTrash`. Both sides refuse so a wrong mapping fails loudly instead of deleting the
wrong thing. `time-machine-local` has no Trash semantics: both actions run
`tmutil deletelocalsnapshots`.

Excluded paths are removed from each group just before its provider executes. A failure on
one path is recorded in `ExecReport.failed` and the run continues. An error returned by a
provider as a whole (such as a refused action) aborts the remaining groups,
after earlier groups have already been acted on.

## Test seams

Tests never depend on what is installed or running on the machine:

- [`process.rs`](../../src/commands/clean/process.rs) — `ProcessChecker`, with `MockChecker`
  for app gating.
- [`runner.rs`](../../src/commands/clean/runner.rs) — `CommandRunner`, with `MockRunner` for
  providers that shell out to `docker`, `go`, `tmutil`, `mdfind` and similar.
- `discover_with_checker` and `execute_with_providers` take those seams as parameters.

The end-to-end tests in [`tests/clean_smoke.rs`](../../tests/clean_smoke.rs) run only
`--dry-run` and validation failures, so they pass on any machine without touching data.
