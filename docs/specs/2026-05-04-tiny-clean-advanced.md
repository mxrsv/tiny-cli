---
date: 2026-05-04
status: APPROVED
---

# Spec: Nâng cấp `tiny clean` — thêm category families + picker hierarchical với drill-down

**Date**: 2026-05-04 | **Status**: APPROVED

## 1. Bối cảnh

**Origin**:

- "tôi cần ý tưởng nâng cấp lệnh tiny clean thêm các chức năng nâng cao hơn"
- Pain point đã chốt: thiếu category, picker chưa đủ tốt khi list dài.

**Problem**:

- `tiny clean` hiện chỉ phủ 5 providers (`dev_caches`, `trash`, `user_caches`, `user_logs`, `xcode`). Còn nhiều khoản chiếm disk lớn không dọn được: Docker, `node_modules` mồ côi, iOS Simulators, Downloads cũ, browser cache, app residue của app đã uninstall, v.v.
- Khi mở rộng lên ~15-20 categories, picker phẳng hiện tại sẽ ngộp; user cũng không có cách review từng path bên trong category trước khi xoá.

**Decisions**:

- ✅ Thêm 3 category families: **Dev-heavy**, **User storage**, **System leftovers**. (User confirm: "các options trên đều cần thiết".)
- ❌ Skip "smart suggestion / preset / learning history" (user yêu cầu skip).
- ✅ Picker đổi sang **hierarchical theo family** (chọn family hoặc category con).
- ✅ Thêm **stage 2 drill-down per-path** để toggle giữ/xoá từng path trước khi execute.
- ✅ macOS-first (giữ nguyên scope hiện tại của `clean`). Linux/Windows out of scope.
- ✅ Giữ nguyên safety model hiện tại (Trash mặc định, `--hard` + `TINY_CONFIRM_HARD=1`, skip running app, symlink-safe).

## 2. Nguồn dữ liệu chuẩn

**Canonical**:

- **Provider trait** (`src/commands/clean/providers/`) là canonical cho mọi category mới. Mỗi category mới = 1 module provider implement trait hiện tại (`discover`, `execute`).
- **Family taxonomy** được khai báo tại 1 chỗ duy nhất (đề xuất: `providers/mod.rs` hoặc `types.rs`) dưới dạng enum/const map `category_id → family`. Không hardcode family ở từng provider, không scatter ở picker.
- **`CleanItem.path` + `CleanItem.size`** vẫn là canonical state cho stage-2 drill-down (toggle = exclude path khỏi execute set, KHÔNG mutate state ở provider).

**KHÔNG phải nguồn chuẩn**:

- Picker UI state (collapse/expand, selection cursor) — ephemeral, không persist.
- Risk badge của family — derive từ max(risk) của category con, không khai báo riêng.
- Family ordering trong picker — derive từ priority cố định (Safe families trước, Destructive sau), không cấu hình runtime.

## 3. Kiến trúc giải pháp

**Components**:

- **Family taxonomy registry** — map tĩnh `category_id → Family { id, label, risk_default }`. Sống trong `providers/mod.rs`. Source of truth khi picker group, khi report summary.
- **Category providers mới** — mỗi cái 1 file dưới `providers/`:
  - **Dev family**: `docker.rs` (images/volumes/build cache), `node_modules.rs` (orphan/old), `python_caches.rs` (`__pycache__`, venv mồ côi), `rust_targets.rs` (idle `target/`), `go_cache.rs`, `gradle_maven.rs`, `jetbrains.rs`, `vscode.rs`, `ios_simulators.rs`, `android_sdk.rs`.
  - **User storage family**: `downloads_old.rs`, `screenshots_old.rs`, `mail_attachments.rs`, `streaming_caches.rs` (Spotify/Netflix/etc.), `chat_caches.rs` (Slack/Discord/Telegram), `browser_caches.rs` (Safari/Chrome/Firefox/Arc).
  - **System family**: `quarantine.rs` (Gatekeeper), `crash_reports.rs`, `app_orphans.rs` (`~/Library/Application Support` của app đã uninstall — tái dùng logic `tiny uninstall` để detect), `time_machine_local.rs`, `font_quicklook_caches.rs`.
- **Hierarchical picker** (`picker.rs` mở rộng) — render tree 2 cấp: family node (có aggregate size + count) + category leaf. Hỗ trợ:
  - Toggle family → toggle hết category con.
  - Toggle category → toggle riêng.
  - Phím tắt expand/collapse all.
  - Mặc định: family `Safe` expand, families chứa `Destructive` collapse.
- **Drill-down stage** (`picker::drill_down`) — sau khi user chọn xong stage 1, nếu user trigger (mặc định: phím `d` trong action menu, HOẶC flag `--review-paths`) → mở 1 multi-select per category đã chọn, list path bên trong, default tất cả checked, user uncheck path nào thì path đó loại khỏi execute set.
- **Execute set filter** — execute.rs nhận thêm `excluded_paths: HashSet<PathBuf>` (rỗng nếu không drill-down). Provider vẫn nhận `&[CleanItem]` đã lọc, không cần biết drill-down tồn tại.

**Data Flow**:

- `discover()` → `Vec<CategoryGroup>` (như hiện tại) → `picker::group_by_family()` → `FamilyTree` → stage-1 picker → `Vec<&CategoryGroup>` (như hiện tại) → (optional) `picker::drill_down()` → `(Vec<&CategoryGroup>, HashSet<PathBuf>)` → `execute()` lọc theo excluded set → `ExecReport`.
- Drill-down KHÔNG đổi shape của `CategoryGroup`; chỉ thêm 1 set excluded — backward-compat với providers hiện có.

## 4. Failure modes

- Khi 1 provider mới crash trong `discover` (vd Docker daemon down khi gọi `docker system df`), discovery phải log warning + skip provider đó, KHÔNG fail toàn lệnh.
- Khi user toggle exclude TOÀN BỘ path của 1 category trong drill-down, category đó phải tự loại khỏi execute set (không gửi empty list xuống provider).
- Khi family chứa cả category Safe và Destructive, picker phải hiện badge family theo max-risk (Destructive) để user không nhầm.
- Khi `node_modules`/`rust_targets`/`python_caches` (venv mồ côi) được detect, phải verify project có `package.json`/`Cargo.toml`/`pyproject.toml` cha — nếu không có, refuse (tránh xoá thư mục trùng tên ở chỗ lạ).
- Khi `app_orphans` detect dir trong `~/Library/Application Support`, phải cross-check với danh sách `.app` đã cài (`/Applications`, `~/Applications`, mdfind) — chỉ flag là orphan nếu CHẮC CHẮN không còn app nào claim. Mặc định risk=Review, KHÔNG bao giờ Safe.
- Khi `ios_simulators`/`android_sdk` detect, phải skip nếu Xcode/Android Studio đang chạy (tái dùng `process::skip_if_running`).
- Khi drill-down multi-select có > N paths (đề xuất N=500) trong 1 category, picker phải fallback về summary mode (group theo subdir) để tránh render lag.

## 5. Hoàn thành & Loại trừ

**Done**:

- ≥ 15 category providers mới được implement, mỗi cái có unit test cho `discover()` với fixture filesystem.
- Family registry tập trung 1 chỗ; thêm category mới chỉ cần edit 1 file để map family.
- Picker render hierarchical 2 cấp, toggle family ↔ category con hoạt động đúng, giữ phím tắt cũ tương thích cho user đã quen.
- Drill-down opt-in qua `--review-paths` flag HOẶC menu action; chạy không có flag → behavior y hệt hiện tại (zero regression).
- `--category <id>` cũ vẫn hoạt động, accept cả category mới lẫn cũ.
- README cập nhật danh sách category + family + flag mới.
- Toàn bộ providers mới tuân thủ symlink-safe + skip-running-app + Trash-default.

**Not done**:

- KHÔNG implement smart suggestion / preset / learning từ history (user skip).
- KHÔNG support Linux/Windows ở vòng này.
- KHÔNG implement schedule/auto-clean nền.
- KHÔNG implement undo/quarantine ngoài macOS Trash sẵn có.
- KHÔNG implement duplicate detection (trùng với roadmap `files`).
- KHÔNG đổi safety model (Trash default, `--hard` + env flag) — giữ nguyên.

## 6. Câu hỏi mở

- **ASSUMPTION**: drill-down là **opt-in** qua `--review-paths` hoặc menu, KHÔNG mặc định bật. Lý do: với 15-20 categories × hàng trăm path, drill-down mặc định sẽ làm lệnh nặng. Cần user confirm assumption này.
- **ASSUMPTION**: family taxonomy tĩnh, không cấu hình runtime. Đủ chưa, hay cần file config (`~/.tiny-cli/clean.toml`) để user tự gán category vào family khác?
- **QUESTION**: với `node_modules`/`rust_targets`, ngưỡng "idle" mặc định là bao nhiêu ngày kể từ lần modify cuối? (đề xuất 30 ngày). Có muốn flag `--idle-days N` để override không?
- **QUESTION**: `docker.rs` nên gọi `docker system prune` qua subprocess hay tự walk `~/Library/Containers/com.docker.docker/...`? Subprocess an toàn hơn nhưng phụ thuộc Docker CLI có sẵn; walk filesystem rủi ro hơn nhưng hoạt động cả khi Docker daemon down.
- **QUESTION**: scope vòng này có thể quá lớn (15+ providers). Có muốn chia thành 2-3 PR/milestone (M1 = Dev family, M2 = User storage, M3 = System leftovers) ngay từ plan, hay làm 1 cục?
- **BLOCKER**: chưa có — không thấy dependency external nào chặn.
