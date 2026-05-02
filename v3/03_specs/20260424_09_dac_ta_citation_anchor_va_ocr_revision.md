# Dac Ta Citation Anchor Va OCR Revision

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Lam cho citation ben vung khi OCR bi sua, re-run hoac khi KSV can nhay dung vi tri tren tai lieu.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`

## Muc Luc

1. Nguyen tac anchor
2. Cau truc OCR revision
3. Cau truc citation anchor
4. Luat cap nhat khi OCR thay doi
5. Hien thi UI va hanh vi nhay den nguon
6. Acceptance criteria

## 1. Nguyen Tac Anchor

Citation khong duoc chi dua vao:

- `page_index`,
- `quote_excerpt`,
- `source_but_luc`.

Citation phai co anchor ben vung o muc revision OCR, neu khong thi sau khi reviewer sua text, link nhay va highlight se bi lech.

## 2. Cau Truc OCR Revision

Moi lan OCR/re-OCR/override text phai tao revision:

- `ocr_revision_id`
- `page_id`
- `revision_no`
- `revision_source` (`engine`, `manual_override`, `reprocess`)
- `parent_revision_id`
- `full_text`
- `avg_confidence`
- `transcription_state`
- `has_uncertain_spans`
- `candidate_bundle_id`
- `created_at`
- `created_by`

Page luon co:

- `active_revision_id`
- `latest_revision_no`

## 3. Cau Truc Citation Anchor

Moi citation phai co:

- `citation_id`
- `source_document_id`
- `source_page_id`
- `source_but_luc`
- `ocr_revision_id`
- `anchor_type` (`line_range`, `char_range`, `region_bbox`, `hybrid`)
- `anchor_start`
- `anchor_end`
- `line_ids`
- `region_bbox`
- `quote_excerpt`
- `quote_hash`
- `confidence`
- `transcription_state`

Khuyen nghi phase 1:

- dung `hybrid anchor`:
  - `ocr_revision_id`,
  - `line_ids`,
  - `char_range`,
  - `quote_hash`.

## 4. Luat Cap Nhat Khi OCR Thay Doi

Neu page co revision moi:

1. Khong xoa citation cu.
2. Kiem tra citation cu co con map duoc sang revision moi khong.
3. Neu map duoc -> tao `rebased_anchor`.
4. Neu khong map duoc -> danh dau `citation_status = stale`.
5. UI phai canh bao citation stale, khong im lang dung citation cu.

Neu user sua OCR bang tay:

- revision moi phai duoc tao truoc,
- citation phu thuoc phai duoc re-validate,
- AI answer cu khong duoc coi la con hoan toan hop le cho toi khi citation duoc check lai.

Neu citation tro toi mot doan text viet tay mo hoac da qua noi suy:

- `region_bbox` la bat buoc,
- `transcription_state` phai hien ro (`candidate_only`, `interpolated_pending_review`, `approved_manual`),
- citation chua duoc coi la "on dinh nghiep vu" neu van con `pending_review`.

## 5. Hien Thi UI Va Hanh Vi Nhay Den Nguon

Khi user click citation:

- mo dung file,
- nhay den dung trang,
- cuon den dung vung line/region,
- highlight theo anchor hien hanh,
- neu citation stale thi hien canh bao truoc khi nhay.

Citation panel phai hien:

- file,
- trang,
- but luc,
- revision no,
- trich doan,
- confidence,
- transcription state,
- trang thai `active` / `rebased` / `stale`.

## 6. Acceptance Criteria

- Citation van dung duoc sau manual OCR correction.
- Citation stale duoc canh bao ro.
- KSV nhay duoc den dung vi tri tren viewer.
- AI output luon gan voi `ocr_revision_id` cu the.
- Khong co feature report/dossier/AI nao chi luu `quote_excerpt` ma khong luu anchor.
- Citation tren doan viet tay mo van giu duoc `region_bbox`, `ocr_revision_id` va `transcription_state`.
