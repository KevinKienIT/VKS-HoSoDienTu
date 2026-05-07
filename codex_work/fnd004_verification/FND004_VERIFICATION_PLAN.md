# FND-004 Verification Plan

## Scope

Verify Page Image Extraction from runtime evidence only:
- Runtime DB rows created by Tauri import.
- `pages.image_path` populated with PNG paths.
- PNG files exist on disk and have nonzero size.
- Page extraction emits terminal governed events.
- OCR reaches terminal status or records clear failure.

Codex constraints:
- Runtime code touched: NO.
- No edits under `PhanMem/src`, `PhanMem/src-tauri`, `PhanMem/scripts`, or `v4`.
- No file deletion.
- Only files under `codex_work/fnd004_verification/` are created.

## Read-Only Inputs Checked

- `v4/README_V4.md`
- `v4/00_PROJECT_BRIEF/PROJECT_SUMMARY.md`
- `v4/01_MEMORY/PROJECT_MEMORY.md`
- `v4/02_RULES/AGENT_RULES.md`
- `v4/02_RULES/DO_NOT_DO.md`
- `v4/03_DESIGN/CURRENT_ARCHITECTURE.md`
- `v4/03_DESIGN/PAGE_IMAGE_PIPELINE.md`
- `v4/05_TASKLIST/ACTIVE_TASKS.md`
- `v4/07_REPORTS/FND_003_RUNTIME_BUNDLE_CODE_REVIEW.md`
- `v4/07_REPORTS/FND_004_PAGE_IMAGE_EXTRACTION_RUNTIME_RESULT.md`
- `v4/07_REPORTS/TODAY_RUNTIME_CODE_RESULT.md`
- `PhanMem/scripts/check_e2e_runtime_result.py`
- `PhanMem/scripts/check_scan_diagram_document.py`
- `PhanMem/src-tauri/src/commands/import_cmd.rs`
- `PhanMem/src-tauri/src/commands/doc_cmd.rs`
- `PhanMem/src-tauri/src/storage.rs`

## Runtime Path Model

Backend DB path:
- `app.path().app_data_dir()/vks-ecms.db`
- Current Windows path: `C:\Users\KKIT\AppData\Roaming\com.vks.ecms\vks-ecms.db`

Managed storage root:
- `app_data_dir/VKS_ECMS_Data`

Expected page image paths:
- `C:\Users\KKIT\AppData\Roaming\com.vks.ecms\VKS_ECMS_Data\processed\ocr_pages\{doc_id}\page_001.png`
- `C:\Users\KKIT\AppData\Roaming\com.vks.ecms\VKS_ECMS_Data\thumbnails\{doc_id}\page_001_thumb.png`

## Acceptance Flow

1. Start Tauri runtime with `npm run tauri:dev`.
2. Import `PhanMem/test_pdfs/test_doc_1.pdf` through the app UI.
3. Wait until import/page extraction/OCR is terminal.
4. Run both checker scripts.
5. Confirm expected DB, PNG, governed event, and OCR evidence.

## Current Gate Result

Current DB is not clean enough for PASS:
- It has one older successful PNG extraction document.
- It has one newer failed/pending document with four page rows and no `image_path`.
- Aggregate checker status is PARTIAL.
- Fixture-path checker cannot match `test_pdfs/test_doc_1.pdf` in runtime DB.

