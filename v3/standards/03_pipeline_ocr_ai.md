# 03 Pipeline OCR AI

## MVP pipeline bắt buộc

1. Import tài liệu
2. OCR tiếng Việt Unicode (UTF-8, NFC)
3. Rule extraction cơ bản
4. Lập chỉ mục SQLite FTS5
5. Viewer cho review/sửa OCR
6. Export cơ bản DOCX/HTML

## OCR schema tối thiểu

Mỗi block OCR cần có:
1. `raw_text`
2. `normalized_text`
3. `bbox/polygon`
4. `confidence`
5. `page_number`
6. `document_id`

## AI policy

1. AI không thay OCR engine.
2. AI không kết luận pháp lý cuối cùng.
3. AI chỉ hoạt động trên dữ liệu đã có citation chain.
4. Thiếu AI model là warning, không fail toàn app.

## Giai đoạn nâng cao

1. Hierarchical summarization (page → document → group → case)
2. Citation validator hard-stop
3. Timeline extraction
4. Handwriting assist theo vùng
