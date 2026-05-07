# FND-007 Document Groups / Natural Sort - Codex Code Review

## Task
- Task ID: FND-007
- Title: Group folder and natural sort
- Owner: Codex GPT-5.5
- Date: 2026-05-07
- Scope: independent review only; no runtime implementation

## V4 Context Loaded
- README_V4.md: yes
- PROJECT_SUMMARY.md: yes
- PROJECT_MEMORY.md: yes
- AGENT_RULES.md: yes
- CURRENT_ARCHITECTURE.md: yes
- ACTIVE_TASKS.md: yes
- FND_002_DB_SCHEMA_CODE_REVIEW.md: yes

## Grapuco Extension Status
- Running in Antigravity/Gravity: no
- Repository sync status: unknown in this non-interactive terminal session
- CLI required: NO - Grapuco is a VS Code extension, not CLI
- Code graph reviewed: GRAPUCO_EXTENSION_UNAVAILABLE_IN_SESSION
- Fallback used: v4/12_AGENT_SYSTEM/MANUAL_CODE_GRAPH_FALLBACK.md

## FND-002 Baseline
- FND-002 confirmed the `pages` table is complete after migrations 014 and 015.
- FND-002 confirmed `pages` has `image_path`, `thumbnail_path`, `current_order`, `rotation`, `is_removed`, `review_required`, `page_quality`, `visual_document_type`, and `quality_warnings`.
- FND-002 confirmed the remaining schema gap for FND-007: no `document_groups` table and no `documents.group_id`.
- Migration 015 already adds `documents.suggested_filename` and `documents.visual_document_type`; FND-007 should not duplicate those columns.

## Grapuco / Manual Findings
- Feature query: folder import, document group schema, relative path preservation, natural sort.
- Related files:
  - `PhanMem/src-tauri/src/commands/import_cmd.rs`
  - `PhanMem/src-tauri/src/commands/doc_cmd.rs`
  - `PhanMem/src-tauri/src/db/mod.rs`
  - `PhanMem/src-tauri/migrations/001_init_schema.sql`
  - `PhanMem/src-tauri/migrations/014_page_image_architecture.sql`
  - `PhanMem/src-tauri/migrations/015_page_quality_classifier.sql`
  - `PhanMem/src/modules/duahosovao/DuaHoSoVaoPage.tsx`
  - `PhanMem/src/modules/duahosovao/duahosovao.service.ts`
  - `PhanMem/src/modules/hosovuan/DanhSachVuAnPage.tsx`
  - `PhanMem/src/services/importService.ts`
  - `PhanMem/src/services/documentService.ts`
  - `PhanMem/src/modules/quantailieu/quantailieu.service.ts`
- Related functions:
  - `collect_supported_files()`
  - `start_import_ocr_folder_job()`
  - `run_import_ocr_background_job()`
  - `import_folder()`
  - `debug_import_test_path()`
  - `import_one_file_for_case()`
  - `get_document_groups()`
  - `db::init()`
  - `db::verify_required_schema()`
- Callers:
  - `DuaHoSoVaoPage.tsx` opens a directory and calls `startImportOcrFolderJob()`.
  - `duahosovao.service.ts` invokes Tauri command `start_import_ocr_folder_job`.
  - `DanhSachVuAnPage.tsx` opens a directory and calls `importService.importFolder()`.
  - `importService.ts` invokes Tauri command `import_folder`.
  - `debug_import_test_path()` also uses the same file collector for directory debug imports.
- Callees:
  - Folder import calls `collect_supported_files()`.
  - Background folder import passes `Vec<PathBuf>` to `run_import_ocr_background_job()`.
  - Worker import calls `import_one_file_for_case()`.
  - Sync folder import inserts documents directly inside `import_folder()`.
  - Document rows then flow into page image extraction and OCR through existing FND-004/FND-005 paths.
- DB tables touched:
  - Current: `cases`, `documents`, `pages`, `ocr_results`, `fts_documents`, `import_jobs`, `pipeline_jobs`, `governed_events`.
  - Target: add `document_groups`; add `documents.group_id` and `documents.relative_path`.
- Governed events touched:
  - Current events include `IMPORT_STARTED`, `IMPORT_JOB_STARTED`, `IMPORT_FILE_STARTED`, `IMPORT_FILE_DONE`, `IMPORT_DONE`, `IMPORT_FAILED`, `PAGE_IMAGE_EXTRACT_STARTED`, `PAGE_IMAGE_EXTRACT_DONE`, and OCR/AI worker events.
  - Target import events should include `relative_path` and `group_id` where a document is imported from a folder.
- UI modules affected:
  - Folder import entry points in `duahosovao` and `hosovuan`.
  - Document list/group summaries in `bangdieukhien` and `quantailieu` may later display folder groups.
- Scripts/checkers affected:
  - No existing FND-007 checker found.
  - A follow-up checker should query `document_groups`, `documents.group_id`, `documents.relative_path`, and natural ordering.

## Current Behavior Observed

### Folder import flattens directory structure
- `collect_supported_files(dir, out)` recursively walks child directories and only pushes absolute `PathBuf` file paths into one flat vector.
- The collector does not preserve import-root-relative paths.
- The collector does not derive folder group metadata.
- The collector does not skip junk folders/files such as `__MACOSX`, `.DS_Store`, `.git`, or temporary `~$` files.

### Folder import sorts lexicographically
- `start_import_ocr_folder_job()` calls `files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()))`.
- `import_folder()` uses the same lexicographic sort.
- `debug_import_test_path()` uses the same lexicographic sort.
- This orders `1.pdf`, `10.pdf`, `2.pdf`; V4 target requires `1.pdf`, `2.pdf`, `10.pdf`.

### Schema lacks persisted folder groups
- Migrations present: 001 through 015 only.
- No migration creates `document_groups`.
- `db::mod.rs` includes `MIGRATION_001` through `MIGRATION_015`; no `MIGRATION_016`.
- `REQUIRED_TABLES` does not include `document_groups`.
- `verify_required_schema()` checks required tables and page image columns, but does not verify `documents.group_id` or `documents.relative_path`.
- `documents` base schema and later migrations do not add `group_id` or `relative_path`.

### Existing `get_document_groups()` is not folder grouping
- `get_document_groups()` groups by `documents.document_type`.
- It returns `DocumentGroup { group_key, count }`.
- Existing UI services use it as a document-type summary.
- FND-007 should not silently change this API into folder hierarchy semantics. Add a separate API for persisted folder groups, or keep compatibility until UI migration is explicit.

### Document inserts cannot persist group metadata
- `import_one_file_for_case()` inserts into `documents` without `group_id` or `relative_path`.
- `import_folder()` direct insert also omits `group_id` and `relative_path`.
- Background import currently passes only `Vec<PathBuf>` into the worker, so folder metadata is lost before document insert.

## Target Behavior (V4 Spec)
- Import folder should preserve source folder structure.
- Each source folder group should be persisted in `document_groups`.
- Each imported document should store:
  - `documents.group_id`
  - `documents.relative_path`
- Sorting should be natural by normalized relative path:
  - `1.pdf`
  - `2.pdf`
  - `10.pdf`
- Folder paths should sort naturally too:
  - `tap_1/a.pdf`
  - `tap_2/a.pdf`
  - `tap_10/a.pdf`
- Import should continue to feed the existing page image extraction and OCR pipeline unchanged after document insertion.

## Proposed Schema - Migration 016

File expected in implementation phase:

- `PhanMem/src-tauri/migrations/016_document_groups.sql`

Recommended SQL:

```sql
-- Migration 016: Document Groups / Folder Hierarchy
CREATE TABLE IF NOT EXISTS document_groups (
    group_id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
    parent_group_id TEXT REFERENCES document_groups(group_id) ON DELETE CASCADE,
    group_name TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(case_id, relative_path)
);

CREATE INDEX IF NOT EXISTS idx_document_groups_case_parent_sort
ON document_groups(case_id, parent_group_id, sort_order, group_name);

CREATE INDEX IF NOT EXISTS idx_document_groups_case_path
ON document_groups(case_id, relative_path);

ALTER TABLE documents ADD COLUMN group_id TEXT REFERENCES document_groups(group_id);
ALTER TABLE documents ADD COLUMN relative_path TEXT;

CREATE INDEX IF NOT EXISTS idx_documents_case_group
ON documents(case_id, group_id);

CREATE INDEX IF NOT EXISTS idx_documents_case_relative_path
ON documents(case_id, relative_path);
```

Optional implementation consideration:
- Root-level files may use `group_id = NULL` and `relative_path = file_name`, or a synthetic root group with `relative_path = ''`. Choose one behavior and document it in the implementation report.

## Expected Diff

```diff
PhanMem/src-tauri/migrations/016_document_groups.sql
+ CREATE TABLE IF NOT EXISTS document_groups (...)
+ ALTER TABLE documents ADD COLUMN group_id TEXT REFERENCES document_groups(group_id);
+ ALTER TABLE documents ADD COLUMN relative_path TEXT;
+ CREATE INDEX IF NOT EXISTS idx_document_groups_case_parent_sort ...
+ CREATE INDEX IF NOT EXISTS idx_documents_case_group ...

PhanMem/src-tauri/src/db/mod.rs
+ const MIGRATION_016: &str = include_str!("../../migrations/016_document_groups.sql");
+ REQUIRED_TABLES includes "document_groups";
+ run_alter_migration(&conn, MIGRATION_016, "MIGRATION_016") after M015;
+ verify_document_group_columns(conn) checks:
+   document_groups.group_id, case_id, parent_group_id, group_name, relative_path, sort_order
+   documents.group_id, documents.relative_path

PhanMem/src-tauri/src/commands/import_cmd.rs
+ struct DiscoveredImportFile {
+   absolute_path: PathBuf,
+   relative_path: String,
+   group_relative_path: String,
+   file_name: String,
+ }
+ fn collect_supported_files_with_relative_paths(root: &Path) -> Result<Vec<DiscoveredImportFile>, String>
+ fn is_junk_path_component(value: &str) -> bool
+ fn natural_sort_key(value: &str) -> Vec<NaturalPart>
+ fn compare_natural_relative_path(a: &str, b: &str) -> Ordering
+ fn ensure_document_group_hierarchy(conn, case_id, group_relative_path, sort_order_map) -> Result<Option<String>, String>
+ import_one_file_for_case(...) accepts optional group_id/relative_path or a metadata struct.
+ start_import_ocr_folder_job() passes Vec<DiscoveredImportFile> to the worker.
+ import_folder() direct path uses same discovered metadata path.
+ debug_import_test_path() uses the same collector and natural sort.
+ INSERT INTO documents includes group_id and relative_path.
+ IMPORT_* governed event payloads include relative_path and group_id when available.

PhanMem/src-tauri/src/commands/doc_cmd.rs
+ Keep get_document_groups() behavior unchanged or rename later with explicit UI migration.
+ Add get_folder_document_groups(case_id) if UI needs persisted folder hierarchy now.
+ Extend DocumentSummary only if current UI needs group_id/relative_path immediately.

PhanMem/src/modules/duahosovao/duahosovao.service.ts
PhanMem/src/services/importService.ts
+ Update returned ImportedFile shape only if backend returns relative_path/group_id.

PhanMem/src/modules/quantailieu/quantailieu.service.ts
PhanMem/src/services/documentService.ts
+ Add folder group types/API only if implementing UI display in FND-007.
```

## Natural Sort Design
- Split a path into alternating digit and non-digit runs.
- Compare numeric runs by integer value.
- Compare text runs case-insensitively and with stable fallback to the original string.
- Apply comparator to the normalized forward-slash `relative_path`, not absolute OS path.
- Keep deterministic tie-breakers:
  - natural key
  - normalized relative path
  - original absolute path

Example model:

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NaturalPart {
    Text(String),
    Number(u64),
}

fn natural_sort_key(value: &str) -> Vec<NaturalPart> {
    // Split contiguous ASCII digit runs from text runs.
}
```

## Impact Analysis
- Files expected to change:
  - `PhanMem/src-tauri/migrations/016_document_groups.sql`
  - `PhanMem/src-tauri/src/db/mod.rs`
  - `PhanMem/src-tauri/src/commands/import_cmd.rs`
  - Optional: `PhanMem/src-tauri/src/commands/doc_cmd.rs`
  - Optional: `PhanMem/src/services/importService.ts`
  - Optional: `PhanMem/src/modules/duahosovao/duahosovao.service.ts`
  - Optional: `PhanMem/src/services/documentService.ts`
  - Optional: `PhanMem/src/modules/quantailieu/quantailieu.service.ts`
- Functions expected to change:
  - `db::init()`
  - `db::verify_required_schema()`
  - `collect_supported_files()`
  - `start_import_ocr_folder_job()`
  - `run_import_ocr_background_job()`
  - `import_folder()`
  - `debug_import_test_path()`
  - `import_one_file_for_case()`
- Callers impacted:
  - Folder import from `DuaHoSoVaoPage.tsx`
  - Folder import from `DanhSachVuAnPage.tsx`
  - Debug import path
- Callees impacted:
  - `storage::copy_to_originals()`
  - `run_page_image_extraction()`
  - `doc_cmd::run_ocr_for_document_with_conn()`
  - `ai_cmd::process_pending_ai_analysis_jobs()`
- DB impact: additive migration plus schema verification; no destructive DB change expected.
- UI impact: none required for core import persistence if backend return shape stays compatible.
- Export impact: downstream FND-008 can use `document_groups.sort_order` and `documents.relative_path` for package folder structure.
- Test impact: migration idempotency, import ordering, group persistence, and existing FND-004/FND-005 import/OCR gates.
- Risk level: MEDIUM because both sync import and background import must preserve identical metadata and ordering.

## Forbidden Changes Checklist
- [x] No runtime code edited in this review
- [x] No migration file created in runtime tree
- [x] No unrelated refactor
- [x] No runtime model download
- [x] No assetProtocol
- [x] No system Python fallback
- [x] No Docling runtime integration
- [x] No deletion outside scope
- [x] No include python_embedded/** in Tauri resources

## Test Gate

Commands for implementation phase:

```powershell
cd PhanMem
cargo test --manifest-path src-tauri\Cargo.toml
npx tsc --noEmit
```

Runtime/manual verification:

```powershell
cd PhanMem
npm run tauri:dev
```

Fixture folder:

```text
fixture_fnd007/
  tap_1/
    1.pdf
    2.pdf
    10.pdf
  tap_2/
    1.pdf
  tap_10/
    1.pdf
  __MACOSX/
    junk.pdf
  .DS_Store
```

Expected DB checks:

```sql
SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'document_groups';
PRAGMA table_info(documents);
SELECT relative_path FROM documents ORDER BY relative_path;
SELECT group_id, case_id, parent_group_id, group_name, relative_path, sort_order FROM document_groups ORDER BY sort_order, relative_path;
SELECT original_filename, relative_path, group_id FROM documents WHERE relative_path LIKE 'tap_%/%';
```

Expected result:
- `document_groups` exists.
- `documents.group_id` exists.
- `documents.relative_path` exists.
- Files in subfolders have non-null `group_id`.
- `relative_path` uses stable forward slashes.
- Junk files/folders are not imported.
- Natural order is `1.pdf`, `2.pdf`, `10.pdf`.
- Existing PNG extraction and OCR still run after import.

## Decision
- REVIEW STATUS: COMPLETE
- APPROVED_TO_EDIT: no
- BLOCKED_REASON: review-only task; implementation must happen in a separate runtime-code task
- HANDOFF_REQUIRED: yes, to FND-007 implementation branch

## Commit Scope
- Runtime code touched: NO
- Files created/edited by Codex for this task:
  - `v4/07_REPORTS/FND_007_DOCUMENT_GROUPS_REVIEW.md`
