# TODAY_RUNTIME_CODE_RESULT

## STATUS: PARTIAL

Runtime code touched: YES

## Files changed

- `PhanMem/src-tauri/src/resource_resolver.rs`

## Commands run

- `cd PhanMem && cargo test` — FAIL: no `Cargo.toml` under `PhanMem/`.
- `cd PhanMem && cargo test --manifest-path src-tauri\Cargo.toml` — PASS, 21 tests passed.
- `cd PhanMem && npx tsc --noEmit` — PASS.
- Embedded Python direct PNG extraction using `PhanMem\resources\runtime\python_embedded\python.exe` + `PhanMem\python` — PASS, generated 2 PNG pages and thumbnails for `PhanMem/test_pdfs/test_doc_1.pdf`.
- Embedded Python direct PNG extraction using runtime-bundle scripts path — PASS, generated 4 PNG pages and thumbnails from the latest imported PDF copy in AppData.
- `cd PhanMem && python scripts\check_e2e_runtime_result.py` — PARTIAL: runtime DB has one successful prior PNG/OCR document and one newer failed import with pending pages.
- `cd PhanMem && python scripts\check_scan_diagram_document.py --pdf test_pdfs\test_doc_1.pdf` — FAIL: checker could not match original fixture path because runtime DB stores copied AppData path.
- `cd PhanMem && python scripts\check_scan_diagram_document.py --document-id doc-1778086172251-2` — FAIL: PNG and OCR checks PASS, diagram classification checks FAIL.

## What was fixed

- Release/local-test Python source resolution now prefers the extracted embedded runtime scripts directory before stale release-side `python/` resources, so `ocr.pdf_to_images` resolves to the bundled runtime copy that supports current page-image arguments.

## What still fails

- Existing runtime DB includes latest document `doc-1778129491670-6` with `PAGE_IMAGE_EXTRACT_FAILED` from before this resolver change; its 4 page rows remain `pending` with no `image_path`.
- `check_e2e_runtime_result.py` reports PARTIAL because those existing pending rows remain in DB and no export package exists.
- `check_scan_diagram_document.py` reports FAIL for the older successful PNG/OCR document because page quality / scene diagram classification was not populated.

## DB evidence

- DB path: `C:\Users\KKIT\AppData\Roaming\com.vks.ecms\vks-ecms.db`.
- Counts from checker: cases=2, documents=2, pages=6, ocr_results=8, document_ai_summaries=1, ai_analysis_jobs=1, governed_events=45.
- Existing successful document: `doc-1778086172251-2`, 2 page rows, OCR terminal as `review_pending`.
- Existing failed/latest document: `doc-1778129491670-6`, 4 page rows, `image_path=NULL`, `extract_status=pending`, `ocr_status=pending`.

## Filesystem evidence

- Direct embedded-Python extraction from fixture generated:
  - `PhanMem/target/runtime_check_pages/page_001.png`
  - `PhanMem/target/runtime_check_pages/page_002.png`
  - `PhanMem/target/runtime_check_thumbs/page_001_thumb.png`
  - `PhanMem/target/runtime_check_thumbs/page_002_thumb.png`
- Direct embedded-Python extraction from latest AppData PDF copy generated 4 PNG pages under `PhanMem/target/runtime_check_pages2/` and 4 thumbnails under `PhanMem/target/runtime_check_thumbs2/`.

## Event evidence

- `IMPORT_STARTED`: 2
- `IMPORT_DONE`: 2
- `IMPORT_FAILED`: 0
- `PAGE_IMAGE_EXTRACT_STARTED`: 2
- `PAGE_IMAGE_EXTRACT_DONE`: 1
- `PAGE_IMAGE_EXTRACT_FAILED`: 1
- `OCR_STARTED`: 2
- `OCR_DONE`: 1
- `OCR_FAILED`: 0
- `AI_DOCUMENT_ANALYSIS_STARTED`: 3
- `AI_DOCUMENT_ANALYSIS_DONE`: 1
- `AI_DOCUMENT_ANALYSIS_FAILED`: 0

## OCR status

- Current DB aggregate: done/review_pending terminal pages=4, pending pages=4.
- OCR source recorded by checker: `stored_page_image`.
- OCR terminal PASS for document `doc-1778086172251-2`.
- OCR terminal FAIL/PARTIAL for whole DB because document `doc-1778129491670-6` remains pending from a previous failed extraction.

## Viewer status

- MANUAL TAURI UI VERIFICATION REQUIRED.
- Viewer was not edited in this pass because the requested scope said do not edit Viewer until PNG generation is OK.

## Export status

- NOT TESTED in Tauri UI.
- Checker found no export package under AppData export directories.

## NOT_DONE

- Did not reset AppData.
- Did not run live `npm run tauri:dev` UI import after resolver patch.
- Did not visually open Viewer.
- Did not run Export through UI.
- Did not fix page quality / diagram classifier output.

## NEXT_ACTION

- Reset AppData or purge stale failed import, then run `npm run tauri:dev` and import `PhanMem/test_pdfs/test_doc_1.pdf` again.
- Rerun `python scripts/check_e2e_runtime_result.py` and require `PAGE_IMAGE_EXTRACT_DONE > 0`, `PAGE_IMAGE_EXTRACT_FAILED = 0`, no pending/queued OCR rows.
- If PNG/OCR pass, verify Viewer manually and run Export.
