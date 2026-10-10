import { InlineCode } from "./code-block";
import styles from "./flag-list.module.css";

export type Flag = { flag: string; text: string };

/** Flag reference as a definition list: stacks cleanly at 390 px, unlike a table. */
export function FlagList({ label, flags }: { label: string; flags: readonly Flag[] }) {
  return (
    <dl className={styles.list} aria-label={label}>
      {flags.map(({ flag, text }) => (
        <div key={flag} className={styles.row}>
          <dt><InlineCode>{flag}</InlineCode></dt>
          <dd className="t-callout">{text}</dd>
        </div>
      ))}
    </dl>
  );
}
