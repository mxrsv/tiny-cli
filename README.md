# tiny-cli

A local macOS desktop app and Rust CLI for everyday performance and productivity.
The [native SwiftUI development app](macos/Sources/Tiny/TinyApp.swift) inspects and
quits local processes and moves cleanup candidates to the Trash through the shared
Rust engine. All filesystem operations stay on your machine.

## Goals

- Keep the engine independent of the GUI and async runtime.
- Separate CLI parsing/presentation (`crates/tiny`) from computation (`crates/tiny-core`).
- Provide useful, real-world commands that are safe by default.

## Build & Run

### Native SwiftUI development app

Requirements: Apple Silicon Mac, macOS 14+, Swift 6, Rust stable and Xcode
Command Line Tools. No Node.js, dev server or separate CLI installation is needed
for the [statically linked native app](macos/Package.swift).

```bash
scripts/build-native-app.sh
open "macos/.build/Tiny Dev.app"
```

The [builder](scripts/build-native-app.sh) creates and verifies an ad-hoc signed
`Tiny Dev.app` with bundle ID `com.mxrsv.tiny.dev`, separate from the existing
`Tiny.app`. It does not install the app or change permissions. Optional installation:
copy `macos/.build/Tiny Dev.app` into `/Applications`.

The [Apps & activity screen](macos/Sources/Tiny/ProcessesView.swift) shows two
whole-machine widgets: CPU utilization across all cores (0–100%) and system RAM
used/total with a percentage. These come from the retained
[system sampler](crates/tiny-core/src/processes/snapshot.rs), not summed process
CPU or memory. Warm-up, unavailable, paused and stale values are explicit.

The [app grouping model](macos/Sources/Tiny/AppGroups.swift) combines processes
by outer app bundle and bounded same-owner ancestry; independent child apps stay
separate. [AppKit metadata](macos/Sources/Tiny/AppCatalog.swift) supplies local
icons, with a native fallback and no network lookup. App rows show member count,
per-core CPU (which may exceed 100%) and summed resident memory. Partial totals
are labelled; resident sums may count shared pages more than once and are not
whole-machine RAM usage. Unresolved processes remain under **System & Background**,
collapsed by default and included when searching.

Search matches app names or member name/PID/owner; sort uses app CPU, resident
memory or name. Select an app, then a member in the
[inspector](macos/Sources/Tiny/ProcessDetailView.swift), to see its parent,
children and listening TCP ports. Automatic refresh runs every two seconds;
pause and manual refresh remain available.

Selecting an app offers Quit, a native request like ⌘Q that is never escalated, plus
Reveal in Finder and Copy Path. Selecting a member process offers Quit, Force Quit…,
Reveal executable and Copy PID; row context menus offer the same items. Every Quit
and Force Quit asks first in a [native alert](macos/Sources/Tiny/ConfirmationAlert.swift)
naming the target and consequence, with Cancel as the default; Force Quit is confirmed
separately. Rust revalidates PID, start time, name and executable before signalling
and refuses Tiny itself, its parent, PID 0/1, other users' and protected system
processes. The [Ports tile](macos/Sources/Tiny/PortsTile.swift) lists the current
user's visible TCP listeners and their owners; without root, other users' sockets
are invisible, so an empty list never means a port is free.

The **Clean** destination in the dock scans cleanup categories read-only and shows
them as tiles: green is safe, orange needs review and is never selected for you, and
report-only categories (Docker prune, Time Machine snapshots, Trash) stay in the
CLI; orphaned Application Support folders are only reported, in the CLI too. **Review…** lists every path, asks Rust
for a preview that expires after 15 minutes, and refuses a folder that would carry an
unselected review item. **Move to Trash** asks for confirmation, then moves each item
through Finder so Put Back works, revalidating it immediately before the move, and
reports moved, failed, skipped and not-attempted items. Moved bytes are not freed
space until the Trash is emptied. The first move asks to let Tiny Dev control Finder;
if that is denied, nothing is deleted and the report explains how to allow it.
If Tiny stops mid-action, the next launch shows an unknown-outcome notice;
nothing is ever retried automatically.

The [engine](macos/Sources/TinyEngine/Engine.swift) preserves unavailable values
and spaces samples for CPU warm-up. Other-user/system processes may not expose
memory, CPU or ports. The [state model](macos/Sources/Tiny/AppState.swift) retains
last-good data with a stale indicator on failure and preserves app selection
across member churn. Member details use PID plus start time, reject obsolete
responses, and revalidate identity after the port probe. Start time has one-second
precision in the [shared sampler](crates/tiny-core/src/processes/snapshot.rs).

Native validation and optional in-process rendering of real data:

```bash
scripts/test-native.sh
"macos/.build/Tiny Dev.app/Contents/MacOS/Tiny" --smoke-test
"macos/.build/Tiny Dev.app/Contents/MacOS/Tiny" --snapshot /tmp/tiny-processes.png
```

Snapshot scenes are selected with an optional argument after the path: `member`,
`app`, `notice`, `minimum`, `clean`, `clean-all`, `review`, `review-fixture` or
`report`. They stage a selection or sample data and never confirm or execute an
action. `scripts/build-native-app.sh --smoke-hooks` additionally compiles
`--quit-test-app /tmp/Name.app`, which launches that disposable app and quits only the
instance it launched; the default build excludes it.

[Native checks](scripts/test-native.sh) compile the actual state/engine sources
with stdlib-only checks; CLT-only installations lack XCTest and Swift Testing,
so `swift test` is not the native test entrypoint. The
[smoke test](macos/Sources/Tiny/TinyApp.swift) creates and terminates only its own
temporary `sleep` child and verifies its appearance/disappearance. Snapshot mode
renders the app's own view without Screen Recording permission. Automated checks
do not replace interactive visual acceptance.

### CLI

```bash
cargo build -p tiny
cargo run -p tiny -- --help
```

CLI source/presentation is in `crates/tiny/src/`; computation and providers live
in `crates/tiny-core/src/`. The SwiftUI app lives in `macos/`.

### Validation

```bash
cargo fmt --all --check
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
scripts/test-native.sh
```

## Commands

### `sys` — system information

```bash
cargo run -p tiny -- sys
```

Reports OS, host, uptime, CPU count and model, memory usage, and per-disk
usage.

### `scan` — scan common folders, report only

```bash
cargo run -p tiny -- scan
cargo run -p tiny -- scan --min-size-mb 100 --older-than-days 30
```

Scans `~/Downloads`, `~/Desktop`, and `~/Documents`. Reports the largest files
and the oldest files that exceed the thresholds. **`scan` is strictly
read-only** — it never deletes anything. For cleanup, see `clean` below.

Options:

- `--min-size-mb` (default `100`): report files at or above this size.
- `--older-than-days` (default `90`): report files older than this many days.

### `processes` — list, inspect and quit your processes

```bash
tiny processes                          # top 20 by CPU
tiny processes --sort memory --limit 50
tiny processes --port 3000              # visible owners of a listening TCP port
tiny processes show 4321 --json         # parent, children and listening ports
tiny processes quit 4321                # SIGTERM after a confirmation prompt
tiny processes quit 4321 --force        # SIGKILL, confirmed separately
TINY_CONFIRM_FORCE=1 tiny processes quit 4321 --force -y   # non-interactive SIGKILL
```

`--json` emits camelCase fields; CPU that has not been measured yet is `null`,
never `0`. `quit` revalidates the PID's start time, name and owner immediately
before signalling, sends exactly one signal, never escalates to SIGKILL and never
signals a process tree. It refuses `tiny` itself, its parent, PID 0/1, other users'
processes and protected system processes (`WindowServer`, `loginwindow`, `Dock`,
`SystemUIServer`, `Finder`), and exits non-zero unless the process actually exited.
`--port` lists only owners visible to the current user: without root, other users'
sockets are invisible, so no owner does not mean the port is free.

### `clean` — interactive cleanup of caches and recoverable data (macOS)

```bash
tiny clean                              # picker → plan → action menu
tiny clean --dry-run                    # show plan and exit, no prompt
tiny clean --include-review             # also list review-risk caches
tiny clean --include-destructive        # also list Trash, tmutil snapshots, Docker volumes
tiny clean --review-paths               # drill-down picker per path before action
tiny clean --idle-days 60               # raise idle threshold for project caches
tiny clean --category node-modules      # interactive, scoped to one category
tiny clean --category trash --hard      # the only sanctioned way to empty Trash
TINY_CONFIRM_HARD=1 tiny clean --category cargo --hard -y   # non-interactive permanent delete
```

Categories are grouped into three families in the picker (32 total):

**Dev caches (18)** — `cargo`, `npm`, `pnpm`, `yarn`, `node-modules`,
`python-caches`, `rust-targets`, `gradle-maven`, `jetbrains`, `vscode`,
`ios-simulators`, `android-sdk`, `go-cache`, `docker`, `docker-volumes`,
`xcode-derived`, `xcode-archives`, `xcode-devicesupport`.

**User storage (6)** — `downloads-old`, `screenshots-old`,
`mail-attachments`, `streaming-caches`, `chat-caches`, `browser-caches`.

**System leftovers (8)** — `user-logs`, `user-caches`, `trash`,
`quarantine`, `crash-reports`, `app-orphans`, `time-machine-local`,
`font-quicklook-caches`.

The default picker shows only **safe** categories. Add `--include-review`
for developer caches and other items that may want manual review. Add
`--include-destructive` to surface `trash`, `time-machine-local` and
`docker-volumes` (snapshots and volumes are not recoverable). `docker`
prunes only the selected types (`docker image prune -af`, `docker builder
prune -af`); `docker-volumes` runs `docker volume prune -af`. Use
`--category <id>` to target a single category — repeating the flag is fine
(`--category user-logs --category xcode-derived`).

**Flags that affect discovery:**

- `--idle-days N` (default `30`): only flag project caches whose project
  manifest hasn't been touched in the last `N` days. Applies to
  `node-modules`, `python-caches`, `rust-targets`, `android-sdk`,
  `downloads-old`, `screenshots-old`. `N` must be `> 0`.
- `--review-paths`: after the family picker, opens a per-path drill-down
  so you can deselect individual paths before the action runs. Also
  available mid-flow as the **Review paths** entry in the action menu.

**Safety model:**

- Default action is **Move to Trash** (recoverable).
- `--hard` is permanent; without `-y` it requires a Y/n confirmation, with
  `-y` it requires `TINY_CONFIRM_HARD=1` in the environment.
- `--yes` requires `--category` so the scope is always explicit.
- The Trash provider is wired so only `Hard delete` reaches it (mapped to
  Finder's empty-trash); `Move to Trash` for the trash category is rejected.
- Discovery skips a category whose owning app is currently running
  (e.g. `xcode-derived` when Xcode is open).
- All filesystem walks are symlink-safe — symlinked directories are never
  followed, so we cannot delete data outside the listed paths.

> Note: even Move to Trash is destructive if macOS "Empty Trash automatically"
> (System Settings → General → Storage) is on. Use `--dry-run` first or
> disable that setting.

`scan` vs `clean` at a glance: `scan` is read-only and surfaces personal
files in `~/Downloads` / `~/Desktop` / `~/Documents` so you can review and
decide. `clean` is interactive deletion of developer caches and recoverable
data. They are intentionally separate — personal files never appear in
`clean`.

### `focus` — local focus timer

```bash
cargo run -p tiny -- focus --minutes 25
cargo run -p tiny -- focus --minutes 50 --label "deep work"
```

Runs a synchronous timer with a simple progress bar. When the session ends,
the entry is appended to `~/.tiny-cli/focus-sessions.json` so you have a
record of completed sessions.

### `uninstall` — remove apps and their `~/Library` leftovers (macOS)

```bash
tiny uninstall                       # picker → report → action menu
tiny uninstall AltTab                # report → action menu
tiny uninstall AltTab --dry-run      # report only, no prompt
tiny uninstall AltTab -y             # skip menu, move to Trash immediately
tiny uninstall AltTab --hard -y      # skip menu, rm -rf (advanced)
tiny uninstall AltTab --shallow      # only /Applications/<Name>.app
tiny uninstall AltTab --leftovers-only  # only ~/Library cleanup
```

After the report, an action menu lets you choose:

- **Move to Trash (recoverable)** — default, safe
- **Dry-run (no changes)**
- **Hard delete (NOT recoverable)** — extra confirm prompt before touching anything
- **Cancel**

**Safety:**

- Default action highlights "Move to Trash" — you must press Enter, nothing auto-runs.
- Trash is recoverable; hard delete requires a separate `[y/N]` confirmation.
- Refuses system apps (`com.apple.*`) and Homebrew casks (use `brew uninstall --cask`; pass `--force` to override).

**Picker sorts:** `--sort=last-used` (default), `size`, `name`.

## Roadmap

- `files` — richer filesystem analytics (duplicates, by-extension breakdowns).
- `doctor` — check common dev environment issues (PATH, dotfiles, tools).
- `today` — daily summary combining focus log and calendar-style notes.
