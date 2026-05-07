# ACTIVE TASKS — VKS ECMS V4

> Chỉ chứa task CHƯA LÀM. Task đã DONE nằm trong COMPLETED_TASKS_ARCHIVE.md.

---

## Foundation Tasks

| ID | Task | Status | Owner | Scope | Acceptance Criteria | Test Command | Evidence |
|---|---|---|---|---|---|---|---|
| FND-001 | Confirm V4 as source of truth | ✅ DONE | Antigravity | Docs | V4 directory exists with all required files | `Test-Path v4/README_V4.md` | File exists |
| FND-002 | Verify DB schema target | ✅ VERIFIED | Claude Sonnet | Schema | M001-M015 confirmed. pages (35 cols) complete. No migration needed for pages. | See FND_002_DB_SCHEMA_CODE_REVIEW.md | v4/07_REPORTS/FND_002_DB_SCHEMA_CODE_REVIEW.md |
| FND-003 | Verify runtime bundle and embedded Python | ✅ DONE | Agent | Infra | `resource_resolver::resolve_python_binary()` works, Python scripts found | `npm run tauri:dev` + startup log | v4/07_REPORTS/FND_003_RUNTIME_BUNDLE_CODE_REVIEW.md |
| FND-004 | Verify page image extraction | ✅ PASS | Agent | Pipeline | PNG 300DPI OK, extract_status=extracted, image_path set, PAGE_IMAGE_EXTRACT_DONE event confirmed | DB query + filesystem check | v4/07_REPORTS/FND_004_PAGE_IMAGE_EXTRACTION_RUNTIME_RESULT.md |
| FND-005 | Verify OCR from stored PNG | ✅ PASS | Agent | Pipeline | Target doc OCR terminal (review_pending), OCR_DONE emitted, ocr_source=stored_page_image | DB query + checker | v4/07_REPORTS/FND_005_OCR_TERMINAL_RUNTIME_RESULT.md |
| FND-006 | Verify Viewer PNG-first | 🔲 NEEDS_INSPECTION | Agent | UI | Read `TrinhXemTaiLieu.tsx` first before any edit | Read viewer file | v4/07_REPORTS/FND_006_VIEWER_CODE_REVIEW.md |
| FND-007 | Group folder and natural sort | ✅ PASS | Codex GPT-5.5 | Pipeline+Schema | M016 created, document_groups table, documents.group_id populated | `cargo test --manifest-path src-tauri\Cargo.toml -j 1` | v4/07_REPORTS/FND_007_DOCUMENT_GROUPS_IMPLEMENTATION_RESULT.md |
| FND-008 | Verify export package and manifest | 🔲 READY_FOR_REVIEW | Agent | Pipeline | `export_pdf_bundle` uses `pages.current_order` not original PDF | Export test + review `export_cmd.rs` | v4/07_REPORTS/FND_008_EXPORT_CODE_REVIEW.md |
| FND-009 | Run E2E runtime gate | 🔲 BLOCKED_UNTIL_CORE | Agent | Verification | Import → OCR → AI → Export verified end-to-end | `check_e2e_runtime_result.py` | Script output: PASS |


## Documentation Tasks

| ID | Task | Status | Owner | Scope | Acceptance Criteria |
|---|---|---|---|---|---|
| DOC-001 | Review V4 docs quality | 🔲 TODO | Human | Review | All V4 docs accurate and complete |
| DOC-002 | Confirm old v3 docs to archive | 🔲 TODO | Human | Review | DEPRECATED_DOCS_PLAN reviewed |
| DOC-003 | Human approval before moving old docs | 🔲 TODO | Human | Decision | Confirmed which files to move/delete |

## Docling Tasks

| ID | Task | Status | Owner | Scope | Acceptance Criteria |
|---|---|---|---|---|---|
| DOC-L-001 | Install Docling in isolated dev venv | 🔲 TODO | Developer | Setup | `pip install docling` in `.venv-docling` |
| DOC-L-002 | Run license check | 🔲 TODO | Developer | Legal | License compatible with government software |
| DOC-L-003 | Prepare Vietnamese PDF fixtures | 🔲 TODO | Developer | Test | PDF with tables, diacritics, scanned pages |
| DOC-L-004 | Run Docling benchmark | 🔲 TODO | Developer | Benchmark | 7 metrics compared |
| DOC-L-005 | Decide Docling stays docs-only or optional feature | 🔲 TODO | Human | Decision | Decision documented |
