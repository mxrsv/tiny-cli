# Cleaning caches

`tiny clean` finds caches and leftovers that are safe to remove, shows how much space each
category takes, lets you pick, and asks what to do before touching anything. It needs macOS.

```bash
tiny clean                              # picker → plan → action menu
tiny clean --dry-run                    # show the plan and exit
tiny clean --include-review             # also list categories worth a look first
tiny clean --include-destructive        # also list Trash and Time Machine snapshots
tiny clean --review-paths               # untick individual paths before acting
tiny clean --category node-modules      # only this category (repeatable)
tiny clean --category trash --hard      # the only way to empty the Trash
```

Run `--dry-run` first on a new machine. Your personal files never appear here; use
[`tiny scan`](scan.md) for those.

## Categories

32 categories in three groups. Each has a risk level that decides whether you see it by
default.

**Dev caches** — `cargo`, `npm`, `pnpm`, `yarn`, `node-modules`, `python-caches`,
`rust-targets`, `gradle-maven`, `jetbrains`, `vscode`, `ios-simulators`, `android-sdk`,
`go-cache`, `docker`, `docker-volumes`, `xcode-derived`, `xcode-archives`,
`xcode-devicesupport`.

**User storage** — `downloads-old`, `screenshots-old`, `mail-attachments`,
`streaming-caches`, `chat-caches`, `browser-caches`.

**System leftovers** — `user-logs`, `user-caches`, `trash`, `quarantine`, `crash-reports`,
`app-orphans`, `time-machine-local`, `font-quicklook-caches`.

| Risk          | Shown                         | Categories                                               |
| ------------- | ----------------------------- | -------------------------------------------------------- |
| `safe`        | always                        | `user-logs`, `crash-reports`, `font-quicklook-caches`, `xcode-derived` |
| `review`      | with `--include-review`       | everything else                                          |
| `destructive` | with `--include-destructive`  | `trash`, `time-machine-local`, `docker-volumes`          |

Naming a category with `--category` shows it whatever its risk; the `--include-*` flags are
then ignored.

## Choosing what to clean

1. **Pick categories.** Grouped by family, with `safe` ones already ticked; Space toggles,
   Enter confirms.
2. **Read the plan.** Every path that would be removed, with sizes.
3. **Choose an action:**
   - **Move to Trash (recoverable)** — the default.
   - **Dry-run (no changes)** — stop here.
   - **Review paths (drill-down)** — untick individual paths, then come back to this menu.
   - **Hard delete (NOT recoverable)** — asks you to confirm once more.
   - **Cancel**.

At the end tiny prints how many paths it removed and lists any it could not.

## Flags that change what is found

- `--idle-days N` (default 30) — project caches (`node-modules`, `python-caches`,
  `rust-targets`, `android-sdk`) are listed only when the project has not been touched for
  `N` days; `downloads-old` and `screenshots-old` use the same threshold. `N` must be above 0.
- Project caches are searched under `~/Documents`, `~/Projects`, `~/Code`, `~/Developer` and
  `~/Workspace`.
- A category is skipped while its app is open — for example Xcode categories while Xcode
  runs, `mail-attachments` while Mail runs. tiny tells you which app to quit.

## Safety

- **Move to Trash is the default.** It goes through Finder, so Put Back works.
- **`--hard` is permanent.** Interactively it asks Y/n. With `-y` it also needs
  `TINY_CONFIRM_HARD=1` in the environment:
  `TINY_CONFIRM_HARD=1 tiny clean --category cargo --hard -y`.
- **`-y` needs `--category`**, so an unattended run always names its scope.
- **The Trash can only be emptied**, with Hard delete. Moving the Trash to the Trash is
  refused.
- **Time Machine snapshots cannot go to the Trash.** Either action deletes them for good.
- **Docker has no Trash either.** `docker` prunes only unused images and build cache.
  Volumes hold container data such as databases, so they are the separate destructive
  category `docker-volumes`.
- **`app-orphans` is report only.** It lists Application Support folders that match no
  installed app, but a folder name does not prove the app is gone, so tiny never moves
  them. Check them in Finder.
- **Symlinks are never followed**, so nothing outside the listed paths can be removed.

> Move to Trash is still deletion if macOS empties the Trash automatically
> (System Settings → General → Storage). Turn that off or use `--dry-run` first.
