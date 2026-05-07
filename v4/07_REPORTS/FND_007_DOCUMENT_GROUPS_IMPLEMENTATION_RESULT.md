# FND-007 Document Groups / Natural Sort - Implementation Result

## Status
- CODEX FND-007 IMPLEMENTATION STATUS: PASS
- Branch: `agent/codex/fnd-007-document-groups-implementation`
- Runtime code touched: YES
- V4 touched: YES
- Grapuco: `GRAPUCO_EXTENSION_UNAVAILABLE_IN_SESSION`; manual fallback used

## Implemented
- Created `PhanMem/src-tauri/migrations/016_document_groups.sql`.
- Added `document_groups` table:
  - `group_id`
  - `case_id`
  - `parent_group_id`
  - `name`
  - `relative_path`
  - `sort_order`
  - `created_at`
  - `updated_at`
- Added `documents.group_id`.
- Added `documents.relative_path`.
- Added indexes for group lookup and document group filtering.
- Updated `db::init()` to run migration 016 after migration 015.
- Added schema verification for `document_groups`, `documents.group_id`, and `documents.relative_path`.
- Replaced folder import flattening with `DiscoveredImportFile` metadata.
- Added natural sort for import-root-relative paths.
- Added junk filtering for `__MACOSX`, `.DS_Store`, `.git`, and `~$*` temp files.
- Added group creation/reuse by folder relative path.
- Stored `relative_path`, `group_id`, and `import_sequence` in imported document rows.
- Updated TypeScript `ImportedFile` types with optional `group_id`, `relative_path`, and `import_order`.

## DB Evidence
- Migration 016 is embedded by `db::init()`.
- `REQUIRED_TABLES` includes `document_groups`.
- `verify_required_schema()` checks:
  - `document_groups.group_id`
  - `document_groups.case_id`
  - `document_groups.parent_group_id`
  - `document_groups.name`
  - `document_groups.relative_path`
  - `document_groups.sort_order`
  - `document_groups.created_at`
  - `documents.group_id`
  - `documents.relative_path`

## Import Evidence
- Automated Rust test creates an equivalent fixture folder with:
  - `tap_1/1.pdf`
  - `tap_1/2.pdf`
  - `tap_1/10.pdf`
- The test imports those PDFs through the same import helper path used by folder import.
- DB verification confirms:
  - `relative_path` order is `tap_1/1.pdf`, `tap_1/2.pdf`, `tap_1/10.pdf`.
  - Every imported row has `group_id`.
  - `import_sequence` is `1, 2, 3`.
  - One `document_groups` row exists for `tap_1`.

## Tests Run

```powershell
cd PhanMem
$env:CARGO_TARGET_DIR='C:\Users\KKIT\AppData\Local\Temp\vks_fnd007_target'
cargo test --manifest-path src-tauri\Cargo.toml -j 1
```

Result:

```text
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Important test:

```text
commands::import_cmd::tests::folder_import_metadata_persists_relative_path_group_and_natural_order ... ok
```

## Additional Checks
- `git diff --check`: PASS, no whitespace errors.
- `npm ci`: PASS.
- `npx tsc --noEmit`: FAIL due pre-existing missing `../lib/catalog` / `../../lib/catalog` imports. No FND-007-specific TypeScript type error was reported before those module-resolution failures.

## Fixture Note
- Literal path checked: `PhanMem/test_pdfs/folder_with_1_2_10.pdf`.
- Result: NOT_FOUND.
- Substitute: automated Rust test constructs an equivalent temp folder from tracked `PhanMem/test_pdfs/test_doc_1.pdf`.

## NOT_DONE
- Tauri UI manual import was not run in this session.
- Exact literal fixture path `test_pdfs/folder_with_1_2_10.pdf` was not available.

## Next Action
- Open PR and review migration/import flow.
- Optional manual UI gate: run `npm run tauri:dev`, import a real folder containing `1.pdf`, `2.pdf`, `10.pdf`, then query `document_groups` and `documents`.
