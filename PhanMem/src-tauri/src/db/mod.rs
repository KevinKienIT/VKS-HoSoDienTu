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
    if let Err(e) = conn.execute_batch(MIGRATION_005) {
        let msg = e.to_string();
        if !msg.contains("duplicate column name") {
            return Err(format!("DB_INIT_MIGRATION_005_FAILED: {e}"));
        }
    }

    conn.execute_batch(MIGRATION_006)
        .map_err(|e| format!("DB_INIT_MIGRATION_006_FAILED: {e}"))?;

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
    fn init_runs_migration_in_memory() {
        let path = PathBuf::from(":memory:");
        let result = init(&path);
        assert!(result.is_ok());
    }
}
