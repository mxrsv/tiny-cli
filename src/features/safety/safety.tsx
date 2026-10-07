import { ArchiveRestore, FolderOpen, ShieldCheck, Trash2 } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ipc, desktop } from "../../lib/ipc";
import { formatBytes, relativeTime } from "../../lib/format";
import { ErrorNotice } from "../../components/error-notice";
import { EmptyState } from "../../components/empty-state";
export function Safety() {
  const client = useQueryClient();
  const entries = useQuery({ queryKey: ["safety", "undo"], queryFn: ipc.undo });
  const restore = useMutation({
    mutationFn: ipc.restore,
    onSuccess: () => {
      void client.invalidateQueries({ queryKey: ["safety"] });
      void client.invalidateQueries({ queryKey: ["system", "monitor"] });
    },
  });
  const trash = useMutation({ mutationFn: ipc.openTrash });
  return (
    <div className="page">
      <div className="page-heading">
        <div>
          <div className="eyebrow">THERE’S A WAY BACK</div>
          <h1>A little peace of mind.</h1>
          <p>A record of every move, and a clear path to recovery.</p>
        </div>
        <button
          className="button secondary"
          disabled={trash.isPending || !desktop}
          onClick={() => trash.mutate()}
        >
          <FolderOpen size={17} />
          Open Trash
        </button>
      </div>
      <ErrorNotice error={entries.error || restore.error || trash.error} />
      <div className="two-column">
        <section className="insight-card">
          <Trash2 size={24} />
          <span className="eyebrow">YOUR MAC’S TRASH</span>
          <h3>Familiar, recoverable.</h3>
          <p>
            Tiny tries macOS Trash first. Open Trash in Finder, select a file
            and choose <b>Put Back</b>. Finder controls Trash retention.
          </p>
        </section>
        <section className="insight-card">
          <ArchiveRestore size={24} />
          <span className="eyebrow">TINY’S QUARANTINE</span>
          <h3>Kept close. Easily restored.</h3>
          <p>
            If Trash refuses a file, Tiny moves it to its own quarantine on the
            same volume. Restore it here. Files are retained for at least 30
            days; this version does not purge automatically.
          </p>
        </section>
      </div>
      <div className="section-heading">
        <h2>Your undo history</h2>
        <span className="caption">Most recent 500 records</span>
      </div>
      {entries.isPending ? (
        <div className="quiet-note">Loading your history…</div>
      ) : !entries.data?.length ? (
        <EmptyState title="Nothing to undo. Yet.">
          Every cleanup will leave a record here. You’re always in control.
        </EmptyState>
      ) : (
        <div className="undo-list">
          {entries.data.map((entry) => (
            <div className="undo-entry" key={entry.id}>
              <div className="opportunity-icon">
                {entry.method === "trash" ? (
                  <Trash2 size={18} />
                ) : (
                  <ArchiveRestore size={18} />
                )}
              </div>
              <div>
                <b title={entry.originalPath}>{entry.originalPath}</b>
                <span>
                  {formatBytes(entry.bytes)} · {relativeTime(entry.createdAt)} ·{" "}
                  {entry.method} · {entry.status}
                </span>
                {entry.error && (
                  <small className="text-error">{entry.error}</small>
                )}
              </div>
              {entry.method === "quarantine" && entry.status === "moved" ? (
                <button
                  className="button secondary"
                  disabled={restore.isPending}
                  onClick={() => restore.mutate(entry.id)}
                >
                  Restore
                </button>
              ) : entry.method === "trash" && entry.status === "moved" ? (
                <button className="text-button" onClick={() => trash.mutate()}>
                  Put Back in Finder
                </button>
              ) : entry.status === "pending" ? (
                <button
                  className="text-button"
                  disabled={restore.isPending}
                  onClick={() => restore.mutate(entry.id)}
                >
                  Recover interrupted move
                </button>
              ) : (
                <span className="pill subtle">{entry.status}</span>
              )}
            </div>
          ))}
        </div>
      )}
      <div className="quiet-note">
        <ShieldCheck size={18} />
        Restore never overwrites an existing file. Move a conflicting file
        aside, then try again.
      </div>
    </div>
  );
}
