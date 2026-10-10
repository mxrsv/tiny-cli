import type { AppScreenStep } from "@/components/app-screen";

/** The pin needs a wide screen and permission to move. Mirror in the CSS modules (hero.module.css, hero-story.module.css). */
export const PIN_QUERY = "(min-width: 1069px) and (prefers-reduced-motion: no-preference)";

/** Distance the stage stays pinned, in viewport heights, and its offset under the sticky header pill, in px. */
export const PIN_VIEWPORTS = 2.1;
export const PIN_TOP_OFFSET = 88;

/** Progress (0 to 1) after which the Clean step shows the items moved to the Trash. */
export const MOVED_AT = 0.86;

export type Step = {
  id: AppScreenStep;
  label: string;
  caption: string;
  /** Availability label, shared with the rest of the site: "Planned for the app" or "Available in the CLI". */
  tag: string;
  /** Which orbit tile lights up while this step plays. */
  tile: "mem" | "disk" | "trash";
};

export const STEPS: readonly Step[] = [
  { id: "processes", label: "Processes", caption: "See what uses your CPU and memory.", tag: "Planned for the app", tile: "mem" },
  {
    id: "scan",
    label: "Scan",
    caption: "A read-only look at Downloads, Desktop and Documents. Not in the first app release.",
    tag: "Available in the CLI",
    tile: "disk",
  },
  { id: "clean", label: "Clean", caption: "Pick what to clear. It goes to the Trash first.", tag: "Planned for the app", tile: "trash" },
];

/** Step index for a pin progress between 0 and 1. */
export function stepAt(progress: number): number {
  return Math.min(STEPS.length - 1, Math.max(0, Math.floor(progress * STEPS.length)));
}
