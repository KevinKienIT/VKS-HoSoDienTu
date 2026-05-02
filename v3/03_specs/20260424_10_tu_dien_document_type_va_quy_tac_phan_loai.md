# Tu Dien Document Type Va Quy Tac Phan Loai

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Chot taxonomy document type de UI, search, sort, dossier, report va AI khong moi noi mot kieu.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`

## Muc Luc

1. Nguyen tac taxonomy
2. Nhom document type chuan
3. Quy tac phan loai
4. Quy tac sinh display name
5. Quy tac review classification
6. Acceptance criteria

## 1. Nguyen Tac Taxonomy

- Mot document chi co 1 `document_type` chinh.
- Co the co nhieu `document_tags` phu.
- `document_type` khong duoc dat tuy y theo UI component.
- Search, sort, report, work product va export phai dung chung tu dien nay.

## 2. Nhom Document Type Chuan

Nhom chuan phase 1:

1. `bien_ban`
2. `loi_khai`
3. `hoi_cung`
4. `doi_chat`
5. `quyet_dinh`
6. `lenh`
7. `ket_luan_dieu_tra`
8. `cao_trang`
9. `thong_bao`
10. `giam_dinh`
11. `tai_lieu_nhan_than`
12. `tai_lieu_tai_chinh`
13. `du_lieu_dien_tu`
14. `vat_chung_hinh_anh`
15. `van_ban_to_tung_khac`
16. `khong_xac_dinh`

`document_tags` co the gom:

- `co_dau`
- `co_chu_ky`
- `co_but_luc`
- `scan_chat_luong_thap`
- `viet_tay`
- `can_review`

## 3. Quy Tac Phan Loai

He thong phan loai phai dung:

- OCR text o nhieu trang dau, khong chi trang 1,
- layout cues,
- heading cues,
- but luc / con dau / mau van ban,
- tu khoa nghiep vu.

Khong duoc:

- chi dua vao filename,
- chi dua vao 1 keyword duy nhat,
- map thang tu OCR confidence sang classification confidence.

Confidence bands:

- `>= 0.90`: tu dong chap nhan neu khong co xung dot
- `0.75 - <0.90`: hien thi cho phep dung tam, dua vao review nhanh
- `< 0.75`: bat buoc vao review queue

## 4. Quy Tac Sinh Display Name

`display_name` khong duoc la ban copy filename timestamp.

Cong thuc uu tien:

`<document_type_display> - <doi_tuong_or_chu_the> - <issued_date_or_but_luc>`

Vi du:

- `Bien ban - Le Thanh Cong - but luc 134`
- `Ket luan dieu tra - 2024-01-25`
- `Tai lieu nhan than - Nguyen Van B`

Neu thieu du lieu:

- van phai sinh ten co nghia toi thieu,
- dong thoi danh dau `needs_review = true`.

## 5. Quy Tac Review Classification

Reviewer phai thay:

- `document_type` hien tai,
- top 3 ung vien classification,
- keyword/heading da dung de suy ra,
- confidence,
- citation/region ho tro neu co.

Khi reviewer override:

- bat buoc ghi audit trail,
- cap nhat search facets,
- cap nhat display name neu phu thuoc loai van ban.

## 6. Acceptance Criteria

- Tat ca UI va schema dung chung cung mot taxonomy.
- Khong con loai document do component tu dat rieng.
- `document_type = khong_xac_dinh` duoc xem la trang thai hop le, khong ep doan.
- `display_name` co the doc duoc boi KSV.
- Classification va manual override deu co the truy vet.
