import Link from "next/link";
import { GITHUB_URL } from "@/components/site-header";
import { CodeBlock, InlineCode } from "./code-block";
import { DocsSection, DocsSubhead } from "./docs-section";
import { StatusTag } from "./status-tag";

export function InstallSection() {
  return (
    <DocsSection id="installation" title="Installation">
      <DocsSubhead>
        tiny for Mac <StatusTag kind="planned">Planned for the app</StatusTag>
      </DocsSubhead>
      <p className="t-body">
        The app is not released, so there is nothing to download yet. Use <em>Get early access</em> on
        the <Link href="/#waitlist">home page</Link> and we will email you when early access opens.
      </p>

      <DocsSubhead>
        Command-line tool <StatusTag kind="cli">Available in the CLI</StatusTag>
      </DocsSubhead>
      <p className="t-body">
        The Rust engine, designed to run behind the app, is open source, and the <InlineCode>tiny</InlineCode> command
        is built from source today. You need a Rust toolchain; <InlineCode>rustup</InlineCode> installs one.
        The source lives on <a href={GITHUB_URL}>GitHub</a>.
      </p>
      <CodeBlock label="Install the tiny command-line tool">{`git clone ${GITHUB_URL}
cd tiny-cli
cargo install --path .
tiny --help`}</CodeBlock>
      <p className="t-body">
        <InlineCode>cargo install</InlineCode> puts <InlineCode>tiny</InlineCode> in{" "}
        <InlineCode>~/.cargo/bin</InlineCode>, so make sure that folder is on your{" "}
        <InlineCode>PATH</InlineCode>. To try it without installing, run any example below with{" "}
        <InlineCode>cargo run --</InlineCode> in place of <InlineCode>tiny</InlineCode>.
      </p>
      <p className="t-body">
        <InlineCode>sys</InlineCode>, <InlineCode>scan</InlineCode> and <InlineCode>focus</InlineCode> only
        read your system or write their own log. <InlineCode>clean</InlineCode> and{" "}
        <InlineCode>uninstall</InlineCode> need macOS and move things to the Trash through Finder, so the
        first run may ask you to allow your terminal app to control Finder.
      </p>
    </DocsSection>
  );
}
