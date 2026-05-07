# FND-AllPDF Extraction Plan

## Status

- Task type: planning only.
- Runtime code touched: NO.
- Commit code only after PASS: REQUIRED.
- Grapuco status: not callable from this terminal session; fallback code graph was built with `rg` and direct file reads.

## Sources Read

- `v4/03_DESIGN/PAGE_IMAGE_PIPELINE.md`
- `v4/07_REPORTS/FND_004_PAGE_IMAGE_EXTRACT.md` - not present in this workspace.
- `v4/07_REPORTS/FND_004_PAGE_IMAGE_EXTRACTION_RUNTIME_RESULT.md`
- `roo_work/allpdf_extraction_runtime_report.md`
- `PhanMem/src-tauri/src/commands/import_cmd.rs`
- `PhanMem/src-tauri/src/commands/doc_cmd.rs`
- `PhanMem/python/ocr/pdf_to_images.py`
- `PhanMem/src-tauri/src/storage.rs`
- `PhanMem/src-tauri/src/resource_resolver.rs`
- `PhanMem/src-tauri/migrations/001_init_schema.sql`
- `PhanMem/src-tauri/migrations/009_page_ocr_scheduler.sql`
- `PhanMem/src-tauri/migrations/014_page_image_architecture.sql`

## Current Extract Flow

Expected design:

```text
Import PDF
  -> count PDF pages into documents.page_count
  -> copy immutable source PDF into originals/<case>/<date>/
  -> run_page_image_extraction()
  -> Python module ocr.pdf_to_images renders 300 DPI PNG + thumbnail per page
  -> Rust writes pages.image_path, thumbnail_path, source_pdf_path, source_page_number
  -> pages.extract_status='extracted', pages.ocr_status='queued'
  -> OCR reads stored PNG, not original PDF
  -> Viewer/export use PNG only
```

Observed code path:

- `import_cmd.rs::try_count_pdf_pages()` / `count_pdf_pages()` uses `lopdf` to set `documents.page_count`.
- `storage::copy_to_originals()` writes the immutable PDF copy.
- `import_cmd.rs::run_page_image_extraction()` builds Python args:

```text
-m ocr.pdf_to_images
--pdf <managed_pdf_path>
--output-dir <processed/ocr_pages/{document_id}>
--thumbnail-dir <thumbnails/{document_id}>
--dpi 300
--json
```

- `doc_cmd.rs::run_python_json_for_import()` calls `run_python_json_with_timeout()`.
- `run_python_json_with_timeout()` resolves embedded Python, sets `PYTHONPATH=python_dir()`, rewrites `-m <module>` into a `runpy.run_module()` command, waits up to `PYTHON_TIMEOUT_SECS = 600`, parses JSON stdout, and returns `OCR_PYTHON_FAILED` on non-zero exit.
- `pdf_to_images.py::extract_pages()` uses PyMuPDF (`fitz`) and supports full-document extract plus `--page N` single-page extract.
- `insert_page_image_row()` inserts the successful page image row with `extract_status='extracted'`.
- If `run_page_image_extraction()` fails, current import fallback creates placeholder page rows with `image_path=NULL` and default `extract_status='pending'`.

Related DB design:

- `pages(document_id, page_index)` is unique via `idx_page_doc_idx`.
- `pages.extract_status` allows `pending`, `extracting`, `extracted`, `error`.
- Migration 014 adds `source_pdf_path`, `source_page_number`, `thumbnail_path`, `current_order`.

## Failure From All-PDF Runtime Test

Runtime test input:

- Folder: `d:\JOBS\VKS-HoSoDienTu\PhanMem\test_pdfs\allpdf_runtime`
- Inventory: 80 PDF files and 3 non-PDF files.
- Runtime DB: `%APPDATA%\com.vks.ecms\vks-ecms.db`

FAIL evidence:

- UI appeared to complete import without visible error.
- Runtime DB showed latest job with only `total_files=1`, `current_file=6.pdf`, not the full 80-PDF folder.
- Latest document `doc-1778129491670-6` had `page_count=4`.
- Pages 1..4 existed only as placeholders:

```text
page 1: extract_status=pending, image_path=NULL, ocr_status=error
page 2: extract_status=pending, image_path=NULL, ocr_status=error
page 3: extract_status=pending, image_path=NULL, ocr_status=error
page 4: extract_status=pending, image_path=NULL, ocr_status=error
```

Root failure event:

```text
event_type: OCR_FAILED
source: doc_cmd.run_ocr_for_document_scope_with_conn
first_error: OCR_PYTHON_FAILED ... Error while finding module specification for 'ocr.pdf_to_images'
             (ModuleNotFoundError: No module named 'ocr')
```

Additional startup gate evidence:

```text
STARTUP_CHECK_FAILED:
- not enough disk for dossier storage
- offline payload invalid/checksum mismatch/extract failed
- OCR model det/rec/cls invalid or checksum mismatch; OCR blocked
```

Conclusion:

- The page extractor itself can pass for small fixture `test_doc_1.pdf` when `PYTHONPATH=python` is correct.
- The all-PDF runtime failure points to runtime bundle/Python path resolution: embedded Python could start, but `ocr.pdf_to_images` was not importable.
- After module failure, placeholder pages remained `pending`; no validation gate rejected the incomplete document as an extract failure with a clear user-facing message.

## Risk Points Found

1. `run_page_image_extraction()` only returns error when zero pages are inserted. It does not compare inserted PNG count to `documents.page_count`.

2. Full-document extraction is all-or-nothing. If one PDF is large or Python times out, there is no per-page retry despite `pdf_to_images.py` already supporting `--page N`.

3. Failed extract writes placeholder pages but leaves `extract_status='pending'`, which looks like unfinished work instead of a concrete extract failure.

4. OCR has a second extract path in `doc_cmd.rs::extract_pdf_pages_for_document_with_conn()` / `run_ocr_for_document_scope_with_conn()`. This can hide import extract failures and creates two places to maintain status logic.

5. Runtime Python path resolution has multiple sources (`resolve_python_source_dir`, `resolve_python_scripts_dir`, debug fallback). The failed event shows the selected runtime path did not expose the `ocr` package to Python.

6. There is no committed all-PDF verifier in repo in the clean branch. The runtime report used `PhanMem/scripts/verify_allpdf_extraction.py`, but that script is currently untracked in the dirty workspace.

## Proposed Fix

### 1. Fix Python module path deterministically

Files:

- `PhanMem/src-tauri/src/commands/doc_cmd.rs`
- `PhanMem/src-tauri/src/resource_resolver.rs`
- `PhanMem/scripts/prepare_offline_bundle.py`
- runtime payload manifest/config files if needed

Expected diff:

- Ensure `python_dir()` always returns a directory that contains `ocr/pdf_to_images.py` in dev and packaged runtime.
- When `resolve_python_scripts_dir()` returns the embedded scripts dir, set `PYTHONPATH` to include both:

```text
<scripts_dir>
<scripts_dir parent if needed>
```

- Add a preflight check before import extract:

```text
python -c "import ocr.pdf_to_images"
```

using the same embedded Python and `PYTHONPATH` that production uses.

- Improve `OCR_ENV_SCRIPT_MISSING` / `OCR_PYTHON_FAILED` event payload with:

```text
python_path
python_dir
script_path
module_name
runtime_bundle_dir
resource_roots
PYTHONPATH
```

Do not fall back to system Python.

### 2. Validate `page_count` after every extract

Files:

- `PhanMem/src-tauri/src/commands/import_cmd.rs`
- optionally shared helper in `doc_cmd.rs` or a new internal module

Expected diff:

- After Python returns JSON, compare:

```text
expected_pages = documents.page_count
json_pages = extracted["pages"].len()
db_extracted_pages = COUNT(pages WHERE document_id=? AND extract_status='extracted' AND image_path IS NOT NULL)
file_verified_pages = number of image_path files that exist and size > 0
```

- `PAGE_IMAGE_EXTRACT_DONE` is emitted only if all four numbers match.
- If count is short, emit `PAGE_IMAGE_EXTRACT_INCOMPLETE` and retry missing pages.
- Use `INSERT OR IGNORE` + `UPDATE` by `(document_id, page_index)`; do not create duplicate page rows.

### 3. Check and persist `extract_status` per page

Files:

- `PhanMem/src-tauri/src/commands/import_cmd.rs`
- `PhanMem/src-tauri/migrations/016_allpdf_extract_integrity.sql` if extra columns are accepted

Expected diff:

- Before full extract, ensure page rows 1..`page_count` exist and set them to `extract_status='extracting'`.
- For each successful page, set:

```text
image_path=<png>
thumbnail_path=<thumb>
source_pdf_path=<pdf>
source_page_number=<page>
current_order=COALESCE(current_order, page)
extract_status='extracted'
ocr_status=CASE WHEN ocr_status='pending' THEN 'queued' ELSE ocr_status END
```

- For failed/missing pages after retry, set:

```text
extract_status='error'
ocr_status='error'
ocr_error='PAGE_IMAGE_EXTRACT_FAILED'
```

### 4. Retry missing pages

Files:

- `PhanMem/src-tauri/src/commands/import_cmd.rs`
- `PhanMem/python/ocr/pdf_to_images.py`

Expected diff:

- Use existing `--page N` option for missing pages.
- Retry policy:

```text
full extract at 300 DPI
if missing pages or timeout:
  retry each missing page at 300 DPI
  if still missing:
    retry each missing page at 200 DPI
```

- For large files, choose per-page extraction from the start:

```text
page_count > 50 OR file_size > 100 MB
```

- Python should include in JSON:

```json
{
  "source_page_count": 4,
  "extracted_page_count": 4,
  "errors": [],
  "duration_ms": 1234
}
```

### 5. Add committed verifier script

File:

- `PhanMem/scripts/verify_allpdf_extraction.py`

Expected diff:

- Commit a stable verifier script with:

```text
--db <path> optional
--case-id <case> optional
--folder <folder> optional
--json-out <path> optional
```

- Checks:

```text
documents.page_count == COUNT(pages)
documents.page_count == COUNT(extracted pages with valid PNG files)
all extract_status values are extracted
all image_path files exist and size > 0
```

- Natural-sort PDF filenames for folder matching (`1.pdf`, `2.pdf`, `10.pdf`).
- Exit 0 only on full PASS; exit 1 on any missing page or bad path.

## Logging Requirements

Events:

- `PAGE_IMAGE_EXTRACT_STARTED`
- `PAGE_IMAGE_EXTRACT_INCOMPLETE`
- `PAGE_IMAGE_EXTRACT_RETRY_STARTED`
- `PAGE_IMAGE_EXTRACT_PAGE_FAILED`
- `PAGE_IMAGE_EXTRACT_DONE`
- `PAGE_IMAGE_EXTRACT_FAILED`

Required event fields:

```text
document_id
source_pdf_path
expected_page_count
extracted_page_count
missing_page_numbers
bad_image_paths
python_path
python_dir
module_name
script_path
runtime_bundle_dir
strategy
dpi
retry_attempt
error_code
error_message
```

Required human-readable messages:

```text
Lỗi import PDF: thiếu trang do module Python ocr.pdf_to_images không tìm thấy. Kiểm tra PYTHONPATH/runtime bundle trước khi retry.
Lỗi import PDF: thiếu trang do timeout khi render PNG; expected=<n>, extracted=<m>, missing=<pages>.
Lỗi import PDF: PNG đã ghi vào DB nhưng file không tồn tại hoặc size=0; kiểm tra quyền ghi processed/ocr_pages.
```

## Files To Change In Implementation

Code files:

- `PhanMem/src-tauri/src/commands/import_cmd.rs`
- `PhanMem/src-tauri/src/commands/doc_cmd.rs`
- `PhanMem/src-tauri/src/resource_resolver.rs`
- `PhanMem/python/ocr/pdf_to_images.py`
- `PhanMem/scripts/prepare_offline_bundle.py`
- `PhanMem/scripts/verify_allpdf_extraction.py` (new tracked script)

Migration if accepted:

- `PhanMem/src-tauri/migrations/016_allpdf_extract_integrity.sql`
- `PhanMem/src-tauri/src/db/mod.rs`
- `PhanMem/src-tauri/src/db/schema.rs`

Suggested DB additions:

```sql
ALTER TABLE documents ADD COLUMN expected_page_count INTEGER;
ALTER TABLE documents ADD COLUMN page_image_status TEXT NOT NULL DEFAULT 'pending'
  CHECK (page_image_status IN ('pending','extracting','complete','partial','error'));
ALTER TABLE documents ADD COLUMN page_image_error TEXT;
ALTER TABLE documents ADD COLUMN page_image_attempts INTEGER NOT NULL DEFAULT 0;

ALTER TABLE pages ADD COLUMN extract_error TEXT;
ALTER TABLE pages ADD COLUMN extract_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE pages ADD COLUMN extracted_at TEXT;
```

## Test Gates

Static:

```powershell
cd PhanMem\src-tauri
cargo check -j 1
cargo test

cd PhanMem
npm exec tsc -- --noEmit
```

Runtime:

```powershell
cd PhanMem && npm run tauri:dev
```

Runtime acceptance:

1. Startup gate must not report invalid offline payload or invalid OCR model checksums.
2. Python preflight must import `ocr.pdf_to_images` using embedded runtime.
3. Import one known fixture (`test_doc_1.pdf`) and verify 2/2 PNG pages.
4. Import all PDFs in `PhanMem/test_pdfs/allpdf_runtime`.
5. Run:

```powershell
cd PhanMem
python scripts\verify_allpdf_extraction.py --folder test_pdfs\allpdf_runtime --json-out ..\roo_work\allpdf_extraction_summary.json
```

6. PASS requires:

```text
all PDF documents found in DB
documents.page_count == pages row count
documents.page_count == extracted PNG count
no pages stuck pending/extracting/error
all PNG files exist and size > 0
PAGE_IMAGE_EXTRACT_DONE exists only for complete documents
```

## Commit Rule

Do not commit implementation code until the test gates above pass.

Allowed before PASS:

- planning reports
- verifier script proposal only if separately requested

Not allowed before PASS:

- runtime Rust changes
- Python extractor changes
- migration changes
- package/runtime bundle changes

If any implementation attempt fails, write the failure into `v4/07_REPORTS/` and leave code uncommitted or reverted by the implementer. This report intentionally commits no runtime code.
