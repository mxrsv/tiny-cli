import { InlineCode } from "./code-block";
import { DocsSection, DocsSubhead } from "./docs-section";
import { StatusTag } from "./status-tag";

export function ProcessesSection() {
  return (
    <DocsSection id="processes" title="Processes" tag={<StatusTag kind="planned">Planned for the app</StatusTag>}>
      <p className="t-body">
        Processes is designed to show what is using your CPU and memory so you can decide what to clean or
        quit. It is planned for the Mac app, which is not released yet, and the CLI has no process command
        today. Everything below describes the design, not shipped behavior.
      </p>

      <DocsSubhead>What it is designed to show</DocsSubhead>
      <ul className="t-body">
        <li>Two widgets for the whole machine: CPU use, and memory used out of the total.</li>
        <li>One row per app, with its icon, process count and combined CPU and memory. Processes that belong to no app sit in a collapsed System &amp; Background group.</li>
        <li>For a selected app or process: name, PID, owner and its member processes, with search and sorting by CPU or memory.</li>
      </ul>
      <p className="t-body">
        CPU is counted per core, so a busy app can read above 100%. Data that could not be measured is
        labelled unavailable or stale, never shown as zero.
      </p>

      <DocsSubhead>How acting on a process is designed to work</DocsSubhead>
      <ul className="t-body">
        <li>Quit asks the process to stop. Force Quit is a separate action with its own confirmation, and tiny never escalates to it on its own.</li>
        <li>Each confirmation names the target and what will happen. tiny re-checks the process just before signalling and refuses system processes, itself and processes owned by other users.</li>
        <li>A Ports tile lists the listeners it can see for your account, so you can find what holds a port. Listeners it cannot see are marked unknown, never free.</li>
        <li>Reveal in Finder and Copy PID or path are available from the same menu.</li>
      </ul>

      <DocsSubhead>On the command line</DocsSubhead>
      <p className="t-body">
        A <InlineCode>tiny processes</InlineCode> command is planned, not built, so the CLI and the app share one
        implementation. Until it ships, <InlineCode>tiny sys</InlineCode> prints a one-off snapshot of CPU,
        memory and disk use.
      </p>
    </DocsSection>
  );
}
