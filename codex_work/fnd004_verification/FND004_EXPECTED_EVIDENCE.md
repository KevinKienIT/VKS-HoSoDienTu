# FND-004 Expected Evidence

## Required Evidence For PASS

1. DB path
   - Expected: `C:\Users\KKIT\AppData\Roaming\com.vks.ecms\vks-ecms.db`
   - Must exist and match backend startup/import log `db_path`.

2. documents count > 0
   - `SELECT COUNT(*) FROM documents;`

3. pages count > 0
   - `SELECT COUNT(*) FROM pages;`

4. `pages.image_path != NULL`
   - Every imported PDF page for the target test document must have a non-empty `image_path`.

5. PNG file exists
   - Each `pages.image_path` must resolve to an existing `.png` file.

6. PNG size > 0
   - Each PNG must have nonzero file size.

7. `extract_status` extracted/terminal
   - Expected success: `extract_status='extracted'`.
   - Terminal failure is evidence only if paired with clear failure event and no queued/pending rows for the checked document.

8. `governed_events` contains `PAGE_IMAGE_EXTRACT_DONE` or equivalent
   - Expected success event: `PAGE_IMAGE_EXTRACT_DONE`.
   - Failure event must be explicit: `PAGE_IMAGE_EXTRACT_FAILED`.

9. OCR status terminal or clear failure
   - Terminal statuses: `done`, `review_pending`, `error`, `skipped`.
   - OCR event evidence: `OCR_DONE` or `OCR_FAILED`.

10. No pending/queued
   - No target pages with `ocr_status IN ('pending','queued','running')`.
   - No target pages with `extract_status IN ('pending','extracting')`.

## Current Independent Runtime Snapshot

DB path:
- `C:\Users\KKIT\AppData\Roaming\com.vks.ecms\vks-ecms.db`
- Exists: YES
- Size from checker: 811008 bytes

Aggregate DB counts from checker:
- `cases`: 2
- `documents`: 2
- `pages`: 6
- `ocr_results`: 8
- `document_ai_summaries`: 1
- `ai_analysis_jobs`: 1
- `governed_events`: 45

Document-level evidence:
- `doc-1778086172251-2`: 2 pages, 2 `image_path`, 2 extracted, 2 terminal OCR.
- `doc-1778129491670-6`: 4 pages, 0 `image_path`, 0 extracted, 4 nonterminal OCR.

PNG evidence found:
- `page_001.png`: exists, 244981 bytes.
- `page_002.png`: exists, 11449319 bytes.
- Both belong to older document `doc-1778086172251-2`.

Missing/pending evidence:
- Latest document `doc-1778129491670-6` has four page rows with `image_path=NULL`.
- Latest document has `extract_status='pending'` and `ocr_status='pending'`.
- Checker for `--pdf test_pdfs/test_doc_1.pdf` failed document lookup.

Current conclusion:
- Evidence is PARTIAL.
- PASS evidence is not ready until a fresh Tauri import produces a target document with no pending/queued rows.

