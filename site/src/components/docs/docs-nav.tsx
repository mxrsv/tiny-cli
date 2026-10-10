import { DOCS_SECTIONS } from "./docs-sections";
import styles from "./docs-nav.module.css";

/** Sticky on wide screens; an inline list above the article on tablets and phones. */
export function DocsNav() {
  return (
    <nav className={styles.nav} aria-labelledby="docs-nav-title">
      <p id="docs-nav-title" className={`${styles.title} t-callout t-strong`}>On this page</p>
      <ol className={`${styles.list} t-callout`}>
        {DOCS_SECTIONS.map(({ id, label }) => (
          <li key={id}>
            <a href={`#${id}`}>{label}</a>
          </li>
        ))}
      </ol>
    </nav>
  );
}
