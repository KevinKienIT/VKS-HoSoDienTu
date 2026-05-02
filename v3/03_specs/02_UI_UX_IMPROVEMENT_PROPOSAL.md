# 🚀 Đề xuất Nâng cấp UI/UX: Biến VKS ECMS thành "Document Workspace" chuyên nghiệp

Dựa trên cấu trúc hiện tại của dự án (Tauri + React + Rust + SQLite), phần mềm hiện đang dừng ở mức "quản lý danh sách" (List-based management). Đối với một hệ thống Quản lý Hồ sơ Điện tử (ECMS) chuyên nghiệp phục vụ ngành kiểm sát, trải nghiệm người dùng (UX) cần chuyển dịch từ **"Tìm và Xem danh sách"** sang **"Không gian làm việc tương tác" (Workspace/Split-pane)**.

Dưới đây là kế hoạch chuyên sâu để nâng cấp UI/UX, đáp ứng những người dùng khó tính nhất.

---

## 1. Kiến trúc Giao diện mới: "Dossier Workspace" (Không gian làm việc)

Thay vì click vào một hồ sơ và chỉ thấy danh sách tài liệu, chúng ta sẽ xây dựng giao diện **Split-Pane (Chia màn hình)** giống như các IDE (VSCode) hoặc các phần mềm quản lý tài liệu chuyên dụng (Evernote, Notion, Adobe Acrobat).

### Layout Đề xuất:
- **Left Sidebar (Cố định):** Danh mục phân cấp (Tree View)
  - Vụ án -> Nhóm tài liệu -> Tài liệu -> Trang (Thumbnails).
  - Cho phép Kéo-thả (Drag & Drop) để sắp xếp lại tài liệu.
- **Center Pane (Main Viewer):** Khu vực hiển thị nội dung tài liệu.
  - Hỗ trợ Tabs: Có thể mở nhiều tài liệu cùng lúc trên các Tab khác nhau.
- **Right Sidebar (Tùy chọn/Đóng mở):** Metadata & Công cụ.
  - Hiển thị thuộc tính tài liệu (Ngày tạo, loại, tag).
  - **AI Notebook:** Ghi chú, tóm tắt tự động, và tra cứu FTS5 (Full-Text Search) liên quan đến tài liệu đang mở.

---

## 2. Tích hợp Document Viewers Native (Không cần mở app ngoài)

Vì đây là Tauri App (.exe), chúng ta có thể load trực tiếp file từ hệ thống (local disk) bằng `convertFileSrc` (asset protocol) cực kỳ mượt mà. Cần xây dựng các Viewer sau:

### 2.1. PDF Viewer Nâng cao (react-pdf hoặc webview iframe)
- **Tính năng:**
  - Render PDF đa trang trực tiếp trong app.
  - Zoom in/out, Fit to width, Fit to page.
  - Xem Thumbnail bên lề trái của Viewer.
  - **Text Selection & Highlight:** Quét chọn text trong PDF để copy hoặc thêm vào AI Notebook.

### 2.2. Image & Scan Viewer (react-zoom-pan-pinch)
- **Tính năng:**
  - Chuyên dùng cho các tài liệu được scan thành ảnh (JPG/PNG).
  - **Deep Zoom & Pan:** Phóng to cực đại không vỡ nét (tận dụng module phóng ảnh HD có sẵn).
  - Xoay ảnh (Rotate), lật ảnh, tinh chỉnh độ sáng/độ tương phản cơ bản nếu ảnh quá mờ.

### 2.3. OCR Text / Markdown Viewer (Monaco Editor / Custom Viewer)
- **Tính năng:**
  - **Side-by-side View:** Nửa trái là Ảnh gốc, nửa phải là Text đã được OCR (phát hiện chữ).
  - Cho phép kiểm sát viên **sửa lỗi OCR** (Proofreading) trực tiếp trên giao diện và lưu lại vào SQLite.
  - Hỗ trợ bôi đậm, đánh dấu màu (Highlight) các từ khóa quan trọng.

---

## 3. Các tính năng "WOW" cho trải nghiệm người dùng

Để đáp ứng những người dùng khó tính, phần mềm cần có những "Micro-interactions" (tương tác siêu nhỏ) mang lại cảm giác cao cấp:

1. **Quick Preview (Nhấn phím Space):**
   - Giống tính năng QuickLook trên macOS. Khi đang ở danh sách tài liệu, chọn 1 file và nhấn `Space` -> Mở popup (Modal) hiển thị nhanh nội dung file mà không cần chuyển trang.
2. **Search Highlight (Nhảy đến trang chứa từ khóa):**
   - Khi tìm kiếm toàn văn (FTS5), click vào kết quả sẽ **tự động mở Document Viewer, cuộn đúng đến trang đó và tô vàng (highlight)** từ khóa.
3. **Offline AI Chat / Notebook kết nối với Viewer:**
   - Khi đang đọc PDF, bôi đen một đoạn text, click chuột phải -> `"Thêm vào Ghi chú"` hoặc `"Giải thích đoạn này"`. Đoạn text sẽ bay sang Right Sidebar (AI Notebook).
4. **Trạng thái xử lý trực quan (Progress Bars):**
   - Khi Import thư mục lớn hoặc đang chạy Pipeline OCR, hiển thị thanh tiến trình mini (Mini Progress) góc dưới màn hình, có thể click để xem chi tiết từng file đang chạy (giống giao diện tải file của Chrome).

---

## 4. Kế hoạch Triển khai (Lộ trình code)

Chúng ta có thể bắt đầu từng bước mà không làm vỡ cấu trúc hiện tại:

### Giai đoạn 1: Xây dựng Core Viewers (Cơ bản)
- Thêm các thư viện (ví dụ: `react-pdf`, `react-zoom-pan-pinch`).
- Nâng cấp `CaseDetailFallback` từ bảng danh sách (Table) thành giao diện **Split-pane** (Cây thư mục bên trái, Viewer bên phải).
- Sử dụng `convertFileSrc` của Tauri để load file PDF và Ảnh từ máy tính lên Viewer.

### Giai đoạn 2: Tương tác & Side-by-side OCR
- Chức năng click vào tài liệu -> Mở PDF/Ảnh ngay trên Viewer.
- Phát triển chế độ xem song song "Ảnh gốc - Text OCR".
- Thêm chức năng chỉnh sửa Text OCR và lưu lại database.

### Giai đoạn 3: Nâng cấp Search & AI Workspace
- Liên kết trang Tìm kiếm (Search) thẳng vào Viewer (nhảy đúng trang, highlight từ khóa).
- Hoàn thiện AI Notebook Sidebar tích hợp với Viewer.
