import {
  ArrowUpRight,
  CalendarClock,
  Cpu,
  HardDrive,
  Layers,
} from "lucide-react";
import { useMonitor } from "../../hooks/use-engine";
import { formatBytes, percent, relativeTime } from "../../lib/format";
import { useUi } from "../../stores/ui-store";
import { ErrorNotice } from "../../components/error-notice";
import { EmptyState } from "../../components/empty-state";
import { CommandChip } from "../../components/command-chip";
export function Monitor() {
  const query = useMonitor();
  const report = query.data;
  const system = report?.system;
  const disk =
    system?.disks.find((d) => d.mountPoint === "/") ?? system?.disks[0];
  const history = [...(report?.history ?? [])].reverse();
  const values = history.map((h) => h.usedBytes);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = Math.max(max - min, 1024 ** 3);
  const points = history
    .map(
      (h, index) =>
        `${40 + (index / Math.max(1, history.length - 1)) * 800},${180 - ((h.usedBytes - min) / range) * 140}`,
    )
    .join(" ");
  return (
    <div className="page">
      <div className="page-heading">
        <div>
          <div className="eyebrow">A LITTLE AWARENESS</div>
          <h1>Keep a gentle eye.</h1>
          <p>What’s happening now, and how your space changes over time.</p>
        </div>
        <span className="pill live">
          <i className="dot" />
          Refreshes every 10 seconds
        </span>
      </div>
      <ErrorNotice error={query.error} />
      <div className="metrics-grid">
        <section className="metric-card">
          <div>
            <Cpu size={19} />
            <span>CPU activity</span>
          </div>
          <strong>{system ? `${Math.round(system.cpuUsage)}%` : "—"}</strong>
          <p>
            {system?.cpuCount ?? "—"} cores · {system?.cpuModel ?? "Loading"}
          </p>
          <div className="mini-meter">
            <span style={{ width: `${system?.cpuUsage ?? 0}%` }} />
          </div>
        </section>
        <section className="metric-card">
          <div>
            <Layers size={19} />
            <span>Memory in use</span>
          </div>
          <strong>{system ? formatBytes(system.memoryUsed) : "—"}</strong>
          <p>
            {system
              ? `${percent(system.memoryUsed, system.memoryTotal).toFixed(0)}% of ${formatBytes(system.memoryTotal)}`
              : "Loading"}
          </p>
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
            <HardDrive size={19} />
            <span>Free space</span>
          </div>
          <strong>{disk ? formatBytes(disk.availableBytes) : "—"}</strong>
          <p>{disk?.name ?? "Storage information"}</p>
          <div className="mini-meter">
            <span
              style={{
                width: `${disk ? percent(disk.availableBytes, disk.totalBytes) : 0}%`,
              }}
            />
          </div>
        </section>
      </div>
      <section className="history-card">
        <div className="card-topline">
          <h2>Your storage over time</h2>
          <span className="caption">Based on saved Smart Scans</span>
        </div>
        {history.length > 1 ? (
          <>
            <svg
              className="history-chart"
              viewBox="0 0 880 220"
              role="img"
              aria-label="Used storage across scan history"
            >
              <defs>
                <linearGradient id="history-fill" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#83a58a" stopOpacity=".35" />
                  <stop offset="100%" stopColor="#83a58a" stopOpacity="0" />
                </linearGradient>
              </defs>
              {[40, 90, 140, 190].map((y) => (
                <line key={y} x1="40" x2="840" y1={y} y2={y} stroke="#e8ede7" />
              ))}
              <polygon
                points={`40,200 ${points} 840,200`}
                fill="url(#history-fill)"
              />
              <polyline
                points={points}
                fill="none"
                stroke="#5c8066"
                strokeWidth="3"
              />
              {history.map((h, index) => (
                <circle
                  key={h.id}
                  cx={40 + (index / Math.max(1, history.length - 1)) * 800}
                  cy={180 - ((h.usedBytes - min) / range) * 140}
                  r="4"
                  fill="#5c8066"
                >
                  <title>
                    {relativeTime(h.createdAt)}: {formatBytes(h.usedBytes)}
                  </title>
                </circle>
              ))}
            </svg>
            <div className="chart-labels">
              <span>{new Date(history[0].createdAt).toLocaleDateString()}</span>
              <span>
                Used disk space · {formatBytes(min)}–{formatBytes(max)}
              </span>
              <span>
                {new Date(
                  history[history.length - 1].createdAt,
                ).toLocaleDateString()}
              </span>
            </div>
          </>
        ) : (
          <EmptyState title="Your story starts here">
            Run at least two Smart Scans to see how storage changes.
          </EmptyState>
        )}
      </section>
      <div className="two-column">
        <section className="insight-card">
          <ArrowUpRight size={22} />
          <span className="eyebrow">THIS WEEK</span>
          <h3>
            {report?.weeklyDeltaBytes != null
              ? `${formatBytes(Math.abs(report.weeklyDeltaBytes))} ${report.weeklyDeltaBytes >= 0 ? "more used" : "less used"}`
              : "A little more history needed"}
          </h3>
          <p>Compared with the oldest saved scan from the past seven days.</p>
        </section>
        <section className="insight-card">
          <CalendarClock size={22} />
          <span className="eyebrow">LOOKING AHEAD</span>
          <h3>
            {report?.daysUntilFull != null
              ? `About ${Math.round(report.daysUntilFull)} days of headroom`
              : "No forecast yet"}
          </h3>
          <p>
            A linear estimate needs at least one day of history and increasing
            disk use. Your habits can change it.
          </p>
        </section>
      </div>
      <button
        className="settings-prompt"
        onClick={() => useUi.getState().setPage("settings")}
      >
        <CalendarClock size={20} />
        <div>
          <b>Let Tiny check in for you.</b>
          <span>
            Configure scheduled scans, change detection and gentle
            notifications.
          </span>
        </div>
        <ArrowUpRight size={18} />
      </button>
      <CommandChip command="tiny sys" />
    </div>
  );
}
