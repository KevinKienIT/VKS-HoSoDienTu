# FND-007 Code Review Before Runtime Change

## Task
- Task ID: FND-007
- Title: Group folder and natural sort
- Owner: Codex GPT-5.5
- Date: 2026-05-07

## V4 Context Loaded
- README_V4.md: yes
- PROJECT_SUMMARY.md: yes
- AGENT_RULES.md: yes
- CURRENT_ARCHITECTURE.md: yes
- ACTIVE_TASKS.md: yes

## Grapuco Extension Status
- Running in Antigravity/Gravity: no
- Repository sync status: unknown
- CLI required: NO - Grapuco is a VS Code extension, not CLI
- Code graph reviewed: GRAPUCO_EXTENSION_UNAVAILABLE_IN_SESSION

## Grapuco / Manual Findings
- Feature query: folder import, migration 016, document groups, relative path, natural sort.
- Related files:
  - `PhanMem/src-tauri/migrations/016_document_groups.sql`
  - `PhanMem/src-tauri/src/db/mod.rs`
  - `PhanMem/src-tauri/src/commands/import_cmd.rs`
  - `v4/07_REPORTS/FND_007_DOCUMENT_GROUPS_IMPLEMENTATION_RESULT.md`
- Related functions:
  - `db::init()`
  - `verify_required_schema()`
  - `collect_supported_files()`
  - `start_import_ocr_folder_job()`
  - `run_import_ocr_background_job()`
  - `import_folder()`
  - `debug_import_test_path()`
  - `import_one_file_for_case()`
- Callers:
  - `DuaHoSoVaoPage.tsx` -> `startImportOcrFolderJob()` -> `start_import_ocr_folder_job`.
  - `DanhSachVuAnPage.tsx` -> `importService.importFolder()` -> `import_folder`.
  - `debug_import_test_path()` uses the same collector for debug imports.
- Callees:
  - `storage::copy_to_originals()`
  - `run_page_image_extraction()`
  - `doc_cmd::run_ocr_for_document_with_conn()`
  - `ai_cmd::process_pending_ai_analysis_jobs()`
- DB tables touched:
  - Current: `cases`, `documents`, `pages`, `ocr_results`, `import_jobs`, `pipeline_jobs`, `governed_events`, `fts_documents`.
  - New: `document_groups`.
- Governed events touched:
  - Import events should include `relative_path` and `group_id` where available.
- UI modules affected:
  - No direct UI behavior change required.
- Scripts/checkers affected:
  - No existing dedicated FND-007 checker found; local DB verification will be used.

## Impact Analysis
- Files expected to change:
  - `PhanMem/src-tauri/migrations/016_document_groups.sql`
  - `PhanMem/src-tauri/src/db/mod.rs`
  - `PhanMem/src-tauri/src/commands/import_cmd.rs`
  - `v4/01_MEMORY/FND_007_IMPLEMENTATION_BEFORE_NOTE.md`
  - `v4/01_MEMORY/FND_007_IMPLEMENTATION_AFTER_NOTE.md`
  - `v4/06_BUGS/FND_007_CODE_REVIEW_BEFORE_RUNTIME_CHANGE.md`
  - `v4/07_REPORTS/FND_007_DOCUMENT_GROUPS_IMPLEMENTATION_RESULT.md`
- Functions expected to change:
  - DB init/schema verification and folder import helpers.
- Callers impacted:
  - Folder import, background import, and debug import.
- Callees impacted:
  - Existing copy/extract/OCR functions continue unchanged.
- DB impact:
  - Additive migration: `document_groups`, `documents.group_id`, `documents.relative_path`, indexes.
- UI impact:
  - None required for import command compatibility.
- Export impact:
  - Downstream FND-008 can use saved groups/relative paths.
- Test impact:
  - Cargo tests plus local SQLite/import fixture verification.
- Risk level: MEDIUM

## Current Behavior (Observed)
- Recursive folder import stores only a flat `Vec<PathBuf>`.
- Folder relative paths are not persisted.
- Document groups are not persisted.
- Folder import sorts lexicographically, so `1.pdf`, `10.pdf`, `2.pdf`.

## Target Behavior (V4 Spec)
- Import folder detects subfolders as document groups.
- `documents.relative_path` preserves the import-root-relative file path.
- `documents.group_id` references a `document_groups` row for files inside subfolders.
- Folder import uses natural sort, so `1.pdf`, `2.pdf`, `10.pdf`.

## Expected Diff
```diff
+ migration 016 creates document_groups and adds document metadata columns
+ db::init() includes and runs migration 016
+ db schema verification checks document group columns
+ import collector returns DiscoveredImportFile metadata instead of only PathBuf for folder paths
+ import insert writes group_id and relative_path
+ folder import order uses natural sort on relative_path
```

## Forbidden Changes (Checklist)
- [x] No unrelated refactor
- [x] No runtime model download
- [x] No assetProtocol
- [x] No system Python fallback
- [x] No Docling runtime integration
- [x] No deletion outside scope
- [x] No include python_embedded/** in Tauri resources

## Test Gate
- Commands:
  - `cd PhanMem && cargo test --manifest-path src-tauri/Cargo.toml`
  - Local DB/import verification fixture for `1.pdf`, `2.pdf`, `10.pdf`.
- Expected result:
  - Rust tests pass.
  - `document_groups` exists.
  - `documents.group_id` and `documents.relative_path` are populated for folder files.
  - Natural order is `1,2,10`.
- Manual verification required:
  - Tauri UI import can be run after code tests if runtime fixture import is not available in this session.

## Decision
- APPROVED_TO_EDIT: yes
- BLOCKED_REASON: none
- HANDOFF_REQUIRED: no
