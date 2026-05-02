// VKS ECMS — Import folder commands

use crate::commands::module_cmd::DbState;
use crate::storage;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

static IMPORT_COUNTER: AtomicU64 = AtomicU64::new(0);

const ALLOWED_EXTENSIONS: &[&str] = &[
    "pdf", "png", "jpg", "jpeg", "tif", "tiff", "bmp", "doc", "docx", "xls", "xlsx", "ppt",
    "pptx", "rtf",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportFolderInput {
    pub folder_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportedFile {
    pub document_id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_ext: String,
    pub file_size: i64,
    pub page_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportFolderResult {
    pub job_id: String,
    pub case_id: String,
    pub total_files: i64,
    pub files: Vec<ImportedFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportMultipleFilesResult {
    pub case_id: String,
    pub total_files: i64,
    pub imported: Vec<ImportedFile>,
    pub duplicates: Vec<String>,
    pub errors: Vec<String>,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn generate_import_job_id() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("imp-{}-{}", now_millis(), n)
}

fn generate_case_id() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("case-{}-{}", now_millis(), n)
}

fn generate_case_code() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("VK-{}-{:04}", now_millis(), n % 10_000)
}

fn generate_document_id() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("doc-{}-{}", now_millis(), n)
}

fn generate_page_id() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("page-{}-{}", now_millis(), n)
}

fn generate_review_id() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("review-{}-{}", now_millis(), n)
}

fn normalize_non_empty(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_display_label(value: &str) -> String {
    value.replace('_', " ").replace('-', " ").trim().to_string()
}

fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ALLOWED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn is_image_extension(ext: &str) -> bool {
    matches!(ext, "png" | "jpg" | "jpeg" | "tif" | "tiff" | "bmp")
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

fn classify_document_type_from_name(file_name: &str) -> String {
    let n = file_name.to_lowercase();
    if n.contains("to khai") || n.contains("lời khai") || n.contains("loi khai") {
        "to_khai".to_string()
    } else if n.contains("bien ban") || n.contains("biên bản") {
        "bien_ban".to_string()
    } else if n.contains("quyet dinh") || n.contains("quyết định") || n.contains("qd") {
        "quyet_dinh".to_string()
    } else if n.contains("ket luan") || n.contains("kết luận") {
        "ket_luan".to_string()
    } else if n.contains("giay") || n.contains("phieu") {
        "phieu".to_string()
    } else {
        "khong_xac_dinh".to_string()
    }
}

fn build_display_name(file_name: &str, document_type: &str) -> String {
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or(file_name)
        .to_string();
    let normalized = normalize_display_label(&stem);
    if normalized.is_empty() {
        format!("Tài liệu {}", document_type)
    } else {
        normalized
    }
}

fn build_summary_short(display_name: &str, document_type: &str) -> String {
    format!("{} ({})", display_name, document_type)
}

fn build_summary_detail(display_name: &str, document_type: &str, file_path: &str) -> String {
    format!(
        "Tài liệu '{}' được phân loại '{}' từ nguồn file '{}'.",
        display_name, document_type, file_path
    )
}

fn to_json_array(items: &[String]) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let escaped = items
        .iter()
        .map(|s| s.replace('\\', "\\\\").replace('"', "\\\""))
        .map(|s| format!("\"{}\"", s))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{}]", escaped)
}

fn count_pdf_pages(path: &Path) -> i32 {
    match lopdf::Document::load(path) {
        Ok(doc) => doc.get_pages().len() as i32,
        Err(_) => 0,
    }
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("HASH_OPEN_FAILED {}: {e}", path.to_string_lossy()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("HASH_READ_FAILED {}: {e}", path.to_string_lossy()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn insert_page_row(
    conn: &rusqlite::Connection,
    document_id: &str,
    page_index: i32,
    image_path: Option<&str>,
) -> Result<String, String> {
    let page_id = generate_page_id();
    conn.execute(
        "INSERT INTO pages (page_id, document_id, page_index, image_path, created_at)
         VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
        params![page_id, document_id, page_index, image_path],
    )
    .map_err(|e| format!("INSERT_PAGE_FAILED: {e}"))?;
    Ok(page_id)
}

fn insert_ocr_result(
    conn: &rusqlite::Connection,
    page_id: &str,
    ocr_text: &str,
    confidence: f64,
    engine: &str,
) -> Result<String, String> {
    let ocr_result_id = generate_page_id();
    conn.execute(
        "INSERT INTO ocr_results (
            ocr_result_id,
            page_id,
            engine,
            raw_text,
            confidence,
            created_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![ocr_result_id, page_id, engine, ocr_text, confidence],
    )
    .map_err(|e| format!("INSERT_OCR_FAILED: {e}"))?;
    Ok(ocr_result_id)
}

fn create_review_entry(
    conn: &rusqlite::Connection,
    document_id: &str,
    review_reason: &str,
) -> Result<(), String> {
    let review_id = generate_review_id();
    conn.execute(
        "INSERT INTO review_queue (
            review_id,
            object_type,
            object_id,
            reason,
            priority,
            status,
            created_at
         ) VALUES (
            ?1,
            'document',
            ?2,
            ?3,
            0,
            'pending',
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![review_id, document_id, review_reason],
    )
    .map_err(|e| format!("CREATE_REVIEW_ENTRY_FAILED: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn import_folder(
    db: State<'_, DbState>,
    folder_path: String,
) -> Result<ImportFolderResult, String> {
    let input = ImportFolderInput { folder_path };
    let folder_input = input.folder_path.trim();
    if folder_input.is_empty() {
        return Err("IMPORT_FOLDER_PATH_EMPTY".to_string());
    }

    let root = PathBuf::from(folder_input);
    if !root.exists() {
        return Err("IMPORT_FOLDER_NOT_FOUND".to_string());
    }
    if !root.is_dir() {
        return Err("IMPORT_FOLDER_NOT_DIRECTORY".to_string());
    }

    let mut files = Vec::<PathBuf>::new();
    collect_supported_files(&root, &mut files);
    files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    if files.is_empty() {
        return Err("IMPORT_FOLDER_NO_SUPPORTED_FILES: chỉ nhận PDF, ảnh scan hoặc file Office".to_string());
    }

    let source_folder_name_raw = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("HoSoNhap")
        .to_string();
    let source_folder_name = normalize_non_empty(&source_folder_name_raw, "HoSoNhap");
    let case_display_name = normalize_non_empty(
        &normalize_display_label(&source_folder_name),
        "Hồ sơ import",
    );
    let primary_person_name = normalize_non_empty(&case_display_name, "Hồ sơ import");

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
            status,
            document_count,
            total_pages,
            created_at,
            updated_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            'to_dieu_tra',
            'active',
            0,
            0,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            case_id,
            case_code,
            case_display_name,
            source_folder_name,
            primary_person_name,
        ],
    )
    .map_err(|e| format!("IMPORT_CREATE_CASE_FAILED: {e}"))?;

    let job_id = generate_import_job_id();
    let total_files = files.len() as i64;
    conn.execute(
        "INSERT INTO import_jobs (
            import_job_id,
            case_id,
            source_folder,
            job_status,
            total_files,
            processed_files,
            failed_files,
            error_log,
            started_at,
            updated_at
         ) VALUES (
            ?1, ?2, ?3,
            'importing',
            ?4,
            0,
            0,
            '[]',
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![job_id, case_id, folder_input, total_files],
    )
    .map_err(|e| format!("IMPORT_CREATE_JOB_FAILED: {e}"))?;

    let mut imported_files = Vec::<ImportedFile>::new();
    let mut errors = Vec::<String>::new();
    let mut failed_files: i64 = 0;
    let mut processed_files: i64 = 0;
    let mut total_pages: i64 = 0;
    let mut last_success_document_id: Option<String> = None;

    for path in files {
        processed_files += 1;

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                failed_files += 1;
                errors.push(format!("METADATA_FAILED {}: {e}", path.to_string_lossy()));
                let _ = conn.execute(
                    "UPDATE import_jobs
                     SET processed_files = ?1,
                         failed_files = ?2,
                         current_file = ?3,
                         updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                     WHERE import_job_id = ?4",
                    params![
                        processed_files,
                        failed_files,
                        path.to_string_lossy().to_string(),
                        job_id,
                    ],
                );
                continue;
            }
        };

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("tai_lieu")
            .to_string();
        let ext_no_dot = path
            .extension()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let file_ext = if ext_no_dot.is_empty() {
            "".to_string()
        } else {
            format!(".{ext_no_dot}")
        };
        let file_size = metadata.len() as i64;
        let page_count = if ext_no_dot == "pdf" {
            count_pdf_pages(&path)
        } else {
            1
        };

        let document_id = generate_document_id();
        let managed_path =
            match storage::copy_to_originals(&path, &case_code, &document_id, &file_name) {
                Ok(value) => value,
                Err(e) => {
                    failed_files += 1;
                    errors.push(e);
                    continue;
                }
            };
        let file_path = managed_path.to_string_lossy().to_string();
        let document_type = classify_document_type_from_name(&file_name);
        let display_name = build_display_name(&file_name, &document_type);
        let summary_short = build_summary_short(&display_name, &document_type);
        let summary_detail = build_summary_detail(&display_name, &document_type, &file_path);

        let insert_result = conn.execute(
            "INSERT INTO documents (
                document_id,
                case_id,
                original_filename,
                file_path,
                file_hash,
                file_size,
                page_count,
                display_name,
                document_title,
                document_type,
                summary_short,
                summary_detail,
                status,
                created_at,
                updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4,
                '', ?5, ?6, ?7, ?7, ?8, ?9, ?10,
                'pending',
                strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             )",
            params![
                document_id,
                case_id,
                file_name,
                file_path,
                file_size,
                page_count,
                display_name,
                document_type,
                summary_short,
                summary_detail,
            ],
        );

        match insert_result {
            Ok(_) => {
                if ext_no_dot == "pdf" {
                    for i in 1..=page_count {
                        if let Ok(page_id) = insert_page_row(&conn, &document_id, i, None) {
                            let _ = insert_ocr_result(&conn, &page_id, "", 0.0, "pending");
                        }
                    }
                } else if is_image_extension(&ext_no_dot) {
                    if let Ok(page_id) = insert_page_row(&conn, &document_id, 1, Some(&file_path)) {
                        let _ = insert_ocr_result(&conn, &page_id, "", 0.0, "pending");
                    }
                } else if let Ok(page_id) = insert_page_row(&conn, &document_id, 1, None) {
                    let _ = insert_ocr_result(&conn, &page_id, "", 0.0, "unsupported_office");
                }

                let _ = conn.execute(
                    "INSERT INTO fts_documents (rowid, document_id, display_name, document_title, summary_short, summary_detail)
                     SELECT rowid, document_id, display_name, document_title, summary_short, summary_detail
                     FROM documents WHERE document_id = ?1",
                    params![document_id],
                );

                let _ = create_review_entry(&conn, &document_id, "ocr_pending_after_import");

                let pages_for_doc: i64 = conn
                    .query_row(
                        "SELECT COUNT(1) FROM pages WHERE document_id = ?1",
                        params![document_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                total_pages += pages_for_doc;

                last_success_document_id = Some(document_id.clone());
                imported_files.push(ImportedFile {
                    document_id,
                    file_path,
                    file_name,
                    file_ext,
                    file_size,
                    page_count,
                });
            }
            Err(e) => {
                failed_files += 1;
                errors.push(format!(
                    "INSERT_DOCUMENT_FAILED {}: {e}",
                    path.to_string_lossy()
                ));
            }
        }

        let _ = conn.execute(
            "UPDATE import_jobs
             SET processed_files = ?1,
                 failed_files = ?2,
                 current_file = ?3,
                 last_success_document_id = ?4,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE import_job_id = ?5",
            params![
                processed_files,
                failed_files,
                path.to_string_lossy().to_string(),
                last_success_document_id,
                job_id,
            ],
        );
    }

    let imported_count = imported_files.len() as i64;
    if imported_count == 0 {
        let _ = conn.execute("DELETE FROM cases WHERE case_id = ?1", params![case_id.clone()]);
        return Err("IMPORT_FOLDER_NO_IMPORTED_FILES: không tạo hồ sơ vì không có file hợp lệ được nhập".to_string());
    }
    conn.execute(
        "UPDATE cases
         SET document_count = ?1,
             total_pages = ?2,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE case_id = ?3",
        params![imported_count, total_pages, case_id],
    )
    .map_err(|e| format!("IMPORT_UPDATE_CASE_COUNTER_FAILED: {e}"))?;

    let final_status = if imported_count == 0 && total_files > 0 {
        "failed"
    } else {
        "completed"
    };
    let error_log = to_json_array(&errors);

    conn.execute(
        "UPDATE import_jobs
         SET job_status = ?1,
             processed_files = ?2,
             failed_files = ?3,
             current_file = NULL,
             error_log = ?4,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
             completed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE import_job_id = ?5",
        params![
            final_status,
            processed_files,
            failed_files,
            error_log,
            job_id
        ],
    )
    .map_err(|e| format!("IMPORT_FINALIZE_JOB_FAILED: {e}"))?;

    Ok(ImportFolderResult {
        job_id,
        case_id,
        total_files,
        files: imported_files,
    })
}

#[tauri::command]
pub fn import_multiple_files(
    db: State<'_, DbState>,
    paths: Vec<String>,
    case_id: Option<String>,
) -> Result<ImportMultipleFilesResult, String> {
    if paths.is_empty() {
        return Err("IMPORT_FILES_EMPTY".to_string());
    }

    let has_supported_file = paths.iter().any(|raw_path| {
        let path = PathBuf::from(raw_path.trim());
        path.exists() && path.is_file() && is_supported_file(&path)
    });
    if !has_supported_file {
        return Err("IMPORT_FILES_NO_SUPPORTED_FILES: chỉ nhận PDF, ảnh scan hoặc file Office".to_string());
    }

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let (target_case_id, target_case_code, created_new_case) = if let Some(existing_case_id) = case_id {
        let case_code: String = conn
            .query_row(
                "SELECT case_code FROM cases WHERE case_id = ?1",
                params![existing_case_id.clone()],
                |row| row.get(0),
            )
            .map_err(|e| format!("IMPORT_FILES_CASE_CHECK_FAILED: {e}"))?;
        (existing_case_id, case_code, false)
    } else {
        let case_id = generate_case_id();
        let case_code = generate_case_code();
        conn.execute(
            "INSERT INTO cases (
                case_id, case_code, case_display_name, source_folder_name,
                primary_person_name, case_type, status, document_count,
                total_pages, created_at, updated_at
             ) VALUES (
                ?1, ?2, 'Import nhiều file', 'multi-file-import',
                'Import nhiều file', 'to_dieu_tra', 'active', 0,
                0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             )",
            params![case_id, case_code],
        )
        .map_err(|e| format!("IMPORT_FILES_CREATE_CASE_FAILED: {e}"))?;
        (case_id, case_code, true)
    };

    let mut imported = Vec::<ImportedFile>::new();
    let mut duplicates = Vec::<String>::new();
    let mut errors = Vec::<String>::new();

    for raw_path in paths {
        let path = PathBuf::from(raw_path.trim());
        if !path.exists() || !path.is_file() || !is_supported_file(&path) {
            errors.push(format!("UNSUPPORTED_OR_MISSING: {}", path.to_string_lossy()));
            continue;
        }

        let file_hash = match hash_file(&path) {
            Ok(value) => value,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("tai_lieu")
            .to_string();

        let duplicate_count: i64 = conn
            .query_row(
                "SELECT COUNT(*)
                 FROM documents
                 WHERE case_id = ?1 AND (file_hash = ?2 OR original_filename = ?3)",
                params![target_case_id, file_hash, file_name],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if duplicate_count > 0 {
            duplicates.push(file_name);
            continue;
        }

        let metadata = match fs::metadata(&path) {
            Ok(value) => value,
            Err(e) => {
                errors.push(format!("METADATA_FAILED {}: {e}", path.to_string_lossy()));
                continue;
            }
        };

        let ext_no_dot = path
            .extension()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let file_ext = if ext_no_dot.is_empty() {
            "".to_string()
        } else {
            format!(".{ext_no_dot}")
        };
        let file_size = metadata.len() as i64;
        let page_count = if ext_no_dot == "pdf" {
            count_pdf_pages(&path)
        } else {
            1
        };
        let document_id = generate_document_id();
        let managed_path =
            match storage::copy_to_originals(&path, &target_case_code, &document_id, &file_name) {
                Ok(value) => value,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            };
        let file_path = managed_path.to_string_lossy().to_string();
        let document_type = classify_document_type_from_name(&file_name);
        let display_name = build_display_name(&file_name, &document_type);
        let summary_short = build_summary_short(&display_name, &document_type);
        let summary_detail = build_summary_detail(&display_name, &document_type, &file_path);

        let insert_result = conn.execute(
            "INSERT INTO documents (
                document_id, case_id, original_filename, file_path, file_hash,
                file_size, page_count, display_name, document_title,
                document_type, summary_short, summary_detail, status,
                created_at, updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9, ?10, ?11,
                'pending', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             )",
            params![
                document_id,
                target_case_id,
                file_name,
                file_path,
                file_hash,
                file_size,
                page_count,
                display_name,
                document_type,
                summary_short,
                summary_detail,
            ],
        );

        match insert_result {
            Ok(_) => {
                if ext_no_dot == "pdf" {
                    for i in 1..=page_count {
                        if let Ok(page_id) = insert_page_row(&conn, &document_id, i, None) {
                            let _ = insert_ocr_result(&conn, &page_id, "", 0.0, "pending");
                        }
                    }
                } else if is_image_extension(&ext_no_dot) {
                    if let Ok(page_id) = insert_page_row(&conn, &document_id, 1, Some(&file_path)) {
                        let _ = insert_ocr_result(&conn, &page_id, "", 0.0, "pending");
                    }
                } else if let Ok(page_id) = insert_page_row(&conn, &document_id, 1, None) {
                    let _ = insert_ocr_result(&conn, &page_id, "", 0.0, "unsupported_office");
                }

                let _ = conn.execute(
                    "INSERT INTO fts_documents (rowid, document_id, display_name, document_title, summary_short, summary_detail)
                     SELECT rowid, document_id, display_name, document_title, summary_short, summary_detail
                     FROM documents WHERE document_id = ?1",
                    params![document_id],
                );
                let _ = create_review_entry(&conn, &document_id, "ocr_pending_after_multi_file_import");
                imported.push(ImportedFile {
                    document_id,
                    file_path,
                    file_name,
                    file_ext,
                    file_size,
                    page_count,
                });
            }
            Err(e) => errors.push(format!("INSERT_DOCUMENT_FAILED {}: {e}", path.to_string_lossy())),
        }
    }

    let doc_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE case_id = ?1",
            params![target_case_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_pages: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(page_count), 0) FROM documents WHERE case_id = ?1",
            params![target_case_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if created_new_case && imported.is_empty() {
        let _ = conn.execute("DELETE FROM cases WHERE case_id = ?1", params![target_case_id.clone()]);
        return Err("IMPORT_FILES_NO_IMPORTED_FILES: không tạo hồ sơ vì không có file hợp lệ được nhập".to_string());
    }
    conn.execute(
        "UPDATE cases
         SET document_count = ?1,
             total_pages = ?2,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE case_id = ?3",
        params![doc_count, total_pages, target_case_id],
    )
    .map_err(|e| format!("IMPORT_FILES_UPDATE_CASE_FAILED: {e}"))?;

    Ok(ImportMultipleFilesResult {
        case_id: target_case_id,
        total_files: (imported.len() + duplicates.len() + errors.len()) as i64,
        imported,
        duplicates,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::count_pdf_pages;
    use std::path::Path;

    #[test]
    fn count_pdf_pages_returns_zero_for_missing_file() {
        let page_count = count_pdf_pages(Path::new("__missing_test_file__.pdf"));
        assert_eq!(page_count, 0);
    }
}
