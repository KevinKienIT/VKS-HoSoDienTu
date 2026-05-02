// VKS ECMS — Catalog Tauri commands

use crate::commands::module_cmd::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

const ALLOWED_EXTENSIONS: &[&str] = &[
    ".pdf", ".png", ".jpg", ".jpeg", ".tif", ".tiff", ".bmp", ".doc", ".docx", ".xls",
    ".xlsx", ".ppt", ".pptx", ".rtf",
];
const ALLOWED_SCAN_STATUS: &[&str] = &["discovered", "cataloged", "imported", "skipped", "error"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatalogEntry {
    pub catalog_entry_id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_ext: String,
    pub file_size: i64,
    pub modified_at: String,
    pub file_hash: Option<String>,
    pub parent_folder: String,
    pub case_id: Option<String>,
    pub scan_status: String,
    pub scan_note: Option<String>,
    pub scanned_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatalogStats {
    pub total_files: i64,
    pub total_size_bytes: i64,
    pub by_extension: HashMap<String, i64>,
    pub by_folder: HashMap<String, i64>,
    pub by_status: HashMap<String, i64>,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn seconds_since_epoch(st: SystemTime) -> String {
    st.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn generate_catalog_id() -> String {
    let n = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("cat-{}-{}", now_millis(), n)
}

fn is_supported_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_ascii_lowercase()));

    match ext {
        Some(value) => ALLOWED_EXTENSIONS.contains(&value.as_str()),
        None => false,
    }
}

fn collect_supported_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_supported_files(&path, out);
            } else if path.is_file() && is_supported_file(&path) {
                out.push(path);
            }
        }
    }
}

fn hash_file(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path)
        .map_err(|e| format!("CATALOG_HASH_OPEN_FAILED {}: {e}", path.display()))?;

    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|e| format!("CATALOG_HASH_READ_FAILED {}: {e}", path.display()))?;

        if read == 0 {
            break;
        }

        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[tauri::command]
pub fn get_catalog_entries(
    db: State<'_, DbState>,
    status: Option<String>,
) -> Result<Vec<CatalogEntry>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let mut entries: Vec<CatalogEntry> = Vec::new();

    if let Some(status_filter) = status {
        let mut stmt = conn
            .prepare(
                "SELECT
                    catalog_entry_id, file_path, file_name, file_ext, file_size,
                    modified_at, file_hash, parent_folder, case_id, scan_status,
                    scan_note, scanned_at
                 FROM catalog_entries
                 WHERE scan_status = ?1
                 ORDER BY scanned_at DESC, file_name ASC",
            )
            .map_err(|e| format!("Prepare failed: {e}"))?;

        let rows = stmt
            .query_map([status_filter], |row| {
                Ok(CatalogEntry {
                    catalog_entry_id: row.get(0)?,
                    file_path: row.get(1)?,
                    file_name: row.get(2)?,
                    file_ext: row.get(3)?,
                    file_size: row.get(4)?,
                    modified_at: row.get(5)?,
                    file_hash: row.get(6)?,
                    parent_folder: row.get(7)?,
                    case_id: row.get(8)?,
                    scan_status: row.get(9)?,
                    scan_note: row.get(10)?,
                    scanned_at: row.get(11)?,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            entries.push(row.map_err(|e| format!("Row read failed: {e}"))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT
                    catalog_entry_id, file_path, file_name, file_ext, file_size,
                    modified_at, file_hash, parent_folder, case_id, scan_status,
                    scan_note, scanned_at
                 FROM catalog_entries
                 ORDER BY scanned_at DESC, file_name ASC",
            )
            .map_err(|e| format!("Prepare failed: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(CatalogEntry {
                    catalog_entry_id: row.get(0)?,
                    file_path: row.get(1)?,
                    file_name: row.get(2)?,
                    file_ext: row.get(3)?,
                    file_size: row.get(4)?,
                    modified_at: row.get(5)?,
                    file_hash: row.get(6)?,
                    parent_folder: row.get(7)?,
                    case_id: row.get(8)?,
                    scan_status: row.get(9)?,
                    scan_note: row.get(10)?,
                    scanned_at: row.get(11)?,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            entries.push(row.map_err(|e| format!("Row read failed: {e}"))?);
        }
    }

    Ok(entries)
}

#[tauri::command]
pub fn get_catalog_stats(db: State<'_, DbState>) -> Result<CatalogStats, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let (total_files, total_size_bytes) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(file_size), 0) FROM catalog_entries",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .map_err(|e| format!("Query total stats failed: {e}"))?;

    let mut by_extension: HashMap<String, i64> = HashMap::new();
    let mut ext_stmt = conn
        .prepare("SELECT file_ext, COUNT(*) FROM catalog_entries GROUP BY file_ext")
        .map_err(|e| format!("Prepare extension stats failed: {e}"))?;
    let ext_rows = ext_stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| format!("Query extension stats failed: {e}"))?;
    for row in ext_rows {
        let (key, count) = row.map_err(|e| format!("Read extension stats row failed: {e}"))?;
        by_extension.insert(key, count);
    }

    let mut by_folder: HashMap<String, i64> = HashMap::new();
    let mut folder_stmt = conn
        .prepare("SELECT parent_folder, COUNT(*) FROM catalog_entries GROUP BY parent_folder")
        .map_err(|e| format!("Prepare folder stats failed: {e}"))?;
    let folder_rows = folder_stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| format!("Query folder stats failed: {e}"))?;
    for row in folder_rows {
        let (key, count) = row.map_err(|e| format!("Read folder stats row failed: {e}"))?;
        by_folder.insert(key, count);
    }

    let mut by_status: HashMap<String, i64> = HashMap::new();
    for status in ALLOWED_SCAN_STATUS {
        by_status.insert((*status).to_string(), 0);
    }
    let mut status_stmt = conn
        .prepare("SELECT scan_status, COUNT(*) FROM catalog_entries GROUP BY scan_status")
        .map_err(|e| format!("Prepare status stats failed: {e}"))?;
    let status_rows = status_stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| format!("Query status stats failed: {e}"))?;
    for row in status_rows {
        let (key, count) = row.map_err(|e| format!("Read status stats row failed: {e}"))?;
        by_status.insert(key, count);
    }

    Ok(CatalogStats {
        total_files,
        total_size_bytes,
        by_extension,
        by_folder,
        by_status,
    })
}

#[tauri::command]
pub fn scan_folder_catalog(db: State<'_, DbState>, folder_path: String) -> Result<i64, String> {
    let root = PathBuf::from(folder_path);
    if !root.exists() {
        return Err("CATALOG_SCAN_FOLDER_NOT_FOUND".to_string());
    }
    if !root.is_dir() {
        return Err("CATALOG_SCAN_FOLDER_NOT_DIRECTORY".to_string());
    }

    let mut files: Vec<PathBuf> = Vec::new();
    collect_supported_files(&root, &mut files);

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut upserted: i64 = 0;

    for path in files {
        let metadata = match fs::metadata(&path) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let file_path = path.to_string_lossy().to_string();
        let file_name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
        let file_ext = path
            .extension()
            .and_then(|v| v.to_str())
            .map(|e| format!(".{}", e.to_ascii_lowercase()))
            .unwrap_or_default();
        let file_size = metadata.len() as i64;
        let modified_at = metadata
            .modified()
            .ok()
            .map(seconds_since_epoch)
            .unwrap_or_else(|| "0".to_string());
        let parent_folder = path
            .parent()
            .and_then(|v| v.file_name())
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();

        let (file_hash, scan_status, scan_note) = match hash_file(&path) {
            Ok(hash) => (Some(hash), "cataloged".to_string(), None),
            Err(err) => (None, "error".to_string(), Some(err)),
        };

        let changed = conn
            .execute(
                "INSERT INTO catalog_entries (
                    catalog_entry_id,
                    file_path,
                    file_name,
                    file_ext,
                    file_size,
                    modified_at,
                    file_hash,
                    parent_folder,
                    scan_status,
                    scan_note,
                    scanned_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9, ?10,
                    strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                )
                ON CONFLICT(file_path) DO UPDATE SET
                    file_name = excluded.file_name,
                    file_ext = excluded.file_ext,
                    file_size = excluded.file_size,
                    modified_at = excluded.modified_at,
                    file_hash = excluded.file_hash,
                    parent_folder = excluded.parent_folder,
                    scan_status = excluded.scan_status,
                    scan_note = excluded.scan_note,
                    scanned_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
                params![
                    generate_catalog_id(),
                    file_path,
                    file_name,
                    file_ext,
                    file_size,
                    modified_at,
                    file_hash,
                    parent_folder,
                    scan_status,
                    scan_note,
                ],
            )
            .map_err(|e| format!("CATALOG_UPSERT_FAILED: {e}"))?;

        upserted += changed as i64;
    }

    Ok(upserted)
}

#[tauri::command]
pub fn update_catalog_entry_status(
    db: State<'_, DbState>,
    entry_id: String,
    status: String,
    note: Option<String>,
) -> Result<(), String> {
    if !ALLOWED_SCAN_STATUS.contains(&status.as_str()) {
        return Err("CATALOG_INVALID_STATUS".to_string());
    }

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let changed = conn
        .execute(
            "UPDATE catalog_entries
             SET scan_status = ?1,
                 scan_note = ?2,
                 scanned_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE catalog_entry_id = ?3",
            params![status, note, entry_id],
        )
        .map_err(|e| format!("CATALOG_UPDATE_STATUS_FAILED: {e}"))?;

    if changed == 0 {
        return Err("CATALOG_ENTRY_NOT_FOUND".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn delete_catalog_entry(db: State<'_, DbState>, entry_id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let changed = conn
        .execute(
            "DELETE FROM catalog_entries WHERE catalog_entry_id = ?1",
            params![entry_id],
        )
        .map_err(|e| format!("CATALOG_DELETE_FAILED: {e}"))?;

    if changed == 0 {
        return Err("CATALOG_ENTRY_NOT_FOUND".to_string());
    }

    Ok(())
}
