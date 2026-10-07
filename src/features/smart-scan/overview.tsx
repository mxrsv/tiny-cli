import {
  ArrowRight,
  Check,
  Cpu,
  HardDrive,
  Layers,
  ScanLine,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { useIsMutating } from "@tanstack/react-query";
import {
  useLatestScan,
  useMonitor,
  useSmartScan,
  selectSafe,
} from "../../hooks/use-engine";
import {
  formatBytes,
  humanSpace,
  percent,
  relativeTime,
  recoverable,
} from "../../lib/format";
import { useUi } from "../../stores/ui-store";
import { ErrorNotice } from "../../components/error-notice";
import { CommandChip } from "../../components/command-chip";
import { EmptyState } from "../../components/empty-state";
export function Overview() {
  const scan = useLatestScan();
  const monitor = useMonitor();
  const run = useSmartScan();
  const setPage = useUi((s) => s.setPage);
  const progress = useUi((s) => s.progress);
  const scanning =
    useIsMutating({ mutationKey: ["scan", "run"] }) > 0 ||
    progress?.operation === "scan";
  const result = scan.data;
  const system = monitor.data?.system ?? result?.system;
  const disk =
    system?.disks.find((d) => d.mountPoint === "/") ?? system?.disks[0];
  const used = disk ? disk.totalBytes - disk.availableBytes : 0;
  const usage = disk ? percent(used, disk.totalBytes) : 0;
  return (
    <div className="page overview-page">
      <div className="page-heading">
        <div>
          <div className="eyebrow">A SMALL UTILITY. A LIGHTER MAC.</div>
          <h1>Room to breathe.</h1>
          <p>Your Mac, a little clearer. Your day, a little lighter.</p>
        </div>
        <button
          className="button primary"
          disabled={scanning || !!progress}
          onClick={() => run.mutate()}
        >
          <ScanLine size={18} />
          {scanning ? "Scanning…" : "Smart Scan"}
        </button>
      </div>
      <ErrorNotice error={scan.error || monitor.error || run.error} />
      <div className="hero-grid">
        <section className="health-card">
          <div className="card-topline">
            <span>
              <Sparkles size={17} /> Mac wellbeing
            </span>
            {result && (
              <span className="pill subtle">
                {result.health.score >= 80
                  ? "Looking good"
                  : result.health.score >= 60
                    ? "Some room to improve"
                    : "Needs attention"}
              </span>
            )}
          </div>
          <div
            className="health-ring"
            style={
              { "--score": result?.health.score ?? 0 } as React.CSSProperties
            }
          >
            <svg viewBox="0 0 180 180" aria-hidden="true">
              <circle className="ring-track" cx="90" cy="90" r="73" />
              <circle
                className="ring-value"
                cx="90"
                cy="90"
                r="73"
                strokeDasharray={`${((result?.health.score ?? 0) / 100) * 458.67} 458.67`}
              />
            </svg>
            <div>
              <strong>{result?.health.score ?? "—"}</strong>
              <span>out of 100</span>
            </div>
          </div>
          <h3>
            {result
              ? result.health.score >= 80
                ? "A comfortable place to work."
                : "A little care goes a long way."
              : "Let’s get to know your Mac."}
          </h3>
          <p>
            {result
              ? "A simple indicator of storage, memory and cleanup opportunities."
              : "Start with a scan. Nothing changes until you review and confirm."}
          </p>
          {result && (
            <details className="score-details">
              <summary>How is this score calculated?</summary>
              {result.health.factors.map((f) => (
                <div key={f.label}>
                  <b>
                    {f.label}
                    {f.penalty > 0 ? ` · −${f.penalty}` : ""}
                  </b>
                  <p>{f.explanation}</p>
                </div>
              ))}
            </details>
          )}
        </section>
        <section className="storage-card">
          <div className="card-topline">
            <span>
              <HardDrive size={17} /> Your storage
            </span>
            <span className="caption">
              {disk?.name ?? "Waiting for system information"}
            </span>
          </div>
          <div className="storage-number">
            <strong>{disk ? formatBytes(disk.availableBytes, 0) : "—"}</strong>
            <span>
              free to create,
              <br />
              save & explore.
            </span>
          </div>
          <div className="storage-landscape" aria-hidden="true">
            <div className="hill hill-back" />
            <div className="hill hill-mid" />
            <div className="hill hill-front" />
            <div className="tiny-sun" />
          </div>
          <div className="storage-bottom">
            <div className="storage-legend">
              <span>
                <i className="dot dark" />
                {formatBytes(used)} used
              </span>
              <span>{disk ? formatBytes(disk.totalBytes, 0) : "—"} total</span>
            </div>
            <div className="storage-track">
              <span style={{ width: `${usage}%` }} />
            </div>
            <p>
              {disk && disk.availableBytes / disk.totalBytes > 0.2
                ? "Plenty of space for what comes next."
                : "A cleanup can help you make more room."}
            </p>
          </div>
        </section>
      </div>
      <section className="reclaim-banner">
        <div className="banner-icon">
          <Layers size={25} />
        </div>
        <div>
          <span className="eyebrow">A LITTLE LIGHTER</span>
          <h2>
            {result
              ? humanSpace(result.health.reclaimableBytes)
              : "Find the space you forgot you had."}
          </h2>
          <p>Review build files, caches and logs. Keep everything you need.</p>
        </div>
        <button
          className="button light"
          onClick={() => {
            if (result) selectSafe(result);
            setPage("cleanup");
          }}
        >
          {result ? "Review cleanup" : "Explore cleanup"}
          <ArrowRight size={16} />
        </button>
      </section>
      <div className="section-heading">
        <h2>At a glance</h2>
        <span className="caption">
          {result
            ? `Last scan ${relativeTime(result.createdAt)}`
            : "Live system information"}
        </span>
      </div>
      <div className="metrics-grid">
        <section className="metric-card">
          <div>
            <Cpu size={19} />
            <span>CPU</span>
          </div>
          <strong>{system ? `${Math.round(system.cpuUsage)}%` : "—"}</strong>
          <p>{system?.cpuModel || "System information loads in the app"}</p>
          <div className="mini-meter">
            <span style={{ width: `${system?.cpuUsage ?? 0}%` }} />
          </div>
        </section>
        <section className="metric-card">
          <div>
            <Layers size={19} />
            <span>Memory</span>
          </div>
          <strong>
            {system ? formatBytes(system.memoryUsed) : "—"}
            <small>
              {" "}
              / {system ? formatBytes(system.memoryTotal, 0) : "—"}
            </small>
          </strong>
          <p>Includes memory used by macOS caches</p>
          <div className="mini-meter">
            <span
              style={{
                width: `${system ? percent(system.memoryUsed, system.memoryTotal) : 0}%`,
              }}
            />
          </div>
        </section>
        <section className="metric-card">
          <div>
            <ShieldCheck size={19} />
            <span>Always your call</span>
          </div>
          <strong className="text-metric">Safe by design</strong>
          <p>
            Preview first. Trash or quarantine.
            <br />A clear way back.
          </p>
          <button className="text-button" onClick={() => setPage("safety")}>
            See your undo history <ArrowRight size={14} />
          </button>
        </section>
      </div>
      <div className="section-heading">
        <h2>Small things, worth a look</h2>
        <button className="text-button" onClick={() => setPage("cleanup")}>
          View all <ArrowRight size={14} />
        </button>
      </div>
      {result ? (
        <div className="opportunity-list">
          {result.discovery.groups
            .filter((g) => recoverable(g.id))
            .slice(0, 3)
            .map((g) => (
              <button key={g.id} onClick={() => setPage("cleanup")}>
                <div className="opportunity-icon">
                  <Layers size={18} />
                </div>
                <div>
                  <strong>{g.label}</strong>
                  <span>
                    {g.items.length}{" "}
                    {g.items.length === 1 ? "location" : "locations"} ·{" "}
                    {g.risk === "safe"
                      ? "Rebuildable files"
                      : "Your review recommended"}
                  </span>
                </div>
                <span className={`risk-dot ${g.risk}`} />
                <b>{formatBytes(g.totalSize)}</b>
                <ArrowRight size={15} />
              </button>
            ))}
          {result.discovery.groups.length === 0 && (
            <EmptyState title="All clear for now">
              No cleanup candidates were found in accessible locations.
            </EmptyState>
          )}
        </div>
      ) : (
        <div className="quiet-note">
          <Check size={17} />A Smart Scan will show your cleanup opportunities
          here.
        </div>
      )}
      <CommandChip command="tiny clean --dry-run --include-review --include-destructive" />
    </div>
  );
}
