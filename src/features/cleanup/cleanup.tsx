import { useEffect, useRef, useState } from "react";
import {
  ArrowLeft,
  ArrowRight,
  ChevronDown,
  ChevronUp,
  File,
  Folder,
  ScanLine,
  ShieldCheck,
  X,
} from "lucide-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  useLatestScan,
  useSmartScan,
  selectSafe,
} from "../../hooks/use-engine";
import { ipc, desktop } from "../../lib/ipc";
import { formatBytes, recoverable } from "../../lib/format";
import { useUi } from "../../stores/ui-store";
import { ErrorNotice } from "../../components/error-notice";
import { EmptyState } from "../../components/empty-state";
import { CommandChip } from "../../components/command-chip";
import type { CleanupPreview, Risk } from "../../types/ipc";
const riskLabels: Record<Risk, string> = {
  safe: "Ready to clear",
  review: "Take a look",
  destructive: "Handle with care",
};
export function Cleanup() {
  const { data: scan, error } = useLatestScan();
  const run = useSmartScan();
  const selected = useUi((s) => s.selectedPaths);
  const select = useUi((s) => s.selectPaths);
  const toggle = useUi((s) => s.togglePath);
  const progress = useUi((s) => s.progress);
  const [expanded, setExpanded] = useState<string[]>([]);
  const [preview, setPreview] = useState<CleanupPreview | null>(null);
  const client = useQueryClient();
  const previewMutation = useMutation({
    mutationFn: () => ipc.preview(scan?.id ?? "", selected),
    onSuccess: setPreview,
  });
  const clean = useMutation({
    mutationFn: () => ipc.clean(preview?.id ?? "", true),
    onSuccess: () => {
      setPreview(null);
      client.setQueryData(["scan", "result"], null);
      select([]);
      void client.invalidateQueries({ queryKey: ["safety"] });
      void client.invalidateQueries({ queryKey: ["system", "monitor"] });
    },
    onSettled: () => useUi.getState().setProgress(null),
  });
  const selectedBytes =
    scan?.discovery.groups
      .flatMap((g) => g.items)
      .filter((i) => selected.includes(i.path))
      .reduce((total, i) => total + i.size, 0) ?? 0;
  const busy = !!progress || previewMutation.isPending || clean.isPending;
  // A background scan invalidates every preview. Avoid displaying an obsolete confirmation.
  useEffect(() => {
    setPreview(null);
  }, [scan?.id]);
  return (
    <div className="page">
      <div className="page-heading">
        <div>
          <div className="eyebrow">KEEP WHAT MATTERS</div>
          <h1>A lighter Mac.</h1>
          <p>A little review now. A lot of space for later.</p>
        </div>
        <button
          className="button secondary"
          disabled={run.isPending || busy}
          onClick={() => run.mutate()}
        >
          <ScanLine size={17} />
          {run.isPending ? "Scanning…" : "Scan again"}
        </button>
      </div>
      <ErrorNotice
        error={error || run.error || previewMutation.error || clean.error}
      />
      {clean.data && (
        <div className="success-notice" role="status">
          <ShieldCheck size={18} />
          {clean.data.movedCount} paths moved ·{" "}
          {formatBytes(clean.data.movedBytes)} cleared from their original
          locations. Check Undo for recovery.
          {clean.data.failed.map(([path, message]) => (
            <p key={path}>
              {path}: {message}
            </p>
          ))}
        </div>
      )}
      {!scan ? (
        <EmptyState title="Start with a fresh look">
          Run Smart Scan to discover candidates. Cleanup always begins with your
          review.
        </EmptyState>
      ) : (
        <>
          <div className="cleanup-toolbar">
            <span>{scan.discovery.groups.length} categories found</span>
            <button
              className="text-button"
              disabled={busy}
              onClick={() => selectSafe(scan)}
            >
              Select safe categories
            </button>
            <button
              className="text-button muted"
              disabled={busy}
              onClick={() => select([])}
            >
              Clear selection
            </button>
          </div>
          <div className="risk-lanes">
            {(["safe", "review", "destructive"] as const).map((risk) => {
              const groups = scan.discovery.groups.filter(
                (g) => g.risk === risk,
              );
              return (
                <section className={`risk-lane ${risk}`} key={risk}>
                  <div className="lane-heading">
                    <i className={`risk-dot ${risk}`} />
                    <h3>{riskLabels[risk]}</h3>
                    <span>{groups.length}</span>
                  </div>
                  <p className="lane-description">
                    {risk === "safe"
                      ? "Rebuildable caches and logs."
                      : risk === "review"
                        ? "May contain files you want to keep."
                        : "Irreversible providers are report-only."}
                  </p>
                  {groups.length === 0 && (
                    <p className="no-candidates">Nothing here. A good sign.</p>
                  )}
                  {groups.map((group) => {
                    const allowed = recoverable(group.id);
                    const allSelected = group.items.every((i) =>
                      selected.includes(i.path),
                    );
                    const open = expanded.includes(group.id);
                    return (
                      <div className="category-card" key={group.id}>
                        <div className="category-main">
                          <input
                            aria-label={`Select ${group.label}`}
                            type="checkbox"
                            disabled={!allowed || busy}
                            checked={allSelected && allowed}
                            onChange={() =>
                              select(
                                allSelected
                                  ? selected.filter(
                                      (path) =>
                                        !group.items.some(
                                          (i) => i.path === path,
                                        ),
                                    )
                                  : [
                                      ...selected,
                                      ...group.items.map((i) => i.path),
                                    ],
                              )
                            }
                          />
                          <button
                            className="category-title"
                            aria-expanded={open}
                            onClick={() =>
                              setExpanded(
                                open
                                  ? expanded.filter((id) => id !== group.id)
                                  : [...expanded, group.id],
                              )
                            }
                          >
                            <span>{group.label}</span>
                            <b>{formatBytes(group.totalSize)}</b>
                            <small>
                              {group.items.length}{" "}
                              {group.items.length === 1
                                ? "location"
                                : "locations"}{" "}
                              {!allowed && "· report only"}
                            </small>
                            {open ? (
                              <ChevronUp size={14} />
                            ) : (
                              <ChevronDown size={14} />
                            )}
                          </button>
                        </div>
                        {open && (
                          <div className="path-list">
                            {group.items.map((item) => (
                              <label key={item.path}>
                                <input
                                  type="checkbox"
                                  aria-label={`Select ${item.path}`}
                                  disabled={!allowed || busy}
                                  checked={
                                    selected.includes(item.path) && allowed
                                  }
                                  onChange={() => toggle(item.path)}
                                />
                                <Folder size={14} />
                                <span title={item.path}>{item.path}</span>
                                <b>{formatBytes(item.size)}</b>
                              </label>
                            ))}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </section>
              );
            })}
          </div>
          {scan.discovery.skippedRunning.length > 0 && (
            <div className="quiet-note">
              <ShieldCheck size={17} />
              Skipped{" "}
              {scan.discovery.skippedRunning
                .map(([category, app]) => `${category} (${app} is open)`)
                .join(", ")}
              . Quit the app and scan again to review those files.
            </div>
          )}
          <div className="cleanup-footer">
            <div>
              <span>{selected.length} paths selected</span>
              <strong>{formatBytes(selectedBytes)}</strong>
            </div>
            <p>Nothing is removed before you confirm.</p>
            <button
              className="button primary"
              disabled={selected.length === 0 || busy}
              onClick={() => {
                clean.reset();
                previewMutation.mutate();
              }}
            >
              Preview cleanup <ArrowRight size={17} />
            </button>
          </div>
        </>
      )}
      {preview && (
        <PreviewDialog
          preview={preview}
          pending={clean.isPending}
          error={clean.error}
          onClose={() => setPreview(null)}
          onConfirm={() => clean.mutate()}
        />
      )}
    </div>
  );
}
function PreviewDialog({
  preview,
  pending,
  error,
  onClose,
  onConfirm,
}: {
  preview: CleanupPreview;
  pending: boolean;
  error: unknown;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);
  return (
    <dialog
      ref={dialog}
      className="preview-dialog"
      onCancel={(event) => {
        if (pending) event.preventDefault();
        else onClose();
      }}
      aria-labelledby="preview-title"
    >
      <div className="dialog-heading">
        <div className="banner-icon">
          <ShieldCheck size={24} />
        </div>
        <button
          className="icon-button"
          aria-label="Close preview"
          disabled={pending}
          onClick={onClose}
        >
          <X size={20} />
        </button>
      </div>
      <div className="eyebrow">YOUR LAST LOOK</div>
      <h2 id="preview-title">Make a little room.</h2>
      <p>
        Review {preview.items.length} paths totaling{" "}
        <b>{formatBytes(preview.totalBytes)}</b>. Tiny tries Trash first, then
        quarantine if needed.
      </p>
      <div className="preview-paths">
        {preview.items.map((item) => (
          <div key={item.path}>
            <File size={16} />
            <span title={item.path}>
              {item.path}
              <small>{item.categoryLabel}</small>
            </span>
            <b>{formatBytes(item.bytes)}</b>
          </div>
        ))}
      </div>
      <div className="quiet-note">
        Trash: use Finder “Put Back”. Quarantine: restore in Tiny. Quarantine
        items are retained for at least 30 days.
      </div>
      <CommandChip command={preview.equivalentCommand} />
      <p className="caption">
        This CLI command opens an equivalent category workflow; the GUI uses
        only the reviewed paths above.
      </p>
      <ErrorNotice error={error} />
      {!desktop && (
        <div className="demo-note">
          Browser demo · file changes are available in the macOS app.
        </div>
      )}
      <div className="dialog-actions">
        <button
          className="button secondary"
          disabled={pending}
          onClick={onClose}
        >
          <ArrowLeft size={16} />
          Keep reviewing
        </button>
        <button
          className="button primary"
          disabled={pending || !desktop}
          onClick={onConfirm}
        >
          {pending ? "Making room…" : "Confirm & move safely"}
        </button>
      </div>
    </dialog>
  );
}
