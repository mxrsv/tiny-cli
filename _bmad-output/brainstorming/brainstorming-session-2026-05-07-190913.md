---
stepsCompleted: [1, 2, 3, 4]
session_active: false
workflow_completed: true
inputDocuments: []
session_topic: "macOS native app cho tiny-cli (GUI app kiểu CleanMyMac, hoặc hơn thế)"
session_goals: "Danh sách ý tưởng features mở rộng để chọn lọc sau (open-ended ideation)"
selected_approach: "ai-recommended"
techniques_used: ["Analogical Thinking", "SCAMPER Method"]
techniques_skipped: ["What If Scenarios"]
ideas_generated: 68
technique_execution_complete: true
context_file: ""
session_continued: true
continuation_date: "2026-05-15"
topic_pivot: "TUI dashboard -> macOS native GUI app (2026-05-15)"
---

# Brainstorming Session Results

**Facilitator:** Kyantran
**Date:** 2026-05-07

## Session Overview

**Topic:** macOS **native GUI app** cho `tiny-cli` — một app có cửa sổ, icon trên Dock, kiểu CleanMyMac hoặc hơn thế. Engine bên dưới là Rust CLI `tiny` (`sys` + `scan` + `clean`).

**Goals:** Danh sách ý tưởng features mở rộng để chọn lọc sau (open-ended ideation, chưa cần scope MVP).

> **Cập nhật 2026-05-15:** Phiên này ban đầu là về TUI dashboard. Vì chưa sinh ý tưởng nào, chủ đề được pivot sang **macOS native app** kiểu CleanMyMac. Bộ technique (Analogical → SCAMPER → What If) giữ nguyên.

### Context Guidance

Project hiện tại là Rust CLI (`tiny`) với 3 commands chính:

- `sys` — system info (OS, host, uptime, CPU, memory, disk).
- `scan` — read-only scan `~/Downloads`, `~/Desktop`, `~/Documents` (largest + oldest files).
- `clean` — interactive cleanup, 21 categories visible mặc định, có `--include-review` / `--include-destructive`, supports `--dry-run`, `--hard`, `--category`.

Dependencies hiện tại: `clap`, `anyhow`, `serde`, `sysinfo`, `dialoguer`. TUI sẽ thêm 1 lib mới (chưa quyết — `ratatui` là ứng viên mặc định).

Vibe tham khảo: `htop` + `ncdu` + `lazygit` — keyboard-driven, visual, ít gõ command, instant feedback.

### Session Setup

User chọn TUI làm **dashboard tổng** (Hướng B), không phải frontend riêng cho `clean` hay app riêng biệt. Mục tiêu là sinh ý tưởng features rộng rãi trước khi scope.

## Technique Selection

**Approach:** AI-Recommended Techniques

**Analysis Context:** Topic là product cụ thể có market reference (CleanMyMac, htop, lazygit). Goal là divergent feature ideation. User vibe technical/concrete.

**Recommended Sequence:**

- **Phase 1 — Analogical Thinking** (creative): Mượn features từ tools tương tự để sinh baseline + bất ngờ. Leverage CleanMyMac context vừa load.
- **Phase 2 — SCAMPER Method** (structured): 7 lens × 3 commands hiện hữu = 21 ô ý tưởng có structure, anti-bias mạnh.
- **Phase 3 — What If Scenarios** (creative): Push qua obvious ideas tìm killer feature/differentiator.

**Backup techniques:** Reverse Brainstorming, Persona Journey, Question Storming.

**AI Rationale:** Sequence này đi từ external reference (Analogical) → systematic internal expansion (SCAMPER) → frontier exploration (What If) — tăng dần độ "uncomfortable" để vượt qua obvious ideas.

## Technique Execution Results

### Phase 1 — Analogical Thinking

**Nguồn analog đã chọn:** A — CleanMyMac / CleanMyMac X
**Định hướng user:** Tập trung vào **core features** (3 trụ cột gốc), gạt nhánh phụ (Malware, Updater, Shredder).

#### Ý tưởng mượn từ CleanMyMac (lượt 1)

**[#1] Smart Scan — 1 nút "Quét"**
_Concept_: Một nút to, bấm chạy gộp `sys` + `scan` + `clean --dry-run`, ra 1 con số "Có thể giải phóng X GB". Zero-decision.
_Novelty_: Đảo ngược mô hình `tiny` hiện tại (user tự chọn category) thành máy tự đề xuất.

**[#2] Menubar Health Widget**
_Concept_: Icon thường trú trên menubar hiện CPU/RAM/disk realtime, click ra mini-panel + nút dọn nhanh.
_Novelty_: Biến `tiny sys` từ snapshot 1 lần thành dòng chảy liên tục.

**[#3] App Uninstaller**
_Concept_: Kéo-thả app vào cửa sổ → tìm sạch leftover (caches, prefs, launch agents, container) rồi xoá cả cụm.
_Novelty_: `clean` đã dọn `app_orphans` sau khi gỡ; đây là gỡ sạch ngay từ đầu.

**[#4] Space Lens — bản đồ ổ đĩa (treemap)**
_Concept_: Treemap trực quan, ô to = file/folder nặng, click zoom sâu.
_Novelty_: `tiny scan` chỉ ra list; treemap cho cảm giác "thấy được" cả ổ đĩa.

**[#5] Maintenance 1-Click**
_Concept_: Gom flush DNS, giải phóng RAM, reindex Spotlight, periodic scripts vào 1 nút.
_Novelty_: Mở rộng `tiny` từ "xoá file" sang "tinh chỉnh hệ thống".

**[#6] App Updater** _(đánh dấu nhánh phụ — tạm gác theo định hướng user)_
_Concept_: Quét app outdated, update hàng loạt.

#### Khảo sát hiện trạng `tiny` vs CleanMyMac (2026-05-15)

`tiny` hiện có 5 command: `sys`, `scan`, `focus`, `uninstall`, `clean`.

| Module CleanMyMac           | `tiny` hiện tại                 | Trạng thái                                     |
| --------------------------- | ------------------------------- | ---------------------------------------------- |
| System Junk / Cleanup       | `tiny clean` — 31 categories    | ✅ Có (mạnh hơn ở dev caches)                  |
| App Uninstaller             | `tiny uninstall`                | ✅ Có sẵn                                      |
| Large & Old Files           | `tiny scan`                     | ✅ Có                                          |
| Duplicate Finder            | `tiny scan --duplicates --hash` | ✅ Có sẵn                                      |
| System Monitor              | `tiny sys`                      | ⚠️ Một phần (snapshot, không realtime/menubar) |
| Space Lens (treemap)        | —                               | ❌ Thiếu                                       |
| Smart Scan (1 nút gộp)      | —                               | ❌ Thiếu                                       |
| Maintenance scripts         | —                               | ❌ Thiếu                                       |
| Malware / Updater / Privacy | —                               | ❌ Thiếu                                       |

**Insight then chốt:** Engine đã có gần đủ phần "core cleanup". macOS app = (1) khoác GUI cho cái có sẵn + (2) lấp 4 module thiếu. User chốt: làm **full parity** — thêm cả 4 module thiếu.

#### Đào sâu core — 3 trụ cột (lượt 2)

**[#7] Làn an toàn 3 màu** — 31 categories gom thành 🟢 An toàn / 🟡 Cần xem / 🔴 Nguy hiểm; mặc định chỉ tick xanh. _Novelty_: GUI-hoá flag `--include-review`/`--include-destructive`.

**[#8] Xem trước khi xoá** — click category → bung file list thật + checkbox từng file. _Novelty_: `--dry-run` dạng text → duyệt từng file có size/hình.

**[#9] Thùng cách ly có hoàn tác** — đồ xoá vào quarantine 30 ngày, restore được. _Novelty_: nâng category `quarantine` thành cơ chế undo toàn app.

**[#10] Top file "sống"** — theo dõi Downloads/Desktop/Documents liên tục, badge khi có file nặng mới. _Novelty_: biến `scan` từ snapshot thành cảnh báo chủ động.

**[#11] "Tuần này có gì đổi"** — so dung lượng theo thời gian, chỉ ra nguyên nhân phình. _Novelty_: nói _nguyên nhân_, không chỉ _tổng_.

**[#12] Đồ thị realtime** — sparkline CPU/RAM/disk/network cuộn liên tục.

**[#13] Bắt thủ phạm ngốn tài nguyên** — list process nặng nhất, kill từ GUI.

**[#14] Điểm sức khoẻ máy (0–100)** — 1 con số tổng hợp disk + rác + áp lực bộ nhớ, làm màn hình chính. _Novelty_: cho user _một_ chỉ số thay vì 31 categories.

#### 4 module CleanMyMac thêm mới vào `tiny`

**Module M1 — Space Lens (treemap)**

**[#15] Treemap tương tác** — bản đồ toàn ổ đĩa, ô = size, double-click zoom, breadcrumb quay lại.
**[#16] Cầu nối hành động** — click ô trên treemap → chuột phải "Xoá / Gỡ / Cách ly", gọi thẳng engine `clean`/`uninstall`.
**[#17] Danh sách loại trừ** — pin folder hệ thống / đang dùng để treemap khỏi đếm nhầm.

**Module M2 — Smart Scan**

**[#18] Nút quét gộp** — 1 nút chạy `scan` + `clean --dry-run` + `sys`, ra "X GB rác + Y cảnh báo".
**[#19] Kết quả chia 3 tab** — Cleanup / Protection / Speed (đúng cấu trúc CleanMyMac), mỗi tab có nút Run.
**[#20] Lịch quét tự động** — Smart Scan chạy nền hằng tuần, gửi notification.

**Module M3 — Maintenance / Speed**

**[#21] Bộ script bảo trì** — flush DNS, reindex Spotlight, repair disk permissions, rebuild Launch Services, run periodic — mỗi cái 1 toggle.
**[#22] Free up RAM** — nút purge bộ nhớ, hiện RAM trước/sau.
**[#23] Quản lý Login Items & Launch Agents** — list cái khởi động cùng máy, tắt từng cái.
**[#24] Heavy consumers** — process ngốn CPU/RAM, đề xuất quit (mở rộng #13).

**Module M4 — Protection (Malware + Privacy + Updater)**

**[#25] Malware/adware scan** — quét known PUP/adware theo signature, cô lập vào quarantine.
**[#26] Privacy cleaner** — xoá lịch sử duyệt web, cookies, recent items, danh sách wifi đã lưu.
**[#27] App Updater** — phát hiện app ngoài App Store bản cũ, update hàng loạt.
**[#28] Permissions audit** — app nào giữ quyền nhạy cảm (camera, mic, full disk, accessibility), thu hồi nhanh.

#### Đào sâu Module M1 — Space Lens (treemap)

**Quét & hiệu năng**

**[#29] Quét toàn ổ đĩa thật** — scan đệ quy cả `~` (hoặc `/`), không chỉ 3 folder mặc định. Cần engine walker mới: song song, nhanh. _Novelty_: `scan` hiện chỉ chạm Downloads/Desktop/Documents.
**[#30] Live re-scan từng phần** — dùng FSEvents watch, chỉ quét lại folder thay đổi, treemap cập nhật mượt thay vì quét lại cả ổ.
**[#31] "Phantom space"** — phát hiện purgeable space, APFS snapshot, file hệ thống ẩn mà Finder không hiện. _Novelty_: giải thích vì sao "ổ đầy mà không thấy file".

**Cách nhìn (visualization)**

**[#32] Hai chế độ xem** — Treemap (ô vuông lồng) ↔ Sunburst (vòng tròn đồng tâm), toggle.
**[#33] Heat by age** — tô màu ô theo tuổi file (cũ → đỏ). "Vừa to vừa cũ" = ứng viên xoá số 1.
**[#34] Heat by type** — tô màu theo loại (video/ảnh/code/archive), thấy ngay "60% ổ đĩa là video".

**Tương tác**

**[#35] Drill + breadcrumb + back** — double-click zoom vào, breadcrumb path trên cùng, phím ⌫ lùi ra.
**[#36] Thanh hành động trên ô** — chọn ô → Reveal in Finder / Quick Look / Cách ly / Gỡ (nếu .app) / Gửi sang `clean`. Mở rộng [#16].

**Theo dõi theo thời gian**

**[#37] So sánh 2 snapshot** — chụp treemap, tuần sau mở lại xem "ô nào phình". Tích hợp [#11].
**[#38] Watch folder** — ghim folder hay phình (Downloads, `~/Library/Developer`) lên dashboard, theo dõi riêng.

**An toàn & chia sẻ**

**[#39] Bỏ qua thông minh** — tự loại trừ folder hệ thống / cloud-synced (iCloud, Dropbox) khỏi gợi ý xoá, tránh xoá nhầm file đang sync.
**[#40] Export treemap** — xuất ảnh PNG hoặc report JSON "ổ đĩa của tôi".

#### Phase 1 — Tổng kết

- **40 ý tưởng** từ 1 nguồn analog (CleanMyMac).
- Scope chốt: macOS app = full CleanMyMac parity (6 module), trong đó 4 module thêm mới vào `tiny`.
- **Insight chủ đạo:** ý tưởng hay nhất là loại _"giải thích nguyên nhân"_ chứ không chỉ _"hiển thị"_ — phantom space, heat-by-age, so sánh snapshot, "tuần này có gì đổi".

### Phase 2 — SCAMPER Method

Áp 7 lăng kính SCAMPER lên app `tiny` (engine = Rust CLI, GUI = 6 module).

**S — Substitute (thay thế)**

**[#41] Thay "user chọn category" bằng "AI xếp hạng"** — engine chấm điểm category theo độ an toàn × dung lượng thu được, app chỉ hiện top đề xuất.
**[#42] Thay con số GB bằng ngôn ngữ đời thường** — "đủ chỗ cho 400 tấm ảnh" / "2 phim 4K" thay vì "12.4 GB".
**[#43] Thay scan thủ công bằng scan nền liên tục** — app không có nút "Scan", nó luôn biết sẵn.
**[#44] Thay quarantine folder riêng bằng Trash thật của macOS** — undo qua Finder, không tạo cơ chế lạ.

**C — Combine (kết hợp)**

**[#45] Gộp `clean` + `uninstall`** — khi gỡ app, tự đề xuất dọn luôn cache cùng họ.
**[#46] Gộp `sys` monitor + `clean`** — khi RAM/disk áp lực cao, app tự popup gợi ý dọn đúng lúc.
**[#47] Gộp treemap + `clean`** — ô đỏ trên treemap có nút "Dọn" ngay tại chỗ.
**[#48] Gộp `focus` + cleanup** — "trong lúc bạn focus 25 phút, tôi dọn nền". Cleanup chạy trong focus session.

**A — Adapt (điều chỉnh / mượn cách làm)**

**[#49] Cleanup dạng "diff"** — mượn `git diff`: hiện kết quả dọn như diff trước/sau, từng dòng +/- dung lượng.
**[#50] Undo stack kiểu editor** — Cmd+Z hoàn tác lần dọn gần nhất.
**[#51] Rules engine kiểu Mail rules** — user tự định nghĩa rule: ".dmg trong Downloads > 7 ngày → tự xoá".
**[#52] Auto-clean như "PR chờ duyệt"** — mỗi lần dọn nền, app gửi 1 bản tóm tắt để user duyệt rồi mới "merge".

**M — Modify / Magnify (phóng to / thu nhỏ)**

**[#53] Deep Clean mode** — quét sâu, lâu hơn, lùng cả category ẩn (vs Quick Scan).
**[#54] Menubar-only mode** — app không cửa sổ chính, sống hoàn toàn trên menubar (power user).
**[#55] Cleanup journal** — nhật ký mọi lần dọn, tổng "đã giải phóng 240 GB từ tháng 1".
**[#56] Ambient mode** — dọn liên tục, im lặng, không hỏi (kiểu Storage Sense).

**P — Put to other uses (dùng vào việc khác)**

**[#57] File finder tổng quát** — tái dùng engine scan để search file theo size/type/age toàn ổ.
**[#58] Báo cáo sức khoẻ máy PDF** — xuất report sys+clean, gửi cho người khác (IT support gia đình).
**[#59] Chế độ "chuẩn bị bán máy"** — wipe profile, reset, dọn sạch trước khi bán/cho.
**[#60] Fleet view** — 1 người quản nhiều Mac (gia đình / team nhỏ), dashboard tổng.

**E — Eliminate (loại bỏ)**

**[#61] Bỏ cửa sổ Preferences** — app zero-config.
**[#62] Ẩn khái niệm "category"** — người dùng thường chỉ thấy "Rác" + "File lớn"; category là chế độ nâng cao.
**[#63] Bỏ nút xác nhận cho category an toàn** — dọn xanh là dọn ngay, undo lo phần an tâm.
**[#64] Bỏ subscription** — bán 1 lần / mã nguồn mở, đối lập mô hình CleanMyMac _(angle business/differentiator)_.

**R — Reverse (đảo ngược)**

**[#65] Tìm thứ đáng giữ, không phải rác** — user đánh dấu file quan trọng, còn lại là ứng viên dọn.
**[#66] App chủ động đến tìm user** — notification "tối nay rảnh dọn 8 GB không?" thay vì user mở app.
**[#67] Dự báo thay vì chữa cháy** — "đà này 18 ngày nữa đầy ổ, dọn sớm không?".
**[#68] App phô CLI thay vì giấu** — mỗi action hiện command `tiny ...` tương đương (transparency + dạy CLI).

_[Phase 2 hoàn tất — 68 ý tưởng. Phase 3 (What If Scenarios) bỏ qua theo lựa chọn của user.]_

## Idea Organization and Prioritization

68 ý tưởng được gom lại thành **6 nhóm chủ đề** (cắt ngang 2 technique, không theo thứ tự sinh ra).

### Nhóm 1 — Bộ mặt "zero-decision" (10 ý tưởng)

_Focus: làm app dễ dùng với người không rành máy — giảm số quyết định người dùng phải đưa ra._
Ý tưởng: #1, #7, #14, #18, #19, #41, #42, #61, #62, #63.
**Pattern:** `tiny` hiện bắt user chọn trong 31 categories. Cả nhóm này đảo mô hình đó — máy đề xuất, user chỉ bấm. Health Score [#14] + Smart Scan [#18] là trục chính.

### Nhóm 2 — Space Lens: trực quan hoá ổ đĩa (14 ý tưởng)

_Focus: module mới — biến `scan` từ list text thành bản đồ tương tác._
Ý tưởng: #4, #15, #16, #17, #29, #30, #31, #32, #33, #34, #35, #36, #40, #47.
**Pattern:** cần engine walker mới (quét toàn ổ, song song). Giá trị cao nhất ở các biến thể "giải thích" — heat-by-age [#33], phantom space [#31].

### Nhóm 3 — An toàn, hoàn tác & minh bạch (8 ý tưởng)

_Focus: tạo niềm tin để user dám để app dọn._
Ý tưởng: #8, #9, #39, #44, #49, #50, #52, #68.
**Pattern:** dọn dẹp = thao tác đáng sợ. Undo [#50], xem trước [#8], phô CLI [#68] là "lưới an toàn" cho mọi hành động phá huỷ.

### Nhóm 4 — Chủ động, ngầm & giải thích (16 ý tưởng)

_Focus: app tự biết, tự nhắc, tự giải thích — không chờ user mở lên._
Ý tưởng: #2, #10, #11, #12, #13, #20, #37, #38, #43, #46, #51, #55, #56, #58, #66, #67.
**Pattern:** đây là nơi insight chủ đạo của phiên sống — "giải thích nguyên nhân, không chỉ hiển thị". Dự báo đầy ổ [#67], "tuần này có gì đổi" [#11].

### Nhóm 5 — Module hệ thống mở rộng: Maintenance + Protection (13 ý tưởng)

_Focus: 2 module CleanMyMac mới + vòng đời app._
Ý tưởng: #3, #5, #6, #21, #22, #23, #24, #25, #26, #27, #28, #45, #53.
**Pattern:** nhóm "nặng" nhất về kỹ thuật và rủi ro (malware, permissions). Phù hợp làm sau khi core đã vững.

### Nhóm 6 — Định vị sản phẩm & differentiator (7 ý tưởng)

_Focus: vì sao app này tồn tại, khác CleanMyMac chỗ nào._
Ý tưởng: #48, #54, #57, #59, #60, #64, #65.
**Pattern:** mang tính chiến lược hơn feature. `focus` + cleanup [#48] và open-source / không subscription [#64] định hình bản sắc: _"CleanMyMac minh bạch, không moi tiền, cho người dùng Mac kỹ thuật"_.

### Breakthrough Concepts

- **[#48] `focus` + cleanup** — `tiny` có sẵn `focus` timer, CleanMyMac không. Dọn nền trong lúc user focus = lợi thế độc nhất.
- **[#64] Open-source / không subscription** — đối lập trực diện mô hình CleanMyMac, là lý do tồn tại.
- **[#67] Dự báo trước khi đầy ổ** — chuyển app từ "chữa cháy" sang "phòng bệnh".
- **[#31/#33/#37] Cụm "giải thích"** — phantom space, heat-by-age, so sánh snapshot: nói _nguyên nhân_, không chỉ _tổng_.
- **[#68] Phô CLI** — mỗi action GUI hiện command `tiny ...`: vừa tạo niềm tin vừa dạy CLI.

### Quick Wins (gần engine có sẵn nhất)

- GUI cho `clean` với làn 3 màu [#7] + xem trước khi xoá [#8] — chỉ là lớp vỏ trên CLI đã chạy.
- Health Score [#14] + Smart Scan [#18] — gộp các lệnh `sys`/`scan`/`clean` đã tồn tại.
- Báo cáo sức khoẻ máy [#58] — tái dùng output `sys` + `clean --dry-run`.
- GUI cho `uninstall` [#3] — command đã đầy đủ, chỉ cần cửa sổ.

### Prioritization Results

**Scope v1 user chọn:** Nhóm 1 + 2 + 3 + 4 (gác Nhóm 5 Maintenance/Protection và Nhóm 6 Định vị sang sau).

⚠️ **Lưu ý:** 4 nhóm = ~48 ý tưởng — đó là **scope v1**, chưa phải MVP. Cần một lát cắt dọc nhỏ hơn để chạy được sớm. Đề xuất:

**MVP slice (tracer bullet)** — một luồng dọc xuyên 3 nhóm, bỏ qua phần nặng nhất (treemap):

1. Cửa sổ app + nút **Smart Scan** [#18] → gọi engine.
2. Hiện **Health Score 0–100** [#14] + danh sách dọn được theo **làn 3 màu** [#7].
3. **Xem trước khi xoá** [#8] → user xác nhận.
4. Chạy `clean`, đồ bỏ vào **quarantine/Trash có hoàn tác** [#9/#44/#50].
5. Mỗi action hiện command `tiny ...` tương đương [#68].

→ MVP chạm Nhóm 1 + 3, và Nhóm 4 ở mức tối thiểu. **Space Lens (Nhóm 2)** là khối nặng nhất (engine walker mới) → để **v1.1**.

## Action Planning

Bước kế tiếp user chọn: **Lên kế hoạch kỹ thuật**. Ba việc cần quyết trước khi viết code:

### Hành động 1 — Tách engine thành thư viện (prerequisite số 1)

- **Vì sao:** `tiny` hiện là binary CLI. GUI không thể "gọi" được logic nằm trong `src/commands/`. Cần tách phần lõi thành **library crate** (`src/lib.rs` + API sạch), để cả CLI lẫn GUI cùng dùng.
- **Tín hiệu tốt:** `scan` đã có `--json` — chứng tỏ logic tách được khỏi phần in ra màn hình.
- **Việc tuần đầu:** rà `src/commands/{sys,scan,clean,uninstall}` — tách "tính toán" khỏi "in ấn / prompt `dialoguer`".

### Hành động 2 — Chốt tech stack cho lớp GUI

| Lựa chọn                    | Ưu                                                     | Nhược                                     |
| --------------------------- | ------------------------------------------------------ | ----------------------------------------- |
| **SwiftUI native**          | Tích hợp macOS tốt nhất (menubar, notification, quyền) | Codebase Swift tách rời; gọi Rust qua FFI |
| **Tauri**                   | Backend Rust thuần, tái dùng engine trực tiếp          | UI là web; bundle nặng hơn                |
| **Rust-native (egui/iced)** | 1 ngôn ngữ duy nhất                                    | Khó đạt cảm giác "app macOS thật"         |

→ Cần một quyết định kiến trúc (ADR) riêng. Treemap [#15] và menubar [#2] là 2 phép thử khắc nghiệt cho lựa chọn này.

### Hành động 3 — Engine walker toàn ổ đĩa (mở khoá Nhóm 2)

- Space Lens cần quét **toàn ổ**, song song, nhanh — `scan` hiện chỉ chạm 3 folder.
- Là khối kỹ thuật lớn nhất và độc lập → tách thành milestone riêng cho v1.1.

## Session Summary and Insights

**Key Achievements:**

- **68 ý tưởng** cho macOS app của `tiny`, qua 2 technique (Analogical Thinking + SCAMPER).
- Gom thành **6 nhóm chủ đề**, chốt **scope v1 = 4 nhóm** + đề xuất **MVP slice** cụ thể.
- Phát hiện quan trọng: engine `tiny` đã có sẵn `uninstall` và duplicate finder — app không phải xây lại tính năng, mà là **khoác GUI + lấp 4 module thiếu**.

**Creative Breakthroughs:**

- Insight chủ đạo: ý tưởng giá trị nhất là loại **"giải thích nguyên nhân"**, không chỉ "hiển thị" — phantom space, heat-by-age, dự báo đầy ổ.
- Differentiator: `focus` + cleanup [#48] và open-source / không subscription [#64] → bản sắc _"CleanMyMac minh bạch, không moi tiền"_.

**Session Reflections:**

- Phiên pivot giữa chừng (TUI → macOS app) nhưng không mất ý tưởng vì pivot sớm.
- User thiên về momentum: quyết nhanh, thích AI chủ động bơm ý tưởng — facilitation nghiêng về generative.
- Phase 3 (What If Scenarios) bỏ qua — 68 ý tưởng đã đủ để hội tụ.

**Next Steps:**

1. Tạo ADR chốt tech stack GUI (Hành động 2).
2. Lên kế hoạch tách engine thành library crate (Hành động 1).
3. Lập kế hoạch kỹ thuật chi tiết cho MVP slice — dùng skill `planning` hoặc `bmad-create-architecture`.
