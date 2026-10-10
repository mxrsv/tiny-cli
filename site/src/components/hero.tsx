import { GITHUB_URL } from "@/components/site-header";
import { HeroStory } from "@/components/hero-story/hero-story";
import { WaitlistForm } from "@/components/waitlist-form";
import { WidgetTile } from "@/components/widget-tile";
import styles from "./hero.module.css";

export function Hero() {
  return (
    <section className={styles.wrap} aria-label="tiny">
      <header className={styles.head}>
        <h1 className="t-display">Know your Mac. Then clean it.</h1>
        <p className={`${styles.intro} t-body`}>
          tiny shows what is using your disk and memory and moves what you pick to the Trash. Sizes come from
          your disk, so you see what each path takes before anything moves.
        </p>
        <div className={styles.cta}>
          <WaitlistForm />
          <a className="link t-callout" href={GITHUB_URL}>View source on GitHub &rsaquo;</a>
          <p className={`${styles.sample} t-footnote`}>
            tiny for Mac is not released yet. The screens below are illustrations with sample data.
          </p>
        </div>
      </header>
      <HeroStory>
        <WidgetTile
          className={`${styles.slot} ${styles.disk}`}
          tile="disk"
          label="Macintosh HD"
          value="61.4 GB free"
          numeric
          meter={{ percent: 88, description: "88 percent used" }}
        />
        <WidgetTile
          className={`${styles.slot} ${styles.mem}`}
          tile="mem"
          label="Memory in use"
          value="14.2 GB"
          numeric
          note="of 24 GB"
        />
        <WidgetTile
          className={`${styles.slot} ${styles.trash}`}
          tile="trash"
          label="Found to clean"
          value="18.6 GB"
          numeric
          note="1 Safe, 3 Review"
        />
      </HeroStory>
    </section>
  );
}
