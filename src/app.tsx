import { useEffect, Component } from "react";
import type { ReactNode } from "react";
import {
  ArchiveRestore,
  ArrowUpRight,
  ChartNoAxesCombined,
  CircleDot,
  Grid2X2,
  LayoutDashboard,
  Leaf,
  Settings2,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import {
  useEngineEvents,
  useLatestScan,
  usePermissions,
} from "./hooks/use-engine";
import { desktop } from "./lib/ipc";
import { useUi } from "./stores/ui-store";
import type { Page } from "./stores/ui-store";
import { Overview } from "./features/smart-scan/overview";
import { Cleanup } from "./features/cleanup/cleanup";
import { SpaceLens } from "./features/space-lens/space-lens";
import { Monitor } from "./features/monitor/monitor";
import { Safety } from "./features/safety/safety";
import { Settings } from "./features/settings/settings";
import { Onboarding } from "./features/onboarding/onboarding";
import { ErrorNotice } from "./components/error-notice";
import { ProgressBar } from "./components/progress-bar";
const navigation = [
  { id: "overview", label: "Overview", icon: LayoutDashboard },
  { id: "cleanup", label: "Cleanup", icon: Sparkles },
  { id: "space-lens", label: "Space Lens", icon: Grid2X2 },
  { id: "monitor", label: "Activity", icon: ChartNoAxesCombined },
  { id: "safety", label: "Undo & recovery", icon: ArchiveRestore },
] as const;
export default function App() {
  useEngineEvents();
  const page = useUi((s) => s.page);
  const engineError = useUi((s) => s.engineError);
  const setPage = useUi((s) => s.setPage);
  const { data: scan } = useLatestScan();
  const permission = usePermissions();
  useEffect(() => {
    document.title = `Tiny · ${page === "overview" ? "room to breathe" : (navigation.find((n) => n.id === page)?.label ?? "Preferences")}`;
  }, [page]);
  const labels: Record<Page, string> = {
    overview: "Overview",
    cleanup: "Cleanup",
    "space-lens": "Space Lens",
    monitor: "Activity",
    safety: "Undo & recovery",
    settings: "Preferences",
  };
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">
            <Leaf size={22} />
          </div>
          <span>
            tiny<span className="brand-dot">.</span>
          </span>
        </div>
        <div className="workspace-label">YOUR LITTLE WORKSPACE</div>
        <nav aria-label="Main navigation">
          {navigation.map(({ id, label, icon: Icon }) => (
            <button
              className={page === id ? "nav-item active" : "nav-item"}
              key={id}
              onClick={() => setPage(id)}
            >
              <Icon size={19} />
              <span>{label}</span>
              {id === "cleanup" && scan?.health.reclaimableBytes ? <i /> : null}
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <div className="sidebar-note">
            <div className="small-leaf">
              <Leaf size={20} />
            </div>
            <h3>Small is a good thing.</h3>
            <p>
              Less clutter.
              <br />
              More room for your day.
            </p>
            <span>
              Made for your Mac <ArrowUpRight size={13} />
            </span>
          </div>
          <button
            className={page === "settings" ? "nav-item active" : "nav-item"}
            onClick={() => setPage("settings")}
          >
            <Settings2 size={19} />
            <span>Preferences</span>
          </button>
          <div className="sidebar-footer">
            <CircleDot size={12} />
            <span>Local by nature</span>
            <small>v0.1</small>
          </div>
        </div>
      </aside>
      <main className="main-panel">
        <header className="app-toolbar">
          <div>
            <span>My Mac</span>
            <span className="toolbar-slash">/</span>
            <strong>{labels[page]}</strong>
          </div>
          <span className="toolbar-status">
            {desktop ? (
              <>
                <ShieldCheck size={14} />
                Everything stays on your Mac
              </>
            ) : (
              <>Read-only browser demo</>
            )}
          </span>
        </header>
        {!desktop && (
          <div className="demo-strip">
            Demo data · No files on your computer are accessed or changed.
          </div>
        )}
        {permission.data?.status === "unknown" && (
          <div className="access-strip">
            <ShieldCheck size={15} />
            Access could not be fully verified. Some locations may be missing.
            <button className="text-button" onClick={() => setPage("settings")}>
              Review access
            </button>
          </div>
        )}
        <ProgressBar />
        {engineError && (
          <div className="engine-error">
            <ErrorNotice error={engineError} />
            <button
              className="text-button"
              onClick={() => useUi.getState().setEngineError(null)}
            >
              Dismiss
            </button>
          </div>
        )}
        <div className="page-container">
          <ErrorBoundary>
            {page === "overview" ? (
              <Overview />
            ) : page === "cleanup" ? (
              <Cleanup />
            ) : page === "space-lens" ? (
              <SpaceLens />
            ) : page === "monitor" ? (
              <Monitor />
            ) : page === "safety" ? (
              <Safety />
            ) : (
              <Settings />
            )}
          </ErrorBoundary>
          <footer className="main-footer">
            <span>
              <Leaf size={13} />A little care goes a long way.
            </span>
            <span>Tiny · made to make room</span>
          </footer>
        </div>
      </main>
      <Onboarding />
    </div>
  );
}
class ErrorBoundary extends Component<
  { children: ReactNode },
  { error: boolean }
> {
  state = { error: false };
  static getDerivedStateFromError() {
    return { error: true };
  }
  render() {
    return this.state.error ? (
      <div className="empty-state">
        <h2>Something interrupted this view.</h2>
        <p>Your files are safe. Reload the app to try again.</p>
        <button
          className="button primary"
          onClick={() => window.location.reload()}
        >
          Reload Tiny
        </button>
      </div>
    ) : (
      this.props.children
    );
  }
}
