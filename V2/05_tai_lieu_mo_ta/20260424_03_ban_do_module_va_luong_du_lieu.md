# Ban Do Module Va Luong Du Lieu

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Lam ro module nao lam gi, phu thuoc vao dau, xuat ra dau, va module nao phuc vu nghiep vu nao.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`

## Muc Luc

1. Ban do tong the
2. Ma tran module
3. Gate du lieu giua cac module
4. Rui ro neu thieu module

## 1. Ban Do Tong The

```text
TaiLieu
  -> ingest_case
  -> split_page
  -> ocr
  -> region_detect
  -> classification
  -> metadata_entity_map
  -> citation_index
  -> search_sort
  -> viewer_dossier_ui
  -> ai_offline_assistant
  -> review_queue
  -> export_report
```

## 2. Ma Tran Module

| Module | Dau vao | Xu ly chinh | Dau ra | Gate sang module sau |
| --- | --- | --- | --- | --- |
| ingest_case | folder, original files | tao case, giu thu tu file, hash | case/doc records | co import_sequence |
| split_page | pdf/image | tach page, gan page index | pages | page_count hop le |
| ocr | page image | OCR, line boxes, confidence | ocr_text | text/coor co du |
| region_detect | page, OCR lines | but luc, dau, chu ky | regions | but luc candidate / null |
| classification | ocr_text, layout | type inference | document_type | confidence / review |
| metadata_entity_map | ocr_text, regions | title, ngay, person, evidence, summary | metadata bundle | co title/summary toi thieu |
| citation_index | doc/page/entity | tao citation graph | citation refs | search/AI co the truy vet |
| search_sort | index, filters, sort keys | search, rank, sort | result list | UI co list |
| viewer_dossier_ui | doc/page/results | render list, viewer, dossier | user interaction | review/export/AI |
| ai_offline_assistant | query + retrieved context | answer, summary, dossier reasoning | cited response | citation bat buoc |
| review_queue | low confidence items | approve/edit/reject | final overrides | index duoc cap nhat |
| export_report | case/index/overrides | toc, report, archive | file output | report hoan chinh |

## 3. Gate Du Lieu Giua Cac Module

- `ingest_case` khong duoc bo mat `original_filename` va `source_folder_name`.
- `ocr` khong duoc chi tra text; phai co confidence.
- `classification` khong duoc chi tra nhom; phai co confidence va ly do.
- `metadata_entity_map` phai tao `display_name` va `summary_short`.
- `citation_index` phai tro nguoc duoc ve file/page/but luc.
- `ai_offline_assistant` khong duoc tra loi neu khong co citation.

## 4. Rui Ro Neu Thieu Module

- Thieu `metadata_entity_map`: KSV chi thay filename timestamp.
- Thieu `citation_index`: AI tro thanh hop den khong kiem chung.
- Thieu `review_queue`: loi OCR lan sang tim kiem va dossier.
- Thieu `search_sort`: giao dien dep nhung khong dung duoc nghiep vu.

