# Desktop app

Tiny Dev is the native Mac app. Its dock switches between two screens: **Activity** (Apps &
activity) and **Clean**. It needs macOS 14 or later; build and open it as described in the
[README](../../README.md#desktop-app). Everything stays on your Mac, and every action asks
first.

## Apps & activity

- **CPU · System** and **RAM · System** show the whole machine: CPU across all cores
  (0–100%) and memory used out of total. Values that are still warming up, unavailable or
  stale say so.
- The list groups processes by app, with the app's icon, member count, CPU and memory. App
  CPU is per core and can exceed 100%; app memory adds up its processes' resident memory, so
  shared pages may count twice. Processes without an app sit under **System & Background**,
  collapsed until you open it or search.
- Search matches an app, process name, PID or owner. Sort by CPU, Memory or Name.
- Select an app, then one of its processes in the inspector, to see its parent, children and
  listening TCP ports.
- The **Ports** tile lists listening TCP ports you can see and who owns them. Without root,
  other users' sockets are invisible, so an empty list never means a port is free.
- The list refreshes every two seconds. **Pause** stops that; **Refresh** (⌘R) reads it now.

An app offers **Quit**, which asks it to quit as if you chose Quit from its menu; it may ask
you to save, and Tiny never force quits it. It also offers Reveal in Finder and Copy Path. A
single process offers **Quit** (SIGTERM), **Force Quit…** (SIGKILL), Reveal Executable in
Finder and Copy PID. Every Quit and Force Quit asks first, with Cancel as the default.
Tiny itself, its parent, PID 0 and 1, other users' processes and protected system processes
such as Finder and the Dock cannot be quit; the menu says why.

## Clean

**Scan** checks the cleanup categories of [`tiny clean`](clean.md) without changing anything.
Permanent ones, such as emptying the Trash or deleting Time Machine snapshots, stay in the
CLI. Each category is a tile with its size, why it is included and its risk; tiles are
grouped by what happens after a move:

- **Rebuilt automatically** — caches and build output that tools or apps create again.
- **Your files** — only the Trash can bring these back; check each path before moving it.
- **Report only** — shown but never moved: Docker cleanup is a tool command, and app
  leftovers are matched by folder name, which is unreliable.

Only Safe items start ticked. Review-risk items are never ticked for you, and downloads that
look private, such as keys or recovery codes, are flagged and never ticked by a category
toggle: tick them one by one. While an app that owns a category or a cache is running, Tiny
leaves it out; quit the app and scan again.

**Review…** lists every selected path. A folder that holds a review item you did not select
is not moved. A review expires after 15 minutes; after that, scan again.

**Move to Trash** asks once more, then moves each item through Finder, so Put Back works.
Tiny re-checks every item just before moving it and skips anything that changed. The report
lists what was moved, failed, skipped or not attempted. Moved items still use disk space
until you empty the Trash.

The first move asks whether Tiny Dev may control Finder. If you deny it, nothing is deleted
and the report explains how to allow it: System Settings → Privacy & Security → Automation,
turn on Finder under Tiny Dev, then scan again.

> Move to Trash is still deletion if Finder empties the Trash automatically. Turn that off
> before cleaning if you want to keep the option to put items back.

## If Tiny stops mid-action

If Tiny quits or crashes while a Quit, Force Quit or Move to Trash is running, the next
launch shows a notice that the result is unknown. Check the target or the Trash yourself;
Tiny never repeats an action on its own.
