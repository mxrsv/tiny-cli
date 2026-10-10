import { Section } from "@/components/section";
import { GITHUB_URL } from "@/components/site-header";
import { PlanCard, type PlanItem } from "./pricing/plan-card";
import styles from "./pricing/pricing.module.css";

const PRICE_TBA = "Price to be announced";

const FREE_ITEMS: PlanItem[] = [
  { text: "The open-source tiny engine and command-line tool", status: "Available in the CLI" },
  { text: "Processes and Clean in the Mac app, with a preview before any change", status: "Draft" },
  { text: "Move to Trash by default, so Put Back works", status: "Draft" },
  { text: "Free tier limits", status: "Draft" },
];

const PRO_ITEMS: PlanItem[] = [
  { text: "Everything in Free", status: "Draft" },
  { text: "Additional app capabilities: the list is not decided yet", status: "Draft" },
  { text: "Billed as a subscription", status: "Draft" },
];

export function Pricing() {
  return (
    <Section id="pricing" eyebrow="Pricing" title="Free and Pro">
      <div className={styles.grid}>
        <PlanCard name="Free" tagline="The open engine, plus an app tier whose contents are not decided." price={PRICE_TBA} items={FREE_ITEMS} />
        <PlanCard name="Pro" tagline="More from the app, on a subscription." price={PRICE_TBA} items={PRO_ITEMS} />
      </div>
      <p className={`${styles.note} t-footnote`}>
        Both plans are drafts. Plans, features, limits and prices are not final, and the app is not released yet.
      </p>
      <p className={styles.source}>
        <a className="link t-callout" href={GITHUB_URL}>View source on GitHub &rsaquo;</a>
      </p>
    </Section>
  );
}
