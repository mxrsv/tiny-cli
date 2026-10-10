import type { ReactNode } from "react";
import { DocsNav } from "./docs-nav";
import styles from "./docs-layout.module.css";

/** Two columns on wide screens (nav + article); one stacked column below 1069 px. */
export function DocsLayout({ children }: { children: ReactNode }) {
  return (
    <div className={styles.layout}>
      <DocsNav />
      <article className={styles.article}>{children}</article>
    </div>
  );
}
