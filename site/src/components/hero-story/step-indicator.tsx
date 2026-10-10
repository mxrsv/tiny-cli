import { StepLabel } from "./step-label";
import { STEPS } from "./steps";
import styles from "./hero-story.module.css";

/** Text labels and captions for the three steps, so the story reads without watching motion. */
export function StepIndicator({ active }: { active: number }) {
  return (
    <ol className={styles.steps} aria-label="What tiny does, in three steps">
      {STEPS.map((step, i) => (
        <li key={step.id} className={styles.step} aria-current={i === active ? "step" : undefined}>
          <p className="t-lead"><StepLabel step={step} /></p>
          <p className="t-callout">{step.caption}</p>
        </li>
      ))}
    </ol>
  );
}
