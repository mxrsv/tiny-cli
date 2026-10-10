import { Section } from "@/components/section";
import { FeatureTile } from "./features/feature-tile";
import { CleanVisual, MeasuredVisual, ProcessesVisual, ScanVisual } from "./features/feature-visuals";
import styles from "./features/features.module.css";

export function Features() {
  return (
    <Section id="features" eyebrow="Features" title="Everything on your Mac, in plain sight.">
      <p className={`${styles.lede} t-callout`}>
        tiny for Mac is not released yet. Features below describe what the app is designed to do.
      </p>
      <div className={styles.grid}>
        <FeatureTile
          className={styles.clean}
          title="Clean, with a preview first"
          badge="Planned for the app"
          body="Pick a category and tiny lists its paths with their sizes. Safe items start ticked, Review items start unticked, and nothing is removed until you confirm. Move to the Trash is the default, so Put Back works for what it moves."
          visual={<CleanVisual />}
        />
        <FeatureTile
          className={styles.measured}
          title="Sizes you can check"
          badge="Planned for the app"
          body="Sizes come from your disk, so you can see what each path takes before anything moves. After a clean, what you selected, what moved to the Trash and what was skipped are reported as separate numbers."
          visual={<MeasuredVisual />}
        />
        <FeatureTile
          className={styles.processes}
          title="See what is using your Mac"
          badge="Planned for the app"
          body="Processes roll up under the app they belong to, with CPU, memory and owner. When macOS withholds a number, tiny says so instead of showing zero."
          visual={<ProcessesVisual />}
        />
        <FeatureTile
          className={styles.scan}
          title="Scan your own folders"
          badge="Available in the CLI"
          body="Find large, old and duplicate files in Downloads, Desktop and Documents. A scan only reads; it never deletes. It is in the open-source tiny CLI. Not in the first app release."
          visual={<ScanVisual />}
        />
      </div>
      <p className={`${styles.note} t-footnote`}>Sizes and names in these tiles are sample data.</p>
    </Section>
  );
}
