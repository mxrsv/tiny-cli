# Uninstalling apps

`tiny uninstall` removes an app from `/Applications` together with the data it left in your
`~/Library`. It shows what it found and asks before removing anything. It needs macOS.

```bash
tiny uninstall                       # pick apps from a list
tiny uninstall AltTab                # one app by name
tiny uninstall AltTab --dry-run      # report only
tiny uninstall AltTab --shallow      # only the .app
tiny uninstall AltTab --leftovers-only   # only the ~/Library data, keep the app
tiny uninstall AltTab -y             # no menu, Move to Trash now
```

## Picking apps

Without a name you get every app in `/Applications`, with its size and when you last opened
it. `--sort` orders the list: `last-used` (default, least recently used first), `size` or
`name`.

## What is removed

The `.app` bundle, plus whatever exists of these for the app's bundle identifier:
Application Support, Caches, Preferences, Containers, Saved Application State, HTTP storage
and cookies, WebKit data, LaunchAgents and Group Containers — all under `~/Library`.

If tiny cannot read the app's bundle identifier it removes only the `.app`, because guessing
by name could hit folders that belong to macOS.

## Choosing an action

After the report:

- **Move to Trash (recoverable)** — the default.
- **Dry-run (no changes)**.
- **Hard delete (NOT recoverable)** — asks you to confirm once more.
- **Cancel**.

`-y` skips the menu and moves to the Trash. **`-y --hard` deletes permanently with no
confirmation at all** — unlike `tiny clean`, no environment variable is required.

## What is refused

- Apple's own apps, recognised by a `com.apple.` bundle identifier, or by name when the
  identifier cannot be read.
- Apps installed as Homebrew casks under `/opt/homebrew/Caskroom` whose cask name is the app
  name in lowercase with hyphens: use `brew uninstall --cask <name>`, or pass `--force` to
  remove them with tiny anyway. Other casks are not recognised.

If any selected app is refused, tiny stops after the report and removes nothing.

If one app fails partway, the others still run and tiny lists the failure at the end; the
failed app may be partly removed.
