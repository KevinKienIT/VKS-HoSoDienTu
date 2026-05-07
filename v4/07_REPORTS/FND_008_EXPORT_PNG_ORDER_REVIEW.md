# FND-008 Export PNG Order — Code Review and Architecture Proposal

## Task

- Task ID: FND-008
- Title: Verify export package and manifest / PNG-ordered PDF export review
- Agent: Roo Code GPT-5.5
- Date: 2026-05-07
- Scope: Review current export implementation and propose PNG-first ordered export architecture.
- Runtime code touched: NO
- Tests run: NO, review-only task.

## V4 Context Loaded

- v4/README_V4.md
- v4/00_PROJECT_BRIEF/PROJECT_SUMMARY.md
- v4/01_MEMORY/PROJECT_MEMORY.md
- v4/02_RULES/AGENT_RULES.md
- v4/03_DESIGN/CURRENT_ARCHITECTURE.md
- v4/05_TASKLIST/ACTIVE_TASKS.md
- v4/07_REPORTS/FND_004_PAGE_IMAGE_EXTRACTION_RUNTIME_RESULT.md
- v4/07_REPORTS/FND_005_OCR_TERMINAL_RUNTIME_RESULT.md

## Grapuco / Code Graph Status

- GRAPUCO_EXTENSION_UNAVAILABLE_IN_SESSION
- Fallback used: read-only source inspection and regex symbol search.
- Runtime edit approval: NO. This task intentionally stops after report + Git push.

## Runtime Evidence Baseline from FND-004 / FND-005

- FND-004 PASS confirms stored PNG page images exist for target document `doc-1778086172251-2`.
- FND-004 target pages have `pages.image_path`, `thumbnail_path`, `extract_status='extracted'`, and existing files on disk.
- FND-005 PASS confirms OCR reads from stored PNGs: `ocr_source=stored_page_image` and target pages are terminal (`review_pending`).
- Therefore FND-008 can safely design export around `pages.image_path` rather than original PDF bytes.

## Current Export Implementation Findings

### `export_pdf_bundle()` still merges original/managed PDF objects

Location: `PhanMem/src-tauri/src/commands/export_cmd.rs`

Observed flow:

1. Reads `documents.display_name`, `documents.file_path`, `documents.status`, and `documents.file_status` by selected document IDs.
2. Rejects documents whose `file_path` does not end with `.pdf`.
3. Loads source PDFs with `lopdf::Document::load(&file_path)`.
4. Copies page objects from those source PDFs into one output PDF.
5. Adds optional cover page, TOC, page numbers, bookmarks.
6. Emits `export_pdf_bundle` governed event.

Critical gap:

- Does not query the `pages` table.
- Does not use `pages.image_path`.
- Does not use `pages.current_order`.
- Does not filter `pages.is_removed`.
- Does not apply `pages.rotation`.
- Therefore the output cannot reflect reviewed page order/removal/rotation and violates V4 PNG-first export target.

### `export_dossier_package()` copies document files, not reviewed PNG pages

Location: `PhanMem/src-tauri/src/commands/export_cmd.rs`

Observed flow:

1. Reads case documents ordered by `documents.created_at ASC`.
2. Chooses `original_path` or `managed_path` depending on `include_originals`.
3. Copies source files into `03_tai_lieu_quan_ly`.
4. Writes text/JSON support files such as index, citation appendix, audit JSON.

Critical gap:

- Does not package page-level PNG output.
- Does not read reviewed page order from `pages.current_order`.
- Does not include per-page removed/rotation mapping in the audit manifest.

## Code Graph / Caller Context

| Area             | File                                                  | Symbol / Usage             | Finding                                                                                                              |
| ---------------- | ----------------------------------------------------- | -------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Backend export   | `PhanMem/src-tauri/src/commands/export_cmd.rs`        | `export_pdf_bundle()`      | Current PDF merge uses `documents.file_path` and `lopdf::Document::load`; no page table query.                       |
| Backend package  | `PhanMem/src-tauri/src/commands/export_cmd.rs`        | `export_dossier_package()` | Copies whole managed/original files; does not build reviewed PNG PDF.                                                |
| Frontend service | `PhanMem/src/services/exportService.ts`               | `exportPdfBundle()`        | Invokes Tauri command `export_pdf_bundle`.                                                                           |
| Business UI      | `PhanMem/src/modules/xuatbangiao/XuatBanGiaoPage.tsx` | `runExport()`              | Sends selected document IDs and options; UI can remain mostly stable.                                                |
| DB page source   | `PhanMem/src-tauri/src/commands/import_cmd.rs`        | page-image insert          | Inserts `image_path`, `thumbnail_path`, `current_order`, `rotation=0`, `is_removed=0`, `extract_status='extracted'`. |
| OCR source proof | `PhanMem/src-tauri/src/commands/doc_cmd.rs`           | OCR from stored page image | Confirms existing runtime code already consumes `pages.image_path`.                                                  |

## Required Export Query

The new export path should query page rows, not PDF page objects:

```sql
SELECT
    d.document_id,
    d.display_name,
    p.page_id,
    p.page_index,
    COALESCE(p.current_order, p.page_index) AS export_order,
    COALESCE(p.rotation, 0) AS rotation,
    COALESCE(p.is_removed, 0) AS is_removed,
    p.image_path,
    p.thumbnail_path,
    p.extract_status,
    COALESCE(p.ocr_status, 'pending') AS ocr_status
FROM documents d
JOIN pages p ON p.document_id = d.document_id
WHERE d.document_id IN (...selected ids...)
  AND COALESCE(p.image_path, '') <> ''
  AND COALESCE(p.extract_status, '') = 'extracted'
ORDER BY
    -- preserve user-selected document order outside SQL or via CASE expression,
    d.created_at ASC,
    COALESCE(p.current_order, p.page_index) ASC,
    p.page_index ASC;
```

Filtering rule:

- Export PDF: include only rows where `is_removed = 0`.
- Manifest: include all rows, including removed rows, with `included=false` when `is_removed=1`.

## Proposed PNG-first Export Architecture

### Phase 1 — Page plan

Build an in-memory `ExportPagePlan` list from DB rows:

```text
ExportPagePlan {
  export_page_number,
  document_id,
  document_title,
  page_id,
  original_page_index,
  current_order,
  image_path,
  rotation,
  is_removed,
  included,
  file_exists,
  file_size,
}
```

Validation:

- Missing PNG path or missing file: skip with warning or fail depending on strict mode.
- `extract_status != extracted`: skip with warning.
- `is_removed=1`: do not include in PDF, but record in manifest.
- Rotation must be normalized to one of `0`, `90`, `180`, `270`.

### Phase 2 — Build PDF from PNG

Recommended implementation path: bundled Python script using Pillow/img2pdf-style behavior.

Reasoning:

- Runtime already has embedded Python and image-processing dependencies for OCR/page extraction.
- Python can read PNG dimensions/DPI reliably and rotate raster images before writing PDF.
- Keeps Rust `lopdf` usage away from manual image XObject construction complexity.
- Must not use system Python and must not download packages at runtime.

Implementation option A — Python embedded script:

- Add a script such as `python/export/png_pages_to_pdf.py` or put under runtime bundle scripts.
- Inputs: JSON manifest/plan path and output PDF path.
- Library choices:
  - Prefer existing/bundled dependencies only.
  - If Pillow is already bundled, use `PIL.Image.open`, apply rotation, save multipage PDF.
  - If `img2pdf` is already bundled or can be bundled offline, use it for PDF image embedding; apply rotation through pre-rotated temp images only when needed.
- Output: PDF path and per-page result JSON.

Implementation option B — Rust native:

- Use a Rust PDF/image crate capable of placing PNG/JPEG as PDF pages.
- Risk is higher than Python because current code uses `lopdf` for PDF object merging, not raster-image PDF creation.
- Native implementation must handle DPI, page media box, RGB/alpha conversion, and rotation.

Recommendation:

- Use Python embedded for first FND-008 implementation because it aligns with existing PNG/OCR runtime and has lower PDF-image construction risk.
- Keep original source PDFs immutable and never overwrite originals.

### Phase 3 — Manifest / audit output

Write a JSON manifest next to the exported PDF, e.g. `<output>.manifest.json` or package `05_audit_export.json` extension:

```json
{
  "schema_version": "export_png_order_v1",
  "exported_at": "2026-05-07T00:00:00+07:00",
  "output_pdf": "...",
  "source": "stored_page_images",
  "document_ids": ["..."],
  "pages_total": 2,
  "pages_included": 2,
  "pages_removed": 0,
  "pages": [
    {
      "export_page_number": 1,
      "document_id": "doc-...",
      "page_id": "page-...",
      "original_page_index": 1,
      "current_order": 1,
      "image_path": ".../page_001.png",
      "rotation": 0,
      "is_removed": false,
      "included": true,
      "sha256": "optional-image-hash"
    }
  ],
  "removed_pages": [],
  "warnings": []
}
```

Governed event payload should include:

- `output_path`
- `manifest_path`
- `source="stored_page_images"`
- `pages_included`
- `pages_removed`
- `warning_count`

## Required Behavior Change for `export_pdf_bundle()`

Target behavior:

1. Keep current API shape if possible: selected document IDs + output path + options.
2. Replace source PDF object merge with page plan from `pages` table.
3. Sort pages by selected document order then `COALESCE(current_order, page_index)`.
4. Exclude removed pages from the exported PDF.
5. Apply rotation to included PNG pages.
6. Create PDF from stored PNGs.
7. Write manifest with mapping and removed-page audit.
8. Emit governed event with manifest path and page counts.

Compatibility note:

- If no stored PNG pages are found for a selected PDF, do not silently fall back to source PDF merge for production behavior.
- Return a clear error such as `EXPORT_NO_STORED_PAGE_IMAGES` or warning in an explicit debug/development fallback only.

## Package Export Follow-up

`export_dossier_package()` should later include:

- Main reviewed PDF generated from PNG pages.
- Page manifest JSON.
- Original files only in a clearly labeled originals folder if requested.
- Citation/audit JSON referencing reviewed export page numbers, not original PDF page objects.

## Risks

| Risk                                                    | Severity | Mitigation                                                                                                |
| ------------------------------------------------------- | -------- | --------------------------------------------------------------------------------------------------------- |
| Existing export ignores reviewed order/removal/rotation | HIGH     | Replace PDF merge with page-plan PNG export.                                                              |
| Missing/invalid PNG path                                | MEDIUM   | Validate `image_path`, `extract_status`, file existence before export.                                    |
| Rotation handling varies by PDF library                 | MEDIUM   | Normalize in image layer before PDF write or set PDF page rotation consistently.                          |
| Huge PNG memory usage                                   | MEDIUM   | Stream or process one page at a time; avoid loading all high-DPI pages simultaneously for large dossiers. |
| Python dependency availability                          | MEDIUM   | Use already-bundled Python/libs only; no runtime downloads; verify offline bundle contents.               |
| UI expectations                                         | LOW      | Preserve `exportPdfBundle()` request/response shape initially.                                            |

## Test Gate for Future Implementation

Do not run for this report-only task. For implementation:

```powershell
cd PhanMem
cargo test --manifest-path src-tauri\Cargo.toml
npx tsc --noEmit
npm run tauri:dev
```

Runtime verification:

1. Import fixture with at least 2 pages.
2. Set `pages.current_order` to reverse order for test only.
3. Set one page `is_removed=1` for test only.
4. Set one page `rotation=90` for test only.
5. Export PDF.
6. Verify output page count equals non-removed pages.
7. Verify manifest records original page index, current order, rotation, removed page, and included pages.
8. Verify governed event contains `source=stored_page_images` and `manifest_path`.

## Decision

- REVIEW STATUS: COMPLETE
- APPROVED_TO_EDIT_RUNTIME_CODE: NO
- EXPORT IMPLEMENTATION: NOT DONE
- PUSH STATUS: handled by task branch after this report is committed

## Next Action

After feedback, implement FND-008 by replacing `export_pdf_bundle()` internals with the PNG page-plan exporter and manifest writer while preserving original PDFs as immutable source evidence.
