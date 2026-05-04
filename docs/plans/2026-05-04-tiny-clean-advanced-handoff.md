# Handoff — `tiny clean` advanced (M0 only)

**For**: session mới tiếp tục implement [2026-05-04-tiny-clean-advanced.md](./2026-05-04-tiny-clean-advanced.md).

## State khi handoff (2026-05-04)

- **Branch**: `feat/clean-advanced` (đã tạo từ `main` sau khi push 2 commit prep — `e96fe90` scan refactor + `b039e45` clippy fix uninstall).
- **Working tree**: untracked `.vscode/` (bỏ qua) + `docs/` (chứa plan + spec + handoff này).
- **Build/test/clippy**: tất cả clean trước khi tách branch (54 test pass, clippy `-D warnings` clean).

## Scope phiên mới

**Chỉ làm M0 (foundation)** — 7 task: M0.1 → M0.7. Sau M0 dừng, để user review trước khi sang M1.

M0 gồm:

- M0.1: `Family` enum + `category_family()` registry ở `providers/mod.rs`
- M0.2: CLI flag `--review-paths`, `--idle-days` (default 30) ở `cli.rs` + validate
- M0.3: `pick_categories_grouped()` picker hierarchical 2 cấp (family header + category con indent)
- M0.4: `drill_down()` ở file mới `picker_drill.rs` — flat mode + summary mode threshold 500
- M0.5: `execute()` nhận `excluded_paths: &HashSet<PathBuf>`, filter trước khi gọi provider
- M0.6: Wire drill-down + family picker vào `mod.rs::run`
- M0.7: `all_providers(opts: &CleanOpts)` nhận opts để inject `idle_days`

## Quyết định đã chốt (ambiguity resolution)

1. **Action menu enum riêng**: tạo enum `ActionChoice { Trash, HardDelete, ReviewPaths, Cancel }` riêng cho picker, KHÔNG nhồi `ReviewPaths` vào `CleanAction`. Lý do: giữ `CleanAction` thuần cho provider trait, không leak UI concern.
2. **Mock subprocess qua trait injection**: provider phụ thuộc subprocess (docker, mdfind, tmutil, defaults, which) sẽ inject command-runner qua field struct (default = real subprocess, test = mock closure). Test phụ thuộc CLI thực dùng `#[ignore]` hoặc `#[cfg(target_os = "macos")]`. _Áp dụng từ M1 trở đi — M0 không đụng tới subprocess._
3. **Drill-down summary mode**: khi 1 category có >500 path, group theo `path.parent().file_name()` cấp 1; uncheck 1 group → expand toàn bộ item có parent khớp group key vào `HashSet<PathBuf>` excluded. Flatten 1 cấp, không recurse.

## Quy ước thực thi

- Theo skill `superpowers:executing-plans`: làm tuần tự M0.1 → M0.7, mark task done sau verify.
- Mỗi task xong chạy `cargo build && cargo test --all-targets && cargo clippy --all-targets -- -D warnings` ngay, fix lỗi trước khi sang task kế.
- Commit theo từng task (hoặc gộp 2-3 task nhỏ liền kề) với message `feat(clean): M0.X — <mô tả>`.
- KHÔNG đụng `main`. KHÔNG xoá / sửa file ngoài scope plan.
- Provider hiện có (cargo/npm/pnpm/yarn/trash/user-caches/user-logs/xcode-\*) phải zero regression — test cũ phải pass.

## Verify cuối M0

Trước khi báo "M0 done":

```bash
cargo build --release
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo run -- clean --help    # phải thấy --review-paths và --idle-days
cargo run -- clean --dry-run  # picker hiện family header indent; vẫn chạy được
cargo run -- clean --yes --category cargo  # workflow cũ zero regression
```

Sau đó dừng, không tự ý làm M1.
