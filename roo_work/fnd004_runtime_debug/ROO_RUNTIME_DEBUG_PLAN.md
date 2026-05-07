# ROO_RUNTIME_DEBUG_PLAN

STATUS: PREPARED
CODE TOUCHED: NO

## Scope
Runtime/debug/test observer only for FND-004 page image extraction.

## Read context
- v4/README_V4.md
- v4/00_PROJECT_BRIEF/PROJECT_SUMMARY.md
- v4/03_DESIGN/PAGE_IMAGE_PIPELINE.md
- v4/07_REPORTS/FND_003_RUNTIME_BUNDLE_CODE_REVIEW.md
- PhanMem/scripts/check_e2e_runtime_result.py
- PhanMem/scripts/check_scan_diagram_document.py

## Planned commands
- cd PhanMem && cargo test
- cd PhanMem && cargo test --manifest-path src-tauri\Cargo.toml (fallback because Cargo.toml is under src-tauri)
- cd PhanMem && npx tsc --noEmit
- cd PhanMem && python scripts\check_e2e_runtime_result.py
- cd PhanMem && python scripts\check_scan_diagram_document.py --pdf test_pdfs\test_doc_1.pdf

## Runtime condition
If app UI import cannot be performed by agent, record MANUAL_TAURI_IMPORT_REQUIRED and ask user to import PhanMem/test_pdfs/test_doc_1.pdf via Tauri UI.

## Evidence to collect
- DB path
- pages.image_path exists and file size>0
- pages.extract_status
- PAGE_IMAGE_EXTRACT_DONE/FAILED events
- pages.ocr_status pending/queued count
- checker status output
