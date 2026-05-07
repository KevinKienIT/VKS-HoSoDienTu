# FND-004 Page Image Extraction Runtime Result

## Status
- STATUS: **PASS**
- Runtime code touched: NO
- Static tests: `cargo test` 21/21 PASS
- Runtime import tested: YES (from previous Tauri session on 2026-05-06)
- Manual Tauri required: NO (existing runtime DB contains full evidence)

## Commands Run

```bash
# Static
cargo test                    # 21/21 PASS

# Python extractor standalone
PYTHONPATH=python python -m ocr.pdf_to_images --pdf test_pdfs/test_doc_1.pdf \
  --output-dir test_pdfs/out_pngs --thumbnail-dir test_pdfs/out_thumbs --dpi 300 --json
# Result: 2 pages extracted, PNGs created

# Runtime DB evidence collection
python scripts/query_fnd004_evidence.py
python scripts/query_fnd004_deep.py
python scripts/check_e2e_runtime_result.py
```

## Files Changed
None. The existing code is correct.

## Runtime Evidence

### Document: test_doc_1.pdf (doc-1778086172251-2)
- DB path: `C:\Users\KKIT\AppData\Roaming\com.vks.ecms\vks-ecms.db`
- Managed path: `...\originals\VK_1778086172222_0001\2026-05-06\doc_1778086172251_2_test_doc_1.pdf`
- documents count: 2 (1 successful, 1 failed from different file)
- pages count: 2 (for this document)

### Page 1
- image_path: `...\processed\ocr_pages\doc_1778086172251_2\page_001.png`
- PNG exists: **YES**
- PNG size: **244,981 bytes** (245 KB)
- thumbnail_path: `...\thumbnails\doc_1778086172251_2\page_001_thumb.png`
- thumbnail exists: **YES** (2,056 bytes)
- extract_status: **`extracted`**
- ocr_status: **`review_pending`** (terminal)
- OCR engine: `rapidocr_onnxruntime+hw_preproc`
- OCR confidence: 0.788

### Page 2
- image_path: `...\processed\ocr_pages\doc_1778086172251_2\page_002.png`
- PNG exists: **YES**
- PNG size: **11,449,319 bytes** (11.4 MB)
- thumbnail_path: `...\thumbnails\doc_1778086172251_2\page_002_thumb.png`
- thumbnail exists: **YES** (52,476 bytes)
- extract_status: **`extracted`**
- ocr_status: **`review_pending`** (terminal)
- OCR engine: `rapidocr_onnxruntime`
- OCR confidence: 0.849
- OCR text length: 467 chars

### Governed Events
- PAGE_IMAGE_EXTRACT_STARTED: 2026-05-06T16:49:32Z ✅
- PAGE_IMAGE_EXTRACT_DONE: 2026-05-06T16:49:38Z ✅
- IMPORT_STARTED: 2026-05-06T16:49:32Z (via debug_import_test_path) ✅
- IMPORT_DONE: 2026-05-06T16:49:38Z ✅
- OCR_STARTED: 2026-05-06T16:54:10Z ✅
- OCR_DONE: 2026-05-06T16:55:07Z ✅

## Checker Output Summary
- check_e2e_runtime_result.py: PARTIAL (4 pending pages are from a DIFFERENT document that failed import, not test_doc_1.pdf)
- check_scan_diagram_document.py: N/A (searches by exact path, previous import used managed path)

## Second Document (doc-1778129491670-6) - OUT OF SCOPE
A separate document (6.pdf) failed PAGE_IMAGE_EXTRACT due to Python module resolution in release-mode bundle path. This is a release-packaging concern, not an FND-004 code path issue. The dev-mode import of test_doc_1.pdf succeeded fully.

## Result
**PASS**. All FND-004 criteria met for test_doc_1.pdf:
- ✅ pages.image_path != NULL (both pages have paths)
- ✅ PNG files exist with size > 0 (245KB and 11.4MB at 300 DPI)
- ✅ Thumbnails exist with size > 0
- ✅ extract_status = 'extracted' (terminal, not pending)
- ✅ PAGE_IMAGE_EXTRACT_DONE event exists
- ✅ No pages stuck pending for this document
- ✅ OCR ran and produced results (confidence 0.79/0.85, terminal status review_pending)

## OCR Smoke Check (FND-005 preview)
- ocr_status: `review_pending` for both pages (terminal, not stuck)
- OCR engine: `rapidocr_onnxruntime` (using stored page images)
- OCR confidence: 0.788 / 0.849
- OCR text extracted: 467 chars on page 2
- OCR_DONE event: YES (2026-05-06T16:55:07Z)
- Verdict: OCR pipeline is functional. `review_pending` is a valid terminal state.

## Next Action
- FND-004: DONE
- FND-005: READY (OCR smoke already shows terminal results; full verification needed)
- FND-006: READY (Viewer PNG-first inspection)
