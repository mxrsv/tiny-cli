# Plan: Nâng cấp `tiny clean` — category families + picker hierarchical + drill-down

**Status**: DRAFT
**Spec**: [2026-05-04-tiny-clean-advanced](../specs/2026-05-04-tiny-clean-advanced.md)
**Goal**: Mở rộng `tiny clean` từ 5 → 20+ category providers, gom theo 3 family (Dev / User storage / System leftovers), thay picker phẳng bằng picker hierarchical 2 cấp + drill-down per-path opt-in.
**Architecture**: Giữ nguyên `CleanProvider` trait. Thêm `Family` registry tĩnh ở `providers/mod.rs` (map `category_id → Family`). Picker mới group theo family, drill-down là 1 stage tuỳ chọn trả về `HashSet<PathBuf>` excluded; `execute()` lọc items theo set đó trước khi gọi provider — không đổi shape `CategoryGroup`, các provider hiện có không phải sửa.

## 1. Kết quả mong đợi

- [ ] `cargo test -p tiny --all-targets` pass — toàn bộ unit test cho 15+ provider mới + family registry + picker grouping + drill-down filter
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo build --release` thành công, `tiny clean --help` liệt kê 2 flag mới `--review-paths` và `--idle-days`
- [ ] `tiny clean --dry-run --include-review --include-destructive` in summary group theo 3 family (Dev / User / System) với aggregate size + count mỗi family
- [ ] `tiny clean --category docker --dry-run` chạy không panic kể cả khi Docker daemon down (graceful skip + warning)
- [ ] `tiny clean --review-paths --category user-caches --dry-run` mở drill-down stage và thoát sạch không xoá gì
- [ ] `tiny clean --yes --category cargo` (workflow cũ) hoạt động y hệt hiện tại — zero regression
- [ ] [README.md](../../README.md) cập nhật danh sách category mới + family + 2 flag mới

## 2. Nguồn dữ liệu chuẩn

**Canonical data**:

- **Trait `CleanProvider`** trong [providers/mod.rs](../../src/commands/clean/providers/mod.rs) — mỗi category mới = 1 module implement trait. Không thêm trait method mới ở vòng này.
- **Family registry** — hàm `category_family(id: &str) -> Family` ở [providers/mod.rs](../../src/commands/clean/providers/mod.rs). Source of truth duy nhất khi picker group + report summary. Không hardcode family ở provider, không scatter ở picker.
- **`CleanItem.path` + `CleanItem.size`** — canonical state cho drill-down. Drill-down trả `HashSet<PathBuf>` excluded; `execute::execute()` lọc `group.items` theo set này TRƯỚC KHI gọi `provider.execute()`. Provider không biết drill-down tồn tại.

**Lấy từ**:

- HOME path → `std::env::var_os("HOME")` (đã dùng pattern này trong dev_caches.rs).
- Idle threshold → CLI flag `--idle-days N` (default 30) → truyền xuống provider qua field trong struct provider khi `all_providers()` build (không phải biến global).
- Docker info → subprocess `docker system df --format json` cho discover, `docker system prune --volumes -f` cho execute. Provider check `which("docker")` ở `available()`; nếu daemon down thì `discover()` return `Ok(vec![])` + log warning qua `eprintln!`, KHÔNG return error.
- Process check → `process::is_running` / `ProcessChecker` (đã có).
- File metadata → `fs::symlink_metadata` (qua helper trong [fs_safe.rs](../../src/commands/clean/fs_safe.rs)).

**KHÔNG lấy từ**:

- File config runtime (vd `~/.tiny-cli/clean.toml`) — taxonomy + idle threshold đều static / CLI flag, không đọc config file ở vòng này.
- `fs::metadata` (follow symlink) — phải dùng `symlink_metadata`.
- `fs::remove_dir_all` — phải dùng `fs_safe::remove_recursive_safe`.

## 3. Business rules & invariants

- **Family registry phải cover 100% category id** — verify bằng test `every_known_category_has_family` trong [providers/mod.rs](../../src/commands/clean/providers/mod.rs) loop qua `known_category_ids()` gọi `category_family(id)`, expect không panic.
- **Symlink-safe** — mọi provider mới chỉ đi qua `fs_safe::dir_size_safe` / `fs_safe::remove_recursive_safe` / helper `top_level_entries` / `root_as_item`. Verify bằng test `does_not_follow_symlinks` riêng cho `node_modules` / `rust_targets` / `python_caches` (3 cái có walk filesystem).
- **Project root verification** cho `node_modules` / `rust_targets` / `python_caches` — chỉ flag thư mục con khi parent dir có manifest tương ứng (`package.json` / `Cargo.toml` / `pyproject.toml | setup.py | requirements.txt`). Verify bằng test `refuses_orphan_dir_without_manifest`.
- **App orphan cross-check** — `app_orphans` phải tra qua `mdfind "kMDItemContentType == 'com.apple.application-bundle'"` để build set bundle id còn cài; chỉ flag là orphan khi `Application Support/<dir>` KHÔNG match bundle id nào. Risk mặc định `Review`. Verify bằng test `mock_mdfind_returns_active_apps_skips_them`.
- **Skip running app** — `ios_simulators` gates Xcode, `android_sdk` gates "Android Studio", `docker` gates "Docker Desktop". Verify bằng test reuse pattern `running_xcode_skips_xcode_categories_at_discovery` đã có.
- **Drill-down empty-set rule** — nếu user uncheck toàn bộ path 1 category trong drill-down, category đó bị loại khỏi execute set (không gọi `provider.execute()` với `&[]`). Verify bằng test `drill_down_filter_removes_empty_categories`.
- **Drill-down fallback summary** — khi 1 category có > 500 path trong drill-down, picker hiện summary mode (group theo subdir cấp 1) thay vì list phẳng. Verify bằng test `drill_down_falls_back_to_summary_above_threshold`.
- **Trash default** — mọi provider mới mặc định Trash, hard delete chỉ qua `--hard` + `TINY_CONFIRM_HARD=1` (giữ nguyên gating hiện tại của `cli_validate.rs`).
- **Idle threshold** — default 30 ngày, override qua `--idle-days N`. Áp dụng cho `node_modules`, `rust_targets`, `python_caches`, `go_cache`, `gradle_maven`, `downloads_old`, `screenshots_old`. Verify bằng test `idle_threshold_filters_recently_modified` cho `rust_targets`.

## 4. Phạm vi / Ngoài phạm vi

**Làm**:

- M0 (foundation): `Family` registry, `category_family()`, picker hierarchical 2 cấp, drill-down stage, CLI flag `--review-paths` + `--idle-days`, execute filter theo `excluded_paths`.
- M1 (Dev family — 10 provider mới): `docker`, `node_modules`, `python_caches`, `rust_targets`, `go_cache`, `gradle_maven`, `jetbrains`, `vscode`, `ios_simulators`, `android_sdk`.
- M2 (User storage — 6 provider mới): `downloads_old`, `screenshots_old`, `mail_attachments`, `streaming_caches`, `chat_caches`, `browser_caches`.
- M3 (System leftovers — 5 provider mới): `quarantine`, `crash_reports`, `app_orphans`, `time_machine_local`, `font_quicklook_caches`.
- Cập nhật README + thêm category id mới vào `known_category_ids()`.

**KHÔNG làm**:

- Smart suggestion / preset / learning từ history.
- Linux/Windows support.
- Schedule / auto-clean nền.
- Undo / quarantine ngoài macOS Trash sẵn có.
- Duplicate detection.
- Đổi safety model (Trash default, `--hard` + env flag).
- Thay đổi shape `CategoryGroup` / `CleanItem` / `ExecAction`.
- File config `~/.tiny-cli/clean.toml`.

## 5. Rủi ro & Quyết định còn mở

**Đã chốt có rủi ro**:

- Drill-down opt-in qua `--review-paths` + menu action `d` — rủi ro: user mới có thể không phát hiện feature drill-down. Mitigation: README + dòng hint `(press 'd' to review paths)` ở action menu.
- `docker.rs` dùng subprocess CLI — rủi ro: phụ thuộc Docker CLI cài; nếu user dùng Docker thuần daemon (vd OrbStack giả lập) có thể `which("docker")` true nhưng `docker system df` fail. Mitigation: graceful skip với warning, không fail toàn lệnh.
- 22 provider mới trong 1 vòng — rủi ro: scope rất lớn, dễ bug regression. Mitigation: chia 3 milestone độc lập, mỗi M có thể merge riêng.
- `app_orphans` dùng `mdfind` — rủi ro: Spotlight indexing bị tắt → mdfind trả empty → toàn bộ `Application Support/*` bị flag là orphan. Mitigation: nếu mdfind trả 0 result, provider refuse discover (return empty) + log warning thay vì flag everything.

**Chưa chốt cần resolve** (bỏ trống — tất cả đã chốt ở Section 6 spec).

## 6. Các task

### M0. Foundation — Family registry + Picker hierarchical + Drill-down

#### Task M0.1: Định nghĩa `Family` enum + registry function

**File(s)**:

- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Decision**: `Family` là `enum { Dev, UserStorage, System }`, có method `id() -> &'static str` và `label() -> &'static str`. Hàm `category_family(id: &str) -> Family` match theo `id`, panic nếu id không known (đảm bảo invariant 100% coverage).

**Build**:

- [ ] Thêm `pub enum Family { Dev, UserStorage, System }` với `id()` (`"dev"` / `"user-storage"` / `"system"`) và `label()` (`"Dev caches"` / `"User storage"` / `"System leftovers"`).
- [ ] Thêm hàm `pub fn category_family(category_id: &str) -> Family` — match từng id, default panic `unknown category id: {}`.
- [ ] Map các category cũ: `user-logs` → System, `xcode-*` → Dev, `user-caches` → System, `cargo|npm|pnpm|yarn` → Dev, `trash` → System.

**Verify**:

- [ ] Test `every_known_category_has_family`: loop qua `known_category_ids()` gọi `category_family(id)`, expect không panic.
- [ ] Test `family_id_and_label_stable`: assert `Family::Dev.id() == "dev"`.

---

#### Task M0.2: Thêm CLI flag `--review-paths` và `--idle-days`

**File(s)**:

- [~] [src/cli.rs](../../src/cli.rs)
- [~] [src/commands/clean/cli_validate.rs](../../src/commands/clean/cli_validate.rs)

**Decision**: `review_paths: bool` default false, `idle_days: u64` default 30. `--review-paths` conflicts với `--yes` (drill-down là interactive). `--idle-days 0` refuse (vô nghĩa).

**Build**:

- [ ] Thêm field `pub review_paths: bool` (`#[arg(long, conflicts_with = "yes")]`) vào `CleanOpts`.
- [ ] Thêm field `pub idle_days: u64` (`#[arg(long, default_value_t = 30)]`) vào `CleanOpts`.
- [ ] Trong `validate_with_env`, thêm rule: nếu `opts.idle_days == 0` → `Err(anyhow!("--idle-days must be > 0"))`.
- [ ] Cập nhật mọi nơi construct `CleanOpts {..}` trong test (thêm 2 field default).

**Verify**:

- [ ] `cargo build` pass — không miss field nào.
- [ ] Test `idle_days_zero_fails` mới trong `cli_validate.rs`.
- [ ] Test cũ `yes_with_category_passes` vẫn pass.

---

#### Task M0.3: Picker hierarchical 2 cấp — `pick_categories_grouped`

**File(s)**:

- [~] [src/commands/clean/picker.rs](../../src/commands/clean/picker.rs)

**Phụ thuộc**: Task M0.1

**Decision**: Hàm mới `pub fn pick_categories_grouped(groups: &[CategoryGroup]) -> Result<Vec<usize>>` thay thế `pick_categories`. Render bằng `MultiSelect` 1-cấp với indent (vì `dialoguer` không có tree native): family header `[D] DEV — 12.3 GiB (3 cats)` ở dòng riêng (disabled item), category con thụt 4 space. Hàm cũ giữ làm internal alias để không break test.

**Build**:

- [ ] Build helper `group_by_family(groups: &[CategoryGroup]) -> Vec<(Family, Vec<usize>)>` — group index theo family, sort family theo thứ tự `Dev → UserStorage → System`.
- [ ] Build helper `family_max_risk(groups: &[CategoryGroup], indices: &[usize]) -> RiskLevel` — derive badge family theo `max(risk)` của category con (ưu tiên Destructive > Review > Safe).
- [ ] Build labels theo dạng: dòng family header (`▸ DEV — 12.3 GiB (3 cats) [destructive]` với badge max-risk), dòng category con prefix `  ` (2 space + dấu •).
- [ ] Default check: chỉ category `Safe` được pre-check; category `Review` / `Destructive` unchecked. Family header item disabled (không toggle được).
- [ ] Map ngược selected indices từ `MultiSelect` (đã chèn family header) về indices gốc trong `groups[]`.
- [ ] Giữ `pick_categories` cũ làm wrapper gọi `pick_categories_grouped`.

**Verify**:

- [ ] Test `group_by_family_orders_dev_first`: input mock 3 group thuộc 3 family → output thứ tự Dev, UserStorage, System.
- [ ] Test `group_by_family_aggregates_sizes`: 2 cargo + 1 trash → family Dev có 2 entry, family System có 1.
- [ ] Manual: `cargo run -- clean --include-review` thấy header family + indent.

---

#### Task M0.4: Drill-down stage — `picker::drill_down`

**File(s)**:

- [~] [src/commands/clean/picker.rs](../../src/commands/clean/picker.rs)
- [+] [src/commands/clean/picker_drill.rs](../../src/commands/clean/picker_drill.rs)

**Phụ thuộc**: Task M0.3

**Decision**: Drill-down là 1 hàm `pub fn drill_down(groups: &[&CategoryGroup]) -> Result<HashSet<PathBuf>>` trả set EXCLUDED (path bị uncheck). Mỗi category đã chọn → 1 `MultiSelect` riêng, default tất cả checked. Nếu category có > 500 path → fallback summary mode: group items theo `path.parent().file_name()`, multi-select theo group, expand toàn bộ path con của group bị uncheck vào excluded set.

**Build**:

- [ ] Tạo file `picker_drill.rs` với `pub fn drill_down(groups: &[&CategoryGroup]) -> Result<HashSet<PathBuf>>`.
- [ ] Threshold const `DRILL_DOWN_FLAT_LIMIT: usize = 500`.
- [ ] Flat mode: build labels `format!("{:>10} {}", format_bytes(item.size), item.path.display())`, defaults all true, return path nào không nằm trong selected.
- [ ] Summary mode: helper `summarize_by_subdir(items: &[CleanItem]) -> Vec<(PathBuf, Vec<usize>)>` group theo parent dir cấp 1, MultiSelect theo group, expand items theo indices.
- [ ] Khai báo module `mod picker_drill;` trong `picker.rs` (hoặc trong `mod.rs` của clean).
- [ ] Re-export `pub use picker_drill::drill_down;` từ `picker.rs`.

**Verify**:

- [ ] Test `drill_down_falls_back_to_summary_above_threshold`: tạo `CategoryGroup` với 501 item fake → assert helper `summarize_by_subdir` trả ≥ 1 group.
- [ ] Test `summarize_by_subdir_groups_by_parent`: 4 item ở 2 parent dir khác nhau → trả 2 group.

---

#### Task M0.5: Execute filter theo `excluded_paths`

**File(s)**:

- [~] [src/commands/clean/execute.rs](../../src/commands/clean/execute.rs)

**Phụ thuộc**: Task M0.4

**Decision**: Đổi signature `execute(groups, action)` → `execute(groups, action, excluded_paths: &HashSet<PathBuf>)`. Mỗi `group.items` được lọc trước khi gọi `provider.execute()`. Group có 0 item sau filter bị skip (không gọi provider).

**Build**:

- [ ] Đổi signature `pub fn execute(groups: &[&CategoryGroup], action: CleanAction, excluded_paths: &HashSet<PathBuf>) -> Result<ExecReport>`.
- [ ] Trong loop: build `let filtered: Vec<CleanItem> = group.items.iter().filter(|i| !excluded_paths.contains(&i.path)).cloned().collect();` — nếu `filtered.is_empty()` → continue.
- [ ] Gọi `provider.execute(&filtered, exec_action)`.
- [ ] Sửa caller `mod.rs` truyền `&HashSet::new()` (vòng default) hoặc set drill-down.

**Verify**:

- [ ] Test `drill_down_filter_removes_empty_categories`: 1 group 2 item, excluded set chứa cả 2 → ExecReport rỗng, provider KHÔNG được gọi (dùng mock provider counter).
- [ ] Test `execute_filters_only_excluded_items`: 1 group 3 item, excluded 1 → 2 path còn lại được pass tới provider.
- [ ] Test cũ trong execute.rs vẫn pass (chỉ thêm `&HashSet::new()` arg).

---

#### Task M0.6: Wire drill-down + family picker vào `mod.rs::run`

**File(s)**:

- [~] [src/commands/clean/mod.rs](../../src/commands/clean/mod.rs)

**Phụ thuộc**: Task M0.3, M0.4, M0.5

**Decision**: Nếu `opts.review_paths` → sau pick stage 1, gọi `drill_down()` lấy excluded set, truyền xuống `execute`. Action menu thêm mục `Review paths (drill-down)` chỉ hiện khi !`review_paths` (để user kích hoạt drill-down giữa chừng).

**Build**:

- [ ] Đổi call site `picker::pick_categories(&discovery.groups)?` → `picker::pick_categories_grouped(&discovery.groups)?`.
- [ ] Thêm item `"Review paths (drill-down)"` vào action menu trong `picker::pick_action` (giữa Trash và Hard delete) — return `CleanAction::ReviewPaths` hoặc dùng signal khác (vd return enum `ActionChoice` riêng) để gọi drill-down rồi loop lại action menu.
- [ ] Sau khi `selected` build xong + `print_plan`, check `opts.review_paths` → gọi `picker::drill_down(&selected)?`, lưu vào biến `excluded_paths: HashSet<PathBuf>`. Nếu menu action = ReviewPaths cũng chạy nhánh này rồi prompt action menu lần 2.
- [ ] Truyền `&excluded_paths` vào `execute::execute(&selected, action, &excluded_paths, &opts)` (chú ý M0.7 đã thêm `opts` arg).
- [ ] Update report để hiện `(N path excluded by review)` nếu set không rỗng.

**Verify**:

- [ ] Manual: `cargo run -- clean --review-paths --include-review --dry-run` mở drill-down, dry-run vẫn print plan + thoát.
- [ ] Test integration `run_with_review_paths_calls_drill_down` (skipped với `#[ignore]` nếu cần TTY) hoặc test unit ở smaller seam.

---

#### Task M0.7: Cập nhật `all_providers()` + `known_category_ids()` để nhận idle_days

**File(s)**:

- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)
- [~] [src/commands/clean/discover.rs](../../src/commands/clean/discover.rs)

**Phụ thuộc**: Task M0.2

**Decision**: Đổi signature `all_providers()` → `all_providers(opts: &CleanOpts) -> Vec<Box<dyn CleanProvider>>`. Provider nào cần idle_days nhận value qua field struct (vd `RustTargets { idle_days: u64 }`). Provider không cần thì ignore. Đổi caller `select_providers` + `execute.rs::execute` (rebuild list khi cần).

**Build**:

- [ ] Đổi signature `all_providers(opts: &CleanOpts)`.
- [ ] Sửa `discover::select_providers` truyền `opts` xuống.
- [ ] Sửa `execute::execute` build providers qua `all_providers(opts)` — phải pass `&CleanOpts` qua `execute()` signature: thêm param `opts: &CleanOpts`.
- [ ] Cập nhật caller `mod.rs::run` truyền opts xuống execute.
- [ ] Cập nhật mọi test gọi `all_providers()` truyền `&CleanOpts::default-equivalent`.

**Verify**:

- [ ] `cargo build` pass.
- [ ] Test `discover.rs` test cũ vẫn pass sau khi update arg.

---

### M1. Dev family — 10 provider mới

#### Task M1.1: Provider `docker`

**File(s)**:

- [+] [src/commands/clean/providers/docker.rs](../../src/commands/clean/providers/docker.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `docker`, risk `Review`, gates `Docker Desktop` running. Discover: `docker system df --format json` parse `LayersSize + BuildCache + Volumes` thành 3 `CleanItem` (path placeholder `<docker:images>` / `<docker:build-cache>` / `<docker:volumes>` — phép tắc đặc biệt vì không có path filesystem thật để xoá tay). Execute: subprocess `docker system prune -af --volumes` (ignore `path` placeholder). Nếu daemon down → `discover` trả `Ok(vec![])` + `eprintln!("warn: docker daemon unavailable, skipping")`.

**Build**:

- [ ] File mới có struct `Docker`, impl `CleanProvider`.
- [ ] `available()`: `which("docker")`.
- [ ] `requires_app_quit()`: `Some("Docker Desktop")` — gate khi user đang có Docker Desktop UI mở.
- [ ] `discover()`: chạy `docker system df --format '{{json .}}'`, parse mỗi line JSON, tổng size theo type.
- [ ] `execute()`: nếu action == HardDelete → `docker system prune -af --volumes`; nếu Trash → cùng command (Docker không có trash, log warning rằng không revertible).
- [ ] Đăng ký trong `all_providers()` và `known_category_ids()`. Map trong `category_family()` → Dev.

**Verify**:

- [ ] Test `docker_unavailable_when_cli_missing` mock `which` (qua trait nếu cần) — tạm thời chỉ test `available()` trên máy không có docker (skip with `#[ignore]`).
- [ ] Test `docker_id_in_known_categories`.
- [ ] Manual: `tiny clean --category docker --dry-run` không panic kể cả khi Docker daemon down.

---

#### Task M1.2: Provider `node_modules` (orphan / idle)

**File(s)**:

- [+] [src/commands/clean/providers/node_modules.rs](../../src/commands/clean/providers/node_modules.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `node-modules`, risk `Review`. Walk `~/Documents`, `~/Projects`, `~/Code`, `~/Developer`, `~/Workspace` (config tĩnh). Cho mỗi `node_modules` tìm thấy: chỉ flag nếu (a) parent có `package.json`, (b) `package.json` mtime > `idle_days` ngày trước. Symlink-safe walk.

**Build**:

- [ ] Struct `NodeModules { idle_days: u64, search_roots: Vec<PathBuf> }`.
- [ ] Helper `find_node_modules(roots, idle_days)` — BFS walk `fs_safe::walk_no_follow`-style nhưng bỏ qua subtree khi gặp `node_modules` (không recurse vào nó).
- [ ] Mỗi match return `CleanItem { path: <node_modules dir>, size: dir_size_safe(..) }`.
- [ ] Helper `is_idle_node_modules(path: &Path, idle_days: u64) -> bool` check `package.json` parent + mtime.
- [ ] Đăng ký + map family Dev.

**Verify**:

- [ ] Test `refuses_node_modules_without_package_json`: tempdir có `node_modules` nhưng không có `package.json` → `find_node_modules` trả empty.
- [ ] Test `idle_threshold_filters_recently_modified` áp cho `node_modules`: tempdir có `package.json` mtime hôm qua → bị skip với idle_days=30.
- [ ] Test `does_not_follow_symlinks` (tương tự `fs_safe`).

---

#### Task M1.3: Provider `python_caches` (`__pycache__`, venv mồ côi)

**File(s)**:

- [+] [src/commands/clean/providers/python_caches.rs](../../src/commands/clean/providers/python_caches.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `python-caches`, risk `Review`. Walk same roots như `node_modules`. Match: (a) bất kỳ `__pycache__` directory (luôn safe để xoá), (b) `venv` / `.venv` / `env` directory CHỈ khi parent có `pyproject.toml` / `setup.py` / `requirements.txt` AND parent mtime idle.

**Build**:

- [ ] Struct `PythonCaches { idle_days: u64, search_roots: Vec<PathBuf> }`.
- [ ] Hai helper riêng: `find_pycache(roots)` (no manifest check) và `find_orphan_venv(roots, idle_days)` (manifest check + idle).
- [ ] Manifest check helper `has_python_manifest(parent: &Path) -> bool`.
- [ ] Đăng ký + map family Dev.

**Verify**:

- [ ] Test `pycache_found_anywhere_no_manifest_required`.
- [ ] Test `venv_requires_python_manifest`: venv không có manifest → skip.
- [ ] Test `venv_idle_threshold_applied`.

---

#### Task M1.4: Provider `rust_targets` (idle `target/`)

**File(s)**:

- [+] [src/commands/clean/providers/rust_targets.rs](../../src/commands/clean/providers/rust_targets.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `rust-targets`, risk `Review`. Walk same roots. Match `target/` directory chỉ khi parent có `Cargo.toml` AND `Cargo.toml` mtime idle. Skip recurse vào `target/`.

**Build**:

- [ ] Struct `RustTargets { idle_days, search_roots }`.
- [ ] Helper `find_rust_targets(roots, idle_days)`.
- [ ] Đăng ký + map family Dev.

**Verify**:

- [ ] Test `refuses_target_without_cargo_toml`.
- [ ] Test `idle_threshold_filters_recently_modified` cho rust target.
- [ ] Test `does_not_descend_into_target` — nested `target/target` không bị flag.

---

#### Task M1.5: Provider `go_cache`

**File(s)**:

- [+] [src/commands/clean/providers/go_cache.rs](../../src/commands/clean/providers/go_cache.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `go-cache`, risk `Review`. Discover qua subprocess `go env GOCACHE` + `go env GOMODCACHE` → 2 `CleanItem` (root paths). available qua `which("go")`.

**Build**:

- [ ] Struct `GoCache`.
- [ ] `discover`: 2 lần `run_for_path("go", &["env", "GOCACHE"])` + GOMODCACHE.
- [ ] Reuse `root_as_item` + `execute_per_item`.
- [ ] Đăng ký + map family Dev.

**Verify**:

- [ ] Test `go_cache_unavailable_when_cli_missing` (`#[ignore]` nếu CI không có go).
- [ ] Test `go_cache_id_in_known_categories`.

---

#### Task M1.6: Provider `gradle_maven`

**File(s)**:

- [+] [src/commands/clean/providers/gradle_maven.rs](../../src/commands/clean/providers/gradle_maven.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `gradle-maven`, risk `Review`. Path tĩnh: `~/.gradle/caches`, `~/.gradle/daemon`, `~/.m2/repository`. Reuse `root_as_item`.

**Build**:

- [ ] Struct `GradleMaven`.
- [ ] `discover`: 3 path.
- [ ] `available`: bất kỳ path nào tồn tại.
- [ ] Đăng ký + map family Dev.

**Verify**:

- [ ] Test `gradle_maven_id_in_known_categories`.

---

#### Task M1.7: Provider `jetbrains` (caches, logs, system)

**File(s)**:

- [+] [src/commands/clean/providers/jetbrains.rs](../../src/commands/clean/providers/jetbrains.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `jetbrains`, risk `Review`. Path: `~/Library/Caches/JetBrains`, `~/Library/Logs/JetBrains` (root-as-item). Top-level entries (mỗi IDE = 1 entry).

**Build**:

- [ ] Struct `JetBrains`.
- [ ] `discover`: dùng `top_level_entries` cho Caches + Logs.
- [ ] Đăng ký + map family Dev.

**Verify**:

- [ ] Test `jetbrains_id_in_known_categories`.

---

#### Task M1.8: Provider `vscode` (caches, logs, CachedExtensions)

**File(s)**:

- [+] [src/commands/clean/providers/vscode.rs](../../src/commands/clean/providers/vscode.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `vscode`, risk `Review`. Path: `~/Library/Application Support/Code/Cache`, `~/Library/Application Support/Code/CachedData`, `~/Library/Application Support/Code/logs`. Risk Review (lo workspace state).

**Build**:

- [ ] Struct `VsCode`.
- [ ] `discover`: 3 path qua `root_as_item`.
- [ ] Đăng ký + map family Dev.

**Verify**:

- [ ] Test `vscode_id_in_known_categories`.

---

#### Task M1.9: Provider `ios_simulators`

**File(s)**:

- [+] [src/commands/clean/providers/ios_simulators.rs](../../src/commands/clean/providers/ios_simulators.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `ios-simulators`, risk `Review`, gates `Xcode` AND `Simulator` (chỉ cần một trong hai chạy → skip). Discover: `top_level_entries` của `~/Library/Developer/CoreSimulator/Caches` + `~/Library/Developer/CoreSimulator/Devices`. Không xoá whole CoreSimulator (mất config), chỉ xoá per-device cache + Caches subtree.

**Build**:

- [ ] Struct `IosSimulators`.
- [ ] `requires_app_quit`: trả `Some("Xcode")` (giữ pattern hiện tại — Simulator gating sẽ thêm sau nếu cần đa app).
- [ ] `discover`: top_level_entries 2 path.
- [ ] Đăng ký + map family Dev.

**Verify**:

- [ ] Test `ios_simulators_skips_when_xcode_running` reuse pattern `running_xcode_skips_xcode_categories_at_discovery`.

---

#### Task M1.10: Provider `android_sdk`

**File(s)**:

- [+] [src/commands/clean/providers/android_sdk.rs](../../src/commands/clean/providers/android_sdk.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `android-sdk`, risk `Review`, gates `Android Studio`. Path tĩnh: `~/.android/cache`, `~/.gradle/.tmp` (overlap intentional với gradle_maven là OK — xoá 2 lần safe), `~/Library/Android/sdk/system-images` (nếu tồn tại + idle 30 ngày → flag).

**Build**:

- [ ] Struct `AndroidSdk { idle_days }`.
- [ ] `requires_app_quit`: `Some("Android Studio")`.
- [ ] `discover`: 2 path safe + 1 path conditional.
- [ ] Đăng ký + map family Dev.

**Verify**:

- [ ] Test `android_sdk_id_in_known_categories`.
- [ ] Test `android_sdk_skips_when_studio_running`.

---

### M2. User storage family — 6 provider mới

#### Task M2.1: Provider `downloads_old`

**File(s)**:

- [+] [src/commands/clean/providers/downloads_old.rs](../../src/commands/clean/providers/downloads_old.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `downloads-old`, risk `Review`. List file trong `~/Downloads` có mtime > `idle_days` (default 30). Mỗi file = 1 `CleanItem`. Trash default (rất quan trọng — file user có thể cần).

**Build**:

- [ ] Struct `DownloadsOld { idle_days }`.
- [ ] Helper `list_old_files(dir: &Path, idle_days: u64) -> Vec<CleanItem>` — `read_dir` non-recursive (file only, không recurse subdir).
- [ ] Đăng ký + map family UserStorage.

**Verify**:

- [ ] Test `downloads_filters_by_age`: tempdir 2 file (1 cũ, 1 mới) → chỉ cũ được flag.
- [ ] Test `downloads_id_in_known_categories`.

---

#### Task M2.2: Provider `screenshots_old`

**File(s)**:

- [+] [src/commands/clean/providers/screenshots_old.rs](../../src/commands/clean/providers/screenshots_old.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M2.1

**Decision**: id `screenshots-old`, risk `Review`. Đọc location qua `defaults read com.apple.screencapture location`; nếu lỗi → fallback `~/Desktop`. Filter `Screenshot *.png` / `Screen Shot *.png` mtime > idle_days.

**Build**:

- [ ] Struct `ScreenshotsOld { idle_days }`.
- [ ] Helper `screenshot_dir() -> PathBuf` qua `run_for_path("defaults", &["read", "com.apple.screencapture", "location"])` fallback `~/Desktop`.
- [ ] Filter file theo prefix tên + idle.
- [ ] Đăng ký + map family UserStorage.

**Verify**:

- [ ] Test `screenshot_dir_falls_back_to_desktop_on_error` (mock subprocess hoặc `#[ignore]`).
- [ ] Test `screenshots_id_in_known_categories`.

---

#### Task M2.3: Provider `mail_attachments`

**File(s)**:

- [+] [src/commands/clean/providers/mail_attachments.rs](../../src/commands/clean/providers/mail_attachments.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `mail-attachments`, risk `Review`, gates `Mail`. Path: `~/Library/Mail/V*/MailData/Attachments` (glob `V*`). Top-level entries.

**Build**:

- [ ] Struct `MailAttachments`.
- [ ] `requires_app_quit`: `Some("Mail")`.
- [ ] Helper resolve `V*` qua `read_dir` filter pattern.
- [ ] Đăng ký + map family UserStorage.

**Verify**:

- [ ] Test `mail_attachments_id_in_known_categories`.
- [ ] Test `mail_skips_when_mail_running`.

---

#### Task M2.4: Provider `streaming_caches` (Spotify, Netflix, Music)

**File(s)**:

- [+] [src/commands/clean/providers/streaming_caches.rs](../../src/commands/clean/providers/streaming_caches.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `streaming-caches`, risk `Review`. Path tĩnh whitelist:

- `~/Library/Caches/com.spotify.client`
- `~/Library/Application Support/Spotify/PersistentCache`
- `~/Library/Containers/com.netflix.Netflix/Data/Library/Caches`
- `~/Library/Containers/com.apple.Music/Data/Library/Caches`

Mỗi path → 1 `CleanItem`. Provider gate per-app: trước khi xoá Spotify path, check `is_running("Spotify")`; nếu running → skip riêng path đó (không skip toàn provider).

**Build**:

- [ ] Struct `StreamingCaches`.
- [ ] Const `STREAMING_PATHS: &[(&str, &str)]` — list `(rel_path, app_name_or_empty)`.
- [ ] `discover`: filter path tồn tại + nếu app_name không empty thì check `is_running` skip.
- [ ] Đăng ký + map family UserStorage.

**Verify**:

- [ ] Test `streaming_paths_filtered_by_running_app`: mock checker `Spotify` running → spotify path bị skip.
- [ ] Test `streaming_id_in_known_categories`.

---

#### Task M2.5: Provider `chat_caches` (Slack, Discord, Telegram)

**File(s)**:

- [+] [src/commands/clean/providers/chat_caches.rs](../../src/commands/clean/providers/chat_caches.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M2.4

**Decision**: id `chat-caches`, risk `Review`. Whitelist:

- `~/Library/Application Support/Slack/Cache`
- `~/Library/Application Support/Slack/Service Worker/CacheStorage`
- `~/Library/Application Support/discord/Cache`
- `~/Library/Group Containers/*.ru.keepcoder.Telegram/account-*/postbox/media`

Pattern provider giống `streaming_caches` (per-app gate).

**Build**:

- [ ] Struct `ChatCaches`.
- [ ] Const path list + per-app gate.
- [ ] Resolve glob `*.ru.keepcoder.Telegram` qua `read_dir`.
- [ ] Đăng ký + map family UserStorage.

**Verify**:

- [ ] Test `chat_paths_skip_running_apps`.
- [ ] Test `chat_id_in_known_categories`.

---

#### Task M2.6: Provider `browser_caches` (Safari, Chrome, Firefox, Arc)

**File(s)**:

- [+] [src/commands/clean/providers/browser_caches.rs](../../src/commands/clean/providers/browser_caches.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M2.4

**Decision**: id `browser-caches`, risk `Review`. Whitelist Cache dirs only (NEVER Cookies, Login Data, Preferences):

- `~/Library/Caches/com.apple.Safari`
- `~/Library/Application Support/Google/Chrome/Default/Cache`
- `~/Library/Application Support/Firefox/Profiles/*/cache2`
- `~/Library/Application Support/Arc/User Data/Default/Cache`

Per-app gate: Safari/Chrome/Firefox/Arc.

**Build**:

- [ ] Struct `BrowserCaches`.
- [ ] Resolve Firefox glob `Profiles/*/cache2` qua `read_dir`.
- [ ] Per-app gate.
- [ ] Đăng ký + map family UserStorage.

**Verify**:

- [ ] Test `browser_id_in_known_categories`.
- [ ] Test `browser_paths_skip_running_apps`.
- [ ] Test `browser_does_not_touch_cookies_or_login`: assert const list không chứa `"Cookies"` / `"Login"`.

---

### M3. System leftovers family — 5 provider mới

#### Task M3.1: Provider `quarantine` (Gatekeeper events DB)

**File(s)**:

- [+] [src/commands/clean/providers/quarantine.rs](../../src/commands/clean/providers/quarantine.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `quarantine`, risk `Review`. Path: `~/Library/Preferences/com.apple.LaunchServices.QuarantineEventsV2`. Single file. Risk Review vì ảnh hưởng "open file from unknown developer" prompt history.

**Build**:

- [ ] Struct `Quarantine`.
- [ ] `discover`: file qua `root_as_item`.
- [ ] Đăng ký + map family System.

**Verify**:

- [ ] Test `quarantine_id_in_known_categories`.

---

#### Task M3.2: Provider `crash_reports` (DiagnosticReports)

**File(s)**:

- [+] [src/commands/clean/providers/crash_reports.rs](../../src/commands/clean/providers/crash_reports.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `crash-reports`, risk `Safe`. Path: `~/Library/Logs/DiagnosticReports`, `/Library/Logs/DiagnosticReports`. Top-level entries (mỗi `.crash` / `.ips` = 1 file).

**Build**:

- [ ] Struct `CrashReports`.
- [ ] `discover`: top_level_entries 2 path.
- [ ] Đăng ký + map family System.

**Verify**:

- [ ] Test `crash_reports_id_in_known_categories`.
- [ ] Test `crash_reports_safe_risk`: assert `risk() == Safe`.

---

#### Task M3.3: Provider `app_orphans`

**File(s)**:

- [+] [src/commands/clean/providers/app_orphans.rs](../../src/commands/clean/providers/app_orphans.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `app-orphans`, risk `Review` (KHÔNG bao giờ Safe). Walk top-level dir của `~/Library/Application Support`. Cross-check với set bundle id còn cài qua `mdfind "kMDItemContentType == 'com.apple.application-bundle'"` + parse `Info.plist` lấy bundle id. Nếu mdfind trả 0 result → discover trả empty + log warning (Spotlight tắt). Match khi tên dir KHÔNG trùng bất kỳ bundle id installed.

**Build**:

- [ ] Struct `AppOrphans`.
- [ ] Helper `installed_bundle_ids() -> Result<HashSet<String>>` — chạy `mdfind`, mỗi dòng là path `.app`, đọc `Contents/Info.plist` lấy `CFBundleIdentifier` qua subprocess `defaults read <path>/Contents/Info CFBundleIdentifier`.
- [ ] `discover`: nếu set rỗng → return empty + warning. Còn lại: filter dir trong Application Support theo set.
- [ ] Đăng ký + map family System.

**Verify**:

- [ ] Test `mock_mdfind_returns_active_apps_skips_them` — mock helper trả set với "com.apple.Safari" → dir tên "com.apple.Safari" trong tempdir bị skip.
- [ ] Test `empty_mdfind_result_returns_empty_safely`.
- [ ] Test `app_orphans_id_in_known_categories`.

---

#### Task M3.4: Provider `time_machine_local`

**File(s)**:

- [+] [src/commands/clean/providers/time_machine_local.rs](../../src/commands/clean/providers/time_machine_local.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `time-machine-local`, risk `Destructive` (xoá local snapshot là không revert). Discover: `tmutil listlocalsnapshots /` parse output. Execute: `tmutil deletelocalsnapshots <date>`. Không có path filesystem để Trash → Trash action = HardDelete (vì tmutil không có khái niệm Trash).

**Build**:

- [ ] Struct `TimeMachineLocal`.
- [ ] `available`: `which("tmutil")`.
- [ ] `discover`: parse output `tmutil listlocalsnapshots /`, mỗi snapshot = 1 `CleanItem` với path placeholder `<tmutil:com.apple.TimeMachine.YYYY-MM-DD-HHMMSS>`, size 0 (tmutil không expose size).
- [ ] `execute`: parse snapshot id từ path placeholder, gọi `tmutil deletelocalsnapshots <id>`.
- [ ] Đăng ký + map family System.

**Verify**:

- [ ] Test `time_machine_id_in_known_categories`.
- [ ] Test `time_machine_destructive_risk`.

---

#### Task M3.5: Provider `font_quicklook_caches`

**File(s)**:

- [+] [src/commands/clean/providers/font_quicklook_caches.rs](../../src/commands/clean/providers/font_quicklook_caches.rs)
- [~] [src/commands/clean/providers/mod.rs](../../src/commands/clean/providers/mod.rs)

**Phụ thuộc**: Task M0.7

**Decision**: id `font-quicklook-caches`, risk `Safe`. Path: `~/Library/Caches/com.apple.QuickLook.thumbnailcache`, `/private/var/folders/.../com.apple.QuickLook.thumbnailcache` (qua `getconf DARWIN_USER_CACHE_DIR`), `~/Library/Caches/com.apple.FontRegistry`.

**Build**:

- [ ] Struct `FontQuicklookCaches`.
- [ ] Helper `darwin_user_cache_dir() -> Option<PathBuf>` qua `run_for_path("getconf", &["DARWIN_USER_CACHE_DIR"])`.
- [ ] `discover`: 3 path.
- [ ] Đăng ký + map family System.

**Verify**:

- [ ] Test `font_quicklook_id_in_known_categories`.
- [ ] Test `font_quicklook_safe_risk`.

---

### M4. Tài liệu + Polish

#### Task M4.1: Cập nhật README

**File(s)**:

- [~] [README.md](../../README.md)

**Phụ thuộc**: M0, M1, M2, M3 done

**Decision**: Bổ sung section `tiny clean` với bảng category × family × risk, mô tả 2 flag mới.

**Build**:

- [ ] Liệt kê 21 category theo thứ tự family Dev / UserStorage / System.
- [ ] Mô tả `--review-paths` (drill-down stage) + ví dụ `tiny clean --review-paths --include-review`.
- [ ] Mô tả `--idle-days N` (default 30) + scope các provider áp dụng.
- [ ] Cập nhật ví dụ `--category` với id mới (vd `--category docker`).

**Verify**:

- [ ] `grep -c "^- " README.md` ≥ 21 trong section `tiny clean` (mỗi category 1 dòng).
- [ ] Manual: read README, đảm bảo bảng category đầy đủ, không sai chính tả id.

---

#### Task M4.2: Smoke test end-to-end

**File(s)**:

- [+] [tests/clean_smoke.rs](../../tests/clean_smoke.rs)

**Phụ thuộc**: M4.1

**Decision**: Integration test gọi `tiny clean --dry-run --include-review --include-destructive` qua `assert_cmd`, expect exit 0 + stdout contain header `Cleanup candidates`.

**Build**:

- [ ] Add dev-dep `assert_cmd` (nếu chưa có) qua `Cargo.toml`.
- [ ] Test `dry_run_with_all_includes_succeeds`: spawn binary, capture stdout, assert contain `"Cleanup candidates"` + `"Total:"`.
- [ ] Test `unknown_category_fails_with_clear_message`: spawn `tiny clean --category bogus`, expect non-zero exit, stderr contain `"unknown --category"`.

**Verify**:

- [ ] `cargo test --test clean_smoke` pass.
- [ ] `cargo test --all-targets` pass.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
