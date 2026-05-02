# 01 Architecture

## Mục tiêu kiến trúc

VKS ECMS là ứng dụng desktop Tauri offline. Lõi bắt buộc là OCR + rule extraction + SQLite FTS5; AI là lớp hỗ trợ, không phải điều kiện chạy tối thiểu.

## Phân tầng chuẩn

1. Presentation: UI hiển thị trạng thái, không giữ dữ liệu nghiệp vụ cốt lõi.
2. Application: điều phối workflow/state machine.
3. Domain: rule nghiệp vụ hồ sơ, citation, review.
4. Infrastructure: Rust commands, Python OCR worker, file system.
5. Persistence: SQLite + migration + audit + FTS5.

## Nguyên tắc bắt buộc

1. Evidence-first, citation-first.
2. Mọi kết luận AI phải truy vết về nguồn (document/page/quote).
3. Nếu thiếu citation hợp lệ: dừng xuất kết luận, chuyển review.
4. Không dùng browser runtime cho nghiệp vụ Tauri API.
