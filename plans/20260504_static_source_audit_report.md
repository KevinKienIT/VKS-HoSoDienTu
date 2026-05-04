# Báo cáo audit tĩnh source code VKS ECMS

Phạm vi: audit tĩnh, không chạy lệnh, tập trung vào version, độ hoàn thiện, khả năng thực thi đọc file, và bugs xác định trực tiếp từ source.

## 1. Kết luận nhanh

- Version hiện khai báo: `1.0.0` ở `PhanMem/package.json` và `PhanMem/src-tauri/Cargo.toml`.
- Bản chất phần mềm: Tauri Desktop, không phải web app.
- Độ hoàn thiện thực tế: chưa đạt mức release ổn định dù version đang là `1.0.0`; phù hợp hơn là MVP/P0 foundation đang dang dở.
- Khả năng thực thi đọc file: có nền tảng đọc PDF/image qua Python OCR và Rust invoke, nhưng còn blockers về schema/lifecycle và một số hỗ trợ file chưa hoàn chỉnh.
- Mức rủi ro tổng thể: cao nếu source workspace vẫn có lệch module path hoặc phase P0 chưa pass build gate.

## 2. Đánh giá version và độ hoàn thiện

### Version khai báo

| Thành phần       | Evidence                                               | Nhận xét                                                   |
| ---------------- | ------------------------------------------------------ | ---------------------------------------------------------- |
| Frontend package | `PhanMem/package.json`                                 | `version = 1.0.0`.                                         |
| Rust package     | `PhanMem/src-tauri/Cargo.toml`                         | `version = 1.0.0`.                                         |
| Runtime nền      | `PhanMem/src-tauri/Cargo.toml`, `PhanMem/package.json` | Tauri v2.                                                  |
| React stack      | `PhanMem/package.json`                                 | React, React Router, Zustand.                              |
| Python OCR       | `PhanMem/python/requirements.txt`                      | RapidOCR, ONNXRuntime, PaddleOCR, PyMuPDF, OpenCV, Ollama. |

### Độ hoàn thiện theo đặc tả dự án

Các chức năng nền đã có code: import, case, document, OCR, search, review, export PDF, AI workspace. Tuy nhiên P0 vẫn chưa xong theo `v3/NEXT_ACTION.md`, gồm Startup Gate, file lifecycle, OCR Unicode schema, citation validator, export package chuẩn.

Đánh giá thực tế:

- UI shell và routes đã có trong `PhanMem/src/App.tsx`.
- App chỉ check Tauri runtime bằng `__TAURI_INTERNALS__`, chưa có Startup Environment Gate như P0-01.
- Storage mới có `originals`, `processed`, `exports`; chưa có đầy đủ per-case `processing`, `reviewed`, `managed`, `exports` theo P0-02.
- DB migration hiện mới embed tới migration 006; chưa có migration 007 lifecycle hoặc 008 normalized OCR như kế hoạch P0.

Kết luận version: version `1.0.0` là quá cao so với trạng thái code; nên xem là pre-release/MVP P0 chưa hoàn thiện.

## 3. Khả năng thực thi đọc file

### Điểm đã có

- PDF đọc được bằng PyMuPDF qua `fitz.open()` trong `PhanMem/python/phantich/xuly_hoso.py`.
- PDF render sang ảnh qua `page.get_pixmap()` và module trích trang trong `PhanMem/python/ocr/pdf_to_images.py`.
- Folder PDF có thể quét đệ quy bằng `extract_folder()`.
- OCR ảnh có pipeline khá đầy đủ trong `PhanMem/python/ocr/ocr_pipeline.py`, gồm RapidOCR, PaddleOCR fallback, layout blocks, stamp/signature/handwriting heuristics.
- Rust có whitelist file import gồm PDF, ảnh và Office extensions trong `PhanMem/src-tauri/src/commands/doc_cmd.rs` hoặc module tương ứng.
- Rust gọi Python có validate script/lang và parse JSON trong command tài liệu.

### Hạn chế thực thi đọc file

- Office files như DOC/DOCX/XLSX/PPTX có thể được cho phép import, nhưng OCR source thực tế chỉ xử lý PDF/image; chưa có parser Office đầy đủ.
- Python `requirements.txt` chưa liệt kê `python-docx` dù pipeline tạo DOCX bằng `from docx import Document`.
- OCR block có hàm normalize nhưng block output chưa đưa `normalized_text`/`unicode_form` vào từng block như P0-03 yêu cầu.

## 4. Bugs chính xác đã xác định từ source

### BUG-001 — Critical: Rust module wiring cần đồng bộ và xác minh build

Evidence:

- Governance/spec từng ghi đổi tên module sang `lenh_*`, trong khi workspace hiện cũng có các file command tên cũ như `ai_cmd.rs`, `case_cmd.rs`, `doc_cmd.rs`, v.v.
- `PhanMem/src-tauri/src/commands/mod.rs` hiện khai báo module cũ: `ai_cmd`, `case_cmd`, `catalog_cmd`, `doc_cmd`, `export_cmd`, `import_cmd`, `module_cmd`, `review_cmd`, `scan_cmd`, `search_cmd`.
- Một số tab/log trước đó từng thể hiện file `lenh_*`; cần tránh trạng thái nửa đổi tên nửa tên cũ.

Tác động:

- Nếu workspace còn file `lenh_*` nhưng `mod.rs`/`main.rs` gọi tên cũ, Rust sẽ fail compile.
- Nếu đã quay về tên cũ, cần cập nhật governance/log hoặc tránh tiếp tục sửa theo hướng `lenh_*` gây regression.

Hướng sửa:

- Chọn một chuẩn duy nhất cho module Rust.
- Nếu giữ tên cũ đang có trong workspace, đảm bảo toàn bộ `main.rs`, `mod.rs`, và `use crate::commands::*` đều dùng tên cũ.
- Nếu đổi sang `lenh_*`, sửa đồng bộ toàn bộ references.
- Sau sửa phải chạy build gate được phép: `cd PhanMem && npm run tauri:build`.

### BUG-002 — High: Startup Environment Gate chưa được implement

Evidence:

- `PhanMem/src-tauri/src/main.rs` setup chỉ tạo data dir, storage dir, init DB.
- `PhanMem/src/App.tsx` chỉ check Tauri runtime bằng `__TAURI_INTERNALS__`.
- Không có command `run_startup_self_check` trong invoke handler.
- `sysinfo` chưa có trong `PhanMem/src-tauri/Cargo.toml` dependencies.

Tác động:

- App có thể vào Dashboard dù thiếu Python, OCR model, disk, quyền ghi storage, hoặc runtime cơ bản.

Hướng sửa:

- Thêm command startup self-check, service frontend và gate page trước AppShell.

### BUG-003 — High: File lifecycle chưa hoàn thiện, có nguy cơ sai trạng thái nghiệp vụ

Evidence:

- Storage chỉ tạo 3 thư mục: originals, processed, exports.
- Chưa có per-case reviewed/managed/processing helpers theo P0-02.
- Schema constants có file lifecycle nhưng migration thực tế chưa có cột `file_status`, `original_path`, `managed_path`, `revision_no`.
- `update_document_status()` vẫn dùng status legacy `pending/processed/reviewed/error`.

Tác động:

- Không đạt nguyên tắc original bất biến đầy đủ và revision lifecycle.
- Export/review không thể enforce `managed_ready` đúng spec.

Hướng sửa:

- Thêm migration lifecycle và cập nhật import/OCR/review/export theo status mới.

### BUG-004 — High: OCR normalized_text chưa được lưu/index đúng spec

Evidence:

- Migration `page_layout_blocks` chỉ có cột `text`, không có `normalized_text`.
- FTS layout block chỉ index `text`, không index `normalized_text`.
- Python có normalize helper nhưng block object chưa set `normalized_text` và `unicode_form`.
- Rust insert layout block không đọc/ghi `normalized_text`.

Tác động:

- Search không dấu tiếng Việt không đảm bảo đúng như P0-03.
- Citation/search trên OCR layout có thể kém chính xác.

Hướng sửa:

- Thêm migration `normalized_text`, update Python output, update Rust insert, rebuild FTS.

### BUG-005 — Medium: `python-docx` bị thiếu trong requirements nhưng code import trực tiếp

Evidence:

- `PhanMem/python/requirements.txt` không có `python-docx`.
- `PhanMem/python/phantich/xuly_hoso.py` import `from docx import Document`.

Tác động:

- Chạy pipeline tổng hợp hồ sơ có thể lỗi `ModuleNotFoundError: No module named docx` ở môi trường sạch.

Hướng sửa:

- Thêm `python-docx` vào `requirements.txt` hoặc tách export DOCX thành optional có fallback.

### BUG-006 — Medium: Import cho phép Office nhưng chưa có parser nội dung Office

Evidence:

- Import whitelist cho phép `doc`, `docx`, `xls`, `xlsx`, `ppt`, `pptx`, `rtf`.
- OCR source thực tế chỉ xử lý PDF/image.
- P2 parser DOCX/HTML vẫn chưa bắt đầu theo `v3/NEXT_ACTION.md`.

Tác động:

- Người dùng có thể import Office nhưng app chưa đọc/parse nội dung, dễ hiểu nhầm là đã hỗ trợ phân tích đầy đủ.

Hướng sửa:

- Tạm giới hạn import UI với trạng thái `unsupported_for_ocr`, hoặc implement parser Phase 2.

### BUG-007 — Medium: Citation validator AI chưa enforce

Evidence:

- P0-04 yêu cầu AI output cần citation warning.
- AI commands đã có nhưng chưa enforce citation validator theo backlog P0.

Tác động:

- AI có thể trả kết luận không đủ nguồn, trái nguyên tắc evidence-first.

Hướng sửa:

- Validate sources/document/page/quote trước khi render kết quả AI, thêm badge warning/review_required.

### BUG-008 — Low/Medium: Version `1.0.0` gây sai kỳ vọng release

Evidence:

- Version đang là `1.0.0` ở frontend và backend.
- P0 còn TODO.

Tác động:

- Dễ nhầm là production-ready trong khi foundation chưa xong.

Hướng sửa:

- Dùng version pre-release như `0.1.0-p0` hoặc `0.2.0-mvp` cho đến khi pass P0/P2/P3 gates.

## 5. Ưu tiên xử lý đề xuất

1. Đồng bộ và xác minh Rust command module paths.
2. Chạy build gate bằng lệnh dự án cho phép sau khi sửa: `cd PhanMem && npm run tauri:build`.
3. Implement P0-03 OCR normalized schema vì đang gần hoàn thiện nhưng thiếu phần DB/FTS.
4. Implement P0-01 Startup Environment Gate để chặn lỗi runtime sớm.
5. Implement P0-02 file lifecycle migration/storage/status.
6. Thêm `python-docx` vào Python requirements nếu tiếp tục dùng pipeline DOCX.
7. Tách rõ file import được nhưng chưa phân tích được đối với Office documents.

## 6. Todo triển khai cho mode Code

- [ ] Đồng bộ tên Rust command modules trong `PhanMem/src-tauri/src/commands/mod.rs`, `PhanMem/src-tauri/src/main.rs`, và toàn bộ `use crate::commands::*`.
- [ ] Kiểm tra lại build sau khi đồng bộ module path bằng lệnh Tauri build được phép.
- [ ] Thêm migration OCR `normalized_text` và update `PhanMem/python/ocr/ocr_pipeline.py` cùng command insert layout block.
- [ ] Thêm Startup Environment Gate theo P0-01.
- [ ] Thêm file lifecycle migration/storage helpers theo P0-02.
- [ ] Bổ sung `python-docx` vào `PhanMem/python/requirements.txt` hoặc thêm fallback khi thiếu dependency.

Audit tĩnh hoàn tất theo phạm vi đã chọn: không chạy lệnh, không sửa source phần mềm, chỉ lưu báo cáo để bắt đầu sửa.
