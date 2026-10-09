# Triển khai Tiny desktop

> Lịch sử (2026-10-09): bản Tauri/React mô tả dưới đây đã bị gỡ khỏi repo; desktop hiện là app SwiftUI trong `macos/`. Giữ lại làm hồ sơ.

Nguồn: `_bmad-output/planning-artifacts/architecture.md` tại commit `3c7e8a4` trên `origin/main` và MVP slice trong brainstorm ngày 2026-05-07.

## Phạm vi đã triển khai

- Cargo workspace: `tiny-core` (engine sync, typed errors, serde, callback tiến độ), `tiny` (clap/render/prompt) và `tiny-desktop` (Tauri).
- Giữ các lệnh CLI `sys`, `scan`, `clean`, `focus`, `uninstall` và toàn bộ registry 31 categories.
- N1: Smart Scan, Health Score giải thích được, xếp hạng theo mức rủi ro rồi dung lượng, picker ba làn, review từng đường dẫn.
- N2: walker song song bằng Rayon, treemap/sunburst, breadcrumb, màu theo nhánh/tuổi, ước tính phần dung lượng không nhìn thấy khi quét `/`.
- N3: preview lưu trong backend, xác nhận, fingerprint cả descendant, scope theo scan, loại trùng parent/child, Trash/quarantine và journal/restore.
- N4: system monitor, SQLite history (120 snapshots), chênh lệch tuần, dự báo tuyến tính, tray, scheduled scans, change watcher, rules chỉ thông báo, preferences.
- FDA onboarding, loading/error events, browser demo chỉ đọc, CI cho Linux/frontend/macOS.

## Các lựa chọn triển khai cần biết

- GUI không thực thi `trash`, `time-machine-local`, `docker`: các thao tác đó không thể hoàn tác. CLI hiện có vẫn giữ workflow riêng.
- Health Score là heuristic minh bạch; ranking không gọi AI/network.
- Quarantine fallback chỉ dùng rename trên cùng volume. Nếu không thực hiện được, giữ nguyên nguồn và báo lỗi.
- Quarantine ghi mốc 30 ngày nhưng giữ file ít nhất 30 ngày; chưa tự purge. Trash retention do Finder quản lý. Khôi phục Trash bằng Finder Put Back, quarantine bằng Tiny.
- Walker không đi theo symlink. Map giữ tối đa 200 entry lớn nhất mỗi thư mục, tree 4 tầng để hạn chế bộ nhớ; có thể click thư mục sâu để map tiếp. Quét có depth limit 64 và báo phần bị giới hạn/không đọc được. Logical bytes và allocated bytes khác nhau; APFS có thể làm ước tính phần không nhìn thấy thiếu chính xác.
- FDA không có public API xác nhận: probe thư mục được bảo vệ, trả `required`/`granted`/`unknown`, không tự cấp quyền.
- Scheduled scans chỉ chạy khi tiến trình Tiny còn sống; đóng cửa sổ sẽ ẩn xuống tray. `launchd` riêng, Developer ID/notarization vẫn thuộc phần deferred trong architecture.
- Các ý tưởng brainstorm không có acceptance criteria riêng (ví dụ command palette, Space Lens animation chuyên sâu) chưa được coi là tính năng hoàn tất.

## Verification

Kiểm tra đã chạy tại Linux: Rust workspace tests, clippy `-D warnings`, frontend TypeScript/Vite build, Vitest và smoke qua Chromium cho toàn bộ màn hình chính. Test tạm chỉ dùng file fixture để kiểm tra rename/restore; không dọn file người dùng.

Native Tauri/FDA/Trash/notification/autostart chưa chạy trên macOS trong phiên này. Workflow `.github/workflows/ci.yml` chạy compile/tests và build `.app` trên `macos-latest` khi push/PR. Cần chạy checklist macOS trong README để xác minh native integration và tạo `.dmg`.
