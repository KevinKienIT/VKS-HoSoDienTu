# Dac Ta Tim Kiem Sap Xep Va Trich Dan

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Chot cac truong, hanh vi va ket qua hien thi cho nhu cau tra cuu cua KSV.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`

## Muc Luc

1. Tim kiem bat buoc
2. Sap xep bat buoc
3. Hien thi ket qua
4. Trich dan bat buoc

## 1. Tim Kiem Bat Buoc

He thong phai ho tro nhung kieu query sau:

- ten file goc
- ten hien thi do he thong sinh ra
- keyword OCR
- text viet tay da duyet
- so but luc
- loai van ban
- ngay van ban
- ten doi tuong / vai tro
- vat chung / thiet bi
- truy van semantic bang AI offline

Bo loc bat buoc:

- case
- document_type
- date range
- has_but_luc
- needs_review
- confidence band
- has_handwriting
- transcription_state

## 1.1 Quy Tac Nhap Lieu Tim Kiem

- **Ho tro Unicode:** Tat ca input field phai ho tro nhap va hien thi dung Unicode tieng Viet.
- **Tim kiem khong dau:** Khi KSV nhap text khong dau (VD: "nguyen van a"), he thong phan mem phai co the match va tim ra cac ket qua co dau tuong ung (VD: "Nguyễn Văn A"). Yeu cau Frontend hoac Backend normalize text truoc khi dua vao FTS index/query.

## 2. Sap Xep Bat Buoc

KSV phai sap xep duoc theo:

- import_sequence
- original_filename
- display_name
- but_luc
- issued_date
- document_type
- OCR confidence
- classification confidence
- last_opened

Sort phai ho tro tang/giam va luu preference gan nhat theo user.

## 3. Hien Thi Ket Qua

Moi dong ket qua phai hien:

- icon loai tai lieu
- original_filename
- display_name
- case_id
- but_luc hoac page ref
- document_type
- issued_date neu co
- summary preview
- snippet noi dung match
- confidence state
- transcription badge (`direct`, `candidate_only`, `interpolated_pending_review`, `approved_manual`)
- citation_anchor_id
- ocr_revision_id

Click vao ket qua phai:

- mo dung file
- nhay den dung trang
- highlight snippet neu co
- mo panel citation

## 4. Trich Dan Bat Buoc

Trich dan chuan gom:

- `source_file`
- `source_page`
- `source_but_luc`
- `quote_excerpt`
- `entity_links`
- `confidence`
- `ocr_revision_id`
- `citation_anchor_id`
- `transcription_state`

Neu trich dan tro toi span viet tay/nghi van:

- bat buoc co `region_bbox`,
- khong duoc hien thi nhu text da chot neu `transcription_state != approved_manual` va `transcription_state != direct`.

Tat ca tinh nang sau phai xuat citation:

- summary chi tiet
- dossier doi tuong
- timeline su kien
- hoi dap AI offline
- report nghiep vu do AI ho tro

