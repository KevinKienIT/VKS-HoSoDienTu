# 04 PHASE AI CITATION AND SUMMARY

## Mục tiêu

Triển khai citation-first đầy đủ + tổng hợp phân tầng page→document→group→case.

## Câu lệnh xử lý theo từng việc

1. `P4-01 Citation hard-stop`
   - Chặn kết luận thiếu citation trong [`ai_cmd.rs`](PhanMem/src-tauri/src/commands/ai_cmd.rs:1).

2. `P4-02 Page summary`
   - Lưu `page_summary` + citation theo page trong DB, mở rộng schema tại [`schema.rs`](PhanMem/src-tauri/src/db/schema.rs:1).

3. `P4-03 Document/group/case summary`
   - Build pipeline summary phân tầng trong scheduler [`scan_cmd.rs`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1).

4. `P4-04 Low confidence review`
   - Với OCR low confidence hoặc handwriting, luôn `review_required=true` tại [`doc_cmd.rs`](PhanMem/src-tauri/src/commands/doc_cmd.rs:1).

