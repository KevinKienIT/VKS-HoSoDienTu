// VKS ECMS - Database module

pub mod schema;

use rusqlite::Connection;
use std::path::Path;

/// Noi dung file migration 001_init_schema.sql duoc embed compile-time.
/// Khi can migration moi, them file vao thu muc migrations/ va them vao day.
const MIGRATION_001: &str = include_str!("../../migrations/001_init_schema.sql");
const MIGRATION_002: &str = include_str!("../../migrations/002_module_configs.sql");
const MIGRATION_003: &str = include_str!("../../migrations/003_scan_ricoh.sql");
const MIGRATION_004: &str = include_str!("../../migrations/004_pipeline_phase1.sql");
const MIGRATION_005: &str = include_str!("../../migrations/005_ocr_layout_utf8.sql");
const MIGRATION_006: &str = include_str!("../../migrations/006_page_layout_extracted_fields.sql");
const MIGRATION_007: &str = include_str!("../../migrations/007_ocr_normalized_text.sql");
const MIGRATION_008: &str = include_str!("../../migrations/008_file_lifecycle.sql");
const MIGRATION_009: &str = include_str!("../../migrations/009_page_ocr_scheduler.sql");
const MIGRATION_010: &str = include_str!("../../migrations/010_parsed_sections.sql");
const MIGRATION_011: &str = include_str!("../../migrations/011_event_governance.sql");
const MIGRATION_012: &str = include_str!("../../migrations/012_ai_summary_flow.sql");
const MIGRATION_013: &str = include_str!("../../migrations/013_ai_post_ocr_pipeline.sql");
const MIGRATION_014: &str = include_str!("../../migrations/014_page_image_architecture.sql");
const MIGRATION_015: &str = include_str!("../../migrations/015_page_quality_classifier.sql");
const MIGRATION_016: &str = include_str!("../../migrations/016_document_groups.sql");

pub const REQUIRED_TABLES: &[&str] = &[
    "cases",
    "documents",
    "pages",
    "ocr_results",
    "page_layout_blocks",
    "document_extracted_fields",
    "document_scan_quality",
    "governed_events",
    "document_ai_summaries",
    "document_but_luc",
    "document_groups",
    "case_ai_summaries",
    "case_people_profiles",
    "people_profile_mentions",
    "ai_analysis_jobs",
    "ai_analysis_events",
];

fn statement_has_sql(statement: &str) -> bool {
    statement.lines().any(|line| {
        let trimmed = line.trim_start();
        !trimmed.is_empty() && !trimmed.starts_with("--")
    })
}

fn run_alter_migration(conn: &Connection, sql: &str, label: &str) -> Result<(), String> {
    for statement in sql.split(';') {
        let stmt = statement.trim();
        if stmt.is_empty() || !statement_has_sql(stmt) {
            continue;
        }
        if let Err(e) = conn.execute(stmt, []) {
            let msg = e.to_string();
            if !msg.contains("duplicate column name") && !msg.contains("already exists") {
                return Err(format!("DB_INIT_{label}_FAILED on statement [{stmt}]: {e}"));
            }
        }
    }
    Ok(())
}

pub fn missing_required_tables(conn: &Connection) -> Result<Vec<String>, String> {
    let mut missing = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT 1
             FROM sqlite_master
             WHERE type IN ('table','view')
               AND name = ?1
             LIMIT 1",
        )
        .map_err(|e| format!("DB_SCHEMA_VERIFY_PREPARE_FAILED: {e}"))?;
    for table in REQUIRED_TABLES {
        let exists = stmt
            .exists([*table])
            .map_err(|e| format!("DB_SCHEMA_VERIFY_TABLE_FAILED:{table}:{e}"))?;
        if !exists {
            missing.push((*table).to_string());
        }
    }
    Ok(missing)
}

pub fn verify_required_schema(conn: &Connection) -> Result<(), String> {
    let missing = missing_required_tables(conn)?;
    if !missing.is_empty() {
        return Err(format!(
            "DB_SCHEMA_REQUIRED_TABLES_MISSING: {}",
            missing.join(", ")
        ));
    }
    verify_page_image_columns(conn)?;
    verify_document_group_columns(conn)?;
    Ok(())
}

fn verify_table_columns(
    conn: &Connection,
    table: &str,
    required_columns: &[&str],
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| format!("DB_SCHEMA_VERIFY_COLUMNS_PREPARE_FAILED:{table}:{e}"))?;
    let column_iter = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("DB_SCHEMA_VERIFY_COLUMNS_QUERY_FAILED:{table}:{e}"))?;
    let mut existing = Vec::<String>::new();
    for column in column_iter {
        existing
            .push(column.map_err(|e| format!("DB_SCHEMA_VERIFY_COLUMN_READ_FAILED:{table}:{e}"))?);
    }
    let missing = required_columns
        .iter()
        .filter(|column| {
            !existing
                .iter()
                .any(|existing_column| existing_column == **column)
        })
        .map(|column| (*column).to_string())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "DB_SCHEMA_REQUIRED_COLUMNS_MISSING:{table}:{}",
            missing.join(", ")
        ))
    }
}

fn verify_page_image_columns(conn: &Connection) -> Result<(), String> {
    verify_table_columns(
        conn,
        "pages",
        &[
            "source_pdf_path",
            "source_page_number",
            "thumbnail_path",
            "current_order",
            "rotation",
            "is_removed",
            "review_required",
            "page_quality",
            "visual_document_type",
            "quality_warnings",
        ],
    )
}

fn verify_document_group_columns(conn: &Connection) -> Result<(), String> {
    verify_table_columns(
        conn,
        "document_groups",
        &[
            "group_id",
            "case_id",
            "parent_group_id",
            "name",
            "relative_path",
            "sort_order",
            "created_at",
        ],
    )?;
    verify_table_columns(conn, "documents", &["group_id", "relative_path"])
}

fn ensure_ocr_normalized_schema(conn: &Connection) -> Result<(), String> {
    for sql in [
        "ALTER TABLE page_layout_blocks ADD COLUMN normalized_text TEXT",
        "ALTER TABLE page_layout_blocks ADD COLUMN unicode_form TEXT NOT NULL DEFAULT 'NFC'",
    ] {
        if let Err(e) = conn.execute(sql, []) {
            let msg = e.to_string();
            if !msg.contains("duplicate column name") {
                return Err(format!("DB_INIT_OCR_NORMALIZED_COLUMN_FAILED: {e}"));
            }
        }
    }

    conn.execute_batch(
        "DROP TABLE IF EXISTS fts_layout_blocks;
         CREATE VIRTUAL TABLE IF NOT EXISTS fts_layout_blocks USING fts5(
             id UNINDEXED,
             document_id UNINDEXED,
             page_number UNINDEXED,
             block_type,
             text,
             normalized_text,
             content=page_layout_blocks,
             content_rowid=rowid
         );
         INSERT INTO fts_layout_blocks(fts_layout_blocks) VALUES('rebuild');",
    )
    .map_err(|e| format!("DB_INIT_OCR_NORMALIZED_FTS_FAILED: {e}"))?;

    Ok(())
}

/// Khoi tao database tai duong dan cho truoc.
/// Tao file .db neu chua ton tai, chay tat ca migration.
///
/// # Errors
/// Tra ve loi neu khong tao duoc file hoac SQL sai syntax.
pub fn init(db_path: &Path) -> Result<(), String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("DB_INIT_OPEN_FAILED at {}: {e}", db_path.display()))?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| format!("DB_INIT_PRAGMA_FAILED: {e}"))?;

    conn.execute_batch(MIGRATION_001)
        .map_err(|e| format!("DB_INIT_MIGRATION_001_FAILED: {e}"))?;

    conn.execute_batch(MIGRATION_002)
        .map_err(|e| format!("DB_INIT_MIGRATION_002_FAILED: {e}"))?;

    conn.execute_batch(MIGRATION_003)
        .map_err(|e| format!("DB_INIT_MIGRATION_003_FAILED: {e}"))?;

    conn.execute_batch(MIGRATION_004)
        .map_err(|e| format!("DB_INIT_MIGRATION_004_FAILED: {e}"))?;

    // Migration 005 uses ALTER TABLE which might fail if already applied.
    run_alter_migration(&conn, MIGRATION_005, "MIGRATION_005")?;

    conn.execute_batch(MIGRATION_006)
        .map_err(|e| format!("DB_INIT_MIGRATION_006_FAILED: {e}"))?;

    // Migration 007: ALTER TABLE — skip if already applied
    run_alter_migration(&conn, MIGRATION_007, "MIGRATION_007")?;

    // P0-03 guard: execute_batch can stop on duplicate ALTER columns before
    // rebuilding the FTS5 table, so enforce the final schema idempotently.
    ensure_ocr_normalized_schema(&conn)?;

    // Migration 008: ALTER TABLE — skip if already applied
    run_alter_migration(&conn, MIGRATION_008, "MIGRATION_008")?;

    // Migration 009: ALTER TABLE — skip if already applied
    run_alter_migration(&conn, MIGRATION_009, "MIGRATION_009")?;

    conn.execute_batch(MIGRATION_010)
        .map_err(|e| format!("DB_INIT_MIGRATION_010_FAILED: {e}"))?;

    conn.execute_batch(MIGRATION_011)
        .map_err(|e| format!("DB_INIT_MIGRATION_011_FAILED: {e}"))?;

    conn.execute_batch(MIGRATION_012)
        .map_err(|e| format!("DB_INIT_MIGRATION_012_FAILED: {e}"))?;

    // Migration 013 mixes ALTER TABLE with new CREATE TABLE statements.
    run_alter_migration(&conn, MIGRATION_013, "MIGRATION_013")?;

    // Migration 014: Page image architecture
    run_alter_migration(&conn, MIGRATION_014, "MIGRATION_014")?;

    // Migration 015: Page quality classifier columns
    run_alter_migration(&conn, MIGRATION_015, "MIGRATION_015")?;

    // Migration 016: Document groups / folder hierarchy
    run_alter_migration(&conn, MIGRATION_016, "MIGRATION_016")?;

    verify_required_schema(&conn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn migration_sql_is_not_empty() {
        assert!(!MIGRATION_001.is_empty(), "Migration SQL must not be empty");
        assert!(
            MIGRATION_001.contains("CREATE TABLE"),
            "Migration must contain CREATE TABLE statements"
        );
    }

    #[test]
    fn migration_002_is_not_empty() {
        assert!(!MIGRATION_002.is_empty());
        assert!(MIGRATION_002.contains("module_configs"));
    }

    #[test]
    fn migration_003_is_not_empty() {
        assert!(!MIGRATION_003.is_empty());
        assert!(MIGRATION_003.contains("scan_jobs"));
    }

    #[test]
    fn migration_004_is_not_empty() {
        assert!(!MIGRATION_004.is_empty());
        assert!(MIGRATION_004.contains("pipeline_jobs"));
    }

    #[test]
    fn migration_005_is_not_empty() {
        assert!(!MIGRATION_005.is_empty());
        assert!(MIGRATION_005.contains("ocr_formatted_text"));
    }

    #[test]
    fn migration_006_is_not_empty() {
        assert!(!MIGRATION_006.is_empty());
        assert!(MIGRATION_006.contains("page_layout_blocks"));
        assert!(MIGRATION_006.contains("document_extracted_fields"));
    }

    #[test]
    fn migration_007_is_not_empty() {
        assert!(!MIGRATION_007.is_empty());
        assert!(MIGRATION_007.contains("normalized_text"));
    }

    #[test]
    fn migration_008_is_not_empty() {
        assert!(!MIGRATION_008.is_empty());
        assert!(MIGRATION_008.contains("file_status"));
        assert!(MIGRATION_008.contains("revision_no"));
    }

    #[test]
    fn migration_009_is_not_empty() {
        assert!(!MIGRATION_009.is_empty());
        assert!(MIGRATION_009.contains("ocr_status"));
        assert!(MIGRATION_009.contains("page_number"));
    }

    #[test]
    fn migration_010_is_not_empty() {
        assert!(!MIGRATION_010.is_empty());
        assert!(MIGRATION_010.contains("document_sections"));
    }

    #[test]
    fn migration_011_is_not_empty() {
        assert!(!MIGRATION_011.is_empty());
        assert!(MIGRATION_011.contains("governed_events"));
    }

    #[test]
    fn migration_012_is_not_empty() {
        assert!(!MIGRATION_012.is_empty());
        assert!(MIGRATION_012.contains("document_ai_summaries"));
        assert!(MIGRATION_012.contains("document_scan_quality"));
    }

    #[test]
    fn migration_013_is_not_empty() {
        assert!(!MIGRATION_013.is_empty());
        assert!(MIGRATION_013.contains("ai_analysis_jobs"));
        assert!(MIGRATION_013.contains("document_but_luc"));
        assert!(MIGRATION_013.contains("case_people_profiles"));
    }

    #[test]
    fn migration_014_is_not_empty() {
        assert!(!MIGRATION_014.is_empty());
        assert!(MIGRATION_014.contains("source_pdf_path"));
        assert!(MIGRATION_014.contains("thumbnail_path"));
        assert!(MIGRATION_014.contains("current_order"));
        assert!(MIGRATION_014.contains("rotation"));
        assert!(MIGRATION_014.contains("is_removed"));
    }

    #[test]
    fn migration_015_is_not_empty() {
        assert!(!MIGRATION_015.is_empty());
        assert!(MIGRATION_015.contains("page_quality"));
        assert!(MIGRATION_015.contains("suggested_filename"));
    }

    #[test]
    fn migration_016_is_not_empty() {
        assert!(!MIGRATION_016.is_empty());
        assert!(MIGRATION_016.contains("document_groups"));
        assert!(MIGRATION_016.contains("group_id"));
        assert!(MIGRATION_016.contains("relative_path"));
    }

    #[test]
    fn init_runs_migration_in_memory() {
        let path = PathBuf::from(":memory:");
        let result = init(&path);
        assert!(result.is_ok(), "db init failed: {result:?}");
    }

    #[test]
    fn init_memory_has_required_tables() {
        let conn = Connection::open_in_memory().expect("open memory db");
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .expect("pragma");
        conn.execute_batch(MIGRATION_001).expect("migration 001");
        conn.execute_batch(MIGRATION_002).expect("migration 002");
        conn.execute_batch(MIGRATION_003).expect("migration 003");
        conn.execute_batch(MIGRATION_004).expect("migration 004");
        run_alter_migration(&conn, MIGRATION_005, "MIGRATION_005").expect("migration 005");
        conn.execute_batch(MIGRATION_006).expect("migration 006");
        run_alter_migration(&conn, MIGRATION_007, "MIGRATION_007").expect("migration 007");
        ensure_ocr_normalized_schema(&conn).expect("normalized schema");
        run_alter_migration(&conn, MIGRATION_008, "MIGRATION_008").expect("migration 008");
        run_alter_migration(&conn, MIGRATION_009, "MIGRATION_009").expect("migration 009");
        conn.execute_batch(MIGRATION_010).expect("migration 010");
        conn.execute_batch(MIGRATION_011).expect("migration 011");
        conn.execute_batch(MIGRATION_012).expect("migration 012");
        run_alter_migration(&conn, MIGRATION_013, "MIGRATION_013").expect("migration 013");
        run_alter_migration(&conn, MIGRATION_014, "MIGRATION_014").expect("migration 014");
        run_alter_migration(&conn, MIGRATION_015, "MIGRATION_015").expect("migration 015");
        run_alter_migration(&conn, MIGRATION_016, "MIGRATION_016").expect("migration 016");
        verify_required_schema(&conn).expect("required schema");
    }

    #[test]
    fn init_memory_has_page_image_columns() {
        let path = PathBuf::from(":memory:");
        init(&path).expect("db init");
        let conn = Connection::open_in_memory().expect("open memory db");
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .expect("pragma");
        conn.execute_batch(MIGRATION_001).expect("migration 001");
        conn.execute_batch(MIGRATION_002).expect("migration 002");
        conn.execute_batch(MIGRATION_003).expect("migration 003");
        conn.execute_batch(MIGRATION_004).expect("migration 004");
        run_alter_migration(&conn, MIGRATION_005, "MIGRATION_005").expect("migration 005");
        conn.execute_batch(MIGRATION_006).expect("migration 006");
        run_alter_migration(&conn, MIGRATION_007, "MIGRATION_007").expect("migration 007");
        ensure_ocr_normalized_schema(&conn).expect("normalized schema");
        run_alter_migration(&conn, MIGRATION_008, "MIGRATION_008").expect("migration 008");
        run_alter_migration(&conn, MIGRATION_009, "MIGRATION_009").expect("migration 009");
        conn.execute_batch(MIGRATION_010).expect("migration 010");
        conn.execute_batch(MIGRATION_011).expect("migration 011");
        conn.execute_batch(MIGRATION_012).expect("migration 012");
        run_alter_migration(&conn, MIGRATION_013, "MIGRATION_013").expect("migration 013");
        run_alter_migration(&conn, MIGRATION_014, "MIGRATION_014").expect("migration 014");
        run_alter_migration(&conn, MIGRATION_015, "MIGRATION_015").expect("migration 015");
        run_alter_migration(&conn, MIGRATION_016, "MIGRATION_016").expect("migration 016");
        verify_page_image_columns(&conn).expect("page image columns");
        verify_document_group_columns(&conn).expect("document group columns");
    }

    #[test]
    fn init_file_twice_is_idempotent() {
        let path = std::env::temp_dir().join(format!("vks_ecms_test_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        init(&path).expect("first init");
        {
            let conn = Connection::open(&path).expect("open after first init");
            verify_required_schema(&conn).expect("schema after first init");
        }
        init(&path).expect("second init");
        {
            let conn = Connection::open(&path).expect("open after second init");
            verify_required_schema(&conn).expect("schema after second init");
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn schema_verifier_reports_missing_required_table() {
        let conn = Connection::open_in_memory().expect("open memory db");
        conn.execute_batch(
            "CREATE TABLE cases(id TEXT);
             CREATE TABLE documents(id TEXT);",
        )
        .expect("partial schema");
        let err = verify_required_schema(&conn).expect_err("schema must be incomplete");
        assert!(
            err.contains("pages"),
            "missing pages must be reported: {err}"
        );
        assert!(
            err.contains("ai_analysis_jobs"),
            "missing ai_analysis_jobs must be reported: {err}"
        );
    }

    #[test]
    fn schema_verifier_reports_missing_page_image_columns() {
        let conn = Connection::open_in_memory().expect("open memory db");
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .expect("pragma");
        conn.execute_batch(MIGRATION_001).expect("migration 001");
        conn.execute_batch(MIGRATION_002).expect("migration 002");
        conn.execute_batch(MIGRATION_003).expect("migration 003");
        conn.execute_batch(MIGRATION_004).expect("migration 004");
        run_alter_migration(&conn, MIGRATION_005, "MIGRATION_005").expect("migration 005");
        conn.execute_batch(MIGRATION_006).expect("migration 006");
        run_alter_migration(&conn, MIGRATION_007, "MIGRATION_007").expect("migration 007");
        ensure_ocr_normalized_schema(&conn).expect("normalized schema");
        run_alter_migration(&conn, MIGRATION_008, "MIGRATION_008").expect("migration 008");
        run_alter_migration(&conn, MIGRATION_009, "MIGRATION_009").expect("migration 009");
        conn.execute_batch(MIGRATION_010).expect("migration 010");
        conn.execute_batch(MIGRATION_011).expect("migration 011");
        conn.execute_batch(MIGRATION_012).expect("migration 012");
        run_alter_migration(&conn, MIGRATION_013, "MIGRATION_013").expect("migration 013");
        run_alter_migration(&conn, MIGRATION_016, "MIGRATION_016").expect("migration 016");
        let err = verify_required_schema(&conn).expect_err("page image columns must be required");
        assert!(
            err.contains("DB_SCHEMA_REQUIRED_COLUMNS_MISSING:pages"),
            "missing page image columns must be reported: {err}"
        );
        assert!(
            err.contains("source_pdf_path") && err.contains("thumbnail_path"),
            "specific page image columns must be reported: {err}"
        );
    }
}
