# FND-007 Document Groups / Natural Sort — Code Review and Migration Plan

## Task

- Task ID: FND-007
- Title: Group folder and natural sort
- Agent: Roo Code GPT-5.5
- Date: 2026-05-07
- Scope: Review current import/schema implementation and plan migration 016. Runtime code was not edited.

## V4 Context Loaded

- v4/README_V4.md
- v4/00_PROJECT_BRIEF/PROJECT_SUMMARY.md
- v4/01_MEMORY/PROJECT_MEMORY.md
- v4/02_RULES/AGENT_RULES.md
- v4/03_DESIGN/CURRENT_ARCHITECTURE.md
- v4/05_TASKLIST/ACTIVE_TASKS.md
- v4/07_REPORTS/FND_002_DB_SCHEMA_CODE_REVIEW.md
- v4/07_REPORTS/FND_004_PAGE_IMAGE_EXTRACTION_RUNTIME_RESULT.md

## Grapuco Status

- GRAPUCO_EXTENSION_UNAVAILABLE_IN_SESSION
- Reason: current tool session cannot interact with the VS Code sidebar extension directly.
- Fallback used: v4/12_AGENT_SYSTEM/MANUAL_CODE_GRAPH_FALLBACK.md, with read-only source inspection and recursive symbol search.
- Runtime edit approval: NO. This report is a plan/review only.

## Findings Summary

### Schema gap confirmed

- FND-002 already confirmed there is no `document_groups` table and no `documents.group_id` column.
- FND-002 also confirmed `documents.suggested_filename` and `documents.visual_document_type` exist from migration 015.
- Current `db::REQUIRED_TABLES` does not include `document_groups`.
- Current `db::init()` runs migrations 001-015 only.

### Import flow currently flattens folders

- `collect_supported_files(dir, out)` recursively walks folders and pushes only supported files.
- It does not preserve import-root-relative paths.
- It does not ignore junk folders/files such as `__MACOSX`, `.DS_Store`, `.git`.
- Both `start_import_ocr_folder_job()` and `import_folder()` call `collect_supported_files()` and then do lexicographic sorting with `to_string_lossy().cmp(...)`.
- Lexicographic sorting produces `1.pdf, 10.pdf, 2.pdf`; FND-007 target requires natural sort: `1.pdf, 2.pdf, 10.pdf`.

### Existing document group API is not folder grouping

- `get_document_groups()` currently groups by `documents.document_type`.
- It does not read a `document_groups` hierarchy.
- UI modules use this as a dashboard/document-type summary today, so FND-007 must avoid breaking this behavior or add a separate folder-group API.

## Symbol / Data Flow Context

| Area                   | File                                                   | Symbol / Lines                                                     | Finding                                                                                           |
| ---------------------- | ------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------- |
| DB init                | `PhanMem/src-tauri/src/db/mod.rs`                      | `MIGRATION_001..MIGRATION_015`, `REQUIRED_TABLES`, `init()`        | Need add migration 016 include, run gate, required table/columns verification.                    |
| File collection        | `PhanMem/src-tauri/src/commands/import_cmd.rs`         | `collect_supported_files()`                                        | Recursive flat collection; no group metadata.                                                     |
| Folder import command  | `PhanMem/src-tauri/src/commands/import_cmd.rs`         | `import_folder()`                                                  | Creates case, loops files, inserts documents without group/relative path.                         |
| Background import/OCR  | `PhanMem/src-tauri/src/commands/import_cmd.rs`         | `start_import_ocr_folder_job()`, `run_import_ocr_background_job()` | Main UI import path; must preserve group metadata across worker thread.                           |
| Single-file import     | `PhanMem/src-tauri/src/commands/import_cmd.rs`         | `import_one_file_for_case()`                                       | Inserts document rows; should accept optional group metadata or have separate folder import path. |
| Existing group summary | `PhanMem/src-tauri/src/commands/doc_cmd.rs`            | `get_document_groups()`                                            | Groups by document_type; not folder hierarchy.                                                    |
| UI import entry        | `PhanMem/src/modules/duahosovao/DuaHoSoVaoPage.tsx`    | folder dialog + `startImportOcrFolderJob()`                        | Main UI path for directory import.                                                                |
| UI service             | `PhanMem/src/modules/duahosovao/duahosovao.service.ts` | `startImportOcrFolderJob()`                                        | Calls Tauri command `start_import_ocr_folder_job`.                                                |
| Case list import       | `PhanMem/src/modules/hosovuan/DanhSachVuAnPage.tsx`    | folder dialog + `importService.importFolder()`                     | Secondary folder import path.                                                                     |

## Proposed Migration 016 Plan

File to create in implementation phase:

- `PhanMem/src-tauri/migrations/016_document_groups.sql`

Runtime DB bootstrap updates in implementation phase:

- Add `const MIGRATION_016: &str = include_str!("../../migrations/016_document_groups.sql");`
- Add `document_groups` to `REQUIRED_TABLES`.
- Run `run_alter_migration(&conn, MIGRATION_016, "MIGRATION_016")?;` after migration 015.
- Extend schema verification for `documents.group_id`, `documents.relative_path`, and document group columns.

### Skeleton SQL

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

## Natural Sort Logic Plan

### Target behavior

- Natural sort compares alternating text and numeric chunks.
- Numeric chunks compare by integer value, not string value.
- Example order: `1.pdf`, `2.pdf`, `10.pdf`.
- Folder order should also be natural: `tap_1/a.pdf`, `tap_2/a.pdf`, `tap_10/a.pdf`.

### Suggested Rust model

- Replace `Vec<PathBuf>` in folder import with `Vec<DiscoveredImportFile>`.
- Fields:
  - `absolute_path: PathBuf`
  - `relative_path: String`
  - `group_relative_path: String`
  - `file_name: String`
  - `sort_key: Vec<NaturalPart>`
- `NaturalPart` variants:
  - `Text(String)` lowercased and optionally Vietnamese-normalized for compare only.
  - `Number(u64)` parsed from contiguous ASCII digits.

### Suggested comparator

```rust
fn natural_sort_key(value: &str) -> Vec<NaturalPart> {
    // Split into digit and non-digit runs.
    // "10_quyet_dinh.pdf" => [Number(10), Text("_quyet_dinh.pdf")]
}

files.sort_by(|a, b| {
    natural_sort_key(&a.relative_path)
        .cmp(&natural_sort_key(&b.relative_path))
        .then_with(|| a.relative_path.cmp(&b.relative_path))
});
```

## Import Implementation Plan

### Collection phase

1. Introduce `collect_supported_files_with_relative_paths(root)`.
2. Skip junk directories/files:
   - `__MACOSX`
   - `.DS_Store`
   - `.git`
   - hidden temp files beginning with `~$`
3. Store relative path using forward slashes for DB stability.
4. Derive `group_relative_path` from parent folder of `relative_path`; root-level files can use empty string or `/`.

### Group persistence phase

1. For each unique non-root `group_relative_path`, create or reuse a `document_groups` row.
2. Build parent groups recursively so `A/B` has parent `A`.
3. Assign deterministic `sort_order` from natural-sorted folder order.
4. For each document insert, set:
   - `documents.group_id`
   - `documents.relative_path`

### Background worker impact

- `start_import_ocr_folder_job()` currently passes `Vec<PathBuf>` into `run_import_ocr_background_job()`.
- FND-007 implementation should pass `Vec<DiscoveredImportFile>` or maintain a map from absolute path to metadata.
- `import_one_file_for_case()` should receive optional metadata, or a new helper should insert folder-import documents with group/relative fields.

### Existing API compatibility

- Keep `get_document_groups()` document-type summary unchanged for current UI widgets, or rename internally later.
- Add a separate API such as `get_folder_document_groups(case_id)` for true folder hierarchy.

## DB / Event / UI Impact

| Impact | Detail                                                                                               | Risk                                                                      |
| ------ | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| DB     | Add table `document_groups`, columns `documents.group_id`, `documents.relative_path`, indexes.       | LOW-MEDIUM additive migration.                                            |
| Import | Folder import must preserve relative paths and group metadata.                                       | MEDIUM because both sync and background imports must stay aligned.        |
| OCR    | No direct OCR algorithm change. OCR continues after document/page rows exist.                        | LOW.                                                                      |
| Events | Import events should include `relative_path` and `group_id` for audit traceability.                  | LOW.                                                                      |
| UI     | Import dialogs unchanged; document list/export can later display folder hierarchy.                   | MEDIUM if `get_document_groups()` behavior is changed; avoid breaking it. |
| Export | FND-008 should use `document_groups.sort_order` and `documents.relative_path` for package structure. | MEDIUM, downstream task.                                                  |

## Test Gate for Future Implementation

Do not run for this review-only task. For implementation phase:

```powershell
cd PhanMem
cargo test --manifest-path src-tauri\Cargo.toml
npx tsc --noEmit
```

Runtime fixture expectation:

- Import folder containing `1.pdf`, `2.pdf`, `10.pdf` under multiple subfolders.
- Verify:
  - `document_groups` exists.
  - `documents.group_id IS NOT NULL` for files under folders.
  - `documents.relative_path` preserves source folder path.
  - Natural order is `1,2,10`, not `1,10,2`.

## Decision

- REVIEW STATUS: COMPLETE
- APPROVED_TO_EDIT_RUNTIME_CODE: NO
- MIGRATION 016: PLANNED ONLY
- TESTS RUN: NO, per user instruction and review-only scope

## Next Action

Implement FND-007 in a separate runtime-code task after Grapuco sidebar review is available or manual fallback is accepted for runtime edits.
