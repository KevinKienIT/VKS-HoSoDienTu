// VKS ECMS — Case Tauri commands

use crate::commands::module_cmd::DbState;
use crate::storage;
use log::warn;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State};

static CASE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaseSummary {
    pub case_id: String,
    pub case_code: String,
    pub case_display_name: String,
    pub primary_person_name: String,
    pub case_type: String,
    pub dossier_type: String,
    pub but_luc: Option<String>,
    pub ocr_state: String,
    pub status: String,
    pub document_count: i32,
    pub missing_document_count: i32,
    pub total_pages: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCaseInput {
    pub case_display_name: String,
    pub primary_person_name: Option<String>,
    pub source_folder_name: Option<String>,
    pub case_type: Option<String>,
    pub prosecutor_office: Option<String>,
    pub investigator_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurgeImportedDossierInput {
    pub case_code: String,
    pub expected_case_identity: String,
    pub confirm_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurgeImportedDossierSummary {
    pub case_id: String,
    pub case_code: String,
    pub case_display_name: String,
    pub deleted_rows_by_table: BTreeMap<String, i64>,
    pub deleted_files_count: i64,
    pub deleted_cache_dirs_count: i64,
    pub errors: Vec<String>,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn generate_case_id() -> String {
    let n = CASE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("case-{}-{}", now_millis(), n)
}

fn generate_case_code() -> String {
    let n = CASE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("VK-{}-{:04}", now_millis(), n % 10_000)
}

fn normalize_non_empty(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn get_case_summary_by_id(
    conn: &rusqlite::Connection,
    case_id: &str,
) -> Result<CaseSummary, String> {
    let missing_document_count = count_missing_documents(conn, case_id)?;
    let dossier_type: String = conn
        .query_row(
            "SELECT COALESCE((
                SELECT d.document_type
                FROM documents d
                WHERE d.case_id = ?1
                GROUP BY d.document_type
                ORDER BY COUNT(*) DESC, d.document_type ASC
                LIMIT 1
            ), 'khong_xac_dinh')",
            params![case_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "khong_xac_dinh".to_string());
    let but_luc: Option<String> = conn
        .query_row(
            "SELECT field_value
             FROM document_extracted_fields
             WHERE document_id IN (SELECT document_id FROM documents WHERE case_id = ?1)
               AND field_name = 'but_luc'
               AND TRIM(COALESCE(field_value, '')) <> ''
             ORDER BY confidence DESC, created_at DESC
             LIMIT 1",
            params![case_id],
            |row| row.get(0),
        )
        .ok();
    let pending_ocr: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE case_id = ?1 AND status IN ('pending', 'ocr_processing')",
            params![case_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_docs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE case_id = ?1",
            params![case_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let ocr_state = if total_docs == 0 {
        "none".to_string()
    } else if pending_ocr > 0 {
        "pending".to_string()
    } else {
        "done".to_string()
    };
    conn.query_row(
        "SELECT
            case_id,
            case_code,
            case_display_name,
            primary_person_name,
            case_type,
            status,
            document_count,
            total_pages,
            created_at
         FROM cases
         WHERE case_id = ?1",
        [case_id],
        |row| {
            Ok(CaseSummary {
                case_id: row.get(0)?,
                case_code: row.get(1)?,
                case_display_name: row.get(2)?,
                primary_person_name: row.get(3)?,
                case_type: row.get(4)?,
                dossier_type: dossier_type.clone(),
                but_luc: but_luc.clone(),
                ocr_state: ocr_state.clone(),
                status: row.get(5)?,
                document_count: row.get(6)?,
                missing_document_count,
                total_pages: row.get(7)?,
                created_at: row.get(8)?,
            })
        },
    )
    .map_err(|e| format!("CASE_QUERY_FAILED: {e}"))
}

fn count_missing_documents(conn: &rusqlite::Connection, case_id: &str) -> Result<i32, String> {
    let mut stmt = conn
        .prepare("SELECT file_path FROM documents WHERE case_id = ?1")
        .map_err(|e| format!("CASE_MISSING_DOC_PREPARE_FAILED: {e}"))?;
    let rows = stmt
        .query_map(params![case_id], |row| row.get::<_, String>(0))
        .map_err(|e| format!("CASE_MISSING_DOC_QUERY_FAILED: {e}"))?;
    let mut missing = 0_i32;
    for row in rows {
        let file_path = row.map_err(|e| format!("CASE_MISSING_DOC_ROW_FAILED: {e}"))?;
        if !Path::new(&file_path).exists() {
            missing += 1;
        }
    }
    Ok(missing)
}

fn collect_string_column(
    conn: &rusqlite::Connection,
    sql: &str,
    bind_value: &str,
) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("CASE_PURGE_PREPARE_FAILED: {e}"))?;
    let rows = stmt
        .query_map(params![bind_value], |row| row.get::<_, String>(0))
        .map_err(|e| format!("CASE_PURGE_QUERY_FAILED: {e}"))?;
    let mut out = Vec::new();
    for row in rows {
        let value = row.map_err(|e| format!("CASE_PURGE_ROW_READ_FAILED: {e}"))?;
        if !value.trim().is_empty() {
            out.push(value);
        }
    }
    Ok(out)
}

fn is_lock_error(err: &io::Error) -> bool {
    if let Some(code) = err.raw_os_error() {
        return code == 32 || code == 33;
    }
    false
}

fn remove_file_with_retry(path: &Path, op: &str, errors: &mut Vec<String>) -> bool {
    let waits_ms = [40_u64, 120, 260];
    for (idx, wait_ms) in waits_ms.iter().enumerate() {
        match fs::remove_file(path) {
            Ok(_) => return true,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return false,
            Err(e) if is_lock_error(&e) => {
                warn!(
                    "file-lock path={} op={} attempt={} wait_ms={} final_error={}",
                    path.display(),
                    op,
                    idx + 1,
                    wait_ms,
                    e
                );
                if idx + 1 < waits_ms.len() {
                    thread::sleep(Duration::from_millis(*wait_ms));
                    continue;
                }
                let marker = path.with_extension(format!(
                    "{}.lockdelete.pending",
                    path.extension().and_then(|v| v.to_str()).unwrap_or("file")
                ));
                let _ = std::fs::write(
                    &marker,
                    format!(
                        "deferred_delete=1\npath={}\nreason=FILE_LOCK\n",
                        path.display()
                    ),
                );
                errors.push(format!(
                    "FILE_LOCK_DEFERRED_DELETE:path={}:op={}:attempt={}:wait_ms={}:final_error={}",
                    path.display(),
                    op,
                    idx + 1,
                    wait_ms,
                    e
                ));
                return false;
            }
            Err(e) => {
                errors.push(format!(
                    "FILE_DELETE_FAILED:path={}:op={}:attempt={}:wait_ms=0:final_error={}",
                    path.display(),
                    op,
                    idx + 1,
                    e
                ));
                return false;
            }
        }
    }
    false
}

fn try_delete_file(path: &Path, errors: &mut Vec<String>) -> bool {
    remove_file_with_retry(path, "purge.remove_file", errors)
}

fn try_delete_empty_dir(path: &Path, errors: &mut Vec<String>) {
    match fs::remove_dir(path) {
        Ok(_) => {}
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound
                || e.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
        Err(e) => errors.push(format!("DIR_DELETE_FAILED:{}:{e}", path.display())),
    }
}

fn try_delete_cache_dir_under(base: &Path, path: &Path, errors: &mut Vec<String>) -> bool {
    if !path.exists() {
        return false;
    }

    let base_abs = match base.canonicalize() {
        Ok(value) => value,
        Err(e) => {
            errors.push(format!(
                "CACHE_BASE_CANONICALIZE_FAILED:{}:{e}",
                base.display()
            ));
            return false;
        }
    };
    let path_abs = match path.canonicalize() {
        Ok(value) => value,
        Err(e) => {
            errors.push(format!(
                "CACHE_DIR_CANONICALIZE_FAILED:{}:{e}",
                path.display()
            ));
            return false;
        }
    };

    if !path_abs.starts_with(&base_abs) {
        errors.push(format!(
            "CACHE_DIR_DELETE_BLOCKED_OUTSIDE_APPDATA:{}",
            path_abs.display()
        ));
        return false;
    }

    let waits_ms = [40_u64, 120, 260];
    for (idx, wait_ms) in waits_ms.iter().enumerate() {
        match fs::remove_dir_all(&path_abs) {
            Ok(_) => return true,
            Err(e) if is_lock_error(&e) => {
                warn!(
                    "file-lock path={} op=purge.remove_dir_all attempt={} wait_ms={} final_error={}",
                    path_abs.display(),
                    idx + 1,
                    wait_ms,
                    e
                );
                if idx + 1 < waits_ms.len() {
                    thread::sleep(Duration::from_millis(*wait_ms));
                    continue;
                }
                errors.push(format!(
                    "CACHE_DIR_DELETE_LOCKED:path={}:op=purge.remove_dir_all:attempt={}:wait_ms={}:final_error={}",
                    path_abs.display(),
                    idx + 1,
                    wait_ms,
                    e
                ));
                return false;
            }
            Err(e) => {
                errors.push(format!(
                    "CACHE_DIR_DELETE_FAILED:path={}:op=purge.remove_dir_all:attempt={}:wait_ms=0:final_error={}",
                    path_abs.display(),
                    idx + 1,
                    e
                ));
                return false;
            }
        }
    }
    false
}

#[tauri::command]
pub fn purge_imported_dossier(
    app: AppHandle,
    db: State<'_, DbState>,
    input: PurgeImportedDossierInput,
) -> Result<PurgeImportedDossierSummary, String> {
    let case_code = input.case_code.trim();
    let expected_identity = input.expected_case_identity.trim();
    let token = input.confirm_token.trim();

    if case_code.is_empty() {
        return Err("CASE_PURGE_INVALID_INPUT: case_code must not be empty".to_string());
    }
    if token != "ok" {
        return Err(
            "CASE_PURGE_CONFIRMATION_FAILED: confirmation token must be exactly ok".to_string(),
        );
    }

    let mut conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let (case_id, case_code_db, case_display_name): (String, String, String) = conn
        .query_row(
            "SELECT case_id, case_code, case_display_name FROM cases WHERE case_code = ?1",
            params![case_code],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("CASE_PURGE_CASE_NOT_FOUND_OR_QUERY_FAILED: {e}"))?;

    let canonical_identity = format!("{} - {}", case_code_db, case_display_name);
    if expected_identity != canonical_identity {
        return Err(
            "CASE_PURGE_CONFIRMATION_FAILED: expected_case_identity must exactly match dossier label"
                .to_string(),
        );
    }

    let mut errors: Vec<String> = Vec::new();
    let mut files_to_delete: HashSet<PathBuf> = HashSet::new();
    let document_ids = collect_string_column(
        &conn,
        "SELECT document_id FROM documents WHERE case_id = ?1",
        &case_id,
    )?;

    for value in collect_string_column(
        &conn,
        "SELECT file_path FROM documents WHERE case_id = ?1",
        &case_id,
    )? {
        files_to_delete.insert(PathBuf::from(value));
    }
    for value in collect_string_column(
        &conn,
        "SELECT image_path FROM pages WHERE document_id IN (SELECT document_id FROM documents WHERE case_id = ?1) AND COALESCE(image_path, '') <> ''",
        &case_id,
    )? {
        files_to_delete.insert(PathBuf::from(value));
    }
    let mut deleted_rows_by_table: BTreeMap<String, i64> = BTreeMap::new();
    let tx = conn
        .transaction()
        .map_err(|e| format!("CASE_PURGE_TX_BEGIN_FAILED: {e}"))?;

    let mut affected = tx
        .execute(
            "DELETE FROM review_queue
             WHERE (object_type = 'document' AND object_id IN (SELECT document_id FROM documents WHERE case_id = ?1))
                OR (object_type = 'page' AND object_id IN (
                    SELECT p.page_id FROM pages p JOIN documents d ON d.document_id = p.document_id WHERE d.case_id = ?1
                ))
                OR (object_type = 'ocr_result' AND object_id IN (
                    SELECT o.ocr_result_id
                    FROM ocr_results o
                    JOIN pages p ON p.page_id = o.page_id
                    JOIN documents d ON d.document_id = p.document_id
                    WHERE d.case_id = ?1
                ))",
            params![case_id.clone()],
        )
        .map_err(|e| format!("CASE_PURGE_DELETE_REVIEW_QUEUE_FAILED: {e}"))? as i64;
    deleted_rows_by_table.insert("review_queue".to_string(), affected);

    affected = tx
        .execute(
            "DELETE FROM audit_events
             WHERE (object_type = 'case' AND object_id = ?1)
                OR (object_type = 'document' AND object_id IN (SELECT document_id FROM documents WHERE case_id = ?1))
                OR (object_type = 'page' AND object_id IN (
                    SELECT p.page_id FROM pages p JOIN documents d ON d.document_id = p.document_id WHERE d.case_id = ?1
                ))",
            params![case_id.clone()],
        )
        .map_err(|e| format!("CASE_PURGE_DELETE_AUDIT_EVENTS_FAILED: {e}"))? as i64;
    deleted_rows_by_table.insert("audit_events".to_string(), affected);

    affected = tx
        .execute(
            "DELETE FROM scan_jobs WHERE case_id = ?1",
            params![case_id.clone()],
        )
        .map_err(|e| format!("CASE_PURGE_DELETE_SCAN_JOBS_FAILED: {e}"))? as i64;
    deleted_rows_by_table.insert("scan_jobs".to_string(), affected);

    affected =
        tx.execute(
            "DELETE FROM catalog_entries WHERE case_id = ?1",
            params![case_id.clone()],
        )
        .map_err(|e| format!("CASE_PURGE_DELETE_CATALOG_ENTRIES_FAILED: {e}"))? as i64;
    deleted_rows_by_table.insert("catalog_entries".to_string(), affected);

    affected =
        tx.execute(
            "DELETE FROM page_layout_blocks
             WHERE document_id IN (SELECT document_id FROM documents WHERE case_id = ?1)",
            params![case_id.clone()],
        )
        .map_err(|e| format!("CASE_PURGE_DELETE_PAGE_LAYOUT_BLOCKS_FAILED: {e}"))? as i64;
    deleted_rows_by_table.insert("page_layout_blocks".to_string(), affected);

    affected = tx
        .execute(
            "DELETE FROM document_extracted_fields
             WHERE document_id IN (SELECT document_id FROM documents WHERE case_id = ?1)",
            params![case_id.clone()],
        )
        .map_err(|e| format!("CASE_PURGE_DELETE_DOCUMENT_EXTRACTED_FIELDS_FAILED: {e}"))?
        as i64;
    deleted_rows_by_table.insert("document_extracted_fields".to_string(), affected);

    affected = tx
        .execute(
            "DELETE FROM import_jobs WHERE case_id = ?1",
            params![case_id.clone()],
        )
        .map_err(|e| format!("CASE_PURGE_DELETE_IMPORT_JOBS_FAILED: {e}"))? as i64;
    deleted_rows_by_table.insert("import_jobs".to_string(), affected);

    affected = tx
        .execute(
            "DELETE FROM work_products WHERE case_id = ?1",
            params![case_id.clone()],
        )
        .map_err(|e| format!("CASE_PURGE_DELETE_WORK_PRODUCTS_FAILED: {e}"))? as i64;
    deleted_rows_by_table.insert("work_products".to_string(), affected);

    affected = tx
        .execute(
            "DELETE FROM cases WHERE case_id = ?1",
            params![case_id.clone()],
        )
        .map_err(|e| format!("CASE_PURGE_DELETE_CASE_FAILED: {e}"))? as i64;
    deleted_rows_by_table.insert("cases".to_string(), affected);

    let _ = tx.execute(
        "INSERT INTO fts_documents(fts_documents) VALUES('rebuild')",
        [],
    );
    let _ = tx.execute("INSERT INTO fts_pages(fts_pages) VALUES('rebuild')", []);
    let _ = tx.execute(
        "INSERT INTO fts_layout_blocks(fts_layout_blocks) VALUES('rebuild')",
        [],
    );
    let _ = tx.execute(
        "INSERT INTO fts_extracted_fields(fts_extracted_fields) VALUES('rebuild')",
        [],
    );

    tx.commit()
        .map_err(|e| format!("CASE_PURGE_TX_COMMIT_FAILED: {e}"))?;

    let mut deleted_files_count = 0_i64;
    let mut deleted_cache_dirs_count = 0_i64;
    let mut parent_dirs: HashSet<PathBuf> = HashSet::new();

    for path in files_to_delete {
        if !storage::is_managed_path(&path) {
            errors.push(format!("SKIP_UNMANAGED_FILE_DELETE:{}", path.display()));
            continue;
        }
        if try_delete_file(&path, &mut errors) {
            deleted_files_count += 1;
            if let Some(parent) = path.parent() {
                parent_dirs.insert(parent.to_path_buf());
            }
        }
    }

    for dir in parent_dirs {
        try_delete_empty_dir(&dir, &mut errors);
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("CASE_PURGE_APP_DATA_DIR_FAILED: {e}"))?;
    let legacy_ocr_pages_dir = app_data_dir.join("ocr_pages");
    let managed_processed_dir = storage::processed_dir()?;
    let managed_ocr_pages_dir = managed_processed_dir.join("ocr_pages");
    for document_id in document_ids {
        let cache_dir = legacy_ocr_pages_dir.join(&document_id);
        if try_delete_cache_dir_under(&app_data_dir, &cache_dir, &mut errors) {
            deleted_cache_dirs_count += 1;
        }
        let managed_cache_dir = managed_ocr_pages_dir.join(&document_id);
        if try_delete_cache_dir_under(&managed_processed_dir, &managed_cache_dir, &mut errors) {
            deleted_cache_dirs_count += 1;
        }
    }

    Ok(PurgeImportedDossierSummary {
        case_id,
        case_code: case_code_db,
        case_display_name,
        deleted_rows_by_table,
        deleted_files_count,
        deleted_cache_dirs_count,
        errors,
    })
}

#[tauri::command]
pub fn list_cases(db: State<'_, DbState>) -> Result<Vec<CaseSummary>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT
                case_id,
                case_code,
                case_display_name,
                primary_person_name,
                case_type,
                status,
                document_count,
                total_pages,
                created_at
             FROM cases
             ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Prepare failed: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CaseSummary {
                case_id: row.get(0)?,
                case_code: row.get(1)?,
                case_display_name: row.get(2)?,
                primary_person_name: row.get(3)?,
                case_type: row.get(4)?,
                dossier_type: "khong_xac_dinh".to_string(),
                but_luc: None,
                ocr_state: "none".to_string(),
                status: row.get(5)?,
                document_count: row.get(6)?,
                missing_document_count: 0,
                total_pages: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut items = Vec::new();
    for row in rows {
        let mut item = row.map_err(|e| format!("Row read failed: {e}"))?;
        item.missing_document_count = count_missing_documents(&conn, &item.case_id)?;
        item.dossier_type = conn
            .query_row(
                "SELECT COALESCE((
                    SELECT d.document_type
                    FROM documents d
                    WHERE d.case_id = ?1
                    GROUP BY d.document_type
                    ORDER BY COUNT(*) DESC, d.document_type ASC
                    LIMIT 1
                ), 'khong_xac_dinh')",
                params![item.case_id.clone()],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "khong_xac_dinh".to_string());
        item.but_luc = conn
            .query_row(
                "SELECT field_value
                 FROM document_extracted_fields
                 WHERE document_id IN (SELECT document_id FROM documents WHERE case_id = ?1)
                   AND field_name = 'but_luc'
                   AND TRIM(COALESCE(field_value, '')) <> ''
                 ORDER BY confidence DESC, created_at DESC
                 LIMIT 1",
                params![item.case_id.clone()],
                |r| r.get(0),
            )
            .ok();
        let pending_ocr: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM documents WHERE case_id = ?1 AND status IN ('pending', 'ocr_processing')",
                params![item.case_id.clone()],
                |r| r.get(0),
            )
            .unwrap_or(0);
        item.ocr_state = if item.document_count == 0 {
            "none".to_string()
        } else if pending_ocr > 0 {
            "pending".to_string()
        } else {
            "done".to_string()
        };
        items.push(item);
    }
    Ok(items)
}

#[tauri::command]
pub fn get_case(db: State<'_, DbState>, case_id: String) -> Result<CaseSummary, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    get_case_summary_by_id(&conn, &case_id)
}

#[tauri::command]
pub fn create_case(db: State<'_, DbState>, input: CreateCaseInput) -> Result<CaseSummary, String> {
    let display_name = normalize_non_empty(&input.case_display_name, "Hồ sơ mới");
    let primary_person = normalize_non_empty(
        input.primary_person_name.as_deref().unwrap_or_default(),
        &display_name,
    );
    let source_folder_name = normalize_non_empty(
        input.source_folder_name.as_deref().unwrap_or_default(),
        &display_name,
    );
    let case_type = normalize_non_empty(
        input.case_type.as_deref().unwrap_or_default(),
        "to_dieu_tra",
    );

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let case_id = generate_case_id();
    let case_code = generate_case_code();

    conn.execute(
        "INSERT INTO cases (
            case_id,
            case_code,
            case_display_name,
            source_folder_name,
            primary_person_name,
            case_type,
            prosecutor_office,
            investigator_name,
            status,
            document_count,
            total_pages,
            created_at,
            updated_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
            'active', 0, 0,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            case_id,
            case_code,
            display_name,
            source_folder_name,
            primary_person,
            case_type,
            input.prosecutor_office,
            input.investigator_name,
        ],
    )
    .map_err(|e| format!("CASE_CREATE_FAILED: {e}"))?;

    get_case_summary_by_id(&conn, &case_id)
}
