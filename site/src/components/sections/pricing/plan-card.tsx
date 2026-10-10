import styles from "./pricing.module.css";

export type PlanItem = { text: string; status: "Available in the CLI" | "Draft" };

type PlanCardProps = {
  name: string;
  tagline: string;
  price: string;
  items: PlanItem[];
};

/** One draft plan: a glass card with an explicit status label on every line. */
export function PlanCard({ name, tagline, price, items }: PlanCardProps) {
  const titleId = `plan-${name.toLowerCase()}`;
  return (
    <article className={`${styles.card} glass`} aria-labelledby={titleId}>
      <header>
        <div className={styles.head}>
          <h3 id={titleId} className="t-title">{name}</h3>
          <span className={`${styles.badge} t-footnote t-strong`}>Draft</span>
        </div>
        <p className={`${styles.tagline} t-body`}>{tagline}</p>
        <p className={`${styles.price} t-lead`}>{price}</p>
      </header>
      <ul className={styles.list}>
        {items.map((item) => (
          <li key={item.text} className={styles.item}>
            <span className="t-body">{item.text}</span>
            <span className={`${styles.status} ${styles[item.status === "Available in the CLI" ? "now" : "soon"]} t-footnote t-strong`}>
              {item.status}
            </span>
          </li>
        ))}
      </ul>
      <div className={styles.cta}>
        <a className="btn btn-primary t-body t-strong" href="/#waitlist">Get early access</a>
      </div>
    </article>
  );
}
