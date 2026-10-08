---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
  - step-12-complete
releaseMode: phased
status: complete
completedAt: "2026-05-20"
vision:
  statement: "tiny = CleanMyMac minh bạch cho người dùng Mac kỹ thuật. macOS native app dọn dẹp + tối ưu hệ thống, engine Rust open-source (open-core: Free + Pro subscription), CLI+GUI share core."
  differentiators:
    - "Open-core: engine + CLI open-source, app có Free + Pro subscription (đổi từ #64 ngày 2026-10-08)"
    - "Giải thích nguyên nhân, không chỉ hiển thị (#11/#31/#33/#37/#67)"
    - "focus + cleanup (#48) — tận dụng lệnh focus dọn nền trong session"
    - "Phô CLI thay vì giấu (#68) — transparency + dạy CLI"
    - "Lưới an toàn 3 lớp (#7/#8/#9/#50) — làn 3 màu + preview + quarantine + undo"
  coreInsight: "Engine tiny đã có gần đủ core cleanup; vấn đề = (1) khoác GUI + (2) lấp 4 module thiếu (Space Lens, Smart Scan, Maintenance, Protection). Architecture đã chốt: tiny-core runtime-agnostic + Tauri shell."
  targetUser: "Mac power user / developer / sysadmin"
  whyNow: "engine tiny đủ trưởng thành; Tauri 2.x ổn định macOS; CleanMyMac thắt chặt subscription → cần alternative"
classification:
  projectType: desktop_app
  domain: general
  complexity: medium
  projectContext: brownfield
scopeDecisions:
  prdScope: "Phase 1 = Processes + Clean (approved 2026-10-06); broader roadmap follows after MVP"
  productScope: "GUI app + CLI (cùng share tiny-core)"
  timeline: "open-ended (personal project)"
inputDocuments:
  - "_bmad-output/project-context.md"
  - "_bmad-output/brainstorming/brainstorming-session-2026-05-07-190913.md"
  - "_bmad-output/planning-artifacts/architecture.md"
  - "docs/plans/2026-05-04-tiny-clean-advanced.md"
  - "docs/plans/2026-05-04-tiny-clean-advanced-handoff.md"
  - "docs/specs/2026-05-04-tiny-clean-advanced.md"
workflowType: "prd"
project_name: "tiny-cli"
user_name: "Kyantran"
date: "2026-05-20"
documentCounts:
  brief: 0
  research: 0
  brainstorming: 1
  projectDocs: 3
  projectContext: 1
  architecture: 1
projectType: "brownfield"
---

# Product Requirements Document - tiny-cli

**Author:** Kyantran
**Date:** 2026-05-20

**Scope updated:** 2026-10-06 — the user approved **Processes + Clean** as
the first desktop MVP. The current Product Scope section and its acceptance
criteria override the older Smart Scan-first phase assignments elsewhere in
this broader product record. Execution belongs in the
[MVP plan](../../docs/plans/2026-10-06-processes-clean-mvp.md).

## Executive Summary

`tiny` là một macOS native app dọn dẹp + tối ưu hệ thống, nhắm đến Mac power user / developer / sysadmin. Sản phẩm được xây trên engine Rust open-source `tiny-core`, với CLI (`tiny`) và GUI (Tauri shell + React 19) cùng share một core duy nhất — đảm bảo CLI và GUI không bao giờ drift về hành vi. Vấn đề cốt lõi: user kỹ thuật cần khả năng dọn dẹp + chẩn đoán Mac thực sự, nhưng các giải pháp hiện tại (CleanMyMac) đóng mã nguồn, đòi subscription, và che giấu cơ chế đằng sau những con số tổng. Sản phẩm này giải quyết bằng cách phơi bày cơ chế (mỗi action GUI hiện command `tiny ...` tương đương), giải thích nguyên nhân (vì sao ổ đầy, không chỉ bao nhiêu), và đảm bảo lưới an toàn (preview + quarantine + undo cho mọi action phá huỷ).

### What Makes This Special

Sản phẩm differentiates trên 5 trục độc lập, mỗi trục đối lập trực tiếp với CleanMyMac:

1. **Open-core** — engine `tiny-core` + CLI mã nguồn công khai, miễn phí; app có bản Free và gói Pro trả theo subscription (chốt 2026-10-08, thay cho "không subscription"). Tính năng Pro, giới hạn Free và giá chưa chốt — xem [landing page spec](../../docs/specs/2026-10-08-landing-page.md).
2. **Giải thích nguyên nhân, không chỉ hiển thị** — phantom space (vì sao "ổ đầy mà không thấy file"), heat-by-age (file vừa to vừa cũ = ứng viên xoá), so sánh snapshot ("tuần này có gì đổi"), dự báo đầy ổ ("18 ngày nữa đầy, dọn sớm không?"). Trả lời _tại sao_, không chỉ _bao nhiêu_.
3. **`focus` + cleanup** — tận dụng lệnh `tiny focus` sẵn có (timer Pomodoro): "trong lúc bạn focus 25 phút, app dọn nền". Lợi thế độc nhất không đối thủ nào có.
4. **Phô CLI thay vì giấu** — mỗi action GUI hiển thị command `tiny ...` tương đương. Đồng thời tăng transparency (user thấy app đang làm gì) và dạy người dùng CLI.
5. **Lưới an toàn 3 lớp** — làn 3 màu (🟢 An toàn / 🟡 Cần xem / 🔴 Nguy hiểm) + xem trước file thật trước khi xoá + quarantine 30 ngày + Cmd+Z undo lần dọn gần nhất.

**Current baseline (2026-10-06):** The checkout is a Rust binary crate with
31 registered cleanup category IDs, `uninstall`, `scan --json`, and `sys`.
It has no desktop shell, shared library crate, process-management command,
or app-managed quarantine/undo. The intended architecture remains a shared
`tiny-core`, the `tiny` CLI, and a Tauri + React desktop shell. The first
delivery now adds Processes and wraps the existing cleanup capabilities.

## Project Classification

- **Project Type**: `desktop_app` — macOS native app (Tauri), với CLI là co-deliverable từ shared core.
- **Domain**: `general` — system utility / cleanup tool, không ràng buộc ngành.
- **Complexity**: `medium` — domain low, nhưng technical complexity cao (filesystem safety, hybrid delete Trash/quarantine/hard, native macOS integration qua Tauri, đồng bộ CLI + GUI qua shared core).
- **Project Context**: `brownfield` — repo hiện có CLI binary `tiny` với 31 cleanup category IDs + architecture đã chốt (D1–D10) chưa code. PRD này định scope GUI app xây trên foundation đó.

## Success Criteria

These targets describe the broader product roadmap. The current MVP release
gates are the PC-P, PC-C, and PC-D criteria in [Product Scope](#product-scope);
Smart Scan, quarantine/undo, adoption targets, and historical timelines below
do not expand the Processes + Clean delivery.

### User Success

Một Mac power user / developer cài `tiny` và đạt được những outcome cụ thể sau:

- **First-run success (≤ 60 giây)**: Mở app lần đầu → 1 click "Smart Scan" → thấy Health Score 0–100 + dung lượng có thể giải phóng ("X GB"), không phải làm setup hay điền config nào.
- **First cleanup không sợ (≤ 5 phút)**: Hoàn thành lần dọn đầu tiên với ≥ 1 GB giải phóng, **0 file quan trọng bị xoá nhầm**, và biết rằng nếu lỡ tay vẫn undo được (quarantine 30 ngày + Cmd+Z).
- **"Aha" moment giải thích nguyên nhân**: Lần đầu app trả lời được câu hỏi user thực sự có ("vì sao ổ đầy?") — qua phantom space, heat-by-age, hoặc "tuần này thư mục X phình thêm Y GB" — chứ không chỉ ra con số tổng.
- **Trust the safety net**: Sau 3 lần dọn, user đủ tin để bật "auto-clean Nhóm xanh không cần xác nhận" (#63) — chứng tỏ lưới an toàn 3 lớp hoạt động.
- **CLI literacy bonus**: Power user học được ≥ 5 command `tiny ...` từ việc nhìn footer "Equivalent CLI" (#68) sau 1 tháng dùng app — measurable qua "Copy CLI" event.

### Business Success

Các chỉ tiêu dưới đây viết khi `tiny` còn là open-source miễn phí: "business" success = adoption + community traction. Từ 2026-10-08 app theo mô hình open-core + Pro subscription; chỉ tiêu revenue/chuyển đổi Pro chưa đặt.

- **3 tháng sau v1.0**: ≥ 500 unique installs (track qua Homebrew tap analytics + GitHub release downloads); ≥ 50 GitHub stars.
- **6 tháng**: ≥ 2000 installs; ≥ 5 contributor PR đã merge (signal: codebase đủ dễ hiểu cho người ngoài đóng góp).
- **12 tháng**: Xuất hiện trong ≥ 1 list "CleanMyMac alternatives" public (Reddit/HackerNews/awesome-mac); ≥ 1 blog post của user kỹ thuật review tích cực.
- **Differentiator validation**: ≥ 30% feedback từ user mention rõ một trong 5 trục differentiator (open-source, giải thích nguyên nhân, focus+cleanup, phô CLI, undo) là lý do họ chọn `tiny` thay vì CleanMyMac.

### Technical Success

Engine + GUI cùng đạt các ngưỡng kỹ thuật cụ thể (đo được, không vague):

- **CLI ↔ GUI parity**: 100% command `tiny clean` chạy được cùng kết quả từ GUI — không có code path nào chỉ tồn tại ở 1 layer. Test bằng: chạy GUI action → so output JSON với CLI tương đương.
- **Safety invariant**: 0 instance `fs::remove_*` trực tiếp trong code (CI grep gate); mọi xoá đi qua `fs_safe::remove_recursive_safe` hoặc hybrid delete layer. Symlink luôn dùng `symlink_metadata`.
- **Registry sync**: Test phủ 100% provider ID qua `category_family()` — thêm provider quên đăng ký = test panic.
- **Performance ngưỡng (M2 baseline, ổ SSD 500 GB)**:
  - `sys` snapshot ≤ 200 ms
  - `scan` 3 folder mặc định (Downloads/Desktop/Documents trung bình 50k files) ≤ 3 s
  - Smart Scan end-to-end (sys + scan + clean --dry-run) ≤ 5 s
  - GUI window cold start (Tauri app launch → ready) ≤ 1.5 s
- **Stability**: 0 panic ở production code path (chỉ chấp nhận `panic!` trong invariant đã test ghim như `category_family()`); CI block PR có `unwrap()`/`expect()` ngoài `#[cfg(test)]` ở Tauri command layer.
- **Crash recovery**: Quarantine restore work được 100% trên file đã quarantine trong 30 ngày, kể cả khi app bị kill giữa chừng.

### Measurable Outcomes

| Outcome              | Metric                                                  | Target   | When           |
| -------------------- | ------------------------------------------------------- | -------- | -------------- |
| First-run success    | % user reach "Smart Scan complete" trong session đầu    | ≥ 80%    | v1.0 + 3 tháng |
| First cleanup safety | File misdelete rate (qua quarantine restore rate < 24h) | < 0.5%   | v1.0 + 3 tháng |
| Trust earned         | % user bật auto-clean Nhóm xanh                         | ≥ 25%    | v1.0 + 6 tháng |
| Open-source health   | Median PR review time                                   | < 7 ngày | sau v1.0       |
| CLI literacy         | % user click "Copy CLI" ≥ 1 lần                         | ≥ 40%    | v1.0 + 3 tháng |
| Smart Scan speed     | P95 end-to-end Smart Scan latency                       | < 5 s    | v1.0           |
| Crash-free           | App sessions không panic                                | ≥ 99.5%  | v1.0           |

## Product Scope

### MVP — Processes + Clean (approved 2026-10-06)

The primary user is a Mac user or developer who needs to identify resource
consumers and review cleanup candidates before acting. On 2026-10-07 the user
approved a SwiftUI macOS app linking the shared Rust core in-process through
UniFFI, kept the Rust CLI, and reconfirmed this MVP scope. The
[native/CLI boundary spec](../../docs/specs/2026-10-07-swiftui-cli-boundary.md)
supersedes the older Tauri shell choice; the Rust core remains shared by the
CLI and the app. Older stack descriptions elsewhere in this broader record are
historical design references. The desktop has two
primary destinations: Processes and Clean. Exact layout and styling require
an interactive demo and user visual acceptance before production integration.
On 2026-10-07 the user deferred showing equivalent `tiny ...` commands in the
GUI for this MVP. FR39, the "Equivalent CLI" footer and the CLI-literacy metric
are deferred, not removed. The shared core remains the CLI/GUI parity mechanism,
and processes gain an additive `tiny processes` CLI command for PC-P5.

#### Processes acceptance criteria

- **PC-P1:** Show process name, PID, CPU, memory, ownership, and sample time;
  support search and CPU/memory sorting. Missing or stale measurements are
  explicit, never displayed as freshly measured zeroes.
- **PC-P2:** Show parent/child relationships and listening ports when macOS
  permits collection. Exited parents, permissions, and failed probes produce
  an explicit unavailable state rather than an empty-success claim.
- **PC-P3:** Let the user request termination of an eligible process with
  confirmation. Use SIGTERM first; force termination is a separate, explicitly
  confirmed action. Do not automatically escalate to SIGKILL or signal a tree.
- **PC-P4:** Revalidate PID and process start time at the action boundary.
  Refuse changed identities, self, protected/system processes, and targets
  outside the permitted current-user scope. Report permission denial and a
  process that remains alive accurately.
- **PC-P5:** Expose the same process data and action semantics through the
  CLI and desktop via the shared core. Fixture tests and a disposable child
  process cover termination; validation never targets real user applications.

#### Clean acceptance criteria

- **PC-C1:** Reuse the existing registered cleanup providers and risk groups.
  Display each candidate's category, path, measured size, reason for inclusion,
  and known skip/permission limitations.
- **PC-C2:** Provide category selection and per-path review/exclusion before
  confirmation. Empty selection and cancellation cause no filesystem change.
- **PC-C3:** Use macOS Trash for the default desktop action. Review-risk items
  require explicit selection. Keep permanent deletion, empty Trash, and
  nonrecoverable provider operations in the existing advanced CLI flow for
  this desktop MVP; do not silently fall back to permanent deletion.
- **PC-C4:** Revalidate selected paths, provider boundaries, and relevant
  running-app checks immediately before acting. An unavailable safety check
  must refuse the affected action and explain why.
- **PC-C5:** Report successful, failed, and skipped items individually.
  Selected bytes, bytes moved to Trash, and measured free disk space are
  different quantities; never claim moving to Trash freed that many bytes.
- **PC-C6:** Preserve the existing CLI commands and flags. Shared core APIs
  return structured data; presentation and interactive prompts remain in
  the CLI or desktop adapter.

#### Desktop acceptance criteria

- **PC-D1:** Keep sampling and cleanup off the UI thread; provide loading,
  cancellation, partial-result, and error states. Overlapping scans/actions
  must not cause duplicate mutations or use another operation's result.
- **PC-D2:** Support keyboard navigation, visible focus, readable contrast,
  and confirmation text that identifies the target and consequence.
- **PC-D3:** Release acceptance requires the user to approve rendered output
  and the Processes/Clean flows on macOS. Build/test success alone is not
  visual or native-app acceptance.

#### Deferred from this MVP

Smart Scan/Health Score, Space Lens, app-managed quarantine and Cmd+Z undo,
GUI uninstall/startup management, menubar monitoring, scheduled or automatic
cleanup, forecasting, focus integration, maintenance/protection, supervised
process restart, multi-platform support, public distribution automation, and
the GUI "Equivalent CLI" footer/copy (FR39, FR40; deferred 2026-10-07).
The existing CLI capabilities remain available. Future work must explicitly
reactivate the relevant requirements; old phase tags do not expand this MVP.

### Growth Features (Post-MVP, v1.1 → v1.x)

Mở rộng theo thứ tự ưu tiên dựa trên nhóm brainstorm còn lại:

- **v1.1 — Space Lens (Nhóm 2, ~14 ý tưởng)**: treemap tương tác (#15), engine walker toàn ổ (#29), heat-by-age (#33), phantom space (#31), drill + breadcrumb (#35), action bar trên ô (#36). **Khối nặng kỹ thuật nhất** — cần parallel walker mới.
- **v1.2 — Chủ động & giải thích (Nhóm 4 mở rộng, ~16 ý tưởng)**: menubar widget (#2), realtime sparkline (#12), "tuần này có gì đổi" (#11), dự báo đầy ổ (#67), top file sống (#10), notification chủ động (#66).
- **v1.3 — `focus` + cleanup (#48)**: tích hợp `tiny focus` — dọn nền trong Pomodoro session. Differentiator độc nhất.
- **v1.4 — Maintenance + Protection (Nhóm 5, ~13 ý tưởng)**: flush DNS / reindex Spotlight / free RAM / login items / malware scan / privacy cleaner / app updater / permissions audit. Tách thành phase riêng vì kỹ thuật phức tạp + rủi ro (Malware signatures, permissions).

### Vision (Future, v2.0+)

- **Nhóm 6 — Định vị + chế độ nâng cao**: menubar-only mode cho power user (#54), file finder tổng quát (#57), báo cáo sức khoẻ PDF (#58), "chuẩn bị bán máy" mode (#59), Fleet view multi-Mac (#60).
- **Rules engine kiểu Mail rules (#51)**: user tự định nghĩa rule (".dmg trong Downloads > 7 ngày → tự xoá").
- **Auto-clean như "PR chờ duyệt" (#52)**: app gửi diff "đã dọn X, Y, Z trong nền — approve để commit?".
- **AI ranking (#41)**: thay user-chooses-category bằng AI xếp hạng category theo (an toàn × dung lượng).
- **Sang ngoài macOS**: Linux/Windows port — nếu `tiny-core` đã được giữ runtime-agnostic chuẩn, port chỉ là Tauri shell mới.

## User Journeys

### Persona A — "Minh, Senior Dev" (Primary user — happy path)

**Backstory**: Minh, 32 tuổi, senior backend engineer ở một startup Sài Gòn. MacBook Pro M2 16 GB, ổ 512 GB. Một cây macOS user 8 năm, có biết CLI nhưng không nghiện. Đã thử CleanMyMac 2 năm trước, bỏ vì subscription $40/năm và "đùn cho mình con số 25 GB mà chẳng giải thích".

**Opening scene**: 9 giờ tối thứ Sáu, anh đang build Docker image cho deploy production. macOS pop-up _"Your startup disk is almost full"_. Free space còn 4 GB. Anh chửi thầm vì đã clear Downloads tuần trước.

**Rising action**:

1. Mở terminal, định gõ `du -sh ~/Library/*` như mọi lần, nhưng nhớ ra hôm trước thấy `tiny` trên HackerNews — "open-source CleanMyMac alternative". Cài qua `brew install tiny`.
2. Mở app GUI lần đầu. **Một nút duy nhất giữa màn hình: "Smart Scan"**. Click.
3. Loading 4 giây. Hiện Health Score = **62/100** màu cam, kèm dòng "Có thể giải phóng **18.4 GB** an toàn". Anh nhíu mày — "thật á?".
4. Bên dưới là 3 cột làn 3 màu. Cột **🟢 An toàn** đã tick sẵn: `xcode_derived_data` (8.2 GB), `npm_cache` (2.1 GB), `docker_overlay` (4.8 GB), `quarantine` (1.3 GB). Anh nhận ra ngay vì sao — vừa build Docker xong.
5. Click vào `xcode_derived_data`. **Bung ra preview file thật**: list từng project + size + last access. Anh thấy `OldClient-2023` chiếm 3 GB, một project anh đã đóng từ năm ngoái. Lòng tin tăng.
6. Footer hiện _"Equivalent CLI: `tiny clean --category=xcode_derived_data,npm_cache,docker_overlay,quarantine`"_ với nút "Copy". Anh copy bỏ vào notes — _"để dùng cho lần sau"_.

**Climax**: Click "Clean". 7 giây sau, app hiện _"Đã giải phóng 16.4 GB. Tất cả file vào quarantine 30 ngày. Cmd+Z để hoàn tác."_ Health Score nhảy lên **84/100** màu xanh. Docker build chạy tiếp.

**Resolution**: Sáng thứ Hai, Minh mở app lại. Health Score vẫn 81. Anh để app sống ở `~/Applications`, không cần subscription. 3 tuần sau anh git clone repo, mở 1 issue nhỏ về i18n tiếng Việt, gửi PR đầu tiên cho dự án open-source. Anh nói với đồng nghiệp _"có cái tool này hay phết, mã nguồn xem được"_.

**Capabilities revealed**: Smart Scan one-click, Health Score 0–100, làn 3 màu với mặc định an toàn, preview file thật, hybrid delete với quarantine 30 ngày, undo Cmd+Z, phô CLI footer, Homebrew distribution, open-source community workflow.

---

### Persona B — "Hà, DevOps + Tech Lead" (Primary user — power user / edge case)

**Backstory**: Hà, 38 tuổi, tech lead 1 team 15 người. Quản 3 con Mac (work, personal, một con cũ để demo). Sống trên CLI 80% thời gian, có `~/.zshrc` 400 dòng. Cô **không cần** app GUI, nhưng muốn dùng `tiny` ở chế độ "không cản đường".

**Opening scene**: Cô đang chuẩn bị reset con Mac demo cho intern mới. Cần dọn sạch nhưng giữ một số dotfiles. Bình thường cô sẽ viết script shell, nhưng muốn thử `tiny`.

**Rising action**:

1. Cô **không mở GUI**. Gõ thẳng `tiny clean --dry-run --include-destructive`. Output JSON nếu thêm `--json`.
2. Thấy có category `quarantine_30d` (mới so với lần trước cô dùng). Đọc help: `tiny clean --help`. Hiểu cơ chế.
3. Cô bật menubar-only mode trong app GUI: _"Settings → Menubar only, hide Dock icon"_. App ẩn khỏi Dock, sống trên menubar như `htop` widget — CPU/RAM/disk realtime.
4. Cô viết một rule trong `~/.config/tiny/rules.toml`: _".dmg trong Downloads > 14 ngày → tự xoá"_. (Phase 2+ feature, nếu chưa có cô tự script qua launchd + `tiny clean --category`.)
5. Trên menubar, cô click icon → mini panel hiện Health Score + nút "Open full app" và "Run Smart Scan". Cô bấm Smart Scan, để chạy nền.

**Climax**: Sau 12 giây, menubar icon đổi từ xám sang xanh. Notification _"Đã giải phóng 24 GB. Click để xem chi tiết hoặc hoàn tác."_ Cô không cần mở app, chỉ Cmd+Click notification để dismiss.

**Resolution**: Cô viết bài blog _"tiny vs CleanMyMac: 6 tháng dùng thử"_, nói rõ "tôi không bao giờ mở cửa sổ chính, chỉ dùng menubar + CLI". Bài được upvote lên trang nhất r/macapps. Repo nhận thêm 800 star trong 1 tuần.

**Capabilities revealed**: CLI parity 100% với GUI (cô không mở GUI vẫn dùng đủ feature), menubar-only mode (#54), realtime sparkline trên menubar (#12), notification system (#66), rules engine optional (#51 — phase 2), JSON output cho mọi command.

---

### Persona C — "Bác Long, người dùng Mac không-tech" (Edge case — sai an toàn)

**Backstory**: Bác Long, 58 tuổi, bố của Minh. Dùng MacBook Air để xem ảnh cháu, đọc báo, gọi Zoom với con. **Không biết** Terminal là gì. Minh cài `tiny` giúp bố vì máy bố hay đầy.

**Opening scene**: Bác mở Mac, máy chậm. Bác nhớ Minh dặn _"con cài app `tiny` rồi, máy đầy bố mở nó lên bấm nút **xanh** to ở giữa"_.

**Rising action**:

1. Bác mở app từ Launchpad (Minh đã pin icon). Bác bấm nút "Smart Scan" to xanh.
2. App hiện 18 GB có thể dọn, Health Score 51. Bác không hiểu Health Score là gì, nhưng thấy số đỏ → biết là _"không ổn"_.
3. Bác bấm "Clean". App hiện confirm dialog: _"Sẽ xoá 18 GB vào thùng rác. Có thể hoàn tác trong 30 ngày. Tiếp tục?"_ — chữ to, 2 nút "Tiếp tục" và "Hủy". Bác bấm "Tiếp tục".
4. 8 giây sau, máy nhẹ hẳn. Bác mở Safari → smooth.
5. **Một tuần sau**, bác phát hiện file ảnh cháu chụp chung Tết 2024 biến mất khỏi Desktop. Hoảng. Gọi Minh.

**Climax**: Minh hướng dẫn qua điện thoại: _"Bố mở `tiny`, ấn vào 'Lịch sử dọn dẹp' ở thanh trái, tìm lần dọn tuần trước, ấn 'Hoàn tác'"_. Bác làm theo. Cmd+Z trong app restore tất cả file từ quarantine. **Ảnh cháu quay lại Desktop**.

**Resolution**: Minh thêm rule trong app cho máy bố: _"Desktop và Documents — KHÔNG dọn dù làn xanh"_. Set exclusion lock bằng password. Bác Long tiếp tục dùng app, không sự cố thêm. Bác kể với hàng xóm _"con tao cài cái app hay lắm, hỏng cũng sửa được"_.

**Capabilities revealed**: Lưới an toàn 3 lớp thật sự cứu được người dùng (quarantine + undo trong app GUI, không cần CLI), confirmation dialog rõ ràng cho destructive action, exclusion folder lock (phase 1.5+, settings), accessibility-friendly UI (chữ to, nút lớn cho user 50+).

---

### Persona D — "OSS Contributor" (Secondary — không phải end-user)

**Backstory**: An, 24 tuổi, đang học Rust. Thấy `tiny` trên awesome-rust. Muốn đóng góp.

**Opening scene**: An clone repo, đọc README. README link sang `_bmad-output/planning-artifacts/architecture.md` + `project-context.md` (rules cho AI agent — cũng dễ đọc cho human contributor).

**Rising action**:

1. An đọc `CONTRIBUTING.md`. Thấy quy trình: tạo provider mới cho category mới = thêm 1 file + đăng ký 3 nơi + viết test.
2. An tạo file `crates/tiny-core/src/clean/providers/safari_cache.rs`. Implement trait `CleanProvider`. Đăng ký ở `all_providers()`, `known_category_ids()`, `category_family()` — 3 nơi như doc dặn.
3. Quên đăng ký `category_family()` cho id mới. Chạy `cargo test`. Registry sync test panic ngay: _"id `safari_cache` không có family mapping"_. An hiểu lỗi trong 10 giây.
4. Fix, test pass. Gửi PR với commit message tiếng Việt theo convention repo: _"feat(clean): thêm provider safari_cache"_.

**Climax**: 3 ngày sau, PR được review + merge. An được thêm vào CONTRIBUTORS.md. Là PR open-source đầu đời.

**Resolution**: An tiếp tục đóng góp 4 PR nữa trong 2 tháng. Trở thành regular contributor. Repo có 1 maintainer mới giúp triage issue.

**Capabilities revealed**: Architecture rõ ràng (`tiny-core` runtime-agnostic), provider pattern dễ extend, test registry sync ngăn lỗi đăng ký thiếu chỗ, documentation cho contributor (`CONTRIBUTING.md` + project-context.md), Conventional Commits tiếng Việt, GitHub PR workflow.

---

### Journey Requirements Summary

Bảng map từ journey → capability cần build:

| Capability                                                                    | Journey nào revealed          | Phase                                  |
| ----------------------------------------------------------------------------- | ----------------------------- | -------------------------------------- |
| Smart Scan one-click + Health Score 0–100                                     | A (Minh), C (Bác Long)        | MVP                                    |
| Làn 3 màu với mặc định an toàn                                                | A, C                          | MVP                                    |
| Preview file thật trước khi xoá (size/path/age)                               | A                             | MVP                                    |
| Hybrid delete: quarantine 30 ngày + Trash macOS option                        | A, C                          | MVP                                    |
| Undo Cmd+Z + "Lịch sử dọn dẹp" panel                                          | A (mention), C (load-bearing) | MVP                                    |
| Phô CLI footer "Equivalent CLI" + Copy                                        | A, B                          | Deferred 2026-10-07 (was MVP)          |
| CLI parity 100% (mọi GUI action có CLI tương đương)                           | B                             | MVP (architecture invariant)           |
| JSON output cho mọi command                                                   | B                             | MVP (`scan --json` đã có, cần mở rộng) |
| Confirmation dialog cho destructive action                                    | C                             | MVP                                    |
| Accessibility — chữ to, nút lớn, contrast cao                                 | C                             | MVP (a11y baseline)                    |
| Menubar-only mode + realtime sparkline                                        | B                             | v1.2 (Nhóm 4)                          |
| Notification system chủ động                                                  | B                             | v1.2                                   |
| Exclusion folder lock (settings)                                              | C (sự cố ảnh cháu)            | v1.1 (a11y/safety follow-up)           |
| Rules engine (`rules.toml`)                                                   | B                             | v2.0 (Vision)                          |
| Open-source workflow (CONTRIBUTING, test registry sync, Conventional Commits) | D                             | MVP (đã có 1 phần)                     |
| Homebrew tap distribution                                                     | A                             | MVP                                    |
| Provider pattern extensible (3-nơi-đăng-ký)                                   | D                             | MVP (architecture invariant — đã có)   |

## Domain-Specific Requirements

Domain `general` (system utility) không có compliance regulatory như healthcare/fintech, nhưng có **constraint nền tảng macOS** + **safety invariant filesystem** mang vai trò tương tự — sai một chỗ là phá dữ liệu user.

### macOS Platform Constraints

- **macOS 13 Ventura trở lên** baseline (Tauri 2.x yêu cầu). Không hỗ trợ Intel Mac (M-series only) cho v1.0 — đơn giản hoá test matrix, M2 baseline đã cover 70%+ user kỹ thuật 2026.
- **Code signing + notarization bắt buộc** cho distribution: bundle Tauri `.app` phải có Apple Developer ID, notarize qua Apple để qua Gatekeeper. Yêu cầu: Apple Developer account ($99/năm — chi phí cá nhân maintainer).
- **Sandboxing — KHÔNG bật App Sandbox**: tool dọn dẹp cần access `~/Library`, `/Applications`, `~/Library/Developer` ngoài container — incompatible với sandbox. Bù lại bằng minh bạch (open-source) + Full Disk Access permission rõ ràng.
- **Full Disk Access (TCC)**: user phải grant trước khi quét được `~/Library/Caches/...`. App phải có onboarding flow hướng dẫn cách grant qua System Settings → Privacy & Security → Full Disk Access.
- **APFS-aware**: snapshot/clone behavior khác HFS+. Tính dung lượng cần dùng `statvfs` + APFS purgeable info (mô tả phantom space #31).
- **Không hỗ trợ Mac App Store** cho v1.x — App Store yêu cầu sandbox. Distribute qua: (1) Homebrew tap chính thức, (2) GitHub Releases `.dmg` notarized, (3) Cask.

### Filesystem Safety Invariants (Domain-Specific Rules)

Đây là "compliance" của domain này — vi phạm = user mất dữ liệu:

- **CẤM `std::fs::remove_dir_all`, `fs::remove_file`, `fs::remove_dir` trực tiếp** trên đường chạy production. CI grep gate fail PR có pattern này ngoài module `fs_safe`.
- **Mọi xoá đi qua `fs_safe::remove_recursive_safe`** hoặc qua hybrid delete layer (`Trash` provider / quarantine). Không bypass.
- **Symlink: luôn `symlink_metadata`, KHÔNG `is_dir()`** (vốn follow symlink → có thể xoá ra ngoài thư mục đích). Test ghim hành vi sẵn ở `fs_safe.rs`.
- **Path canonicalization trước khi xoá**: resolve symlink chain, check parent path không nằm ngoài whitelist (`~/Library`, `~/Downloads`, `~/.cache`, v.v. tuỳ provider).
- **Atomic operation**: quarantine = move trong cùng filesystem. Không copy + delete (race condition + double disk space).
- **Disk space check trước**: free space < 2× size cần move → fail-fast.
- **App đang chạy = không xoá cache**: provider có `requires_app_quit()` phải được tôn trọng. Cơ chế `skipped_running_app` đã có ở engine — GUI phải hiển thị skip này, không silent.

### Privacy & Data Handling

- **Zero telemetry mặc định**: không thu thập analytics, không phone home. Settings có toggle "Send anonymous crash report" tắt mặc định, opt-in. Đối lập CleanMyMac.
- **Crash report (nếu opt-in)**: chỉ stack trace + OS version, KHÔNG file path, KHÔNG hostname.
- **Quarantine path**: `~/.tiny/quarantine/<timestamp>-<uuid>/` — readable bởi user, không hidden trick. User có thể `cd` vào inspect bất cứ lúc nào.
- **Cleanup journal** (`~/.tiny/journal.sqlite`): lưu local. User xoá được bất cứ lúc nào qua Settings → Clear journal.
- **CLI command history**: KHÔNG log argument (có thể chứa path nhạy cảm), chỉ log command name + timestamp.

### Risk Mitigations (Domain-Specific)

| Rủi ro                                         | Mitigation                                                                                               | Mức            |
| ---------------------------------------------- | -------------------------------------------------------------------------------------------------------- | -------------- |
| Xoá nhầm file user quan trọng                  | Hybrid delete (quarantine 30 ngày default) + undo Cmd+Z + preview file thật trước khi xoá                | Hard           |
| Provider mới đăng ký thiếu chỗ (lệch registry) | Test ghim `category_family()` panic id lạ + CI gate                                                      | Hard           |
| App panic giữa chừng dọn → quarantine mồ côi   | Transactional quarantine: ghi journal entry TRƯỚC khi move; auto-cleanup sau 30 ngày qua background task | Hard           |
| Symlink trick xoá ra ngoài whitelist           | `symlink_metadata` + canonicalize + whitelist check                                                      | Hard           |
| User cài bản fake/malicious giả `tiny`         | Notarization Apple + Homebrew official tap + reproducible build (best-effort)                            | Soft (phase 2) |
| User non-tech xoá nhầm dù có lưới an toàn      | Exclusion folder lock (Persona C), confirmation dialog chữ to, default-on quarantine                     | Hard MVP       |
| Quarantine cũ chiếm dung lượng vô hạn          | Auto-purge sau 30 ngày + notify khi size > 10 GB                                                         | Hard           |
| App treo (long-running scan)                   | Scan/clean chạy thread nền + emit progress event qua Tauri, nút "Cancel" responsive < 1 s                | Hard           |

## Innovation & Novel Patterns

### Detected Innovation Areas

Sản phẩm có **3 đổi mới thực sự** (không phải innovation theatre) — tách biệt với "feature mới" thuần tuý:

**1. `focus` + cleanup (#48) — Pomodoro-aware background maintenance**

_Đổi mới gì_: Tận dụng lệnh `tiny focus` (Pomodoro timer đã có trong CLI) như **trigger** để chạy cleanup nền. Trong lúc user focus 25 phút, app dọn các category an toàn ở background, hết session báo cáo "đã giải phóng X GB, không cần làm gì". Cleanup trở thành **side-effect** của thói quen làm việc, không phải task riêng.

_Vì sao novel_: CleanMyMac, MacKeeper, OnyX không có timer/focus. Storage Sense của Windows có "ambient cleanup" nhưng không tích hợp với productivity tool. Đây là intersection chưa ai làm: **productivity tool ∩ cleanup tool** trong cùng app, lấy chính work session làm cue.

_Validation approach_: Đo `% session focus có cleanup hoàn thành thành công / tổng session focus`. Mục tiêu: ≥ 70% session focus → cleanup ≥ 1 category xanh không user-interrupt.

**2. "Giải thích nguyên nhân, không chỉ hiển thị" — explainability-first storage analysis**

_Đổi mới gì_: Mọi panel hiển thị đều phải trả lời câu hỏi **"tại sao"**, không chỉ "bao nhiêu":

- Phantom space (#31): "ổ đầy mà không thấy file" → giải thích APFS snapshot + purgeable space.
- Heat-by-age (#33): file vừa to vừa cũ tô đỏ trên treemap → câu trả lời cho "xoá cái nào trước".
- "Tuần này có gì đổi" (#11): so 2 snapshot → "thư mục X phình thêm 12 GB do build Docker thứ Tư".
- Dự báo đầy ổ (#67): trend line → "18 ngày nữa đầy, dọn sớm không?".

_Vì sao novel_: CleanMyMac hiển thị "25 GB rác" → user phải tin. GrandPerspective/DaisyDisk hiển thị treemap → user phải tự diễn giải. Chưa có app nào tổng hợp **explainability** thành design principle xuyên suốt UI. Đây là reframe loại sản phẩm: từ "tool dọn dẹp" sang "tool _giải thích_ storage".

_Validation approach_: Qualitative test với 10 user — đưa cho họ Health Score 51/100 và quan sát: họ có hỏi "tại sao 51?" không, và app có **trả lời được trong UI** không (không cần Google). Pass = 8/10 user tìm thấy explain trong < 30 giây.

**3. CLI-transparent GUI (#68) — phô CLI thay vì giấu**

_Đổi mới gì_: Mỗi action GUI hiển thị footer "Equivalent CLI: `tiny clean --category=...`" + nút "Copy". App vừa là GUI, vừa là **CLI literacy tool**.

_Vì sao novel_: GUI app truyền thống _giấu_ implementation chi tiết — đó là điểm mạnh của UX. Phô CLI đi ngược tradition đó vì target persona là power user kỹ thuật, người **muốn** biết app đang gọi gì để (a) trust, (b) script hoá sau này, (c) học CLI. Trở thành differentiator culture chứ không phải feature.

_Validation approach_: Đo "Copy CLI" event rate. Mục tiêu ≥ 40% user click ≥ 1 lần trong 30 ngày đầu. Cross-reference với survey "tại sao bạn chọn `tiny`?" — kỳ vọng ≥ 20% mention phô CLI.

### Market Context & Competitive Landscape

| Competitor                       | Mô hình                                                    | Khoảng cách với `tiny`                                                                                                           |
| -------------------------------- | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| **CleanMyMac X**                 | Subscription $40/năm, closed-source, target user phổ thông | `tiny` engine + CLI open-source, app Free + Pro subscription, target user kỹ thuật. Khác hoàn toàn target market                 |
| **OnyX**                         | Free, GUI utility cho macOS maintenance scripts            | `tiny` rộng hơn (cleanup + scan + monitor + focus), có quarantine/undo. OnyX không có cleanup phân loại                          |
| **DaisyDisk / GrandPerspective** | Một-shot $10–$30, chỉ làm treemap                          | `tiny` có treemap (v1.1) + cleanup + monitor integrated. DaisyDisk không xoá được, GrandPerspective không giải thích nguyên nhân |
| **ncdu / dust (CLI)**            | Free, CLI-only, không có GUI                               | `tiny` CLI parity với chúng nhưng có GUI option cho task lớn (treemap visual). ncdu/dust không có quarantine/undo                |
| **Storage Sense (Windows)**      | Built-in OS, ambient cleanup                               | Reference cho UX ambient mode (#56). macOS không có cái tương đương — gap thị trường                                             |
| **Sensei / MacCleaner Pro**      | One-time purchase, closed-source                           | Tương tự CleanMyMac nhưng không subscription. `tiny` mở mã + có CLI                                                              |

**Khoảng trống thị trường rõ ràng**: Không có sản phẩm nào trên macOS hiện tại đồng thời (a) open-source + (b) có cả CLI + GUI parity + (c) explainability-first + (d) tích hợp productivity (focus). 4 trục cùng giao = không gian trống.

### Validation Approach

Ba lớp validation cho innovation:

**Lớp 1 — Technical proof (trong phase MVP)**:

- Build prototype focus+cleanup hook trên 1 category an toàn duy nhất (vd: `npm_cache`). Đo: cleanup hoàn tất trong < 25 phút focus session 90% lần.
- Build phantom space detector (parse APFS snapshot info). Đo: trên 5 máy thật, phát hiện ≥ 80% lượng "ổ đầy không thấy file".
- Build CLI footer hook (mỗi GUI action emit equivalent command). Đo: 100% action có CLI tương đương, copy được.

**Lớp 2 — User reaction (MVP + 3 tháng)**:

- 10 user kỹ thuật dùng app trong 2 tuần. Phỏng vấn:
  - "Bạn có nhận ra footer CLI không? Có click không? Có học được gì?"
  - "Bạn có dùng focus + cleanup không? Có thấy hữu ích không?"
  - "Khi app báo phantom space, bạn có hiểu vì sao không?"
- Pass: ≥ 7/10 mention ít nhất 1 trong 3 innovation là lý do tích cực.

**Lớp 3 — Market response (MVP + 6 tháng)**:

- Đo organic mention trên HackerNews, r/macapps, awesome-mac. Pass: ≥ 1 list "CleanMyMac alternatives" public include `tiny`.
- Đo % GitHub star growth tháng-trên-tháng. Pass: ≥ 20% MoM trong 6 tháng đầu sau v1.0.

### Risk Mitigation

| Rủi ro innovation                                               | Mức    | Fallback                                                                                                                                         |
| --------------------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| **`focus` + cleanup không ai dùng** (user bỏ qua focus command) | Medium | Innovation vẫn tồn tại như tuỳ chọn; không phải core flow. Không affect MVP nếu user không bật focus                                             |
| **Phantom space detection sai** (false positive APFS report)    | Medium | Hiển thị disclaimer "ước tính từ APFS — có thể khác Finder", link Apple doc. Không xoá purgeable space, chỉ explain                              |
| **Phô CLI confuse user phổ thông** (như Bác Long persona)       | Low    | Footer ẩn được trong Settings (default ON cho `tiny`, OFF nếu detect không phải dev qua heuristic — vd: chưa từng mở Terminal)                   |
| **Treemap quá nặng cho ổ lớn** (Space Lens v1.1)                | High   | Tách v1.1: ship MVP không treemap; treemap với engine walker mới + virtualization (chỉ render tile visible)                                      |
| **Open-source maintain burden** vượt khả năng cá nhân           | High   | Architecture đã thiết kế modular (provider pattern + crate `tiny-core` riêng) — dễ accept contributor. Document `CONTRIBUTING.md` rõ ràng từ MVP |
| **Innovation overshadows execution** (đẹp ý tưởng nhưng buggy)  | High   | Lock MVP scope: nếu 1 innovation chưa đạt validation Lớp 1, defer sang v1.1 thay vì ship half-baked                                              |

## Desktop App Specific Requirements

### Project-Type Overview

`tiny` là **Tauri 2.x desktop app** macOS-only cho v1.0, kèm CLI binary cùng share core. Bundle phân phối: `.app` notarized + `.dmg` qua GitHub Releases, đồng thời Homebrew tap chính thức. Tech stack đã chốt qua architecture decision D1–D10:

- **Backend**: Rust workspace 3 crate — `crates/tiny-core` (engine library, runtime-agnostic), `crates/tiny` (CLI binary, code hiện hành), `src-tauri/` (Tauri shell).
- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS + TanStack Query 5.100 (data fetch qua `invoke`) + Zustand 5.0 (UI state) + d3-hierarchy (treemap v1.1) + Vitest (test).
- **Persistence**: SQLite (`rusqlite` 0.39) ở `src-tauri/` cho journal/quarantine/snapshot. `tauri-plugin-store` cho settings KV. `tiny-core` stateless tuyệt đối.
- **IPC**: Tauri command (`#[tauri::command]` ↔ `invoke()`) + event (`app.emit()` ↔ `listen()`) cho progress streaming.

### Technical Architecture Considerations

**Workspace layout:**

```
tiny-cli/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── tiny-core/              # engine library — runtime-agnostic
│   │   ├── src/
│   │   │   ├── lib.rs          # public API surface
│   │   │   ├── sys/            # sysinfo wrapper
│   │   │   ├── scan/           # scan engine + walker
│   │   │   ├── clean/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── fs_safe.rs  # safety primitives (CẤM bypass)
│   │   │   │   ├── providers/  # 21+ CleanProvider impls
│   │   │   │   └── registry.rs # all_providers/known_ids/category_family
│   │   │   ├── focus/          # Pomodoro timer (sẵn có)
│   │   │   └── uninstall/      # app removal
│   │   └── tests/              # unit + integration test
│   └── tiny/                   # CLI binary
│       ├── src/main.rs         # clap entry
│       ├── src/cli.rs          # struct derive
│       ├── src/commands/       # 1 file / lệnh, gọi tiny-core
│       └── tests/              # assert_cmd + predicates (integration)
└── src-tauri/                  # Tauri app
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── src/
    │   ├── main.rs             # tauri::Builder
    │   ├── commands/           # 1 file / domain, gọi tiny-core
    │   │   ├── scan.rs
    │   │   ├── clean.rs
    │   │   ├── sys.rs
    │   │   └── quarantine.rs
    │   ├── store/              # SQLite repo (journal, snapshot, undo)
    │   ├── events/             # progress emitter
    │   └── tray/               # menubar widget (v1.2)
    └── frontend/               # Vite + React
        ├── src/
        │   ├── features/       # smart-scan, space-lens, settings...
        │   ├── components/     # shared UI
        │   ├── lib/            # pure parsing/util
        │   └── App.tsx
        ├── vite.config.ts
        └── package.json
```

**Dependency rules (architecture invariants — không vi phạm)**:

- `tiny-core` KHÔNG phụ thuộc: `tauri`, `tokio`, `rusqlite`, `trash`, `dialoguer`, `println!`, `eprintln!`. Lý do: phải chạy được trong CLI sync + GUI async cùng lúc, không cần runtime.
- `tiny-core` chỉ phơi: struct serde-serializable + typed error (`thiserror` 2.0).
- `tiny` (CLI) phụ thuộc `tiny-core` + `clap` + `anyhow` + `dialoguer`. Presentation layer.
- `src-tauri` phụ thuộc `tiny-core` + `tauri` + `tokio` + `rusqlite` + `trash`. Async runtime + persistence + Tauri-specific.
- Frontend dependency flow: `features/ → components/ → lib/`. CẤM ngược chiều (kể cả type-only import).
- Cross-cutting: tất cả struct qua IPC `#[serde(rename_all = "camelCase")]` BẮT BUỘC.

### Platform Support

- **macOS 13 Ventura+ (Apple Silicon)** baseline cho v1.0.
- **macOS Intel**: không support v1.0. Cân nhắc v1.x nếu có contributor + test machine.
- **Linux / Windows**: không support v1.x. `tiny-core` được giữ runtime-agnostic → port là khả thi sau v2.0 (đổi shell Tauri, viết lại provider macOS-specific).
- **Test matrix v1.0**: macOS 13 / 14 / 15 (latest) × Apple Silicon M1/M2/M3. CI dùng GitHub Actions `macos-14` (M-series runner). Manual smoke test 1 máy M1 + 1 máy M3.

### System Integration

**macOS-specific APIs sử dụng**:

| Capability              | API / Crate                                                          | Sử dụng cho                                    |
| ----------------------- | -------------------------------------------------------------------- | ---------------------------------------------- |
| File trash              | `trash` 5.2 crate                                                    | Hybrid delete D5 — file thường vào Trash macOS |
| Symlink-safe metadata   | `std::fs::symlink_metadata`                                          | Mọi xoá filesystem (safety invariant)          |
| System info             | `sysinfo` 0.32                                                       | `tiny sys`, Health Score components            |
| APFS snapshot info      | `statvfs` + `diskutil apfs list` (subprocess) hoặc IOKit binding     | Phantom space (#31) detection                  |
| Tray icon + menubar     | Tauri `tauri::tray` (built-in v2)                                    | Menubar widget (#2) v1.2                       |
| Notification            | Tauri `tauri::notification`                                          | Notification chủ động (#66) v1.2               |
| Full Disk Access detect | Đọc `~/Library/Caches/com.apple.Spotlight` → nếu fail = không có FDA | Onboarding flow D7                             |
| Reveal in Finder        | `NSWorkspace` qua `objc2` binding                                    | Action bar trên file preview                   |
| Quick Look              | `qlmanage` subprocess hoặc `QuickLookUI` binding                     | Preview file v1.2                              |
| FSEvents (watch)        | `fsevent` crate hoặc `notify-rs` macOS backend                       | Live re-scan (#30) v1.1+                       |
| Login Items             | `SMAppService` (macOS 13+)                                           | Login Items management (#23) v1.4              |
| TCC permissions audit   | `tccutil` subprocess (read-only)                                     | Permissions audit (#28) v1.4                   |
| Code sign + notarize    | Tauri bundler + `notarytool`                                         | Distribution                                   |

**Tích hợp với engine CLI hiện hành**:

- Lệnh CLI hiện tại (`tiny sys`, `tiny scan`, `tiny clean`, `tiny focus`, `tiny uninstall`) giữ nguyên hành vi sau khi migrate sang workspace. Backward-compatible cho user CLI hiện tại.
- Test integration `assert_cmd` đảm bảo CLI behavior không drift sau refactor.

### Update Strategy

- **Mechanism**: Tauri updater plugin (`tauri-plugin-updater`) — check release qua GitHub API, download `.dmg`, prompt user install. Signature verification bằng public key embed.
- **Frequency check**: 1 lần / 24 giờ khi app launch + manual "Check for Updates" trong menu app.
- **User control**: settings có toggle "Auto-check updates" (default ON), "Auto-download" (default OFF), "Auto-install" (default OFF — luôn cần user confirm trước khi install).
- **Versioning**: SemVer strict — `0.x.y` cho pre-1.0, `1.x.y` sau ship MVP. Breaking change CLI = major bump.
- **Release channel**: chỉ stable cho v1.0; cân nhắc beta channel sau khi có > 1k user.
- **Distribution channels parity**:
  - GitHub Releases → primary, có `.dmg` + checksums + signatures.
  - Homebrew Cask (`brew install --cask tiny`) → mirror từ GitHub Releases, auto-update qua `brew upgrade`.
  - **Không** Mac App Store v1.x (sandbox incompatible).

### Offline Capabilities

App **100% offline-first** — không network dependency cho core feature:

- `sys`, `scan`, `clean`, `quarantine restore`, `focus`, `uninstall`: 0 network call.
- `tiny-core` không có bất kỳ HTTP client dependency.
- Network chỉ dùng cho 2 thứ optional + opt-in/limited:
  - **Update check** (24h interval, có thể tắt) — chỉ gọi GitHub Releases API.
  - **Crash report** (opt-in, default OFF) — gửi sentry/GitHub issue endpoint nếu user bật.
- App **không có** account/login, không cloud sync, không telemetry mặc định.
- Trade-off chấp nhận: không có cross-device sync settings/journal (v2.0 vision có thể add iCloud sync optional).

### Implementation Considerations

**Critical implementation sequence (từ architecture D1–D10)**:

1. **D1 — Dựng workspace + tách `tiny-core`** (prerequisite tuyệt đối). Refactor `src/commands/clean` để tách compute khỏi `dialoguer` prompt. Có thể tốn 1–2 sprint.
2. **D8 — Chốt error type** (`thiserror` ở core, `ErrorPayload` ở Tauri command, `anyhow` ở CLI binary). Làm cùng D1.
3. **D2 — Progress callback API**: `fn scan(opts: ScanOpts, progress: impl Fn(Progress)) -> Result<ScanReport, CoreError>`. Sync, runtime-agnostic.
4. **Khởi tạo `src-tauri/` + React frontend** (D3): scaffolding qua `npm create tauri-app@latest` với React/TS/Vite, sau đó copy vào `src-tauri/`.
5. **D7 — FDA onboarding** (critical path — không có FDA = không scan được).
6. **D4 + D5 — Persistence + hybrid delete**: SQLite schema cho journal/quarantine, `trash` crate cho macOS Trash integration.
7. **D6 — Tray icon + menubar widget** (v1.2, sau khi MVP ship).

**Performance targets cụ thể (M2 baseline, ổ SSD 500 GB ~70% full)**:

| Operation                               | Target P50 | Target P95 | Hard cap |
| --------------------------------------- | ---------- | ---------- | -------- |
| App cold start (Tauri launch → ready)   | < 800 ms   | < 1.5 s    | 3 s      |
| `sys` snapshot                          | < 150 ms   | < 200 ms   | 500 ms   |
| `scan` 3 folder mặc định (50k files)    | < 2 s      | < 3 s      | 10 s     |
| `scan` toàn ổ (~500k files, v1.1)       | < 30 s     | < 60 s     | 120 s    |
| Smart Scan end-to-end                   | < 4 s      | < 5 s      | 10 s     |
| `clean --dry-run` 1 category            | < 500 ms   | < 1 s      | 3 s      |
| `clean` execute 1 category (quarantine) | < 2 s      | < 5 s      | 10 s     |
| GUI frame budget (60 fps)               | < 16.6 ms  | < 16.6 ms  | —        |
| Cancel responsive                       | < 500 ms   | < 1 s      | 1.5 s    |

**Sections skipped per CSV** (không liên quan desktop_app domain):

- `web_seo` — không phải web app, không cần SEO.
- `mobile_features` — macOS-only desktop, không có touch/mobile gesture.

## Project Scoping & Phased Development

The current first release is **Processes + Clean**, approved on 2026-10-06.
The [Product Scope](#product-scope) section owns
its acceptance criteria. The [MVP plan](../../docs/plans/2026-10-06-processes-clean-mvp.md)
owns task ordering and verification.

### MVP Strategy & Philosophy

Deliver the shared Rust core, process-management capability, and the existing
cleanup flow through a Tauri + React desktop shell. Validate an interactive
demo before integrating the production UI. Keep the existing CLI available
throughout the migration; validate mutations using fixtures and disposable
children rather than live user files or applications.

### MVP Feature Set

The accepted first release is defined by PC-P1 through PC-P5, PC-C1 through
PC-C6, and PC-D1 through PC-D3 above. Earlier Smart Scan-first requirements
and the following versioned roadmap are broader product references, not
additional release gates for this MVP. No schedule or distribution date has
been agreed for the revised scope.

### Post-MVP Features

**Phase 2 — v1.1: Space Lens (~6–8 tuần sau MVP ship)**

Mục tiêu: Mở khoá insight "explain causes" thật sự — treemap + heat-by-age + phantom space. Khối kỹ thuật lớn nhất của toàn project.

- Engine walker mới: scan toàn ổ (`~` hoặc `/`), parallel walker (rayon hoặc tokio-based ở src-tauri layer), respect symlink boundary.
- Treemap React component (d3-hierarchy + custom render, virtualization cho ổ lớn) + Sunburst toggle.
- Heat-by-age + heat-by-type tô màu ô.
- Drill-down + breadcrumb + back navigation (⌫ key).
- Phantom space detection (APFS purgeable + snapshot via `diskutil apfs list`).
- Action bar trên ô: Reveal in Finder, Quick Look, Move to Trash, Send to `clean`.
- Smart ignore (loại trừ cloud-synced folder).
- Exclusion folder lock (Settings → Protected Folders).

**Phase 3 — v1.2: Menubar + Active explanation (~4–6 tuần)**

- Menubar tray icon với realtime sparkline (CPU/RAM/disk).
- Notification system chủ động ("đà này 18 ngày nữa đầy").
- "Tuần này có gì đổi" panel (so 2 snapshot SQLite).
- Snapshot history + diff view.
- Top file "sống" badge (Downloads/Desktop/Documents watch qua FSEvents).
- Menubar-only mode (Settings → Hide Dock icon).

**Phase 4 — v1.3: `focus` + cleanup (~2–3 tuần)**

- Tích hợp `tiny focus` (timer Pomodoro) với cleanup nền.
- "Cleanup during focus" toggle trong focus settings.
- Hết session → notification "đã giải phóng X GB".
- Choose which categories run during focus (default: chỉ Nhóm xanh an toàn).

**Phase 5 — v1.4: Maintenance + Protection (~8–10 tuần — khối nặng + rủi ro)**

- Free up RAM, flush DNS, reindex Spotlight, repair disk permissions, rebuild Launch Services, run periodic — mỗi cái toggle riêng.
- Login Items + Launch Agents management (`SMAppService`).
- Heavy consumers panel (process ngốn CPU/RAM, đề xuất quit).
- Malware/adware scan với signature database — **đây là rủi ro lớn nhất** vì cần ML/signature pipeline.
- Privacy cleaner (lịch sử browser, cookies, recent items).
- App Updater (phát hiện app outdated ngoài App Store) — tích hợp với Homebrew/Sparkle.
- Permissions audit (TCC) — đọc `tccutil` để hiển thị app giữ quyền nhạy cảm.

**Vision — v2.0+ (open-ended)**

- Rules engine (`~/.config/tiny/rules.toml`).
- Auto-clean "PR chờ duyệt" mode.
- AI ranking category theo (safety × dung lượng).
- Fleet view multi-Mac.
- "Chuẩn bị bán máy" mode.
- Linux/Windows port (nếu `tiny-core` đã runtime-agnostic chuẩn).

### Risk Mitigation Strategy

**Technical Risks**:

| Rủi ro                                                                    | Likelihood               | Impact | Mitigation                                                                                                             |
| ------------------------------------------------------------------------- | ------------------------ | ------ | ---------------------------------------------------------------------------------------------------------------------- |
| Refactor `clean` tách `dialoguer` khỏi compute khó hơn dự kiến (D1 block) | Medium                   | High   | Spike 1 tuần riêng cho 1 provider (vd: `trash`) để đo độ phức tạp; nếu > 2 tuần → tách thành pre-MVP milestone         |
| Tauri 2.x learning curve quá dốc                                          | Low (user đã quen React) | Medium | Học qua scaffolding `create-tauri-app` + 1 tiny prototype không scope (helloworld + 1 invoke command). Estimate 2 tuần |
| FDA onboarding flow flaky (TCC quirks)                                    | Medium                   | High   | Test trên 3 fresh user account (Minh/Hà/Long persona) — đo % onboard success first attempt                             |
| APFS purgeable detection unreliable                                       | Medium                   | Medium | Mark phantom space là "ước tính"; không xoá purgeable space (chỉ explain). Phase 2 problem                             |
| Treemap render lag với ổ 500k+ files                                      | High                     | Medium | Virtualization (chỉ render visible tile), throttle re-render, defer detail load đến drill. Phase 2 problem             |
| macOS notarization fail sau 1 lần thay đổi build                          | Medium                   | Medium | CI release workflow test notarize trên branch trước khi tag release                                                    |

**Market Risks**:

| Rủi ro                                 | Likelihood | Impact | Mitigation                                                                                                   |
| -------------------------------------- | ---------- | ------ | ------------------------------------------------------------------------------------------------------------ |
| 0 adoption sau khi ship (no community) | High       | High   | Pre-MVP: 5 dev bạn bè dùng beta, feedback loop. Post-MVP: post lên HN/r/macapps tuần đầu, sẵn sàng AMA       |
| User non-tech phàn nàn UI quá kỹ thuật | Medium     | Low    | Persona C (Bác Long) là edge case — ưu tiên Persona A/B. Có thể ẩn "Equivalent CLI" qua heuristic phase 2    |
| CleanMyMac copy `focus + cleanup`      | Low        | Low    | Differentiator còn 4 trục khác (open-source, phô CLI, undo Cmd+Z, explainability). Không phụ thuộc 1 feature |
| GitHub repo bị spam issue/PR           | Low        | Low    | CONTRIBUTING.md chặt, label spam, có thể lock repo nếu cần                                                   |

**Resource Risks**:

| Rủi ro                                    | Likelihood | Impact | Mitigation                                                                                                                                 |
| ----------------------------------------- | ---------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Maintainer kiệt sức trước khi ship        | High       | High   | Lock MVP scope cứng — bất kỳ feature creep nào đẩy v1.1. Sprint 2 tuần, retro mỗi sprint                                                   |
| Apple Developer $99/năm cản phát hành     | Low        | Medium | Có budget cá nhân. Nếu không, fallback: distribute không sign (user phải Right-Click → Open lần đầu) — chấp nhận UX kém hơn                |
| Test machine không đủ (chỉ có 1 Mac)      | Medium     | Medium | CI GitHub Actions `macos-14` (M-series) cho CI test. Cross-version test (macOS 13/14/15) qua VM hoặc nhờ contributor                       |
| Không có người review React frontend code | Medium     | Low    | Frontend tách module rõ ràng (`features/ → components/ → lib/`), self-review qua agent tools (`/code-review` skill), Vitest coverage > 70% |

## Functional Requirements

> **Capability contract** — UX designer, architect, và epic breakdown chỉ làm những gì liệt kê ở đây. Bất kỳ feature nào không có FR = không tồn tại trong sản phẩm. Phase mỗi FR ghi rõ trong `[Pn]` ở cuối.

### Cleanup & Categories

- **FR1**: User can trigger a scan of all 21+ cleanup categories with a single action and see total reclaimable disk space. `[P1]`
- **FR2**: User can view cleanup categories grouped into 3 safety lanes (🟢 Safe / 🟡 Review / 🔴 Destructive), with 🟢 ticked by default and 🟡/🔴 requiring explicit opt-in via a flag. `[P1]`
- **FR3**: User can drill into any category to inspect every file targeted for deletion, with size, path, and last-access timestamp shown per file. `[P1]`
- **FR4**: User can selectively include or exclude individual files within a category before executing cleanup. `[P1]`
- **FR5**: User can execute cleanup on selected categories and receive an end-of-run report showing freed space, files moved to quarantine, files sent to Trash, and files skipped (with reason). `[P1]`
- **FR6**: Contributor can add a new cleanup category by implementing the `CleanProvider` trait and registering at three registry sites, with a registry-sync test enforcing all sites are kept in sync. `[P1, ongoing]`
- **FR7**: User can clean up app leftovers (caches, prefs, launch agents, containers) for any installed app via the `uninstall` flow. `[P1, existing]`
- **FR8**: User can discover and clean duplicate files across the home directory by content hash. `[P1, existing via CLI]`

### Safety Net & Undo

- **FR9**: System sends every deleted file either to macOS Trash (Finder Put-Back) or to a `~/.tiny/quarantine/` retention area, never directly to permanent deletion, unless the user explicitly invokes a hard-delete flag. `[P1]`
- **FR10**: User can restore any file from quarantine within 30 days via an in-app "Cleanup History" panel, organized per cleanup session. `[P1]`
- **FR11**: User can undo the most recent cleanup session entirely via Cmd+Z keyboard shortcut, restoring all files from that session's quarantine batch. `[P1]`
- **FR12**: System auto-purges quarantine entries older than 30 days via a background task and notifies user when quarantine size exceeds 10 GB. `[P1]`
- **FR13**: System refuses to delete files via symlink traversal — every filesystem operation must use `symlink_metadata` and canonicalize paths against per-provider whitelist before deletion. `[P1, architectural invariant]`
- **FR14**: System detects when an app is currently running before cleaning its caches, skips those caches, and surfaces the skip reason in the cleanup report. `[P1, existing in engine]`
- **FR15**: User can lock specific folders (e.g., Desktop, Documents) so no provider, regardless of safety lane, can touch them; lock is settable per-folder with optional password protection. `[P2]`

### Smart Scan & Health Score

- **FR16**: User can trigger a "Smart Scan" that runs `sys` + `scan` + `clean --dry-run` in sequence and produces a single Health Score (0–100) plus a one-line summary of reclaimable space. `[P1]`
- **FR17**: User can view the breakdown of inputs to the Health Score (disk pressure, junk count, memory pressure, quarantine age) and see explanations of each component's contribution. `[P1]`
- **FR18**: User can schedule Smart Scan to run automatically at a chosen interval (weekly/daily/never) with results delivered via system notification. `[P3]`

### Space Lens (Disk Visualization)

- **FR19**: User can view an interactive treemap representing all files and folders in their home directory or full volume, with tile size proportional to disk usage. `[P2]`
- **FR20**: User can drill down into the treemap via double-click and navigate back via breadcrumb or ⌫ key. `[P2]`
- **FR21**: User can toggle between treemap and sunburst visualization for the same dataset. `[P2]`
- **FR22**: User can apply color heatmaps to the visualization by file age, file type, or both. `[P2]`
- **FR23**: User can right-click any tile to reveal in Finder, Quick Look, send to Trash, send to quarantine, or invoke uninstall (for `.app` bundles). `[P2]`
- **FR24**: User can exclude system, cloud-synced, or pinned folders from Space Lens calculations. `[P2]`
- **FR25**: System detects and explains "phantom space" — APFS purgeable space, snapshots, and hidden system files that account for missing disk capacity. `[P2]`
- **FR26**: System uses live filesystem events (FSEvents) to re-scan only changed folders and update Space Lens incrementally, instead of re-scanning the entire volume. `[P2]`
- **FR27**: User can export Space Lens results to PNG image or JSON report. `[P2]`

### Active Monitoring & Explanation

- **FR28**: User can install a menubar widget displaying live CPU, RAM, and disk usage with sparkline history. `[P3]`
- **FR29**: User can click the menubar widget to open a mini-panel with current Health Score and a "Run Smart Scan" button. `[P3]`
- **FR30**: User can run the app in "menubar-only" mode, hiding the Dock icon entirely. `[P3]`
- **FR31**: System captures periodic disk-usage snapshots and stores them locally so user can ask "what changed this week?" and see which folders grew, by how much. `[P3]`
- **FR32**: System forecasts disk-full timeline based on usage trend and notifies user proactively when forecast falls below a configurable threshold. `[P3]`
- **FR33**: System detects "newly large files" in Downloads/Desktop/Documents via FSEvents watch and badges the app icon when a file above a configurable size threshold appears. `[P3]`
- **FR34**: User can inspect and manage processes through the Processes destination. The current MVP requirements are PC-P1 through PC-P5; eligible targets and confirmed actions replace the older unrestricted "quit any process" wording. `[Current MVP]`

### Focus Integration

- **FR35**: User can enable a "cleanup during focus" toggle in the focus timer settings, causing cleanup of safe-lane categories to run in the background during each Pomodoro session. `[P4]`
- **FR36**: System reports cleanup results to user as a focus-session-end notification, showing freed space and never interrupting the focus session itself. `[P4]`
- **FR37**: User can choose which categories run during focus sessions, with default restricted to 🟢 Safe lane only. `[P4]`

### CLI Parity & Transparency

- **FR38**: Every action available in the GUI has an equivalent `tiny ...` CLI command that produces the same effect when run independently. `[P1, architectural invariant]`
- **FR39**: Every GUI action displays its equivalent CLI command in a footer/info area with a one-click "Copy" button. `[P1]` *(Deferred from the MVP on 2026-10-07.)*
- **FR40**: User can toggle the CLI footer visibility in app settings (default ON for desktop app; per-user override). `[P1]` *(Deferred with FR39 on 2026-10-07.)*
- **FR41**: All CLI commands support `--json` output, returning the same data structure the GUI consumes via Tauri commands. `[P1]`
- **FR42**: All long-running CLI commands support a progress callback that emits structured updates (percent + current item), consumed by GUI via Tauri events. `[P1]`

### Maintenance & System Optimization

- **FR43**: User can free up RAM (purge inactive memory) with a single action and see RAM usage before/after. `[P5]`
- **FR44**: User can run macOS periodic maintenance scripts (flush DNS, reindex Spotlight, repair disk permissions, rebuild Launch Services) individually or as a batch, each as a separate toggle. `[P5]`
- **FR45**: User can view and disable Login Items and Launch Agents via a unified management panel. `[P5]`

### Protection (Malware, Privacy, Updater)

- **FR46**: System can scan for known PUP/adware signatures and quarantine matches. `[P5]`
- **FR47**: User can clear browser history, cookies, recent items, and saved Wi-Fi networks via a privacy cleaner module. `[P5]`
- **FR48**: System can detect outdated versions of apps installed outside the Mac App Store and offer to update them. `[P5]`
- **FR49**: User can audit which apps hold sensitive permissions (camera, microphone, Full Disk Access, Accessibility) and revoke selectively. `[P5]`

### Onboarding & Permissions

- **FR50**: System detects on launch whether Full Disk Access is granted and, if missing, shows an onboarding flow with step-by-step instructions to grant it via System Settings. `[P1]`
- **FR51**: System detects which categories require additional permissions (e.g., Accessibility for Login Items) and prompts only when those categories are first used. `[P1, P5]`

### Settings & Configuration

- **FR52**: User can configure quarantine retention period (default 30 days, range 7–90 days). `[P1]`
- **FR53**: User can configure auto-update behavior independently for check / download / install. `[P1]`
- **FR54**: User can opt in to anonymous crash reports (default OFF) with a settings toggle that requires explicit confirmation. `[P1]`
- **FR55**: User can clear local journal (cleanup history) and quarantine entirely from settings. `[P1]`
- **FR56**: User can define custom cleanup rules in a `~/.config/tiny/rules.toml` file (e.g., "`.dmg` in Downloads > 7 days → auto-delete") that the engine applies on schedule. `[Vision]`

### Distribution & Updates

- **FR57**: User can install the app via Homebrew Cask (`brew install --cask tiny`) with auto-update via `brew upgrade`. `[P1]`
- **FR58**: User can download a signed and notarized `.dmg` from GitHub Releases with checksums and signatures published. `[P1]`
- **FR59**: System checks for updates via GitHub Releases API on launch (max once per 24 h, configurable) and prompts user to download and install. `[P1]`

### Accessibility

- **FR60**: User can navigate the entire app via keyboard only, with visible focus indicators on every interactive element. `[P1]`
- **FR61**: User can scale the app font size in settings and the layout remains usable at 200% scaling. `[P1]`
- **FR62**: System maintains contrast ratio at WCAG AA level (4.5:1 for normal text, 3:1 for large text) across all themes. `[P1]`
- **FR63**: All destructive actions present a confirmation dialog with large-readable text and two clearly labeled buttons. `[P1]`

### Open-Source & Community

- **FR64**: Source code is publicly available on GitHub under a permissive open-source license (MIT or Apache 2.0). `[P1]`
- **FR65**: Contributor can read `CONTRIBUTING.md` and successfully submit a working PR (such as adding a new cleanup category) without needing one-on-one mentorship from the maintainer. `[P1]`
- **FR66**: Project uses Conventional Commits in Vietnamese (`<type>(<scope>): <mô tả>`) for all commits, enforced via commitlint or pre-commit hook. `[P1, existing convention]`

## Non-Functional Requirements

> NFR chỉ liệt kê category thực sự quan trọng với `tiny` — bỏ qua scalability đa người dùng / multi-tenant / cloud-grade reliability vì sản phẩm là **single-user desktop app, offline-first, open-source**. Mỗi NFR phải đo được (số cụ thể hoặc test gate cụ thể), không vague.

### Performance

Performance là **differentiator trực tiếp với user kỹ thuật** — chậm thì họ chuyển về `ncdu`/`dust`.

- **NFR-P1**: App cold start (Tauri launch → main window ready to interact) ≤ 800 ms P50, ≤ 1.5 s P95, hard cap 3 s. Baseline: M2 8 GB RAM, ổ SSD ~70% full.
- **NFR-P2**: `tiny sys` snapshot ≤ 150 ms P50, ≤ 200 ms P95.
- **NFR-P3**: `tiny scan` 3 folder mặc định (Downloads + Desktop + Documents, ~50k files trung bình) ≤ 2 s P50, ≤ 3 s P95.
- **NFR-P4**: Smart Scan end-to-end (sys + scan + clean --dry-run sequence) ≤ 4 s P50, ≤ 5 s P95, hard cap 10 s.
- **NFR-P5**: `clean --dry-run` cho 1 category ≤ 500 ms P50, ≤ 1 s P95.
- **NFR-P6**: `clean` execute 1 category (hybrid delete vào quarantine) ≤ 2 s P50, ≤ 5 s P95.
- **NFR-P7**: GUI frame budget 60 fps — mọi interaction (click, scroll, drill) hoàn thành render trong ≤ 16.6 ms P95. Không drop frame trong scan progress animation.
- **NFR-P8**: User-initiated cancel responsive ≤ 500 ms P50, ≤ 1 s P95 — không chấp nhận "wait until next checkpoint".
- **NFR-P9** (v1.1): `scan` toàn ổ (~500k files trên ổ SSD 500 GB) ≤ 30 s P50, ≤ 60 s P95. Parallel walker bắt buộc.
- **NFR-P10** (v1.1): Treemap render initial (sau khi walker xong, 500k files) ≤ 1 s P95 với virtualization (chỉ tile visible).
- **NFR-P11**: Memory footprint app idle ≤ 150 MB RSS; peak trong scan ≤ 500 MB RSS. Hard cap 1 GB — vượt → log warning.
- **NFR-P12**: CPU idle (app open, không action) ≤ 1% trung bình 1 phút trên M2.
- **NFR-P13** (v1.2): Menubar widget refresh interval 1 s, CPU cost ≤ 0.5% trung bình.

**Test gate**: bench-suite chạy trong CI mỗi PR; PR fail nếu regress > 20% trên bench mốc. Bench rời nằm ở `crates/tiny-core/benches/`.

### Security

Security focus = **filesystem integrity** + **supply chain integrity**, không phải auth/encryption (không có server-side, không cloud).

- **NFR-S1**: 0 instance của `std::fs::remove_file`, `fs::remove_dir`, `fs::remove_dir_all` trên đường chạy production (loại trừ `crates/tiny-core/src/clean/fs_safe.rs`). CI grep gate fail PR vi phạm.
- **NFR-S2**: 100% xoá filesystem đi qua `fs_safe::remove_recursive_safe` hoặc hybrid delete layer (`trash` crate / quarantine move). Code review checklist enforce.
- **NFR-S3**: 100% xoá filesystem dùng `symlink_metadata` (KHÔNG `is_dir()`) và canonicalize path. Có test ghim cho mọi provider (`tests/clean_smoke.rs` + provider unit test).
- **NFR-S4**: Provider registry sync 100% — `category_family()` phải có nhánh cho mọi id trong `known_category_ids()`. Test panic nếu thiếu (đã có).
- **NFR-S5**: `panic!`, `unwrap()`, `expect()` chỉ xuất hiện trong code path `#[cfg(test)]` hoặc invariant đã được test ghim (vd: `category_family()`). CI grep gate (allow-list cho test code).
- **NFR-S6**: Mọi Tauri command trả `Result<T, ErrorPayload>` — không panic crossing IPC boundary.
- **NFR-S7**: Release artifact (`.dmg`, `.app`) signed bằng Apple Developer ID + notarized qua Apple notarytool. Distribution không signed = fail release pipeline.
- **NFR-S8**: GitHub Releases publish kèm SHA256 checksum + GPG signature (key dài hạn của maintainer). Homebrew Cask formula verify checksum.
- **NFR-S9**: Dependency audit `cargo audit` chạy mỗi PR; fail nếu có CVE severity ≥ Medium. Whitelist với justification cho FP.
- **NFR-S10**: `npm audit` chạy mỗi PR cho frontend; fail nếu có vulnerability severity ≥ High trong production deps.
- **NFR-S11**: Update mechanism verify signature của downloaded `.dmg` bằng public key embed trong app trước khi prompt install.

### Privacy

- **NFR-Pr1**: Zero outbound network call mặc định cho mọi core feature (`sys`, `scan`, `clean`, `quarantine restore`, `focus`, `uninstall`). Audit qua integration test.
- **NFR-Pr2**: Settings toggle "Send anonymous crash report" default OFF. Bật yêu cầu explicit confirm dialog.
- **NFR-Pr3**: Crash report (nếu opt-in) **không** chứa file path, hostname, user name, hay IP. Chỉ stack trace + macOS version + app version.
- **NFR-Pr4**: Update check (gọi GitHub Releases API) là **chỉ HTTP request loại 1 mặc định**; user có thể tắt qua settings.
- **NFR-Pr5**: CLI command history **không** log argument; chỉ log command name + UTC timestamp.
- **NFR-Pr6**: Cleanup journal SQLite local-only ở `~/.tiny/journal.sqlite`; user có quyền xoá hoàn toàn qua Settings → Clear journal.

### Reliability

- **NFR-R1**: Crash-free session rate ≥ 99.5% (đo qua opt-in crash report khi có ≥ 100 active users). Pre-launch: 0 panic trong 50 dev session 1 tuần.
- **NFR-R2**: Transactional quarantine: journal entry ghi TRƯỚC khi move file. Nếu app crash giữa chừng, recovery on next launch khôi phục state nhất quán (không có file mồ côi không ai biết).
- **NFR-R3**: Quarantine auto-purge sau 30 ngày qua background task; failure tolerance — retry 3 lần, log warning, không block app.
- **NFR-R4**: Mọi long-running operation (scan/clean toàn ổ) hỗ trợ cancel mid-flight không corrupt state. Test ghim: kill -9 giữa clean → next launch không có quarantine entry dở.
- **NFR-R5**: Free disk space check trước mọi quarantine move: nếu free < 2× target size, fail-fast với clear error message; không attempt và rollback.
- **NFR-R6**: App restart không mất context — main view giữ trạng thái (selected category, last scan result hash) qua app restart.

### Accessibility

- **NFR-A1**: Full keyboard navigation — mọi interactive element reachable qua Tab/Shift+Tab; visible focus indicator (≥ 2px outline contrast ≥ 3:1).
- **NFR-A2**: Contrast ratio WCAG AA: text ≥ 4.5:1 (small ≤ 18pt), ≥ 3:1 (large ≥ 18pt bold hoặc 24pt regular). Test gate: axe-core trong Vitest e2e.
- **NFR-A3**: Font scaling responsive — UI usable tại system font scale 200%; layout không vỡ.
- **NFR-A4**: VoiceOver compatible — mọi action có ARIA label / aria-describedby; landmark regions (`main`, `nav`, `aside`) đúng.
- **NFR-A5**: Color không phải means truyền tải thông tin duy nhất — làn 3 màu kèm icon + label text (🟢 "Safe", 🟡 "Review", 🔴 "Destructive").
- **NFR-A6**: Confirmation dialog destructive action — chữ ≥ 14pt, 2 button rõ ràng (label "Hủy" và "Xoá vào Trash" cụ thể, không "OK"/"Cancel" mờ nghĩa).
- **NFR-A7**: Reduced Motion respect — nếu macOS `Reduce Motion` bật, tắt mọi animation > 200ms.

### Integration

- **NFR-I1**: Tauri IPC contract — mọi struct qua boundary có `#[serde(rename_all = "camelCase")]`; mọi error là `ErrorPayload` serializable. Test integration kiểm tra contract.
- **NFR-I2**: Homebrew Cask formula auto-update khi GitHub Release mới publish; PR sang `homebrew/homebrew-cask` chính thức (hoặc maintain tap riêng `homebrew-tiny`).
- **NFR-I3**: GitHub Releases API rate-limit aware — update check tôn trọng `X-RateLimit-Remaining` header, exponential backoff khi 403.
- **NFR-I4**: CLI command stable contract — argument breaking change = major version bump, có migration notes trong CHANGELOG. JSON output schema versioned (`"version": 1`).
- **NFR-I5** (v1.5+): If app updater module (FR48) reaches scope, integration với Homebrew (`brew outdated --cask`) + Sparkle feed parse.

### Maintainability & Observability (open-source critical)

Open-source = contributor-first. Maintainability là requirement, không phải nice-to-have.

- **NFR-M1**: Test coverage `tiny-core` ≥ 80% (line); CLI integration test cover happy path mọi command; frontend Vitest coverage ≥ 70%.
- **NFR-M2**: File size discipline — mỗi file ≤ 400 dòng trung bình, hard cap 800. CI warn (không block) nếu vượt 600.
- **NFR-M3**: Dependency count — production dependency `tiny-core` ≤ 15 direct crate; CLI `tiny` ≤ 20 direct crate. Frontend ≤ 30 direct npm package. New dep cần justification trong PR.
- **NFR-M4**: Public API của `tiny-core` documented qua `///` rustdoc; `cargo doc --no-deps` build clean (0 warning).
- **NFR-M5**: Provider authoring — thêm 1 cleanup provider mới = ≤ 1 file mới + 3 line trong registry + 1 file test. Time budget contributor mới: ≤ 1 giờ end-to-end (từ clone repo đến PR submit).
- **NFR-M6**: Local dev setup — `git clone && cargo run -p tiny -- scan` work trên fresh macOS với chỉ `rustup` cài sẵn. Time: ≤ 5 phút (không tính download).
- **NFR-M7**: GUI dev `npm install && npm run tauri dev` work trên fresh setup với `node 20+ + rustup`. Time: ≤ 10 phút.
- **NFR-M8**: Logging level configurable qua env `RUST_LOG`; default WARN cho production, DEBUG cho dev. Log structured (json hoặc structured text), KHÔNG log path nhạy cảm.

### Compatibility & Distribution

- **NFR-C1**: macOS 13 Ventura+ supported. CI test matrix: macOS 13 + 14 + 15 (latest).
- **NFR-C2**: Apple Silicon (M1+) primary; Intel Mac không support v1.0, đánh giá lại v1.x dựa trên contributor + test machine availability.
- **NFR-C3**: `.dmg` bundle size ≤ 15 MB (Tauri tối ưu, không bundle Electron runtime). Hard cap 25 MB.
- **NFR-C4**: Backward compat — `tiny-core` API change minor version = additive only; major version = có migration guide cho CLI consumer.
- **NFR-C5**: Settings file format (`tauri-plugin-store` JSON + `~/.config/tiny/rules.toml`) backward-compatible trong major version; migration tự động khi cần.

## Design Principles & UX Language

Phần này gom các "soft" idea từ brainstorm (tone, vibe, ngôn ngữ) — quan trọng vì những idea này không map vào FR/NFR cứng nhưng định hình cảm giác sản phẩm. Designer + reviewer dùng phần này để đánh giá "tinh thần" mỗi feature.

### Vibe — "htop + ncdu + lazygit"

Vibe tham khảo từ brainstorm: **keyboard-driven, visual, ít gõ, instant feedback**. Cụ thể:

- **Keyboard-first**: mọi action thường dùng có phím tắt (Cmd+S Smart Scan, Cmd+Z Undo, Cmd+, Settings, ⌫ back navigation, Tab/Shift+Tab navigation). Power user không cần chuột.
- **Instant feedback**: progress event emit liên tục từ engine; loading state không bao giờ static spinner > 500 ms mà không có byte count / file count / percent.
- **Visual without bloat**: dùng màu/icon thay vì text dài; làn 3 màu là core pattern; treemap (v1.1) thay cho bảng số khi đã có Space Lens.
- **No-modal-where-possible**: prefer inline expansion (drill, drawer) hơn modal dialog; modal chỉ cho destructive confirmation (`FR63`).

### Tone & Voice

- **Minh bạch, không patronizing**: không giấu chi tiết kỹ thuật. Power user thấy data thật (file path, size byte, exit code). Beginner không bị block bởi sự minh bạch đó — vì có visual layer phía trên.
- **Không marketing-speak**: tránh "boost", "supercharge", "blazingly fast", "AI-powered". Dùng "scan", "clean", "free", "explain".
- **Tiếng Việt là first-class** trong commit message + PR convention; UI label có thể đa ngôn ngữ (v1.x), nhưng error message / log của engine tiếng Anh để contributor quốc tế đọc được.
- **Đối lập CleanMyMac tone**: không có badge "Critical alert", không scare-tactic ("Your Mac is at risk!"). Health Score đỏ chỉ "Cần dọn ~18 GB" — neutral.

### UX Writing — Human Language

Brainstorm idea [#42]: thay con số GB bằng ngôn ngữ đời thường. Áp dụng có chọn lọc:

- **Cùng hiển thị cả hai**: "18.4 GB · đủ chỗ cho ~400 ảnh hoặc 2 phim 4K". Số byte cho power user, ngôn ngữ đời thường cho non-tech. Default: cả hai (toggle settings để tắt secondary unit).
- **Thời gian human-readable**: "3 ngày trước" thay vì "2026-05-17T09:23:11Z" trong UI. Hover hiện full timestamp. CLI luôn dùng ISO 8601.
- **Action verb cụ thể**: "Đưa vào quarantine 30 ngày" thay vì "Delete"; "Hoàn tác" thay vì "Restore"; "Phô CLI" thay vì "Show command".

### Cleanup Result as Diff (#49)

Khi cleanup xong, kết quả trình bày dạng **diff-like**, không dạng list-of-files:

```
+ 18.4 GB freed
- xcode_derived_data    -8.2 GB   (4 projects, 1,247 files → quarantine)
- npm_cache             -2.1 GB   (cleared)
- docker_overlay        -4.8 GB   (2 unused images → trash)
- quarantine_30d_expired -1.3 GB   (auto-purge)
- safari_cache          -2.0 GB   (cleared)
```

Format này quen thuộc với dev (giống `git diff`/`git status`) và explicit: thấy ngay total + breakdown + destination (quarantine vs trash vs cleared).

### Modes Spectrum

Brainstorm có cluster ý tưởng từ "rất chủ động" (#43 ambient, #56 silent) đến "user kiểm soát chặt" (#52 PR-style approval). Hỗ trợ cả spectrum qua **settings**:

| Mode                  | Behavior                                                              | Default    |
| --------------------- | --------------------------------------------------------------------- | ---------- |
| **Manual**            | User mở app, bấm Smart Scan, review, clean. Mặc định MVP              | ✅ Phase 1 |
| **Scheduled**         | App chạy Smart Scan định kỳ, notify user kết quả `[FR18]`             | Phase 3    |
| **Focus-triggered**   | Cleanup chạy nền trong focus session, report cuối session `[FR35-37]` | Phase 4    |
| **PR-style approval** | App tự dọn nền, gom thành "PR" chờ user duyệt batch                   | Vision     |
| **Ambient**           | Silent continuous cleanup kiểu Storage Sense (chỉ category xanh)      | Vision     |

Không force mode — user chọn. Default Manual để giữ trust trước.

### Philosophy — "Explain, don't just show"

Quy tắc thiết kế xuyên suốt: **mỗi panel số phải trả lời được "tại sao"**.

- Health Score 51/100 → click breakdown thấy "60% disk pressure + 30% junk pile-up + 10% quarantine age" — không chỉ con số.
- "18.4 GB có thể dọn" → click thấy 5 category contributing, không phải "trust me".
- "Phantom space 12 GB" → tooltip giải thích APFS snapshot + link Apple doc.
- "Tuần này +24 GB" → click thấy thư mục/file cụ thể phình.

Designer reviewer dùng quy tắc này: **nếu một số xuất hiện mà không có path drill-down giải thích → fail review**.

## Document Map (Quick Reference)

| Section                              | Mục đích                                                                         | Audience chính               |
| ------------------------------------ | -------------------------------------------------------------------------------- | ---------------------------- |
| Executive Summary                    | Tóm 1 trang: vision + differentiator + classification                            | Stakeholder, contributor mới |
| Success Criteria                     | User/Business/Technical success + measurable outcomes                            | PM, QA                       |
| Product Scope (Phase)                | MVP scope + Growth phases + Vision                                               | Toàn team                    |
| User Journeys                        | 4 persona narrative (A/B/C/D)                                                    | UX, PM                       |
| Domain-Specific Requirements         | macOS platform constraints + filesystem safety invariants                        | Engineering                  |
| Innovation & Novel Patterns          | 3 innovation thực sự + competitive landscape + validation                        | Stakeholder, marketing       |
| Desktop App Specific Requirements    | Workspace layout + tech stack + platform/system/update/offline                   | Architect, engineering       |
| Project Scoping & Phased Development | MVP strategy + 5 phase roadmap + risk matrix                                     | PM, maintainer               |
| Functional Requirements              | FR1–FR66, capability contract, gắn phase tag `[Pn]`                              | Engineering, QA, UX          |
| Non-Functional Requirements          | Performance/Security/Privacy/Reliability/A11y/Integration/Maintainability/Compat | Architect, SRE, QA           |
| Design Principles & UX Language      | Vibe + tone + UX writing + modes spectrum + "explain don't show"                 | Designer, UX writer          |
