export function formatBytes(bytes: number, decimals = 1): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const unit = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), 4);
  return `${(bytes / 1024 ** unit).toLocaleString("en-US", { maximumFractionDigits: decimals })} ${["B", "KB", "MB", "GB", "TB"][unit]}`;
}
export function percent(used: number, total: number): number {
  return total > 0 ? Math.max(0, Math.min(100, (used / total) * 100)) : 0;
}
export function relativeTime(iso: string): string {
  const seconds = Math.max(0, (Date.now() - new Date(iso).getTime()) / 1000);
  if (seconds < 60) return "just now";
  if (seconds < 3600) return `${Math.floor(seconds / 60)} min ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} hr ago`;
  return new Date(iso).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });
}
export function errorMessage(error: unknown): string {
  if (error && typeof error === "object" && "message" in error)
    return String(error.message);
  return typeof error === "string"
    ? error
    : "Something went wrong. Please try again.";
}
export function recoverable(id: string): boolean {
  return !["trash", "time-machine-local", "docker"].includes(id);
}
export function humanSpace(bytes: number): string {
  if (bytes >= 1024 ** 3)
    return `About ${Math.max(1, Math.round(bytes / 1024 ** 3))} GB of breathing room.`;
  if (bytes > 0) return `${formatBytes(bytes)} you can review and reclaim.`;
  return "No recoverable candidates in the latest scan.";
}
