import { AppScreen } from "@/components/app-screen";
import { StepLabel } from "./step-label";
import { STEPS } from "./steps";
import styles from "./hero-story.module.css";

/** The three steps stacked, each with a picture cropped to the app window. Phones, tablets and reduced motion. */
export function StaticSequence() {
  return (
    <ol className={styles.sequence} aria-label="What tiny does, in three steps">
      {STEPS.map((step) => (
        <li key={step.id} className={styles.item}>
          <p className="t-lead"><StepLabel step={step} /></p>
          <p className={`${styles.itemCaption} t-body`}>{step.caption}</p>
          <div className={styles.crop}>
            <AppScreen step={step.id} />
          </div>
        </li>
      ))}
    </ol>
  );
}
