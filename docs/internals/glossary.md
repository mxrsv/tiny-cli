# Glossary

> For maintainers. Using tiny? See [docs/user/](../user/).

The words tiny's code, tests and documents use, each with the terms to avoid. Most belong to
`clean`; types are in [`types.rs`](../../src/commands/clean/types.rs) and
[`providers/mod.rs`](../../src/commands/clean/providers/mod.rs).

## Clean

**Category** — One kind of removable data with a stable kebab-case id (`node-modules`,
`trash`) accepted by `--category`. The user-facing unit of choice. Not: provider, type.

**Provider** — The `CleanProvider` implementation behind one category: it declares the id,
label and risk, discovers items and executes actions on them. One file under `providers/`
may define several (`xcode.rs` has three). Not: category, plugin.

**Family** — The picker's top-level grouping: Dev caches, User storage, System leftovers.
Assigned only by `category_family()`; a provider never declares its own. Not: group.

**Risk level** — `Safe`, `Review` or `Destructive`. Decides default visibility in the picker
and whether a category is pre-ticked (only `Safe`). Not: severity.

**Item** (`CleanItem`) — One path a provider proposes to remove, with its size and risk.
Not: entry, file (an item is often a directory).

**Group** (`CategoryGroup`) — All items one provider discovered, with their total size. What
the picker lists and the plan prints. Not: family.

**Plan** — The groups the user selected, printed path by path before the action menu.
Nothing is removed before the plan is shown.

**Clean action** (`CleanAction`) — What the user picked in the menu: Trash, HardDelete,
DryRun, Cancel. Not: exec action.

**Exec action** (`ExecAction`) — What a provider is asked to do: Trash, HardDelete or
EmptyTrash. Derived from the clean action per provider by `map_action`. Not: clean action.

**Excluded paths** — Paths the user unticked in the review-paths drill-down. Filtered out of
each group just before execution.

**Idle** — A project whose manifest has not been modified for `--idle-days`; only idle
projects' caches are listed. An unreadable modification time counts as not idle.

**Dev search roots** — `~/Documents`, `~/Projects`, `~/Code`, `~/Developer`, `~/Workspace`:
the folders the project-cache providers walk.

**App gating** — A provider's `requires_app_quit()` names a process; while it runs, the
category is skipped at discovery and reported as skipped.

## Uninstall

**Leftover** — A `~/Library` path keyed by the app's bundle identifier. Not: cache (a
leftover may be preferences or containers).

**Blocked** — A plan that must not run: a system app, or a Homebrew cask without `--force`.
One blocked plan stops the whole invocation.
