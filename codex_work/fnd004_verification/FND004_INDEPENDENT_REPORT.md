CODEX FND-004 VERIFY STATUS: PARTIAL

RUNTIME CODE TOUCHED: NO

CHECKERS RUN:
- `cd PhanMem && python scripts/check_e2e_runtime_result.py`
  - Exit code: 0
  - Result: `>>> STATUS: PARTIAL (pages.ocr_status chưa terminal — còn 4 trang queued/pending)`
- `cd PhanMem && python scripts/check_scan_diagram_document.py --pdf test_pdfs/test_doc_1.pdf`
  - Exit code: 1
  - Result: `[FAIL] Document lookup failed: pdf=D:\JOBS\VKS-HoSoDienTu\PhanMem\test_pdfs\test_doc_1.pdf not found in imported documents`

DB EVIDENCE:
- DB path: `C:\Users\KKIT\AppData\Roaming\com.vks.ecms\vks-ecms.db`
- DB exists: YES
- DB size from checker: 811008 bytes
- Counts: `cases=2`, `documents=2`, `pages=6`, `ocr_results=8`, `document_ai_summaries=1`, `ai_analysis_jobs=1`, `governed_events=45`
- Older document `doc-1778086172251-2`: 2 pages, 2 image paths, 2 extracted rows, 2 OCR terminal rows.
- Latest document `doc-1778129491670-6`: 4 pages, 0 image paths, 0 extracted rows, 4 nonterminal OCR rows.

PNG EVIDENCE:
- `doc-1778086172251-2` page 1 PNG exists: YES, size 244981 bytes.
- `doc-1778086172251-2` page 2 PNG exists: YES, size 11449319 bytes.
- Latest document `doc-1778129491670-6` PNG evidence: NO, all four rows have `image_path=NULL`.
- Aggregate `PAGE_IMAGE_EXTRACT_DONE`: 1.
- Aggregate `PAGE_IMAGE_EXTRACT_FAILED`: 1.

OCR TERMINAL:
- `doc-1778086172251-2`: OCR terminal YES, statuses are `review_pending`.
- `doc-1778129491670-6`: OCR terminal NO, four pages are `pending`.
- Aggregate checker reported OCR source: `stored_page_image`.
- Aggregate pending/nonterminal rows prevent PASS.

MISMATCH WITH SONNET REPORT:
- Main finding matches Sonnet report: DB contains one older successful PNG/OCR document and one newer failed/pending document.
- Independent raw DB query shows actual OCR terminal rows are 2 and pending rows are 4. The Sonnet/TODAY wording `done/review_pending terminal pages=4, pending pages=4` appears double-counted because the checker print label includes `review_pending` inside its `done` display and also prints `review_pending` separately.
- Sonnet report says `check_scan_diagram_document.py --pdf test_pdfs/test_doc_1.pdf` fails due fixture path mismatch; Codex reproduced the same failure.

NOT_DONE:
- No live Tauri UI import was performed by Codex.
- No DB cleanup, purge, or reset was performed.
- No runtime code was changed.
- No v4 task/status/report file was modified.
- PASS was not claimed because latest DB state still contains pending extraction/OCR rows and the fixture-path checker fails.

NEXT_ACTION:
- Run `cd PhanMem && npm run tauri:dev`.
- Import `PhanMem/test_pdfs/test_doc_1.pdf` through the Tauri UI after Sonnet's runtime fix is present.
- Rerun both checkers.
- PASS requires target document lookup success, all target page PNGs present and nonzero, `PAGE_IMAGE_EXTRACT_DONE > 0`, and no target page pending/queued/running.

