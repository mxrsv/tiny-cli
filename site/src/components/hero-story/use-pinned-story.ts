"use client";

import { useEffect, type RefObject } from "react";
import { MOVED_AT, PIN_QUERY, PIN_TOP_OFFSET, PIN_VIEWPORTS, STEPS, stepAt } from "./steps";

type Handlers = {
  onStep: (index: number) => void;
  onMoved: (moved: boolean) => void;
};

/** The element the URL hash points at, or null. */
function hashTarget(): Element | null {
  const id = window.location.hash.slice(1);
  if (!id) return null;
  try {
    return document.getElementById(decodeURIComponent(id));
  } catch {
    return null;
  }
}

/** True when the browser has just scrolled the hash target to the top of the viewport. */
function landedOn(target: Element | null): target is Element {
  if (!target) return false;
  const top = target.getBoundingClientRect().top;
  return top > -200 && top < 200;
}

/**
 * Pins `stage` and reports which step and sub-state its scroll progress is in.
 * GSAP, ScrollTrigger and Lenis are imported only while PIN_QUERY matches, so
 * phones and reduced-motion users never download them. Outside the query the
 * hook does nothing and the static sequence is what the page shows. Everything
 * is torn down when the query stops matching and on unmount.
 *
 * Adding the pin spacer moves everything below the stage, so after each pin the
 * hook puts the viewport back: on the hash target when the page opened on one,
 * otherwise, after a query flip, on the scroll position it had before.
 */
export function usePinnedStory(stage: RefObject<HTMLElement | null>, { onStep, onMoved }: Handlers) {
  useEffect(() => {
    const el = stage.current;
    if (!el) return;
    const query = window.matchMedia(PIN_QUERY);
    let generation = 0; // a late import compares against it and gives up if it lost
    let teardown: (() => void) | undefined;
    let firstPin = true;

    const disable = () => {
      generation += 1;
      teardown?.();
      teardown = undefined;
    };

    const enable = async (resumeY: number) => {
      generation += 1;
      const mine = generation;
      const [{ default: gsap }, { ScrollTrigger }, { startSmoothScroll }] = await Promise.all([
        import("gsap"),
        import("gsap/ScrollTrigger"),
        import("./smooth-scroll"),
      ]);
      if (mine !== generation) return;

      gsap.registerPlugin(ScrollTrigger);
      // On the first pin nothing above the stage has moved, so only a hash needs help.
      const isFirst = firstPin;
      const hash = isFirst ? hashTarget() : null;
      const resumeAtHash = landedOn(hash);
      firstPin = false;

      const stopSmoothScroll = startSmoothScroll();
      const apply = (progress: number) => {
        const index = stepAt(progress);
        onStep(index);
        onMoved(index === STEPS.length - 1 && progress >= MOVED_AT);
      };
      const trigger = ScrollTrigger.create({
        trigger: el,
        start: `top top+=${PIN_TOP_OFFSET}`,
        end: () => `+=${Math.round(window.innerHeight * PIN_VIEWPORTS)}`,
        pin: true,
        invalidateOnRefresh: true,
        onUpdate: (self) => apply(self.progress),
        onRefresh: (self) => apply(self.progress),
      });

      let stopped = false;
      void document.fonts.ready.then(() => {
        if (!stopped) ScrollTrigger.refresh();
      });
      teardown = () => {
        stopped = true;
        trigger.kill();
        stopSmoothScroll();
        onStep(0);
        onMoved(false);
      };

      ScrollTrigger.refresh();
      if (resumeAtHash) hash.scrollIntoView({ behavior: "instant" });
      else if (!isFirst) window.scrollTo({ top: resumeY, behavior: "instant" });
    };

    const onChange = () => {
      const y = window.scrollY;
      disable();
      if (query.matches) void enable(y);
    };

    query.addEventListener("change", onChange);
    if (query.matches) void enable(window.scrollY);

    return () => {
      query.removeEventListener("change", onChange);
      disable();
    };
  }, [stage, onStep, onMoved]);
}
