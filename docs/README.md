# tiny-cli documentation

Three tiers, split by reader. Every page describes the `tiny` CLI and the Tiny desktop app on
this branch in the present tense.

## `user/` — using tiny

- [Getting started](user/getting-started.md) — build and install, the command set, `sys` and
  `focus`.
- [Scanning files](user/scan.md) — the read-only report, custom roots, `.tinyignore`,
  duplicates and JSON output.
- [Cleaning caches](user/clean.md) — categories and risk, the picker and action menu, the
  flags, your own files, the safety model.
- [Uninstalling apps](user/uninstall.md) — what is removed, the action menu, what is refused.
- [Processes](user/processes.md) — the list, ports, and quitting a process.
- [Desktop app](user/desktop.md) — Apps & activity, Clean, and what happens if Tiny stops
  mid-action.

## `internals/` — how tiny is built

- [Overview](internals/overview.md) — the workspace and the desktop app, how a command is
  wired, read-only versus destructive.
- [Glossary](internals/glossary.md) — the words the code, docs and UI use.
- [Clean pipeline](internals/clean.md) — validation, providers, discovery, action mapping,
  the desktop's Trash path, test seams.
- [Known traps](internals/traps.md) — what is not obvious from reading one file.

## `operations/` — building and checking

- [Development](operations/development.md) — toolchain, commands, tests, what a green run
  proves.

There is no release runbook: tiny has no tags or published artifacts yet. CI runs the
`cargo` gates only.

## Design language

- [Design language](DESIGN-LANGUAGE.md) — the approved colour system, flexible UI guidance,
  and motion rules shared by the desktop app and the landing page.

## Records

- [`specs/`](specs/) — requirements and acceptance criteria, one file per scope.
- [`plans/`](plans/) — implementation tasks, progress, verification and handoff; each links
  its spec.

## Elsewhere

- [`../README.md`](../README.md) — the project front page.
- [`../AGENTS.md`](../AGENTS.md) — repository rules for contributors and agents.
