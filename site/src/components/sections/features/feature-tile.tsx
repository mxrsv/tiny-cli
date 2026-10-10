import type { ReactNode } from "react";
import styles from "./features.module.css";

type FeatureTileProps = {
  title: string;
  body: string;
  visual: ReactNode;
  /** Text label for anything not shipped yet, e.g. "Planned for the app". */
  badge?: string;
  className?: string;
};

/** One bento tile: title, copy, then a small rendered mini-visual. */
export function FeatureTile({ title, body, visual, badge, className = "" }: FeatureTileProps) {
  return (
    <article className={`${styles.tile} glass ${className}`}>
      <h3 className={`${styles.title} t-title`}>{title}</h3>
      {badge && <p className={`${styles.badge} t-callout t-strong`}>{badge}</p>}
      <p className={`${styles.body} t-body`}>{body}</p>
      <div className={styles.visual}>{visual}</div>
    </article>
  );
}
