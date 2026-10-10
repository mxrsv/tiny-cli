import type { ReactNode } from "react";
import styles from "./macbook.module.css";

/** CSS MacBook around whatever the screen shows. */
export function MacBook({ children }: { children: ReactNode }) {
  return (
    <div className={styles.mbp}>
      <div className={styles.lid}>
        <div className={styles.screen}>
          <div className={styles.notch} />
          {children}
        </div>
      </div>
      <div className={styles.base} />
    </div>
  );
}
