import { DocsSection, DocsSubhead } from "./docs-section";
import { InlineCode } from "./code-block";
import { StatusTag } from "./status-tag";

export function FullDiskAccessSection() {
  return (
    <DocsSection id="full-disk-access" title="Full Disk Access">
      <p className="t-body">
        Some folders, such as Mail and Safari data, are protected by macOS. An app cannot read them until
        you allow it, and nothing in tiny can bypass that check. Without access tiny still works, but the
        protected locations are out of reach.
      </p>

      <DocsSubhead>
        Grant it to the Mac app <StatusTag kind="planned">Planned for the app</StatusTag>
      </DocsSubhead>
      <ol className="t-body">
        <li>Open System Settings and choose Privacy &amp; Security, then Full Disk Access.</li>
        <li>Turn tiny on. If it is not listed, click the plus button and pick it from Applications. macOS asks for your password or Touch ID.</li>
        <li>Quit and reopen tiny. macOS applies the change only to apps that start afterwards.</li>
      </ol>
      <p className="t-body">
        The app is designed to say plainly which locations it could not read, rather than show them as
        empty. You can turn the switch off again at any time.
      </p>

      <DocsSubhead>
        Grant it to the CLI <StatusTag kind="cli">Available in the CLI</StatusTag>
      </DocsSubhead>
      <p className="t-body">
        The command-line tool runs with your terminal&apos;s permissions. To let it read protected folders,
        add the terminal app you run it from (Terminal, iTerm and so on) in the same Full Disk Access list.
        The command line does not report what it could not read: it skips unreadable folders without a
        message, so a category can look smaller than it is until the terminal has access.
      </p>

      <DocsSubhead>What access does and does not mean</DocsSubhead>
      <ul className="t-body">
        <li>tiny looks only in the locations its cleanup categories name, and in the folders you give <InlineCode>scan</InlineCode>.</li>
        <li>Access lets tiny read. It never moves a path you did not tick and confirm, and it never follows symlinks out of the listed locations.</li>
        <li><InlineCode>scan</InlineCode> is strictly read-only.</li>
      </ul>

      <DocsSubhead>Finder access for the Trash</DocsSubhead>
      <p className="t-body">
        Moving files to the Trash goes through Finder so that Put Back works. For the command line, macOS
        asks once whether to allow your terminal app to control Finder; the Mac app is designed to ask for
        itself. If you refused, turn it back on under Privacy &amp; Security, then Automation. Until then
        the Trash action fails and tiny does not fall back to deleting anything.
      </p>
    </DocsSection>
  );
}
