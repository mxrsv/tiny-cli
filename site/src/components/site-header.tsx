import Link from "next/link";
import { NavLink } from "./nav-link";
import styles from "./site-header.module.css";

export const GITHUB_URL = "https://github.com/mxrsv/tiny-cli";

export function SiteHeader() {
  return (
    <nav className={`${styles.nav} t-body`} aria-label="Main">
      <Link className={`${styles.brand}`} href="/#top">tiny</Link>
      <Link href="/#features">Features</Link>
      <Link href="/#pricing">Pricing</Link>
      <NavLink href="/docs">Docs</NavLink>
      <a href={GITHUB_URL}>GitHub</a>
    </nav>
  );
}
