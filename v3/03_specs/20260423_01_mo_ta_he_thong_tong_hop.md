# Mo Ta He Thong Tong Hop - VKS Ho So Dien Tu

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Tai lieu goc de agent hieu day du nghiep vu, corpus tai lieu, module, luong du lieu, tim kiem, trich dan va AI offline truoc khi de xuat code.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`
Nguon corpus: Toan bo file PDF trong `TaiLieu`

## Muc Luc

1. Vai tro cua he thong
2. Corpus nguon thuc te quan sat duoc
3. Muc tieu nghiep vu cua kiem sat vien
4. Nguyen tac thiet ke
5. Luong xu ly tong the
6. Ban do module truc quan
7. Danh muc module can co
8. Mo hinh du lieu cot loi
9. Tim kiem, sap xep va trich dan
10. AI offline phan tich tai lieu
11. Yeu cau giao dien
12. Ngoai le, review va do tin cay
13. Tai lieu bo tro bat buoc
14. Dinh nghia hoan thanh cho agent

---

## 1. Vai Tro Cua He Thong

He thong nay khong chi la OCR scanner. Day la mot workspace nghiep vu offline cho KSV de:

- nhap va to chuc ho so scan,
- biet moi file la gi,
- tim duoc noi dung nhanh,
- sap xep ho so theo nhieu truong nghiep vu,
- xem trich dan ro file/trang/but luc,
- theo doi doi tuong, vat chung, timeline,
- va dung AI offline de tom tat, hoi dap va phan tich co kiem soat.

Ket qua dau ra ma he thong phai tao duoc:

- danh sach tai lieu co ten de doc,
- metadata chuan hoa de loc/sap xep,
- preview noi dung de nhin nhanh,
- text viet tay trong to loi khai, don viet tay va ghi chu tay duoc tach rieng theo revision, co danh dau doan nghi van va de xuat noi suy neu can,
- lien ket tu file goc -> trang -> but luc -> doi tuong -> vat chung,
- cau tra loi AI co trich dan nguon,
- work product nghiep vu co citation,
- audit trail cho override va sua tay.

---

## 2. Corpus Nguon Thuc Te Quan Sat Duoc

Du lieu hien tai trong `TaiLieu` cho thay mot corpus scan thuc te voi dac diem:

- 1 thu muc case duoc quan sat
- 68 file PDF
- tong dung luong khoang 211.20 MB
- file dau tien: `2024-01-25-08-48-50-01.pdf`
- file cuoi cung: `2024-01-25-09-33-23-01.pdf`
- 23 file < 1 MB
- 33 file tu 1 den 5 MB
- 7 file tu 5 den 10 MB
- 5 file >= 10 MB

Nhan xet nghiep vu tu corpus nay:

1. Ten file hien tai chi la timestamp, khong cho biet ten van ban.
2. Ten thu muc case co chua thong tin nghiep vu nhung chua tach thanh truong du lieu ro rang.
3. Quy mo file khong dong deu, cho thay co file ngan, file dai, file tap hop nhieu trang.
4. Khong the dua vao filename de phan loai, bat buoc phai OCR + metadata extraction.
5. He thong phai tu tao `display_name`, `document_title`, `summary_preview` va `search_index`.

He qua trien khai:

- import order phai giu duoc thu tu thoi gian goc,
- phai luu `original_filename`,
- phai sinh ten hien thi nghiep vu rieng de KSV doc,
- phai co review queue cho file OCR yeu hoac khong xac dinh loai.

---

## 3. Muc Tieu Nghiep Vu Cua Kiem Sat Vien

KSV can lam duoc it nhat 10 viec sau:

1. Xem nhanh mot case dang co nhung file nao.
2. Biet tung file la van ban gi va doc duoc ca phan text viet tay quan trong ma khong can mo thu cong tung PDF.
3. Tim tai lieu theo ten, noi dung, but luc, loai van ban, doi tuong, vat chung, ngay ban hanh.
4. Sap xep danh sach tai lieu theo thu tu minh muon.
5. Mo tai lieu va thay ro file goc, trang, but luc, do tin cay OCR, span text viet tay nghi van, trich dan.
6. Xem ho so doi tuong va cac vat chung lien quan.
7. Hoi AI offline de tom tat, tra cuu nhanh, so sanh, va rut trich thong tin.
8. Review va sua cac muc AI/OCR nghi ngo sai.
9. Tiep tuc cong viec dang do ma khong mat note, bookmark, reading queue.
10. Tao contradiction item, timeline, dossier draft va report draft co citation.

Neu he thong khong giup KSV lam 8 viec nay nhanh hon ro ret, thi chua dat muc tieu san pham.

---

## 4. Nguyen Tac Thiet Ke

- Offline 100%, khong phu thuoc internet.
- Prompt, docs va nghiep vu phai dan dat implementation; agent khong duoc tu suy dien scope.
- Tach theo page-level nhung van giu duoc ngu canh document-level va case-level.
- AI khong thay OCR va khong thay tai lieu goc.
- To loi khai bi can viet tay, don viet tay va ghi chu tay la truong hop bat buoc phai xu ly duoc, khong duoc day sang phase mo rong.
- Doan chu/noi suy text chi duoc luu duoi dang de xuat co confidence, candidate list va review state; khong duoc im lang ghi de len text goc.
- Moi ket qua quan trong phai truy nguoc duoc ve file/trang/but luc.
- Giao dien uu tien toc do tra cuu, khong uu tien trang tri thuần tuy.
- Moi truong scan xau la mac dinh, khong phai truong hop hiem.

---

## 5. Luong Xu Ly Tong The

```text
TaiLieu
  ->
Ingest Case
  ->
Split / Page Index
  ->
Preprocess
  ->
OCR
  ->
Region Detect
  ->
Metadata + Classification + Entity Mapping
  ->
Citation Index + Search Index
  ->
Review Queue
  ->
Viewer / Search / Dossier / AI Offline
  ->
Export / Report
```

Luong nghiep vu song song:

- luong tai lieu: file -> page -> but luc -> title -> preview
- luong doi tuong: case -> person -> role -> evidence -> citations
- luong kiem soat: confidence -> review queue -> manual confirm -> final index

---

## 6. Ban Do Module Truc Quan

```text
[module_ingest_case]
    -> [module_split_and_page_index]
    -> [module_ocr]
    -> [module_region_detect]
    -> [module_classification]
    -> [module_metadata_and_entity_map]
    -> [module_document_dictionary]
    -> [module_citation_index]
    -> [module_search_and_sort]
    -> [module_viewer_ui]
    -> [module_workspace_support]
    -> [module_ai_offline_assistant]
    -> [module_review_queue]
    -> [module_audit_trail_and_override]
    -> [module_import_job_and_recovery]
    -> [module_export_and_report]
```

Ban do nghiep vu <-> module:

| Nhu cau nghiep vu | Module chinh |
| --- | --- |
| Nhap ho so va giu thu tu file goc | `module_ingest_case` |
| Tach page va gan page index | `module_split_and_page_index` |
| Doc noi dung scan | `module_ocr` |
| Tim but luc, dau, chu ky | `module_region_detect` |
| Biet file la gi | `module_classification` |
| Trich ten, ngay, doi tuong, summary | `module_metadata_and_entity_map` |
| Chot taxonomy va display name theo tu dien chuan | `module_document_dictionary` |
| Tim nhanh va sap xep | `module_search_and_sort` |
| Mo file va xem trich dan | `module_viewer_ui` + `module_citation_index` |
| Ghi nho, note, pin, compare, contradiction, draft | `module_workspace_support` |
| Hoi dap va tom tat offline | `module_ai_offline_assistant` |
| Xu ly file yeu confidence | `module_review_queue` |
| Ghi vet before/after, reason, approval | `module_audit_trail_and_override` |
| Resume import, backup, restore | `module_import_job_and_recovery` |
| Xuat tai lieu, bao cao, muc luc | `module_export_and_report` |

---

## 7. Danh Muc Module Can Co

| Module | Muc dich | Dau vao | Dau ra | Nguoi dung chinh |
| --- | --- | --- | --- | --- |
| `module_ingest_case` | Tao case tu thu muc scan, giu ngu canh folder/file goc | folder path, original files | case record, document records | KSV / operator |
| `module_split_and_page_index` | Tach document thanh page logic | pdf/image | page records, page order | pipeline |
| `module_ocr` | Lay text va confidence cho text may + text viet tay | page image | ocr_text, line boxes, confidence, uncertain spans, candidate readings | pipeline |
| `module_region_detect` | Tim but luc, dau, chu ky, vung quan trong | page image, OCR lines | regions, but_luc candidates | pipeline / reviewer |
| `module_classification` | Xac dinh loai van ban | ocr_text, layout hints | document_type, confidence | pipeline / reviewer |
| `module_metadata_and_entity_map` | Trich ngay, title, person, evidence, event, summary | ocr_text, regions | metadata, entities, event markers | KSV / AI |
| `module_document_dictionary` | Chot taxonomy document type va quy tac display name | metadata, ocr_text, review signals | canonical document_type, display_name | pipeline / reviewer |
| `module_citation_index` | Tao lien ket truy vet | file/page/but_luc/entities | citation records | KSV / AI / report |
| `module_search_and_sort` | Tim kiem va sap xep ket qua | query, filters, sort key | result list co preview | KSV |
| `module_viewer_ui` | Xem tai lieu, tab, zoom, preview, dossier | selected doc/page | interactive viewer | KSV |
| `module_workspace_support` | Quan ly reading queue, bookmark, note, compare, contradiction, draft | documents, citations, user actions | work products dang thao tac | KSV |
| `module_ai_offline_assistant` | Tom tat, hoi dap, phan tich co trich dan va goi y doc chu viet tay mo | indexed docs, query | answer, summary, citation list, candidate reading bundle | KSV |
| `module_review_queue` | Xu ly ngoai le | low confidence items | approve/reject/override | reviewer |
| `module_audit_trail_and_override` | Ghi vet before/after, actor, reason, approval | manual edits, AI drafts, overrides | audit events, revision history | reviewer / KSV / admin |
| `module_import_job_and_recovery` | Quan ly import job, resume, backup, restore | import requests, snapshots | import state, backup package, restore result | operator / KSV |
| `module_export_and_report` | Xuat muc luc, ho so, report, archive | case selection | txt/xlsx/pdf/report package | KSV / admin |

Agent nao de xuat code phai chi ro dang dong vao module nao trong bang nay.

---

## 8. Mo Hinh Du Lieu Cot Loi

### 8.1 Case-level

- `case_code`
- `case_display_name`
- `source_folder_name`
- `case_sequence_no`
- `primary_person_name`
- `case_group_label`
- `status`

### 8.2 Document-level

- `original_filename`
- `import_sequence`
- `file_path`
- `file_hash`
- `file_size`
- `page_count`
- `display_name`
- `document_title`
- `document_type`
- `issued_date`
- `summary_short`
- `summary_detail`
- `ocr_confidence_avg`
- `classification_confidence`
- `needs_review`

### 8.3 Page-level

- `page_index`
- `image_path`
- `ocr_text`
- `ocr_preview`
- `but_luc`
- `confidence`
- `regions`

### 8.4 Entity-level

- `person`
- `role`
- `evidence`
- `device`
- `event`
- `legal_reference`

### 8.5 Citation-level

- `source_file`
- `source_page`
- `source_but_luc`
- `quote_excerpt`
- `confidence`
- `entity_links`

### 8.6 Work-product level

- `work_product_type`
- `work_product_status`
- `owner_actor_id`
- `title`
- `body`
- `linked_citation_ids`
- `linked_entity_ids`
- `review_state`

### 8.7 Audit-level

- `audit_event_id`
- `object_type`
- `object_id`
- `field_name`
- `before_value`
- `after_value`
- `actor_type`
- `actor_id`
- `reason_code`
- `approval_status`

Khong du bo truong tren thi he thong se tim kiem duoc, nhung KSV se khong “nam duoc” ho so.

---

## 9. Tim Kiem, Sap Xep Va Trich Dan

He thong phai ho tro:

### 9.1 Tim kiem

- theo ten file goc
- theo ten hien thi
- theo noi dung OCR
- theo but luc
- theo loai van ban
- theo ngay ban hanh
- theo ten doi tuong
- theo vat chung / thiet bi
- theo tu khoa AI / semantic search

### 9.2 Sap xep

- theo import sequence
- theo original filename
- theo display name
- theo but luc tang/giam
- theo issued date
- theo document_type
- theo OCR confidence
- theo last opened

### 9.3 Hien thi ket qua

Moi ket qua tim kiem phai co toi thieu:

- original filename
- display name
- document type
- but luc hoac page ref
- summary preview
- match snippet
- source folder / case
- confidence state

### 9.4 Trich dan

Moi cau tra loi AI, summary chi tiet, dossier, hoac report suy luan phai dan duoc ve:

- file nao
- trang nao
- but luc nao
- doan text nao neu co
- confidence o muc nao

---

## 10. AI Offline Phan Tich Tai Lieu

AI offline trong du an nay co 3 lop:

1. Lop rule + extractive:
   - luon chay duoc,
   - dung de classification, keywording, extractive summary.
2. Lop retrieval:
   - tim trang lien quan, lay context, sap xep do phu hop.
3. Lop local LLM:
   - hoi dap, tong hop, dossier generation, comparative reasoning.

Yeu cau bat buoc cho AI:

- khong tra loi neu khong co context du,
- khong phat minh chi tiet khong co trong ho so,
- uu tien tra ve citation truoc, ket luan sau,
- co che do “chi tra cuu” va “phan tich co giai thich”.

Dong model chap nhan:

- CPU-only baseline: extractive + retrieval
- Local LLM tier: Qwen / Gemma / Llama dang local
- Model nao duoc chon sau cung phai phu hop hardware offline

---

## 11. Yeu Cau Giao Dien

Giao dien phai giup KSV lam viec nhanh, khong chi dep:

- list view co the scan nhanh 50-100 tai lieu
- viewer PDF/anh co zoom, pan, next/prev, tab
- pane metadata / citation / dossier mo rong thu gon nhanh
- ket qua tim kiem co snippet ro rang
- sort/filter khong lag
- auto-save cho preferences
- responsive trong cua so desktop
- co reading queue, continue reading, pin, bookmark, note
- co split view de so sanh tai lieu/citation
- co contradiction board va report draft panel

Phong cach:

- web-like desktop workspace
- uu tien mat do thong tin va kha nang doc
- animation vua du, khong lam cham nghiep vu

---

## 12. Ngoai Le, Review Va Do Tin Cay

Review queue bat buoc cho:

- OCR confidence thap
- khong tim thay but luc
- document_type mo ho
- summary khong ro
- entity mapping xung dot

Moi item review phai co:

- ly do bi day vao queue
- du lieu goc
- suggestion cua he thong
- thao tac approve / edit / reject

---

## 13. Tai Lieu Bo Tro Bat Buoc

Agent truoc khi code phai doc them:

- `V2/05_tai_lieu_mo_ta/20260424_02_nghiep_vu_kiem_sat_vien_va_user_story.md`
- `V2/05_tai_lieu_mo_ta/20260424_03_ban_do_module_va_luong_du_lieu.md`
- `V2/05_tai_lieu_mo_ta/20260424_04_dac_ta_tim_kiem_sap_xep_va_trich_dan.md`
- `V2/05_tai_lieu_mo_ta/20260424_05_ho_so_nguon_tai_lieu_va_chien_luoc_ingest.md`
- `V2/05_tai_lieu_mo_ta/20260424_06_dac_ta_ai_offline_phan_tich_tai_lieu.md`
- `V2/05_tai_lieu_mo_ta/20260424_07_dac_ta_case_lifecycle_va_work_product.md`
- `V2/05_tai_lieu_mo_ta/20260424_08_dac_ta_audit_trail_va_manual_override.md`
- `V2/05_tai_lieu_mo_ta/20260424_09_dac_ta_citation_anchor_va_ocr_revision.md`
- `V2/05_tai_lieu_mo_ta/20260424_10_tu_dien_document_type_va_quy_tac_phan_loai.md`
- `V2/05_tai_lieu_mo_ta/20260424_11_dac_ta_import_job_backup_restore.md`
- `V2/05_tai_lieu_mo_ta/20260424_12_dac_ta_workspace_ho_tro_kiem_sat_vien.md`

---

## 14. Dinh Nghia Hoan Thanh Cho Agent

Mot de xuat code hoac giao dien chi duoc xem la bam tai lieu khi:

- chi ro dang sua module nao,
- khong bo sot nghiep vu tim kiem/sap xep/trich dan,
- khong bo sot review queue,
- co phuong an AI offline ro rang,
- va khong lam mat lien ket tu file goc den citation.

