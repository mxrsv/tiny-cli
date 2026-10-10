import type { Flag } from "./flag-list";

export type CliCommand = {
  id: string;
  name: string;
  title: string;
  summary: string;
  example: string;
  flags: readonly Flag[];
  notes: readonly string[];
};

/** Verified against src/cli.rs and `tiny <command> --help` (tiny 0.1.0). */
export const CLI_COMMANDS: readonly CliCommand[] = [
  {
    id: "cli-sys",
    name: "tiny sys",
    title: "System information",
    summary:
      "Prints one snapshot of the OS, host, uptime, CPU, memory and per-disk usage, then exits. Nothing is written anywhere.",
    example: "tiny sys",
    flags: [],
    notes: [],
  },
  {
    id: "cli-scan",
    name: "tiny scan",
    title: "Scan files",
    summary:
      "Reports large and old files so you can decide what to keep. It never deletes, moves or changes anything. By default it looks in ~/Downloads, ~/Desktop and ~/Documents.",
    example: `tiny scan
tiny scan --min-size-mb 500 --older-than-days 365
tiny scan --path ~/Movies --path ./assets
tiny scan --by-ext --duplicates
tiny scan --json > report.json`,
    flags: [
      { flag: "--min-size-mb <MIN_SIZE_MB>", text: "Report files at or above this size in megabytes. Default 100." },
      { flag: "--older-than-days <OLDER_THAN_DAYS>", text: "Report files not modified for this many days. Default 90." },
      { flag: "--path <DIR>", text: "Scan this folder instead of the default three. Repeatable; each use replaces the default set." },
      { flag: "--limit <LIMIT>", text: "Items shown per section in text mode. Default 20." },
      { flag: "--sort <SORT>", text: "size (largest first, default), age (oldest first) or path." },
      { flag: "--by-ext", text: "Add a per-extension breakdown: file count and total size." },
      { flag: "--duplicates", text: "Group files that share both size and filename." },
      { flag: "--hash", text: "With --duplicates, confirm groups by comparing file content. Slower, but drops files that only looked alike." },
      { flag: "--json", text: "Print one JSON object on stdout instead of text." },
    ],
    notes: [
      "Version-control folders, build and package output such as node_modules and target, macOS metadata folders and anything deeper than 12 levels are always skipped. A .tinyignore file in a scanned folder or your home folder skips more: one folder name or *.ext pattern per line.",
    ],
  },
  {
    id: "cli-clean",
    name: "tiny clean",
    title: "Clean caches",
    summary:
      "Interactive cleanup of caches and recoverable data. It needs macOS, shows a plan first and defaults to Move to Trash, except for Docker pruning and Time Machine snapshots, which are permanent. The Clean section above explains the categories and risk levels.",
    example: `tiny clean --dry-run
tiny clean
tiny clean --include-review --review-paths
tiny clean --category node-modules --idle-days 60`,
    flags: [
      { flag: "--dry-run", text: "Show the report and exit without prompting for cleanup." },
      { flag: "--category <CATEGORY>", text: "Restrict discovery and selection to this category. Repeatable; shows it whatever its risk level." },
      { flag: "--include-review", text: "List review-risk categories in the picker, unticked." },
      { flag: "--include-destructive", text: "List destructive categories such as Trash, unticked." },
      { flag: "--review-paths", text: "Untick individual paths after picking categories. Cannot be combined with --yes." },
      { flag: "--idle-days <IDLE_DAYS>", text: "Project caches and old downloads or screenshots are listed only when untouched for this many days. Default 30, must be above 0." },
      { flag: "-y, --yes", text: "Skip the picker and the action menu. Requires --category and cannot be combined with --dry-run." },
      { flag: "--hard", text: "Delete permanently instead of moving to the Trash. Not recoverable." },
    ],
    notes: [
      "Permanent deletion without a prompt needs TINY_CONFIRM_HARD=1 in the environment as well as -y and --hard.",
    ],
  },
  {
    id: "cli-focus",
    name: "tiny focus",
    title: "Focus timer",
    summary:
      "A focus timer with a progress bar. A finished session is appended to ~/.tiny-cli/focus-sessions.json; stopping early with Ctrl-C records nothing.",
    example: `tiny focus
tiny focus --minutes 50 --label "deep work"`,
    flags: [
      { flag: "--minutes <MINUTES>", text: "Length of the session in minutes. Default 25." },
      { flag: "--label <LABEL>", text: "Optional label stored with the session." },
    ],
    notes: [
      "If the log file is not valid JSON, tiny refuses to overwrite it: it copies the file to focus-sessions.json.bak and asks you to fix or delete the original.",
    ],
  },
  {
    id: "cli-uninstall",
    name: "tiny uninstall",
    title: "Uninstall apps",
    summary:
      "Removes an app from /Applications together with the data it left in ~/Library. It needs macOS, reports what it found and asks before removing anything.",
    example: `tiny uninstall
tiny uninstall AltTab --dry-run
tiny uninstall AltTab --leftovers-only`,
    flags: [
      { flag: "[NAME]", text: "App name. Without it, an interactive picker lists every app in /Applications." },
      { flag: "--dry-run", text: "Only show the report and exit." },
      { flag: "--shallow", text: "Remove only the .app, skip ~/Library. Cannot be combined with --leftovers-only." },
      { flag: "--leftovers-only", text: "Clean only the ~/Library data and keep the app. Cannot be combined with --shallow." },
      { flag: "--sort <SORT>", text: "Picker order: last-used (default, least recently used first), size or name." },
      { flag: "-y, --yes", text: "Skip the action menu and move to the Trash immediately." },
      { flag: "--hard", text: "Delete permanently instead of using the Trash. Not recoverable." },
      { flag: "--force", text: "Allow removing apps installed as Homebrew casks, which tiny refuses by default." },
    ],
    notes: [
      "Apple's own apps are always refused. If any selected app is refused, tiny stops after the report and removes nothing.",
      "Unlike clean, uninstall --hard -y deletes permanently with no further confirmation, so run --dry-run first.",
    ],
  },
];
