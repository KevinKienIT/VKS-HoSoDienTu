// VKS ECMS — Import folder commands

use crate::commands::governed_event::persist_governed_event;
use crate::commands::{ai_cmd, doc_cmd};

use crate::commands::module_cmd::DbState;
use crate::storage;
use log::{error, info, warn};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

static IMPORT_COUNTER: AtomicU64 = AtomicU64::new(0);

const ALLOWED_EXTENSIONS: &[&str] = &[
    "pdf", "png", "jpg", "jpeg", "tif", "tiff", "bmp", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
    "rtf",
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
    pub group_id: Option<String>,
    pub relative_path: Option<String>,
    pub import_order: Option<i64>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugImportTestPathResult {
    pub db_path: String,
    pub case_id: String,
    pub job_id: String,
    pub status: String,
    pub documents_before: i64,
    pub documents_after: i64,
    pub cases_before: i64,
    pub cases_after: i64,
    pub pages_before: i64,
    pub pages_after: i64,
    pub events_before: i64,
    pub events_after: i64,
    pub imported: Vec<ImportedFile>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportOcrWorkerPlan {
    pub cpu_total: i64,
    pub cpu_reserved: i64,
    pub cpu_workers: i64,
    pub import_workers: i64,
    pub ocr_workers: i64,
    pub ai_workers: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportOcrJobStartResult {
    pub job_id: String,
    pub case_id: String,
    pub total_files: i64,
    pub status: String,
    pub auto_ocr: bool,
    pub worker_plan: ImportOcrWorkerPlan,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportOcrProgressEvent {
    pub event_type: String,
    pub job_id: String,
    pub case_id: String,
    pub document_id: Option<String>,
    pub file_name: String,
    pub phase: String,
    pub current: i64,
    pub total: i64,
    pub page: i64,
    pub total_pages: i64,
    pub status: String,
    pub message: String,
    pub error: Option<String>,
    pub timestamp: String,
    pub is_heartbeat: bool,
}

#[derive(Debug, Clone)]
struct DiscoveredImportFile {
    path: PathBuf,
    relative_path: String,
    group_relative_path: Option<String>,
    import_order: i64,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum NaturalPart {
    Text(String),
    Number(u64),
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

fn generate_document_group_id() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("group-{}-{}", now_millis(), n)
}

fn generate_page_id() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("page-{}-{}", now_millis(), n)
}

fn generate_review_id() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("review-{}-{}", now_millis(), n)
}

fn generate_pipeline_event_id() -> String {
    let n = IMPORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("pipevent-{}-{}", now_millis(), n)
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn derive_import_ocr_worker_plan() -> ImportOcrWorkerPlan {
    let cpu_total = std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(2)
        .max(1);
    let cpu_reserved = (cpu_total as f64 * 0.25).round() as i64;
    let cpu_reserved = cpu_reserved.max(2).min(cpu_total.saturating_sub(1).max(1));
    let cpu_workers = (cpu_total - cpu_reserved).clamp(1, 12);
    let ocr_workers = ((cpu_workers as f64) * 0.60).round() as i64;
    ImportOcrWorkerPlan {
        cpu_total,
        cpu_reserved,
        cpu_workers,
        import_workers: cpu_workers.min(2).max(1),
        // SQLite writes and Python OCR are still coordinated document-by-document
        // to avoid DB locks; the plan is persisted for future adaptive scaling.
        ocr_workers: ocr_workers.clamp(1, 4),
        ai_workers: 1,
    }
}

fn classify_io_error_kind(err: &io::Error) -> &'static str {
    if err
        .raw_os_error()
        .map(|code| code == 32 || code == 33)
        .unwrap_or(false)
    {
        return "FILE_LOCKED";
    }
    match err.kind() {
        io::ErrorKind::NotFound => "FILE_NOT_FOUND",
        io::ErrorKind::PermissionDenied => "PERMISSION_DENIED",
        _ => "UNKNOWN",
    }
}

fn format_path_error(prefix: &str, path: &Path, kind: &str, detail: &str) -> String {
    format!(
        "{prefix}: kind={kind}; path={}; detail={detail}",
        path.display()
    )
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

fn is_junk_import_name(name: &str) -> bool {
    matches!(name, "__MACOSX" | ".DS_Store" | ".git") || name.starts_with("~$")
}

fn normalize_relative_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn group_relative_path(relative_path: &str) -> Option<String> {
    relative_path
        .rsplit_once('/')
        .map(|(group, _)| group.trim_matches('/').to_string())
        .filter(|group| !group.is_empty())
}

fn group_name_from_relative_path(relative_path: &str) -> String {
    relative_path
        .rsplit('/')
        .next()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Nhóm tài liệu")
        .to_string()
}

fn natural_sort_key(value: &str) -> Vec<NaturalPart> {
    let mut parts = Vec::<NaturalPart>::new();
    let mut current = String::new();
    let mut current_is_digit: Option<bool> = None;

    for ch in value.chars() {
        let is_digit = ch.is_ascii_digit();
        if current_is_digit == Some(is_digit) || current_is_digit.is_none() {
            current.push(ch);
            current_is_digit = Some(is_digit);
            continue;
        }

        if current_is_digit == Some(true) {
            parts.push(NaturalPart::Number(
                current.parse::<u64>().unwrap_or(u64::MAX),
            ));
        } else {
            parts.push(NaturalPart::Text(current.to_lowercase()));
        }
        current.clear();
        current.push(ch);
        current_is_digit = Some(is_digit);
    }

    if !current.is_empty() {
        if current_is_digit == Some(true) {
            parts.push(NaturalPart::Number(
                current.parse::<u64>().unwrap_or(u64::MAX),
            ));
        } else {
            parts.push(NaturalPart::Text(current.to_lowercase()));
        }
    }

    parts
}

fn compare_natural_path(a: &str, b: &str) -> std::cmp::Ordering {
    natural_sort_key(a)
        .cmp(&natural_sort_key(b))
        .then_with(|| a.cmp(b))
}

fn collect_supported_import_files(root: &Path) -> Result<Vec<DiscoveredImportFile>, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("IMPORT_FOLDER_CANONICALIZE_FAILED:{}:{e}", root.display()))?;
    let mut files = Vec::<DiscoveredImportFile>::new();
    collect_supported_import_files_inner(&root, &root, &mut files)?;
    files.sort_by(|a, b| compare_natural_path(&a.relative_path, &b.relative_path));
    for (idx, file) in files.iter_mut().enumerate() {
        file.import_order = idx as i64 + 1;
    }
    Ok(files)
}

fn collect_supported_import_files_inner(
    root: &Path,
    dir: &Path,
    out: &mut Vec<DiscoveredImportFile>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(dir)
        .map_err(|e| format!("IMPORT_FOLDER_READ_DIR_FAILED:{}:{e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        let a_name = a
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let b_name = b
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        compare_natural_path(a_name, b_name)
    });

    for path in entries {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if is_junk_import_name(name) {
            continue;
        }

        if path.is_dir() {
            collect_supported_import_files_inner(root, &path, out)?;
        } else if path.is_file() && is_supported_file(&path) {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| {
                    format!(
                        "IMPORT_FOLDER_RELATIVE_PATH_FAILED:{}:{}:{e}",
                        root.display(),
                        path.display()
                    )
                })
                .map(normalize_relative_path)?;
            out.push(DiscoveredImportFile {
                path,
                group_relative_path: group_relative_path(&relative),
                relative_path: relative,
                import_order: 0,
            });
        }
    }
    Ok(())
}

fn build_group_sort_orders(files: &[DiscoveredImportFile]) -> HashMap<String, i64> {
    let mut groups = Vec::<String>::new();
    for file in files {
        if let Some(group_path) = file.group_relative_path.as_deref() {
            let mut prefix_parts = Vec::<&str>::new();
            for part in group_path.split('/').filter(|part| !part.is_empty()) {
                prefix_parts.push(part);
                let prefix = prefix_parts.join("/");
                if !groups.iter().any(|existing| existing == &prefix) {
                    groups.push(prefix);
                }
            }
        }
    }
    groups.sort_by(|a, b| compare_natural_path(a, b));
    groups
        .into_iter()
        .enumerate()
        .map(|(idx, group)| (group, idx as i64 + 1))
        .collect()
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

fn db_count(conn: &rusqlite::Connection, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    conn.query_row(&sql, [], |row| row.get(0)).unwrap_or(-1)
}

fn persist_import_failed(
    conn: &rusqlite::Connection,
    source: &str,
    phase: &str,
    payload: serde_json::Value,
) {
    let _ = persist_governed_event(conn, "IMPORT_FAILED", source, phase, &payload);
}

fn ensure_document_group_hierarchy(
    conn: &rusqlite::Connection,
    case_id: &str,
    group_relative_path: Option<&str>,
    sort_orders: &HashMap<String, i64>,
) -> Result<Option<String>, String> {
    let Some(group_path) = group_relative_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    let mut parent_group_id: Option<String> = None;
    let mut current_group_id: Option<String> = None;
    let mut prefix_parts = Vec::<&str>::new();

    for part in group_path.split('/').filter(|part| !part.trim().is_empty()) {
        prefix_parts.push(part);
        let relative_path = prefix_parts.join("/");
        let existing_group_id = conn
            .query_row(
                "SELECT group_id
                 FROM document_groups
                 WHERE case_id = ?1 AND relative_path = ?2
                 LIMIT 1",
                params![case_id, relative_path],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| format!("DOCUMENT_GROUP_LOOKUP_FAILED:{case_id}:{relative_path}:{e}"))?;

        if let Some(existing_group_id) = existing_group_id {
            parent_group_id = Some(existing_group_id.clone());
            current_group_id = Some(existing_group_id);
            continue;
        }

        let group_id = generate_document_group_id();
        let name = group_name_from_relative_path(&relative_path);
        let sort_order = sort_orders.get(&relative_path).copied().unwrap_or(0);
        conn.execute(
            "INSERT INTO document_groups (
                group_id,
                case_id,
                parent_group_id,
                name,
                relative_path,
                sort_order,
                created_at,
                updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             )",
            params![
                group_id,
                case_id,
                parent_group_id.as_deref(),
                name,
                relative_path,
                sort_order
            ],
        )
        .map_err(|e| format!("DOCUMENT_GROUP_INSERT_FAILED:{case_id}:{relative_path}:{e}"))?;

        parent_group_id = Some(group_id.clone());
        current_group_id = Some(group_id);
    }

    Ok(current_group_id)
}

fn count_pdf_pages(path: &Path) -> i32 {
    match lopdf::Document::load(path) {
        Ok(doc) => doc.get_pages().len() as i32,
        Err(_) => 0,
    }
}

fn try_count_pdf_pages(path: &Path) -> Result<i32, String> {
    if !path.exists() {
        return Err(format_path_error(
            "PDF_PAGE_COUNT_FAILED",
            path,
            "FILE_NOT_FOUND",
            "file does not exist",
        ));
    }
    lopdf::Document::load(path)
        .map(|doc| doc.get_pages().len().max(1) as i32)
        .map_err(|e| {
            format_path_error(
                "PDF_PAGE_COUNT_FAILED",
                path,
                "PDF_CORRUPT_OR_UNREADABLE",
                &e.to_string(),
            )
        })
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| {
        format_path_error(
            "HASH_OPEN_FAILED",
            path,
            classify_io_error_kind(&e),
            &e.to_string(),
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|e| {
            format_path_error(
                "HASH_READ_FAILED",
                path,
                classify_io_error_kind(&e),
                &e.to_string(),
            )
        })?;
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

fn insert_page_image_row(
    conn: &rusqlite::Connection,
    document_id: &str,
    page_index: i32,
    source_pdf_path: &str,
    image_path: &str,
    thumbnail_path: Option<&str>,
) -> Result<String, String> {
    let page_id = generate_page_id();
    conn.execute(
        "INSERT INTO pages (
            page_id, document_id, page_index, image_path,
            source_pdf_path, source_page_number, thumbnail_path,
            current_order, rotation, is_removed,
            extract_status, ocr_status, review_required, created_at
         ) VALUES (
            ?1, ?2, ?3, ?4,
            ?5, ?3, ?6,
            ?3, 0, 0,
            'extracted', 'queued', 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            page_id,
            document_id,
            page_index,
            image_path,
            source_pdf_path,
            thumbnail_path
        ],
    )
    .map_err(|e| format!("INSERT_PAGE_IMAGE_FAILED: {e}"))?;
    Ok(page_id)
}

fn run_page_image_extraction(
    conn: &rusqlite::Connection,
    document_id: &str,
    source_pdf_path: &Path,
) -> Result<i64, String> {
    let output_dir = storage::page_images_dir(document_id)?;
    let thumb_dir = storage::page_thumbnails_dir(document_id)?;
    let args = vec![
        "-m".to_string(),
        "ocr.pdf_to_images".to_string(),
        "--pdf".to_string(),
        source_pdf_path.to_string_lossy().to_string(),
        "--output-dir".to_string(),
        output_dir.to_string_lossy().to_string(),
        "--thumbnail-dir".to_string(),
        thumb_dir.to_string_lossy().to_string(),
        "--dpi".to_string(),
        "300".to_string(),
        "--json".to_string(),
    ];
    let extracted = doc_cmd::run_python_json_for_import(&args)?;
    let pages = extracted
        .get("pages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "PAGE_IMAGE_EXTRACT_NO_PAGES_JSON".to_string())?;

    // Save extracted pages JSON for classifier input
    let pages_json_path = output_dir.join("pages_extracted.json");
    if let Ok(json_str) = serde_json::to_string_pretty(&extracted) {
        let _ = std::fs::write(&pages_json_path, &json_str);
    }

    let mut count = 0_i64;
    for page in pages {
        let page_num = page.get("page_num").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        let image_path = page
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if image_path.trim().is_empty() || !Path::new(image_path).exists() {
            continue;
        }
        let thumbnail_path = page
            .get("thumbnail_path")
            .and_then(|v| v.as_str())
            .filter(|v| !v.trim().is_empty());
        let page_id = insert_page_image_row(
            conn,
            document_id,
            page_num,
            &source_pdf_path.to_string_lossy(),
            image_path,
            thumbnail_path,
        )?;
        let _ = insert_ocr_result(conn, &page_id, "", 0.0, "pending");
        count += 1;
    }
    if count == 0 {
        return Err("PAGE_IMAGE_EXTRACT_ZERO_PAGES".to_string());
    }

    // Run quality classification after extraction
    if pages_json_path.exists() {
        match run_page_quality_classifier(conn, document_id, &pages_json_path) {
            Ok(summary) => {
                log::info!(
                    "PAGE_QUALITY_CLASSIFY document_id={} summary={}",
                    document_id,
                    summary
                );
            }
            Err(e) => {
                log::warn!(
                    "PAGE_QUALITY_CLASSIFY_FAILED document_id={} error={}",
                    document_id,
                    e
                );
                // Non-fatal: classification failure should not block import
            }
        }
    }

    Ok(count)
}

/// Run the page quality classifier Python script on extracted pages.
/// Updates pages table with page_quality, visual_document_type, quality_warnings, review_required.
/// Updates documents table with visual_document_type and suggested_filename.
fn run_page_quality_classifier(
    conn: &rusqlite::Connection,
    document_id: &str,
    pages_json_path: &Path,
) -> Result<String, String> {
    let args = vec![
        "-m".to_string(),
        "ocr.page_quality_classifier".to_string(),
        "--pages-json".to_string(),
        pages_json_path.to_string_lossy().to_string(),
        "--json".to_string(),
    ];
    let classifier_result = doc_cmd::run_python_json_for_import(&args)?;

    // Parse per-page results
    let pages = classifier_result
        .get("pages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "PAGE_QUALITY_NO_PAGES_JSON".to_string())?;

    // Get page_ids mapped by page_index (0-based)
    let mut page_id_map: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT page_id, page_index FROM pages WHERE document_id = ?1 ORDER BY page_index",
            )
            .map_err(|e| format!("PAGE_QUALITY_QUERY_FAILED: {e}"))?;
        let rows = stmt
            .query_map(params![document_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| format!("PAGE_QUALITY_QUERY_FAILED: {e}"))?;
        for row in rows {
            if let Ok((page_id, page_index)) = row {
                page_id_map.insert(page_index, page_id);
            }
        }
    }

    let mut review_count = 0;
    let mut quality_summary = Vec::<String>::new();

    for page_result in pages {
        let page_index = page_result
            .get("index")
            .and_then(|v| v.as_i64())
            .unwrap_or(-1);
        // page_index from classifier is 0-based, page_index in DB is 1-based
        let db_page_index = page_index + 1;

        let page_id = match page_id_map.get(&db_page_index) {
            Some(id) => id.clone(),
            None => continue, // Skip pages not in DB
        };

        let page_quality = page_result
            .get("page_quality")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let visual_doc_type = page_result
            .get("visual_document_type")
            .and_then(|v| v.as_str());
        let quality_warnings = page_result
            .get("quality_warnings")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "[]".to_string());
        let review_required = page_result
            .get("review_required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if review_required {
            review_count += 1;
        }

        conn.execute(
            "UPDATE pages SET page_quality = ?1, visual_document_type = ?2,
             quality_warnings = ?3, review_required = ?4
             WHERE page_id = ?5",
            params![
                page_quality,
                visual_doc_type,
                quality_warnings,
                review_required as i32,
                page_id,
            ],
        )
        .map_err(|e| format!("PAGE_QUALITY_UPDATE_FAILED:{page_id}: {e}"))?;

        quality_summary.push(format!("p{}={}", db_page_index, page_quality));
    }

    // Update document-level fields
    let doc_visual_type = classifier_result
        .get("document_visual_type")
        .and_then(|v| v.as_str());

    // Get original filename for suggested_filename
    let original_filename: String = conn
        .query_row(
            "SELECT COALESCE(original_filename, display_name, '') FROM documents WHERE document_id = ?1",
            params![document_id],
            |row| row.get(0),
        )
        .unwrap_or_default();

    // Build suggested filename based on document type
    let suggested = if doc_visual_type == Some("SO_DO_HIEN_TRUONG") {
        format!("So_do_hien_truong_{}tr_review.pdf", pages.len())
    } else if doc_visual_type == Some("ANH_HIEN_TRUONG") {
        format!("Anh_hien_truong_{}tr_review.pdf", pages.len())
    } else if !original_filename.is_empty() {
        original_filename.clone()
    } else {
        String::new()
    };

    conn.execute(
        "UPDATE documents
         SET visual_document_type = ?1,
             suggested_filename = ?2,
             document_type = CASE WHEN ?1 IS NOT NULL THEN ?1 ELSE document_type END,
             needs_review = CASE WHEN ?3 > 0 THEN 1 ELSE needs_review END
         WHERE document_id = ?4",
        params![
            doc_visual_type,
            if suggested.is_empty() {
                None
            } else {
                Some(&suggested)
            },
            review_count,
            document_id
        ],
    )
    .map_err(|e| format!("DOC_QUALITY_UPDATE_FAILED: {e}"))?;

    // Create review entry if any page needs review
    if review_count > 0 {
        let _ = create_review_entry(
            conn,
            document_id,
            &format!("Page quality review: {} pages cần rà soát", review_count),
        );
    }

    let summary = format!(
        "pages={} review={} doc_type={} [{}]",
        pages.len(),
        review_count,
        doc_visual_type.unwrap_or("null"),
        quality_summary.join(", "),
    );
    Ok(summary)
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

fn is_ocr_extension(ext: &str) -> bool {
    matches!(ext, "pdf" | "png" | "jpg" | "jpeg" | "tif" | "tiff" | "bmp")
}

fn persist_import_ocr_pipeline_event(conn: &rusqlite::Connection, event: &ImportOcrProgressEvent) {
    let _ = persist_governed_event(
        conn,
        &event.event_type,
        "import_cmd.import_ocr_job",
        "P0-import-ocr",
        &json!(event),
    );

    let event_id = generate_pipeline_event_id();
    let level = if event.error.is_some() || event.status == "failed" {
        "error"
    } else if event.status == "paused" || event.status == "cancelled" {
        "warn"
    } else {
        "info"
    };
    let _ = conn.execute(
        "INSERT OR IGNORE INTO pipeline_events (
            event_id, job_id, task_id, level, event_type, message,
            from_status, to_status, meta_json, created_at
         ) VALUES (
            ?1, ?2, NULL, ?3, ?4, ?5,
            NULL, ?6, ?7, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            event_id,
            &event.job_id,
            level,
            &event.event_type,
            &event.message,
            &event.status,
            json!(event).to_string(),
        ],
    );
}

fn emit_import_ocr_event(
    app: &AppHandle,
    conn: Option<&rusqlite::Connection>,
    event: ImportOcrProgressEvent,
) {
    if let Some(conn) = conn {
        persist_import_ocr_pipeline_event(conn, &event);
    }
    if let Err(e) = app.emit("import-ocr-progress", event.clone()) {
        warn!(
            "IMPORT_OCR_EVENT_EMIT_FAILED job_id={} event_type={} error={}",
            event.job_id, event.event_type, e
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn make_import_ocr_event(
    event_type: &str,
    job_id: &str,
    case_id: &str,
    document_id: Option<String>,
    file_name: &str,
    phase: &str,
    current: i64,
    total: i64,
    page: i64,
    total_pages: i64,
    status: &str,
    message: &str,
    error: Option<String>,
    is_heartbeat: bool,
) -> ImportOcrProgressEvent {
    ImportOcrProgressEvent {
        event_type: event_type.to_string(),
        job_id: job_id.to_string(),
        case_id: case_id.to_string(),
        document_id,
        file_name: file_name.to_string(),
        phase: phase.to_string(),
        current,
        total,
        page,
        total_pages,
        status: status.to_string(),
        message: message.to_string(),
        error,
        timestamp: now_iso(),
        is_heartbeat,
    }
}

fn update_pipeline_status(
    conn: &rusqlite::Connection,
    job_id: &str,
    status: &str,
    last_error: Option<&str>,
) {
    let _ = conn.execute(
        "UPDATE pipeline_jobs
         SET status = ?2,
             last_error = COALESCE(?3, last_error),
             started_at = CASE WHEN ?2 = 'running' AND started_at IS NULL THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now') ELSE started_at END,
             completed_at = CASE WHEN ?2 IN ('completed','completed_with_errors') THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now') ELSE completed_at END,
             cancelled_at = CASE WHEN ?2 = 'cancelled' THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now') ELSE cancelled_at END,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE job_id = ?1",
        params![job_id, status, last_error],
    );
}

fn bump_pipeline_count(conn: &rusqlite::Connection, job_id: &str, ok: bool) {
    let _ = conn.execute(
        "UPDATE pipeline_jobs
         SET completed_tasks = completed_tasks + CASE WHEN ?2 THEN 1 ELSE 0 END,
             failed_tasks = failed_tasks + CASE WHEN ?2 THEN 0 ELSE 1 END,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE job_id = ?1",
        params![job_id, ok],
    );
}

fn job_control_state(
    conn: &rusqlite::Connection,
    job_id: &str,
) -> Result<(String, bool, bool), String> {
    conn.query_row(
        "SELECT status, pause_requested, cancel_requested FROM pipeline_jobs WHERE job_id = ?1",
        params![job_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? != 0,
                row.get::<_, i64>(2)? != 0,
            ))
        },
    )
    .map_err(|e| format!("JOB_CONTROL_QUERY_FAILED: {e}"))
}

fn wait_for_resume_or_cancel(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    job_id: &str,
    case_id: &str,
    total: i64,
    current: i64,
    file_name: &str,
) -> Result<(), String> {
    let mut pause_announced = false;
    loop {
        let (status, pause_requested, cancel_requested) = job_control_state(conn, job_id)?;
        if cancel_requested || status == "cancelled" {
            doc_cmd::request_ocr_cancel();
            let event = make_import_ocr_event(
                "JOB_CANCELLED",
                job_id,
                case_id,
                None,
                file_name,
                "cancel",
                current,
                total,
                0,
                0,
                "cancelled",
                "Job đã bị hủy theo yêu cầu người dùng.",
                None,
                false,
            );
            emit_import_ocr_event(app, Some(conn), event);
            return Err("JOB_CANCELLED".to_string());
        }
        if !pause_requested && status != "paused" {
            if pause_announced {
                let event = make_import_ocr_event(
                    "JOB_RESUMED",
                    job_id,
                    case_id,
                    None,
                    file_name,
                    "resume",
                    current,
                    total,
                    0,
                    0,
                    "running",
                    "Job đã tiếp tục.",
                    None,
                    false,
                );
                emit_import_ocr_event(app, Some(conn), event);
            }
            return Ok(());
        }
        if !pause_announced {
            pause_announced = true;
            let event = make_import_ocr_event(
                "JOB_PAUSED",
                job_id,
                case_id,
                None,
                file_name,
                "pause",
                current,
                total,
                0,
                0,
                "paused",
                "Job đang tạm dừng; worker sẽ không nhận file tiếp theo.",
                None,
                false,
            );
            emit_import_ocr_event(app, Some(conn), event);
        }
        thread::sleep(Duration::from_millis(800));
    }
}

fn import_one_file_for_case(
    conn: &rusqlite::Connection,
    path: &Path,
    case_id: &str,
    case_code: &str,
    duplicate_check: bool,
    group_id: Option<&str>,
    relative_path: Option<&str>,
    import_order: Option<i64>,
) -> Result<ImportedFile, String> {
    if !path.exists() {
        return Err(format_path_error(
            "IMPORT_FILE_OPEN_FAILED",
            path,
            "FILE_NOT_FOUND",
            "file does not exist",
        ));
    }
    if !path.is_file() {
        return Err(format_path_error(
            "IMPORT_FILE_OPEN_FAILED",
            path,
            "UNKNOWN",
            "path is not a file",
        ));
    }
    if !is_supported_file(path) {
        return Err(format_path_error(
            "IMPORT_FILE_UNSUPPORTED",
            path,
            "UNKNOWN",
            "unsupported extension",
        ));
    }

    let source = path.canonicalize().map_err(|e| {
        format_path_error(
            "IMPORT_FILE_CANONICALIZE_FAILED",
            path,
            classify_io_error_kind(&e),
            &e.to_string(),
        )
    })?;
    let metadata = fs::metadata(&source).map_err(|e| {
        format_path_error(
            "IMPORT_FILE_METADATA_FAILED",
            &source,
            classify_io_error_kind(&e),
            &e.to_string(),
        )
    })?;
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("tai_lieu")
        .to_string();
    let ext_no_dot = source
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
        try_count_pdf_pages(&source)?
    } else {
        1
    };
    let file_hash = hash_file(&source)?;

    if duplicate_check {
        let duplicate_count: i64 = conn
            .query_row(
                "SELECT COUNT(*)
                 FROM documents
                 WHERE case_id = ?1 AND (file_hash = ?2 OR original_filename = ?3)",
                params![case_id, file_hash, file_name],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if duplicate_count > 0 {
            return Err(format!(
                "DUPLICATE_FILE: path={}; file_name={}",
                source.display(),
                file_name
            ));
        }
    }

    let document_id = generate_document_id();
    let managed_path = storage::copy_to_originals(&source, case_code, &document_id, &file_name)
        .map_err(|e| {
            format!(
                "IMPORT_COPY_TO_WORKSPACE_FAILED: kind=UNKNOWN; path={}; detail={e}",
                source.display()
            )
        })?;
    let file_path = managed_path.to_string_lossy().to_string();
    let document_type = classify_document_type_from_name(&file_name);
    let display_name = build_display_name(&file_name, &document_type);
    let summary_short = build_summary_short(&display_name, &document_type);
    let summary_detail = build_summary_detail(&display_name, &document_type, &file_path);

    conn.execute(
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
            file_status,
            original_path,
            managed_path,
            group_id,
            relative_path,
            import_sequence,
            revision_no,
            created_at,
            updated_at
         ) VALUES (
            ?1, ?2, ?3, ?4,
            ?11, ?5, ?6, ?7, ?7, ?8, ?9, ?10,
            'pending',
            'imported', ?4, NULL, ?12, ?13, ?14, 1,
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
            file_hash,
            group_id,
            relative_path,
            import_order,
        ],
    )
    .map_err(|e| {
        format!(
            "DB_WRITE_FAILED: INSERT_DOCUMENT_FAILED path={}: {e}",
            source.display()
        )
    })?;

    if ext_no_dot == "pdf" {
        let _ = persist_governed_event(
            conn,
            "PAGE_IMAGE_EXTRACT_STARTED",
            "import_cmd.import_one_file_for_case",
            "page-image-phase1",
            &json!({ "document_id": document_id, "source_pdf_path": file_path }),
        );
        match run_page_image_extraction(conn, &document_id, &managed_path) {
            Ok(extracted_pages) => {
                let _ = persist_governed_event(
                    conn,
                    "PAGE_IMAGE_EXTRACT_DONE",
                    "import_cmd.import_one_file_for_case",
                    "page-image-phase1",
                    &json!({ "document_id": document_id, "source_pdf_path": file_path, "pages": extracted_pages }),
                );
            }
            Err(e) => {
                let _ = persist_governed_event(
                    conn,
                    "PAGE_IMAGE_EXTRACT_FAILED",
                    "import_cmd.import_one_file_for_case",
                    "page-image-phase1",
                    &json!({ "document_id": document_id, "source_pdf_path": file_path, "error": e }),
                );
                for i in 1..=page_count {
                    if let Ok(page_id) = insert_page_row(conn, &document_id, i, None) {
                        let _ = insert_ocr_result(conn, &page_id, "", 0.0, "pending");
                    }
                }
            }
        }
    } else if is_image_extension(&ext_no_dot) {
        if let Ok(page_id) = insert_page_row(conn, &document_id, 1, Some(&file_path)) {
            let _ = insert_ocr_result(conn, &page_id, "", 0.0, "pending");
        }
    } else if let Ok(page_id) = insert_page_row(conn, &document_id, 1, None) {
        let _ = insert_ocr_result(conn, &page_id, "", 0.0, "unsupported_office");
    }

    let _ = conn.execute(
        "INSERT INTO fts_documents (rowid, document_id, display_name, document_title, summary_short, summary_detail)
         SELECT rowid, document_id, display_name, document_title, summary_short, summary_detail
         FROM documents WHERE document_id = ?1",
        params![document_id],
    );
    let _ = create_review_entry(conn, &document_id, "ocr_pending_after_import_job");

    Ok(ImportedFile {
        document_id,
        file_path,
        file_name,
        file_ext,
        file_size,
        page_count,
        group_id: group_id.map(str::to_string),
        relative_path: relative_path.map(str::to_string),
        import_order,
    })
}

fn create_case_for_folder(
    conn: &rusqlite::Connection,
    folder: &Path,
) -> Result<(String, String), String> {
    let source_folder_name_raw = folder
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
    let case_id = generate_case_id();
    let case_code = generate_case_code();
    conn.execute(
        "INSERT INTO cases (
            case_id, case_code, case_display_name, source_folder_name,
            primary_person_name, case_type, status, document_count,
            total_pages, created_at, updated_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            'to_dieu_tra', 'active', 0, 0,
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
    .map_err(|e| format!("DB_WRITE_FAILED: IMPORT_JOB_CREATE_CASE_FAILED: {e}"))?;
    Ok((case_id, case_code))
}

fn create_case_for_files(conn: &rusqlite::Connection) -> Result<(String, String), String> {
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
    .map_err(|e| format!("DB_WRITE_FAILED: IMPORT_JOB_CREATE_CASE_FAILED: {e}"))?;
    Ok((case_id, case_code))
}

fn create_import_ocr_job_records(
    conn: &rusqlite::Connection,
    job_id: &str,
    case_id: &str,
    source: &str,
    total_files: i64,
    auto_ocr: bool,
    worker_plan: &ImportOcrWorkerPlan,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO import_jobs (
            import_job_id, case_id, source_folder, job_status,
            total_files, processed_files, failed_files, error_log,
            started_at, updated_at
         ) VALUES (
            ?1, ?2, ?3, 'created',
            ?4, 0, 0, '[]',
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![job_id, case_id, source, total_files],
    )
    .map_err(|e| format!("DB_WRITE_FAILED: IMPORT_JOB_CREATE_FAILED: {e}"))?;

    let total_tasks = total_files * if auto_ocr { 2 } else { 1 };
    conn.execute(
        "INSERT INTO pipeline_jobs (
            job_id, source_type, status, total_tasks, completed_tasks,
            failed_tasks, cancelled_tasks, pause_requested, cancel_requested,
            created_at, updated_at
         ) VALUES (
            ?1, 'import_ocr_job', 'created', ?2, 0,
            0, 0, 0, 0,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![job_id, total_tasks],
    )
    .map_err(|e| format!("DB_WRITE_FAILED: PIPELINE_JOB_CREATE_FAILED: {e}"))?;

    let _ = persist_governed_event(
        conn,
        "IMPORT_JOB_STARTED",
        "import_cmd.start_import_ocr_job",
        "P0-import-ocr",
        &json!({
            "job_id": job_id,
            "case_id": case_id,
            "source": source,
            "total_files": total_files,
            "auto_ocr": auto_ocr,
            "status": "queued",
            "worker_plan": worker_plan,
        }),
    );
    let _ = persist_governed_event(
        conn,
        "IMPORT_STARTED",
        "import_cmd.start_import_ocr_job",
        "P0-import-ocr",
        &json!({
            "job_id": job_id,
            "case_id": case_id,
            "source": source,
            "total_files": total_files,
            "auto_ocr": auto_ocr,
            "worker_plan": worker_plan,
        }),
    );
    Ok(())
}

fn update_case_counters(conn: &rusqlite::Connection, case_id: &str) {
    let _ = conn.execute(
        "UPDATE cases
         SET document_count = (SELECT COUNT(*) FROM documents WHERE case_id = ?1),
             total_pages = (SELECT COALESCE(SUM(page_count), 0) FROM documents WHERE case_id = ?1),
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE case_id = ?1",
        params![case_id],
    );
}

fn short_error(error: &str) -> String {
    if error.chars().count() > 360 {
        error.chars().take(360).collect::<String>() + "..."
    } else {
        error.to_string()
    }
}

#[allow(clippy::too_many_arguments)]
fn run_import_ocr_background_job(
    app: AppHandle,
    db_path: PathBuf,
    job_id: String,
    case_id: String,
    case_code: String,
    files: Vec<DiscoveredImportFile>,
    auto_ocr: bool,
    duplicate_check: bool,
) {
    doc_cmd::clear_ocr_cancel_request();
    let total = files.len() as i64;
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(conn) => conn,
        Err(e) => {
            error!(
                "IMPORT_OCR_JOB_DB_OPEN_FAILED job_id={} error={}",
                job_id, e
            );
            return;
        }
    };
    let _ = conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;");
    update_pipeline_status(&conn, &job_id, "running", None);
    let _ = conn.execute(
        "UPDATE import_jobs
         SET job_status = 'importing',
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE import_job_id = ?1",
        params![job_id],
    );

    let heartbeat_event = Arc::new(Mutex::new(make_import_ocr_event(
        "IMPORT_JOB_STARTED",
        &job_id,
        &case_id,
        None,
        "",
        "import",
        0,
        total,
        0,
        0,
        "running",
        "Import/OCR worker nền đã bắt đầu.",
        None,
        false,
    )));
    let heartbeat_active = Arc::new(AtomicBool::new(true));
    let hb_app = app.clone();
    let hb_event = heartbeat_event.clone();
    let hb_active = heartbeat_active.clone();
    thread::spawn(move || {
        while hb_active.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(5));
            if !hb_active.load(Ordering::SeqCst) {
                break;
            }
            if let Ok(mut event) = hb_event.lock().map(|event| event.clone()) {
                event.event_type = "OCR_JOB_HEARTBEAT".to_string();
                event.is_heartbeat = true;
                event.timestamp = now_iso();
                let _ = hb_app.emit("import-ocr-progress", event);
            }
        }
    });

    let started = Instant::now();
    let mut imported_count = 0_i64;
    let mut failed_files = 0_i64;
    let mut ocr_done = 0_i64;
    let mut ocr_failed = 0_i64;
    let mut page_images_extracted = 0_i64;
    let mut errors = Vec::<String>::new();
    let group_sort_orders = build_group_sort_orders(&files);

    emit_import_ocr_event(
        &app,
        Some(&conn),
        heartbeat_event
            .lock()
            .map(|e| e.clone())
            .unwrap_or_else(|_| {
                make_import_ocr_event(
                    "IMPORT_JOB_STARTED",
                    &job_id,
                    &case_id,
                    None,
                    "",
                    "import",
                    0,
                    total,
                    0,
                    0,
                    "running",
                    "Import/OCR worker nền đã bắt đầu.",
                    None,
                    false,
                )
            }),
    );

    for (idx, file_plan) in files.iter().enumerate() {
        let path = &file_plan.path;
        let current = idx as i64 + 1;
        let file_name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("tai_lieu")
            .to_string();

        if wait_for_resume_or_cancel(&app, &conn, &job_id, &case_id, total, current, &file_name)
            .is_err()
        {
            update_pipeline_status(&conn, &job_id, "cancelled", Some("JOB_CANCELLED"));
            let _ = conn.execute(
                "UPDATE import_jobs
                 SET job_status = 'cancelled',
                     error_log = ?2,
                     completed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE import_job_id = ?1",
                params![job_id, to_json_array(&errors)],
            );
            heartbeat_active.store(false, Ordering::SeqCst);
            return;
        }

        let import_started = make_import_ocr_event(
            "IMPORT_FILE_STARTED",
            &job_id,
            &case_id,
            None,
            &file_name,
            "import",
            current,
            total,
            0,
            0,
            "running",
            &format!("Đang copy/hash/import file {current}/{total}: {file_name}"),
            None,
            false,
        );
        if let Ok(mut hb) = heartbeat_event.lock() {
            *hb = import_started.clone();
        }
        emit_import_ocr_event(&app, Some(&conn), import_started);

        let group_id = match ensure_document_group_hierarchy(
            &conn,
            &case_id,
            file_plan.group_relative_path.as_deref(),
            &group_sort_orders,
        ) {
            Ok(group_id) => group_id,
            Err(e) => {
                failed_files += 1;
                let message = format!("DOCUMENT_GROUP_FAILED {}: {e}", file_plan.relative_path);
                errors.push(message.clone());
                persist_import_failed(
                    &conn,
                    "import_cmd.import_ocr_job",
                    "P0-import-ocr",
                    json!({
                        "job_id": job_id,
                        "case_id": case_id,
                        "relative_path": &file_plan.relative_path,
                        "error": message,
                    }),
                );
                bump_pipeline_count(&conn, &job_id, false);
                continue;
            }
        };

        match import_one_file_for_case(
            &conn,
            path,
            &case_id,
            &case_code,
            duplicate_check,
            group_id.as_deref(),
            Some(&file_plan.relative_path),
            Some(file_plan.import_order),
        ) {
            Ok(file) => {
                imported_count += 1;
                let _ = conn.execute(
                    "UPDATE import_jobs
                     SET processed_files = ?2,
                         current_file = ?3,
                         last_success_document_id = ?4,
                         updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                     WHERE import_job_id = ?1",
                    params![
                        job_id,
                        current,
                        path.to_string_lossy().to_string(),
                        file.document_id
                    ],
                );
                update_case_counters(&conn, &case_id);
                let extracted_for_doc: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM pages WHERE document_id = ?1 AND extract_status = 'extracted' AND COALESCE(image_path, '') <> ''",
                        params![file.document_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                page_images_extracted += extracted_for_doc;
                bump_pipeline_count(&conn, &job_id, true);
                let imported_event = make_import_ocr_event(
                    "IMPORT_FILE_DONE",
                    &job_id,
                    &case_id,
                    Some(file.document_id.clone()),
                    &file.file_name,
                    "import",
                    current,
                    total,
                    0,
                    file.page_count as i64,
                    "done",
                    &format!("Đã import file {current}/{total}: {}", file.file_name),
                    None,
                    false,
                );
                if let Ok(mut hb) = heartbeat_event.lock() {
                    *hb = imported_event.clone();
                }
                emit_import_ocr_event(&app, Some(&conn), imported_event);
                let _ = persist_governed_event(
                    &conn,
                    "IMPORT_DONE",
                    "import_cmd.import_ocr_job",
                    "P0-import-ocr",
                    &json!({
                        "job_id": job_id,
                        "case_id": case_id,
                        "document_id": &file.document_id,
                        "file_name": &file.file_name,
                        "file_path": &file.file_path,
                        "relative_path": &file.relative_path,
                        "group_id": &file.group_id,
                        "import_order": file.import_order,
                        "page_count": file.page_count,
                        "current": current,
                        "total_files": total,
                    }),
                );

                if !auto_ocr {
                    continue;
                }
                if !is_ocr_extension(file.file_ext.trim_start_matches('.')) {
                    let message =
                        "File không phải PDF/ảnh scan; OCR tự động được bỏ qua.".to_string();
                    let _ = doc_cmd::mark_document_ocr_failure_for_import(
                        &conn,
                        &file.document_id,
                        "OCR_UNSUPPORTED_FILE_TYPE",
                    );
                    let _ = doc_cmd::queue_ai_after_ocr_for_import(
                        &conn,
                        &file.document_id,
                        "ocr_unsupported_file_import",
                    );
                    let skipped = make_import_ocr_event(
                        "OCR_DOCUMENT_FAILED",
                        &job_id,
                        &case_id,
                        Some(file.document_id.clone()),
                        &file.file_name,
                        "ocr",
                        current,
                        total,
                        0,
                        file.page_count as i64,
                        "failed",
                        &message,
                        Some("OCR_UNSUPPORTED_FILE_TYPE".to_string()),
                        false,
                    );
                    ocr_failed += 1;
                    bump_pipeline_count(&conn, &job_id, false);
                    emit_import_ocr_event(&app, Some(&conn), skipped);
                    continue;
                }

                let _ = conn.execute(
                    "UPDATE import_jobs
                     SET job_status = 'ocr_processing',
                         current_file = ?2,
                         updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                     WHERE import_job_id = ?1",
                    params![job_id, file.file_name],
                );
                let ocr_started = make_import_ocr_event(
                    "OCR_DOCUMENT_STARTED",
                    &job_id,
                    &case_id,
                    Some(file.document_id.clone()),
                    &file.file_name,
                    "ocr",
                    current,
                    total,
                    0,
                    file.page_count as i64,
                    "running",
                    &format!("Đang OCR file {current}/{total}: {}", file.file_name),
                    None,
                    false,
                );
                if let Ok(mut hb) = heartbeat_event.lock() {
                    *hb = ocr_started.clone();
                }
                emit_import_ocr_event(&app, Some(&conn), ocr_started);

                match doc_cmd::run_ocr_for_document_with_conn(&app, &conn, &file.document_id) {
                    Ok(result) if result.processed_pages > 0 => {
                        ocr_done += 1;
                        bump_pipeline_count(&conn, &job_id, true);
                        let ocr_done_event = make_import_ocr_event(
                            "OCR_DOCUMENT_DONE",
                            &job_id,
                            &case_id,
                            Some(file.document_id.clone()),
                            &file.file_name,
                            "ocr",
                            current,
                            total,
                            result.processed_pages,
                            file.page_count as i64,
                            "done",
                            &format!("OCR xong: {}", result.message),
                            None,
                            false,
                        );
                        if let Ok(mut hb) = heartbeat_event.lock() {
                            *hb = ocr_done_event.clone();
                        }
                        emit_import_ocr_event(&app, Some(&conn), ocr_done_event);

                        emit_import_ocr_event(
                            &app,
                            Some(&conn),
                            make_import_ocr_event(
                                "AI_DOCUMENT_ANALYSIS_STARTED",
                                &job_id,
                                &case_id,
                                Some(file.document_id.clone()),
                                &file.file_name,
                                "ai",
                                current,
                                total,
                                0,
                                0,
                                "running",
                                "Đang xử lý AI offline/fallback cho tài liệu.",
                                None,
                                false,
                            ),
                        );
                        let _ = ai_cmd::process_pending_ai_analysis_jobs(&conn, 1);
                        emit_import_ocr_event(
                            &app,
                            Some(&conn),
                            make_import_ocr_event(
                                "AI_DOCUMENT_ANALYSIS_DONE",
                                &job_id,
                                &case_id,
                                Some(file.document_id),
                                &file.file_name,
                                "ai",
                                current,
                                total,
                                0,
                                0,
                                "done",
                                "AI offline/fallback đã được xử lý hoặc đã chuyển hàng đợi review.",
                                None,
                                false,
                            ),
                        );
                    }
                    Ok(result) => {
                        ocr_failed += 1;
                        bump_pipeline_count(&conn, &job_id, false);
                        let message = format!("OCR không có trang thành công: {}", result.message);
                        errors.push(format!("{}: {}", file.file_name, message));
                        emit_import_ocr_event(
                            &app,
                            Some(&conn),
                            make_import_ocr_event(
                                "OCR_DOCUMENT_FAILED",
                                &job_id,
                                &case_id,
                                Some(file.document_id.clone()),
                                &file.file_name,
                                "ocr",
                                current,
                                total,
                                0,
                                file.page_count as i64,
                                "failed",
                                &message,
                                Some("OCR_WORKER_FAILED".to_string()),
                                false,
                            ),
                        );
                        // AI fallback: try analysis with whatever metadata exists
                        let _ = ai_cmd::process_pending_ai_analysis_jobs(&conn, 1);
                        // Emit explicit AI skipped/fallback event
                        emit_import_ocr_event(
                            &app,
                            Some(&conn),
                            make_import_ocr_event(
                                "AI_SKIPPED_AFTER_OCR_FAILURE",
                                &job_id,
                                &case_id,
                                Some(file.document_id.clone()),
                                &file.file_name,
                                "ai",
                                current,
                                total,
                                0,
                                0,
                                "review_required",
                                "OCR lỗi — AI fallback/review_required đã được tạo.",
                                None,
                                false,
                            ),
                        );
                        // Create review_required entry
                        let _ = create_review_entry(
                            &conn,
                            &file.document_id,
                            "ocr_failed_ai_review_required",
                        );
                    }
                    Err(e) => {
                        ocr_failed += 1;
                        let err = short_error(&e);
                        errors.push(format!("{}: {}", file.file_name, err));
                        bump_pipeline_count(&conn, &job_id, false);
                        emit_import_ocr_event(
                            &app,
                            Some(&conn),
                            make_import_ocr_event(
                                "OCR_DOCUMENT_FAILED",
                                &job_id,
                                &case_id,
                                Some(file.document_id.clone()),
                                &file.file_name,
                                "ocr",
                                current,
                                total,
                                0,
                                file.page_count as i64,
                                "failed",
                                "OCR lỗi; batch tiếp tục file kế tiếp.",
                                Some(err),
                                false,
                            ),
                        );
                        // AI fallback: try analysis with whatever metadata exists
                        let _ = ai_cmd::process_pending_ai_analysis_jobs(&conn, 1);
                        // Emit explicit AI skipped/fallback event
                        emit_import_ocr_event(
                            &app,
                            Some(&conn),
                            make_import_ocr_event(
                                "AI_SKIPPED_AFTER_OCR_FAILURE",
                                &job_id,
                                &case_id,
                                Some(file.document_id.clone()),
                                &file.file_name,
                                "ai",
                                current,
                                total,
                                0,
                                0,
                                "review_required",
                                "OCR lỗi — AI fallback/review_required đã được tạo.",
                                None,
                                false,
                            ),
                        );
                        // Create review_required entry
                        let _ = create_review_entry(
                            &conn,
                            &file.document_id,
                            "ocr_failed_ai_review_required",
                        );
                    }
                }
            }
            Err(e) => {
                failed_files += 1;
                let err = short_error(&e);
                errors.push(format!("{}: {}", file_name, err));
                bump_pipeline_count(&conn, &job_id, false);
                persist_import_failed(
                    &conn,
                    "import_cmd.import_ocr_job",
                    "P0-import-ocr",
                    json!({ "job_id": job_id, "case_id": case_id, "path": path, "error": err }),
                );
                let _ = conn.execute(
                    "UPDATE import_jobs
                     SET processed_files = ?2,
                         failed_files = ?3,
                         current_file = ?4,
                         error_log = ?5,
                         updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                     WHERE import_job_id = ?1",
                    params![
                        job_id,
                        current,
                        failed_files,
                        path.to_string_lossy().to_string(),
                        to_json_array(&errors),
                    ],
                );
                emit_import_ocr_event(
                    &app,
                    Some(&conn),
                    make_import_ocr_event(
                        "IMPORT_FILE_FAILED",
                        &job_id,
                        &case_id,
                        None,
                        &file_name,
                        "import",
                        current,
                        total,
                        0,
                        0,
                        "failed",
                        "Import file lỗi; batch tiếp tục file kế tiếp.",
                        Some(err),
                        false,
                    ),
                );
            }
        }
    }

    update_case_counters(&conn, &case_id);
    let final_status = if imported_count == 0 {
        "failed"
    } else if ocr_failed > 0 || failed_files > 0 {
        "completed_with_errors"
    } else {
        "completed"
    };
    let import_job_status = if final_status == "failed" {
        "failed"
    } else if final_status == "completed_with_errors" {
        "completed"
    } else {
        "completed"
    };
    update_pipeline_status(
        &conn,
        &job_id,
        final_status,
        errors.first().map(String::as_str),
    );
    let _ = conn.execute(
        "UPDATE import_jobs
         SET job_status = ?2,
             processed_files = ?3,
             failed_files = ?4,
             current_file = NULL,
             error_log = ?5,
             completed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE import_job_id = ?1",
        params![
            job_id,
            import_job_status,
            total,
            failed_files,
            to_json_array(&errors),
        ],
    );
    let _ = persist_governed_event(
        &conn,
        if imported_count == 0 {
            "IMPORT_FAILED"
        } else {
            "IMPORT_DONE"
        },
        "import_cmd.import_ocr_job",
        "P0-import-ocr",
        &json!({
            "job_id": job_id,
            "case_id": case_id,
            "total_files": total,
            "imported_count": imported_count,
            "failed_files": failed_files,
            "ocr_done": ocr_done,
            "ocr_failed": ocr_failed,
            "page_images_extracted": page_images_extracted,
            "elapsed_ms": started.elapsed().as_millis(),
        }),
    );
    heartbeat_active.store(false, Ordering::SeqCst);
    let job_event_type = match final_status {
        "failed" => "JOB_FAILED",
        "completed_with_errors" => "JOB_PARTIAL_DONE",
        _ => "JOB_DONE",
    };
    let job_message = if final_status == "completed_with_errors" {
        format!(
            "Import xong có lỗi: import OK {}/{}, lỗi import {}, OCR OK {}, OCR lỗi {}.",
            imported_count, total, failed_files, ocr_done, ocr_failed
        )
    } else if final_status == "failed" {
        format!(
            "Import thất bại: import OK {}/{}, lỗi import {}, OCR OK {}, OCR lỗi {}.",
            imported_count, total, failed_files, ocr_done, ocr_failed
        )
    } else {
        format!(
            "Hoàn tất: import OK {}/{}, page extracted {}, OCR OK {}, AI đã xử lý.",
            imported_count, total, page_images_extracted, ocr_done
        )
    };
    emit_import_ocr_event(
        &app,
        Some(&conn),
        make_import_ocr_event(
            job_event_type,
            &job_id,
            &case_id,
            None,
            "",
            "done",
            total,
            total,
            0,
            0,
            final_status,
            &job_message,
            errors.first().cloned(),
            false,
        ),
    );
    info!(
        "IMPORT_OCR_JOB_DONE job_id={} case_id={} imported={} failed={} ocr_done={} ocr_failed={} elapsed_ms={}",
        job_id,
        case_id,
        imported_count,
        failed_files,
        ocr_done,
        ocr_failed,
        started.elapsed().as_millis()
    );
}

#[tauri::command]
pub fn start_import_ocr_folder_job(
    app: AppHandle,
    db: State<'_, DbState>,
    folder_path: String,
    auto_ocr: Option<bool>,
) -> Result<ImportOcrJobStartResult, String> {
    log::info!(
        "IMPORT_TRACE_BACKEND_ENTERED command=start_import_ocr_folder_job folder_path={} auto_ocr={:?} db_path={} init_error={:?}",
        folder_path,
        auto_ocr,
        db.db_path.display(),
        db.init_error
    );
    if let Some(init_error) = db.init_error.as_ref() {
        return Err(format!("IMPORT_DB_INIT_ERROR: {init_error}"));
    }
    let folder_input = folder_path.trim();
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
    let files = collect_supported_import_files(&root)?;
    if files.is_empty() {
        return Err(
            "IMPORT_FOLDER_NO_SUPPORTED_FILES: chỉ nhận PDF, ảnh scan hoặc file Office".to_string(),
        );
    }

    let auto_ocr = auto_ocr.unwrap_or(true);
    let worker_plan = derive_import_ocr_worker_plan();
    let job_id = generate_import_job_id();
    let (case_id, case_code) = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;
        let (case_id, case_code) = create_case_for_folder(&conn, &root)?;
        create_import_ocr_job_records(
            &conn,
            &job_id,
            &case_id,
            folder_input,
            files.len() as i64,
            auto_ocr,
            &worker_plan,
        )?;
        emit_import_ocr_event(
            &app,
            Some(&conn),
            make_import_ocr_event(
                "IMPORT_JOB_STARTED",
                &job_id,
                &case_id,
                None,
                "",
                "import",
                0,
                files.len() as i64,
                0,
                0,
                "queued",
                "Job import/OCR đã được đưa vào hàng đợi nền.",
                None,
                false,
            ),
        );
        (case_id, case_code)
    };

    let db_path = db.db_path.clone();
    thread::spawn({
        let app = app.clone();
        let job_id = job_id.clone();
        let case_id = case_id.clone();
        let case_code = case_code.clone();
        let files = files.clone();
        move || {
            run_import_ocr_background_job(
                app, db_path, job_id, case_id, case_code, files, auto_ocr, false,
            );
        }
    });

    Ok(ImportOcrJobStartResult {
        job_id,
        case_id,
        total_files: files.len() as i64,
        status: "queued".to_string(),
        auto_ocr,
        worker_plan,
    })
}

#[tauri::command]
pub fn start_import_ocr_files_job(
    app: AppHandle,
    db: State<'_, DbState>,
    paths: Vec<String>,
    case_id: Option<String>,
    auto_ocr: Option<bool>,
) -> Result<ImportOcrJobStartResult, String> {
    log::info!(
        "IMPORT_TRACE_BACKEND_ENTERED command=start_import_ocr_files_job path_count={} case_id={:?} auto_ocr={:?} db_path={} init_error={:?}",
        paths.len(),
        case_id,
        auto_ocr,
        db.db_path.display(),
        db.init_error
    );
    if let Some(init_error) = db.init_error.as_ref() {
        return Err(format!("IMPORT_DB_INIT_ERROR: {init_error}"));
    }
    if paths.is_empty() {
        return Err("IMPORT_FILES_EMPTY".to_string());
    }
    let mut files = paths
        .into_iter()
        .map(|path| PathBuf::from(path.trim()))
        .filter(|path| !path.as_os_str().is_empty())
        .enumerate()
        .map(|(idx, path)| {
            let relative_path = path
                .file_name()
                .and_then(|value| value.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| normalize_relative_path(&path));
            DiscoveredImportFile {
                path,
                relative_path,
                group_relative_path: None,
                import_order: idx as i64 + 1,
            }
        })
        .collect::<Vec<_>>();
    if files.is_empty() {
        return Err("IMPORT_FILES_EMPTY".to_string());
    }
    files.sort_by(|a, b| a.import_order.cmp(&b.import_order));

    let auto_ocr = auto_ocr.unwrap_or(true);
    let worker_plan = derive_import_ocr_worker_plan();
    let job_id = generate_import_job_id();
    let (target_case_id, case_code, duplicate_check) = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;
        let (target_case_id, case_code, duplicate_check) = if let Some(existing_case_id) = case_id {
            let case_code: String = conn
                .query_row(
                    "SELECT case_code FROM cases WHERE case_id = ?1",
                    params![existing_case_id.clone()],
                    |row| row.get(0),
                )
                .map_err(|e| format!("IMPORT_FILES_CASE_CHECK_FAILED: {e}"))?;
            (existing_case_id, case_code, true)
        } else {
            let (new_case_id, new_case_code) = create_case_for_files(&conn)?;
            (new_case_id, new_case_code, false)
        };
        create_import_ocr_job_records(
            &conn,
            &job_id,
            &target_case_id,
            "multi-file-import",
            files.len() as i64,
            auto_ocr,
            &worker_plan,
        )?;
        emit_import_ocr_event(
            &app,
            Some(&conn),
            make_import_ocr_event(
                "IMPORT_JOB_STARTED",
                &job_id,
                &target_case_id,
                None,
                "",
                "import",
                0,
                files.len() as i64,
                0,
                0,
                "queued",
                "Job import nhiều file/OCR đã được đưa vào hàng đợi nền.",
                None,
                false,
            ),
        );
        (target_case_id, case_code, duplicate_check)
    };

    let db_path = db.db_path.clone();
    thread::spawn({
        let app = app.clone();
        let job_id = job_id.clone();
        let case_id = target_case_id.clone();
        let case_code = case_code.clone();
        let files = files.clone();
        move || {
            run_import_ocr_background_job(
                app,
                db_path,
                job_id,
                case_id,
                case_code,
                files,
                auto_ocr,
                duplicate_check,
            );
        }
    });

    Ok(ImportOcrJobStartResult {
        job_id,
        case_id: target_case_id,
        total_files: files.len() as i64,
        status: "queued".to_string(),
        auto_ocr,
        worker_plan,
    })
}

#[tauri::command]
pub fn import_folder(
    db: State<'_, DbState>,
    folder_path: String,
) -> Result<ImportFolderResult, String> {
    log::info!(
        "IMPORT_TRACE_BACKEND_ENTERED command=import_folder folder_path={} db_path={} init_error={:?}",
        folder_path,
        db.db_path.display(),
        db.init_error
    );
    if let Some(init_error) = db.init_error.as_ref() {
        return Err(format!("IMPORT_DB_INIT_ERROR: {init_error}"));
    }
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

    let files = collect_supported_import_files(&root)?;
    if files.is_empty() {
        return Err(
            "IMPORT_FOLDER_NO_SUPPORTED_FILES: chỉ nhận PDF, ảnh scan hoặc file Office".to_string(),
        );
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

    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    log::info!(
        "IMPORT_TRACE db_counts_before command=import_folder db_path={} cases={} documents={} pages={} governed_events={}",
        db.db_path.display(),
        db_count(&conn, "cases"),
        db_count(&conn, "documents"),
        db_count(&conn, "pages"),
        db_count(&conn, "governed_events")
    );

    let case_id = generate_case_id();
    let case_code = generate_case_code();
    let job_id = generate_import_job_id();
    let total_files = files.len() as i64;

    let _ = persist_governed_event(
        &conn,
        "IMPORT_STARTED",
        "import_cmd.import_folder",
        "P0-import",
        &serde_json::json!({
            "case_id": case_id,
            "job_id": job_id,
            "folder": folder_input,
            "total_files": total_files,
        }),
    );

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
    let group_sort_orders = build_group_sort_orders(&files);

    for file_plan in files {
        let path = file_plan.path;
        let relative_path = file_plan.relative_path;
        let group_relative_path = file_plan.group_relative_path;
        let import_order = file_plan.import_order;
        processed_files += 1;

        let group_id = match ensure_document_group_hierarchy(
            &conn,
            &case_id,
            group_relative_path.as_deref(),
            &group_sort_orders,
        ) {
            Ok(group_id) => group_id,
            Err(e) => {
                failed_files += 1;
                let message = format!("DOCUMENT_GROUP_FAILED {}: {e}", relative_path);
                errors.push(message.clone());
                persist_import_failed(
                    &conn,
                    "import_cmd.import_folder",
                    "P0-import",
                    serde_json::json!({
                        "job_id": job_id,
                        "path": path,
                        "relative_path": relative_path,
                        "error": message
                    }),
                );
                continue;
            }
        };

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                failed_files += 1;
                let message = format!("METADATA_FAILED {}: {e}", path.to_string_lossy());
                errors.push(message.clone());
                persist_import_failed(
                    &conn,
                    "import_cmd.import_folder",
                    "P0-import",
                    serde_json::json!({ "job_id": job_id, "path": path, "error": message }),
                );
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
        let file_hash = match hash_file(&path) {
            Ok(value) => value,
            Err(e) => {
                failed_files += 1;
                persist_import_failed(
                    &conn,
                    "import_cmd.import_folder",
                    "P0-import",
                    serde_json::json!({ "job_id": job_id, "path": path, "error": e }),
                );
                errors.push(e);
                continue;
            }
        };

        let managed_path =
            match storage::copy_to_originals(&path, &case_code, &document_id, &file_name) {
                Ok(value) => value,
                Err(e) => {
                    failed_files += 1;
                    persist_import_failed(
                        &conn,
                        "import_cmd.import_folder",
                        "P0-import",
                        serde_json::json!({ "job_id": job_id, "path": path, "error": e }),
                    );
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
                file_status,
                original_path,
                managed_path,
                group_id,
                relative_path,
                import_sequence,
                revision_no,
                created_at,
                updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4,
                ?11, ?5, ?6, ?7, ?7, ?8, ?9, ?10,
                'pending',
                'imported', ?4, NULL, ?12, ?13, ?14, 1,
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
                file_hash,
                group_id.as_deref(),
                &relative_path,
                import_order,
            ],
        );

        match insert_result {
            Ok(_) => {
                if ext_no_dot == "pdf" {
                    let _ = persist_governed_event(
                        &conn,
                        "PAGE_IMAGE_EXTRACT_STARTED",
                        "import_cmd.import_folder",
                        "page-image-phase1",
                        &serde_json::json!({ "document_id": document_id, "source_pdf_path": file_path }),
                    );
                    match run_page_image_extraction(&conn, &document_id, &managed_path) {
                        Ok(extracted_pages) => {
                            let _ = persist_governed_event(
                                &conn,
                                "PAGE_IMAGE_EXTRACT_DONE",
                                "import_cmd.import_folder",
                                "page-image-phase1",
                                &serde_json::json!({ "document_id": document_id, "source_pdf_path": file_path, "pages": extracted_pages }),
                            );
                        }
                        Err(e) => {
                            let _ = persist_governed_event(
                                &conn,
                                "PAGE_IMAGE_EXTRACT_FAILED",
                                "import_cmd.import_folder",
                                "page-image-phase1",
                                &serde_json::json!({ "document_id": document_id, "source_pdf_path": file_path, "error": e }),
                            );
                            for i in 1..=page_count {
                                if let Ok(page_id) = insert_page_row(&conn, &document_id, i, None) {
                                    let _ = insert_ocr_result(&conn, &page_id, "", 0.0, "pending");
                                }
                            }
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
                    group_id,
                    relative_path: Some(relative_path),
                    import_order: Some(import_order),
                });
            }
            Err(e) => {
                failed_files += 1;
                let message = format!("INSERT_DOCUMENT_FAILED {}: {e}", path.to_string_lossy());
                persist_import_failed(
                    &conn,
                    "import_cmd.import_folder",
                    "P0-import",
                    serde_json::json!({ "job_id": job_id, "path": path, "error": message }),
                );
                errors.push(message);
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
        let _ = conn.execute(
            "DELETE FROM cases WHERE case_id = ?1",
            params![case_id.clone()],
        );
        return Err(
            "IMPORT_FOLDER_NO_IMPORTED_FILES: không tạo hồ sơ vì không có file hợp lệ được nhập"
                .to_string(),
        );
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
    // Persist governed event for audit trail
    let _ = persist_governed_event(
        &conn,
        "IMPORT_DONE",
        "import_cmd.import_folder",
        "P0-import",
        &serde_json::json!({
            "case_id": case_id,
            "job_id": job_id,
            "total_files": total_files,
            "imported_count": imported_count,
            "failed_files": failed_files,
            "folder": folder_input,
        }),
    );
    log::info!(
        "IMPORT_TRACE db_counts_after command=import_folder db_path={} case_id={} job_id={} cases={} documents={} pages={} governed_events={}",
        db.db_path.display(),
        case_id,
        job_id,
        db_count(&conn, "cases"),
        db_count(&conn, "documents"),
        db_count(&conn, "pages"),
        db_count(&conn, "governed_events")
    );

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
    log::info!(
        "IMPORT_TRACE_BACKEND_ENTERED command=import_multiple_files path_count={} case_id={:?} db_path={} init_error={:?}",
        paths.len(),
        case_id,
        db.db_path.display(),
        db.init_error
    );
    if let Some(init_error) = db.init_error.as_ref() {
        return Err(format!("IMPORT_DB_INIT_ERROR: {init_error}"));
    }
    if paths.is_empty() {
        return Err("IMPORT_FILES_EMPTY".to_string());
    }

    let has_supported_file = paths.iter().any(|raw_path| {
        let path = PathBuf::from(raw_path.trim());
        path.exists() && path.is_file() && is_supported_file(&path)
    });
    if !has_supported_file {
        return Err(
            "IMPORT_FILES_NO_SUPPORTED_FILES: chỉ nhận PDF, ảnh scan hoặc file Office".to_string(),
        );
    }

    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    log::info!(
        "IMPORT_TRACE db_counts_before command=import_multiple_files db_path={} cases={} documents={} pages={} governed_events={}",
        db.db_path.display(),
        db_count(&conn, "cases"),
        db_count(&conn, "documents"),
        db_count(&conn, "pages"),
        db_count(&conn, "governed_events")
    );
    let (target_case_id, target_case_code, created_new_case) =
        if let Some(existing_case_id) = case_id {
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

    let _ = persist_governed_event(
        &conn,
        "IMPORT_STARTED",
        "import_cmd.import_multiple_files",
        "P0-import",
        &serde_json::json!({
            "case_id": target_case_id,
            "requested_paths": paths.len(),
            "target_case_code": target_case_code,
        }),
    );

    let mut imported = Vec::<ImportedFile>::new();
    let mut duplicates = Vec::<String>::new();
    let mut errors = Vec::<String>::new();

    for (idx, raw_path) in paths.into_iter().enumerate() {
        let path = PathBuf::from(raw_path.trim());
        if !path.exists() || !path.is_file() || !is_supported_file(&path) {
            let message = format!("UNSUPPORTED_OR_MISSING: {}", path.to_string_lossy());
            persist_import_failed(
                &conn,
                "import_cmd.import_multiple_files",
                "P0-import",
                serde_json::json!({ "case_id": target_case_id, "path": path, "error": message }),
            );
            errors.push(message);
            continue;
        }

        let file_hash = match hash_file(&path) {
            Ok(value) => value,
            Err(e) => {
                persist_import_failed(
                    &conn,
                    "import_cmd.import_multiple_files",
                    "P0-import",
                    serde_json::json!({ "case_id": target_case_id, "path": path, "error": e }),
                );
                errors.push(e);
                continue;
            }
        };

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("tai_lieu")
            .to_string();
        let relative_path = file_name.clone();
        let import_order = idx as i64 + 1;

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
                let message = format!("METADATA_FAILED {}: {e}", path.to_string_lossy());
                persist_import_failed(
                    &conn,
                    "import_cmd.import_multiple_files",
                    "P0-import",
                    serde_json::json!({ "case_id": target_case_id, "path": path, "error": message }),
                );
                errors.push(message);
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
                    persist_import_failed(
                        &conn,
                        "import_cmd.import_multiple_files",
                        "P0-import",
                        serde_json::json!({ "case_id": target_case_id, "path": path, "error": e }),
                    );
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
                file_status, original_path, managed_path, revision_no,
                relative_path, import_sequence,
                created_at, updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9, ?10, ?11,
                'pending', 'imported', ?4, NULL, 1,
                ?12, ?13,
                strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
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
                relative_path,
                import_order,
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
                let _ =
                    create_review_entry(&conn, &document_id, "ocr_pending_after_multi_file_import");
                imported.push(ImportedFile {
                    document_id,
                    file_path,
                    file_name,
                    file_ext,
                    file_size,
                    page_count,
                    group_id: None,
                    relative_path: Some(relative_path),
                    import_order: Some(import_order),
                });
            }
            Err(e) => {
                let message = format!("INSERT_DOCUMENT_FAILED {}: {e}", path.to_string_lossy());
                persist_import_failed(
                    &conn,
                    "import_cmd.import_multiple_files",
                    "P0-import",
                    serde_json::json!({ "case_id": target_case_id, "path": path, "error": message }),
                );
                errors.push(message);
            }
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
        let _ = conn.execute(
            "DELETE FROM cases WHERE case_id = ?1",
            params![target_case_id.clone()],
        );
        return Err(
            "IMPORT_FILES_NO_IMPORTED_FILES: không tạo hồ sơ vì không có file hợp lệ được nhập"
                .to_string(),
        );
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

    // Persist governed event for audit trail
    let _ = persist_governed_event(
        &conn,
        "IMPORT_DONE",
        "import_cmd.import_multiple_files",
        "P0-import",
        &serde_json::json!({
            "case_id": target_case_id,
            "imported_count": imported.len(),
            "duplicate_count": duplicates.len(),
            "error_count": errors.len(),
        }),
    );
    log::info!(
        "IMPORT_TRACE db_counts_after command=import_multiple_files db_path={} case_id={} cases={} documents={} pages={} governed_events={}",
        db.db_path.display(),
        target_case_id,
        db_count(&conn, "cases"),
        db_count(&conn, "documents"),
        db_count(&conn, "pages"),
        db_count(&conn, "governed_events")
    );

    Ok(ImportMultipleFilesResult {
        case_id: target_case_id,
        total_files: (imported.len() + duplicates.len() + errors.len()) as i64,
        imported,
        duplicates,
        errors,
    })
}

#[tauri::command]
pub fn debug_import_test_path(
    _app: AppHandle,
    db: State<'_, DbState>,
    path: String,
) -> Result<DebugImportTestPathResult, String> {
    log::info!(
        "IMPORT_TRACE_BACKEND_ENTERED command=debug_import_test_path path={} db_path={} init_error={:?}",
        path,
        db.db_path.display(),
        db.init_error
    );
    if let Some(init_error) = db.init_error.as_ref() {
        return Err(format!("IMPORT_DB_INIT_ERROR: {init_error}"));
    }
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("DEBUG_IMPORT_PATH_EMPTY".to_string());
    }
    let input_path = PathBuf::from(trimmed);
    if !input_path.exists() {
        return Err(format!("DEBUG_IMPORT_PATH_NOT_FOUND: {trimmed}"));
    }
    if !input_path.is_dir() && !input_path.is_file() {
        return Err(format!("DEBUG_IMPORT_PATH_UNSUPPORTED_TYPE: {trimmed}"));
    }
    let files = if input_path.is_dir() {
        collect_supported_import_files(&input_path)?
    } else {
        let relative_path = input_path
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| normalize_relative_path(&input_path));
        vec![DiscoveredImportFile {
            path: input_path.clone(),
            relative_path,
            group_relative_path: None,
            import_order: 1,
        }]
    };
    if files.is_empty() {
        return Err("DEBUG_IMPORT_NO_SUPPORTED_FILES".to_string());
    }

    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let cases_before = db_count(&conn, "cases");
    let documents_before = db_count(&conn, "documents");
    let pages_before = db_count(&conn, "pages");
    let events_before = db_count(&conn, "governed_events");

    let (case_id, case_code) = if input_path.is_dir() {
        create_case_for_folder(&conn, &input_path)?
    } else {
        create_case_for_files(&conn)?
    };
    let job_id = format!("debug_{}", generate_page_id());
    let _ = persist_governed_event(
        &conn,
        "IMPORT_STARTED",
        "import_cmd.debug_import_test_path",
        "P0-import-debug",
        &json!({ "case_id": case_id, "job_id": job_id, "path": trimmed, "db_path": db.db_path.display().to_string() }),
    );

    let mut imported = Vec::<ImportedFile>::new();
    let mut errors = Vec::<String>::new();
    let group_sort_orders = build_group_sort_orders(&files);
    for file in &files {
        let group_id = match ensure_document_group_hierarchy(
            &conn,
            &case_id,
            file.group_relative_path.as_deref(),
            &group_sort_orders,
        ) {
            Ok(group_id) => group_id,
            Err(e) => {
                persist_import_failed(
                    &conn,
                    "import_cmd.debug_import_test_path",
                    "P0-import-debug",
                    json!({ "case_id": case_id, "path": file.path.display().to_string(), "relative_path": &file.relative_path, "error": e }),
                );
                errors.push(e);
                continue;
            }
        };
        match import_one_file_for_case(
            &conn,
            &file.path,
            &case_id,
            &case_code,
            true,
            group_id.as_deref(),
            Some(&file.relative_path),
            Some(file.import_order),
        ) {
            Ok(item) => imported.push(item),
            Err(e) => {
                persist_import_failed(
                    &conn,
                    "import_cmd.debug_import_test_path",
                    "P0-import-debug",
                    json!({ "case_id": case_id, "path": file.path.display().to_string(), "relative_path": &file.relative_path, "error": e }),
                );
                errors.push(e);
            }
        }
    }

    log::info!(
        "IMPORT_TRACE debug_import_test_path imported_count={} error_count={} case_id={} path={}",
        imported.len(),
        errors.len(),
        case_id,
        trimmed
    );

    let status = if imported.is_empty() {
        let _ = persist_governed_event(
            &conn,
            "IMPORT_FAILED",
            "import_cmd.debug_import_test_path",
            "P0-import-debug",
            &json!({ "case_id": case_id, "path": trimmed, "errors": errors }),
        );
        "failed".to_string()
    } else {
        let doc_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM documents WHERE case_id = ?1",
                params![case_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let total_pages: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(page_count), 0) FROM documents WHERE case_id = ?1",
                params![case_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let _ = conn.execute(
            "UPDATE cases SET document_count = ?1, total_pages = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE case_id = ?3",
            params![doc_count, total_pages, case_id],
        );
        let _ = persist_governed_event(
            &conn,
            "IMPORT_DONE",
            "import_cmd.debug_import_test_path",
            "P0-import-debug",
            &json!({ "case_id": case_id, "path": trimmed, "imported_count": imported.len() }),
        );
        "done".to_string()
    };

    let result = DebugImportTestPathResult {
        db_path: db.db_path.display().to_string(),
        case_id,
        job_id,
        status,
        documents_before,
        documents_after: db_count(&conn, "documents"),
        cases_before,
        cases_after: db_count(&conn, "cases"),
        pages_before,
        pages_after: db_count(&conn, "pages"),
        events_before,
        events_after: db_count(&conn, "governed_events"),
        imported,
        errors,
    };
    log::info!("IMPORT_TRACE debug_import_test_path result={:?}", result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{
        build_group_sort_orders, collect_supported_import_files, count_pdf_pages,
        create_case_for_folder, ensure_document_group_hierarchy, import_one_file_for_case,
    };
    use crate::db;
    use rusqlite::Connection;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn count_pdf_pages_returns_zero_for_missing_file() {
        let page_count = count_pdf_pages(Path::new("__missing_test_file__.pdf"));
        assert_eq!(page_count, 0);
    }

    fn temp_test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "vks_ecms_{name}_{}_{}",
            std::process::id(),
            super::now_millis()
        ))
    }

    #[test]
    fn collect_supported_import_files_preserves_groups_and_natural_order() {
        let root = temp_test_dir("fnd007_collect");
        fs::create_dir_all(root.join("tap_1")).expect("tap_1");
        fs::create_dir_all(root.join("tap_2")).expect("tap_2");
        fs::create_dir_all(root.join("tap_10")).expect("tap_10");
        fs::create_dir_all(root.join("__MACOSX")).expect("junk");
        fs::write(root.join("tap_1").join("10.pdf"), b"%PDF-1.4\n").expect("10");
        fs::write(root.join("tap_1").join("2.pdf"), b"%PDF-1.4\n").expect("2");
        fs::write(root.join("tap_1").join("1.pdf"), b"%PDF-1.4\n").expect("1");
        fs::write(root.join("tap_2").join("1.pdf"), b"%PDF-1.4\n").expect("tap2");
        fs::write(root.join("tap_10").join("1.pdf"), b"%PDF-1.4\n").expect("tap10");
        fs::write(root.join("__MACOSX").join("junk.pdf"), b"%PDF-1.4\n").expect("junk pdf");
        fs::write(root.join("tap_1").join("~$skip.pdf"), b"%PDF-1.4\n").expect("skip");

        let files = collect_supported_import_files(&root).expect("collect");
        let relative_paths = files
            .iter()
            .map(|file| file.relative_path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            relative_paths,
            vec![
                "tap_1/1.pdf",
                "tap_1/2.pdf",
                "tap_1/10.pdf",
                "tap_2/1.pdf",
                "tap_10/1.pdf"
            ]
        );
        assert_eq!(files[0].group_relative_path.as_deref(), Some("tap_1"));
        assert_eq!(files[0].import_order, 1);
        assert_eq!(files[4].import_order, 5);

        let group_orders = build_group_sort_orders(&files);
        assert_eq!(group_orders.get("tap_1"), Some(&1));
        assert_eq!(group_orders.get("tap_2"), Some(&2));
        assert_eq!(group_orders.get("tap_10"), Some(&3));

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn ensure_document_group_hierarchy_creates_parent_and_child_groups() {
        let conn = Connection::open_in_memory().expect("open memory db");
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE cases(case_id TEXT PRIMARY KEY);
             INSERT INTO cases(case_id) VALUES ('case-1');
             CREATE TABLE document_groups (
                group_id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
                parent_group_id TEXT REFERENCES document_groups(group_id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                relative_path TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                UNIQUE(case_id, relative_path)
             );",
        )
        .expect("schema");

        let mut sort_orders = HashMap::<String, i64>::new();
        sort_orders.insert("tap_10".to_string(), 10);
        sort_orders.insert("tap_10/sub_2".to_string(), 11);

        let child_group_id =
            ensure_document_group_hierarchy(&conn, "case-1", Some("tap_10/sub_2"), &sort_orders)
                .expect("ensure child")
                .expect("child group id");

        let group_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM document_groups", [], |row| row.get(0))
            .expect("count");
        assert_eq!(group_count, 2);

        let (name, parent_group_id): (String, String) = conn
            .query_row(
                "SELECT name, parent_group_id FROM document_groups WHERE group_id = ?1",
                [child_group_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("child row");
        assert_eq!(name, "sub_2");
        assert!(!parent_group_id.is_empty());
    }

    #[test]
    fn folder_import_metadata_persists_relative_path_group_and_natural_order() {
        let fixture_pdf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("phanmem root")
            .join("test_pdfs")
            .join("test_doc_1.pdf");
        assert!(
            fixture_pdf.exists(),
            "fixture PDF missing: {}",
            fixture_pdf.display()
        );

        let root = temp_test_dir("folder_with_1_2_10_pdf");
        let group_dir = root.join("tap_1");
        fs::create_dir_all(&group_dir).expect("fixture group dir");
        fs::copy(&fixture_pdf, group_dir.join("10.pdf")).expect("copy 10");
        fs::copy(&fixture_pdf, group_dir.join("2.pdf")).expect("copy 2");
        fs::copy(&fixture_pdf, group_dir.join("1.pdf")).expect("copy 1");

        let db_path = temp_test_dir("fnd007_import_db").with_extension("db");
        let _ = fs::remove_file(&db_path);
        db::init(&db_path).expect("db init");
        let conn = Connection::open(&db_path).expect("open db");
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .expect("pragma");
        let (case_id, case_code) = create_case_for_folder(&conn, &root).expect("case");

        let files = collect_supported_import_files(&root).expect("collect");
        let group_sort_orders = build_group_sort_orders(&files);
        for file in &files {
            let group_id = ensure_document_group_hierarchy(
                &conn,
                &case_id,
                file.group_relative_path.as_deref(),
                &group_sort_orders,
            )
            .expect("group");
            import_one_file_for_case(
                &conn,
                &file.path,
                &case_id,
                &case_code,
                false,
                group_id.as_deref(),
                Some(&file.relative_path),
                Some(file.import_order),
            )
            .expect("import file");
        }

        let mut stmt = conn
            .prepare(
                "SELECT relative_path, group_id, import_sequence
                 FROM documents
                 WHERE case_id = ?1
                 ORDER BY import_sequence ASC",
            )
            .expect("prepare documents");
        let rows = stmt
            .query_map([case_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ))
            })
            .expect("query documents")
            .collect::<Result<Vec<_>, _>>()
            .expect("rows");
        assert_eq!(
            rows.iter()
                .map(|(relative_path, _, _)| relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["tap_1/1.pdf", "tap_1/2.pdf", "tap_1/10.pdf"]
        );
        assert!(rows.iter().all(|(_, group_id, _)| group_id.is_some()));
        assert_eq!(
            rows.iter()
                .map(|(_, _, import_order)| import_order.unwrap_or_default())
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );

        let group_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM document_groups WHERE case_id = ?1",
                [case_id.as_str()],
                |row| row.get(0),
            )
            .expect("group count");
        assert_eq!(group_count, 1);

        drop(stmt);
        drop(conn);
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_file(&db_path);
    }
}
