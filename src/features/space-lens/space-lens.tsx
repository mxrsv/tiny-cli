import { useEffect, useState } from "react";
import {
  ChevronRight,
  Folder,
  Grid2X2,
  ScanLine,
  CircleDot,
  ArrowLeft,
  Info,
} from "lucide-react";
import { useMutation } from "@tanstack/react-query";
import { ipc } from "../../lib/ipc";
import { formatBytes } from "../../lib/format";
import { layoutSunburst, layoutTreemap } from "../../lib/treemap";
import type { HeatMode } from "../../lib/treemap";
import type { SpaceNode } from "../../types/ipc";
import { useMonitor, usePermissions } from "../../hooks/use-engine";
import { useUi } from "../../stores/ui-store";
import { ErrorNotice } from "../../components/error-notice";
import { EmptyState } from "../../components/empty-state";
import { CommandChip } from "../../components/command-chip";
export function SpaceLens() {
  const permissions = usePermissions();
  const monitor = useMonitor();
  const [path, setPath] = useState("");
  const [crumbs, setCrumbs] = useState<SpaceNode[]>([]);
  const [heat, setHeat] = useState<HeatMode>("type");
  const [view, setView] = useState<"treemap" | "sunburst">("treemap");
  const busy = useUi((s) => !!s.progress);
  useEffect(() => {
    if (permissions.data)
      setPath((current) => current || permissions.data.homePath);
  }, [permissions.data]);
  const scan = useMutation({
    mutationFn: (targetPath: string) => ipc.spaceLens(targetPath),
    onMutate: () =>
      useUi.getState().setProgress({
        operation: "space-lens",
        completed: 0,
        total: null,
        message: path,
      }),
    onSuccess: (result) => setCrumbs([result.root]),
    onSettled: () => useUi.getState().setProgress(null),
  });
  const current = crumbs[crumbs.length - 1];
  const tiles = current ? layoutTreemap(current, 900, 380, heat) : [];
  const arcs = current ? layoutSunburst(current, 175, heat) : [];
  const disk = monitor.data?.system.disks.find((d) => d.mountPoint === "/");
  const unexplained =
    scan.data?.root.path === "/" && disk
      ? Math.max(
          0,
          disk.totalBytes - disk.availableBytes - scan.data.root.allocatedBytes,
        )
      : null;
  const drill = (node: SpaceNode) => {
    if (node.isDirectory) {
      setCrumbs([...crumbs, node]);
      setPath(node.path);
      if (node.children.length === 0 && node.bytes > 0)
        scan.mutate(node.path, {
          onSuccess: (result) => setCrumbs([...crumbs, result.root]),
        });
    }
  };
  return (
    <div className="page">
      <div className="page-heading">
        <div>
          <div className="eyebrow">SEE THE BIG PICTURE</div>
          <h1>Space, made visible.</h1>
          <p>
            A map of what lives on your Mac. Explore without changing a thing.
          </p>
        </div>
        <span className="pill">
          <Folder size={14} />
          Read-only
        </span>
      </div>
      <form
        className="path-form"
        onSubmit={(event) => {
          event.preventDefault();
          scan.mutate(path);
        }}
      >
        <Folder size={18} />
        <input
          aria-label="Directory to map"
          value={path}
          onChange={(event) => setPath(event.target.value)}
          placeholder="/Users/your-name"
        />
        <button
          type="button"
          className="text-button"
          onClick={() => setPath("/")}
          disabled={busy}
        >
          Whole disk
        </button>
        <button className="button primary" disabled={busy || !path}>
          <ScanLine size={17} />
          {scan.isPending ? "Mapping…" : "Map space"}
        </button>
      </form>
      <ErrorNotice error={scan.error} />
      {!current ? (
        <EmptyState title="Every file has a place">
          Choose a directory or the whole disk, then map your space. Larger
          blocks mean larger folders.
        </EmptyState>
      ) : (
        <>
          <div className="lens-toolbar">
            <div className="breadcrumbs">
              {crumbs.map((node, index) => (
                <span key={node.path}>
                  <button
                    onClick={() => {
                      setCrumbs(crumbs.slice(0, index + 1));
                      setPath(node.path);
                    }}
                  >
                    {index === 0 ? <Folder size={14} /> : null}
                    {node.name}
                  </button>
                  {index < crumbs.length - 1 && <ChevronRight size={13} />}
                </span>
              ))}
            </div>
            <div className="segmented">
              <button
                className={view === "treemap" ? "active" : ""}
                aria-label="Treemap view"
                onClick={() => setView("treemap")}
              >
                <Grid2X2 size={16} />
              </button>
              <button
                className={view === "sunburst" ? "active" : ""}
                aria-label="Sunburst view"
                onClick={() => setView("sunburst")}
              >
                <CircleDot size={16} />
              </button>
            </div>
            <select
              aria-label="Color map by"
              value={heat}
              onChange={(e) => setHeat(e.target.value as HeatMode)}
            >
              <option value="type">Color by branch</option>
              <option value="age">Color by age</option>
            </select>
          </div>
          <section className="lens-map">
            <div className="map-topline">
              <span>{current.name}</span>
              <strong>{formatBytes(current.bytes)}</strong>
            </div>
            {current.bytes === 0 ? (
              <EmptyState title="No visible files">
                This folder is empty or could not be read.
              </EmptyState>
            ) : view === "treemap" ? (
              <div className="treemap">
                {tiles.map((tile) => (
                  <button
                    key={tile.node.path}
                    className="tree-tile"
                    title={`${tile.node.path} · ${formatBytes(tile.node.bytes)}`}
                    onClick={() => drill(tile.node)}
                    style={{
                      left: `${tile.x / 9}%`,
                      top: `${tile.y / 3.8}%`,
                      width: `${tile.width / 9}%`,
                      height: `${tile.height / 3.8}%`,
                      background: tile.color,
                    }}
                  >
                    <span>
                      {tile.node.isDirectory && <Folder size={13} />}{" "}
                      {tile.node.name}
                    </span>
                    <b>{formatBytes(tile.node.bytes)}</b>
                    {tile.node.isDirectory && tile.height > 80 && (
                      <small>
                        Click to explore <ChevronRight size={12} />
                      </small>
                    )}
                  </button>
                ))}
              </div>
            ) : (
              <div className="sunburst">
                <svg
                  viewBox="-195 -195 390 390"
                  role="img"
                  aria-label="Disk usage sunburst"
                >
                  {arcs.map((arc, index) => (
                    <path
                      key={`${arc.node.path}-${index}`}
                      d={arc.path}
                      fill={arc.color}
                      stroke="#f7f8f6"
                      strokeWidth="2"
                      onClick={() => drill(arc.node)}
                    >
                      <title>
                        {arc.node.name}: {formatBytes(arc.node.bytes)}
                      </title>
                    </path>
                  ))}
                  <text textAnchor="middle" y="-4">
                    {formatBytes(current.bytes)}
                  </text>
                  <text textAnchor="middle" y="16" className="sunburst-caption">
                    {current.name}
                  </text>
                </svg>
              </div>
            )}
            <div className="map-bottomline">
              <span>
                {scan.data?.filesScanned.toLocaleString()} files scanned
              </span>
              {crumbs.length > 1 ? (
                <button
                  className="text-button"
                  onClick={() => {
                    setPath(crumbs[crumbs.length - 2].path);
                    setCrumbs(crumbs.slice(0, -1));
                  }}
                >
                  <ArrowLeft size={13} />
                  Back one folder
                </button>
              ) : (
                <span>Click a folder to look inside</span>
              )}
            </div>
          </section>
          {heat === "age" && (
            <div className="age-legend">
              <span>Recently modified</span>
              <div />
              <span>Older than 6 months</span>
            </div>
          )}
          <div className="lens-file-list">
            {current.children.slice(0, 12).map((node) => (
              <button key={node.path} onClick={() => drill(node)}>
                <Folder size={16} />
                <span>{node.name}</span>
                <b>{formatBytes(node.bytes)}</b>
                {node.isDirectory && <ChevronRight size={15} />}
              </button>
            ))}
          </div>
          {(scan.data?.truncated || !!scan.data?.skippedPaths.length) && (
            <div className="quiet-note">
              <Info size={17} />
              <span>
                {scan.data.truncated
                  ? "Large folders show the 200 biggest entries. Explore deeper folders to expand the map. Scan depth is limited to 64; totals may be incomplete beyond that. "
                  : ""}
                {scan.data.skippedPaths.length > 0
                  ? `${scan.data.skippedPaths.length} inaccessible paths were recorded. Totals may be incomplete.`
                  : ""}
              </span>
            </div>
          )}
          {unexplained !== null && (
            <div className="quiet-note">
              <Info size={17} />
              {formatBytes(unexplained)} of disk use is outside the visible
              allocated files. APFS snapshots, shared blocks and inaccessible
              locations can affect this estimate.
            </div>
          )}
        </>
      )}
      <CommandChip
        command={`tiny scan --path '${path.replaceAll("'", "'\\''")}' --by-ext --json`}
      />
      <p className="caption">
        CLI scan uses its own ignore rules and depth limit; Space Lens includes
        build folders and scans more deeply.
      </p>
    </div>
  );
}
