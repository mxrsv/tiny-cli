import styles from "./feature-visuals.module.css";

const APPS = [
  { name: "Docker", detail: "6 processes", memory: "3.1 GB", percent: 78 },
  { name: "Xcode", detail: "4 processes", memory: "2.4 GB", percent: 60 },
  { name: "Safari", detail: "9 processes", memory: "1.2 GB", percent: 30 },
];

/** Grouped app rows, plus one row whose CPU is unavailable (not zero). */
export function ProcessesVisual() {
  return (
    <ul className={styles.list} aria-label="Sample apps by memory">
      {APPS.map((app) => (
        <li key={app.name} className={styles.appRow}>
          <span className={`${styles.rowMain} t-callout`}>
            <span className="t-strong">{app.name}</span>
            <span className={styles.dim}>{app.detail}</span>
          </span>
          <span className={`${styles.rowValue} t-callout t-num`}>{app.memory}</span>
          <span className={styles.bar} aria-hidden="true"><i style={{ width: `${app.percent}%` }} /></span>
        </li>
      ))}
      <li className={styles.appRow}>
        <span className={`${styles.rowMain} t-callout`}>
          <span className="t-strong">Slack</span>
          <span className={styles.dim}>3 processes</span>
        </span>
        <span className={`${styles.rowValue} ${styles.dim} t-callout`}>CPU unavailable</span>
      </li>
    </ul>
  );
}

const CANDIDATES = [
  { name: "Xcode DerivedData", size: "9.8 GB", risk: "safe" },
  { name: "Crash reports", size: "412 MB", risk: "safe" },
  { name: "node_modules, idle 30+ days", size: "5.1 GB", risk: "review" },
] as const;

/** Category rows with text risk labels and a skipped-because-open row. */
export function CleanVisual() {
  return (
    <div>
      <ul className={styles.list} aria-label="Sample cleanup categories">
        {CANDIDATES.map((c) => (
          <li key={c.name} className={styles.cleanRow}>
            <span className={`${styles.rowMain} t-callout`}>
              <span className="t-strong">{c.name}</span>
              {c.risk === "review" && <span className={styles.dim}>Starts unticked</span>}
            </span>
            <span className={`${styles.rowValue} t-callout t-num`}>{c.size}</span>
            <span className={`${styles.risk} ${c.risk === "safe" ? styles.safe : styles.review} t-footnote t-strong`}>
              {c.risk === "safe" ? "Safe" : "Review"}
            </span>
          </li>
        ))}
        <li className={styles.cleanRow}>
          <span className={`${styles.rowMain} t-callout`}>
            <span className="t-strong">Mail attachments</span>
            <span className={styles.dim}>Skipped while Mail is open</span>
          </span>
          <span className={`${styles.risk} ${styles.skipped} t-footnote t-strong`}>Skipped</span>
        </li>
      </ul>
      <p className={`${styles.pill} t-callout t-strong`}>Move 10.2 GB to the Trash</p>
    </div>
  );
}

const TOTALS = [
  { label: "Selected", value: "10.2 GB" },
  { label: "Moved to the Trash", value: "9.8 GB" },
  { label: "Skipped", value: "1 item" },
];

/** The three separate numbers reported after a clean. */
export function MeasuredVisual() {
  return (
    <dl className={styles.totals}>
      {TOTALS.map((t) => (
        <div key={t.label} className={styles.totalRow}>
          <dt className="t-callout">{t.label}</dt>
          <dd className="t-callout t-strong t-num">{t.value}</dd>
        </div>
      ))}
    </dl>
  );
}

const FOLDERS = ["Downloads", "Desktop", "Documents"];

/** Three scanned folders, each marked read-only in text. */
export function ScanVisual() {
  return (
    <ul className={styles.list} aria-label="Folders a scan reads">
      {FOLDERS.map((name) => (
        <li key={name} className={styles.cleanRow}>
          <span className={`${styles.rowMain} t-callout t-strong`}>{name}</span>
          <span className={`${styles.risk} ${styles.readonly} t-footnote t-strong`}>Read only</span>
        </li>
      ))}
    </ul>
  );
}
