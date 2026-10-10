import styles from "./code-block.module.css";

type CodeBlockProps = { label: string; children: string };

/** Mono block for shell commands. Scrolls inside itself so the page never overflows. */
export function CodeBlock({ label, children }: CodeBlockProps) {
  return (
    <pre className={`${styles.block} t-code`} tabIndex={0} aria-label={label}>
      <code>{children}</code>
    </pre>
  );
}

/** Inline command, flag or path inside prose. */
export function InlineCode({ children }: { children: string }) {
  return <code className="t-code">{children}</code>;
}
