import { Fragment } from "react";
import { CodeBlock, InlineCode } from "./code-block";
import { CLI_COMMANDS } from "./cli-commands";
import { DocsSection, DocsSubhead } from "./docs-section";
import { FlagList } from "./flag-list";
import { StatusTag } from "./status-tag";

export function CliSection() {
  return (
    <DocsSection id="cli" title="Command line" tag={<StatusTag kind="cli">Available in the CLI</StatusTag>}>
      <p className="t-body">
        This engine is designed to run behind the app. Every command that can remove something shows a plan
        first. <InlineCode>clean</InlineCode> and <InlineCode>uninstall</InlineCode> default to Move to
        Trash, except Docker pruning and Time Machine snapshot deletion, which are permanent. <InlineCode>tiny --help</InlineCode> and{" "}
        <InlineCode>tiny &lt;command&gt; --help</InlineCode> list every flag, and{" "}
        <InlineCode>tiny --version</InlineCode> prints the version.
      </p>
      {CLI_COMMANDS.map((cmd) => (
        <Fragment key={cmd.id}>
          <DocsSubhead id={cmd.id}>{cmd.title}</DocsSubhead>
          <p className="t-body">{cmd.summary}</p>
          <CodeBlock label={`Examples for ${cmd.name}`}>{cmd.example}</CodeBlock>
          {cmd.flags.length > 0 && <FlagList label={`${cmd.name} options`} flags={cmd.flags} />}
          {cmd.notes.map((note) => (
            <p key={note} className="t-body">{note}</p>
          ))}
        </Fragment>
      ))}
    </DocsSection>
  );
}
