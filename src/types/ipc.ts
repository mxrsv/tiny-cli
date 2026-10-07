export type Risk = "safe" | "review" | "destructive";
export interface Progress {
  operation: string;
  completed: number;
  total: number | null;
  message: string;
}
export interface DiskInfo {
  name: string;
  mountPoint: string;
  totalBytes: number;
  availableBytes: number;
}
export interface SystemInfo {
  os: string | null;
  host: string | null;
  uptimeSeconds: number | null;
  cpuCount: number;
  cpuModel: string;
  cpuUsage: number;
  memoryTotal: number;
  memoryUsed: number;
  disks: DiskInfo[];
}
export interface CleanItem {
  categoryId: string;
  categoryLabel: string;
  path: string;
  size: number;
  risk: Risk;
}
export interface CategoryGroup {
  id: string;
  label: string;
  risk: Risk;
  items: CleanItem[];
  totalSize: number;
}
export interface HealthFactor {
  label: string;
  penalty: number;
  explanation: string;
}
export interface SmartScan {
  id: string;
  createdAt: string;
  system: SystemInfo;
  discovery: { groups: CategoryGroup[]; skippedRunning: [string, string][] };
  health: { score: number; reclaimableBytes: number; factors: HealthFactor[] };
}
export interface ScanSummary {
  id: string;
  createdAt: string;
  usedBytes: number;
  reclaimableBytes: number;
  healthScore: number;
}
export interface MonitorReport {
  system: SystemInfo;
  history: ScanSummary[];
  weeklyDeltaBytes: number | null;
  daysUntilFull: number | null;
}
export interface PreviewItem {
  categoryId: string;
  categoryLabel: string;
  path: string;
  bytes: number;
  risk: Risk;
}
export interface CleanupPreview {
  id: string;
  items: PreviewItem[];
  totalBytes: number;
  equivalentCommand: string;
  expiresAt: string;
}
export interface CleanupResult {
  operationId: string;
  movedCount: number;
  movedBytes: number;
  failed: [string, string][];
}
export interface QuarantineEntry {
  id: string;
  operationId: string;
  originalPath: string;
  storedPath: string | null;
  categoryId: string;
  bytes: number;
  createdAt: string;
  expiresAt: string;
  method: string;
  status: string;
  error: string | null;
}
export interface SpaceNode {
  name: string;
  path: string;
  bytes: number;
  allocatedBytes: number;
  ageSeconds: number;
  isDirectory: boolean;
  children: SpaceNode[];
}
export interface SpaceReport {
  root: SpaceNode;
  filesScanned: number;
  skippedPaths: string[];
  truncated: boolean;
}
export interface PermissionStatus {
  status: "required" | "unknown" | "granted";
  homePath: string;
  explanation: string;
}
export interface Settings {
  scanIntervalHours: number;
  notifications: boolean;
  launchAtLogin: boolean;
  watchChanges: boolean;
  idleDays: number;
}
export interface Rule {
  id: string;
  categoryId: string;
  minBytes: number;
  enabled: boolean;
}
export interface ErrorPayload {
  code: string;
  message: string;
}
