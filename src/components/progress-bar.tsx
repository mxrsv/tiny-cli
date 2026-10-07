import { useUi } from "../stores/ui-store";
export function ProgressBar() {
  const progress = useUi((s) => s.progress);
  if (!progress) return null;
  const width = progress.total
    ? Math.min(100, (progress.completed / progress.total) * 100)
    : 25;
  return (
    <div className="operation-progress" role="status">
      <div className="progress-label">
        <span>
          {progress.operation === "clean" ? "Making room" : "Looking around"}
        </span>
        <span title={progress.message}>{progress.message}</span>
      </div>
      <div
        className={`progress-track ${progress.total ? "" : "indeterminate"}`}
      >
        <div style={{ width: `${width}%` }} />
      </div>
    </div>
  );
}
