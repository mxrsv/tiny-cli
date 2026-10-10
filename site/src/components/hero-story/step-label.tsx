import type { Step } from "./steps";
import styles from "./hero-story.module.css";

/** Step name with its availability tag, so the story never implies a released app. */
export function StepLabel({ step }: { step: Step }) {
  return (
    <>
      {step.label}
      <span className={`${styles.tag} t-footnote`}>{step.tag}</span>
    </>
  );
}
