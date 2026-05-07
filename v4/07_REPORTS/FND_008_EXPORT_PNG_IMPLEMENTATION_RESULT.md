# FND-008 Export PNG Order — Implementation Result

## Status

- STATUS: **PARTIAL**
- Agent: Roo Code GPT-5.5
- Date: 2026-05-07
- Branch: `agent/roo/fnd-008-export-png-implementation`

## Scope Implemented

- Replaced `export_pdf_bundle` internals to build export from stored page PNGs instead of merging original PDF objects.
- Reads `pages.image_path`, `pages.current_order`, `pages.rotation`, `pages.is_removed`, and `pages.extract_status`.
- Excludes removed pages from the output PDF while retaining removed-page entries in the manifest.
- Writes a JSON manifest beside the output PDF.
- Invokes a Python exporter module that uses Pillow to create a multipage PDF from ordered PNG files.
- Updates frontend export result typing/UI to show manifest path, exported page count, removed page count, and warnings.

## Files Changed

- `PhanMem/src-tauri/src/commands/export_cmd.rs`
- `PhanMem/python/export/__init__.py`
- `PhanMem/python/export/png_pages_to_pdf.py`
- `PhanMem/src/services/exportService.ts`
- `PhanMem/src/modules/xuatbangiao/xuatbangiao.service.ts`
- `PhanMem/src/modules/xuatbangiao/XuatBanGiaoPage.tsx`
- `v4/07_REPORTS/FND_008_EXPORT_PNG_IMPLEMENTATION_RESULT.md`
- `roo_work/fnd008_export_png_implementation/fnd008_fixture_export.pdf.manifest.json`
- `roo_work/fnd008_export_png_implementation/fnd008_fixture_result.json`

## Manual Code Graph Fallback

- Grapuco sidebar access was not available in this non-interactive session.
- Used manual source inspection and regex search.
- Export caller path confirmed:
  - UI: `PhanMem/src/modules/xuatbangiao/XuatBanGiaoPage.tsx`
  - Service: `PhanMem/src/modules/xuatbangiao/xuatbangiao.service.ts`
  - Tauri command: `PhanMem/src-tauri/src/commands/export_cmd.rs`

## Tests Run

### Rust tests

```cmd
cargo test --manifest-path PhanMem\src-tauri\Cargo.toml
```

Result: PASS — 21 tests passed.

### TypeScript check

```cmd
cd PhanMem && npx tsc --noEmit
```

Result: PASS.

### Python PNG → PDF fixture smoke

```cmd
set PYTHONPATH=PhanMem\python&& python -m export.png_pages_to_pdf --plan roo_work\fnd008_export_png_implementation\fnd008_fixture_export.pdf.manifest.json --output roo_work\fnd008_export_png_implementation\fnd008_fixture_export.pdf --result roo_work\fnd008_export_png_implementation\fnd008_fixture_result.json
```

Result: PASS.

- Output PDF magic: `%PDF-`
- Output PDF size: 289,508 bytes
- Result JSON: `pages_written=1`

## Runtime Export Verification

- Full Tauri UI export was **not run** in this session.
- Reason: task requested implementation after fixture import readiness; this session validated static Rust/TS and the Python PDF builder against an existing extracted PNG fixture.

## Not Done

- Did not run a fresh UI export from the Tauri application.
- Did not verify multipage reorder/removed/rotation via live DB mutation.
- Did not update package export (`export_dossier_package`) to include reviewed PNG PDF; this task focused on `export_pdf_bundle`.

## Next Action

- In Tauri UI, select the already imported FND-004/FND-005 document and run Export PDF.
- Confirm generated PDF and `.manifest.json` reflect `current_order`, `rotation`, and `is_removed` after any review edits.
