"use client";

import { useRef, useState, type ReactNode } from "react";
import { AppScreen } from "@/components/app-screen";
import { MacBook } from "@/components/macbook";
import { StaticSequence } from "./static-sequence";
import { StepIndicator } from "./step-indicator";
import { STEPS } from "./steps";
import { usePinnedStory } from "./use-pinned-story";
import styles from "./hero-story.module.css";

/**
 * The hero product. On wide screens that allow motion the stage pins while the
 * MacBook display plays Processes, Scan and Clean as the page scrolls. Anywhere
 * else the same three steps render as a static sequence. Both are in the server
 * HTML; CSS picks one, so nothing waits on JavaScript or an animation.
 * `children` are the orbit tiles.
 */
export function HeroStory({ children }: { children: ReactNode }) {
  const stage = useRef<HTMLDivElement>(null);
  const [step, setStep] = useState(0);
  const [moved, setMoved] = useState(false);
  usePinnedStory(stage, { onStep: setStep, onMoved: setMoved });

  return (
    <div ref={stage} className={styles.stage} data-step={STEPS[step].id}>
      <div className={styles.pinned}>
        <div className={styles.device}>
          <MacBook>
            {STEPS.map((s, i) => (
              <div key={s.id} className={styles.layer} data-pos={i < step ? "past" : i > step ? "next" : "now"}>
                <AppScreen step={s.id} moved={s.id === "clean" && moved} />
              </div>
            ))}
          </MacBook>
        </div>
        <StepIndicator active={step} />
      </div>
      <StaticSequence />
      <div className={styles.tiles} role="group" aria-label="Sample data from the illustration">{children}</div>
    </div>
  );
}
