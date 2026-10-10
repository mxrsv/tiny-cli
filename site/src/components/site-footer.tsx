import { GITHUB_URL } from "@/components/site-header";
import styles from "./site-footer.module.css";

export function SiteFooter() {
  return (
    <footer className={`${styles.footer} t-footnote`}>
      <p className={styles.tagline}>The engine is open source on GitHub.</p>
      <a className={`link ${styles.link}`} href={GITHUB_URL}>GitHub</a>
      <p>&copy; 2026 tiny</p>
    </footer>
  );
}
