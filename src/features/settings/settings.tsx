import { useEffect, useState } from "react";
import { Bell, CalendarClock, Check, Plus, Trash2 } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ipc, desktop } from "../../lib/ipc";
import type { Settings as Preferences } from "../../types/ipc";
import { useLatestScan, usePermissions } from "../../hooks/use-engine";
import { ErrorNotice } from "../../components/error-notice";
import { formatBytes } from "../../lib/format";
export function Settings() {
  const client = useQueryClient();
  const settings = useQuery({ queryKey: ["settings"], queryFn: ipc.settings });
  const rules = useQuery({ queryKey: ["rules"], queryFn: ipc.rules });
  const { data: scan } = useLatestScan();
  const permission = usePermissions();
  const [draft, setDraft] = useState<Preferences | null>(null);
  const [category, setCategory] = useState("user-logs");
  const [threshold, setThreshold] = useState(1);
  useEffect(() => {
    if (settings.data) setDraft({ ...settings.data });
  }, [settings.data]);
  const save = useMutation({
    mutationFn: ipc.saveSettings,
    onSuccess: (value) => client.setQueryData(["settings"], value),
  });
  const addRule = useMutation({
    mutationFn: () =>
      ipc.saveRule({
        id: crypto.randomUUID(),
        categoryId: category,
        minBytes: Math.round(threshold * 1024 ** 3),
        enabled: true,
      }),
    onSuccess: () => {
      void client.invalidateQueries({ queryKey: ["rules"] });
    },
  });
  const removeRule = useMutation({
    mutationFn: ipc.deleteRule,
    onSuccess: () => {
      void client.invalidateQueries({ queryKey: ["rules"] });
    },
  });
  const openPermissions = useMutation({ mutationFn: ipc.openPermissions });
  return (
    <div className="page settings-page">
      <div className="page-heading">
        <div>
          <div className="eyebrow">AT YOUR PACE</div>
          <h1>Small preferences.</h1>
          <p>Make Tiny fit the way you work.</p>
        </div>
      </div>
      <ErrorNotice
        error={
          settings.error ||
          save.error ||
          rules.error ||
          addRule.error ||
          removeRule.error ||
          openPermissions.error
        }
      />
      {draft && (
        <section className="settings-card">
          <div className="card-topline">
            <h2>
              <CalendarClock size={19} /> Background care
            </h2>
            <span className="pill subtle">While Tiny is running</span>
          </div>
          <div className="setting-row">
            <div>
              <b>Scheduled Smart Scan</b>
              <p>Runs in the background while the app lives in your menubar.</p>
            </div>
            <select
              aria-label="Scan schedule"
              value={draft.scanIntervalHours}
              onChange={(e) =>
                setDraft({
                  ...draft,
                  scanIntervalHours: Number(e.target.value),
                })
              }
            >
              <option value={0}>Off</option>
              <option value={24}>Every day</option>
              <option value={168}>Every week</option>
              <option value={720}>Every 30 days</option>
            </select>
          </div>
          <div className="setting-row">
            <div>
              <b>Project idle threshold</b>
              <p>
                Only suggest age-filtered project caches after this many days.
              </p>
            </div>
            <label className="number-input">
              <input
                type="number"
                min="1"
                max="36500"
                aria-label="Idle days"
                value={draft.idleDays}
                onChange={(e) =>
                  setDraft({ ...draft, idleDays: Number(e.target.value) })
                }
              />
              days
            </label>
          </div>
          {(
            [
              {
                key: "notifications",
                label: "Gentle notifications",
                description:
                  "Notify when a rule matches or the health score falls below 60.",
              },
              {
                key: "watchChanges",
                label: "Watch for changes",
                description:
                  "Watch common folders and scan at most once every five minutes.",
              },
              {
                key: "launchAtLogin",
                label: "Launch at login",
                description: "Keep Tiny close by when you start your Mac.",
              },
            ] as const
          ).map(({ key, label, description }) => (
            <div className="setting-row" key={key}>
              <div>
                <b>{label}</b>
                <p>{description}</p>
              </div>
              <input
                className="switch"
                type="checkbox"
                aria-label={label}
                checked={draft[key]}
                onChange={(e) =>
                  setDraft({ ...draft, [key]: e.target.checked })
                }
              />
            </div>
          ))}
          <div className="settings-save">
            {save.isSuccess && (
              <span className="success-text">
                <Check size={15} />
                Preferences saved
              </span>
            )}
            <button
              className="button primary"
              disabled={save.isPending || !desktop || draft.idleDays < 1}
              onClick={() => save.mutate(draft)}
            >
              Save preferences
            </button>
          </div>
        </section>
      )}
      <section className="settings-card">
        <div className="card-topline">
          <h2>
            <Bell size={19} /> A helpful nudge
          </h2>
          <span className="caption">
            Rules notify you. They never delete files.
          </span>
        </div>
        <p className="muted">
          Ask Tiny to let you know when a category passes a size threshold after
          a background scan.
        </p>
        <form
          className="rule-form"
          onSubmit={(e) => {
            e.preventDefault();
            addRule.mutate();
          }}
        >
          <select
            aria-label="Rule category"
            value={category}
            onChange={(e) => setCategory(e.target.value)}
          >
            {[
              ...new Map(
                [
                  { id: "user-logs", label: "Application logs" },
                  ...(scan?.discovery.groups ?? []),
                ].map((g) => [g.id, g]),
              ).values(),
            ].map((g) => (
              <option value={g.id} key={g.id}>
                {g.label}
              </option>
            ))}
          </select>
          <span>is larger than</span>
          <label className="number-input">
            <input
              type="number"
              aria-label="Rule threshold in GB"
              min=".01"
              step=".1"
              value={threshold}
              onChange={(e) => setThreshold(Number(e.target.value))}
            />
            GB
          </label>
          <button
            className="button secondary"
            disabled={addRule.isPending || !desktop || threshold <= 0}
          >
            <Plus size={16} />
            Add rule
          </button>
        </form>
        {rules.data?.map((rule) => (
          <div className="setting-row" key={rule.id}>
            <span>
              {rule.categoryId} &gt; {formatBytes(rule.minBytes)}
            </span>
            <button
              className="icon-button"
              aria-label={`Delete rule for ${rule.categoryId}`}
              disabled={removeRule.isPending}
              onClick={() => removeRule.mutate(rule.id)}
            >
              <Trash2 size={16} />
            </button>
          </div>
        ))}
      </section>
      <section className="settings-card">
        <h2>Full Disk Access</h2>
        <p>{permission.data?.explanation ?? "Checking macOS permissions…"}</p>
        <button
          className="button secondary"
          disabled={!desktop}
          onClick={() => openPermissions.mutate()}
        >
          Open privacy settings
        </button>
      </section>
    </div>
  );
}
