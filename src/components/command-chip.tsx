import { Terminal } from "lucide-react";
export function CommandChip({ command }: { command: string }) {
  return (
    <div className="command-chip">
      <Terminal size={14} />
      <code>{command}</code>
    </div>
  );
}
