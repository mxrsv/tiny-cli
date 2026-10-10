import type { ReactNode } from "react";
import styles from "./docs-section.module.css";

type DocsSectionProps = { id: string; title: string; tag?: ReactNode; children: ReactNode };

/** One anchored article section: h2, optional status tag, then prose. */
export function DocsSection({ id, title, tag, children }: DocsSectionProps) {
  const titleId = `${id}-title`;
  return (
    <section id={id} className={styles.section} aria-labelledby={titleId}>
      <h2 id={titleId} className={`${styles.title} t-title`}>
        {title} {tag}
      </h2>
      <div className={styles.prose}>{children}</div>
    </section>
  );
}

/** h3 inside a section; gives every sub-block the same step and spacing. */
export function DocsSubhead({ id, children }: { id?: string; children: ReactNode }) {
  return <h3 id={id} className={`${styles.subhead} t-lead`}>{children}</h3>;
}
