# Dac Ta Case Lifecycle Va Work Product

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dinh nghia vong doi nghiep vu cua mot case va cac san pham lam viec ma KSV thuc su can tao ra trong qua trinh nghien cuu ho so.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`
Nguon corpus: Toan bo file PDF trong `TaiLieu`

## Muc Luc

1. Vai tro cua tai lieu
2. Vong doi case nghiep vu
3. Work product bat buoc
4. Gate chuyen trang thai
5. Luat nghiep vu cho AI va agent
6. Acceptance criteria

## 1. Vai Tro Cua Tai Lieu

He thong khong duoc dung lai o muc:

- import PDF,
- OCR,
- tim kiem,
- viewer,
- AI tra loi co citation.

He thong phai di tiep den muc KSV tao duoc san pham lam viec thuc te tren ho so. Tai lieu nay dinh nghia ro:

- mot case di qua nhung trang thai nao,
- KSV tao ra nhung work product nao,
- khi nao moi du lieu duoc xem la san sang cho nghien cuu,
- khi nao duoc chuyen tu "du lieu OCR" sang "co so nghiep vu".

## 2. Vong Doi Case Nghiep Vu

Case trong he thong phai di qua cac trang thai sau:

1. `moi_nhap`
   - da chon folder,
   - da tao case record,
   - chua OCR day du.

2. `dang_xu_ly_ingest`
   - dang tach page,
   - dang OCR,
   - dang tao metadata ban dau.

3. `cho_review_du_lieu`
   - da co du lieu co ban,
   - con item confidence thap,
   - chua duoc dung de ket luan nghiep vu.

4. `san_sang_nghien_cuu`
   - metadata cot loi da du,
   - review queue o muc chap nhan duoc,
   - search, viewer, citation hoat dong.

5. `dang_nghien_cuu`
   - KSV dang doc ho so,
   - dang tao note, bookmark, contradiction item, timeline item, dossier draft.

6. `dang_tao_san_pham_nghiep_vu`
   - dang tong hop tai lieu,
   - dang tao de cuong, bang van de, ho so doi tuong, timeline, draft report.

7. `cho_chot_dot_lam_viec`
   - work product da co,
   - can review/kiem tra lai truoc khi chot.

8. `tam_ket_thuc`
   - dot nghien cuu da chot,
   - con the mo lai neu co tai lieu moi hoac review moi.

9. `luu_tru_noi_bo`
   - khong con nam trong luong thao tac chinh,
   - chi doc va tra cuu.

## 3. Work Product Bat Buoc

He thong phai ho tro tao va quan ly it nhat cac work product sau:

1. `reading_queue`
   - danh sach tai lieu can doc tiep,
   - thu tu doc uu tien,
   - ly do duoc dua vao queue.

2. `bookmark`
   - danh dau tai file/trang/but_luc/cuom trich dan quan trong.

3. `note_ca_nhan`
   - ghi chu cua KSV,
   - khong tu dong xem la ket luan nghiep vu chinh thuc.

4. `question_item`
   - cau hoi nghiep vu chua duoc giai quyet.

5. `issue_item`
   - van de can lam ro trong ho so.

6. `contradiction_item`
   - mâu thuan giua 2 hoac nhieu tai lieu/citation.

7. `lead_item`
   - huong dieu tra, huong kiem tra, huong xac minh bo sung.

8. `timeline_item`
   - su kien co ngay gio hoac moc thoi gian,
   - phai tro nguoc duoc ve citation.

9. `dossier_draft`
   - ho so doi tuong,
   - ho so vat chung/thiet bi,
   - ho so su kien.

10. `report_draft`
   - de cuong nghien cuu,
   - tong hop chung cu,
   - goi y ket luan,
   - nhap nhap bao cao nghiep vu.

11. `export_package`
   - bo tai lieu xuat ra JSON/PDF/XLSX/ZIP cho mot dot nghien cuu.

## 4. Gate Chuyen Trang Thai

| Tu trang thai | Sang trang thai | Dieu kien bat buoc |
| --- | --- | --- |
| `moi_nhap` | `dang_xu_ly_ingest` | folder hop le, job import duoc tao |
| `dang_xu_ly_ingest` | `cho_review_du_lieu` | da OCR/index xong, da co review items neu can |
| `cho_review_du_lieu` | `san_sang_nghien_cuu` | da co `display_name`, `document_type`, `ocr_confidence_avg`, citation co ban |
| `san_sang_nghien_cuu` | `dang_nghien_cuu` | KSV da mo case va tao work product dau tien |
| `dang_nghien_cuu` | `dang_tao_san_pham_nghiep_vu` | da co timeline/dossier/question/issue dang dung |
| `dang_tao_san_pham_nghiep_vu` | `cho_chot_dot_lam_viec` | draft co citation day du |
| `cho_chot_dot_lam_viec` | `tam_ket_thuc` | report dot viec va work product duoc review |
| `tam_ket_thuc` | `luu_tru_noi_bo` | khong con task active |

## 5. Luat Nghiep Vu Cho AI Va Agent

- AI chi duoc tao `draft`, khong duoc tu dong nang cap thanh ket luan nghiep vu chinh thuc.
- Moi work product do AI de xuat phai co:
  - citation bundle,
  - warning/confidence,
  - trang thai `can_review`.
- Agent code khong duoc bo qua lop work product. Neu chi xay viewer/search ma khong co work product thi chua dat scope nghiep vu.
- Moi dashboard, layout, panel hay API de xuat deu phai chi ro no phuc vu work product nao.

## 6. Acceptance Criteria

- Case co trang thai ro rang, khong dung mot status chung chung.
- Moi work product tro nguoc duoc ve citation.
- Note ca nhan va ket luan chinh thuc duoc tach ro.
- AI chi tao draft, khong tu dong chot ket luan.
- UI cho phep KSV tiep tuc cong viec dang do ma khong mat reading queue, bookmark, note va contradiction item.
