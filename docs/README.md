# tiny-cli documentation

Three tiers, split by reader. Every page describes the shipped `tiny` CLI on `main` in the
present tense; the planned native desktop app says so wherever it appears.

## `user/` — using tiny

- [Getting started](user/getting-started.md) — build and install, the command set, `sys` and
  `focus`.
- [Scanning files](user/scan.md) — the read-only report, custom roots, `.tinyignore`,
  duplicates and JSON output.
- [Cleaning caches](user/clean.md) — categories and risk, the picker and action menu, the
  flags, the safety model.
- [Uninstalling apps](user/uninstall.md) — what is removed, the action menu, what is refused.

## `internals/` — how tiny is built

- [Overview](internals/overview.md) — the binary's shape, how a command is wired, how deletion
  works, where the desktop migration stands.
- [Glossary](internals/glossary.md) — the words the code and docs use.
- [Clean pipeline](internals/clean.md) — validation, provider selection, discovery, the
  action mapping, the test seams.
- [Known traps](internals/traps.md) — what is not obvious from reading one file.

## `operations/` — building and checking

- [Development](operations/development.md) — toolchain, commands, tests, what a green run
  proves.

There is no release runbook: tiny has no tags, CI or published artifacts yet.

## Design language

- [Design language](DESIGN-LANGUAGE.md) — the approved primary colour, flexible UI guidance,
  and motion rules shared by the desktop app and the landing page.

## Records

- [`specs/`](specs/) — requirements and acceptance criteria, one file per scope.
- [`plans/`](plans/) — implementation tasks, progress, verification and handoff; each links
  its spec.

## Elsewhere

- [`../README.md`](../README.md) — the project front page.
- [`../AGENTS.md`](../AGENTS.md) — repository rules for contributors and agents.
- [`../_bmad-output/planning-artifacts/`](../_bmad-output/planning-artifacts/) — PRD,
  architecture and epics for the desktop migration, owned by the BMad workflow.
