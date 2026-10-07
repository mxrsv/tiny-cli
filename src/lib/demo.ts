import type {
  CategoryGroup,
  MonitorReport,
  SmartScan,
  SpaceReport,
  QuarantineEntry,
  SpaceNode,
} from "../types/ipc";
const GB = 1024 ** 3;
function group(
  id: string,
  label: string,
  risk: CategoryGroup["risk"],
  paths: [string, number][],
): CategoryGroup {
  return {
    id,
    label,
    risk,
    totalSize: paths.reduce((total, [, size]) => total + size, 0),
    items: paths.map(([path, size]) => ({
      categoryId: id,
      categoryLabel: label,
      risk,
      path,
      size,
    })),
  };
}
export const demoScan: SmartScan = {
  id: "browser-demo",
  createdAt: new Date().toISOString(),
  system: {
    os: "macOS",
    host: "Your Mac",
    uptimeSeconds: 146800,
    cpuCount: 10,
    cpuModel: "Apple silicon",
    cpuUsage: 12,
    memoryTotal: 16 * GB,
    memoryUsed: 6.9 * GB,
    disks: [
      {
        name: "Macintosh HD",
        mountPoint: "/",
        totalBytes: 512 * GB,
        availableBytes: 144 * GB,
      },
    ],
  },
  discovery: {
    skippedRunning: [["browser-caches", "Safari"]],
    groups: [
      group("xcode-derived", "Xcode build files", "safe", [
        ["/Users/demo/Library/Developer/Xcode/DerivedData", 4.3 * GB],
      ]),
      group("user-logs", "Application logs", "safe", [
        ["/Users/demo/Library/Logs/DiagnosticReports", 0.9 * GB],
        ["/Users/demo/Library/Logs/Adobe", 0.3 * GB],
      ]),
      group("cargo", "Rust package cache", "review", [
        ["/Users/demo/.cargo/registry/cache", 2.1 * GB],
      ]),
      group("downloads-old", "Older downloads", "review", [
        ["/Users/demo/Downloads/Archive.zip", 1.4 * GB],
      ]),
      group("trash", "Trash", "destructive", [
        ["/Users/demo/.Trash", 2.8 * GB],
      ]),
    ],
  },
  health: {
    score: 82,
    reclaimableBytes: 9 * GB,
    factors: [
      {
        label: "Free disk space",
        penalty: 0,
        explanation: "28.1% free. Storage has comfortable headroom.",
      },
      {
        label: "Memory use",
        penalty: 0,
        explanation: "43.1% used. A pressure indicator, including OS caches.",
      },
      {
        label: "Cleanup opportunities",
        penalty: 18,
        explanation:
          "2 points per GiB of recoverable candidates, capped at 20.",
      },
    ],
  },
};
export const demoMonitor: MonitorReport = {
  system: demoScan.system,
  weeklyDeltaBytes: 6 * GB,
  daysUntilFull: 168,
  history: [0, 1, 2, 3, 4, 5, 6].map((day) => ({
    id: `demo-${day}`,
    createdAt: new Date(Date.now() - day * 86400000).toISOString(),
    usedBytes: (368 - day + Math.sin(day) * 0.6) * GB,
    reclaimableBytes: 9 * GB,
    healthScore: 82,
  })),
};
function leaf(name: string, bytes: number, ageSeconds: number): SpaceNode {
  return {
    name,
    path: `/Users/demo/${name}`,
    bytes,
    allocatedBytes: bytes,
    ageSeconds,
    isDirectory: false,
    children: [],
  };
}
function folder(name: string, children: SpaceNode[]): SpaceNode {
  return {
    name,
    path: `/Users/demo/${name}`,
    bytes: children.reduce((a, b) => a + b.bytes, 0),
    allocatedBytes: children.reduce((a, b) => a + b.bytes, 0),
    ageSeconds: 0,
    isDirectory: true,
    children,
  };
}
export const demoSpace: SpaceReport = {
  filesScanned: 12483,
  skippedPaths: [],
  truncated: false,
  root: folder("demo", [
    folder("Documents", [
      leaf("Design assets.zip", 24 * GB, 90 * 86400),
      leaf("Projects", 35 * GB, 7 * 86400),
    ]),
    folder("Library", [
      leaf("Application Support", 37 * GB, 14 * 86400),
      leaf("Caches", 18 * GB, 3 * 86400),
    ]),
    folder("Downloads", [
      leaf("Footage.mov", 26 * GB, 60 * 86400),
      leaf("Archive.zip", 12 * GB, 120 * 86400),
    ]),
    folder("Pictures", [leaf("Photo library", 32 * GB, 86400)]),
    folder("Desktop", [leaf("Work in progress", 8 * GB, 2 * 86400)]),
  ]),
};
export const demoUndo: QuarantineEntry[] = [];
