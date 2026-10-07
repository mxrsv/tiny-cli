import { invoke, isTauri } from "@tauri-apps/api/core";
import type {
  CleanupPreview,
  CleanupResult,
  MonitorReport,
  PermissionStatus,
  QuarantineEntry,
  Rule,
  Settings,
  SmartScan,
  SpaceReport,
} from "../types/ipc";
import { demoMonitor, demoScan, demoSpace, demoUndo } from "./demo";
export const desktop = isTauri();
const nativeOnly = () =>
  Promise.reject({
    code: "desktop_required",
    message:
      "Open the macOS app to change local files or settings. Browser demo is read-only.",
  });
export const ipc = {
  latestScan: (): Promise<SmartScan | null> =>
    desktop ? invoke("get_latest_scan") : Promise.resolve(demoScan),
  smartScan: (): Promise<SmartScan> =>
    desktop
      ? invoke("run_smart_scan")
      : Promise.resolve({ ...demoScan, createdAt: new Date().toISOString() }),
  monitor: (): Promise<MonitorReport> =>
    desktop ? invoke("get_monitor") : Promise.resolve(demoMonitor),
  permissions: (): Promise<PermissionStatus> =>
    desktop
      ? invoke("get_permissions")
      : Promise.resolve({
          status: "granted",
          homePath: "/Users/demo",
          explanation:
            "Read-only browser demo. No files on your computer are accessed.",
        }),
  openPermissions: (): Promise<void> =>
    desktop ? invoke("open_permission_settings") : nativeOnly(),
  preview: (
    scanId: string,
    selectedPaths: string[],
  ): Promise<CleanupPreview> => {
    if (desktop) return invoke("preview_cleanup", { scanId, selectedPaths });
    const items = demoScan.discovery.groups
      .flatMap((g) => g.items)
      .filter((i) => selectedPaths.includes(i.path))
      .map((i) => ({
        categoryId: i.categoryId,
        categoryLabel: i.categoryLabel,
        path: i.path,
        bytes: i.size,
        risk: i.risk,
      }));
    const ids = [...new Set(items.map((i) => i.categoryId))];
    return Promise.resolve({
      id: "demo-preview",
      totalBytes: items.reduce((a, i) => a + i.bytes, 0),
      items,
      equivalentCommand: `tiny clean ${ids.map((id) => `--category ${id}`).join(" ")} --review-paths --idle-days 30`,
      expiresAt: new Date(Date.now() + 600000).toISOString(),
    });
  },
  clean: (previewId: string, confirmed: boolean): Promise<CleanupResult> =>
    desktop
      ? invoke("execute_cleanup", { previewId, confirmed })
      : nativeOnly(),
  spaceLens: (path: string): Promise<SpaceReport> =>
    desktop ? invoke("run_space_lens", { path }) : Promise.resolve(demoSpace),
  undo: (): Promise<QuarantineEntry[]> =>
    desktop ? invoke("get_undo_entries") : Promise.resolve(demoUndo),
  restore: (entryId: string): Promise<void> =>
    desktop ? invoke("restore_entry", { entryId }) : nativeOnly(),
  openTrash: (): Promise<void> =>
    desktop ? invoke("open_trash") : nativeOnly(),
  settings: (): Promise<Settings> =>
    desktop
      ? invoke("get_settings")
      : Promise.resolve({
          scanIntervalHours: 168,
          notifications: true,
          launchAtLogin: false,
          watchChanges: false,
          idleDays: 30,
        }),
  saveSettings: (settings: Settings): Promise<Settings> =>
    desktop ? invoke("save_settings", { settings }) : nativeOnly(),
  rules: (): Promise<Rule[]> =>
    desktop ? invoke("get_rules") : Promise.resolve([]),
  saveRule: (rule: Rule): Promise<void> =>
    desktop ? invoke("save_rule", { rule }) : nativeOnly(),
  deleteRule: (ruleId: string): Promise<void> =>
    desktop ? invoke("delete_rule", { ruleId }) : nativeOnly(),
};
