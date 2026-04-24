# Ho So Nguon Tai Lieu Va Chien Luoc Ingest

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dua corpus `TaiLieu` thanh rang buoc thiet ke cu the cho ingest, naming va mapping.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`
Nguon corpus: Toan bo file PDF trong `TaiLieu`

## Muc Luc

1. Quan sat tu corpus
2. He qua thiet ke
3. Quy tac ingest
4. Truong du lieu bat buoc khi ingest

## 1. Quan Sat Tu Corpus

Corpus hien tai:

- 1 case folder
- ten folder goc quan sat: `15. Le Thanh Cong - 134 - Lien`
- 68 file PDF
- tong dung luong khoang 211.20 MB
- file dat ten theo timestamp, khong mang nghia nghiep vu
- dải thoi gian file tu 2024-01-25 08:48:50 den 2024-01-25 09:33:23

Phan bo dung luong:

- 23 file < 1 MB
- 33 file tu 1 den 5 MB
- 7 file tu 5 den 10 MB
- 5 file >= 10 MB

## 2. He Qua Thiet Ke

- He thong phai coi `original_filename` la du lieu tham chieu, khong phai ten de doc.
- `import_sequence` phai duoc luu ro rang.
- `source_folder_name` phai duoc luu o muc case.
- `display_name` va `document_title` phai duoc sinh sau OCR + classification.
- File lon co the la tai lieu nhieu trang, phai tach page va page_count trong ingest.

## 3. Quy Tac Ingest

1. Quet toan bo folder.
2. Tao case tu ten folder.
3. Giu thu tu file theo timestamp filename + last write time.
4. Tinh hash de tranh import lap.
5. Tach page va luu page_count.
6. OCR tung page.
7. Tao metadata document-level tu tong hop cac page.
8. Day item confidence thap vao review queue.

## 4. Truong Du Lieu Bat Buoc Khi Ingest

- source_folder_name
- original_filename
- import_sequence
- file_hash
- file_size
- page_count
- detected_title
- detected_document_type
- summary_short
- ocr_confidence_avg
- needs_review

Neu ingest xong ma chua co nhung truong nay, thi dataset chua san sang cho giao dien tra cuu.

