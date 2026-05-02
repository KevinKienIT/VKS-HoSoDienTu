# V3 — Bộ quy chuẩn triển khai tinh gọn

Thư mục [`v3/`](v3) là bộ tài liệu điều hướng triển khai chính thức, tách từ [`tonghop.md`](tonghop.md) theo hướng ngắn gọn, ưu tiên backlog thực thi.

Điểm vào chính khi điều phối task mới là [`TONGHOP.md`](TONGHOP.md). File này gom thứ tự đọc lệnh, phase active, maps giao diện, maps code, câu lệnh minh họa và checklist đối chiếu trước khi sửa source.

## Cấu trúc

- [`TONGHOP.md`](TONGHOP.md): bản điều phối trung tâm cho toàn bộ V3
- [`NEXT_ACTION.md`](NEXT_ACTION.md): việc tiếp theo và phase đang active
- [`agent/`](v3/agent): điều hướng agent + bộ nhớ vận hành
- [`standards/01_architecture.md`](v3/standards/01_architecture.md)
- [`standards/02_runtime_offline_requirements.md`](v3/standards/02_runtime_offline_requirements.md)
- [`standards/03_pipeline_ocr_ai.md`](v3/standards/03_pipeline_ocr_ai.md)
- [`standards/04_file_lifecycle_export.md`](v3/standards/04_file_lifecycle_export.md)
- [`standards/05_backlog_implementation.md`](v3/standards/05_backlog_implementation.md)

## Quy tắc vệ sinh workspace

1. Không tạo script rời rạc ngoài nhu cầu runtime cốt lõi.
2. Tài liệu kế hoạch cũ không dùng trực tiếp cho triển khai phải loại bỏ.
3. Backlog kỹ thuật chuẩn chỉ đọc tại [`05_backlog_implementation.md`](v3/standards/05_backlog_implementation.md).
