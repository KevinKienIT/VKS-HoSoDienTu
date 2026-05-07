# FND-AllPDF Extraction Plan

## 1. Phạm vi và nguồn đã đọc

Mục tiêu của FND-AllPDF là bảo đảm mọi PDF được import đều có đủ PNG theo số trang thực tế trước khi viewer/OCR/export sử dụng dữ liệu đó.

Nguồn đối chiếu:

- `v4/07_REPORTS/FND_004_PAGE_IMAGE_EXTRACTION_RUNTIME_RESULT.md`
- `v4/07_REPORTS/FND_005_OCR_TERMINAL_RUNTIME_RESULT.md`
- `v4/07_REPORTS/FND_006_VIEWER_PNG_FIRST_FIX.md`
- `roo_work/allpdf_extraction_runtime_report.md` từ branch `agent/roo/fnd-allpdf-extraction-runtime`
- `v4/07_REPORTS/FND_AllPDF_PageExtraction.md` từ branch `agent/sonnet/fnd-allpdf-extraction-test`
- `PhanMem/python/ocr/pdf_to_images.py`
- `PhanMem/src-tauri/src/commands/import_cmd.rs`
- `PhanMem/src-tauri/src/commands/doc_cmd.rs`
- `PhanMem/src-tauri/migrations/001_init_schema.sql`, `009_page_ocr_scheduler.sql`, `014_page_image_architecture.sql`, `015_page_quality_classifier.sql`

Grapuco không có CLI khả dụng trong session này, nên đã dùng fallback code graph bằng `rg`:

- `import_cmd.rs`: `try_count_pdf_pages`, `count_pdf_pages`, `run_page_image_extraction`, `insert_page_image_row`, `insert_page_row`, `import_one_file_for_case`, các nhánh `import_folder`/`import_multiple_files`, và các event `PAGE_IMAGE_EXTRACT_*`.
- `doc_cmd.rs`: `run_python_json_for_import`, `run_python_json_with_timeout`, `run_ocr_for_document_scope_with_conn`, `extract_pdf_pages_for_document_with_conn`, `upsert_ocr_text`, `read_image_base64`.
- Frontend PNG-first: `TrinhXemTaiLieu.tsx`, `AnhThuNho.tsx`, `documentService.getDocument`, `readImageBase64`, trạng thái `PAGE_IMAGE_MISSING`.

## 2. Luồng hiện tại

Luồng đúng theo kiến trúc V4:

```text
Import folder/files
  -> đếm page_count từ PDF
  -> copy PDF gốc vào originals/<case_code>/<date>/
  -> extract PDF thành PNG 300 DPI
  -> lưu pages.image_path, pages.thumbnail_path, pages.extract_status='extracted'
  -> OCR đọc từ stored PNG
  -> viewer/export dùng PNG, không render PDF bytes
```

Chi tiết code hiện tại:

- `import_cmd.rs` đếm `page_count` bằng `lopdf` (`try_count_pdf_pages` hoặc `count_pdf_pages`).
- `storage::copy_to_originals()` copy file nguồn vào vùng `originals`.
- `run_page_image_extraction()` gọi Python qua `doc_cmd::run_python_json_for_import()` với module `ocr.pdf_to_images`.
- `pdf_to_images.py` dùng PyMuPDF, render từng page thành `page_###.png`, tạo thumbnail nếu có `--thumbnail-dir`, trả JSON `pages[]`.
- `insert_page_image_row()` ghi `pages.image_path`, `thumbnail_path`, `source_pdf_path`, `source_page_number`, `current_order`, `extract_status='extracted'`, `ocr_status='queued'`.
- `doc_cmd.rs` ưu tiên OCR từ `pages.image_path` khi đủ stored PNG; nếu thiếu, vẫn có nhánh OCR tự extract tạm từ PDF.
- `TrinhXemTaiLieu.tsx` lấy `page_count` từ DB bằng `documentService.getDocument(documentId)`, sau đó tải PNG từng trang qua `getPageOcr`/`readImageBase64`.
- Nếu DB báo có trang nhưng không có `image_path`, viewer hiện `PAGE_IMAGE_MISSING` và không render PDF fallback.

FND-004/FND-005 đã chứng minh fixture `test_doc_1.pdf` chạy được: 2/2 trang có PNG, `extract_status='extracted'`, OCR terminal `review_pending`, `ocr_source=stored_page_image`.

## 3. Điểm có thể gây thiếu trang

1. `run_page_image_extraction()` chỉ lỗi khi `count == 0`. Nếu PDF có 20 trang nhưng Python chỉ trả 19 PNG, import vẫn có thể ghi `PAGE_IMAGE_EXTRACT_DONE`.

2. Chưa có kiểm tra bắt buộc sau extract:

```sql
SELECT documents.page_count
FROM documents
WHERE document_id = ?

SELECT COUNT(*)
FROM pages
WHERE document_id = ?

SELECT COUNT(*)
FROM pages
WHERE document_id = ?
  AND extract_status = 'extracted'
  AND COALESCE(image_path, '') <> ''
```

Ba số này phải bằng nhau cho PDF. Hiện chưa có gate cứng sau mỗi import.

3. `pdf_to_images.py` render toàn bộ PDF trong một tiến trình ở 300 DPI. PDF lớn/nhiều trang có thể vượt timeout 600s hoặc áp lực RAM. Khi fail, Rust tạo page placeholder `extract_status='pending'`, `image_path=NULL`, nhưng không retry theo từng page.

4. Nhánh OCR fallback trong `run_ocr_for_document_scope_with_conn()` có thể tự extract PDF nếu stored PNG chưa đủ. Nhánh này đưa `image_path` vào `upsert_ocr_text()`, nhưng `upsert_ocr_text()` không set `extract_status='extracted'`, làm DB không đạt invariant PNG-first.

5. Các entry point chưa đồng nhất:

- `import_one_file_for_case()` và một số nhánh `import_folder()` đã gọi `run_page_image_extraction()`.
- `import_multiple_files()` vẫn có nhánh chỉ insert page placeholder theo `page_count`.
- `scan_cmd.rs` import từ Ricoh inbox cũng tạo page rows trước, chưa dùng cùng helper extract/validate.

6. Sort file đang là lexicographic (`1.pdf`, `10.pdf`, `2.pdf`). Khi import hồ sơ đánh số tự nhiên, thứ tự tài liệu/trang có thể sai với kỳ vọng nghiệp vụ.

7. Log lỗi hiện chỉ ở governed event mức document. Thiếu `extract_attempts`, `extract_error`, `extracted_at` ở mức page/document nên debug timeout/partial extract khó.

## 4. Invariant bắt buộc

Với mỗi document PDF:

```text
expected = documents.expected_page_count hoặc documents.page_count
page_rows = COUNT(pages WHERE document_id=?)
png_rows = COUNT(pages WHERE document_id=? AND extract_status='extracted' AND image_path exists on disk and size > 0)

PASS khi expected > 0 AND page_rows == expected AND png_rows == expected
```

Nếu không đạt:

- không ghi `PAGE_IMAGE_EXTRACT_DONE`;
- ghi `PAGE_IMAGE_EXTRACT_INCOMPLETE` hoặc `PAGE_IMAGE_EXTRACT_FAILED`;
- set page thiếu sang `extract_status='error'`;
- set document sang trạng thái cần review/lỗi extract;
- không cho viewer/export fallback về PDF bytes.

## 5. Migration/struct changes đề xuất

Có thể dùng `documents.page_count` làm expected count ngắn hạn. Tuy nhiên nên thêm cột để audit rõ ràng:

Migration đề xuất: `016_allpdf_extraction_integrity.sql`

```sql
ALTER TABLE documents ADD COLUMN expected_page_count INTEGER;
ALTER TABLE documents ADD COLUMN page_image_status TEXT NOT NULL DEFAULT 'pending'
  CHECK (page_image_status IN ('pending','extracting','complete','partial','error'));
ALTER TABLE documents ADD COLUMN page_image_error TEXT;
ALTER TABLE documents ADD COLUMN page_image_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE documents ADD COLUMN last_page_extract_at TEXT;

ALTER TABLE pages ADD COLUMN extract_error TEXT;
ALTER TABLE pages ADD COLUMN extract_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE pages ADD COLUMN extracted_at TEXT;

CREATE INDEX IF NOT EXISTS idx_documents_page_image_status
  ON documents(page_image_status);
CREATE INDEX IF NOT EXISTS idx_pages_doc_extract
  ON pages(document_id, extract_status, page_index);
```

Struct changes:

- `ImportedFile`: thêm `expected_page_count`, `extracted_page_count`, `page_image_status`, `warnings`.
- Thêm internal struct `PageExtractionValidation`:

```rust
struct PageExtractionValidation {
    expected_pages: i32,
    page_rows: i64,
    extracted_png_rows: i64,
    missing_page_numbers: Vec<i32>,
    bad_image_paths: Vec<String>,
    status: String, // complete | partial | error
}
```

## 6. Sửa luồng import/extract

### 6.1 Tách helper chung

Tạo helper dùng chung trong `import_cmd.rs` hoặc module riêng:

- `ensure_page_placeholders(conn, document_id, expected_pages)`
- `upsert_extracted_page_image(conn, document_id, page_num, image_path, thumbnail_path, source_pdf_path)`
- `validate_document_page_images(conn, document_id)`
- `mark_page_extraction_result(conn, document_id, validation)`

Không dùng insert thuần cho retry. Dùng upsert dựa trên unique index sẵn có `idx_page_doc_idx(document_id, page_index)`.

### 6.2 Extract với kiểm tra page_count

Luồng mới trong `run_page_image_extraction()`:

1. Load expected count từ `documents.expected_page_count` hoặc `documents.page_count`.
2. Set document `page_image_status='extracting'`.
3. Đảm bảo có page rows 1..expected với `extract_status='extracting'`.
4. Chạy full-document extract nếu file nhỏ/vừa.
5. Kiểm tra JSON:
   - `pages.len()` phải bằng expected.
   - mỗi `page_num` phải nằm trong 1..expected.
   - file PNG và thumbnail tồn tại, size > 0.
6. Upsert từng page.
7. Chạy `validate_document_page_images()`.
8. Nếu thiếu trang, retry missing pages.
9. Chỉ ghi `PAGE_IMAGE_EXTRACT_DONE` khi validation complete.

### 6.3 Retry/fallback cho PDF lớn hoặc timeout

Chiến lược fallback:

- Nếu full-document extract timeout hoặc thiếu trang, retry theo từng page bằng option `--page N` hiện đã có trong `pdf_to_images.py`.
- Với PDF lớn, bỏ qua full-document extract và chạy per-page ngay từ đầu nếu:
  - `expected_page_count > 50`, hoặc
  - file size > 100 MB, hoặc
  - lần import trước đó timeout.
- Mỗi page retry tối đa 2 lần:
  - lần 1: 300 DPI;
  - lần 2: 200 DPI, ghi warning `LOWER_DPI_RETRY`.
- Nếu vẫn fail, set page `extract_status='error'`, lưu `extract_error`, ghi event `PAGE_IMAGE_EXTRACT_PAGE_FAILED`.

Python script cần bổ sung:

- trả `source_page_count = len(doc)` riêng với `extracted_pages = len(pages)`;
- trả `duration_ms`, `dpi`, `errors[]`;
- bắt lỗi theo từng page thay vì abort toàn bộ document khi một page lỗi;
- giữ `--page` để Rust retry page đơn.

### 6.4 Đồng nhất entry points

Các nơi sau phải gọi cùng helper extract/validate:

- `import_one_file_for_case()`
- `import_folder()`
- `import_multiple_files()`
- `debug_import_test_path()`
- scan/Ricoh import trong `scan_cmd.rs`

Không entry point nào được chỉ tạo placeholder pages cho PDF rồi kết thúc import như thành công.

### 6.5 Sửa OCR fallback

Trong `doc_cmd.rs`:

- Trước OCR, gọi validation stored PNG.
- Nếu thiếu PNG, gọi helper extract/validate chung, không dùng nhánh extract tạm không cập nhật DB.
- Nếu `upsert_ocr_text()` nhận `image_path=Some(...)` và file tồn tại, update thêm:

```sql
extract_status = 'extracted',
extracted_at = COALESCE(extracted_at, now)
```

OCR chỉ được chạy từ stored PNG sau khi DB đã phản ánh đúng `pages.image_path`/`extract_status`.

## 7. Natural sort

Áp dụng cho import folder và script verify.

Logic:

- Tách chuỗi filename/path thành token số và token chữ.
- Token số so sánh bằng giá trị integer.
- Nếu giá trị integer bằng nhau, so tiếp độ dài/token gốc để ổn định.
- Token chữ so sánh case-insensitive.
- Giữ relative directory trước filename để thứ tự thư mục ổn định.

Ví dụ:

```text
1.pdf, 2.pdf, 3.pdf, 10.pdf, 11.pdf
```

Không dùng:

```text
1.pdf, 10.pdf, 11.pdf, 2.pdf, 3.pdf
```

## 8. Script tự động nhập và kiểm tra toàn bộ

Tạo `PhanMem/scripts/verify_allpdf_extraction.py` hoặc nâng cấp bản từ branch `agent/sonnet/fnd-allpdf-extraction-test`.

Chức năng:

- nhận `--db`, `--case-id`, `--folder`, `--json-out`;
- natural-sort danh sách PDF trong folder;
- tìm documents theo `case_id` hoặc `original_filename`/hash;
- so sánh `expected_page_count/page_count` với số rows trong `pages`;
- kiểm tra `extract_status='extracted'`;
- kiểm tra `image_path` tồn tại và size > 0;
- xuất summary theo document và exit code 1 nếu có thiếu trang.

Tạo thêm script điều phối dev: `PhanMem/scripts/run_allpdf_import_check.ps1`.

Luồng:

1. Chuẩn bị folder fixture, ví dụ `PhanMem/test_pdfs/allpdf_runtime`.
2. Import folder qua Tauri dev command/debug path hiện có (`debug_import_test_path`) hoặc UI automation nội bộ.
3. Chờ import job hoàn tất.
4. Chạy `verify_allpdf_extraction.py`.
5. Ghi artifact:
   - `roo_work/allpdf_extraction_verify.log`
   - `roo_work/allpdf_extraction_summary.json`
   - `roo_work/allpdf_extraction_detailed_pages.json`

## 9. Log lỗi và cảnh báo cần chuẩn hóa

Mục tiêu log là sau này nhìn DB/event là biết thiếu trang do đâu, không cần đoán từ UI.

Governed events cần thêm/chuẩn hóa:

- `PAGE_IMAGE_EXTRACT_STARTED`: có `document_id`, `source_pdf_path`, `expected_page_count`, `file_size`, `strategy` (`full_document` hoặc `per_page`), `dpi`.
- `PAGE_IMAGE_EXTRACT_INCOMPLETE`: có `expected_page_count`, `extracted_page_count`, `page_rows`, `missing_page_numbers`, `bad_image_paths`, `retry_planned=true/false`.
- `PAGE_IMAGE_EXTRACT_RETRY_STARTED`: có `retry_attempt`, `strategy`, `missing_page_numbers`, `dpi`.
- `PAGE_IMAGE_EXTRACT_PAGE_FAILED`: có `page_number`, `retry_attempt`, `error_code`, `error_message`.
- `PAGE_IMAGE_EXTRACT_DONE`: chỉ ghi khi validation complete.
- `PAGE_IMAGE_EXTRACT_FAILED`: có thông điệp nghiệp vụ rõ, ví dụ: `Lỗi import PDF: thiếu trang do timeout khi render PNG; expected=20, extracted=17, missing=[18,19,20]`.

Thông điệp lỗi đề xuất:

```text
Lỗi import PDF: thiếu trang do python extractor timeout. File vẫn được giữ nguyên trong originals; chưa được phép viewer/export bằng PDF fallback. Hãy retry extract các trang thiếu.
Lỗi import PDF: thiếu trang do PNG không tồn tại trên ổ đĩa sau extract. Kiểm tra quyền ghi thư mục processed/ocr_pages và retry.
Lỗi import PDF: số trang trong JSON extractor khác documents.page_count. Kiểm tra PDF hỏng/encrypted hoặc lỗi PyMuPDF.
Lỗi import PDF: trang N render thất bại sau 2 lần retry. Tài liệu cần review thủ công, không đánh dấu extract complete.
```

DB fields nên lưu cùng event:

- `documents.page_image_error`: thông điệp người vận hành đọc được.
- `pages.extract_error`: lỗi kỹ thuật theo từng trang.
- `pages.extract_attempts`: số lần thử render page.
- `documents.page_image_attempts`: số vòng extract/retry cấp document.

## 10. Ghi chú lỗi react-pdf/PDF fallback

Trong runtime FND-AllPDF, Tauri dev từng báo lỗi dependency:

- `react-pdf` unresolved từ `PhanMem/src/modules/phantichtailieu/AnhThuNho.tsx`
- `react-pdf` unresolved từ `PhanMem/src/modules/phantichtailieu/TrinhXemTaiLieu.tsx`
- `pdfjs-dist/build/pdf.worker.min.mjs?url` và CSS import liên quan cũng unresolved

FND-006 fix report ghi nhận thêm các lỗi runtime cũ:

- `Cannot read properties of null (reading 'getAnnotations')`
- `Canvas detached`
- PDF.js Web Worker xung đột với Tauri WebView
- bundle bị kéo theo `react-pdf`/`pdfjs-dist`, tăng rủi ro memory leak

Các fallback cũ đã bị loại bỏ theo hướng PNG-first:

- `PhanMem/src/components/PdfPageThumbnail.tsx` dùng `react-pdf`/`pdfjs-dist`
- `PhanMem/src/components/DocumentViewer.tsx` render PDF bytes trực tiếp
- `PhanMem/src/modules/hosovuan/ChiTietVuAnPage.tsx` import thumbnail PDF cũ, cần dùng `PdfPageThumbnail` export từ `AnhThuNho.tsx`
- `TrinhXemTaiLieu.tsx` không được suy luận số trang từ số ảnh render được; phải lấy `page_count` từ DB

Commit sửa từ Sonnet:

- `65c4920 [FND-006] Implement viewer PNG-first`

Kết quả fix cần được giữ:

- mọi import `react-pdf` trong runtime source bị xóa;
- `Document`, `Page`, `pdfjs` không còn trong viewer/thumbnail runtime;
- viewer dùng `readImageBase64` để đọc PNG cố định;
- khi thiếu PNG, viewer hiển thị `PAGE_IMAGE_MISSING: ... không render PDF fallback`;
- `npx tsc --noEmit`, `cargo test`, và `npm run tauri:dev` không load PDF.js worker.

Yêu cầu cho FND-AllPDF:

- loại bỏ hoàn toàn fallback render PDF trong production viewer;
- viewer chỉ đọc `pages.image_path`/`thumbnail_path`;
- export chỉ dùng ordered PNG pages;
- `rg "react-pdf|pdfjs-dist" PhanMem/src PhanMem/package.json` phải không còn dependency/import runtime. Nếu chỉ còn `overrides.pdfjs-dist` trong `package.json`, cần xóa nốt khi không còn dependency bắc cầu cần override.

Điểm chưa chắc chắn, chưa commit code:

- `PhanMem/package.json` hiện vẫn có `overrides.pdfjs-dist`. Cần kiểm tra dependency tree trước khi xóa; nếu không có package nào cần override này thì xóa để tránh hiểu nhầm rằng PDF.js vẫn được chấp nhận.
- Các thư mục backup như `src/components/_backup_pages_v2` có thể còn code viewer cũ hoặc reference liên quan PDF. Nếu backup không nằm trong route runtime thì không chặn FND-AllPDF, nhưng nên loại khỏi source scan/runtime bundle hoặc ghi rõ là archive.

## 11. Test plan

### Static checks

- `cargo check -j 1` trong `PhanMem/src-tauri`
- `npm exec tsc -- --noEmit` trong `PhanMem`

### Runtime checks

Chỉ test app bằng:

```powershell
cd PhanMem && npm run tauri:dev
```

Test cases:

1. Single PDF 2 trang (`test_doc_1.pdf`): expected 2, pages 2, PNG 2, OCR dùng `stored_page_image`.
2. Folder `1.pdf,2.pdf,10.pdf`: import order phải là natural-sort.
3. Folder nhiều PDF `PhanMem/test_pdfs/allpdf_runtime`: tất cả PDF pass checker.
4. PDF lớn/nhiều trang: không timeout full job; nếu full extract timeout thì per-page retry hoàn tất hoặc báo lỗi cụ thể trang thiếu.
5. PDF corrupt/password/không đọc được: import không tạo trạng thái thành công giả; document/page có error rõ.
6. Mất PNG sau import: checker phát hiện `image_path` bad/missing, retry extract lại.
7. OCR sau import: không chạy OCR nếu page extraction chưa complete, trừ khi flow được đánh dấu partial + review_required rõ ràng.
8. Viewer: trang thiếu PNG hiển thị lỗi extract, không render PDF fallback.
9. Export: reject document chưa đủ PNG hoặc ghi warning cứng, không merge original PDF thay thế.

Pass criteria:

```text
For every imported PDF:
documents.expected_page_count/page_count == COUNT(pages)
documents.expected_page_count/page_count == COUNT(extracted PNG pages)
all PNG paths exist and size > 0
no production PDF renderer dependency/import remains
```
