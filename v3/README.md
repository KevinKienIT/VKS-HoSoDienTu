# V3 — Bộ quy chuẩn triển khai VKS ECMS

Thư mục `v3/` là bộ tài liệu điều hướng triển khai chính thức cho toàn bộ dự án.

## ★ Điểm vào duy nhất cho mọi agent

1. **[`01_governance/01_AGENTS.md`](01_governance/01_AGENTS.md)** — Quy chuẩn bắt buộc toàn dự án (kiến trúc, code, UI/UX, DB, workflow)
2. **[`NEXT_ACTION.md`](NEXT_ACTION.md)** — Phase đang active + lộ trình
3. **[`TONGHOP.md`](TONGHOP.md)** — Bản điều phối trung tâm (code maps, UI maps, command playbook)

## Cấu trúc

- [`01_governance/`](01_governance/) — ★ LUẬT TỔNG cho mọi agent
- [`03_specs/`](03_specs/) — Đặc tả sản phẩm ([index](03_specs/00_SPEC_INDEX.md))
- [`05_logs/`](05_logs/) — Log điều phối theo ngày/phase
- [`06_reference_samples/`](06_reference_samples/) — Mẫu DOCX/HTML đầu ra
- [`07_external_refs/`](07_external_refs/) — Tham khảo ngoài
- [`08_execution_phases/`](08_execution_phases/) — Phase thực thi P0-P6
- [`standards/`](standards/) — Luật kỹ thuật cứng (architecture, runtime, OCR, lifecycle, backlog)

## Quy tắc vệ sinh workspace

1. Không tạo script rời rạc ngoài nhu cầu runtime cốt lõi.
2. Tài liệu kế hoạch cũ không dùng trực tiếp cho triển khai phải loại bỏ.
3. Backlog kỹ thuật chuẩn chỉ đọc tại [`05_backlog_implementation.md`](standards/05_backlog_implementation.md).
4. Mọi agent phải đọc `01_governance/01_AGENTS.md` trước khi làm bất cứ gì.
