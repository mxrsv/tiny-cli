import Link from "next/link";
import { Section } from "@/components/section";
import { GITHUB_URL } from "@/components/site-header";
import styles from "./docs-quickstart/docs-quickstart.module.css";

const STEPS = [
  {
    title: "Install the app",
    tag: "Planned for the app",
    copy: "tiny for Mac is not released yet. Add your email on the home page and we will email you when early access opens.",
    link: { href: "/#waitlist", label: "Get early access" },
  },
  {
    title: "Grant Full Disk Access",
    tag: "Planned for the app",
    copy:
      "Some folders, such as Mail and Safari data, are protected by macOS. In System Settings, open Privacy & Security, then Full Disk Access, and switch tiny on. The app is designed to say plainly which locations it could not read. For the command line, add your terminal app to the same list.",
  },
  {
    title: "Run your first Clean",
    tag: "Planned for the app",
    copy:
      "Pick categories and read the preview: the paths in each category with their sizes. Safe items start ticked, Review items start unticked, and nothing changes until you confirm. Move to the Trash is the default, so Put Back works.",
  },
];

const CLI_COMMANDS = `# Build the engine (needs Rust)
git clone ${GITHUB_URL}
cd tiny-cli
cargo install --path .

# Preview the cleanup plan. Changes nothing.
tiny clean --dry-run

# Pick, read the plan, Move to Trash
tiny clean`;

export function DocsQuickstart() {
  return (
    <Section id="docs-quickstart" eyebrow="Docs" title="Get started">
      <div className={styles.layout}>
        <ol className={styles.steps}>
          {STEPS.map((step, index) => (
            <li key={step.title} className={styles.step}>
              <span className={`${styles.num} t-lead t-num`} aria-hidden="true">{index + 1}</span>
              <div>
                <div className={styles.stepTitle}>
                  <h3 className="t-title">{step.title}</h3>
                  <span className={`${styles.tag} t-footnote t-strong`}>{step.tag}</span>
                </div>
                <p className={`${styles.copy} t-body`}>{step.copy}</p>
                {step.link && (
                  <a className={`${styles.inline} link t-body t-strong`} href={step.link.href}>{step.link.label} &rsaquo;</a>
                )}
              </div>
            </li>
          ))}
        </ol>
        <div className={`${styles.block} glass`}>
          <p className={`${styles.label} t-callout`}>Available in the CLI: the command-line engine, designed to run behind the app</p>
          <pre className={`${styles.code} t-code`} tabIndex={0} aria-label="Command-line steps">
            <code className="t-code">{CLI_COMMANDS}</code>
          </pre>
          <p className={`${styles.foot} t-footnote`}>
            Move to Trash is recoverable until macOS empties the Trash. Docker pruning and Time Machine snapshot
            deletion are permanent. Add <code className="t-code">--dry-run</code> to any <code className="t-code">clean</code> or{" "}
            <code className="t-code">uninstall</code> run to stop at the plan.
          </p>
        </div>
      </div>
      <p className={styles.docs}>
        <Link className="link t-body t-strong" href="/docs">Read the full docs &rsaquo;</Link>
      </p>
    </Section>
  );
}
