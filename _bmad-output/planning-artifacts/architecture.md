---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments:
  - "_bmad-output/brainstorming/brainstorming-session-2026-05-07-190913.md"
workflowType: "architecture"
lastStep: 8
status: "complete"
completedAt: "2026-05-18"
project_name: "tiny-cli"
user_name: "Kyantran"
date: "2026-05-17"
prd_status: "Không có PRD chính thức — dùng brainstorm doc làm requirements input (user xác nhận 2026-05-17)"
---

# Architecture Decision Document

_Tài liệu này xây dựng dần qua từng bước thảo luận. Các section được nối thêm khi đi qua mỗi quyết định kiến trúc._

## Bối cảnh

**Sản phẩm:** macOS native app cho `tiny-cli` — app GUI kiểu CleanMyMac, engine bên dưới là Rust CLI `tiny` hiện có (`sys`, `scan`, `focus`, `uninstall`, `clean`).

**Đầu vào:** Phiên brainstorm `_bmad-output/brainstorming/brainstorming-session-2026-05-07-190913.md` — 68 ý tưởng, scope v1 = Nhóm 1+2+3+4, kèm một MVP slice 5 bước.

**Câu hỏi kiến trúc trọng tâm:**

1. Chốt tech stack lớp GUI (SwiftUI native / Tauri / Rust-native egui).
2. Kế hoạch tách engine Rust thành library crate để cả CLI lẫn GUI cùng dùng.

## Project Context Analysis

### Requirements Overview

**Functional Requirements** (scope v1 = Nhóm 1–4 từ brainstorm doc):

- **N1 — Bộ mặt zero-decision:** Smart Scan 1 nút, Health Score 0–100, GUI cho `clean` với làn 3 màu, AI xếp hạng category, hiển thị dung lượng bằng ngôn ngữ đời thường.
- **N2 — Space Lens:** treemap toàn ổ đĩa, sunburst, heat-by-age/type, drill-down + breadcrumb, phantom space.
- **N3 — An toàn & minh bạch:** xem trước khi xoá, quarantine có hoàn tác, tích hợp Trash macOS, undo stack, phô command `tiny` tương đương.
- **N4 — Chủ động & giải thích:** menubar widget realtime, scan nền, dự báo đầy ổ, "tuần này có gì đổi", rules engine, notifications.

**Non-Functional Requirements** (driver kiến trúc chính):

- **macOS native** — menubar agent, notifications, xin quyền Full Disk Access.
- **Hiệu năng** — quét toàn ổ song song, không block UI; monitor realtime cần luồng nền.
- **An toàn** — mọi thao tác xoá phải đảo ngược được (quarantine/Trash).
- **Tái dùng engine** — logic CLI phải tách thành library crate cho cả CLI + GUI.
- **Minh bạch** — GUI hiển thị command `tiny ...` tương đương.
- **Chạy nền** — menubar app + scheduled scan + FSEvents watch.

**Scale & Complexity:**

- Primary domain: desktop app macOS native + systems programming.
- Complexity level: trung bình–cao — không vì logic nghiệp vụ, mà vì ranh giới FFI/IPC, long-running ops, quyền hệ thống, vòng đời tiến trình nền.
- Component kiến trúc ước tính: ~5–6 (engine library, CLI binary, GUI shell, background agent, IPC layer, permissions handling).

### Technical Constraints & Dependencies

- Codebase hiện tại: Rust **binary crate** (`tiny`), deps `clap` / `anyhow` / `serde` / `sysinfo` / `dialoguer`.
- `scan` đã có `--json` → compute tách được khỏi presentation (tín hiệu tốt cho việc tách library).
- `clean` dùng `dialoguer` cho prompt tương tác → compute + UI dính chặt; đây là chỗ cần refactor mạnh nhất.
- macOS-only (đúng định hướng — `clean` vốn đã macOS-specific).
- App cần quyền Full Disk Access; nếu phát hành cần code signing + notarization.

### Cross-Cutting Concerns Identified

- Ranh giới API của library crate (engine) — hợp đồng dùng chung giữa CLI và GUI.
- Progress reporting cho long-running operations (scan/clean chậm) — cần callback/stream.
- IPC giữa GUI shell và Rust engine — hình thái phụ thuộc lựa chọn tech stack (cùng process / FFI / Tauri command).
- Mô hình quyền hệ thống (Full Disk Access...).
- Quản lý state undo/quarantine.
- Vòng đời tiến trình nền (menubar agent, launch agent).

## Starter Template Evaluation

### Primary Technology Domain

**Desktop app (macOS)** — lớp GUI bằng **Tauri 2**, backend Rust tái dùng trực tiếp engine `tiny`.

### Starter Options Considered

| Lựa chọn       | Phiên bản           | Kết quả                                                                            |
| -------------- | ------------------- | ---------------------------------------------------------------------------------- |
| **Tauri 2**    | 2.11.2 (2026-05-16) | ✅ Chọn — engine là Cargo dependency (không FFI), IPC built-in, treemap dễ với web |
| SwiftUI native | macOS SDK           | ❌ Loại — cần FFI bridge + 2 toolchain, đi ngược NFR "tái dùng engine"             |
| egui / iced    | 0.34.2 / 0.14.0     | ❌ Loại — native feel thấp nhất, menubar bolt-on, treemap tự vẽ                    |

### Selected Starter: `create-tauri-app` (Tauri 2.11.2)

**Rationale for Selection:**
2 NFR nặng ký nhất của dự án — tái dùng engine Rust và IPC GUI↔engine — được Tauri xử lý gần như miễn phí: engine chỉ là một crate trong Cargo workspace, IPC chính là command system của Tauri. Treemap (Nhóm 2) dễ nhất với web (d3.js). Đánh đổi chấp nhận được: cần kỹ năng frontend web (user đã quen React) và native feel "rất tốt" thay vì "tuyệt đối".

**Initialization Command:**

```bash
npm create tauri-app@latest
# Frontend: React · Flavor: TypeScript · Bundler: Vite
```

**Architectural Decisions Provided by Starter:**

| Hạng mục           | Quyết định                                                     |
| ------------------ | -------------------------------------------------------------- |
| Ngôn ngữ           | Rust (backend `src-tauri/`) + TypeScript (frontend)            |
| Frontend framework | React 19 — tận dụng kỹ năng React sẵn có của user              |
| Build tooling      | Vite (frontend) + Cargo (backend)                              |
| IPC                | Tauri command/event (`#[tauri::command]` ↔ `invoke()`)         |
| Đóng gói           | Tauri bundler → `.app` / `.dmg`, có slot cấu hình code signing |
| Dev experience     | `npm run tauri dev` — hot reload cả hai phía                   |

**Cần bổ sung (starter không có sẵn):**

- Styling solution — đề xuất Tailwind CSS (khớp thói quen React của user).
- Testing framework — frontend (Vitest) + backend (cargo test sẵn có).
- Thư viện treemap — d3.js (hoặc d3-hierarchy gọn hơn).

**Tích hợp với repo hiện tại:**
Repo `tiny-cli` chuyển thành Cargo **workspace** — engine tách ra `crates/tiny-core`, CLI hiện tại thành `crates/tiny` (cùng dùng core), Tauri app ở `src-tauri/`. Chi tiết cấu trúc module chốt ở bước Architectural Decisions.

**Note:** Lệnh khởi tạo + dựng workspace nên là story triển khai đầu tiên.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):** D1 Ranh giới engine · D2 Long-running ops · D7 Quyền Full Disk Access.
**Important Decisions (Shape Architecture):** D3 State frontend · D4 Persistence · D5 Cơ chế xoá · D6 Menubar/nền · D8 Error handling.
**Deferred Decisions (Post-MVP):** D10 Code signing & notarization (dev dùng ad-hoc signing); tách `launchd` agent riêng cho scheduled scan.

### D1 — Ranh giới engine (`tiny-core`)

- **Quyết định:** Repo thành Cargo workspace 3 crate — `crates/tiny-core` (engine library), `crates/tiny` (CLI binary, code hiện tại), `src-tauri` (Tauri app). Cả `tiny` và `src-tauri` phụ thuộc `tiny-core`.
- **Quy tắc API:** `tiny-core` chỉ phơi hàm trả struct serde-serializable. **Cấm `println!`, cấm `dialoguer`** trong core — mọi hiển thị/prompt thuộc về bên gọi (CLI text, GUI, hay JSON).
- **Rationale:** `scan` đã có `--json` chứng tỏ compute tách được. `clean` dính `dialoguer` là việc refactor nặng nhất.
- **Affects:** toàn bộ codebase; là prerequisite cho mọi thứ khác.

### D2 — Long-running operations & progress reporting

- **Quyết định:** Hàm trong `tiny-core` là **sync**, nhận một progress callback tuỳ chọn (`Fn(Progress)`). Core không phụ thuộc async runtime.
- Lớp Tauri command chạy hàm core trên **thread nền**, chuyển progress callback → **Tauri events** (`app.emit()`); frontend subscribe bằng `listen()`. `tokio` 1.52 chỉ tồn tại ở lớp `src-tauri`.
- **Rationale:** giữ core runtime-agnostic → CLI không phải kéo theo tokio; vẫn cho GUI cập nhật tiến độ realtime.
- **Affects:** mọi luồng scan/clean; thiết kế API của core.

### D3 — Frontend state management

- **Quyết định:** **TanStack Query 5.100** cho dữ liệu fetch từ backend qua `invoke` (caching, loading, refetch) + **Zustand 5.0** cho state UI thuần.
- **Rationale:** mỗi lời gọi `invoke` là một request/response — TanStack Query map thẳng vào đó. Zustand lo phần UI nhẹ.
- **Affects:** toàn bộ frontend React.

### D4 — Local persistence

- **Quyết định:** **SQLite** (`rusqlite` 0.39) cho lịch sử scan time-series + bản ghi quarantine/undo + rules. **`tauri-plugin-store`** cho settings dạng key-value.
- DB thuộc **lớp app** (`src-tauri`), **không** nằm trong `tiny-core` — core vẫn stateless.
- **Rationale:** truy vấn "tuần này có gì đổi" [#11] và so sánh snapshot [#37] cần query time-series → SQLite. Settings đơn giản → KV store.
- **Affects:** N4 (chủ động & giải thích), N3 (quarantine).

### D5 — Cơ chế xoá file & hoàn tác

- **Quyết định:** **Hybrid** — file thường → macOS Trash (qua crate `trash` 5.2, undo = Finder "Put Back"); mục Trash không nhận → thư mục **quarantine riêng** của app, retention 30 ngày, restore trong app. Metadata mọi thao tác xoá ghi vào SQLite.
- **Rationale:** dung hoà [#44] (Trash thật, quen thuộc) và [#9] (quarantine có kiểm soát).
- **Affects:** N3 (an toàn & hoàn tác); undo stack [#50].

### D6 — Menubar & tiến trình nền

- **Quyết định:** v1 dùng **một tiến trình Tauri** kèm tray icon. Đóng cửa sổ → ẩn xuống tray, không thoát. Scheduled scan [#20] = timer trong tiến trình.
- Tách `launchd` agent riêng → hoãn sau v1.
- **Affects:** N4 (menubar widget [#2], scan nền [#43]).

### D7 — Quyền macOS (Full Disk Access)

- **Quyết định:** App phát hiện trạng thái FDA lúc khởi động → nếu thiếu, hiện màn **onboarding** hướng dẫn user mở System Settings cấp quyền. Không thể cấp bằng code.
- **Affects:** mọi luồng scan/clean chạm vùng hệ thống; là critical path.

### D8 — Error handling

- **Quyết định:** `tiny-core` dùng `thiserror` 2.0 (typed errors). Tauri command trả `Result<T, ErrorPayload>` serializable. CLI binary dùng `anyhow` ở ranh giới ngoài cùng.
- **Affects:** API của core; lớp command.

### D9 — Testing

- **Quyết định:** Core + CLI: `cargo test` (đã có `assert_cmd`/`predicates`, bổ sung unit test cho `tiny-core`). Frontend: **Vitest**.

### D10 — Code signing & distribution (deferred)

- Hoãn sau MVP. Dev dùng ad-hoc signing; phát hành cần Developer ID + notarization — quyết định sau.

### Decision Impact Analysis

**Implementation Sequence:**

1. D1 — dựng workspace + tách `tiny-core` (prerequisite tuyệt đối).
2. D8 — chốt kiểu error trong khi tách core.
3. D2 — thêm progress callback vào API core.
4. Khởi tạo `src-tauri` + frontend (D3), nối IPC.
5. D7 — onboarding quyền FDA (chặn mọi tính năng quét).
6. D4, D5 — persistence + cơ chế xoá khi làm tính năng `clean`.
7. D6 — tray/menubar khi làm N4.

**Cross-Component Dependencies:**

- D2 (progress callback) thay đổi chữ ký hàm core → phải chốt cùng lúc tách core ở D1.
- D4 (SQLite) và D5 (quarantine metadata) dùng chung schema → thiết kế cùng nhau.
- D7 (FDA) chặn đường mọi tính năng quét → ưu tiên sớm dù không "hấp dẫn".

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

Các vùng AI agent dễ chọn khác nhau gây xung đột — đã chốt convention thống nhất bên dưới. Điểm rủi ro cao nhất: ranh giới serde camelCase giữa Rust và TypeScript.

### Naming Patterns

| Vùng          | Quy tắc                                                                                 |
| ------------- | --------------------------------------------------------------------------------------- |
| Rust          | `snake_case` cho hàm/file/module, `PascalCase` cho type. Crate: `tiny-core`, `tiny`     |
| React / TS    | File **kebab-case** (`health-score.tsx`), component `PascalCase`, hàm `camelCase`       |
| SQLite        | Bảng + cột `snake_case`, bảng dùng số nhiều (`scan_snapshots`), khoá ngoại `<table>_id` |
| Tauri command | Tên hàm `snake_case` — `invoke("run_smart_scan")`                                       |
| Tauri event   | `domain:action` — `scan:progress`, `scan:done`, `clean:progress`, `clean:done`          |

### Format Patterns

⚠️ **Điểm dễ xung đột nhất:**

- **Mọi struct đi qua IPC** bắt buộc `#[serde(rename_all = "camelCase")]` — Rust giữ `snake_case` nội bộ, frontend nhận `camelCase` idiomatic.
- **Lỗi:** Tauri command trả `Result<T, ErrorPayload>`; `ErrorPayload = { code: string, message: string }`.
- **Thời gian:** IPC truyền **ISO 8601 string**; SQLite lưu **Unix epoch INTEGER**.
- **Dung lượng:** truyền **số byte thô** (`u64` → JS number — an toàn tới 9 PB), format ở frontend.

### Structure Patterns

- Rust unit test: `#[cfg(test)]` inline trong file; integration test: thư mục `tests/` (giữ pattern `assert_cmd` sẵn có).
- Frontend: tổ chức **theo feature**, test co-located `*.test.ts` (Vitest), pure function tách ra `lib/`.
- Tauri command đặt ở `src-tauri/src/commands/`, mỗi domain một file.

### Communication & Process Patterns

- **Immutability tuyệt đối** — Zustand update bất biến, không mutate tại chỗ.
- Long-running operation **không bao giờ block UI** — luôn chạy thread nền + emit progress event.
- TanStack Query key dạng mảng: `['scan', 'result']`, `['health', 'score']`.
- Loading/error đi qua TanStack Query (`isPending`, `error`) + React error boundary.
- App layer (`src-tauri`) log bằng `tracing`; `tiny-core` **không log** — chỉ trả data/error.

### Enforcement Guidelines

**Mọi AI agent BẮT BUỘC:**

1. Struct đi qua IPC luôn có `#[serde(rename_all = "camelCase")]`.
2. `tiny-core` không `println!`, không `dialoguer`, không log, không I/O ra stdout.
3. Mọi thao tác xoá đi qua lớp delete (Trash/quarantine) — **cấm** gọi `fs::remove_*` trực tiếp.
4. Long-operation luôn chạy thread nền + progress event, không block UI thread.

**Anti-patterns cần tránh:**

- Trả struct `snake_case` thô qua IPC (frontend phải xài field `snake_case` lạc lõng).
- Đặt logic nghiệp vụ trong Tauri command thay vì `tiny-core` (CLI mất đồng bộ).
- `unwrap()` trong command layer thay vì map sang `ErrorPayload`.

## Project Structure & Boundaries

### Complete Project Directory Structure

```
tiny-cli/                          # workspace root (Cargo workspace + npm project)
├── Cargo.toml                     # [workspace] members: crates/*, src-tauri
├── package.json                   # React, Vite, TanStack Query, Zustand, d3-hierarchy, Tailwind, Vitest
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.ts
├── index.html
├── README.md
│
├── crates/
│   ├── tiny-core/                 # ENGINE LIBRARY — không I/O presentation, không log
│   │   ├── Cargo.toml             # deps: serde, sysinfo, thiserror (KHÔNG tauri/tokio/rusqlite)
│   │   ├── src/
│   │   │   ├── lib.rs             # public API surface
│   │   │   ├── sys/               # system info
│   │   │   ├── scan/              # disk scan + duplicates + walker toàn ổ
│   │   │   ├── clean/             # cleanup engine + providers/
│   │   │   ├── uninstall/         # app uninstall
│   │   │   ├── focus/             # focus timer
│   │   │   ├── progress.rs        # type Progress + progress callback trait
│   │   │   └── error.rs           # thiserror error types
│   │   └── tests/                 # integration test cho core
│   │
│   └── tiny/                      # CLI BINARY (code hiện tại)
│       ├── Cargo.toml             # bin "tiny", deps: tiny-core, clap, anyhow, dialoguer
│       ├── src/
│       │   ├── main.rs
│       │   ├── cli.rs             # định nghĩa clap
│       │   └── render/            # text/json output — println! + dialoguer SỐNG Ở ĐÂY
│       └── tests/                 # assert_cmd integration tests (giữ nguyên)
│
├── src-tauri/                     # TAURI APP CRATE
│   ├── Cargo.toml                 # deps: tiny-core, tauri, rusqlite, trash, tokio, tracing
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── capabilities/              # Tauri 2 permission capabilities
│   ├── icons/
│   └── src/
│       ├── main.rs                # entry, khởi tạo app + tray
│       ├── commands/              # #[tauri::command] — mỗi domain 1 file
│       │   ├── mod.rs
│       │   ├── health.rs          # Smart Scan + Health Score
│       │   ├── scan.rs
│       │   ├── clean.rs
│       │   ├── space_lens.rs
│       │   └── system.rs          # sys monitor + permissions
│       ├── db/                    # SQLite (rusqlite) — CHỈ ở lớp app
│       │   ├── mod.rs
│       │   ├── schema.rs          # migrations
│       │   ├── scan_history.rs
│       │   └── quarantine.rs
│       ├── delete.rs              # lớp xoá hybrid: Trash + quarantine
│       ├── permissions.rs         # phát hiện Full Disk Access
│       ├── tray.rs                # menubar tray
│       └── events.rs              # helper emit progress event
│
├── src/                           # REACT FRONTEND (src/ gốc — convention Tauri)
│   ├── main.tsx
│   ├── App.tsx
│   ├── index.css                  # Tailwind entry
│   ├── features/                  # tổ chức theo feature
│   │   ├── smart-scan/            # N1 — Health Score, nút Smart Scan
│   │   ├── cleanup/               # N1/N3 — GUI clean, làn 3 màu, xem trước
│   │   ├── space-lens/            # N2 — treemap
│   │   ├── monitor/               # N4 — realtime, dự báo, "tuần này có gì đổi"
│   │   ├── safety/                # N3 — undo, quarantine, phô command
│   │   └── onboarding/            # màn xin quyền Full Disk Access
│   ├── components/                # UI component dùng chung
│   ├── lib/                       # pure function — KHÔNG phụ thuộc React
│   │   ├── ipc.ts                 # typed invoke wrappers
│   │   ├── format.ts              # bytes → ngôn ngữ đời thường
│   │   └── treemap.ts             # d3-hierarchy layout (thuần)
│   ├── hooks/                     # TanStack Query hooks dùng chung
│   ├── stores/                    # Zustand stores
│   └── types/                     # TS types — mirror struct serde của backend
│
└── docs/
```

### Architectural Boundaries

**Crate boundaries:**

- `tiny-core` ← được `tiny` (CLI) và `src-tauri` cùng phụ thuộc. Core thuần: không `tauri`/`tokio`/`rusqlite`, không presentation, không log.
- `println!` + `dialoguer` chỉ tồn tại trong `crates/tiny/src/render/`.
- SQLite (`rusqlite`) chỉ tồn tại trong `src-tauri/src/db/` — không bao giờ trong core.

**IPC boundary:**

- Frontend ↔ backend chỉ qua Tauri IPC: lệnh đi qua `src/lib/ipc.ts` (typed wrapper) → `src-tauri/src/commands/`; sự kiện đi qua `src-tauri/src/events.rs` → frontend `listen()`.
- Không có đường nào khác giữa hai phía.

**Frontend dependency flow (CRITICAL):**

- Cho phép: `features/ → components/ → lib/`.
- Cấm: `lib/ → features/` hay `lib/ → components/` (kể cả type-only import).

### Requirements to Structure Mapping

| Nhóm v1         | Frontend                                          | Backend command                                       | Engine                            |
| --------------- | ------------------------------------------------- | ----------------------------------------------------- | --------------------------------- |
| N1 — Bộ mặt     | `features/smart-scan/`, `features/cleanup/`       | `commands/health.rs`, `commands/clean.rs`             | `tiny-core/clean/`, `sys/`        |
| N2 — Space Lens | `features/space-lens/`, `lib/treemap.ts`          | `commands/space_lens.rs`                              | `tiny-core/scan/` (walker toàn ổ) |
| N3 — An toàn    | `features/safety/`, `features/cleanup/` (preview) | `delete.rs`, `db/quarantine.rs`                       | `tiny-core/clean/`                |
| N4 — Chủ động   | `features/monitor/`                               | `commands/system.rs`, `tray.rs`, `db/scan_history.rs` | `tiny-core/sys/`, `scan/`         |

**Cross-cutting:**

- Quyền FDA: `src-tauri/src/permissions.rs` + `src/features/onboarding/`.
- Progress event: `src-tauri/src/events.rs` + các `listen()` ở `src/hooks/`.

### Integration Points & Data Flow

- **Data flow một luồng scan:** frontend gọi `ipc.ts` → `commands/scan.rs` spawn thread nền chạy `tiny-core::scan` (có progress callback) → callback đẩy qua `events.rs` (`scan:progress`) → frontend `listen()` cập nhật UI → kết quả cuối lưu `db/scan_history.rs`.
- **External:** chỉ macOS APIs (filesystem, FSEvents, Trash, FDA). Không backend mạng.

### Development Workflow

- `npm run tauri dev` — chạy Vite dev server + Tauri shell, hot reload cả hai phía.
- `cargo test` ở workspace — test `tiny-core` + `tiny`. `npm run test` — Vitest cho frontend.
- `npm run tauri build` — bundle `.app` / `.dmg`.

## Architecture Validation Results

### Coherence Validation ✅

- **Decision Compatibility:** Tauri 2.11 + React 19 + rusqlite 0.39 + trash 5.2 + tokio 1.52 + thiserror 2.0 — tương thích, không xung đột phiên bản.
- **Pattern Consistency:** convention naming/format/communication khớp với stack đã chọn; quy tắc serde camelCase nhất quán toàn IPC.
- **Structure Alignment:** workspace 3 crate + frontend feature-based đỡ được mọi quyết định D1–D10; ranh giới crate và IPC rõ ràng.

### Requirements Coverage Validation ✅

- **Feature Coverage:** cả 4 nhóm v1 (N1–N4) đều có đủ frontend feature + backend command + engine module (xem bảng mapping).
- **NFR Coverage:** macOS native (tray, onboarding FDA), an toàn (D5 hybrid delete), tái dùng engine (D1), chạy nền (D6) — đều có quyết định tương ứng.
- **Lưu ý:** N2 Space Lens (walker toàn ổ) là khối nặng nhất; brainstorm từng đề xuất v1.1 nhưng user chọn vào v1 — kiến trúc map đầy đủ.

### Implementation Readiness Validation ✅

- **Decision Completeness:** D1–D10 đã ghi kèm phiên bản đã xác minh từ crates.io/npm.
- **Structure Completeness:** cây thư mục đầy đủ, không placeholder; ranh giới component xác định rõ.
- **Pattern Completeness:** naming/format/communication/process đều có rule + anti-pattern.

### Gap Analysis Results

**Critical Gaps:** không có — bản thân kiến trúc đầy đủ và mạch lạc.

**Important Gaps:**

1. **Không có PRD** — requirements suy ra từ brainstorm doc, chưa validate chính thức (thiếu acceptance criteria, success metrics).
2. **Chiến lược song song hoá walker toàn ổ chưa chốt** — `rayon` / `jwalk` / thread thủ công (NFR hiệu năng).
3. **Bộ Tauri plugin chưa liệt kê** — `tauri-plugin-store`, notification, autostart (launch-at-login cho menubar).

**Nice-to-Have Gaps:**

- Library FSEvents (`notify` crate) cho live re-scan [#30] chưa chốt.

### Validation Issues Addressed

- **Rủi ro execution đã ghi nhận:** tách `tiny-core` khỏi binary crate — `clean` dính chặt `dialoguer` — là refactor nặng nhất; xử lý bằng cách đặt D1 làm story đầu tiên, tách dần từng command.
- Các Important Gap chuyển sang bước planning để giải quyết; không cản việc bắt đầu code D1.

### Architecture Completeness Checklist

**Requirements Analysis**

- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped

**Architectural Decisions**

- [x] Critical decisions documented with versions
- [x] Technology stack fully specified
- [x] Integration patterns defined
- [ ] Performance considerations addressed — _chiến lược song song hoá walker toàn ổ chưa chốt_

**Implementation Patterns**

- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process patterns documented

**Project Structure**

- [x] Complete directory structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements to structure mapping complete

### Architecture Readiness Assessment

**Overall Status:** READY WITH MINOR GAPS

**Confidence Level:** medium — kéo xuống vì thiếu PRD; các gap còn lại đều xử được ở bước planning, không cản việc bắt đầu code.

**Key Strengths:**

- Tauri thu gọn 2 NFR khó nhất (tái dùng engine + IPC) gần như miễn phí.
- Ranh giới crate sạch — `tiny-core` thuần làm cả CLI lẫn GUI dùng chung được.
- Cơ chế an toàn (hybrid delete + undo) bám sát insight chủ đạo của brainstorm.

**Areas for Future Enhancement:**

- Viết PRD chính thức để validate requirements.
- Chốt chiến lược song song hoá + bộ Tauri plugin trong bước planning.
- D10 code signing / notarization cho phát hành.

### Implementation Handoff

**AI Agent Guidelines:**

- Tuân thủ chính xác D1–D10 và các Implementation Pattern.
- `tiny-core` tuyệt đối không presentation/log/IO; mọi struct IPC `#[serde(rename_all = "camelCase")]`.
- Tôn trọng ranh giới crate và frontend dependency flow.
- Mọi câu hỏi kiến trúc tra cứu tài liệu này.

**First Implementation Priority:**

D1 — chuyển repo thành Cargo workspace, tách `tiny-core` (làm cùng D8 error types và D2 progress callback vì chúng đổi chữ ký API core). Sau đó mới `npm create tauri-app@latest`.
