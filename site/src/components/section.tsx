import type { ReactNode } from "react";
import styles from "./section.module.css";

type SectionProps = {
  id: string;
  eyebrow: string;
  title: string;
  children: ReactNode;
};

/** Shared wrapper for every landing section: eyebrow, headline, then content. */
export function Section({ id, eyebrow, title, children }: SectionProps) {
  const titleId = `${id}-title`;
  return (
    <section id={id} className={styles.section} aria-labelledby={titleId}>
      <header className={styles.header}>
        <p className={`${styles.eyebrow} t-lead`}>{eyebrow}</p>
        <h2 id={titleId} className={`${styles.title} t-headline`}>{title}</h2>
      </header>
      <div className={styles.body}>{children}</div>
    </section>
  );
}
