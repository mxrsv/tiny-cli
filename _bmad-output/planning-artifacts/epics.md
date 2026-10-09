---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - "_bmad-output/planning-artifacts/prd.md"
  - "_bmad-output/planning-artifacts/architecture.md"
project_name: "tiny-cli"
user_name: "Kyantran"
date: "2026-05-21"
status: "complete"
completedAt: "2026-05-22"
epicCount: 10
mvpEpics: 5
postMvpEpics: 5
storyCount: 93
---

# tiny-cli - Epic Breakdown

> **Scope superseded on 2026-10-06:** This breakdown describes the previous
> Smart Scan-first roadmap. The user now prioritizes Processes + Clean.
> Use the [current PRD scope](prd.md#product-scope) and
> [MVP execution plan](../../docs/plans/2026-10-06-processes-clean-mvp.md)
> for the active release; story completion and readiness are not revalidated
> by this scope change. Keep the older stories as future references.

## Overview

This document provides the complete epic and story breakdown for tiny-cli, decomposing the requirements from the PRD, UX Design language (embedded in PRD), and Architecture decisions into implementable stories.

## Requirements Inventory

### Functional Requirements

#### Cleanup & Categories

- **FR1**: User can trigger a scan of all 21+ cleanup categories with a single action and see total reclaimable disk space. `[P1]`
- **FR2**: User can view cleanup categories grouped into 3 safety lanes (🟢 Safe / 🟡 Review / 🔴 Destructive), with 🟢 ticked by default and 🟡/🔴 requiring explicit opt-in via a flag. `[P1]`
- **FR3**: User can drill into any category to inspect every file targeted for deletion, with size, path, and last-access timestamp shown per file. `[P1]`
- **FR4**: User can selectively include or exclude individual files within a category before executing cleanup. `[P1]`
- **FR5**: User can execute cleanup on selected categories and receive an end-of-run report showing freed space, files moved to quarantine, files sent to Trash, and files skipped (with reason). `[P1]`
- **FR6**: Contributor can add a new cleanup category by implementing the `CleanProvider` trait and registering at three registry sites, with a registry-sync test enforcing all sites are kept in sync. `[P1, ongoing]`
- **FR7**: User can clean up app leftovers (caches, prefs, launch agents, containers) for any installed app via the `uninstall` flow. `[P1, existing]`
- **FR8**: User can discover and clean duplicate files across the home directory by content hash. `[P1, existing via CLI]`

#### Safety Net & Undo

- **FR9**: System sends every deleted file either to macOS Trash (Finder Put-Back) or to a `~/.tiny/quarantine/` retention area, never directly to permanent deletion, unless the user explicitly invokes a hard-delete flag. `[P1]`
- **FR10**: User can restore any file from quarantine within 30 days via an in-app "Cleanup History" panel, organized per cleanup session. `[P1]`
- **FR11**: User can undo the most recent cleanup session entirely via Cmd+Z keyboard shortcut, restoring all files from that session's quarantine batch. `[P1]`
- **FR12**: System auto-purges quarantine entries older than 30 days via a background task and notifies user when quarantine size exceeds 10 GB. `[P1]`
- **FR13**: System refuses to delete files via symlink traversal — every filesystem operation must use `symlink_metadata` and canonicalize paths against per-provider whitelist before deletion. `[P1, architectural invariant]`
- **FR14**: System detects when an app is currently running before cleaning its caches, skips those caches, and surfaces the skip reason in the cleanup report. `[P1, existing in engine]`
- **FR15**: User can lock specific folders (e.g., Desktop, Documents) so no provider, regardless of safety lane, can touch them; lock is settable per-folder with optional password protection. `[P2]`

#### Smart Scan & Health Score

- **FR16**: User can trigger a "Smart Scan" that runs `sys` + `scan` + `clean --dry-run` in sequence and produces a single Health Score (0–100) plus a one-line summary of reclaimable space. `[P1]`
- **FR17**: User can view the breakdown of inputs to the Health Score (disk pressure, junk count, memory pressure, quarantine age) and see explanations of each component's contribution. `[P1]`
- **FR18**: User can schedule Smart Scan to run automatically at a chosen interval (weekly/daily/never) with results delivered via system notification. `[P3]`

#### Space Lens (Disk Visualization)

- **FR19**: User can view an interactive treemap representing all files and folders in their home directory or full volume, with tile size proportional to disk usage. `[P2]`
- **FR20**: User can drill down into the treemap via double-click and navigate back via breadcrumb or ⌫ key. `[P2]`
- **FR21**: User can toggle between treemap and sunburst visualization for the same dataset. `[P2]`
- **FR22**: User can apply color heatmaps to the visualization by file age, file type, or both. `[P2]`
- **FR23**: User can right-click any tile to reveal in Finder, Quick Look, send to Trash, send to quarantine, or invoke uninstall (for `.app` bundles). `[P2]`
- **FR24**: User can exclude system, cloud-synced, or pinned folders from Space Lens calculations. `[P2]`
- **FR25**: System detects and explains "phantom space" — APFS purgeable space, snapshots, and hidden system files that account for missing disk capacity. `[P2]`
- **FR26**: System uses live filesystem events (FSEvents) to re-scan only changed folders and update Space Lens incrementally, instead of re-scanning the entire volume. `[P2]`
- **FR27**: User can export Space Lens results to PNG image or JSON report. `[P2]`

#### Active Monitoring & Explanation

- **FR28**: User can install a menubar widget displaying live CPU, RAM, and disk usage with sparkline history. `[P3]`
- **FR29**: User can click the menubar widget to open a mini-panel with current Health Score and a "Run Smart Scan" button. `[P3]`
- **FR30**: User can run the app in "menubar-only" mode, hiding the Dock icon entirely. `[P3]`
- **FR31**: System captures periodic disk-usage snapshots and stores them locally so user can ask "what changed this week?" and see which folders grew, by how much. `[P3]`
- **FR32**: System forecasts disk-full timeline based on usage trend and notifies user proactively when forecast falls below a configurable threshold. `[P3]`
- **FR33**: System detects "newly large files" in Downloads/Desktop/Documents via FSEvents watch and badges the app icon when a file above a configurable size threshold appears. `[P3]`
- **FR34**: User can view a "Heavy Consumers" panel listing processes with highest CPU/RAM usage and quit any process directly from the panel. `[P3]`

#### Focus Integration

- **FR35**: User can enable a "cleanup during focus" toggle in the focus timer settings, causing cleanup of safe-lane categories to run in the background during each Pomodoro session. `[P4]`
- **FR36**: System reports cleanup results to user as a focus-session-end notification, showing freed space and never interrupting the focus session itself. `[P4]`
- **FR37**: User can choose which categories run during focus sessions, with default restricted to 🟢 Safe lane only. `[P4]`

#### CLI Parity & Transparency

- **FR38**: Every action available in the GUI has an equivalent `tiny ...` CLI command that produces the same effect when run independently. `[P1, architectural invariant]`
- **FR39**: Every GUI action displays its equivalent CLI command in a footer/info area with a one-click "Copy" button. `[P1]`
- **FR40**: User can toggle the CLI footer visibility in app settings (default ON for desktop app; per-user override). `[P1]`
- **FR41**: All CLI commands support `--json` output, returning the same data structure the GUI consumes via Tauri commands. `[P1]`
- **FR42**: All long-running CLI commands support a progress callback that emits structured updates (percent + current item), consumed by GUI via Tauri events. `[P1]`

#### Maintenance & System Optimization

- **FR43**: User can free up RAM (purge inactive memory) with a single action and see RAM usage before/after. `[P5]`
- **FR44**: User can run macOS periodic maintenance scripts (flush DNS, reindex Spotlight, repair disk permissions, rebuild Launch Services) individually or as a batch, each as a separate toggle. `[P5]`
- **FR45**: User can view and disable Login Items and Launch Agents via a unified management panel. `[P5]`

#### Protection (Malware, Privacy, Updater)

- **FR46**: System can scan for known PUP/adware signatures and quarantine matches. `[P5]`
- **FR47**: User can clear browser history, cookies, recent items, and saved Wi-Fi networks via a privacy cleaner module. `[P5]`
- **FR48**: System can detect outdated versions of apps installed outside the Mac App Store and offer to update them. `[P5]`
- **FR49**: User can audit which apps hold sensitive permissions (camera, microphone, Full Disk Access, Accessibility) and revoke selectively. `[P5]`

#### Onboarding & Permissions

- **FR50**: System detects on launch whether Full Disk Access is granted and, if missing, shows an onboarding flow with step-by-step instructions to grant it via System Settings. `[P1]`
- **FR51**: System detects which categories require additional permissions (e.g., Accessibility for Login Items) and prompts only when those categories are first used. `[P1, P5]`

#### Settings & Configuration

- **FR52**: User can configure quarantine retention period (default 30 days, range 7–90 days). `[P1]`
- **FR53**: User can configure auto-update behavior independently for check / download / install. `[P1]`
- **FR54**: User can opt in to anonymous crash reports (default OFF) with a settings toggle that requires explicit confirmation. `[P1]`
- **FR55**: User can clear local journal (cleanup history) and quarantine entirely from settings. `[P1]`
- **FR56**: User can define custom cleanup rules in a `~/.config/tiny/rules.toml` file (e.g., "`.dmg` in Downloads > 7 days → auto-delete") that the engine applies on schedule. `[Vision]`

#### Distribution & Updates

- **FR57**: User can install the app via Homebrew Cask (`brew install --cask tiny`) with auto-update via `brew upgrade`. `[P1]`
- **FR58**: User can download a signed and notarized `.dmg` from GitHub Releases with checksums and signatures published. `[P1]`
- **FR59**: System checks for updates via GitHub Releases API on launch (max once per 24 h, configurable) and prompts user to download and install. `[P1]`

#### Accessibility

- **FR60**: User can navigate the entire app via keyboard only, with visible focus indicators on every interactive element. `[P1]`
- **FR61**: User can scale the app font size in settings and the layout remains usable at 200% scaling. `[P1]`
- **FR62**: System maintains contrast ratio at WCAG AA level (4.5:1 for normal text, 3:1 for large text) across all themes. `[P1]`
- **FR63**: All destructive actions present a confirmation dialog with large-readable text and two clearly labeled buttons. `[P1]`

#### Open-Source & Community

- **FR64**: Source code is publicly available on GitHub under a permissive open-source license (MIT or Apache 2.0). `[P1]`
- **FR65**: Contributor can read `CONTRIBUTING.md` and successfully submit a working PR (such as adding a new cleanup category) without needing one-on-one mentorship from the maintainer. `[P1]`
- **FR66**: Project uses Conventional Commits in Vietnamese (`<type>(<scope>): <mô tả>`) for all commits, enforced via commitlint or pre-commit hook. `[P1, existing convention]`

### NonFunctional Requirements

#### Performance

- **NFR-P1**: App cold start ≤ 800 ms P50, ≤ 1.5 s P95, hard cap 3 s (M2 baseline).
- **NFR-P2**: `tiny sys` snapshot ≤ 150 ms P50, ≤ 200 ms P95.
- **NFR-P3**: `tiny scan` 3 folder mặc định (~50k files) ≤ 2 s P50, ≤ 3 s P95.
- **NFR-P4**: Smart Scan end-to-end ≤ 4 s P50, ≤ 5 s P95, hard cap 10 s.
- **NFR-P5**: `clean --dry-run` 1 category ≤ 500 ms P50, ≤ 1 s P95.
- **NFR-P6**: `clean` execute 1 category (hybrid delete) ≤ 2 s P50, ≤ 5 s P95.
- **NFR-P7**: GUI 60 fps — interaction render ≤ 16.6 ms P95, không drop frame trong scan progress animation.
- **NFR-P8**: User-initiated cancel responsive ≤ 500 ms P50, ≤ 1 s P95.
- **NFR-P9** (v1.1): `scan` toàn ổ (~500k files) ≤ 30 s P50, ≤ 60 s P95, parallel walker bắt buộc.
- **NFR-P10** (v1.1): Treemap render initial 500k files ≤ 1 s P95 (virtualization).
- **NFR-P11**: Memory idle ≤ 150 MB RSS, peak scan ≤ 500 MB RSS, hard cap 1 GB.
- **NFR-P12**: CPU idle ≤ 1% trung bình 1 phút trên M2.
- **NFR-P13** (v1.2): Menubar widget refresh 1 s, CPU cost ≤ 0.5%.
- **Test gate**: bench-suite chạy CI mỗi PR, fail nếu regress > 20%.

#### Security

- **NFR-S1**: 0 instance `std::fs::remove_*` production (trừ `fs_safe.rs`). CI grep gate enforce.
- **NFR-S2**: 100% xoá filesystem qua `fs_safe::remove_recursive_safe` hoặc hybrid delete layer.
- **NFR-S3**: 100% xoá dùng `symlink_metadata` + canonicalize path, có test ghim mọi provider.
- **NFR-S4**: Provider registry sync 100% — `category_family()` panic nếu thiếu id, test ghim.
- **NFR-S5**: `panic!`/`unwrap()`/`expect()` chỉ ở `#[cfg(test)]` hoặc invariant đã test ghim. CI grep gate.
- **NFR-S6**: Mọi Tauri command trả `Result<T, ErrorPayload>`, không panic qua IPC boundary.
- **NFR-S7**: Release artifact signed Apple Developer ID + notarized qua `notarytool`.
- **NFR-S8**: GitHub Releases publish kèm SHA256 checksum + GPG signature.
- **NFR-S9**: `cargo audit` chạy mỗi PR, fail nếu CVE severity ≥ Medium.
- **NFR-S10**: `npm audit` chạy mỗi PR, fail nếu vulnerability severity ≥ High production deps.
- **NFR-S11**: Update mechanism verify signature `.dmg` bằng public key embed trước khi prompt install.

#### Privacy

- **NFR-Pr1**: Zero outbound network call mặc định cho core feature.
- **NFR-Pr2**: "Send anonymous crash report" default OFF, bật yêu cầu explicit confirm.
- **NFR-Pr3**: Crash report (opt-in) không chứa file path/hostname/user name/IP.
- **NFR-Pr4**: Update check là HTTP request duy nhất mặc định, có thể tắt qua settings.
- **NFR-Pr5**: CLI command history không log argument, chỉ command name + UTC timestamp.
- **NFR-Pr6**: Cleanup journal SQLite local-only ở `~/.tiny/journal.sqlite`, user xoá được.

#### Reliability

- **NFR-R1**: Crash-free session rate ≥ 99.5% (đo qua opt-in crash report).
- **NFR-R2**: Transactional quarantine — journal entry ghi TRƯỚC khi move file. Recovery khi crash.
- **NFR-R3**: Quarantine auto-purge sau 30 ngày, retry 3 lần, không block app.
- **NFR-R4**: Long-running ops support cancel mid-flight không corrupt state.
- **NFR-R5**: Free disk check trước quarantine move — nếu < 2× target size, fail-fast.
- **NFR-R6**: App restart giữ context (selected category, last scan result hash).

#### Accessibility

- **NFR-A1**: Full keyboard navigation, visible focus indicator (≥ 2px outline contrast ≥ 3:1).
- **NFR-A2**: Contrast WCAG AA: text ≥ 4.5:1 (small), ≥ 3:1 (large). Test gate axe-core Vitest e2e.
- **NFR-A3**: Font scaling responsive — UI usable tại system font scale 200%.
- **NFR-A4**: VoiceOver compatible — ARIA label, landmark regions đúng.
- **NFR-A5**: Color không phải means duy nhất — làn 3 màu kèm icon + label text.
- **NFR-A6**: Confirmation dialog destructive action — chữ ≥ 14pt, button label cụ thể.
- **NFR-A7**: Reduced Motion respect — nếu macOS `Reduce Motion` bật, tắt animation > 200ms.

#### Integration

- **NFR-I1**: Tauri IPC contract — mọi struct boundary `#[serde(rename_all = "camelCase")]`, error là `ErrorPayload`.
- **NFR-I2**: Homebrew Cask formula auto-update khi GitHub Release mới publish.
- **NFR-I3**: GitHub Releases API rate-limit aware — tôn trọng `X-RateLimit-Remaining`, exponential backoff.
- **NFR-I4**: CLI command stable contract — breaking change = major version bump, JSON output schema versioned.
- **NFR-I5** (v1.5+): App updater module integration với Homebrew + Sparkle feed.

#### Maintainability

- **NFR-M1**: Test coverage `tiny-core` ≥ 80% line, CLI integration test cover happy path, frontend Vitest ≥ 70%.
- **NFR-M2**: File size discipline — trung bình ≤ 400 dòng, hard cap 800, CI warn > 600.
- **NFR-M3**: Production deps `tiny-core` ≤ 15 crate, CLI ≤ 20, frontend ≤ 30 npm package.
- **NFR-M4**: Public API `tiny-core` documented qua `///` rustdoc, `cargo doc --no-deps` 0 warning.
- **NFR-M5**: Provider authoring — 1 file mới + 3 line registry + 1 file test, ≤ 1 giờ contributor mới.
- **NFR-M6**: Local dev setup `git clone && cargo run -p tiny -- scan` ≤ 5 phút fresh macOS.
- **NFR-M7**: GUI dev `npm install && npm run tauri dev` ≤ 10 phút fresh setup.
- **NFR-M8**: Logging configurable qua `RUST_LOG`, structured (json), không log path nhạy cảm.

#### Compatibility & Distribution

- **NFR-C1**: macOS 13 Ventura+ supported, CI matrix macOS 13/14/15.
- **NFR-C2**: Apple Silicon primary, Intel không support v1.0.
- **NFR-C3**: `.dmg` bundle size ≤ 15 MB, hard cap 25 MB.
- **NFR-C4**: Backward compat — `tiny-core` API minor = additive only, major = migration guide CLI consumer.
- **NFR-C5**: Settings file format backward-compatible trong major version, auto-migration khi cần.

### Additional Requirements

> Trích từ Architecture (D1–D10) và Implementation Patterns — technical requirements impact epic/story creation.

#### Starter Template & Project Init

- **AR1 — Starter template ĐÃ CHỌN**: `npm create tauri-app@latest` (Tauri 2.11.2, React 19, TypeScript, Vite). Cần bổ sung thủ công: Tailwind CSS, Vitest, d3-hierarchy, TanStack Query, Zustand. **IMPACT: Epic 1 Story 1 phải là dựng workspace + chạy starter.**
- **AR2**: Trước khi chạy starter, repo phải chuyển thành Cargo workspace 3 crate — `crates/tiny-core` (engine lib), `crates/tiny` (CLI binary hiện hành), `src-tauri/` (Tauri shell). Đây là **D1 — prerequisite tuyệt đối** cho mọi story khác.
- **AR3**: Refactor `src/commands/clean` hiện tại để tách compute (logic) khỏi `dialoguer` (prompt). Compute đi vào `tiny-core`, `dialoguer` chỉ sống ở `crates/tiny/src/render/`. Đây là refactor nặng nhất trong D1.

#### Tech Stack Versions (đã chốt)

- **AR4**: Backend Rust workspace với `serde` 1, `thiserror` 2.0, `sysinfo` 0.32 (core); `tauri` 2.11.2, `tokio` 1.52, `rusqlite` 0.39, `trash` 5.2, `tracing` (src-tauri).
- **AR5**: Frontend React 19 + TypeScript + Vite + Tailwind CSS + TanStack Query 5.100 + Zustand 5.0 + d3-hierarchy + Vitest.
- **AR6**: `tiny-core` KHÔNG được phụ thuộc `tauri`/`tokio`/`rusqlite`/`trash`/`dialoguer` — phải runtime-agnostic. Vi phạm = CI fail.

#### IPC, Events, Persistence Contract

- **AR7**: Tauri command đặt ở `src-tauri/src/commands/`, mỗi domain 1 file (`health.rs`, `scan.rs`, `clean.rs`, `space_lens.rs`, `system.rs`).
- **AR8**: Event đặt tên `domain:action` — `scan:progress`, `scan:done`, `clean:progress`, `clean:done`.
- **AR9**: Long-running ops LUÔN chạy thread nền + emit progress event, KHÔNG block UI thread. Core hàm sync, nhận progress callback (`Fn(Progress)`); src-tauri spawn thread + chuyển callback thành Tauri event.
- **AR10**: SQLite schema cho journal/quarantine/snapshot ở `src-tauri/src/db/` (`schema.rs`, `scan_history.rs`, `quarantine.rs`). `tiny-core` stateless tuyệt đối.
- **AR11**: Settings dạng KV qua `tauri-plugin-store`; rules qua `~/.config/tiny/rules.toml` (vision).

#### Permissions & Platform

- **AR12 — D7 Critical path**: FDA onboarding flow detect khi launch — nếu thiếu Full Disk Access, hiện màn onboarding hướng dẫn cấp qua System Settings. Block mọi luồng scan/clean chạm vùng hệ thống.
- **AR13**: Per-category permission detect (Accessibility cho Login Items, Camera/Mic cho permissions audit) — prompt chỉ khi category đó được dùng lần đầu.
- **AR14**: macOS-specific APIs sử dụng: `trash` 5.2, `symlink_metadata`, `sysinfo` 0.32, `statvfs` + `diskutil apfs list` (phantom space), Tauri tray + notification, `SMAppService` (Login Items v1.4), `tccutil` (permissions audit v1.4), `notify-rs` macOS backend (FSEvents v1.1+).

#### Distribution Pipeline

- **AR15**: GitHub Actions release workflow build `.dmg`, code-sign Apple Developer ID, notarize qua `notarytool`. Tauri updater plugin verify signature trước khi prompt install.
- **AR16**: Homebrew Cask tap (`homebrew-tiny`) auto-update mirror từ GitHub Releases. `.dmg` publish kèm SHA256 checksum + GPG signature.

#### Quality Gates

- **AR17**: CI grep gate cho `fs::remove_*` ngoài `fs_safe.rs` → fail PR.
- **AR18**: CI grep gate cho `unwrap()`/`expect()` ngoài `#[cfg(test)]` → fail PR.
- **AR19**: Registry sync test phủ 100% provider id qua `category_family()`. Panic id lạ.
- **AR20**: `cargo audit` + `npm audit` mỗi PR; bench-suite regression test < 20%.
- **AR21**: Conventional Commits tiếng Việt, enforce qua commitlint hoặc pre-commit hook.

#### Decision Implementation Sequence (từ architecture.md)

- **AR22**: Thứ tự implement: D1 (workspace + tiny-core) → D8 (error types) → D2 (progress callback) → khởi tạo src-tauri + frontend (D3) → D7 (FDA onboarding) → D4+D5 (persistence + hybrid delete) → D6 (tray/menubar v1.2). **IMPACT: Epic 1 phải cover D1+D8+D2; Epic 2 cover Tauri scaffolding + FDA; Epic 3+ cover features.**

### UX Design Requirements

> Trích từ PRD section "Design Principles & UX Language" — coi là UX-DR (UX Design Requirements). PRD đóng vai trò UX spec do không có file UX riêng.

#### Design Language & Tokens

- **UX-DR1**: Implement design tokens cho làn 3 màu — 🟢 Safe / 🟡 Review / 🔴 Destructive. Mỗi làn = (1) màu primary + variant disabled, (2) icon ổn định, (3) label text tiếng Việt ("An toàn"/"Cần xem"/"Nguy hiểm"). Color không phải means truyền tải duy nhất (NFR-A5).
- **UX-DR2**: Implement Tailwind theme với palette neutral cho text + làn 3 màu accent. Dark mode + light mode parity (auto-detect system preference). Contrast ratio đạt WCAG AA mọi theme.
- **UX-DR3**: Typography scale — font ≥ 14pt cho destructive dialog (NFR-A6), responsive scale tới 200% mà UI không vỡ (NFR-A3, FR61).

#### Reusable UI Components (proposed)

- **UX-DR4**: Component `SafetyLane` (3 cột UI làn 3 màu) — receive list categories, render group header với icon + label + summary size, mỗi category là card có checkbox + name + reclaimable size. Mặc định 🟢 ticked, 🟡/🔴 untick.
- **UX-DR5**: Component `CategoryDrillDown` (preview file list bung từ category) — table size/path/last-access, checkbox per file, virtualization nếu > 200 file.
- **UX-DR6**: Component `EquivalentCLIFooter` — sticky footer hiển thị command `tiny ...` tương đương + button Copy. Toggle visibility qua settings (FR40).
- **UX-DR7**: Component `HealthScoreBadge` — số 0–100, màu (🟢 ≥ 80 / 🟡 50–79 / 🔴 < 50), click bung breakdown (disk pressure %, junk count %, memory %, quarantine age %).
- **UX-DR8**: Component `CleanupDiffReport` — kết quả cleanup dạng diff (`+ X.X GB freed` + breakdown per category với destination icon: → quarantine / → trash / → cleared).
- **UX-DR9**: Component `DestructiveConfirmDialog` — modal chữ to (≥ 14pt), 2 nút label cụ thể ("Hủy" / "Xoá vào Trash" hoặc "Đưa vào quarantine 30 ngày" — không "OK/Cancel"), keyboard accessible.
- **UX-DR10**: Component `ProgressStream` — loading indicator KHÔNG bao giờ là static spinner > 500 ms, phải show byte count / file count / percent realtime từ progress event.
- **UX-DR11**: Component `EmptyState` — placeholder khi không có data (chưa scan, không có file trong category). Text minh bạch, không marketing-speak (vibe section).
- **UX-DR12**: Component `FocusIndicator` — visible outline ≥ 2px contrast ≥ 3:1 cho mọi interactive element (NFR-A1).

#### Interaction Patterns & UX Writing

- **UX-DR13**: Keyboard shortcuts mandatory cho power user — Cmd+S Smart Scan, Cmd+Z Undo, Cmd+, Settings, ⌫ back navigation, Tab/Shift+Tab navigate, Space toggle checkbox.
- **UX-DR14**: Inline expansion (drill, drawer) thay cho modal dialog khi không destructive. Modal CHỈ cho confirmation destructive action.
- **UX-DR15**: UX writing dùng human language song song với số byte — "18.4 GB · đủ chỗ cho ~400 ảnh hoặc 2 phim 4K". Toggle secondary unit trong settings.
- **UX-DR16**: Thời gian human-readable trong UI ("3 ngày trước"), hover hiện ISO 8601 đầy đủ. CLI giữ ISO 8601.
- **UX-DR17**: Action verb cụ thể — "Đưa vào quarantine 30 ngày" thay "Delete", "Hoàn tác" thay "Restore", "Phô CLI" thay "Show command".
- **UX-DR18**: Tone neutral, không scare-tactic — Health Score đỏ chỉ nói "Cần dọn ~18 GB", không "Critical alert". Cấm "boost"/"supercharge"/"blazingly fast"/"AI-powered" trong copy.
- **UX-DR19**: "Explain, don't just show" — mọi số trong UI phải có path drill-down giải thích "tại sao". Designer reviewer dùng rule này; số không có drill = fail review.

#### Accessibility (chi tiết hoá NFR-A)

- **UX-DR20**: ARIA landmark regions đúng (`main`, `nav`, `aside`); VoiceOver compatible — mỗi action có `aria-label`/`aria-describedby`.
- **UX-DR21**: Reduced Motion respect — nếu macOS `Reduce Motion` bật, tắt animation > 200ms (NFR-A7).
- **UX-DR22**: Color blindness audit — làn 3 màu kèm icon + label distinguishable trong protanopia/deuteranopia/tritanopia simulator.

### FR Coverage Map

| FR   | Epic                                 | Mô tả ngắn                                                             |
| ---- | ------------------------------------ | ---------------------------------------------------------------------- |
| FR1  | Epic 2                               | Smart Scan single-action — trigger scan + total reclaimable            |
| FR2  | Epic 3                               | 3 safety lanes 🟢🟡🔴 — default tick 🟢, opt-in 🟡/🔴                  |
| FR3  | Epic 3                               | Drill category → preview file size/path/age                            |
| FR4  | Epic 3                               | Selective include/exclude individual files                             |
| FR5  | Epic 3                               | Execute cleanup + end-of-run diff report                               |
| FR6  | Epic 1                               | Provider trait + registry sync test (architectural)                    |
| FR7  | Epic 1 (CLI via Story 1.4)           | App uninstall flow — CLI preserved; GUI wrapper deferred v1.1          |
| FR8  | Epic 1 (CLI via Story 1.2)           | Duplicate finder by hash — CLI preserved; GUI wrapper deferred v1.1    |
| FR9  | Epic 3                               | Never permanent delete (Trash/quarantine invariant)                    |
| FR10 | Epic 3                               | Restore from quarantine ≤ 30 days via Cleanup History                  |
| FR11 | Epic 3                               | Cmd+Z undo last session                                                |
| FR12 | Epic 3                               | Auto-purge 30d + size > 10 GB notify                                   |
| FR13 | Epic 1 + Epic 3                      | `symlink_metadata` + canonicalize (architectural, verify per-provider) |
| FR14 | Epic 1 + Epic 3                      | Skip cache when app running (existing engine, verify in cleanup flow)  |
| FR15 | Epic 6                               | Lock specific folders (Space Lens follow-up)                           |
| FR16 | Epic 2                               | Smart Scan = sys + scan + dry-run + Health Score                       |
| FR17 | Epic 2                               | Health Score breakdown explainability                                  |
| FR18 | Epic 7                               | Scheduled Smart Scan (weekly/daily) + notification                     |
| FR19 | Epic 6                               | Interactive treemap home/full volume                                   |
| FR20 | Epic 6                               | Treemap drill-down + breadcrumb + ⌫ back                               |
| FR21 | Epic 6                               | Toggle treemap ↔ sunburst                                              |
| FR22 | Epic 6                               | Color heatmap by age/type                                              |
| FR23 | Epic 6                               | Right-click tile → Reveal/Quick Look/Trash/quarantine/uninstall        |
| FR24 | Epic 6                               | Exclude system/cloud-synced/pinned folders                             |
| FR25 | Epic 6                               | Phantom space detection (APFS purgeable + snapshot)                    |
| FR26 | Epic 6                               | FSEvents incremental re-scan                                           |
| FR27 | Epic 6                               | Export Space Lens PNG/JSON                                             |
| FR28 | Epic 7                               | Menubar widget realtime CPU/RAM/disk + sparkline                       |
| FR29 | Epic 7                               | Menubar mini-panel Health Score + Smart Scan button                    |
| FR30 | Epic 7                               | Menubar-only mode (hide Dock icon)                                     |
| FR31 | Epic 7                               | Periodic snapshots + "what changed this week?"                         |
| FR32 | Epic 7                               | Disk-full forecast + proactive notify                                  |
| FR33 | Epic 7                               | FSEvents "newly large files" badge                                     |
| FR34 | Epic 7                               | Heavy Consumers panel (process CPU/RAM quit)                           |
| FR35 | Epic 8                               | "Cleanup during focus" toggle Pomodoro                                 |
| FR36 | Epic 8                               | End-of-session cleanup notification                                    |
| FR37 | Epic 8                               | Choose categories run during focus (default 🟢)                        |
| FR38 | Epic 1                               | CLI parity invariant (architectural)                                   |
| FR39 | Epic 1 (slot) + Epic 2/3 (feed)      | CLI footer per GUI action                                              |
| FR40 | Epic 4                               | CLI footer toggle in settings                                          |
| FR41 | Epic 1 (infra) + Epic 3 (clean JSON) | `--json` output mọi CLI command                                        |
| FR42 | Epic 1                               | Progress callback structured updates                                   |
| FR43 | Epic 9                               | Free up RAM single action                                              |
| FR44 | Epic 9                               | macOS periodic maintenance scripts                                     |
| FR45 | Epic 9                               | Login Items + Launch Agents panel                                      |
| FR46 | Epic 10                              | Scan PUP/adware signatures                                             |
| FR47 | Epic 10                              | Privacy cleaner (history/cookies/recent/Wi-Fi)                         |
| FR48 | Epic 10                              | App updater non-MAS                                                    |
| FR49 | Epic 10                              | Permissions audit (TCC)                                                |
| FR50 | Epic 1                               | FDA onboarding flow on launch                                          |
| FR51 | Epic 10                              | Per-category permission prompts (Accessibility, Camera...)             |
| FR52 | Epic 4                               | Quarantine retention period config (7–90 days)                         |
| FR53 | Epic 4                               | Auto-update behavior (check/download/install)                          |
| FR54 | Epic 4                               | Anonymous crash report opt-in                                          |
| FR55 | Epic 4                               | Clear local journal + quarantine entirely                              |
| FR56 | Vision                               | Custom cleanup rules `rules.toml` (v2.0)                               |
| FR57 | Epic 5                               | Homebrew Cask install                                                  |
| FR58 | Epic 5                               | Signed + notarized `.dmg` + SHA256 + GPG                               |
| FR59 | Epic 5                               | Update check GitHub Releases API                                       |
| FR60 | Epic 4                               | Full keyboard navigation                                               |
| FR61 | Epic 4                               | Font size scaling 200%                                                 |
| FR62 | Epic 4                               | WCAG AA contrast across themes                                         |
| FR63 | Epic 4                               | Destructive confirmation dialog (large text + clear labels)            |
| FR64 | Epic 1                               | Public GitHub + permissive license                                     |
| FR65 | Epic 1                               | CONTRIBUTING.md cho contributor mới                                    |
| FR66 | Epic 1                               | Conventional Commits tiếng Việt + commitlint                           |

**Coverage check:**

- ✅ 65/66 FRs mapped to MVP/Post-MVP epics
- ⏸️ FR56 (rules engine) defer sang workflow Vision (v2.0+)

## Epic List

### Epic 1: Foundation — Workspace, Engine Boundary, Tauri Shell, FDA Onboarding

Maintainer + contributor có project workspace với `tiny-core` runtime-agnostic, CLI binary giữ nguyên hành vi (zero regression), Tauri 2 shell chạy được, FDA onboarding hoạt động, CI quality gates enforce. Contributor mới có thể submit PR thêm provider mới trong ≤ 1 giờ. Đây là **prerequisite tuyệt đối** cho mọi epic khác.

**FRs covered:** FR6, FR13, FR14, FR38, FR41 (infrastructure), FR42, FR50, FR64, FR65, FR66
**Architecture decisions:** D1 (workspace + tiny-core extraction), D2 (progress callback), D7 (FDA onboarding), D8 (error types), AR1–AR12, AR17–AR22
**Phase:** MVP P1

### Epic 2: Smart Scan & Health Score

User mở app, click 1 nút "Smart Scan", thấy Health Score 0–100 với màu (🟢 ≥ 80 / 🟡 50–79 / 🔴 < 50) + reclaimable space + drill-down breakdown explain "tại sao 51?" (disk pressure %, junk %, memory %, quarantine age %). Progress event realtime, không spinner câm > 500 ms.

**FRs covered:** FR1, FR16, FR17
**UX components:** `HealthScoreBadge`, `ProgressStream`, `EmptyState`, "Explain don't show" drill (UX-DR7, UX-DR10, UX-DR11, UX-DR19)
**Phase:** MVP P1

### Epic 3: Cleanup Flow with Safety Net (3 Lanes + Preview + Quarantine + Undo + History)

User chọn category theo làn 3 màu (🟢 default ticked, 🟡/🔴 opt-in), preview file thật trước khi xoá (size/path/age + virtualization), execute hybrid delete (Trash/quarantine), xem diff report dạng `git diff`. Restore từng file qua "Cleanup History" panel ≤ 30 ngày HOẶC Cmd+Z undo lần dọn gần nhất. Auto-purge sau 30 ngày + notify khi > 10 GB. Mọi xoá an toàn (`fs_safe`, `symlink_metadata` canonicalize). App đang chạy → skip cache + surface skip reason.

**FRs covered:** FR2, FR3, FR4, FR5, FR9, FR10, FR11, FR12, FR13 (verify), FR14 (verify)
**FRs satisfied at CLI level (GUI deferred v1.1):** FR7 (uninstall — Epic 1 Story 1.4), FR8 (duplicates — Epic 1 Story 1.2)
**Architecture decisions:** D4 (SQLite journal + quarantine), D5 (hybrid delete Trash/quarantine), AR10
**UX components:** `SafetyLane`, `CategoryDrillDown`, `CleanupDiffReport`, `DestructiveConfirmDialog`, human language bytes, action verbs (UX-DR1, UX-DR4, UX-DR5, UX-DR8, UX-DR9, UX-DR15, UX-DR17)
**Phase:** MVP P1

### Epic 4: Settings, Preferences & Accessibility Foundation

User customize quarantine retention (7–90 days), auto-update behavior (check/download/install độc lập), opt-in crash report (default OFF), clear journal + quarantine. Accessibility baseline: dark/light theme parity, FocusIndicator visible (≥ 2px outline contrast ≥ 3:1), full keyboard navigation, 200% font scaling, WCAG AA contrast, VoiceOver compatible, Reduced Motion respect. CLI footer toggle (default ON) sống ở Settings.

**FRs covered:** FR40, FR52, FR53, FR54, FR55, FR60, FR61, FR62, FR63
**UX components:** Design tokens dark/light (UX-DR2, UX-DR3), `FocusIndicator` (UX-DR12), `EquivalentCLIFooter` global shell (UX-DR6), ARIA landmarks (UX-DR20), Reduced Motion (UX-DR21), color blindness audit (UX-DR22)
**Phase:** MVP P1

### Epic 5: Distribution & Update Pipeline

User `brew install --cask tiny` hoặc download `.dmg` notarized từ GitHub Releases với SHA256 checksum + GPG signature. App auto-check update qua GitHub Releases API (max 24h, configurable), verify signature `.dmg` bằng public key embed TRƯỚC khi prompt install. Homebrew Cask formula auto-update khi GitHub Release mới publish.

**FRs covered:** FR57, FR58, FR59
**Architecture decisions:** AR15, AR16, NFR-S7, S8, S11, NFR-I2, I3
**Phase:** MVP P1

---

### Epic 6: Space Lens — Disk Visualization

User explore disk usage qua treemap/sunburst tương tác (full volume hoặc home dir), drill-down double-click + breadcrumb + ⌫ back, heat-by-age/heat-by-type color overlay, phantom space detection (APFS purgeable + snapshot explain). Right-click tile → Reveal in Finder / Quick Look / Trash / quarantine / uninstall. Exclude system/cloud-synced/pinned folders. FSEvents incremental re-scan. Export PNG/JSON. **Khối kỹ thuật lớn nhất** — parallel walker mới, treemap virtualization 500k files.

**FRs covered:** FR15, FR19, FR20, FR21, FR22, FR23, FR24, FR25, FR26, FR27
**Phase:** Post-MVP v1.1 (P2)

### Epic 7: Active Monitoring & Explanation

Menubar tray widget realtime CPU/RAM/disk + sparkline. Click → mini-panel Health Score + "Run Smart Scan" button. Menubar-only mode (hide Dock icon). Periodic snapshots → "tuần này có gì đổi" diff. Disk-full forecast + proactive notify. FSEvents "newly large files" badge ở Downloads/Desktop/Documents. Heavy Consumers panel — quit process trực tiếp. Scheduled Smart Scan weekly/daily.

**FRs covered:** FR18, FR28, FR29, FR30, FR31, FR32, FR33, FR34
**Architecture decisions:** D6 (tray + menubar), tauri-plugin-notification
**Phase:** Post-MVP v1.2 (P3)

### Epic 8: Focus + Cleanup Integration

User bật "cleanup during focus" toggle trong focus timer settings → trong Pomodoro session app dọn nền các category 🟢 Safe lane (default, customizable). End-of-session notification "đã giải phóng X GB", không bao giờ interrupt focus session. Differentiator độc nhất của `tiny`.

**FRs covered:** FR35, FR36, FR37
**Phase:** Post-MVP v1.3 (P4)

### Epic 9: Maintenance Scripts

User free RAM (purge inactive memory + RAM trước/sau), run macOS periodic maintenance scripts (flush DNS / reindex Spotlight / repair disk permissions / rebuild Launch Services) individually hoặc batch. Manage Login Items + Launch Agents qua unified panel (`SMAppService`).

**FRs covered:** FR43, FR44, FR45
**Phase:** Post-MVP v1.4 part (P5)

### Epic 10: Protection — Malware, Privacy, App Updater, Permissions Audit

Scan PUP/adware signatures → quarantine matches. Privacy cleaner (browser history/cookies/recent items/saved Wi-Fi). Detect outdated apps non-MAS + offer update. Permissions audit qua `tccutil` — list apps giữ camera/mic/FDA/Accessibility, revoke selectively. Per-category permission prompts khi category dùng lần đầu.

**FRs covered:** FR46, FR47, FR48, FR49, FR51
**Phase:** Post-MVP v1.4 part (P5)

---

### Vision Backlog (defer to v2.0+ workflow)

- **FR56**: Custom cleanup rules `~/.config/tiny/rules.toml` — rules engine kiểu Mail rules. Defer sang workflow Vision riêng vì cần thiết kế DSL + safety guarantees riêng.

---

## Epic 1: Foundation — Workspace, Engine Boundary, Tauri Shell, FDA Onboarding

Maintainer + contributor có project workspace với `tiny-core` runtime-agnostic, CLI binary giữ nguyên hành vi (zero regression), Tauri 2 shell chạy được, FDA onboarding hoạt động, CI quality gates enforce. Contributor mới có thể submit PR thêm provider mới trong ≤ 1 giờ. Đây là **prerequisite tuyệt đối** cho mọi epic khác.

### Story 1.1: Convert repo thành Cargo workspace 3 crate skeleton

As a maintainer of `tiny-cli`,
I want to convert the single-binary repository into a Cargo workspace with 3 crates (`tiny-core`, `tiny`, `src-tauri`),
So that engine logic, CLI binary, and the future Tauri app can share code without duplication while each builds independently.

**Acceptance Criteria:**

**Given** the repo is currently a single binary crate
**When** I run `cargo build --workspace`
**Then** 3 members build successfully (`crates/tiny-core` empty lib, `crates/tiny` CLI binary, placeholder `src-tauri/` lib crate)

**Given** the workspace root `Cargo.toml` exists
**When** inspected
**Then** it has `[workspace] members = ["crates/tiny-core", "crates/tiny", "src-tauri"]` and `resolver = "2"`

**Given** current CLI source lives at `src/`
**When** I relocate via `git mv`
**Then** code moves to `crates/tiny/src/` preserving git history (`git log --follow` works on relocated files)

**Given** existing integration tests in `tests/`
**When** relocated to `crates/tiny/tests/`
**Then** `cargo test -p tiny` passes 100% (zero regression on `clean_smoke.rs` and any other existing integration test)

**Given** `Cargo.lock` was previously gitignored
**When** workspace is established
**Then** `Cargo.lock` is committed at the workspace root (workspace with binaries should pin lockfile)

**Given** the binary name must stay `tiny`
**When** I run `cargo build --release -p tiny`
**Then** `target/release/tiny` exists and `tiny --version` returns the expected output unchanged

**Given** `.gitignore` had `/target`
**When** workspace is created
**Then** no `target/` directories are tracked by git

### Story 1.2: Extract `sys` + `scan` modules vào `tiny-core` + define `thiserror` errors

As an engine developer,
I want `sys` and `scan` compute logic moved into `tiny-core` with typed errors via `thiserror`,
So that the future Tauri command layer can call them without depending on `anyhow` or CLI presentation.

**Acceptance Criteria:**

**Given** `sys` and `scan` modules currently live in the CLI binary
**When** I move source files
**Then** they reside under `crates/tiny-core/src/sys/` and `crates/tiny-core/src/scan/`

**Given** `crates/tiny-core/Cargo.toml`
**When** inspected
**Then** it has dependencies `serde`, `sysinfo` 0.32, `thiserror` 2.0 but NOT `tauri`, `tokio`, `rusqlite`, `dialoguer`, `anyhow`

**Given** `crates/tiny-core/src/error.rs` defines errors
**When** inspected
**Then** it has `#[derive(thiserror::Error, Debug)] pub enum CoreError` with variants covering IO failures, parse failures, and scan-specific errors

**Given** the CLI binary `crates/tiny` now depends on `tiny-core`
**When** CLI code calls `tiny_core::sys::snapshot()`
**Then** it returns `Result<SysSnapshot, CoreError>` and the CLI wraps with `anyhow::Result` only at the outer-most boundary (e.g., `main.rs`)

**Given** `scan --json` flag previously produced known output
**When** I run `tiny scan --json` after refactor on a fixture directory
**Then** output is byte-identical to a golden snapshot captured pre-refactor

**Given** `crates/tiny-core/src/lib.rs`
**When** inspected
**Then** it re-exports `pub use sys::*; pub use scan::*; pub use error::CoreError;`

**Given** the architectural rule that `tiny-core` produces no I/O
**When** `grep -rE "println!|eprintln!|dialoguer|log::" crates/tiny-core/src` runs
**Then** there are 0 matches

### Story 1.3: Refactor `clean` — tách compute vào `tiny-core`, giữ `dialoguer` ở CLI render

As an engine developer,
I want `clean` providers, registry, and safety primitives moved into `tiny-core` while interactive prompts stay in the CLI render layer,
So that the GUI can drive cleanup with the same engine and the registry-sync test continues to enforce 3-site registration.

**Acceptance Criteria:**

**Given** `clean` currently mixes compute and `dialoguer` prompts in the CLI
**When** refactored
**Then** `crates/tiny-core/src/clean/` contains `providers/`, `registry.rs` (with `all_providers()`, `known_category_ids()`, `category_family()`), `fs_safe.rs`, and `ExecAction` / `CleanReport` types

**Given** `dialoguer` prompts previously lived inside the clean command flow
**When** refactored
**Then** they relocate to `crates/tiny/src/render/clean.rs` and `tiny-core` never imports `dialoguer`

**Given** the registry-sync test asserts `category_family()` panics for an unknown id
**When** I run `cargo test -p tiny-core`
**Then** the test passes and is inline in `registry.rs` via `#[cfg(test)] mod tests`

**Given** a contributor adds a new provider but forgets to update `category_family()`
**When** `cargo test` runs
**Then** the test panics with a clear message naming the missing id (e.g., `"id 'safari_cache' has no family mapping"`)

**Given** `fs_safe::remove_recursive_safe` is the safety primitive
**When** inspected
**Then** it lives in `tiny-core/src/clean/fs_safe.rs` and uses `symlink_metadata` (not `is_dir()`) plus path canonicalization against a per-provider whitelist (per FR13 invariant)

**Given** existing CLI integration test `tests/clean_smoke.rs`
**When** I run `cargo test -p tiny --test clean_smoke`
**Then** it passes 100% post-refactor

**Given** the CI grep gate for `fs::remove_*`
**When** it runs across `crates/tiny-core/src`, `crates/tiny/src`, `src-tauri/src` excluding `tiny-core/src/clean/fs_safe.rs`
**Then** 0 matches are found

### Story 1.4: Extract `focus` + `uninstall` modules vào `tiny-core`

As an engine developer,
I want `focus` (Pomodoro timer) and `uninstall` (app removal) modules moved into `tiny-core`,
So that the GUI can later integrate focus-driven cleanup (Epic 8) and reuse uninstall logic without rewriting it.

**Acceptance Criteria:**

**Given** `focus` module currently lives in the CLI binary
**When** moved
**Then** it lives at `crates/tiny-core/src/focus/` with no `println!` or `dialoguer` (timer state is pure; presentation stays in CLI render)

**Given** `uninstall` similarly relocates
**When** inspected
**Then** it lives at `crates/tiny-core/src/uninstall/` and exposes `pub fn uninstall(app_path: &Path) -> Result<UninstallReport, CoreError>`

**Given** the focus timer needs no I/O during tick (pure state machine)
**When** I run `cargo test -p tiny-core focus`
**Then** unit tests cover start/pause/resume/finish transitions without any `sleep` calls

**Given** the CLI `tiny focus` command pre-existed
**When** I run it after refactor
**Then** output and behavior match pre-refactor (regression test via `assert_cmd`)

**Given** the CLI `tiny uninstall <app>` command pre-existed
**When** I run it on a fixture app bundle
**Then** the same caches/prefs/launch agents are detected and the report shape is identical to pre-refactor

**Given** `crates/tiny-core/src/lib.rs`
**When** inspected
**Then** it re-exports `pub use focus::*; pub use uninstall::*;` alongside earlier exports

### Story 1.5: Define `Progress` type + progress callback API trong `tiny-core` + integrate vào `scan`

As an engine developer,
I want a runtime-agnostic progress callback API in `tiny-core` and `scan` integrated with it,
So that the GUI can display realtime progress events without `tiny-core` depending on `tokio` or any async runtime.

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/progress.rs` is created
**When** inspected
**Then** it defines `#[derive(Clone, Debug, Serialize)] #[serde(rename_all = "camelCase")] pub struct Progress { pub current: u64, pub total: u64, pub message: String }`

**Given** the `scan` function signature in core
**When** inspected
**Then** it becomes `pub fn scan(opts: ScanOpts, progress: impl Fn(Progress) + Send) -> Result<ScanReport, CoreError>` (synchronous, callback closure, no runtime dependency)

**Given** `crates/tiny-core/Cargo.toml`
**When** inspected
**Then** it does NOT depend on `tokio` or any async runtime — verified via `cargo tree -p tiny-core | grep tokio` returning 0 matches

**Given** the CLI binary now calls `scan` with a no-op closure `|_| {}`
**When** `tiny scan --json` runs over a fixture directory
**Then** output matches the golden snapshot from Story 1.2

**Given** a unit test that captures progress events into a `Vec<Progress>` via a closure
**When** `scan` runs over a fixture directory of ≥ 100 files
**Then** the vector contains ≥ 1 event with `current > 0` and a final event where `current == total`

**Given** progress events should not fire on every single file (cost) nor only at end (useless)
**When** scan iterates
**Then** the callback is invoked with reasonable buffering — at least once per 100 entries or 100 ms tick, whichever is sooner

### Story 1.6: Scaffold Tauri 2 app via `create-tauri-app` + frontend deps + lint rules

As a maintainer,
I want a Tauri 2.11.2 app scaffolded with React 19 + TypeScript + Vite + Tailwind + TanStack Query + Zustand + Vitest + d3-hierarchy, and an eslint rule enforcing the `features → components → lib` dependency flow,
So that frontend development can begin with the architecture's chosen stack already in place and dependency rules pre-enforced.

**Acceptance Criteria:**

**Given** the workspace has a placeholder `src-tauri/`
**When** I run `npm create tauri-app@latest` selecting React + TypeScript + Vite
**Then** `src-tauri/`, `src/`, `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html` are scaffolded at the repo root

**Given** `package.json`
**When** inspected
**Then** it lists `@tanstack/react-query` ^5.100, `zustand` ^5.0, `d3-hierarchy` (latest), `tailwindcss` (latest), `vitest` (latest), `@tauri-apps/api` ^2 with `@tauri-apps/plugin-store` placeholder

**Given** `src-tauri/Cargo.toml`
**When** inspected
**Then** it depends on `tauri` 2.11.2, `tokio` 1.52, `tracing` (latest), `tiny-core` (workspace path), and is listed as a workspace member in the root `Cargo.toml`

**Given** `tailwind.config.ts` and `src/index.css` with `@tailwind base; @tailwind components; @tailwind utilities;`
**When** `npm run dev` runs
**Then** a smoke component using Tailwind utility classes renders correctly

**Given** the architecture invariant `features/ → components/ → lib/` (no reverse imports, even type-only)
**When** `eslint.config.js` is inspected
**Then** it has `no-restricted-imports` (or equivalent) rules forbidding imports from `features/*` or `components/*` inside files under `src/lib/`

**Given** `vitest.config.ts`
**When** `npm run test` runs
**Then** a placeholder smoke test `src/lib/format.test.ts` passes (asserts a trivial pure function)

**Given** Tauri integration with Vite
**When** `npm run tauri dev` runs
**Then** a Tauri window opens displaying the default scaffold (proves dev-time IPC + bundling work)

**Given** the architecture invariant that `tiny-core` MUST NOT depend on `tauri`/`tokio`/`rusqlite`/`trash`/`dialoguer`
**When** `cargo tree -p tiny-core | grep -E "tauri|tokio|rusqlite|trash|dialoguer"` runs
**Then** 0 matches are found

### Story 1.7: Tauri IPC smoke — `sys_snapshot` command + camelCase serde + ErrorPayload

As a frontend developer,
I want a working end-to-end Tauri command `sys_snapshot` that returns a typed struct in camelCase JSON with a proper error envelope,
So that the IPC contract pattern (struct shape + ErrorPayload mapping) is concrete and replicable for every future command.

**Acceptance Criteria:**

**Given** `src-tauri/src/commands/system.rs`
**When** inspected
**Then** it has `#[tauri::command] pub async fn sys_snapshot() -> Result<SysSnapshot, ErrorPayload>` which calls `tiny_core::sys::snapshot()`

**Given** `src-tauri/src/error.rs`
**When** inspected
**Then** it defines `#[derive(Serialize)] #[serde(rename_all = "camelCase")] pub struct ErrorPayload { pub code: String, pub message: String }` and `impl From<CoreError> for ErrorPayload`

**Given** the `SysSnapshot` struct returned by core
**When** marshaled to JSON via IPC
**Then** field names are camelCase (e.g., `totalMemory`, `usedMemory` — never `total_memory`), enforced by `#[serde(rename_all = "camelCase")]`

**Given** `src/lib/ipc.ts`
**When** inspected
**Then** it has a typed wrapper `export async function sysSnapshot(): Promise<SysSnapshot>` calling `invoke("sys_snapshot")` with the matching TypeScript type

**Given** the frontend uses TanStack Query
**When** `App.tsx` calls `useQuery({ queryKey: ['sys'], queryFn: sysSnapshot })`
**Then** the page displays `totalMemory` and `usedMemory` from the engine

**Given** a forced error (e.g., a debug feature flag returning `Err(CoreError::Io(...))`)
**When** `sys_snapshot` fails
**Then** the frontend receives `ErrorPayload { code, message }` populated and TanStack Query's `error` field is set (no app crash)

**Given** the rule that no command may panic across the IPC boundary
**When** `grep -rE "\.unwrap\(\)|\.expect\(" src-tauri/src/commands/` runs (excluding `#[cfg(test)]`)
**Then** 0 matches are found

### Story 1.8: Tauri progress event bridge — `scan_run` spawn thread + emit `scan:progress`

As a frontend developer,
I want `scan_run` to execute `tiny_core::scan` on a background thread while emitting `scan:progress` Tauri events,
So that the UI receives realtime progress updates without blocking the main thread.

**Acceptance Criteria:**

**Given** `src-tauri/src/commands/scan.rs`
**When** inspected
**Then** `#[tauri::command] pub async fn scan_run(app: AppHandle, opts: ScanOpts) -> Result<ScanReport, ErrorPayload>` exists and spawns the sync core call via `tokio::task::spawn_blocking`

**Given** the progress callback adapter wraps `app.emit("scan:progress", progress)`
**When** core invokes the callback during scan
**Then** a Tauri event is emitted carrying the `Progress` payload serialized as camelCase

**Given** the frontend hook `useScanProgress()`
**When** it subscribes via `listen("scan:progress", ...)` and the component unmounts
**Then** the listener is cleaned up (unlisten called — verified via test mock or smoke check that re-mounting does not double-subscribe)

**Given** the UI displays a progress bar driven by `scan:progress` events
**When** I run `scan_run` over `~/Downloads`
**Then** the bar updates ≥ 1 time mid-scan and reaches 100% on completion

**Given** the scan completes
**When** the final `ScanReport` returns via promise resolution
**Then** the UI transitions out of the loading state and the TanStack Query result is populated with the report

**Given** the requirement that long-running ops never block UI (AR9, NFR-P7)
**When** `scan_run` runs for ≥ 3 s
**Then** the main window remains interactive — the window can be dragged and buttons respond, and DevTools shows no frame drop > 16.6 ms on the main thread

### Story 1.9: FDA (Full Disk Access) onboarding flow

As a first-time user (Persona A/C),
I want the app to detect missing Full Disk Access on launch and guide me through granting it via System Settings,
So that I cannot accidentally run scans against `~/Library` without the permission they require.

**Acceptance Criteria:**

**Given** `src-tauri/src/permissions.rs`
**When** inspected
**Then** it has `pub fn detect_full_disk_access() -> bool` that attempts to read a probe path under `~/Library/Caches/com.apple.Spotlight` and returns false on permission error

**Given** the Tauri command `check_full_disk_access() -> Result<bool, ErrorPayload>`
**When** the frontend invokes it on app start
**Then** it returns the FDA status before any scan command is allowed to run

**Given** the user has not granted FDA
**When** the app launches
**Then** `features/onboarding/` displays an instructional screen containing: (1) plain-language explanation of why FDA is needed, (2) "Mở System Settings" button, (3) "Kiểm tra lại" button

**Given** the "Mở System Settings" button
**When** clicked
**Then** Tauri command `open_fda_settings()` opens `x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles` (Privacy & Security → Full Disk Access)

**Given** the user grants FDA and clicks "Kiểm tra lại"
**When** `check_full_disk_access()` returns true
**Then** the onboarding screen dismisses and the main app renders

**Given** the user already has FDA granted on a subsequent launch
**When** the app starts
**Then** the onboarding screen does NOT appear and the main app renders directly

**Given** the user dismisses or closes the onboarding window without granting
**When** they reopen the app
**Then** onboarding re-appears (no silent bypass — there is no "skip" button)

**Given** Persona C accessibility requirements
**When** the onboarding screen renders
**Then** button labels are large readable text (≥ 14 pt) with clear action verbs in Vietnamese ("Mở System Settings", "Kiểm tra lại") — not generic "OK" / "Cancel"

### Story 1.10: CI quality gates — grep gates + cargo/npm audit + bench harness scaffold

As a maintainer,
I want CI to enforce architectural invariants and dependency safety automatically,
So that PRs violating the safety net (raw `fs::remove_*`, `unwrap()` in production code, CVE deps) cannot merge.

**Acceptance Criteria:**

**Given** `.github/workflows/ci.yml`
**When** inspected
**Then** it has jobs `rust-build-test`, `frontend-test`, `audit`, and `grep-gate`

**Given** the `fs::remove_*` grep gate
**When** CI runs `grep -rE "fs::remove_(file|dir|dir_all)" crates/tiny-core/src crates/tiny/src src-tauri/src` excluding `crates/tiny-core/src/clean/fs_safe.rs`
**Then** 0 matches are found, and the job fails with a clear error message naming any future offending file/line

**Given** the `unwrap()` / `expect()` grep gate
**When** CI runs `grep -rnE "\.unwrap\(\)|\.expect\(" crates/ src-tauri/src` excluding lines inside `#[cfg(test)]` blocks
**Then** 0 matches are found in production code

**Given** `cargo audit`
**When** CI runs it
**Then** the job fails on any advisory severity ≥ Medium that lacks a whitelisted justification in `audit.toml`

**Given** `npm audit --production --audit-level=high`
**When** CI runs it
**Then** the job fails on any vulnerability severity ≥ High in production deps

**Given** the placeholder bench harness `crates/tiny-core/benches/scan_bench.rs`
**When** `cargo bench -p tiny-core` runs in CI as a non-blocking job
**Then** it executes a smoke benchmark using `Criterion` over a small fixture dir without errors — real NFR-P performance gates are wired in Epic 2

**Given** the CI matrix
**When** PRs target `main`
**Then** macOS-latest (Apple Silicon) is the primary runner; macOS-13 + 14 are listed as additional matrix entries per NFR-C1

### Story 1.11: Open-source baseline — README, MIT license, CONTRIBUTING.md, commitlint

As a potential contributor (Persona D),
I want clear repo documentation and commit conventions enforced by tooling,
So that I can submit a working PR (e.g., adding a new cleanup provider) within 1 hour without one-on-one mentorship.

**Acceptance Criteria:**

**Given** `LICENSE` at the repo root
**When** inspected
**Then** it contains MIT license text with `Copyright (c) 2026 Kyantran`

**Given** `README.md`
**When** inspected
**Then** it has sections: project description, install instructions (cargo + Homebrew placeholder), usage examples, "Architecture & Conventions" linking to `_bmad-output/planning-artifacts/architecture.md` and `_bmad-output/project-context.md`, "Contributing" linking to `CONTRIBUTING.md`

**Given** `CONTRIBUTING.md`
**When** inspected
**Then** it has sections: setup prerequisites (rustup + node 20+), provider authoring guide (3-site registration with a concrete example file), test commands (`cargo test` + `npm run test`), commit message convention (`<type>(<scope>): <mô tả tiếng Việt>`) with 3 examples, and a PR checklist

**Given** `.commitlintrc.json` (or `commitlint.config.js`)
**When** inspected
**Then** it extends `@commitlint/config-conventional` with the subject-case-lowercase rule and a max header length of 70 characters

**Given** Husky (or simple-git-hooks) installed
**When** a contributor runs `git commit -m "wrong format"`
**Then** the pre-commit hook rejects the commit and prints a clear error pointing to the convention

**Given** `.github/pull_request_template.md`
**When** a contributor opens a PR
**Then** the body pre-fills sections `## Tóm tắt`, `## Lý do`, `## Thay đổi chính`, `## Cách test`, `## Breaking Changes`

**Given** a fresh `git clone` on macOS with `rustup` + `node 20+`
**When** a contributor follows README setup
**Then** `cargo run -p tiny -- scan` produces output in ≤ 5 minutes wall-clock (NFR-M6)

### Story 1.12: `EquivalentCLIFooter` global shell component slot

As a frontend developer,
I want a reusable `EquivalentCLIFooter` component that displays a CLI command + Copy button as a sticky footer slot,
So that future feature epics (Smart Scan, Cleanup) can plug their equivalent CLI commands into this single slot per FR39.

**Acceptance Criteria:**

**Given** `src/components/equivalent-cli-footer.tsx`
**When** inspected
**Then** it exports `export interface EquivalentCLIFooterProps { command: string; visible?: boolean; }` with a default-export React component

**Given** the component renders
**When** given `command: "tiny clean --category=npm_cache"`
**Then** it displays the command in a monospace font with a "Copy" button on the right

**Given** the Copy button
**When** clicked
**Then** `navigator.clipboard.writeText(command)` is called and a transient toast "Đã copy CLI" appears for 1.5 s

**Given** the layout shell (`App.tsx` or a `Layout.tsx` wrapper)
**When** inspected
**Then** the footer slot uses Tailwind `sticky bottom-0` with sufficient padding so it never overlaps main content

**Given** the `visible` prop
**When** set to `false`
**Then** the component returns `null` — this prepares for the FR40 settings toggle in Epic 4 to wire in without further component changes

**Given** keyboard accessibility per NFR-A1
**When** the user tabs through the page
**Then** the Copy button receives a visible focus indicator (≥ 2 px outline, contrast ≥ 3:1)

**Given** the Vitest test `src/components/equivalent-cli-footer.test.tsx`
**When** it runs
**Then** it asserts (a) the command renders in a monospace element, (b) clicking Copy calls `navigator.clipboard.writeText` (mocked), and (c) `visible: false` returns `null`

---

## Epic 2: Smart Scan & Health Score

User mở app, click 1 nút "Smart Scan", thấy Health Score 0–100 với màu (🟢 ≥ 80 / 🟡 50–79 / 🔴 < 50) + reclaimable space + drill-down breakdown explain "tại sao 51?" (disk pressure %, junk %, memory %, quarantine age %). Progress event realtime, không spinner câm > 500 ms.

### Story 2.1: Define Health Score algorithm + `HealthScore` struct trong `tiny-core`

As an engine developer,
I want a pure Health Score calculation function in `tiny-core` that takes system inputs and returns a 0–100 score plus a transparent breakdown,
So that the GUI can show "tại sao 51?" without re-implementing the math.

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/health/mod.rs` is created
**When** inspected
**Then** it defines `#[derive(Clone, Debug, Serialize)] #[serde(rename_all = "camelCase")] pub struct HealthScore { pub value: u8, pub breakdown: HealthBreakdown }` with `value` constrained to 0–100

**Given** `HealthBreakdown` struct
**When** inspected
**Then** it has four `u8` fields with `#[serde(rename_all = "camelCase")]`: `disk_pressure_contribution`, `junk_contribution`, `memory_pressure_contribution`, `quarantine_age_contribution`, each in range 0–100 with documented weights

**Given** `pub fn compute_health_score(inputs: HealthInputs) -> HealthScore`
**When** inspected
**Then** it is a pure function (no I/O, no time-of-day branching), `HealthInputs` carries `disk_free_ratio: f64`, `junk_bytes: u64`, `memory_pressure_ratio: f64`, `quarantine_oldest_age_days: u32`

**Given** edge-case unit tests in `#[cfg(test)] mod tests`
**When** `cargo test -p tiny-core health` runs
**Then** the following cases pass: (a) all-good inputs → score ≥ 90, (b) full disk (free_ratio < 0.05) → score < 30, (c) zero junk → `junk_contribution == 0`, (d) inputs that would produce > 100 → clamped to 100, (e) inputs that would produce < 0 → clamped to 0

**Given** the score must be reproducible
**When** the same `HealthInputs` are passed twice
**Then** the returned `HealthScore` is byte-identical (no random/time-based variance)

**Given** the explanation text per breakdown component
**When** the struct is inspected
**Then** each `*_contribution` field has a documented `///` comment explaining what the user-visible "why" string should say (consumed by the breakdown drill panel later in this epic)

### Story 2.2: Implement `tiny_core::smart_scan()` orchestration

As an engine developer,
I want a single `smart_scan` function that orchestrates `sys` + `scan` + `clean --dry-run` and returns one consolidated report,
So that the GUI can drive Persona A's one-click experience without coordinating three calls.

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/smart_scan.rs`
**When** inspected
**Then** it defines `pub fn smart_scan(opts: SmartScanOpts, progress: impl Fn(Progress) + Send) -> Result<SmartScanReport, CoreError>`

**Given** `SmartScanReport` struct
**When** inspected
**Then** it has `#[serde(rename_all = "camelCase")]` fields: `health_score: HealthScore`, `reclaimable_bytes: u64`, `categories: Vec<CategorySummary>` where each summary carries `id`, `family`, `reclaimable_bytes`, `file_count`

**Given** the function body
**When** invoked
**Then** it sequentially runs `sys::snapshot()`, `scan::run(opts.scan_opts, ...)`, and `clean::dry_run_all_providers(...)`, feeding outputs into `compute_health_score()`

**Given** the four orchestration stages
**When** progress callback fires
**Then** it emits `Progress` events with `message` field set to one of: `"sys"`, `"scan"`, `"clean_dry_run"`, `"health"` so the GUI can render stage transitions

**Given** the requirement that Smart Scan end-to-end ≤ 5 s P95 (NFR-P4)
**When** the Epic 2 performance bench runs on a fixture of ~50k files
**Then** wall-clock duration is ≤ 5 s (P95)

**Given** the function must not depend on a runtime
**When** `cargo tree -p tiny-core --features smart_scan` runs
**Then** no `tokio` or other async runtime appears in the dependency tree

**Given** that `clean --dry-run` exists for individual categories
**When** `smart_scan` invokes it
**Then** it iterates `all_providers()` and aggregates results without mutating disk state (zero file moves, zero deletes during a Smart Scan)

### Story 2.3: Tauri command `smart_scan_run` + progress event `smartScan:progress`

As a frontend developer,
I want a Tauri command `smart_scan_run` that wraps the engine `smart_scan` with progress events,
So that the GUI receives stage transitions and a final report through the established IPC pattern from Story 1.8.

**Acceptance Criteria:**

**Given** `src-tauri/src/commands/smart_scan.rs`
**When** inspected
**Then** it has `#[tauri::command] pub async fn smart_scan_run(app: AppHandle) -> Result<SmartScanReport, ErrorPayload>` and uses `tokio::task::spawn_blocking` for the sync core call

**Given** the progress callback adapter
**When** core emits `Progress`
**Then** the command emits Tauri event `"smartScan:progress"` carrying the payload serialized as camelCase

**Given** the event naming convention from the architecture
**When** inspected
**Then** the event is `"smartScan:progress"` (camelCase domain), and a final event `"smartScan:done"` is emitted with the full `SmartScanReport` payload on success

**Given** `src/lib/ipc.ts`
**When** inspected
**Then** it adds `export async function smartScanRun(): Promise<SmartScanReport>` calling `invoke("smart_scan_run")`

**Given** the GUI hook `useSmartScan()`
**When** it subscribes to `smartScan:progress` and the component unmounts
**Then** the listener is unlistened (verified by Vitest mock or smoke check)

**Given** a forced engine error
**When** `smart_scan_run` fails
**Then** the frontend receives `ErrorPayload { code, message }` and no panic crosses the IPC boundary

### Story 2.4: `HealthScoreBadge` component (UX-DR7)

As a user,
I want the Health Score displayed as a colored 0–100 badge that I can click to see what makes it that number,
So that the score is both at-a-glance and explainable.

**Acceptance Criteria:**

**Given** `src/components/health-score-badge.tsx`
**When** inspected
**Then** it exports `export interface HealthScoreBadgeProps { score: HealthScore; onClick?: () => void; }` with a default-export React component

**Given** the `value` prop
**When** rendered
**Then** the badge background color matches the threshold rules: `value ≥ 80` → 🟢 green token, `50 ≤ value < 80` → 🟡 amber token, `value < 50` → 🔴 red token (tokens from Epic 4 — for Epic 2 the values use Tailwind defaults pending those tokens)

**Given** color must not be the sole signal (NFR-A5)
**When** rendered
**Then** the badge also renders an icon (✅ / ⚠️ / 🚨) and a label text ("An toàn" / "Cần dọn" / "Cấp bách")

**Given** the badge is clickable
**When** the user clicks it (or focuses + Enter)
**Then** `onClick` fires — a parent view may use this to toggle a detail panel (the badge itself only exposes the callback and has no dependency on any consumer)

**Given** keyboard accessibility (NFR-A1)
**When** the badge is interactive
**Then** it is a `<button>` element with `aria-label` describing the score and outcome (e.g., "Health Score 51 trên 100 — cần dọn"), and tab navigation reaches it with a visible focus outline ≥ 2 px

**Given** Vitest tests `src/components/health-score-badge.test.tsx`
**When** they run
**Then** they assert (a) score 95 renders green + ✅, (b) score 65 renders amber + ⚠️, (c) score 30 renders red + 🚨, (d) clicking fires `onClick` once

### Story 2.5: `ProgressStream` component (UX-DR10)

As a user,
I want the in-progress UI to always show concrete numbers (files counted, bytes scanned, percent) updating in real time,
So that I never stare at a static spinner wondering if the app is alive.

**Acceptance Criteria:**

**Given** `src/components/progress-stream.tsx`
**When** inspected
**Then** it exports `export interface ProgressStreamProps { stage: string; current: number; total: number; }` and renders a progress bar plus a label `"{stage}: {current}/{total} ({percent}%)"`

**Given** the parent feature listens to the Tauri event `smartScan:progress`
**When** an event arrives
**Then** the component re-renders within the same frame (no debounce), so visual update lag ≤ 16.6 ms (NFR-P7)

**Given** the rule that loading state must never be a static spinner for > 500 ms (UX-DR10)
**When** Smart Scan starts and the first progress event has not arrived
**Then** the component shows an immediate indeterminate state with the stage label "Đang khởi động…" — never a bare spinner

**Given** the stage transitions through `sys` → `scan` → `clean_dry_run` → `health`
**When** each stage begins
**Then** the label updates to a Vietnamese-friendly stage name (e.g., "Đang quét hệ thống", "Đang quét file", "Đang thử dọn", "Đang tính điểm")

**Given** the `total` field is `0` (unknown at stage start)
**When** the component renders
**Then** the progress bar shows an indeterminate animation rather than 0%, and the label reads "{stage}: đang tính tổng…"

**Given** macOS Reduced Motion (NFR-A7)
**When** the user has Reduce Motion enabled
**Then** the progress bar uses opacity/color change instead of animated stripes (no motion > 200 ms)

### Story 2.6: Frontend `features/smart-scan/` Smart Scan button + landing layout

As a first-time user (Persona A),
I want to see a single, prominent "Smart Scan" button as the app's landing experience,
So that I can scan my system with zero decision overhead.

**Acceptance Criteria:**

**Given** `src/features/smart-scan/`
**When** inspected
**Then** it has `index.tsx` (default export), `use-smart-scan.ts` (mutation hook), and `smart-scan-button.tsx`

**Given** the user opens the app for the first time after FDA onboarding
**When** the main route renders
**Then** a single centered button labeled "Smart Scan" appears with no other primary actions competing for attention

**Given** the user clicks the Smart Scan button
**When** the click handler fires
**Then** it triggers `useSmartScan()`'s TanStack Query mutation which calls `smartScanRun()` from `lib/ipc.ts`

**Given** the mutation is pending
**When** the UI re-renders
**Then** the button is disabled, replaced by the `ProgressStream` panel (Story 2.5)

**Given** the mutation succeeds
**When** the UI re-renders
**Then** it shows the result page containing the `HealthScoreBadge` (Story 2.4), reclaimable bytes summary, and a "Tiếp tục dọn dẹp" CTA placeholder (wired in Epic 3)

**Given** the mutation fails with an `ErrorPayload`
**When** the UI re-renders
**Then** an error card shows `message` (Vietnamese-friendly) plus a "Thử lại" button — no app crash, no blank screen

**Given** the keyboard shortcut Cmd+S (per UX-DR13)
**When** the user presses it on the landing route
**Then** the Smart Scan mutation triggers (same as button click)

### Story 2.7: Health Score breakdown drill-down panel (UX-DR19 "Explain don't show")

As a user,
I want to click the Health Score and see a panel that explains which factors contributed how much,
So that I can answer "why 51?" without guessing.

**Acceptance Criteria:**

**Given** `src/features/smart-scan/health-breakdown-panel.tsx`
**When** inspected
**Then** it accepts `HealthBreakdown` and renders 4 rows: disk pressure, junk, memory pressure, quarantine age

**Given** each row
**When** rendered
**Then** it shows: (a) factor name in Vietnamese, (b) contribution % (e.g., "Đóng góp 30%"), (c) one-line explanation pulled from a static dictionary keyed by factor (e.g., "Ổ đĩa đầy 85% — dấu hiệu lớn nhất kéo điểm xuống"), (d) a horizontal mini-bar visualizing the contribution magnitude

**Given** the panel is opened by clicking the `HealthScoreBadge`
**When** the user clicks the badge
**Then** the panel slides/fades in below the badge using an inline drawer (NOT a modal — UX-DR14)

**Given** macOS Reduced Motion (NFR-A7)
**When** the user has Reduce Motion enabled
**Then** the panel appears instantly (no transition > 200 ms)

**Given** every numeric value must be explainable (UX-DR19)
**When** the panel renders
**Then** none of the 4 contributions appear as a bare number without text explanation — automated guard: `data-testid="health-factor-row"` must contain both a contribution value and a non-empty explanation node

**Given** keyboard accessibility (NFR-A1)
**When** the panel is open
**Then** focus is trapped to within the panel until closed, Esc closes the panel, and the badge regains focus on close

### Story 2.8: `EmptyState` first-launch + Smart Scan `EquivalentCLIFooter` feed

As a first-time user,
I want a clean empty state before any scan has run, and once on Smart Scan I want to see the equivalent CLI command,
So that the app feels uncluttered initially and transparent during use.

**Acceptance Criteria:**

**Given** `src/components/empty-state.tsx`
**When** inspected
**Then** it exports `export interface EmptyStateProps { title: string; description: string; action?: ReactNode; }` and renders a centered card

**Given** the user opens the app and no scan has been run yet
**When** `features/smart-scan/index.tsx` renders the pre-scan state
**Then** it composes `<EmptyState title="Chưa có lần quét nào" description="Bấm Smart Scan để bắt đầu" action={<SmartScanButton/>} />`

**Given** the rule that text must avoid marketing-speak (UX-DR18)
**When** the empty-state copy is inspected
**Then** it contains no words from a forbidden list: "boost", "supercharge", "blazingly fast", "AI-powered", "critical alert"

**Given** the global `EquivalentCLIFooter` shell slot from Story 1.12
**When** the user is on the Smart Scan view
**Then** the footer's `command` prop receives `"tiny scan && tiny clean --dry-run"` (the CLI equivalent of Smart Scan's read-only pass)

**Given** the user clicks Copy in the footer
**When** the clipboard is read
**Then** it contains exactly `"tiny scan && tiny clean --dry-run"` (no extra whitespace, no quotes)

**Given** the footer visibility defaults to ON (FR40 default; toggle wired in Epic 4)
**When** the user is on Smart Scan
**Then** the footer is visible at the bottom of the viewport

### Story 2.9: Smart Scan performance bench gate (NFR-P4)

As a maintainer,
I want a CI benchmark that asserts Smart Scan completes ≤ 5 s on a representative fixture and fails PRs that regress by > 20%,
So that NFR-P4 is enforced automatically before merge.

**Acceptance Criteria:**

**Given** `crates/tiny-core/benches/smart_scan_bench.rs`
**When** inspected
**Then** it defines a Criterion benchmark group named `smart_scan` with a single bench function calling `tiny_core::smart_scan()` over a fixed fixture directory

**Given** the fixture
**When** the bench setup runs
**Then** it generates a deterministic synthetic tree of ~50k files (or downloads a checked-in tarball) so bench timing is reproducible across runs

**Given** the bench runs in CI on `macos-latest` (Apple Silicon)
**When** the workflow completes
**Then** the published HTML report (Criterion artifact) shows median duration and the wall-clock P95 is ≤ 5 s on at least one of the last 3 runs

**Given** the regression gate from Story 1.10
**When** a PR causes the bench median to increase by > 20% versus the baseline stored in `crates/tiny-core/benches/baselines/smart_scan.json`
**Then** the CI job fails with a clear diff showing baseline vs new median

**Given** the baseline file
**When** a maintainer intentionally accepts a regression (e.g., trading speed for correctness)
**Then** they update `baselines/smart_scan.json` in the same PR and the gate compares against the new baseline going forward

**Given** the bench harness must not flake
**When** CI runs the bench 5 times in a row
**Then** the coefficient of variation across runs is < 10% (asserted by a follow-up post-process step)

---

## Epic 3: Cleanup Flow with Safety Net (3 Lanes + Preview + Quarantine + Undo + History)

User chọn category theo làn 3 màu (🟢 default ticked, 🟡/🔴 opt-in), preview file thật trước khi xoá (size/path/age + virtualization), execute hybrid delete (Trash/quarantine), xem diff report dạng `git diff`. Restore từng file qua "Cleanup History" panel ≤ 30 ngày HOẶC Cmd+Z undo lần dọn gần nhất. Auto-purge sau 30 ngày + notify khi > 10 GB. Mọi xoá an toàn (`fs_safe`, `symlink_metadata` canonicalize). App đang chạy → skip cache + surface skip reason.

### Story 3.1: SQLite schema + migrations (journal / quarantine / sessions)

As a backend developer,
I want a versioned SQLite schema with tables for cleanup sessions, journal entries, and quarantine records,
So that every cleanup action is transactional, undoable, and inspectable per the architecture's D4 decision.

**Acceptance Criteria:**

**Given** `src-tauri/src/db/schema.rs`
**When** inspected
**Then** it has a `pub fn migrate(conn: &Connection) -> rusqlite::Result<()>` that creates 3 tables via versioned migrations (`schema_version` table tracks applied migration ids)

**Given** the `cleanup_sessions` table
**When** inspected
**Then** it has columns: `id INTEGER PRIMARY KEY`, `started_at INTEGER NOT NULL` (Unix epoch), `finished_at INTEGER`, `total_bytes_freed INTEGER NOT NULL DEFAULT 0`, `status TEXT NOT NULL CHECK (status IN ('running','done','cancelled','failed'))`

**Given** the `cleanup_journal_entries` table
**When** inspected
**Then** it has columns: `id INTEGER PRIMARY KEY`, `session_id INTEGER NOT NULL REFERENCES cleanup_sessions(id) ON DELETE CASCADE`, `provider_id TEXT NOT NULL`, `original_path TEXT NOT NULL`, `bytes INTEGER NOT NULL`, `destination TEXT NOT NULL CHECK (destination IN ('trash','quarantine','cleared'))`, `created_at INTEGER NOT NULL`

**Given** the `quarantine_entries` table
**When** inspected
**Then** it has columns: `id INTEGER PRIMARY KEY`, `journal_id INTEGER NOT NULL REFERENCES cleanup_journal_entries(id) ON DELETE CASCADE`, `quarantine_path TEXT NOT NULL UNIQUE`, `expires_at INTEGER NOT NULL`, `purged INTEGER NOT NULL DEFAULT 0`

**Given** SQLite paths and times follow the architecture convention
**When** inspected
**Then** all timestamps store Unix epoch INTEGER (not ISO strings) and the DB file lives at `~/.tiny/journal.sqlite`

**Given** indices for query performance
**When** the migration runs
**Then** it creates: `idx_journal_session` on `cleanup_journal_entries(session_id)`, `idx_quarantine_expires` on `quarantine_entries(expires_at) WHERE purged = 0`

**Given** the migration runs twice
**When** the second run executes
**Then** it is a no-op — no errors, no duplicate rows in `schema_version`

### Story 3.2: Hybrid delete layer — Trash primary + quarantine fallback

As a backend developer,
I want `src-tauri/src/delete.rs` to encapsulate the hybrid delete policy (Trash via `trash` 5.2 → quarantine fallback) with disk-space pre-check and journal-first writes,
So that every cleanup operation is reversible and transactional per FR9 / NFR-R2 / NFR-R5.

**Acceptance Criteria:**

**Given** `src-tauri/src/delete.rs`
**When** inspected
**Then** it exposes `pub fn delete_with_safety_net(path: &Path, provider_id: &str, session_id: i64, conn: &Connection) -> Result<DeleteOutcome, DeleteError>` where `DeleteOutcome { destination: Destination, bytes_freed: u64 }` and `Destination ∈ { Trash, Quarantine }`

**Given** the architecture rule that no module outside `fs_safe` may call `fs::remove_*`
**When** I grep `delete.rs`
**Then** it contains 0 `fs::remove_file|remove_dir|remove_dir_all` calls — it uses `trash::delete()` and `std::fs::rename()` (atomic same-fs move for quarantine) only

**Given** `trash::delete()` returns Ok
**When** inspected
**Then** the journal entry is written with `destination = 'trash'` and NO `quarantine_entries` row is created

**Given** `trash::delete()` returns Err (e.g., source is on a non-supported filesystem)
**When** the fallback runs
**Then** the file is `std::fs::rename`-moved to `~/.tiny/quarantine/<session_id>/<uuid>/<basename>`, journal entry uses `destination = 'quarantine'`, and a row appears in `quarantine_entries` with `expires_at = now + 30 days * 86400`

**Given** the NFR-R2 transactional rule
**When** delete runs
**Then** the journal INSERT is committed BEFORE the `trash::delete()` / `fs::rename()` call — verified by integration test that kills the process mid-operation and confirms the journal row exists without an orphan file mismatch on next start

**Given** the NFR-R5 disk-space check
**When** the target operation is a quarantine move and free disk < 2× source size
**Then** `delete_with_safety_net` returns `DeleteError::InsufficientDiskSpace` BEFORE any file or row is touched

**Given** the symlink safety invariant (FR13)
**When** the function receives a path
**Then** it calls `symlink_metadata` first; if the path is a symlink it refuses with `DeleteError::SymlinkRefused` rather than following

### Story 3.3: `clean_execute` Tauri command + `clean:progress` + `clean:done` events

As a frontend developer,
I want a `clean_execute` Tauri command that runs the engine's clean across selected categories using the hybrid delete layer, emitting progress events,
So that the GUI can drive cleanup with realtime updates and a final structured report.

**Acceptance Criteria:**

**Given** `src-tauri/src/commands/clean.rs`
**When** inspected
**Then** it has `#[tauri::command] pub async fn clean_execute(app: AppHandle, opts: CleanExecuteOpts) -> Result<CleanReport, ErrorPayload>` invoked via `spawn_blocking`

**Given** `CleanExecuteOpts`
**When** inspected
**Then** it has `#[serde(rename_all = "camelCase")]` fields: `session_label: Option<String>`, `selections: Vec<CategorySelection>` where each `CategorySelection { provider_id, file_paths: Option<Vec<PathBuf>> }` (None ⇒ all files in category)

**Given** the command body
**When** it runs
**Then** it (1) opens a SQLite transaction, (2) inserts a new `cleanup_sessions` row with `status='running'`, (3) iterates selections delegating each file to `delete_with_safety_net`, (4) updates `total_bytes_freed`, (5) sets `status='done'` and `finished_at`

**Given** progress callback wiring
**When** each file is processed
**Then** Tauri event `"clean:progress"` is emitted with `{ sessionId, currentBytes, totalBytes, currentPath, providerId }` (camelCase)

**Given** the session completes
**When** the command returns
**Then** Tauri event `"clean:done"` is emitted with the final `CleanReport { sessionId, freedBytes, byCategory: [...], skipped: [...] }` and the same payload is returned via the promise

**Given** FR14 (skip running app) verified end-to-end
**When** a provider's `requires_app_quit()` is true and the app is running
**Then** the affected files appear in `skipped` with `reason: "app_running"` and are NOT counted in `freedBytes`

**Given** the user cancels mid-execution (a future `clean_cancel` command sets a flag)
**When** the engine's loop checks the flag
**Then** the session row updates to `status='cancelled'` within ≤ 1 s and the partial freed bytes are preserved

### Story 3.4: `cleanup_history_list` + `cleanup_history_restore` Tauri commands

As a frontend developer,
I want commands to list previous cleanup sessions and restore files (single file or whole session) from quarantine,
So that the GUI's Cleanup History panel and Cmd+Z undo can operate per FR10 / FR11.

**Acceptance Criteria:**

**Given** `src-tauri/src/commands/cleanup_history.rs`
**When** inspected
**Then** it has `#[tauri::command] pub async fn cleanup_history_list(limit: u32, offset: u32) -> Result<Vec<CleanupSessionSummary>, ErrorPayload>` returning sessions sorted `started_at DESC`

**Given** `CleanupSessionSummary` shape
**When** inspected
**Then** it has `#[serde(rename_all = "camelCase")]` fields: `session_id`, `started_at` (ISO 8601 string), `finished_at` (ISO 8601 or null), `status`, `total_bytes_freed`, `restorable_file_count`, `category_breakdown: Vec<{providerId, bytes, fileCount}>`

**Given** `cleanup_history_restore` command
**When** inspected
**Then** it has signature `pub async fn cleanup_history_restore(scope: RestoreScope) -> Result<RestoreReport, ErrorPayload>` where `RestoreScope ∈ { Session(i64), Entry(i64) }`

**Given** restoring a single quarantined file
**When** the command runs
**Then** it moves the file from quarantine back to `original_path`, marks the `quarantine_entries.purged = 1`, and inserts a new journal row with `destination = 'restored'` (added to the CHECK constraint in Story 3.1 if missing)

**Given** restoring an entire session
**When** the command runs
**Then** it iterates every quarantine entry linked to that session, attempts restore for each, and returns `RestoreReport { restored: Vec<Path>, failed: Vec<{ path, reason }> }`

**Given** files originally sent to macOS Trash (not quarantine)
**When** the user tries to restore
**Then** the command returns those entries in `failed` with `reason = "in_macos_trash_use_finder_put_back"` — never attempts to fish out of Trash

**Given** a restore destination already has a file at `original_path` (collision)
**When** the command runs
**Then** it renames the restored file to `<basename>.restored-<timestamp><ext>` and records the conflict in `RestoreReport.restored`

### Story 3.5: `SafetyLane` component (UX-DR1, UX-DR4)

As a frontend developer,
I want a reusable `SafetyLane` component that renders one of the 3 lanes (🟢 / 🟡 / 🔴) with categories as cards,
So that the cleanup view can compose a 3-column layout with consistent visual language and a11y semantics.

**Acceptance Criteria:**

**Given** `src/components/safety-lane.tsx`
**When** inspected
**Then** it exports `export interface SafetyLaneProps { lane: 'safe' | 'review' | 'destructive'; categories: CategoryWithSelection[]; onToggle: (id: string, checked: boolean) => void; }`

**Given** the lane renders
**When** displayed
**Then** the header shows: lane icon (✅ / ⚠️ / 🚨), Vietnamese label ("An toàn" / "Cần xem" / "Nguy hiểm"), total reclaimable bytes formatted human-readable per UX-DR15

**Given** color is not the sole signal (NFR-A5)
**When** the lane renders in a colorblind simulator
**Then** the icon + label remain distinguishable in protanopia/deuteranopia/tritanopia (verified visually + automated test via `@accessibility/contrast` against tokens)

**Given** each category card
**When** rendered
**Then** it has: checkbox, category name, reclaimable bytes (e.g., "8.2 GB"), an aria-describedby tooltip with the engine description

**Given** the default selection state per lane
**When** the safe lane initializes
**Then** all categories there are pre-ticked (FR2); for `review` and `destructive` lanes nothing is pre-ticked

**Given** keyboard navigation (NFR-A1)
**When** the user tabs through a lane
**Then** focus order is header → first card checkbox → next card checkbox; Space toggles, Enter on a card label emits an expand event (the consuming view wires the drill-down panel)

**Given** Vitest tests
**When** they run
**Then** they assert (a) safe lane defaults all checkboxes to true, (b) onToggle fires with correct id and new state, (c) lane header reflects sum of selected reclaimable bytes

### Story 3.6: `CategoryDrillDown` component (UX-DR5) — virtualized file preview

As a user,
I want to expand any category and see the actual files (with size, path, last-access date) that will be deleted, with checkboxes to deselect specific ones,
So that I can verify before clicking Clean (Persona A "lòng tin tăng" moment).

**Acceptance Criteria:**

**Given** `src/components/category-drill-down.tsx`
**When** inspected
**Then** it accepts `CategoryDrillDownProps { providerId: string; files: FilePreview[]; onFileToggle: (path, checked) => void }` and renders an inline drawer (NOT a modal per UX-DR14)

**Given** a category with ≥ 200 files
**When** the drawer opens
**Then** the file list is virtualized (e.g., via `react-window` or `@tanstack/react-virtual`) — only the visible rows are mounted, scroll position is preserved on re-open

**Given** each file row
**When** rendered
**Then** it shows: checkbox (default checked), basename, full path (truncated middle with `…` if long), size formatted human-readable, last access as "3 ngày trước" with hover ISO 8601 tooltip (UX-DR16)

**Given** the user unticks a file
**When** the parent receives `onFileToggle`
**Then** the consuming view's selection store removes that file path from the category's selection (immutable update); a parent may surface an `--exclude=` flag if non-empty (the component itself only exposes the callback)

**Given** the drawer is open
**When** the user presses Esc
**Then** the drawer collapses, focus returns to the category's checkbox

**Given** reduced motion (NFR-A7)
**When** Reduce Motion is on
**Then** the drawer opens without slide animation (instant or fade ≤ 200 ms)

**Given** Vitest test
**When** it runs
**Then** it asserts (a) virtualization is enabled (assert only visible rows are in DOM for a 500-row mock), (b) toggling a checkbox fires `onFileToggle` once

### Story 3.7: `DestructiveConfirmDialog` component (UX-DR9, FR63)

As a user,
I want a clear, large-text confirmation modal before any destructive action,
So that I cannot accidentally delete files without explicit acknowledgement (Persona C requirement).

**Acceptance Criteria:**

**Given** `src/components/destructive-confirm-dialog.tsx`
**When** inspected
**Then** it exports `DestructiveConfirmDialogProps { open: boolean; title: string; bodyText: string; confirmLabel: string; cancelLabel: string; onConfirm: () => void; onCancel: () => void; }`

**Given** the rule that text must be readable (NFR-A6)
**When** rendered
**Then** all text is ≥ 14 pt, the modal box has minimum 360 × 200 px footprint, contrast ratio ≥ 4.5:1

**Given** UX-DR17 action verb rule
**When** consumers pass labels
**Then** the rendered confirm button text is concrete (e.g., "Đưa vào quarantine 30 ngày", "Xoá vào Trash") and is NOT one of the forbidden generic strings: "OK", "Cancel" (enforced via a runtime warning in dev mode)

**Given** keyboard handling
**When** the modal opens
**Then** focus moves to the **Cancel** button (default safe action), Esc fires `onCancel`, Enter fires `onConfirm` only if confirm button is focused

**Given** focus trap
**When** open
**Then** tabbing cycles within the modal; the rest of the page receives `aria-hidden="true"` and is not focusable

**Given** the modal closes
**When** `open` transitions false
**Then** focus returns to the element that opened it (saved on `open` rising edge)

**Given** Reduced Motion (NFR-A7)
**When** enabled
**Then** the modal appears without scale/slide animation

### Story 3.8: `features/cleanup/` main view — 3-lane layout with default tick 🟢

As a user,
I want to see my Smart Scan results laid out as 3 safety lanes with 🟢 ticked by default and 🟡/🔴 requiring my explicit opt-in,
So that I can review what will be cleaned without risking destructive deletes by accident.

**Acceptance Criteria:**

**Given** `src/features/cleanup/index.tsx`
**When** the user lands here after Smart Scan
**Then** the page shows 3 columns rendered by 3 `SafetyLane` components (Story 3.5; safe / review / destructive), categorized by the `family` field from `SmartScanReport.categories`

**Given** the default selection state per FR2
**When** the page first renders
**Then** 🟢 categories are pre-ticked, 🟡 and 🔴 are unticked, and a global "Bao gồm làn 🟡 / 🔴" toggle (Settings or inline) controls visibility of opt-in lanes (default hidden until user expands)

**Given** the user expands a category
**When** the drawer opens
**Then** it embeds the `CategoryDrillDown` component (Story 3.6) for per-file preview and selection

**Given** the user clicks "Clean" without expanding 🟡 or 🔴
**When** the action fires
**Then** only 🟢 selections are sent to `clean_execute` and the `EquivalentCLIFooter` shows e.g. `tiny clean --category=npm_cache,docker_overlay,xcode_derived_data` (no `--include-destructive` flag)

**Given** the user expands 🔴 and ticks a category
**When** they click "Clean"
**Then** the `DestructiveConfirmDialog` (Story 3.7) must confirm before `clean_execute` is invoked

**Given** the `EquivalentCLIFooter` shell from Story 1.12
**When** selections change
**Then** the footer's `command` prop updates live with the equivalent `tiny clean --category=...` (FR39)

**Given** Zustand store for selections
**When** the user toggles a checkbox
**Then** the store updates immutably (not mutating the existing array), per coding-style rule

**Given** the user has zero items selected
**When** the page renders
**Then** the "Clean" button is disabled with `aria-disabled="true"` and a tooltip "Chọn ít nhất 1 mục"

### Story 3.9: `CleanupDiffReport` component (UX-DR8) — diff-style result

As a user,
I want the cleanup result displayed in a `git diff` style format showing freed total + per-category breakdown with destinations,
So that I can immediately see what changed, where files went, and trust the safety net.

**Acceptance Criteria:**

**Given** `src/components/cleanup-diff-report.tsx`
**When** inspected
**Then** it accepts `CleanReport` and renders: top line `+ {humanBytes} freed` in bold green, followed by per-category rows

**Given** each per-category row
**When** rendered
**Then** it shows: provider id (e.g., `xcode_derived_data`), bytes freed (e.g., `-8.2 GB`), destination icon (→ quarantine / → trash / → cleared), and file count (`1,247 files`)

**Given** any files were skipped due to FR14 (app running)
**When** the report renders
**Then** a "Đã bỏ qua" section appears listing skip reasons grouped by provider (e.g., `safari_cache: app đang chạy`)

**Given** the user clicks any category row
**When** the row expands
**Then** it lists the actual journal entries with original paths (read from the Cleanup History API of Story 3.4)

**Given** UX-DR18 tone rule
**When** any text in the report is rendered
**Then** none of it contains marketing-speak ("boosted", "supercharged"); only neutral verbs ("đã dọn", "đã giải phóng", "đã đưa vào")

**Given** the report exposes an undo affordance
**When** rendered
**Then** it shows a "Hoàn tác (Cmd+Z)" button whose handler is provided by the parent via an `onUndo` callback prop (the actual undo logic is implemented in the History/undo story later in this epic, which wires this callback — this component has no dependency on that implementation)

**Given** the `EquivalentCLIFooter` while on the report view
**When** inspected
**Then** the footer shows the actual CLI used (e.g., `tiny clean --category=npm_cache,xcode_derived_data --output=json`)

### Story 3.10: Cmd+Z undo + `CleanupHistory` panel (FR10, FR11)

As a user,
I want to undo my last cleanup with Cmd+Z and to browse / selectively restore from a Cleanup History panel,
So that mistakes are recoverable (Persona C "ảnh cháu quay lại Desktop" moment).

**Acceptance Criteria:**

**Given** `src/features/cleanup/use-undo-handler.ts`
**When** registered globally on the cleanup feature
**Then** pressing Cmd+Z anywhere inside the feature triggers `cleanup_history_restore({ scope: 'session', sessionId: lastSessionId })`

**Given** an undo is in flight
**When** the UI re-renders
**Then** a toast "Đang hoàn tác lần dọn gần nhất…" appears and Cmd+Z is debounced to prevent double-fire within 2 s

**Given** the undo finishes
**When** the `RestoreReport` returns
**Then** a toast shows `Đã khôi phục {restoredCount} file. {failedCount} file gặp lỗi.` with a "Xem chi tiết" link

**Given** `src/features/cleanup/history-panel.tsx`
**When** the user opens "Lịch sử dọn dẹp" from the side nav
**Then** the panel calls `cleanup_history_list({ limit: 30, offset: 0 })` and displays sessions newest-first

**Given** each session row in the history panel
**When** rendered
**Then** it shows: started_at as human-readable ("hôm qua, 14:23"), total bytes freed, category breakdown, "Hoàn tác toàn bộ" button (restores whole session) and "Xem file" link (expands per-file restore list)

**Given** the user expands a session
**When** the per-file list loads
**Then** each file row has its own "Hoàn tác file này" button which calls `cleanup_history_restore({ scope: 'entry', entryId })`

**Given** items in macOS Trash (destination = `trash`)
**When** the user clicks "Hoàn tác"
**Then** instead of restoring, the UI shows a hint: "File ở Trash macOS — mở Finder → Trash → Put Back" with a button to reveal Trash

### Story 3.11: Auto-purge background task + 10 GB notification (FR12)

As a maintainer,
I want a background task that purges quarantine entries older than 30 days and notifies the user when total quarantine size exceeds 10 GB,
So that quarantine doesn't grow unbounded and the user is alerted before it becomes a problem.

**Acceptance Criteria:**

**Given** `src-tauri/src/quarantine/auto_purge.rs`
**When** inspected
**Then** it has `pub fn start_auto_purge(app: AppHandle, conn: Pool<...>) -> JoinHandle<()>` spawning a `tokio::spawn` interval task running every 1 hour

**Given** the task tick runs
**When** it queries quarantine entries
**Then** it selects `WHERE expires_at <= now AND purged = 0`, deletes each backing file via `fs_safe::remove_recursive_safe`, sets `purged = 1`

**Given** any purge step fails
**When** the task handles the error
**Then** it logs at WARN with the path + reason via `tracing`, retries up to 3 times across subsequent ticks (NFR-R3), and never panics the task

**Given** total live quarantine size > 10 GB
**When** the task computes the sum
**Then** it emits a single Tauri event `"quarantine:sizeWarning"` once per 24 h with payload `{ totalBytes, oldestExpiresAt }`; the frontend converts this into a macOS notification via `tauri-plugin-notification`

**Given** the user has zero opt-in for crash report (NFR-Pr2), this task must NOT phone home
**When** logging
**Then** all log lines stay local; no network call is initiated by the auto-purge code path

**Given** app crash mid-purge (NFR-R2)
**When** the app restarts
**Then** any half-purged entries (file deleted but row not flipped) are reconciled on next tick — the task treats "row says not purged but file missing" as already purged and updates the row

### Story 3.12: Symlink safety verification — per-provider tests (FR13 architectural)

As a maintainer,
I want a CI integration test that asserts every cleanup provider uses `symlink_metadata` and refuses symlink traversal,
So that the FR13 safety invariant cannot regress unnoticed.

**Acceptance Criteria:**

**Given** `crates/tiny-core/tests/symlink_safety.rs`
**When** inspected
**Then** it iterates `all_providers()` and for each provider sets up a tmp dir containing: a real file, a symlink pointing OUTSIDE the tmp dir (to a sentinel file), then invokes the provider's dry-run scan against the tmp dir

**Given** the assertion per provider
**When** the test runs
**Then** the provider's planned actions must NOT include the sentinel file path — verified by comparing planned-paths against a canonicalized whitelist

**Given** the test also exercises hybrid delete
**When** `delete_with_safety_net` is given the symlink path directly
**Then** it returns `DeleteError::SymlinkRefused` and the sentinel file remains untouched on disk

**Given** the registry-sync test from Story 1.3
**When** the symlink test discovers a provider id not in `category_family()`
**Then** it fails with a clear message naming the missing id (defensive coupling with FR6)

**Given** CI runs this test
**When** it executes on `macos-latest`
**Then** it completes in ≤ 5 s and 0 sentinel files are touched across all providers

### Story 3.13: Cleanup performance bench gates (NFR-P5, NFR-P6)

As a maintainer,
I want CI benches that assert `clean --dry-run` ≤ 1 s P95 and `clean execute` ≤ 5 s P95 per category,
So that NFR-P5 and NFR-P6 are enforced automatically.

**Acceptance Criteria:**

**Given** `crates/tiny-core/benches/clean_dry_run_bench.rs`
**When** it runs
**Then** it benches `dry_run` for each of `xcode_derived_data`, `npm_cache`, `docker_overlay` (representative MVP providers) using a checked-in fixture tree

**Given** `crates/tiny-core/benches/clean_execute_bench.rs`
**When** it runs
**Then** it benches `delete_with_safety_net` for the same providers using a generated fixture with ~1k files, restoring the fixture between runs

**Given** the CI bench job
**When** results are computed
**Then** P95 for dry-run is ≤ 1 s and P95 for execute is ≤ 5 s per category; the job fails if any provider exceeds

**Given** baselines from Story 1.10
**When** a PR increases median by > 20%
**Then** CI fails with a diff showing baseline vs new median

**Given** the benches must not flake
**When** CI runs them 5 times in a row
**Then** coefficient of variation < 10% per provider

**Given** the bench artifact
**When** CI uploads
**Then** Criterion HTML report is published as a workflow artifact for inspection

---

## Epic 4: Settings, Preferences & Accessibility Foundation

User customize quarantine retention (7–90 days), auto-update behavior (check/download/install độc lập), opt-in crash report (default OFF), clear journal + quarantine. Accessibility baseline: dark/light theme parity, FocusIndicator visible (≥ 2px outline contrast ≥ 3:1), full keyboard navigation, 200% font scaling, WCAG AA contrast, VoiceOver compatible, Reduced Motion respect. CLI footer toggle (default ON) sống ở Settings.

### Story 4.1: Design tokens — palette + typography + spacing scale

As a frontend developer,
I want a consolidated set of Tailwind design tokens covering the 3-lane palette, neutral colors, typography scale, and spacing,
So that every component in the app uses a consistent visual language and contrast/scaling can be enforced systemically.

**Acceptance Criteria:**

**Given** `tailwind.config.ts`
**When** inspected
**Then** the `theme.extend.colors` section defines: `lane.safe` (green token + variants for bg/text/border), `lane.review` (amber), `lane.destructive` (red), `neutral` palette (50–950), and semantic aliases (`text-primary`, `text-secondary`, `surface-base`, `surface-elevated`)

**Given** the typography scale
**When** inspected
**Then** `theme.extend.fontSize` defines named steps with explicit pixel sizes: `caption: 12px`, `body: 14px`, `body-lg: 16px`, `subtitle: 18px`, `title: 22px`, `display: 28px` (14px is the minimum body per NFR-A6)

**Given** the spacing scale
**When** inspected
**Then** `theme.extend.spacing` covers `0` through `32` with a `4 px` base unit consistent with the rest of the scale

**Given** UX-DR1 — color is not the only signal
**When** lane tokens are documented in `src/lib/tokens.md` (or JSDoc-style header)
**Then** each lane token has a documented icon + Vietnamese label pairing: `lane.safe → ✅ "An toàn"`, `lane.review → ⚠️ "Cần xem"`, `lane.destructive → 🚨 "Nguy hiểm"`

**Given** Vitest snapshot or jest-extended test `src/lib/tokens.test.ts`
**When** run
**Then** it asserts each lane token resolves to a valid Tailwind color and the pairing dictionary is non-empty for all three lanes

**Given** the `SafetyLane` component from Story 3.5
**When** it now consumes these tokens
**Then** existing tests pass without visual regression (manual snapshot diff acceptable for this story; automated visual regression deferred)

### Story 4.2: Dark/light theme parity + auto-detect macOS appearance + manual override

As a user,
I want the app to follow my macOS appearance by default and to let me override it manually in Settings,
So that the UI matches my system preference and respects my deliberate choice when I make one.

**Acceptance Criteria:**

**Given** `src/lib/theme.ts`
**When** inspected
**Then** it exports `type Theme = 'light' | 'dark' | 'system'` and `useTheme()` returns the active theme considering: (1) settings store override, (2) `window.matchMedia('(prefers-color-scheme: dark)')` when override is `'system'`

**Given** the app boots
**When** the theme resolves
**Then** `<html data-theme="...">` reflects the resolved theme and Tailwind dark-mode utilities (`dark:`) apply correctly

**Given** the user changes macOS Appearance while the app is open (and override is `system`)
**When** the `prefers-color-scheme` media query fires
**Then** `data-theme` updates within ≤ 200 ms without an app restart

**Given** the Settings → Appearance section
**When** the user selects "Sáng" / "Tối" / "Theo hệ thống"
**Then** the choice persists via `tauri-plugin-store` and survives app restart

**Given** the lane tokens from Story 4.1 in dark mode
**When** rendered
**Then** they meet WCAG AA contrast against `surface-base.dark` (≥ 4.5:1 for body text, ≥ 3:1 for large) — verified by the axe-core contrast gate later in this epic

**Given** Reduced Motion is on
**When** the theme switches (manual override)
**Then** the transition is instant (no cross-fade animation > 200 ms)

### Story 4.3: Settings panel scaffold (`features/settings/`)

As a frontend developer,
I want a `features/settings/` module with a sidebar navigation and routed sub-pages,
So that every per-setting story can plug into a consistent host without reinventing layout.

**Acceptance Criteria:**

**Given** `src/features/settings/`
**When** inspected
**Then** it contains `index.tsx` (default export with sidebar layout), `sections/` (one file per section), and `use-settings-store.ts` (Zustand store wrapping `tauri-plugin-store` reads/writes)

**Given** the sidebar nav
**When** rendered
**Then** it lists sections in Vietnamese: "Lưu trữ" (Storage), "Cập nhật" (Updates), "Quyền riêng tư" (Privacy), "Giao diện" (Appearance), "Nâng cao" (Advanced)

**Given** the user opens Settings via `Cmd+,` (UX-DR13)
**When** the keyboard handler fires
**Then** the Settings route opens with the first section (Lưu trữ) selected by default

**Given** `tauri-plugin-store` is initialized
**When** the app boots
**Then** the store file lives at `~/Library/Application Support/com.tinycli.app/settings.json` (or Tauri's default), and the Zustand store hydrates from it before the first render

**Given** any setting write
**When** the user changes a value
**Then** the store updates immutably (new object) and `tauri-plugin-store` persists asynchronously without blocking the UI

**Given** the architecture invariant `lib/ → ...` (no reverse imports)
**When** `use-settings-store.ts` is inspected
**Then** the Zustand store lives in `features/settings/` (not `lib/`) because it pulls in feature-specific schema; only pure helpers (e.g., `parse-version`) belong in `lib/`

**Given** Vitest test
**When** it runs
**Then** it asserts: (a) store hydrates from a mock plugin-store backend, (b) writing a setting fires the persist call, (c) reading after persist returns the new value

### Story 4.4: Quarantine retention setting (FR52)

As a user,
I want to configure how long quarantined files are kept (7–90 days, default 30),
So that I can match the safety net to my workflow without being forced into the default.

**Acceptance Criteria:**

**Given** the Settings → Lưu trữ section
**When** rendered
**Then** it shows a labeled slider "Thời gian giữ quarantine" with range 7–90 days, step 1, default 30, current value displayed beside the slider

**Given** the user moves the slider
**When** they release it
**Then** the new value persists immediately and a toast confirms "Đã lưu — quarantine giữ trong {n} ngày"

**Given** the auto-purge background task from Story 3.11
**When** the retention value changes
**Then** the next tick reads the new value when computing `expires_at`-based selects (existing `quarantine_entries.expires_at` rows are NOT retroactively rewritten — old policy applies until natural expiration)

**Given** the user enters an out-of-range value via keyboard
**When** they type `5`
**Then** the slider clamps to `7` and shows an inline warning "Tối thiểu 7 ngày"

**Given** keyboard accessibility
**When** the slider has focus
**Then** Arrow keys change by 1 day, Shift+Arrow by 7 days, Home/End jump to 7/90

**Given** the `EquivalentCLIFooter` while on this settings page
**When** the value changes
**Then** the footer shows `tiny settings set quarantine.retention_days=<n>` as the equivalent (the CLI subcommand may not yet exist; Story marks it as forward-declared so it appears in copy-paste even before CLI parity is implemented for this setting)

### Story 4.5: Auto-update behavior setting (FR53)

As a user,
I want three independent toggles for update check / download / install behavior,
So that I can control how aggressively the app updates without an all-or-nothing choice.

**Acceptance Criteria:**

**Given** the Settings → Cập nhật section
**When** rendered
**Then** it shows 3 toggles: "Tự động kiểm tra cập nhật" (default ON), "Tự động tải cập nhật" (default OFF), "Tự động cài cập nhật" (default OFF)

**Given** the toggles have a logical dependency (cannot download without check, cannot install without download)
**When** the user disables "Tự động kiểm tra cập nhật"
**Then** the download and install toggles become disabled with `aria-disabled="true"` and a tooltip "Cần bật kiểm tra trước"

**Given** the user changes any toggle
**When** the change persists
**Then** the Tauri updater plugin reads the new value on its next scheduled run (no app restart needed)

**Given** the "Auto-install" toggle is ON
**When** the design rule from PRD: "luôn cần user confirm trước khi install" (PRD update strategy)
**Then** even with the toggle on, the installer still prompts a final confirm dialog before applying — verified in code path

**Given** the `EquivalentCLIFooter`
**When** any toggle changes
**Then** the footer shows the equivalent set command (forward-declared, e.g., `tiny settings set update.auto_check=true`)

**Given** Vitest test
**When** it runs
**Then** it asserts (a) defaults match FR53, (b) disabling auto-check cascades to disable other two, (c) toggle persistence round-trips correctly

### Story 4.6: Crash report opt-in (FR54)

As a user,
I want crash reporting OFF by default and to require an explicit confirmation when I enable it,
So that no data leaves my machine without my deliberate consent (NFR-Pr2).

**Acceptance Criteria:**

**Given** the Settings → Quyền riêng tư section
**When** rendered
**Then** it shows a toggle "Gửi crash report ẩn danh" with default OFF and a description listing exactly what is sent (stack trace + macOS version + app version, never path/hostname/IP per NFR-Pr3)

**Given** the user attempts to enable the toggle
**When** they click ON
**Then** before the value flips, a `DestructiveConfirmDialog` (or similar explicit dialog) appears with title "Bật crash report?", body listing what is sent and what is NOT sent, and a "Xác nhận bật" / "Hủy" pair (no "OK"/"Cancel")

**Given** the user clicks "Xác nhận bật"
**When** the confirm fires
**Then** the toggle moves to ON and persists

**Given** the user clicks "Hủy" or dismisses the dialog
**When** the dialog closes
**Then** the toggle stays OFF (no silent toggle)

**Given** the toggle is OFF
**When** the app crashes
**Then** no network call is initiated by the crash handler (verified by integration test using a mocked network layer)

**Given** the toggle is ON
**When** the app crashes
**Then** the report payload is verified by an integration test to contain only fields: `stack_trace`, `macos_version`, `app_version`, `tiny_core_version`, `occurred_at` — no `path`, `hostname`, `user_name`, `ip`

### Story 4.7: Clear journal + quarantine action (FR55)

As a user,
I want a single action that clears all local cleanup history and quarantined files entirely,
So that I can wipe state without resorting to manual file deletion.

**Acceptance Criteria:**

**Given** the Settings → Quyền riêng tư section
**When** rendered
**Then** it shows a labeled destructive button "Xoá toàn bộ lịch sử dọn dẹp và quarantine" with explanatory text

**Given** the user clicks the button
**When** the handler fires
**Then** the `DestructiveConfirmDialog` from Story 3.8 appears with: title "Xoá toàn bộ?", body warning that all quarantined files will be permanently deleted (not undoable), confirm label "Xoá vĩnh viễn", cancel label "Hủy"

**Given** the user confirms
**When** the Tauri command `clear_journal_and_quarantine` runs
**Then** it deletes all rows from `cleanup_sessions`, `cleanup_journal_entries`, `quarantine_entries` (CASCADE handles links), and removes the `~/.tiny/quarantine/` directory contents via `fs_safe::remove_recursive_safe`

**Given** the command succeeds
**When** the UI re-renders
**Then** the History panel (Story 3.10) is empty and a toast confirms "Đã xoá toàn bộ lịch sử"

**Given** the operation fails partway (e.g., a file is locked)
**When** the command returns
**Then** it returns `ErrorPayload` listing what was cleared and what failed, and the UI shows a recoverable error card (not a blank screen)

**Given** auto-purge background task from Story 3.11
**When** clear runs concurrently
**Then** they are serialized via a mutex/file-lock so journal rows are not deleted out from under an in-flight purge (no double-delete error)

### Story 4.8: CLI footer visibility toggle (FR40)

As a user,
I want to toggle the equivalent-CLI footer on/off in Settings,
So that I can hide it if the transparency value is not relevant to my workflow.

**Acceptance Criteria:**

**Given** the Settings → Giao diện section
**When** rendered
**Then** it shows a toggle "Hiện footer CLI tương đương" with default ON (FR40 default), and a short explanation "Mỗi action GUI sẽ kèm command `tiny ...` tương đương"

**Given** the user flips the toggle OFF
**When** the setting persists
**Then** `EquivalentCLIFooter`'s `visible` prop (from Story 1.12) becomes `false` across all features, hiding the footer immediately

**Given** the toggle is OFF
**When** the user re-enters any feature that previously fed a command
**Then** no footer renders — main content uses the full viewport height with no spacer

**Given** the toggle flips back to ON
**When** the UI re-renders
**Then** the footer reappears with the current view's command (no stale state)

**Given** Vitest test
**When** it runs
**Then** it asserts the toggle drives the `visible` prop in `EquivalentCLIFooter` via a Zustand selector

### Story 4.9: `FocusIndicator` baseline + keyboard navigation audit

As a power user (Persona B),
I want every interactive element to show a clear focus outline and to be reachable by Tab/Shift+Tab,
So that I can drive the app fully from the keyboard without ever touching the mouse.

**Acceptance Criteria:**

**Given** a global stylesheet (Tailwind plugin or `src/index.css`)
**When** inspected
**Then** it defines `:focus-visible` styles that render an outline ≥ 2 px with contrast ≥ 3:1 against both `surface-base.light` and `surface-base.dark` per NFR-A1

**Given** the focus style uses tokens from Story 4.1
**When** inspected
**Then** the focus ring color is a semantic token (e.g., `--focus-ring`) that meets the contrast requirement in both themes (verified by the axe-core contrast gate later in this epic)

**Given** a Vitest e2e test (`focus-audit.test.tsx` or equivalent using Playwright if added)
**When** it walks `Tab` through the main routes (Smart Scan, Cleanup, Settings, Cleanup History)
**Then** every interactive element is reached, every focus state is visually distinct, no element is unreachable

**Given** non-interactive containers
**When** inspected
**Then** they do NOT have `tabIndex={0}` (no extra noise in the tab order)

**Given** UX-DR13 keyboard shortcuts
**When** the audit runs
**Then** all listed shortcuts (Cmd+S, Cmd+Z, Cmd+, `⌫`, Tab/Shift+Tab, Space) are wired and do not conflict with macOS reserved shortcuts

### Story 4.10: Font scaling 200% responsive

As a user with macOS "Larger Text" preference (NFR-A3),
I want the app's UI to remain usable when system font scale is at 200%,
So that the app is accessible to low-vision users (Persona C).

**Acceptance Criteria:**

**Given** the app's root font-size uses `rem` units throughout
**When** inspected
**Then** no font-size in components is in `px` (except deliberately fixed elements like icons; tracked via ESLint plugin or grep gate)

**Given** the user enables macOS "Larger Text" at 200%
**When** the app reloads (Tauri picks up the WebKit scaling)
**Then** all interactive elements remain reachable; no text is clipped or covered by neighboring elements

**Given** the SafetyLane and CategoryDrillDown components
**When** rendered at 200%
**Then** category cards reflow vertically rather than overflowing horizontally; the file list scroll still works

**Given** the DestructiveConfirmDialog (Story 3.8)
**When** rendered at 200%
**Then** the modal grows to fit, buttons remain large + clearly labeled, no scroll inside the modal

**Given** a Vitest visual smoke test (CSS pixel snapshot at 200% scale)
**When** it runs against the main route
**Then** no element has `position: absolute` with hard pixel offsets that would break (encoded as a regression guard)

### Story 4.11: WCAG AA contrast audit + axe-core CI gate

As a maintainer,
I want axe-core run on every route in CI and to fail the build on any contrast/a11y violation,
So that the FR62 / NFR-A2 contrast invariant cannot regress unnoticed (folds in color-blindness check from UX-DR22).

**Acceptance Criteria:**

**Given** `package.json`
**When** inspected
**Then** it includes `@axe-core/playwright` (or `axe-core` + a Vitest-compatible runner)

**Given** a Vitest e2e suite `src/tests/a11y.test.tsx` (or Playwright suite)
**When** it runs
**Then** it loads each top-level route (Smart Scan, Cleanup, Cleanup History, Settings — all sections) in both light and dark themes and runs axe-core against each

**Given** the axe-core ruleset
**When** evaluated
**Then** it includes the `wcag2aa` + `wcag21aa` tag, with severities Critical and Serious failing the test

**Given** lane tokens (🟢 / 🟡 / 🔴)
**When** simulated in protanopia, deuteranopia, and tritanopia (e.g., via `color-blind` npm package in test)
**Then** the lane icon + label remain distinguishable; no two lanes are visually identical (covers UX-DR22)

**Given** CI is wired
**When** the a11y suite fails
**Then** PR cannot merge until the violation is resolved or explicitly waived in `a11y-waivers.json` with justification

**Given** a known-bad commit (artificially low contrast color)
**When** CI runs
**Then** the test fails with a clear axe-core report naming the element and the contrast ratio (must catch the regression)

### Story 4.12: ARIA landmarks + VoiceOver smoke test

As a screen-reader user,
I want the app to expose proper landmark regions and labeled interactive elements,
So that VoiceOver can navigate the app meaningfully (NFR-A4 / UX-DR20).

**Acceptance Criteria:**

**Given** the app shell layout
**When** inspected
**Then** the DOM contains exactly one `<main>`, one `<nav>` (primary nav / sidebar), and `<aside>` for ancillary panels (History, Settings sidebar) — no extra unlabeled landmark regions

**Given** every `<button>` and interactive element
**When** inspected
**Then** it either has visible text label OR an `aria-label` attribute with a meaningful Vietnamese phrase (no "Click here", no empty labels)

**Given** the Health Score Badge (Story 2.5)
**When** inspected by VoiceOver
**Then** the announced label is "Health Score 51 trên 100, cần dọn" (concrete narration, not just "button")

**Given** the SafetyLane (Story 3.5)
**When** focused
**Then** the lane header is announced as a region (e.g., `role="region"` + `aria-labelledby`) so VoiceOver users can jump between lanes

**Given** the CategoryDrillDown (Story 3.7)
**When** opened
**Then** the drawer has `role="region"` + `aria-labelledby` pointing to the category name; closing returns focus to the lane card

**Given** a Vitest test using `@testing-library/jest-dom` matchers
**When** it asserts landmark presence
**Then** it covers: `getByRole('main')`, `getByRole('navigation')`, `getAllByRole('region')` — failing the build on missing landmarks

### Story 4.13: Reduced Motion global respect

As a user with macOS "Reduce Motion" enabled,
I want all animations in the app to be replaced by instant transitions or fades ≤ 200 ms,
So that motion does not cause discomfort (NFR-A7 / UX-DR21).

**Acceptance Criteria:**

**Given** a global CSS rule
**When** inspected
**Then** `@media (prefers-reduced-motion: reduce) { *, *::before, *::after { animation-duration: 0s !important; transition-duration: 0s !important; } }` is present (with allowed exceptions ≤ 200 ms enumerated explicitly)

**Given** components that previously had > 200 ms animations
**When** Reduce Motion is on
**Then** they degrade gracefully: `ProgressStream` uses opacity change instead of stripes (already required in Story 2.6); CategoryDrillDown opens instantly (already in Story 3.7); DestructiveConfirmDialog appears without scale animation

**Given** the `useReducedMotion()` hook
**When** inspected
**Then** it returns the current `prefers-reduced-motion` state via `window.matchMedia` and updates on change

**Given** components that rely on animation for meaning (e.g., focus ring transitions)
**When** Reduce Motion is on
**Then** they replace transition with instant state (no ambiguity about whether the change happened)

**Given** Vitest test that mocks `matchMedia`
**When** Reduce Motion is on
**Then** snapshot tests for animated components show the reduced-motion render path

---

## Epic 5: Distribution & Update Pipeline

User `brew install --cask tiny` hoặc download `.dmg` notarized từ GitHub Releases với SHA256 checksum + GPG signature. App auto-check update qua GitHub Releases API (max 24h, configurable), verify signature `.dmg` bằng public key embed TRƯỚC khi prompt install. Homebrew Cask formula auto-update khi GitHub Release mới publish.

### Story 5.1: Tauri bundler config — `.app` + `.dmg` cho macOS

As a maintainer,
I want `tauri.conf.json` configured to produce a macOS `.app` and `.dmg` with correct identifier, icons, and metadata,
So that the local `tauri build` command produces a release-quality artifact even before CI signing is wired.

**Acceptance Criteria:**

**Given** `src-tauri/tauri.conf.json`
**When** inspected
**Then** `identifier` is `com.tinycli.app`, `productName` is `tiny`, `version` is read from `Cargo.toml` (or hard-coded matching it), `bundle.targets` includes `dmg`

**Given** `src-tauri/icons/`
**When** inspected
**Then** it contains the required icon set (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico` — the last is harmless on macOS, retained for future cross-platform) generated by `tauri icon`

**Given** `npm run tauri build` runs locally on macOS
**When** the build completes
**Then** `src-tauri/target/release/bundle/dmg/tiny_*.dmg` exists and double-clicking it mounts a volume where dragging the app to `/Applications` installs it

**Given** the `.dmg`'s embedded `.app/Contents/Info.plist`
**When** inspected
**Then** `CFBundleIdentifier == com.tinycli.app`, `CFBundleShortVersionString` matches `Cargo.toml` version, `LSMinimumSystemVersion >= 13.0` (NFR-C1)

**Given** the bundle size requirement (NFR-C3 ≤ 15 MB ideal, ≤ 25 MB cap)
**When** `du -sh tiny_*.dmg` runs after build
**Then** size is ≤ 25 MB; if > 15 MB an issue is logged with bundle-size budget breakdown for follow-up

**Given** the architecture rule about not bundling unnecessary runtimes
**When** the `.app/Contents/MacOS/` is inspected
**Then** no Electron framework, no Node runtime, only the Tauri binary + WebKit linkage

### Story 5.2: GitHub Actions release workflow — build + sign + notarize

As a maintainer,
I want a GitHub Actions workflow that on a `v*` tag builds the Tauri app, code-signs with Apple Developer ID, and notarizes via Apple's `notarytool`,
So that every release artifact is Gatekeeper-acceptable without manual steps.

**Acceptance Criteria:**

**Given** `.github/workflows/release.yml`
**When** inspected
**Then** it triggers on `push: tags: ['v*']` and runs on `macos-latest`

**Given** required secrets configured
**When** the workflow runs
**Then** it reads: `APPLE_DEVELOPER_ID_CERT_P12_BASE64`, `APPLE_DEVELOPER_ID_CERT_PASSWORD`, `APPLE_NOTARY_ID`, `APPLE_NOTARY_TEAM_ID`, `APPLE_NOTARY_PASSWORD` (app-specific password) from GitHub Secrets

**Given** the code-sign step
**When** it runs
**Then** it imports the certificate into a temporary keychain, runs `tauri build` with `--bundles dmg`, then signs the inner `.app` and the `.dmg` itself with `codesign --deep --options runtime` using the imported identity

**Given** the notarize step
**When** it runs
**Then** it calls `xcrun notarytool submit tiny_*.dmg --apple-id $APPLE_NOTARY_ID --team-id $APPLE_NOTARY_TEAM_ID --password $APPLE_NOTARY_PASSWORD --wait` and the job fails if the submission returns anything other than `status: Accepted`

**Given** notarization succeeds
**When** the workflow runs `stapler staple tiny_*.dmg`
**Then** the staple succeeds and `xcrun stapler validate tiny_*.dmg` confirms it locally before upload

**Given** the architecture rule that `cargo audit` and `npm audit` must gate releases too
**When** the workflow runs
**Then** the `audit` jobs from Story 1.10 are dependencies of this workflow (release blocked on audit pass)

**Given** the workflow finishes signing + notarizing
**When** the GitHub Release is drafted
**Then** the `.dmg` is uploaded as a release asset and the release is left as a draft (the checksum + GPG signature step that follows in this epic publishes it)

### Story 5.3: SHA256 checksum + GPG signature publish

As a security-conscious user,
I want each release's `.dmg` accompanied by a SHA256 checksum and a GPG signature signed by the maintainer's long-term key,
So that I can verify integrity and authenticity before installing (NFR-S8).

**Acceptance Criteria:**

**Given** the release workflow continues from Story 5.2
**When** the artifact is ready
**Then** a step runs `shasum -a 256 tiny_*.dmg > tiny_*.dmg.sha256` and uploads `.dmg.sha256` to the GitHub Release

**Given** a GPG private key is provisioned via secret `GPG_PRIVATE_KEY` + passphrase `GPG_PASSPHRASE`
**When** the workflow runs
**Then** it imports the key into a temporary GPG home, runs `gpg --detach-sign --armor tiny_*.dmg`, and uploads `tiny_*.dmg.asc` to the release

**Given** the GPG public key
**When** the release page is published
**Then** the release notes include the maintainer's public key fingerprint + a link to `https://github.com/<maintainer>/tiny-cli/blob/main/SECURITY.md` (which is created in this story) documenting verification steps for end users

**Given** a verification step in CI
**When** the workflow runs after publishing
**Then** it downloads the published `.dmg`, `.sha256`, `.asc` from the draft release and verifies:

- `shasum -a 256 -c tiny_*.dmg.sha256` succeeds
- `gpg --verify tiny_*.dmg.asc tiny_*.dmg` returns `Good signature`

**Given** the draft release was held in Story 5.2
**When** all checksum + signature steps pass
**Then** the release is published (status flips from draft to published)

**Given** the GPG key must be auditable
**When** `SECURITY.md` is inspected
**Then** it documents the key fingerprint, expiration policy, and rotation process for the maintainer

### Story 5.4: Homebrew Cask tap (`homebrew-tiny`) + auto-update on release publish

As a Persona A / B user,
I want to install via `brew install --cask tiny` and have `brew upgrade` pick up new releases automatically,
So that distribution follows the macOS power-user norm without manual `.dmg` downloads (FR57, NFR-I2).

**Acceptance Criteria:**

**Given** a sibling repository `homebrew-tiny` exists on GitHub
**When** inspected
**Then** it contains `Casks/tiny.rb` with a Cask formula referencing the GitHub Releases `.dmg` URL, version, SHA256 checksum, and `app "tiny.app"`

**Given** the Cask formula
**When** `brew install --cask tiny` runs on a fresh macOS
**Then** the installer downloads the `.dmg`, verifies SHA256, mounts it, copies `tiny.app` to `/Applications`, and the user can launch it (Gatekeeper passes due to notarization from Story 5.2)

**Given** a new release is published on the main repo
**When** the post-release GitHub Action (`auto-update-cask.yml`) runs
**Then** it computes the new version + SHA256 from the published assets, opens a PR on `homebrew-tiny` updating `Casks/tiny.rb`, and labels it `automated`

**Given** the auto-update PR is merged
**When** a user runs `brew upgrade --cask tiny`
**Then** the new version is downloaded and replaces the existing install without manual steps

**Given** `brew uninstall --cask tiny`
**When** run
**Then** the app is removed from `/Applications` and the auto-generated `Caveats` (if any) prints uninstall guidance (e.g., "user data at `~/.tiny/` is preserved — delete manually if desired")

**Given** the Cask formula must be valid
**When** `brew audit --cask --strict ./Casks/tiny.rb` runs as a CI gate on the `homebrew-tiny` repo
**Then** it passes with 0 errors and 0 warnings

### Story 5.5: In-app update check qua GitHub Releases API

As a user,
I want the app to check for updates on launch (max once per 24 h, configurable) and to expose a manual "Check for Updates" action,
So that I learn about new versions promptly without aggressive polling (FR59, NFR-I3).

**Acceptance Criteria:**

**Given** `src-tauri/src/updater/check.rs`
**When** inspected
**Then** it exposes `pub async fn check_for_updates() -> Result<UpdateCheckResult, UpdateError>` calling GitHub Releases API `https://api.github.com/repos/<maintainer>/tiny-cli/releases/latest`

**Given** `UpdateCheckResult`
**When** inspected
**Then** it has `#[serde(rename_all = "camelCase")]` fields: `current_version: String`, `latest_version: Option<String>`, `release_notes_url: String`, `dmg_url: String`, `sha256_url: String`, `is_newer: bool`

**Given** the user opens the app
**When** the auto-check is enabled (FR53 default ON) and the last check was > 24 h ago
**Then** `check_for_updates()` runs in the background and stores the result in a Zustand store accessible to Settings + the menu bar later

**Given** `X-RateLimit-Remaining` header from GitHub
**When** the API responds
**Then** the value is stored; if `< 10`, subsequent checks wait at least until `X-RateLimit-Reset`; on `403` exponential backoff is applied (NFR-I3)

**Given** the user clicks "Kiểm tra cập nhật ngay" in the menu bar or Settings
**When** the manual check fires
**Then** it bypasses the 24 h gate and calls the API immediately, returning the result for UI consumption

**Given** offline / DNS failure
**When** `check_for_updates()` runs
**Then** it returns `UpdateError::Offline` and the UI surfaces a "Không kết nối được" hint — never blocks the app launch

**Given** privacy invariant (NFR-Pr4)
**When** auto-check is disabled by the user
**Then** no GitHub API call is made on launch — verified by integration test with a mocked HTTP client

### Story 5.6: Tauri updater integration + GPG signature verification

As a user,
I want updates downloaded by the app to be verified against the maintainer's GPG signature before any install prompt,
So that a tampered `.dmg` (e.g., MITM) cannot reach the install dialog (NFR-S11).

**Acceptance Criteria:**

**Given** `src-tauri/src/updater/download.rs`
**When** inspected
**Then** it has `pub async fn download_and_verify(result: &UpdateCheckResult) -> Result<PathBuf, UpdateError>` that downloads `.dmg`, `.sha256`, `.asc` to a temp dir

**Given** the SHA256 verification step
**When** it runs
**Then** the downloaded checksum file is compared against `sha256sum` of the downloaded `.dmg`; mismatch returns `UpdateError::ChecksumMismatch` and the temp files are wiped

**Given** the maintainer's GPG public key is embedded into the app binary at build time (`src-tauri/assets/public-key.asc`)
**When** the GPG verify step runs
**Then** the app uses a Rust GPG library (e.g., `sequoia-openpgp` or shells out to `gpg` if installed) to verify `.dmg.asc` against the embedded public key

**Given** the GPG verification fails (forged signature, wrong key)
**When** it returns
**Then** `UpdateError::SignatureInvalid` is returned, the temp files are wiped, and a hard error is shown to the user with link to SECURITY.md — no install prompt is shown

**Given** verification succeeds
**When** `download_and_verify` returns
**Then** it returns the temp path of the verified `.dmg` ready for install

**Given** the architecture invariant that no panic crosses IPC boundary (Story 1.7)
**When** any verification step encounters a panic case
**Then** it is caught and converted to a typed `UpdateError` variant, never bubbling raw panic to Tauri

### Story 5.7: Update UX — Settings panel + "New version available" dialog

As a user,
I want clear UI for current vs available versions, changelog preview, and explicit confirm before install,
So that I'm always in control of when the app updates (no surprise restarts).

**Acceptance Criteria:**

**Given** the Settings → Cập nhật section (extends Story 4.5)
**When** rendered
**Then** it shows: current version (e.g., "Phiên bản hiện tại: 1.0.0"), last check timestamp ("Kiểm tra lần cuối: 3 giờ trước"), and a "Kiểm tra ngay" button

**Given** an update is available
**When** the Settings page loads
**Then** the section shows a green callout "Có bản mới: 1.1.0" with "Xem chi tiết" expanding to release notes (fetched from `release_notes_url`)

**Given** the user clicks "Tải về"
**When** the download starts
**Then** a progress bar appears (reuse `ProgressStream` from Story 2.6) showing bytes downloaded vs total

**Given** download + verification (Stories 5.5–5.6) succeed
**When** the verification completes
**Then** an `InstallConfirmDialog` (reusing `DestructiveConfirmDialog` pattern from Story 3.8) appears with title "Cài đặt phiên bản 1.1.0?", body listing what changes, confirm label "Cài đặt và khởi động lại", cancel label "Để sau"

**Given** the user confirms install
**When** Tauri's installer hook fires
**Then** the app launches the install (mounts `.dmg`, prompts for `/Applications` replacement) and quits gracefully — UI state is saved (NFR-R6) so on relaunch the user is at the same page

**Given** the user defers install ("Để sau")
**When** they later relaunch
**Then** the verified `.dmg` is still in temp and Settings shows "Bản 1.1.0 đã tải về — sẵn sàng cài" with a single "Cài đặt" button (no re-download needed)

**Given** FR53 toggle "Tự động cài cập nhật"
**When** the toggle is ON and a verified download exists
**Then** the install confirm dialog STILL appears (per PRD: "luôn cần user confirm trước khi install") — auto-install does not mean silent install

---

## Epic 6: Space Lens — Disk Visualization

User explore disk usage qua treemap/sunburst tương tác (full volume hoặc home dir), drill-down double-click + breadcrumb + ⌫ back, heat-by-age/heat-by-type color overlay, phantom space detection (APFS purgeable + snapshot explain). Right-click tile → Reveal in Finder / Quick Look / Trash / quarantine / uninstall. Exclude system/cloud-synced/pinned folders. FSEvents incremental re-scan. Export PNG/JSON. Khối kỹ thuật lớn nhất — parallel walker mới, treemap virtualization 500k files.

### Story 6.1: Engine walker — parallel scan + exclusion defaults

As an engine developer,
I want a parallel filesystem walker in `tiny-core/src/space_lens/` that can scan home directory or a full volume with built-in exclusion defaults,
So that downstream UI can render a complete tree without burning CPU on system + cloud-synced folders.

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/space_lens/walker.rs`
**When** inspected
**Then** it exposes `pub fn walk(root: &Path, opts: WalkOpts, progress: impl Fn(Progress) + Send) -> Result<SpaceTree, CoreError>` returning a tree of `Node { path, size_bytes, child_count, kind }` with `kind ∈ { File, Dir, Symlink }`

**Given** parallelism
**When** the walker runs
**Then** it uses `rayon` or `jwalk` for parallel directory traversal; CPU utilization on M2 reaches ≥ 4 cores during a full-volume scan (verified manually + bench)

**Given** the default exclusion list
**When** `WalkOpts::default()` is used
**Then** these paths are skipped: `/System`, `/private`, `/Volumes`, `~/Library/CloudStorage` (and known cloud-sync providers: iCloud Drive, Dropbox, Google Drive folders detected via known patterns), `~/.Trash`, plus any path in the caller-supplied `opts.exclude_paths` (the app layer populates this from user folder locks — `tiny-core` stays stateless and never reads SQLite)

**Given** symlink safety (FR13)
**When** a symlink is encountered
**Then** the walker records the node but does NOT recurse through it — `symlink_metadata` is used, never `metadata`

**Given** progress callback wiring
**When** the walker iterates
**Then** the callback fires at most once per 100 ms with cumulative bytes/files counted (buffering rules consistent with Story 1.5)

**Given** NFR-P9 (full-volume scan ≤ 60 s P95 on ~500k files SSD)
**When** the Epic 6 walker performance bench runs
**Then** the walker meets this target

**Given** the architecture invariant
**When** `cargo tree -p tiny-core` is checked
**Then** new deps (e.g., `rayon` 1.x or `jwalk` 0.8) are runtime-agnostic — no `tokio`, no `tauri`

### Story 6.2: Phantom space detection (APFS purgeable + snapshot)

As a user,
I want the app to detect and explain "phantom space" — APFS purgeable bytes and snapshots — so I understand why disk usage exceeds visible files,
So that I can answer the Persona A question "vì sao ổ đầy mà không thấy file?" (FR25).

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/space_lens/phantom.rs`
**When** inspected
**Then** it has `pub fn detect_phantom_space(volume: &Path) -> Result<PhantomReport, CoreError>` returning `PhantomReport { purgeable_bytes, snapshot_bytes, snapshot_count, snapshots: Vec<SnapshotMeta> }`

**Given** APFS purgeable detection
**When** the function runs
**Then** it uses `statvfs` + `URLResourceKey` (`com.apple.disk.NSURLVolumeAvailableCapacityForImportantUsage`) via FFI or, where unsupported, a shell to `df` and parses the "purgeable" field

**Given** APFS snapshots
**When** the function runs `diskutil apfs list -plist` as a subprocess and parses the output
**Then** it extracts each snapshot's name, creation date, and size — returning them in `SnapshotMeta`

**Given** the result must be flagged as estimate (per PRD risk mitigation)
**When** `PhantomReport` is serialized
**Then** it has a `is_estimate: true` field with an `explanation_url` pointing to an Apple developer doc on APFS storage

**Given** the function MUST NOT delete or modify purgeable space
**When** I grep for `delete`, `remove`, `rm` inside `phantom.rs`
**Then** 0 matches — this module is read-only

**Given** Tauri command `space_lens_phantom_detect()`
**When** invoked
**Then** it returns the `PhantomReport` via the usual `Result<_, ErrorPayload>` pattern (Stories 1.7 + 1.8)

### Story 6.3: FSEvents watcher — incremental re-scan

As a user,
I want the Space Lens to update incrementally as I create/delete files in watched folders,
So that I don't need to manually re-trigger a full scan after every change (FR26).

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/space_lens/watcher.rs`
**When** inspected
**Then** it uses `notify-rs` (macOS FSEvents backend) to watch the root directories from `WalkOpts`

**Given** an FSEvents notification arrives
**When** the watcher dispatches
**Then** it computes a delta `Vec<TreeChange> { Added(Node), Removed(Path), SizeChanged(Path, new_size) }` for the affected folder only, NOT re-walking the entire root

**Given** Tauri commands `space_lens_watch_start(root)` and `space_lens_watch_stop()`
**When** invoked
**Then** they manage the watcher lifecycle; events emit Tauri event `"space_lens:change"` carrying `Vec<TreeChange>` serialized as camelCase

**Given** event burst (e.g., npm install creating thousands of files)
**When** events arrive in < 100 ms window
**Then** the watcher coalesces them into a single `space_lens:change` emission to avoid UI thrash

**Given** the watcher must be cancellable
**When** the app closes the window or the user navigates away
**Then** the watcher is stopped within ≤ 100 ms and the OS handles are released (verified by `lsof` showing no leaked file descriptors after close)

**Given** the app crashes while watching
**When** it restarts
**Then** there is no leftover FSEvents handle holding (FSEvents is process-scoped — automatic on macOS — verified by integration test)

### Story 6.4: SQLite schema cho space_lens snapshots + folder locks

As a backend developer,
I want SQLite tables for periodic Space Lens snapshots and user-defined folder locks (FR15 / FR24),
So that "tuần này có gì đổi" and folder-protection features have durable backing storage.

**Acceptance Criteria:**

**Given** `src-tauri/src/db/space_lens.rs` (new migration)
**When** inspected
**Then** it creates table `space_lens_snapshots { id INTEGER PK, root TEXT NOT NULL, taken_at INTEGER NOT NULL, total_bytes INTEGER NOT NULL, tree_blob BLOB NOT NULL }` where `tree_blob` is a compressed (zstd) serialization of the `SpaceTree`

**Given** the same migration
**When** inspected
**Then** it also creates `folder_locks { id INTEGER PK, path TEXT NOT NULL UNIQUE, password_hash TEXT, created_at INTEGER NOT NULL }` — `password_hash` is nullable; uses Argon2id if non-null

**Given** indices for query performance
**When** the migration runs
**Then** it creates `idx_snapshot_root_time` on `(root, taken_at DESC)` for "show snapshots of this root sorted recent-first"

**Given** the snapshot blob may grow (500k files ≈ ~10 MB raw → ~2 MB compressed)
**When** snapshots accumulate
**Then** an auto-prune rule keeps the most recent 30 snapshots per root (older rows deleted via background task on insert)

**Given** the architecture rule (D4)
**When** inspected
**Then** these tables live in `src-tauri/src/db/`, not in `tiny-core` — `tiny-core` remains stateless

**Given** the migration runs twice
**When** the second run executes
**Then** it is idempotent (schema_version table from Story 3.1 prevents replay)

### Story 6.5: Tauri commands `space_lens_*`

As a frontend developer,
I want a complete set of Tauri commands for Space Lens (scan, watch, lock, export),
So that the frontend can drive the feature through one consistent IPC surface.

**Acceptance Criteria:**

**Given** `src-tauri/src/commands/space_lens.rs`
**When** inspected
**Then** it has these commands wrapped in `Result<T, ErrorPayload>`:

- `space_lens_scan(root: PathBuf) -> SpaceTree`
- `space_lens_watch_start(root: PathBuf) -> ()` + `space_lens_watch_stop() -> ()`
- `space_lens_lock_folder(path: PathBuf, password: Option<String>) -> ()` + `space_lens_unlock_folder(path: PathBuf, password: Option<String>) -> ()`
- `space_lens_list_locks() -> Vec<FolderLock>`
- `space_lens_export(scope: ExportScope, format: ExportFormat) -> PathBuf` (export backend; the export UI is a later story in this epic)
- `space_lens_phantom_detect(volume: PathBuf) -> PhantomReport` (covers Story 6.2)

**Given** `space_lens_scan`
**When** invoked
**Then** it uses `spawn_blocking` to call `tiny_core::space_lens::walk`, emitting `space_lens:progress` events (consistent with Stories 1.8 / 2.3)

**Given** `space_lens_lock_folder` with a password
**When** invoked
**Then** the password is hashed with Argon2id (`argon2` crate, default work factor) and stored — plaintext never persists; the original is zeroized after use

**Given** `space_lens_unlock_folder`
**When** invoked with a password
**Then** the provided password is hashed and compared via constant-time comparison; on success the lock row is deleted

**Given** any folder lock at scan time
**When** the walker reads `folder_locks`
**Then** locked paths are excluded from the walked tree AND the user's selections in Cleanup (Epic 3) cannot target them — verified by integration test from Story 3.6 + this story

**Given** no command panics across IPC
**When** I grep `unwrap()` / `expect()` in `commands/space_lens.rs`
**Then** 0 matches outside `#[cfg(test)]`

### Story 6.6: Treemap layout helper `lib/treemap.ts` (pure d3-hierarchy)

As a frontend developer,
I want a pure TypeScript function `layoutTreemap(tree, width, height)` in `src/lib/treemap.ts` that returns flat positioned rectangles,
So that the Treemap component (built in the next story) renders without coupling to React lifecycle or Web APIs.

**Acceptance Criteria:**

**Given** `src/lib/treemap.ts`
**When** inspected
**Then** it exports `export interface TreemapTile { id: string; path: string; x: number; y: number; w: number; h: number; bytes: number; depth: number; }` and `export function layoutTreemap(root: SpaceTreeNode, width: number, height: number, padding?: number): TreemapTile[]`

**Given** the architecture rule (`lib/ → no features/, no components/`)
**When** inspected
**Then** the file imports only from `d3-hierarchy` and other `lib/` modules — never from `features/` or `components/`

**Given** the function is pure
**When** called twice with identical inputs
**Then** the returned tile array is structurally identical (byte-equal JSON)

**Given** Vitest tests `src/lib/treemap.test.ts`
**When** they run
**Then** they cover: (a) layout for a 3-node tree, (b) layout for an empty tree returns `[]`, (c) layout respects the `padding` argument, (d) tiles tile the full `[0,0,w,h]` rectangle (no overlap, no gap > padding)

**Given** large inputs
**When** the function is called with 500k nodes
**Then** it completes in ≤ 800 ms on M2 (measured by the Epic 6 performance bench)

**Given** the function does NOT call `window`, `document`, or any DOM API
**When** I grep for them
**Then** 0 matches — the helper is testable in jsdom or pure node

### Story 6.7: `Treemap` + `Sunburst` components + toggle

As a user,
I want to see my disk usage as either a treemap or a sunburst, with a single toggle to switch views over the same data,
So that I can choose the visualization that fits my mental model (FR19, FR21).

**Acceptance Criteria:**

**Given** `src/features/space-lens/treemap.tsx` and `sunburst.tsx`
**When** inspected
**Then** both accept the same prop shape `{ tree: SpaceTreeNode; onTileClick: (path) => void; onTileContextMenu: (path, x, y) => void; }`

**Given** virtualization for large trees (NFR-P10)
**When** the tree has > 5,000 visible-depth tiles
**Then** Treemap only renders tiles whose bounding box intersects the visible viewport (or whose `bytes / total > threshold`) — confirmed by mounting test that counts DOM elements

**Given** rendering 500k-file tree
**When** measured
**Then** initial render P95 is ≤ 1 s (NFR-P10) — measured by the Epic 6 performance bench

**Given** the visualization toggle
**When** the user clicks "Treemap" / "Sunburst" toggle
**Then** the view crossfades (or instantly swaps if Reduce Motion is on per NFR-A7) keeping the current drill-down state

**Given** color is not the sole signal
**When** rendered
**Then** each tile carries a text label (path basename + size) for any tile ≥ 60 px in either dimension; smaller tiles rely on hover tooltip

**Given** Vitest test
**When** it runs
**Then** it asserts (a) the same tree produces tiles in both views, (b) toggle changes which component is rendered without re-fetching data

### Story 6.8: Heat overlay — color tiles by age / type / both

As a user,
I want to apply a heat-by-age or heat-by-type color overlay on the visualization,
So that I can spot "candidate to delete" tiles (large + old) at a glance (FR22).

**Acceptance Criteria:**

**Given** `src/features/space-lens/heat-overlay.tsx`
**When** inspected
**Then** it exports a `HeatMode = 'none' | 'age' | 'type' | 'age_and_type'` controller plus a `computeTileColor(tile, mode)` pure helper in `src/lib/heat.ts`

**Given** mode `age`
**When** a tile's `last_modified` is > 365 days ago
**Then** it receives a red shade; < 30 days receives green; 30–365 days gradient amber-to-green (token-driven from Story 4.1)

**Given** mode `type`
**When** tiles are colored
**Then** known type groups (code, media, archive, log, cache) map to a fixed palette documented in `src/lib/heat.ts` — distinguishable in protanopia/deuteranopia/tritanopia (per Story 4.11)

**Given** mode `age_and_type`
**When** both overlays apply
**Then** type drives base hue, age drives saturation — combination remains accessible (verified by axe-core in Story 4.11)

**Given** the legend
**When** any heat mode is active
**Then** a legend panel renders showing the mapping (e.g., "Red = > 365 days", "Green = recent") so color meaning is never opaque

**Given** the heat mode toggle persists
**When** user picks a mode and reopens Space Lens
**Then** the last-used mode is restored from the Zustand store

### Story 6.9: Drill-down + breadcrumb + ⌫ back navigation

As a user,
I want to double-click a tile to drill down and navigate back via breadcrumb or ⌫ key,
So that I can explore the tree without getting lost (FR20).

**Acceptance Criteria:**

**Given** the current drill state lives in a Zustand store
**When** the user double-clicks a tile
**Then** the store updates `currentRoot = clicked_node.path` and the visualization re-renders rooted at that node

**Given** the breadcrumb component
**When** the drill state changes
**Then** the breadcrumb shows the path from the original root to the current node as clickable segments (e.g., `/ Users / kyantran / Documents`)

**Given** any breadcrumb segment is clicked
**When** the click fires
**Then** the drill state pops to that level

**Given** the keyboard
**When** the user presses ⌫ (backspace)
**Then** the drill state pops one level up; if already at the top-level root, no-op

**Given** keyboard accessibility (NFR-A1)
**When** focus is inside the visualization area
**Then** ⌫ works; outside, the global keyboard handler does not steal it (e.g., ⌫ in a text input still deletes a character)

**Given** Reduce Motion (NFR-A7)
**When** enabled
**Then** drill transitions are instant (no zoom animation > 200 ms)

### Story 6.10: Right-click tile action menu

As a user,
I want to right-click any tile to reach common actions (Reveal in Finder, Quick Look, send to Trash, send to quarantine, uninstall),
So that I can act on a tile without leaving the visualization (FR23).

**Acceptance Criteria:**

**Given** `src/features/space-lens/tile-context-menu.tsx`
**When** opened by right-click (or Shift+F10)
**Then** it shows menu items: "Hiện trong Finder", "Xem nhanh (Quick Look)", "Đưa vào Trash", "Đưa vào quarantine 30 ngày", and conditionally "Gỡ cài đặt app" (only for `.app` paths)

**Given** "Hiện trong Finder"
**When** invoked
**Then** Tauri command `reveal_in_finder(path)` runs `NSWorkspace.shared.selectFile(path, inFileViewerRootedAtPath: ...)` via `objc2`

**Given** "Xem nhanh (Quick Look)"
**When** invoked
**Then** Tauri command `quick_look(path)` runs `qlmanage -p <path>` as a subprocess and the Quick Look preview opens

**Given** "Đưa vào Trash"
**When** invoked
**Then** the path is sent through `delete_with_safety_net` from Story 3.2 with provider context `space_lens_tile`; the affected tile is removed from the visualization (or marked struck-through) via the FSEvents stream from Story 6.3

**Given** "Đưa vào quarantine 30 ngày"
**When** invoked
**Then** it forces the quarantine code path (skipping Trash attempt) for files the user explicitly wants kept recoverable

**Given** "Gỡ cài đặt app"
**When** the right-clicked path ends in `.app`
**Then** the action invokes `tiny_core::uninstall::uninstall(path)` and reuses the existing CLI uninstall flow

**Given** keyboard parity (NFR-A1)
**When** a tile is focused
**Then** Shift+F10 (macOS context-menu key) opens the same menu with the first item focused

### Story 6.11: Phantom space panel UI

As a user,
I want a panel that explains "phantom space" (APFS purgeable + snapshots) with concrete numbers and a link to Apple's doc,
So that I understand why my disk shows full without visible files (FR25 + UX-DR19 "explain don't show").

**Acceptance Criteria:**

**Given** `src/features/space-lens/phantom-panel.tsx`
**When** it mounts
**Then** it fetches `space_lens_phantom_detect(volume)` and displays: total purgeable bytes, snapshot count, list of snapshots (name + created_at as "5 ngày trước" + size)

**Given** the `is_estimate` flag is true
**When** rendered
**Then** the panel shows a banner: "Số này là ước tính từ APFS — có thể khác với Finder. Xem chi tiết →" with the link to `explanation_url`

**Given** UX-DR19 rule (every number needs explanation)
**When** the panel renders
**Then** each number has a paired explanation: total purgeable → "Đây là dung lượng macOS có thể giải phóng tự động khi cần"; snapshot list → "Snapshot Time Machine giữ phiên bản cũ — xoá qua `tmutil thinlocalsnapshots`"

**Given** the panel must NOT delete purgeable space (per Story 6.2)
**When** rendered
**Then** there are NO destructive buttons — only informational + read-only

**Given** keyboard accessibility
**When** the link is focused
**Then** Enter opens the URL in the default browser via Tauri `tauri::shell::open`

**Given** loading state
**When** the detection is in flight
**Then** the panel shows a skeleton (with explanatory text "Đang đọc thông tin APFS…") — not a bare spinner

### Story 6.12: Folder lock + exclusion settings UI

As a user (Persona C's son acting on her behalf),
I want a Settings UI to lock specific folders so no provider can ever touch them, with optional password protection,
So that critical user folders (Desktop, Documents, custom paths) are protected even if I tick the wrong category (FR15 / FR24).

**Acceptance Criteria:**

**Given** Settings → Lưu trữ section (extends Story 4.4)
**When** rendered
**Then** it shows a "Thư mục được bảo vệ" subsection with a list of locked paths and an "Thêm" button

**Given** the user clicks "Thêm"
**When** the file-picker opens (Tauri's `dialog::open`)
**Then** they can pick any directory; on selection a "Có yêu cầu mật khẩu để mở khoá?" prompt appears (optional)

**Given** the user adds a path
**When** confirmed
**Then** `space_lens_lock_folder(path, password?)` is called and the locked path appears in the list

**Given** the user clicks "Mở khoá" on a locked path
**When** the path has a password
**Then** a password prompt appears; on correct entry `space_lens_unlock_folder(path, password)` runs and the row is removed; on incorrect entry the row remains and a "Sai mật khẩu" message shows (no app crash)

**Given** the locked path
**When** Cleanup flow (Epic 3) computes selections
**Then** files inside locked paths are excluded — verified by integration test that pre-locks `~/Documents` and ensures no provider can target files there

**Given** the locked path
**When** Space Lens scans
**Then** it is excluded from the walked tree by Story 6.1's exclusion logic

**Given** default-suggested protections per Persona C
**When** the user opens this UI for the first time
**Then** a "Gợi ý: bảo vệ Desktop, Documents" callout shows with one-click buttons to lock those defaults

### Story 6.13: Export Space Lens → PNG / JSON

As a user,
I want to export the current Space Lens view as a PNG image or a JSON report,
So that I can share with collaborators or archive for comparison (FR27).

**Acceptance Criteria:**

**Given** an "Export" button in the Space Lens toolbar
**When** clicked
**Then** a small popover offers two options: "Lưu thành ảnh PNG" and "Lưu thành báo cáo JSON"

**Given** PNG export
**When** the user clicks it
**Then** the current canvas (Treemap or Sunburst) is rasterized via `html2canvas` (or `dom-to-image`) and saved via `tauri::dialog::save` to the user-chosen path; the resulting PNG matches the viewport at 2× DPI

**Given** JSON export
**When** the user clicks it
**Then** the current tree (with current drill-down root) is serialized to a structured JSON `{ root, generatedAt, totalBytes, nodes: [{ path, sizeBytes, lastModified, kind }] }` saved via `tauri::dialog::save`

**Given** the JSON format
**When** inspected
**Then** it includes a `schemaVersion: 1` field (per NFR-I4 versioned JSON output) and follows camelCase

**Given** privacy guard
**When** the user exports
**Then** the user is presented an inline notice "File JSON sẽ chứa path đầy đủ — không gửi cho người lạ" — saving is gated only by their explicit confirmation

**Given** large exports
**When** the tree has > 100k nodes
**Then** the JSON export streams to disk (not held entirely in memory) to avoid OOM

**Given** CLI parity (FR41)
**When** the equivalent CLI is computed
**Then** the footer shows `tiny space_lens export --format=json --output=...` (forward-declared if the CLI subcommand has not been implemented yet)

### Story 6.14: Space Lens performance bench gates

As a maintainer,
I want CI benches asserting full-volume scan ≤ 60 s P95 and treemap render 500k files ≤ 1 s P95,
So that NFR-P9 and NFR-P10 are enforced automatically (folds with Story 1.10 baseline mechanism).

**Acceptance Criteria:**

**Given** `crates/tiny-core/benches/space_lens_walk_bench.rs`
**When** run on a synthetic ~500k-file fixture
**Then** the walker's P95 is ≤ 60 s on `macos-latest` (Apple Silicon) CI runner

**Given** `src/lib/treemap.bench.ts` (Vitest bench-mode or a `vitest --bench` suite)
**When** run on a 500k-node tree
**Then** `layoutTreemap()` completes in ≤ 800 ms P95 (room under the 1 s NFR-P10 cap to leave budget for paint)

**Given** a frontend render bench (Playwright + DevTools trace, optional)
**When** the Treemap component is mounted on the 500k tree
**Then** first paint is ≤ 1 s P95 (NFR-P10)

**Given** the regression baseline from Story 1.10
**When** any bench median regresses by > 20%
**Then** the CI job fails with a clear diff (baseline vs new)

**Given** bench flakiness control
**When** the benches run 5 times consecutively
**Then** coefficient of variation < 10%

**Given** Criterion HTML artifact upload
**When** CI completes
**Then** results are published as a workflow artifact alongside Story 2.9 reports for trend comparison

---

## Epic 7: Active Monitoring & Explanation

Menubar tray widget realtime CPU/RAM/disk + sparkline. Click → mini-panel Health Score + "Run Smart Scan" button. Menubar-only mode (hide Dock icon). Periodic snapshots → "tuần này có gì đổi" diff. Disk-full forecast + proactive notify. FSEvents "newly large files" badge ở Downloads/Desktop/Documents. Heavy Consumers panel — quit process trực tiếp. Scheduled Smart Scan weekly/daily.

### Story 7.1: Menubar tray icon + hide-to-tray behavior

As a power user (Persona B),
I want a menubar tray icon that keeps the app alive when I close the window,
So that monitoring continues without a Dock window in the way.

**Acceptance Criteria:**

**Given** `src-tauri/src/tray.rs`
**When** the app launches
**Then** a Tauri tray icon appears in the macOS menubar with a left-click handler (wired to the mini-panel in a later story) and a right-click context menu (Open / Run Smart Scan / Settings / Quit)

**Given** the user closes the main window
**When** the close event fires
**Then** the window hides (does not terminate the process) and the tray icon remains; reopening via the tray restores the window with prior state (NFR-R6)

**Given** the explicit Quit menu item
**When** clicked
**Then** the app terminates fully, releasing the tray and stopping background tasks (sampler, watchers)

**Given** the tray icon must reflect health at a glance
**When** the latest Health Score is computed
**Then** the tray icon tints (green/amber/red) matching the lane thresholds from Story 2.5

**Given** Reduced Motion / accessibility
**When** the tray menu is opened
**Then** all menu items have clear Vietnamese labels and are reachable via the macOS menubar keyboard navigation

### Story 7.2: Realtime system sampler (engine)

As an engine developer,
I want a sampler in `tiny-core` that polls CPU/RAM/disk and maintains a bounded history buffer,
So that the menubar widget and sparkline have a runtime-agnostic data source.

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/sys/sampler.rs`
**When** inspected
**Then** it exposes `pub fn sample() -> SysSample` returning `SysSample { cpu_percent, mem_used_bytes, mem_total_bytes, disk_used_bytes, disk_total_bytes, sampled_at }` with camelCase serde

**Given** a history buffer
**When** inspected
**Then** the caller (src-tauri) maintains a fixed-capacity ring buffer (e.g., last 60 samples) — the buffer logic lives in src-tauri (stateful), while `sample()` itself is stateless in core

**Given** the sampler is called every 1 s (NFR-P13)
**When** measured on M2
**Then** a single `sample()` call costs ≤ 0.5% CPU averaged over 1 minute

**Given** the architecture invariant
**When** `cargo tree -p tiny-core` is checked
**Then** the sampler adds no async-runtime dependency (uses `sysinfo` only)

**Given** unit tests
**When** run
**Then** they assert `sample()` returns plausible bounds (cpu 0–100, mem_used ≤ mem_total, disk_used ≤ disk_total)

### Story 7.3: Notification system integration

As a frontend/backend developer,
I want a shared notification wrapper around `tauri-plugin-notification`,
So that FR32 (forecast), FR18 (scheduled scan), and Story 3.11 (quarantine size) emit consistent native notifications.

**Acceptance Criteria:**

**Given** `src-tauri/src/notify.rs`
**When** inspected
**Then** it exposes `pub fn notify(app: &AppHandle, kind: NotificationKind, title: &str, body: &str)` wrapping the plugin, with `NotificationKind` enum for telemetry/grouping

**Given** macOS notification permission
**When** the app first attempts to notify
**Then** it requests notification permission via the plugin; if denied, notifications degrade silently (logged at DEBUG, no crash)

**Given** notification rate-limiting
**When** the same `NotificationKind` would fire more than once per its configured cooldown (e.g., quarantine size warning once / 24 h)
**Then** the wrapper suppresses the duplicate

**Given** privacy (NFR-Pr1, NFR-Pr3)
**When** any notification body is built
**Then** it contains no sensitive file paths beyond folder-level summaries; no network call is made

**Given** the user clicks a notification
**When** the action fires
**Then** it focuses the app to the relevant view (e.g., forecast notification → monitor tab)

### Story 7.4: Menubar widget realtime display + sparkline

As a user,
I want the menubar (and its dropdown) to show live CPU/RAM/disk with a small sparkline history,
So that I have an at-a-glance system pulse (FR28).

**Acceptance Criteria:**

**Given** the tray dropdown / widget
**When** opened
**Then** it shows current CPU %, RAM used/total, disk used/total, each with a sparkline of the last 60 samples (Story 7.2 buffer)

**Given** the sampler emits `sys:sample` events every 1 s
**When** an event arrives
**Then** the widget updates within the same frame; the sparkline scrolls by one point

**Given** NFR-P13
**When** the widget is open and updating
**Then** the rendering cost stays ≤ 0.5% CPU averaged over 1 minute (sparkline uses canvas or lightweight SVG, not per-point DOM nodes)

**Given** color is not the sole signal (NFR-A5)
**When** a metric crosses a warning threshold (e.g., disk > 90%)
**Then** the value shows both a red tint AND a warning icon + text

**Given** Reduced Motion (NFR-A7)
**When** enabled
**Then** the sparkline updates without sweep animation (instant point shift)

### Story 7.5: Menubar mini-panel — Health Score + Run Smart Scan

As a user,
I want clicking the tray to open a mini-panel with the current Health Score and a one-click Smart Scan,
So that I can act without opening the full app (FR29).

**Acceptance Criteria:**

**Given** the user left-clicks the tray icon
**When** the mini-panel opens
**Then** it shows the latest `HealthScoreBadge` (Story 2.5) and a "Run Smart Scan" button

**Given** no scan has run yet this session
**When** the mini-panel opens
**Then** the Health Score area shows an `EmptyState` ("Chưa quét") with the Smart Scan button prominent

**Given** the user clicks "Run Smart Scan" from the mini-panel
**When** the scan triggers
**Then** it runs in the background (Story 2.3) and the mini-panel shows `ProgressStream`; on completion the Health Score updates in place

**Given** the user wants more detail
**When** they click "Mở app đầy đủ"
**Then** the main window opens to the Smart Scan result view

**Given** keyboard accessibility
**When** the mini-panel is open
**Then** it is fully keyboard-navigable and Esc closes it

### Story 7.6: Menubar-only mode (hide Dock icon)

As a power user (Persona B),
I want to hide the Dock icon and run the app purely from the menubar,
So that it stays out of my way like an htop widget (FR30).

**Acceptance Criteria:**

**Given** Settings → Giao diện
**When** rendered
**Then** it has a toggle "Chỉ hiện trên menubar (ẩn icon Dock)" default OFF

**Given** the user enables the toggle
**When** it applies
**Then** the app sets `NSApplication.ActivationPolicy` to `.accessory` (via Tauri config or runtime call), the Dock icon disappears, and the tray remains the only entry point

**Given** menubar-only mode is on
**When** the user opens the main window from the tray
**Then** the window appears without restoring the Dock icon

**Given** the user disables the toggle
**When** it applies
**Then** the Dock icon reappears (policy `.regular`) — may require relaunch; if so, a "Cần khởi động lại" prompt appears

**Given** the setting persists
**When** the app relaunches
**Then** the activation policy matches the saved preference from app start

### Story 7.7: Periodic snapshot capture + storage

As a backend developer,
I want the app to periodically capture disk-usage snapshots and store them,
So that "what changed this week" and disk-full forecast have a time series to query (FR31 backing).

**Acceptance Criteria:**

**Given** a background task in src-tauri
**When** it runs
**Then** it captures a lightweight disk-usage snapshot (top-level folder sizes + total) on a schedule (default daily) and stores it in a `disk_usage_snapshots` table

**Given** the snapshot table
**When** inspected
**Then** it has `{ id, taken_at INTEGER, total_used_bytes, folder_breakdown BLOB }` where breakdown is a compact serialization of `~` top-level folder sizes

**Given** snapshot retention
**When** snapshots accumulate
**Then** the task keeps at least 90 days of daily snapshots, pruning older ones

**Given** the capture must be cheap
**When** it runs
**Then** it does NOT do a full-volume walk — it samples top-level folder sizes only (≤ 3 s on M2)

**Given** app was off for several days
**When** it relaunches
**Then** it captures one snapshot on launch (gap-filling), not one per missed day

### Story 7.8: "Tuần này có gì đổi" diff view

As a user,
I want to see which folders grew (and by how much) over the past week,
So that I can answer "vì sao ổ đầy thêm?" with specifics (FR31, UX-DR19).

**Acceptance Criteria:**

**Given** `features/monitor/whats-changed.tsx`
**When** it loads
**Then** it compares the most recent snapshot against one ~7 days prior and lists folders by delta bytes (largest growth first)

**Given** each row
**When** rendered
**Then** it shows folder name, delta (e.g., "+12 GB"), and an explanation when inferable (e.g., "Docker build thứ Tư" if a known cache folder) — every number paired with context (UX-DR19)

**Given** insufficient history (< 2 snapshots)
**When** the view loads
**Then** it shows an `EmptyState` "Cần thêm dữ liệu — quay lại sau vài ngày" instead of an error

**Given** a folder shrank
**When** rendered
**Then** it shows negative deltas (e.g., "-3 GB") in a distinct but accessible style (icon + sign, not color alone)

**Given** the user clicks a changed folder
**When** the click fires
**Then** it deep-links to Space Lens (Epic 6) rooted at that folder

### Story 7.9: Disk-full forecast + proactive notification

As a user,
I want the app to forecast when my disk will be full and notify me ahead of time,
So that I can clean before hitting an emergency (FR32).

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/forecast.rs`
**When** inspected
**Then** it has `pub fn forecast_disk_full(snapshots: &[DiskSnapshotPoint]) -> ForecastResult` returning `{ days_until_full: Option<u32>, trend_bytes_per_day, confidence }` using linear regression over the trend

**Given** fewer than 7 data points
**When** forecast runs
**Then** it returns `days_until_full: None` with low confidence (no misleading prediction)

**Given** the forecast falls below a configurable threshold (default 14 days)
**When** the daily task evaluates
**Then** it fires a notification via Story 7.3 ("Đà này ~12 ngày nữa đầy ổ. Dọn sớm không?") at most once per 24 h

**Given** the threshold setting
**When** the user adjusts it in Settings
**Then** the forecast task respects the new value on next evaluation

**Given** UX-DR19 explainability
**When** the forecast is shown in the monitor view
**Then** it shows the trend line + "dựa trên xu hướng 30 ngày" caption, never a bare number

### Story 7.10: FSEvents "newly large files" badge

As a user,
I want the app to notice when a large new file appears in Downloads/Desktop/Documents and badge the app icon,
So that I'm aware of space-eaters as they land (FR33).

**Acceptance Criteria:**

**Given** an FSEvents watch on Downloads/Desktop/Documents (reuses Story 6.3 watcher)
**When** a new file appears above the configured threshold (default 1 GB)
**Then** the app icon (Dock or tray) shows a badge count of newly-large files since last acknowledgement

**Given** the threshold setting
**When** the user changes it in Settings
**Then** the watcher applies the new threshold going forward

**Given** the user opens the app
**When** they view the "newly large files" list
**Then** each entry shows name, size, location, age; acknowledging clears the badge

**Given** a flood of files (e.g., a download of many large files)
**When** events arrive
**Then** the badge count is coalesced and updated without UI thrash (debounce per Story 6.3)

**Given** privacy
**When** badge/notification text is built
**Then** it shows folder-level info only (no full sensitive path in the notification body)

### Story 7.11: Heavy Consumers panel — process list + quit

As a user,
I want a panel listing processes with highest CPU/RAM and the ability to quit them,
So that I can free resources directly (FR34).

**Acceptance Criteria:**

**Given** `features/monitor/heavy-consumers.tsx`
**When** it loads
**Then** it lists the top N processes by CPU and by RAM (toggle), each row showing process name, PID, CPU %, RAM bytes

**Given** the engine source
**When** inspected
**Then** process enumeration comes from `tiny-core` via `sysinfo` (runtime-agnostic), exposed through a Tauri command `list_processes(sort_by)`

**Given** the user clicks "Quit" on a process
**When** the action fires
**Then** a `DestructiveConfirmDialog` (Story 3.8) confirms ("Thoát tiến trình {name}? Dữ liệu chưa lưu có thể mất."), then the app sends SIGTERM (not SIGKILL) via a Tauri command

**Given** the process does not exit after SIGTERM within a timeout
**When** the user is offered escalation
**Then** a secondary confirm offers "Buộc thoát (SIGKILL)" — never auto-escalates silently

**Given** a protected/system process
**When** listed
**Then** the Quit button is disabled with a tooltip explaining it cannot be terminated safely

**Given** the list refreshes
**When** open
**Then** it updates every 2 s without resetting the user's sort/scroll position

### Story 7.12: Scheduled Smart Scan (weekly/daily) + notification

As a user,
I want to schedule Smart Scan to run automatically at an interval and get the result via notification,
So that maintenance happens without me remembering (FR18).

**Acceptance Criteria:**

**Given** Settings → Cập nhật (or a Schedule section)
**When** rendered
**Then** it offers "Tự động Smart Scan": Never / Daily / Weekly (default Never)

**Given** a schedule is set
**When** the in-process timer (D6) reaches the interval
**Then** it runs Smart Scan in the background (no window needed) and, on completion, fires a notification (Story 7.3) "Smart Scan xong — Health Score {n}, có thể dọn {x} GB"

**Given** the app is closed to tray (Story 7.1)
**When** the scheduled time arrives
**Then** the scan still runs (tray process alive)

**Given** the app is fully quit
**When** the scheduled time arrives
**Then** the scan does NOT run (v1 uses in-process timer per D6; launchd agent deferred) — this limitation is documented in the schedule UI ("Chỉ chạy khi app đang mở hoặc ở menubar")

**Given** the user clicks the result notification
**When** it fires
**Then** the app opens to the Smart Scan result view

**Given** a scheduled scan coincides with a manual scan
**When** both would run
**Then** the scheduler skips its run if a scan is already in progress (no concurrent scans)

---

## Epic 8: Focus + Cleanup Integration

User bật "cleanup during focus" toggle trong focus timer settings → trong Pomodoro session app dọn nền các category 🟢 Safe lane (default, customizable). End-of-session notification "đã giải phóng X GB", không bao giờ interrupt focus session. Differentiator độc nhất của `tiny`.

### Story 8.1: Focus-cleanup engine hook

As an engine developer,
I want the focus timer in `tiny-core` to expose lifecycle hooks (session start / tick / end),
So that the app layer can trigger background cleanup during a focus session without coupling the timer to cleanup logic.

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/focus/`
**When** inspected
**Then** the focus session exposes a callback interface `FocusEvents { on_start: Fn(SessionMeta), on_end: Fn(SessionResult) }` (or an event enum returned via the existing progress-style callback)

**Given** the timer remains pure (Story 1.4)
**When** inspected
**Then** the hook does NOT perform cleanup itself — it only signals; the actual cleanup is orchestrated by the src-tauri layer

**Given** a focus session starts
**When** the `on_start` hook fires
**Then** it carries `SessionMeta { duration_secs, started_at }`

**Given** a focus session ends (or is cancelled)
**When** the `on_end` hook fires
**Then** it carries `SessionResult { completed: bool, elapsed_secs }` so the app can decide whether to report cleanup results

**Given** unit tests
**When** run
**Then** they assert hooks fire in order start → end for a completed session, and start → end(completed=false) for a cancelled one, with no sleep

### Story 8.2: "Cleanup during focus" settings toggle + category selection

As a user,
I want to enable cleanup-during-focus and choose which categories run, defaulting to safe lane only,
So that I control what gets cleaned in the background (FR35, FR37).

**Acceptance Criteria:**

**Given** Settings → a "Focus" section (or focus timer settings)
**When** rendered
**Then** it shows a toggle "Dọn dẹp trong lúc focus" default OFF

**Given** the toggle is ON
**When** the category selector renders
**Then** it lists cleanup categories grouped by lane, with 🟢 Safe lane all pre-selected and 🟡/🔴 unselected; the user may add 🟡 but a warning notes destructive categories are excluded by default (FR37)

**Given** the user selects categories
**When** the selection persists
**Then** it is stored via the settings store (consumed by the focus-cleanup execution story later in this epic)

**Given** the user has not enabled the toggle
**When** a focus session runs
**Then** no background cleanup occurs (feature fully opt-in)

**Given** the equivalent CLI footer
**When** the selection changes
**Then** it shows `tiny focus --cleanup --category=...` (forward-declared if CLI flag not yet implemented)

### Story 8.3: Background cleanup execution during focus session

As a user,
I want safe-lane cleanup to run quietly in the background during my Pomodoro session,
So that cleanup becomes a side effect of working, never an interruption (FR35).

**Acceptance Criteria:**

**Given** the focus session starts and cleanup-during-focus is enabled
**When** the `on_start` hook (Story 8.1) fires
**Then** the app schedules cleanup of the selected categories on a background thread using the hybrid delete layer (Epic 3)

**Given** the cleanup runs during focus
**When** it executes
**Then** it NEVER shows a modal, dialog, or notification mid-session, and it does not steal focus or play sounds (non-interrupting requirement)

**Given** a destructive category was somehow selected
**When** background cleanup runs
**Then** it still routes through quarantine/Trash (never hard-delete) and respects FR14 (skip running apps) and FR13 (symlink safety)

**Given** the focus session is cancelled early
**When** the `on_end(completed=false)` fires
**Then** any in-flight cleanup completes its current file then stops gracefully (no corrupt state, NFR-R4)

**Given** cleanup encounters an error during focus
**When** it fails
**Then** the error is logged and surfaced only at session end (see the end-of-session notification story), never as a mid-session popup

### Story 8.4: End-of-session cleanup notification

As a user,
I want a single end-of-session notification summarizing what was cleaned,
So that I see the benefit without being interrupted during focus (FR36).

**Acceptance Criteria:**

**Given** the focus session ends and background cleanup ran
**When** the `on_end` hook fires
**Then** a single notification (Story 7.3) shows "Focus xong — đã giải phóng {x} GB từ {n} mục" with a "Xem chi tiết" action

**Given** nothing was cleaned (no reclaimable space)
**When** the session ends
**Then** the notification reads "Focus xong — không có gì cần dọn" (or is suppressed per a setting), never a misleading "0 GB freed" alarm

**Given** the user clicks "Xem chi tiết"
**When** the action fires
**Then** the app opens the `CleanupDiffReport` (Story 3.9) for that focus session, recorded as a normal cleanup session in the journal

**Given** the cleanup is fully undoable
**When** the user reviews the session
**Then** Cmd+Z / History restore (Story 3.10) works identically to a manual cleanup session

**Given** cleanup errored during the session
**When** the notification fires
**Then** it reports partial results honestly ("đã giải phóng {x} GB, {k} mục gặp lỗi") with details available in the report

---

## Epic 9: Maintenance Scripts

User free RAM (purge inactive memory + RAM trước/sau), run macOS periodic maintenance scripts (flush DNS / reindex Spotlight / repair disk permissions / rebuild Launch Services) individually hoặc batch. Manage Login Items + Launch Agents qua unified panel (`SMAppService`).

### Story 9.1: Maintenance engine module — typed operations

As an engine developer,
I want a `tiny-core/src/maintenance/` module exposing each maintenance task as a typed, runtime-agnostic operation,
So that the GUI and CLI can run them consistently with structured results.

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/maintenance/mod.rs`
**When** inspected
**Then** it defines `pub enum MaintenanceTask { FreeRam, FlushDns, ReindexSpotlight, RepairDiskPermissions, RebuildLaunchServices }` and `pub fn run(task: MaintenanceTask) -> Result<MaintenanceReport, CoreError>`

**Given** each task is implemented via a documented macOS command
**When** inspected
**Then** the module documents the underlying mechanism per task (e.g., FlushDns → `dscacheutil -flushcache` + `killall -HUP mDNSResponder`; ReindexSpotlight → `mdutil -E /`) and notes which require elevated privileges

**Given** a task requires `sudo`
**When** `run` is called without privileges
**Then** it returns `CoreError::RequiresPrivilege(task)` rather than silently failing — the GUI layer handles privilege elevation

**Given** the architecture invariant (no presentation/log in core)
**When** inspected
**Then** the module returns structured `MaintenanceReport { task, success, detail, before, after }` and emits no `println!`/log

**Given** unit/integration tests
**When** run on CI
**Then** non-destructive tasks (FreeRam dry, FlushDns) are smoke-tested; privileged/destructive tasks are gated behind a `--ignored` flag to avoid mutating the CI runner

### Story 9.2: Free up RAM action + before/after UI

As a user,
I want to purge inactive memory with one action and see RAM usage before and after,
So that I can quickly recover memory and see the effect (FR43).

**Acceptance Criteria:**

**Given** `features/maintenance/free-ram.tsx`
**When** rendered
**Then** it shows current RAM used/total and a "Giải phóng RAM" button

**Given** the user clicks "Giải phóng RAM"
**When** the action runs `MaintenanceTask::FreeRam` (which invokes `purge`)
**Then** the UI captures RAM before, runs the task, captures RAM after, and shows a before/after comparison (e.g., "Trước: 13.2 GB → Sau: 9.8 GB, giải phóng 3.4 GB")

**Given** `purge` requires privilege on some macOS versions
**When** elevation is needed
**Then** the app prompts for authorization via the macOS auth dialog; on denial it shows a clear message, no crash

**Given** the result is negligible (< 100 MB freed)
**When** displayed
**Then** the copy is honest ("Giải phóng không đáng kể — RAM đang được dùng hiệu quả"), no exaggeration (UX-DR18)

**Given** the equivalent CLI footer
**When** on this view
**Then** it shows `tiny maintenance free-ram`

### Story 9.3: Periodic maintenance scripts panel — individual + batch toggles

As a user,
I want to run macOS maintenance scripts individually or as a batch, each as a separate toggle,
So that I can perform standard system upkeep without the Terminal (FR44).

**Acceptance Criteria:**

**Given** `features/maintenance/scripts-panel.tsx`
**When** rendered
**Then** it lists each task (Flush DNS, Reindex Spotlight, Repair Disk Permissions, Rebuild Launch Services) with an individual toggle/run button and a description of what it does + risk level

**Given** the user selects multiple tasks and clicks "Chạy tất cả đã chọn"
**When** the batch runs
**Then** tasks execute sequentially with per-task progress, and a summary report shows success/failure per task

**Given** a task requires privilege
**When** it runs
**Then** the macOS auth dialog appears once for the batch (where possible) rather than per-task

**Given** a destructive/long task (e.g., Reindex Spotlight can take minutes)
**When** selected
**Then** a `DestructiveConfirmDialog` (Story 3.8) warns about the duration/impact before running

**Given** a task fails
**When** the batch continues
**Then** the failure is recorded, the batch proceeds with remaining tasks, and the summary clearly marks which failed and why

**Given** the equivalent CLI footer
**When** tasks are selected
**Then** it shows `tiny maintenance run --task=flush-dns,reindex-spotlight`

### Story 9.4: Login Items + Launch Agents management panel

As a user,
I want to view and disable Login Items and Launch Agents in one panel,
So that I can control what starts at boot without digging through System Settings (FR45).

**Acceptance Criteria:**

**Given** `features/maintenance/login-items.tsx`
**When** rendered
**Then** it lists Login Items and Launch Agents/Daemons with name, source path, type, and enabled state, read via `SMAppService` (macOS 13+) and `~/Library/LaunchAgents` enumeration

**Given** the user toggles an item off
**When** the action runs
**Then** the corresponding `SMAppService` is unregistered (or the LaunchAgent plist disabled via `launchctl`), and the list reflects the new state

**Given** a system-critical agent
**When** listed
**Then** it is marked protected and cannot be disabled (toggle disabled with explanation)

**Given** disabling requires privilege
**When** needed
**Then** the macOS auth dialog appears; on denial the item remains unchanged with a clear message

**Given** an item path no longer exists (orphan agent)
**When** listed
**Then** it is flagged "Orphan — app đã gỡ" with a one-click "Xoá mục mồ côi" action

**Given** the equivalent CLI footer
**When** on this view
**Then** it shows `tiny maintenance login-items list` (and `--disable <id>` when toggling)

---

## Epic 10: Protection — Malware, Privacy, App Updater, Permissions Audit

Scan PUP/adware signatures → quarantine matches. Privacy cleaner (browser history/cookies/recent items/saved Wi-Fi). Detect outdated apps non-MAS + offer update. Permissions audit qua `tccutil` — list apps giữ camera/mic/FDA/Accessibility, revoke selectively. Per-category permission prompts khi category dùng lần đầu.

### Story 10.1: PUP/adware signature scan + quarantine matches

As a user,
I want the app to scan for known PUP/adware signatures and quarantine any matches,
So that I can remove unwanted software safely (FR46).

**Acceptance Criteria:**

**Given** `crates/tiny-core/src/protection/signatures.rs`
**When** inspected
**Then** it loads a signature database (bundled JSON/YAML of known PUP/adware paths + bundle IDs + file hashes) and exposes `pub fn scan_signatures(roots: &[Path], db: &SignatureDb) -> Result<Vec<ThreatMatch>, CoreError>`

**Given** a signature match
**When** found
**Then** `ThreatMatch { path, signature_id, severity, description }` is returned; the scan is read-only (no auto-removal)

**Given** the user reviews matches in `features/protection/malware-scan.tsx`
**When** they choose to remove
**Then** matched files are routed through the hybrid delete layer (Epic 3) into quarantine (never hard-delete), with a `DestructiveConfirmDialog` first

**Given** the signature DB needs updates
**When** the app checks
**Then** the DB version is shown; updates are fetched only with user consent (respecting NFR-Pr1 zero-default-network) and verified by checksum before use

**Given** false positives are possible
**When** a match is shown
**Then** the UI shows the signature description + path and a "Đánh dấu an toàn (whitelist)" option, so the user is never forced to trust blindly

**Given** the signature scan must not block
**When** it runs
**Then** it executes on a background thread with progress events (Story 1.8 pattern)

### Story 10.2: Privacy cleaner module

As a user,
I want to clear browser history, cookies, recent items, and saved Wi-Fi networks,
So that I can remove privacy-sensitive traces (FR47).

**Acceptance Criteria:**

**Given** `features/protection/privacy-cleaner.tsx`
**When** rendered
**Then** it lists privacy categories: browser history, cookies, macOS recent items, saved Wi-Fi networks — each with a separate checkbox and reclaimable/affected count

**Given** browser categories
**When** scanned
**Then** the cleaner detects installed browsers (Safari, Chrome, Firefox, Arc...) and shows per-browser breakdown; only detected browsers appear

**Given** the user selects categories and runs the cleaner
**When** it executes
**Then** removable files go through the safety net (quarantine where reversible); irreversible items (e.g., cookies) require an explicit `DestructiveConfirmDialog` clearly stating they cannot be undone

**Given** saved Wi-Fi networks
**When** the user removes one
**Then** it requires privilege (auth dialog) and clearly warns the device will forget the network

**Given** an active browser
**When** the user tries to clear its data
**Then** the cleaner detects the running browser and warns (FR14-style) "Đóng {browser} trước khi dọn để tránh lỗi" rather than corrupting an open profile

**Given** the equivalent CLI footer
**When** categories are selected
**Then** it shows `tiny privacy clean --category=browser-history,cookies`

### Story 10.3: App updater non-MAS detection + offer update

As a user,
I want the app to detect outdated apps installed outside the Mac App Store and offer to update them,
So that I can keep third-party apps current (FR48).

**Acceptance Criteria:**

**Given** `features/protection/app-updater.tsx`
**When** it scans `/Applications`
**Then** it lists non-MAS apps with current version vs latest known version, detected via Homebrew Cask (`brew outdated --cask`) and/or Sparkle appcast feeds (NFR-I5)

**Given** an app has an available update
**When** displayed
**Then** the row shows current → latest version and an "Cập nhật" button; MAS apps are excluded (directed to App Store)

**Given** the user clicks "Cập nhật" for a Homebrew-managed app
**When** the action runs
**Then** it invokes `brew upgrade --cask <token>` and shows progress; on completion the version refreshes

**Given** an app is not managed by Homebrew or Sparkle
**When** listed
**Then** it shows "Không tự cập nhật được" with a link to the app's download page rather than a non-functional button

**Given** update detection requires network
**When** offline
**Then** the panel shows cached results with a "Không kết nối — hiển thị dữ liệu cũ" banner, never an error wall

**Given** the equivalent CLI footer
**When** on this view
**Then** it shows `tiny apps outdated` (and `tiny apps update <name>`)

### Story 10.4: Permissions audit (TCC) panel — list + revoke

As a user,
I want to see which apps hold sensitive permissions (camera, mic, Full Disk Access, Accessibility) and revoke selectively,
So that I can audit and tighten my privacy posture (FR49).

**Acceptance Criteria:**

**Given** `features/protection/permissions-audit.tsx`
**When** it loads
**Then** it lists permission categories (Camera, Microphone, Full Disk Access, Accessibility, Screen Recording) and the apps holding each, read via `tccutil`/TCC database (read-only enumeration)

**Given** the TCC database read requires FDA
**When** FDA is missing
**Then** the panel shows an inline prompt to grant FDA (reusing Story 1.9 onboarding) rather than failing silently

**Given** the user wants to revoke a permission
**When** they click "Thu hồi"
**Then** the app runs `tccutil reset <service> <bundle-id>` (with auth) and confirms; the list refreshes to reflect the change

**Given** revoking a permission is impactful
**When** the user clicks revoke
**Then** a `DestructiveConfirmDialog` warns "App {name} sẽ mất quyền {permission} và có thể không hoạt động đúng" before applying

**Given** a system-protected entry
**When** listed
**Then** it is marked non-revocable with explanation

**Given** the equivalent CLI footer
**When** on this view
**Then** it shows `tiny permissions audit` (and `tiny permissions revoke <app> <permission>`)

### Story 10.5: Per-category permission prompts

As a user,
I want the app to request additional permissions only when I first use a feature that needs them,
So that I'm not asked for everything upfront (FR51).

**Acceptance Criteria:**

**Given** a feature requiring a specific permission (e.g., Login Items needs Accessibility, permissions audit needs FDA)
**When** the user first invokes that feature
**Then** the app detects the missing permission and shows a focused prompt explaining why it's needed + how to grant it

**Given** the permission is already granted
**When** the feature is used
**Then** no prompt appears (silent success)

**Given** the user declines to grant
**When** they cancel the prompt
**Then** the feature is gracefully disabled with a "Cần quyền {x} để dùng" state — the rest of the app keeps working

**Given** a central permission registry
**When** inspected
**Then** there is a single mapping `feature → required_permission` in code, so adding a new permission-gated feature is one entry (maintainability)

**Given** the user grants the permission then returns
**When** they retry the feature
**Then** a "Kiểm tra lại" affordance re-detects the now-granted permission without an app restart

**Given** accessibility
**When** any permission prompt renders
**Then** it follows the FDA onboarding a11y standard (large text, clear Vietnamese action verbs, keyboard navigable)
