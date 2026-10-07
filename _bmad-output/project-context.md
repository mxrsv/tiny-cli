---
project_name: "tiny-cli"
user_name: "Kyantran"
date: "2026-05-19"
sections_completed:
  [
    "technology_stack",
    "language_rules",
    "framework_rules",
    "testing_rules",
    "quality_rules",
    "workflow_rules",
    "anti_patterns",
  ]
status: "complete"
rule_count: 39
optimized_for_llm: true
existing_patterns_found: 8
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

### Hiện tại (workspace, migration desktop ngày 2026-10-06)

- **Rust 2021 edition** — workspace `crates/tiny-core`, `crates/tiny`, `src-tauri`; binary CLI tên `tiny`, desktop macOS-only
- `clap` 4.5 (feature `derive`) — định nghĩa CLI bằng struct derive
- `anyhow` 1.0 — error handling ở CLI boundary
- `serde` 1.0 (feature `derive`) + `serde_json` 1.0 — `scan --json`
- `sysinfo` 0.32 — đọc thông tin hệ thống cho lệnh `sys`
- `dialoguer` 0.11 — prompt tương tác cho lệnh `clean`
- Dev: `assert_cmd` 2 + `predicates` 3 — integration test

### Desktop đã triển khai (architecture.md — D1–D10)

- Cargo **workspace 3 crate**: `crates/tiny-core` · `crates/tiny` · `src-tauri`
- `tauri` 2.11.2 · React 19 · Vite · runtime Tauri/Tokio CHỈ ở `src-tauri` (lockfile chốt version)
- `rusqlite` 0.39 · `trash` 5.2 · `thiserror` 2.0
- Frontend: TanStack Query 5.100 · Zustand 5.0 · Tailwind · d3-hierarchy · Vitest

### Ràng buộc phiên bản

- `tiny-core` KHÔNG được phụ thuộc `tauri` / `tokio` / `rusqlite` — phải runtime-agnostic
- `dialoguer` chỉ sống ở lớp CLI render, KHÔNG vào core

## Critical Implementation Rules

### Language-Specific Rules (Rust)

- **Error handling 2 tầng:** CLI binary dùng `anyhow::Result` ở ranh giới ngoài cùng;
  `tiny-core` dùng `thiserror` typed errors. KHÔNG dùng `anyhow` trong core.
- **Cấm `unwrap()` / `expect()`** trên đường chạy thật — đặc biệt ở lớp Tauri command
  (phải map sang `ErrorPayload`). Chỉ chấp nhận trong `#[cfg(test)]`.
- **`panic!` chỉ cho invariant** đã được test ghim. Ví dụ có sẵn: `category_family()`
  panic với id lạ — và có test phủ mọi id để đảm bảo không bao giờ panic thật.
- **`#[serde(rename_all = "camelCase")]` BẮT BUỘC** trên mọi struct đi qua IPC
  (Rust giữ `snake_case` nội bộ, frontend nhận `camelCase`).
- **Thời gian/dung lượng qua IPC:** thời gian = ISO 8601 string; SQLite lưu Unix epoch
  INTEGER; dung lượng = số byte thô `u64`, format ở frontend.
- **`#[allow(dead_code)]`** được dùng có chủ đích để forward-declare API cho milestone
  sau — giữ nguyên, đừng xoá code "chưa dùng" nếu có attribute này.
- **Naming:** `snake_case` cho hàm/file/module, `PascalCase` cho type. Crate: `tiny-core`, `tiny`.

### Framework-Specific Rules

#### CLI (clap)

- Toàn bộ định nghĩa CLI nằm ở `crates/tiny/src/cli.rs` — struct derive. Mỗi lệnh một `*Opts` struct.
- Lệnh mới: thêm variant vào enum `Commands` + nhánh `match` trong `crates/tiny/src/main.rs` gọi `render::<x>::run(opts)`.

#### Provider pattern cho lệnh `clean` (QUAN TRỌNG NHẤT)

- Thêm 1 category dọn dẹp = implement trait `CleanProvider` + đăng ký ở **3 nơi, phải đồng bộ**:
  1. `all_providers()` — thêm `Box::new(...)` đúng thứ tự canonical
  2. `known_category_ids()` — thêm id (id `kebab-case`)
  3. `category_family()` — thêm nhánh map id → `Family` (test sẽ panic nếu thiếu)
- `Family` (Dev / UserStorage / System) có **source of truth duy nhất** ở `category_family()`
  — provider TUYỆT ĐỐI không tự khai family.
- Phân biệt `ExecAction` (ngữ nghĩa: Trash/HardDelete/EmptyTrash) vs `CleanAction` (UI menu).
  Provider có quyền từ chối action không hợp lệ bằng `Err` (xem `TrashProvider`).

#### Tauri + React (architecture.md)

- Tauri command ở `src-tauri/src/commands/`, mỗi domain 1 file; tên `snake_case`.
- Event đặt tên `domain:action` (`scan:progress`, `clean:done`).
- Long-running op LUÔN chạy thread nền + emit progress event — KHÔNG block UI thread.
- Logic nghiệp vụ ở `tiny-core`, KHÔNG ở Tauri command (giữ CLI + GUI đồng bộ).
- Frontend dependency flow: `features/ → components/ → lib/`. CẤM chiều ngược lại
  (kể cả type-only import). TanStack Query cho `invoke`, Zustand cho UI state, update bất biến.

### Testing Rules

- **Unit test:** `#[cfg(test)] mod tests` inline trong chính file đang test
  (xem `providers/trash.rs`). KHÔNG tách file unit test riêng.
- **Integration test:** thư mục `tests/` dùng `assert_cmd` + `predicates` — test ở mức
  CLI binary (chạy lệnh `tiny ...` thật rồi assert stdout/exit). Hiện có `tests/clean_smoke.rs`.
- **Registry sync test BẮT BUỘC:** có test phủ mọi id trong `known_category_ids()` qua
  `category_family()` — thêm provider mới mà quên đăng ký family → test panic. Đừng xoá test này.
- **Provider test ghim hành vi:** mỗi provider có quyền từ chối `ExecAction` sai —
  viết test khẳng định việc từ chối đó (mẫu: `trash.rs` test `rejects_trash_action`).
- **Frontend:** Vitest, test co-located `*.test.ts` cạnh file nguồn.
- Chạy: `cargo test` (Rust core + CLI) · `npm run test` (Vitest frontend).

### Code Quality & Style Rules

- **File nhỏ, gom theo domain:** mỗi provider 1 file trong `providers/`; mỗi lệnh 1 module
  trong `commands/`. ~200–400 dòng/file là chuẩn, tránh file khổng lồ.
- **Immutability:** không mutate object tại chỗ — tạo bản copy mới. Bắt buộc với Zustand.
- **Module privacy:** chỉ `pub` thứ thật sự cần phơi ra; mặc định private. `mod.rs` khai báo
  rõ `pub mod` vs `mod` (xem `clean/mod.rs`: `fs_safe` + `providers` pub, còn lại private).
- **Doc comment giải thích "tại sao":** comment `///` mô tả invariant/lý do thiết kế, không
  thuật lại code (mẫu: `fs_safe.rs` giải thích vì sao không follow symlink).
- **Hằng số thay vì literal:** id/label provider khai báo `const ID` / `const LABEL` đầu file.
- **`tiny-core` thuần:** cấm `println!`, `eprintln!`, `dialoguer`, log, mọi I/O ra stdout —
  chỉ trả struct/error. Presentation thuộc lớp gọi.

### Development Workflow Rules

- **Commit message:** Conventional Commits, mô tả bằng tiếng Việt — `<type>(<scope>): <mô tả>`
  (mẫu lịch sử repo: `feat(clean): M3.1 — quarantine, crash_reports`).
- **PR:** tiêu đề + nội dung tiếng Việt; identifier/path/CLI giữ tiếng Anh. Mỗi PR một mối quan tâm.
- **Plan/spec:** tài liệu kế hoạch nằm ở `docs/plans/` và `docs/specs/`, đặt tên `YYYY-MM-DD-<slug>.md`.
- **`.DS_Store` và `/target`** đã trong `.gitignore` — không commit. `Cargo.lock` và `package-lock.json` được commit để build có thể lặp lại.

### Critical Don't-Miss Rules

- **CẤM `fs::remove_dir_all` và `fs::remove_*` trực tiếp.** Mọi thao tác xoá đi qua `fs_safe`
  (`remove_recursive_safe`) hoặc lớp delete hybrid (Trash/quarantine). Đây là rule an toàn cốt lõi.
- **CẤM follow symlink khi quét/xoá:** luôn dùng `symlink_metadata`, không `is_dir()` (vốn follow
  symlink). Sai chỗ này có thể xoá nhầm ra ngoài thư mục đích.
- **Đăng ký provider thiếu chỗ:** quên 1 trong 3 nơi (`all_providers` / `known_category_ids` /
  `category_family`) → registry lệch. `category_family()` sẽ panic id lạ.
- **Trả struct `snake_case` thô qua IPC** → frontend phải xài field lạc lõng. Luôn `rename_all = "camelCase"`.
- **Đặt logic nghiệp vụ trong Tauri command** thay vì `tiny-core` → CLI và GUI mất đồng bộ.
- **`unwrap()` ở command layer** thay vì map sang `ErrorPayload` → app panic thay vì báo lỗi mềm.
- **App đang chạy:** provider có `requires_app_quit()` phải được tôn trọng — không xoá cache
  của app đang mở (mẫu cơ chế `skipped_running_app` trong `ExecReport`).

---

## Usage Guidelines

**Cho AI Agent:**

- Đọc file này TRƯỚC khi implement bất kỳ code nào.
- Tuân thủ MỌI rule đúng như mô tả; khi phân vân, chọn phương án an toàn/khắt khe hơn.
- Cập nhật file này khi xuất hiện pattern mới.

**Cho con người:**

- Giữ file lean, chỉ tập trung những gì agent cần.
- Cập nhật khi tech stack hoặc pattern thay đổi — đặc biệt sau khi migration Tauri workspace
  hoặc khi thay đổi contract IPC và safety.
- Rà soát định kỳ, bỏ rule đã trở nên hiển nhiên.

Last Updated: 2026-10-06

## Desktop implementation notes

- `tiny-core::options` không chứa flag presentation (`--json`, `--yes`, picker...). CLI chuyển options bằng `From<&...>`.
- `src-tauri::service` chỉ nhận đường dẫn trong scan đang lưu; preview id là server-generated, TTL 10 phút, dùng một lần. Luôn fingerprint trước move, re-discover để tôn trọng app-running gates.
- GUI không thực thi provider irreversible (`trash`, `docker`, `time-machine-local`). Trash Put Back qua Finder; quarantine restore không overwrite. Không tự purge quarantine ở bản này.
- `AppState::begin()` serialize các long-operation; mọi event stream có `:done` hoặc `:error`, kể cả khi worker fail.
- Native Tauri modules/deps chỉ compile trên macOS. Linux vẫn test portable app services. Browser ngoài Tauri là demo được ghi nhãn và không được thực thi mutation.
- `space_lens::validate_path` từ chối symlink ở mọi ancestor. Với fixture trên macOS, canonicalize temporary root trước khi tạo file (`/var` và `/tmp` có thể là symlink hệ thống).
