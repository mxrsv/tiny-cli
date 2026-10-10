import styles from "./status-tag.module.css";

type StatusTagProps = { kind: "planned" | "cli"; children: string };

/** Text label that separates what exists today ("Available in the CLI") from what is "Planned for the app". */
export function StatusTag({ kind, children }: StatusTagProps) {
  return <span className={`${styles.tag} ${styles[kind]} t-footnote t-strong`}>{children}</span>;
}
