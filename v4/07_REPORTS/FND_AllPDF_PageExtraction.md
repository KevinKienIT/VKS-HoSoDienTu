# FND-AllPDF Page Extraction Audit Report

## 1. Phương pháp kiểm tra
- **Audit Codebase:** Phân tích luồng thực thi trong `PhanMem/src-tauri/src/commands/import_cmd.rs` (quản lý import), `doc_cmd.rs` (quản lý trích xuất và OCR) và `PhanMem/python/ocr/pdf_to_images.py` (script extract ảnh PNG).
- **Automated DB Verification:** Xây dựng script Python (`PhanMem/scripts/verify_allpdf_extraction.py`) truy cập trực tiếp vào CSDL SQLite của hệ thống (`vks-ecms.db`) để:
  1. Lấy danh sách toàn bộ các file PDF đã được import.
  2. So sánh `page_count` của file gốc với số lượng bản ghi tương ứng trong bảng `pages`.
  3. Kiểm tra trường `extract_status` có giá trị `'extracted'` hay không.
  4. Xác thực đường dẫn `image_path` có hợp lệ và file PNG vật lý có thực sự tồn tại trên ổ cứng.

## 2. Kết quả kiểm tra
**FAIL** - Pipeline trích xuất ảnh hiện tại đang tồn tại lỗ hổng khiến ảnh không được sinh ra đầy đủ cho toàn bộ PDF.

- **Về số lượng bản ghi:** Luồng Import (`import_cmd.rs`) ĐÃ insert ĐỦ số lượng dòng vào bảng `pages` tương ứng với `page_count` của PDF (Sử dụng hàm `count_pdf_pages` lúc nhập file).
- **Về tiến trình trích xuất (Extraction):** Phát hiện nhiều Document (ví dụ: `VK_1778129491582_0002_pdf.pdf`) có toàn bộ trang nằm ở trạng thái lỗi:
  - `extract_status` = `pending`.
  - `image_path` = `None`.
  - File PNG vật lý chưa được sinh ra.

## 3. Phân tích nguyên nhân (Root Causes)
Việc thất thoát trích xuất ảnh không phải do script Python lỗi (test chạy thủ công `pdf_to_images.py` cho thấy script extract 4/4 trang hoàn hảo), mà do lỗi logic trong luồng Pipeline của Rust:

1. **Extraction bị phụ thuộc cứng vào OCR (Coupling):**
   Trong file `import_cmd.rs` (dòng 495), quá trình Import chỉ tạo các metadata rỗng (`image_path = None`). Tiến trình gọi `pdf_to_images.py` lại đang bị "gắn chết" bên trong hàm `run_ocr_for_document_scope_with_conn` của `doc_cmd.rs`.
   👉 **Hệ quả:** Nếu người dùng chỉ import file mà chưa bấm chạy OCR, các file PDF sẽ hoàn toàn không có ảnh PNG để xem. Điều này phá vỡ kiến trúc "PNG-First Viewer" vừa được thiết lập ở `FND-006`.

2. **Thiếu sót Update DB Status:**
   Ngay cả với những file đã chạy qua OCR (như `test_doc_1.pdf`), hàm lưu kết quả `upsert_ocr_text` trong `doc_cmd.rs` (dòng 468) chỉ cập nhật trường `image_path` và bỏ sót hoàn toàn trường `extract_status`. Trạng thái này không bao giờ được cập nhật thành `'extracted'` trong codebase của Rust.

## 4. Kế hoạch khắc phục chi tiết (Remediation Plan)

### Các file cần chỉnh sửa
- `PhanMem/src-tauri/src/commands/doc_cmd.rs`
- `PhanMem/src-tauri/src/commands/import_cmd.rs`

### Expected Diff (Dự kiến thay đổi)
1. **Tách biệt Pipeline Extract ảnh khỏi OCR:**
   - Cần bổ sung một hàm độc lập (ví dụ `extract_images_for_document`) trong `doc_cmd.rs` chuyên gọi `pdf_to_images.py`.
   - Hàm này phải được gọi tự động (có thể qua async background task) ngay khi `import_cmd.rs` hoàn tất quá trình insert DB của file PDF.

2. **Cập nhật Status Database:**
   - Thêm câu lệnh UPDATE để gán giá trị `'extracted'` cho cột `extract_status` và gán `image_path` thực tế vào DB ngay khi Python script chạy xong.
   - Sửa query trong `upsert_ocr_text` (nếu cần) để tránh override sai trạng thái extraction.

### Test Gate
Sau khi apply các thay đổi trên, tiến hành:
1. Import mới một thư mục chứa nhiều file PDF.
2. Chạy lại `python PhanMem/scripts/verify_allpdf_extraction.py`.
3. Yêu cầu **100% PASS**: Không có trang nào bị `image_path = None` và toàn bộ phải có `extract_status = 'extracted'` trước khi người dùng kịp bấm chạy OCR.
