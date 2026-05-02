# 07 PREPLAN FEASIBILITY AND COMMAND PLAYBOOK

## 1) Mục tiêu bắt buộc trước khi đưa vào plan

Mọi task chỉ được đưa vào plan khi đã qua 4 bước:
1. Kiểm tra version tương thích.
2. Kiểm tra độ khả thi kỹ thuật (feasibility gate).
3. Vẽ map giao diện + map code có điểm chạm cụ thể.
4. Viết lệnh xử lý chi tiết kèm ví dụ và đối chiếu kết quả.

---

## 2) Compatibility Gate (version tương thích)

### 2.1 Danh mục bắt buộc check

1. Node.js / npm (phục vụ build dev).
2. Rust / Cargo (Tauri backend).
3. Tauri CLI / crate version.
4. Python runtime cho OCR.
5. OCR stack (`rapidocr`, `onnxruntime`, `paddleocr`, `opencv-python`, `numpy`).
6. SQLite (runtime + FTS5).

### 2.2 Câu lệnh minh họa kiểm tra

1. `node -v`
2. `npm -v`
3. `cargo -V`
4. `python --version`
5. `py -m pip show rapidocr onnxruntime paddleocr opencv-python numpy`
6. `cd PhanMem && npm run tauri:dev` (smoke test runtime)

### 2.3 Đối chiếu kết quả

- Nếu thiếu công cụ build (`node`, `cargo`) => chỉ chặn luồng dev, không chặn runtime user cuối.
- Nếu OCR dependency conflict => chặn plan OCR mới, xử lý dependency trước.
- Nếu `npm run tauri:dev` fail => không được mở phase code mới.

---

## 3) Feasibility Gate (độ khả thi)

### 3.1 Checklist khả thi kỹ thuật

1. Có điểm chạm code rõ ràng chưa?
2. Có schema dữ liệu để lưu kết quả chưa?
3. Có UI surface để hiển thị kết quả chưa?
4. Có test route để chứng minh pass/fail chưa?
5. Có fallback khi thiếu tài nguyên/hạ tầng chưa?

### 3.2 Mẫu đánh giá khả thi

`Task:` Startup self-check với chặn dashboard.

`Điểm chạm code:`
- [`main.rs`](PhanMem/src-tauri/src/main.rs:1)
- [`scanService.ts`](PhanMem/src/services/scanService.ts:1)
- [`main.tsx`](PhanMem/src/main.tsx:1)

`Kết luận khả thi:` PASS khi đủ backend command + frontend guard + UI fail screen.

---

## 4) UI Maps (maps giao diện)

## 4.1 Map tổng thể màn hình

1. `Startup Check Screen`
2. `Dashboard`
3. `Case List`
4. `Document Viewer`
5. `Export Studio`
6. `Settings (profile/mode)`

## 4.2 Map luồng tương tác startup

`App Launch` -> `System Startup Check` ->
- `PASS`: vào `Dashboard`
- `WARNING`: cho vào `Dashboard` + banner
- `FAIL`: giữ tại màn fail + nút `Copy log` / `Retry` / `Exit`

## 4.3 Câu lệnh kiểm tra UI map

1. Chạy `npm run tauri:dev`.
2. Giả lập thiếu component (`pdfium`) để xác nhận màn `FAIL`.
3. Giả lập thiếu AI model để xác nhận `WARNING` không chặn app.

---

## 5) Code Maps (maps code)

## 5.1 Backend map

1. Command entry: [`main.rs`](PhanMem/src-tauri/src/main.rs:1)
2. Runtime commands: [`PhanMem/src-tauri/src/commands/`](PhanMem/src-tauri/src/commands)
3. Storage/DB: [`storage.rs`](PhanMem/src-tauri/src/storage.rs:1), [`db/schema.rs`](PhanMem/src-tauri/src/db/schema.rs:1)

## 5.2 Frontend map

1. App entry: [`main.tsx`](PhanMem/src/main.tsx:1)
2. Service layer: [`PhanMem/src/services/`](PhanMem/src/services)
3. State layer: [`PhanMem/src/store/`](PhanMem/src/store)
4. UI style/layout: [`PhanMem/src/styles/`](PhanMem/src/styles)

## 5.3 OCR/Python map

1. OCR pipeline: [`ocr_pipeline.py`](PhanMem/python/ocr/ocr_pipeline.py:1)
2. PDF image conversion: [`pdf_to_images.py`](PhanMem/python/ocr/pdf_to_images.py:1)
3. OCR quality verify: [`verify_vietnamese_ocr.py`](PhanMem/python/ocr/verify_vietnamese_ocr.py:1)

---

## 6) Command Playbook (chi tiết từng việc, có ví dụ)

## 6.1 Ví dụ việc: Startup Gate

### Lệnh xử lý

1. Backend:
   - Thêm `run_startup_self_check`.
2. Frontend:
   - Gọi check trước mount dashboard.
3. UI:
   - Render màn fail nếu `status=fail`.

### Lệnh test minh họa

1. `cd PhanMem && npm run tauri:dev`
2. Trigger mock fail => verify UI chặn.
3. Trigger warning => verify app vẫn vào dashboard.

### Đối chiếu pass

`PASS` khi:
1. Có payload check đầy đủ `status/checks/fatal_errors/warnings`.
2. `FAIL` chặn vào dashboard.
3. `WARNING` không chặn app.

## 6.2 Ví dụ việc: OCR Unicode schema

### Lệnh xử lý

1. Cập nhật output block trong [`ocr_pipeline.py`](PhanMem/python/ocr/ocr_pipeline.py:1).
2. Lưu trường vào DB ở [`schema.rs`](PhanMem/src-tauri/src/db/schema.rs:1).

### Lệnh test minh họa

1. OCR sample tiếng Việt.
2. Query DB kiểm tra `raw_text` giữ dấu và `normalized_text` không dấu.

### Đối chiếu pass

`PASS` khi search không dấu vẫn tìm ra văn bản có dấu.

---

## 7) Quy tắc trước khi commit plan

Plan chỉ được ghi vào log điều phối khi đủ:
1. Compatibility Gate = PASS.
2. Feasibility Gate = PASS.
3. UI Map + Code Map đã chốt.
4. Có command playbook + expected evidence.

Nếu thiếu một mục, trạng thái bắt buộc: `HOLD`.

