# 05 Backlog Implementation

## Ưu tiên 5 việc trước (P0)

### P0-01 Startup Environment Gate
- Mục tiêu: chặn app khi thiếu runtime tối thiểu.
- File code cần sửa:
  - `PhanMem/src-tauri/src/commands/mod.rs`
  - `PhanMem/src-tauri/src/main.rs`
  - `PhanMem/src/services/*` (service gọi self-check)
- Điều kiện hoàn thành:
  - Có command `run_startup_self_check` trả `pass|warning|fail`.
  - UI không vào Dashboard nếu `fail`.
- Test bắt buộc:
  - Thiếu `pdfium` -> fail cứng.
  - RAM giả lập dưới ngưỡng -> fail cứng.

### P0-02 File lifecycle
- Mục tiêu: bảo toàn file gốc và revision.
- File code cần sửa:
  - `PhanMem/src-tauri/src/commands/import_cmd.rs`
  - `PhanMem/src-tauri/src/commands/doc_cmd.rs`
  - `PhanMem/src-tauri/src/commands/export_cmd.rs`
  - migration SQL tại `PhanMem/src-tauri/migrations/`
- Điều kiện hoàn thành:
  - Có đủ `original/processing/reviewed/managed/exports`.
  - Không ghi đè `original`.
- Test bắt buộc:
  - Sửa OCR tạo revision mới.
  - Export chỉ nhận file `managed_ready`.

### P0-03 OCR block schema + Unicode
- Mục tiêu: chuẩn dữ liệu OCR làm nền search/citation/export.
- File code cần sửa:
  - `PhanMem/python/ocr/ocr_pipeline.py`
  - `PhanMem/src-tauri/src/commands/scan_cmd.rs`
  - `PhanMem/src-tauri/src/db/schema.rs`
- Điều kiện hoàn thành:
  - Lưu `raw_text`, `normalized_text`, `bbox`, `confidence`, `unicode_form`.
- Test bắt buộc:
  - Dữ liệu tiếng Việt còn dấu ở `raw_text`.
  - Search không dấu vẫn tìm được.

### P0-04 Citation validator
- Mục tiêu: chặn kết luận không có chứng cứ.
- File code cần sửa:
  - `PhanMem/src-tauri/src/commands/ai_cmd.rs`
  - `PhanMem/src/services/aiService.ts`
- Điều kiện hoàn thành:
  - Câu không có citation -> `review_required`/reject.
- Test bắt buộc:
  - Input thiếu citation bị chặn xuất chính thức.

### P0-05 Export package chuẩn
- Mục tiêu: bàn giao hồ sơ dùng ngay.
- File code cần sửa:
  - `PhanMem/src-tauri/src/commands/export_cmd.rs`
- Điều kiện hoàn thành:
  - Sinh đúng cấu trúc gói export chuẩn.
- Test bắt buộc:
  - Có đủ DOCX/HTML/citation/audit trong thư mục export.

## Phân phase thực hiện

### MVP (phải có)
1. Import hồ sơ
2. Lưu file gốc
3. OCR tiếng Việt
4. Search FTS5
5. Xem/sửa OCR
6. Export DOCX/HTML cơ bản
7. Log/audit

### Phase 2
1. AI tóm tắt phân tầng
2. Citation validator hoàn chỉnh
3. Timeline
4. Group summary
5. Model profile

### Phase 3
1. Handwriting assist
2. Adaptive scheduler nâng cao
3. Master dossier compiler
4. Full offline installer bundle hoàn chỉnh

## Rủi ro kỹ thuật

1. Embedded PaddleOCR trên Windows nặng, đóng gói phức tạp.
2. Qwen/Ollama offline không nên là bắt buộc ở bản đầu.
3. HTR tiếng Việt không cam kết chính xác cao.
4. Export DOCX chuẩn mẫu cần template engine, không dùng prompt thuần.
5. Citation theo bbox/page phải khóa schema ngay từ đầu.
