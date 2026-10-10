import { CodeBlock, InlineCode } from "./code-block";
import { DocsSection, DocsSubhead } from "./docs-section";
import { StatusTag } from "./status-tag";

export function CleanSection() {
  return (
    <DocsSection id="clean" title="Clean" tag={<StatusTag kind="cli">Available in the CLI</StatusTag>}>
      <p className="t-body">
        Clean finds caches and leftovers, shows how much space each category takes, lets you pick, and
        asks what to do before it touches anything. Run the preview first on a new machine.
      </p>
      <CodeBlock label="Preview cleanup without changing anything">{`tiny clean --dry-run`}</CodeBlock>

      <DocsSubhead>Categories</DocsSubhead>
      <p className="t-body">There are 32 categories in three groups.</p>
      <ul className="t-body">
        <li><span className="t-strong">Dev caches:</span> cargo, npm, pnpm, yarn, node_modules, Python and Rust build output, Gradle and Maven, JetBrains, VS Code, iOS simulators, Docker, Xcode and more.</li>
        <li><span className="t-strong">User storage:</span> old downloads and screenshots, Mail attachments, streaming, chat and browser caches.</li>
        <li><span className="t-strong">System leftovers:</span> logs, user caches, crash reports, quarantine records, the Trash, Time Machine local snapshots and orphaned app data.</li>
      </ul>

      <DocsSubhead>Safe and Review labels</DocsSubhead>
      <p className="t-body">
        Every category has a risk level that decides whether you see it, and whether it starts ticked.
      </p>
      <ul className="t-body">
        <li><span className="t-strong">Safe</span> is always listed and ticked: user logs, crash reports, font and Quick Look caches, and Xcode DerivedData.</li>
        <li><span className="t-strong">Review</span> is everything else. It is listed with <InlineCode>--include-review</InlineCode> and starts unticked, because a quick look first is wise.</li>
        <li><span className="t-strong">Destructive</span> covers the Trash, Time Machine snapshots and Docker volumes. It is listed with <InlineCode>--include-destructive</InlineCode>.</li>
      </ul>

      <DocsSubhead>Preview, then Trash first</DocsSubhead>
      <ol className="t-body">
        <li>Pick categories. Safe ones are already ticked.</li>
        <li>Read the plan: up to 20 paths per category with their sizes, then a count of the rest.</li>
        <li>Choose Move to Trash (the default), Dry-run, Review paths to untick single items (or start with <InlineCode>--review-paths</InlineCode>), Hard delete, or Cancel.</li>
      </ol>
      <p className="t-body">
        Move to Trash goes through Finder, so Put Back works. Hard delete is permanent and asks you to
        confirm once more. Docker pruning and Time Machine snapshot deletion have no Trash at all: they
        are permanent whichever action you choose. Moving to the Trash is still deletion if macOS empties the Trash automatically
        (System Settings, General, Storage), so turn that off or preview first. At the end tiny prints how
        many paths it removed and lists any it could not.
      </p>

      <DocsSubhead>What it skips</DocsSubhead>
      <ul className="t-body">
        <li>A category is skipped while its app is open, for example Xcode categories while Xcode runs. tiny tells you which app to quit.</li>
        <li>Project caches are listed only when the project has been idle for 30 days. Change that with <InlineCode>--idle-days</InlineCode>.</li>
        <li>Orphaned app data is report only. A folder name does not prove an app is gone, so tiny never moves it.</li>
        <li>Docker cleanup prunes only unused images and build cache, and it is permanent. Volumes hold container data and form their own destructive category.</li>
        <li>Personal files appear only in three Review categories: old downloads, old screenshots and Mail attachments. Use <InlineCode>tiny scan</InlineCode> to look through the rest.</li>
      </ul>

      <DocsSubhead>
        In the Mac app <StatusTag kind="planned">Planned for the app</StatusTag>
      </DocsSubhead>
      <p className="t-body">
        The app is designed to use the same engine, so categories, risk levels and safety checks match the
        CLI. It is designed to show each category as a tile coloured by risk, open a per-path review sheet
        before anything moves, and report each item as moved, failed or skipped. It is designed to offer
        Move to Trash only: permanent deletion
        and emptying the Trash stay in the CLI, and Docker cleanup is report only.
      </p>
    </DocsSection>
  );
}
