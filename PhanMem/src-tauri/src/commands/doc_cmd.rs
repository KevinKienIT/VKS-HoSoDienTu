// VKS ECMS — Document Tauri commands

use crate::commands::module_cmd::DbState;
use crate::storage;
use log::warn;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State};

static DOC_COUNTER: AtomicU64 = AtomicU64::new(0);
const IMPORT_ALLOWED_EXTENSIONS: &[&str] = &[
    "pdf", "png", "jpg", "jpeg", "tif", "tiff", "bmp", "doc", "docx", "xls", "xlsx", "ppt",
    "pptx", "rtf",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentSummary {
    pub document_id: String,
    pub case_id: String,
    pub display_name: String,
    pub original_filename: String,
    pub file_path: String,
    pub document_type: String,
    pub page_count: i32,
    pub status: String,
    pub created_at: String,
    pub file_missing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportDocumentInput {
    pub case_id: String,
    pub file_path: String,
    pub document_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentGroup {
    pub group_key: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexRebuildResult {
    pub indexed_documents: i64,
    pub indexed_pages: i64,
    pub indexed_entities: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageOcrResult {
    pub page_id: String,
    pub document_id: String,
    pub page_index: i32,
    pub ocr_text: String,
    pub ocr_formatted_text: String,
    pub ocr_layout_blocks: String,
    pub confidence: f64,
    pub engine: String,
    pub but_luc: Option<String>,
    pub transcription_state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilenameSuggestion {
    pub document_id: String,
    pub suggested_filename: String,
    pub current_filename: String,
    pub missing_fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkProductSummary {
    pub work_product_id: String,
    pub case_id: String,
    pub title: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrRunResult {
    pub document_id: String,
    pub processed_pages: i64,
    pub failed_pages: i64,
    pub handwritten_pages: i64,
    pub average_confidence: f64,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone)]
struct PageOcrPayload {
    page_num: i32,
    image_path: Option<String>,
    text: String,
    formatted_text: String,
    layout_blocks_json: String,
    confidence: f64,
    engine: String,
    has_handwriting: bool,
    regions_json: String,
    handwriting_regions_json: String,
    warning: Option<String>,
    extracted_fields_json: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RescanOcrOptions {
    pub scope: Option<String>,
    pub quality: Option<String>,
    pub mode: Option<String>,
    pub detect_layout: Option<bool>,
    pub detect_marks: Option<bool>,
    pub rebuild_index: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageLayoutBlock {
    pub id: String,
    pub document_id: String,
    pub page_id: String,
    pub page_number: i32,
    pub block_type: String,
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub page_width: Option<f64>,
    pub page_height: Option<f64>,
    pub confidence: f64,
    pub reading_order: i32,
    pub engine: String,
    pub source: String,
    pub reviewed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtractedField {
    pub id: String,
    pub document_id: String,
    pub field_name: String,
    pub field_value: String,
    pub page_number: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub confidence: f64,
    pub source: String,
    pub created_at: String,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn is_supported_import_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMPORT_ALLOWED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn is_image_import_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "tif" | "tiff" | "bmp"
            )
        })
        .unwrap_or(false)
}

fn is_ocr_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "pdf" | "png" | "jpg" | "jpeg" | "tif" | "tiff" | "bmp"
            )
        })
        .unwrap_or(false)
}

fn chrono_like_now() -> String {
    format!("{}", now_millis())
}

fn generate_document_id() -> String {
    let n = DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("doc-{}-{}", now_millis(), n)
}

fn generate_audit_event_id() -> String {
    let n = DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("audit-{}-{}", now_millis(), n)
}

fn generate_ocr_result_id() -> String {
    let n = DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("ocr-{}-{}", now_millis(), n)
}

fn generate_work_product_id() -> String {
    let n = DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("wp-{}-{}", now_millis(), n)
}

fn generate_layout_block_id() -> String {
    let n = DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("layout-{}-{}", now_millis(), n)
}

fn generate_extracted_field_id() -> String {
    let n = DOC_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("field-{}-{}", now_millis(), n)
}

fn python_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("python")
}

fn is_lock_error(err: &io::Error) -> bool {
    err.raw_os_error()
        .map(|c| c == 32 || c == 33)
        .unwrap_or(false)
}

fn script_path(module: &str) -> PathBuf {
    let rel = module.replace('.', "/") + ".py";
    python_dir().join(rel)
}

fn validate_ocr_env(module: &str, lang: Option<&str>) -> Result<(), String> {
    let script = script_path(module);
    if !script.exists() {
        return Err(format!(
            "OCR_ENV_SCRIPT_MISSING: module={module}; script_path={}",
            script.display()
        ));
    }
    if !script.is_file() {
        return Err(format!(
            "OCR_ENV_SCRIPT_INVALID: module={module}; script_path={}",
            script.display()
        ));
    }
    if let Some(l) = lang {
        let ll = l.trim().to_lowercase();
        if ll != "vie" && ll != "eng" && ll != "vie+eng" {
            return Err(format!(
                "OCR_ENV_LANG_INVALID: lang={}; expected=vie|eng|vie+eng; script_path={}",
                l,
                script.display()
            ));
        }
    }
    Ok(())
}

fn run_rename_with_retry(current: &Path, target: &Path) -> Result<(), String> {
    let waits_ms = [40_u64, 120, 260];
    for (idx, wait_ms) in waits_ms.iter().enumerate() {
        match fs::rename(current, target) {
            Ok(_) => return Ok(()),
            Err(e) if is_lock_error(&e) => {
                warn!(
                    "file-lock path={} op=rename_document_file attempt={} wait_ms={} final_error={}",
                    current.display(),
                    idx + 1,
                    wait_ms,
                    e
                );
                if idx + 1 < waits_ms.len() {
                    thread::sleep(Duration::from_millis(*wait_ms));
                    continue;
                }
                return Err(format!(
                    "RENAME_FILE_LOCKED:path={}:op=rename_document_file:attempt={}:wait_ms={}:final_error={}",
                    current.display(),
                    idx + 1,
                    wait_ms,
                    e
                ));
            }
            Err(e) => return Err(format!("RENAME_FS_FAILED: {e}")),
        }
    }
    Err("RENAME_FS_FAILED: unknown".to_string())
}

fn run_python_json(args: &[String]) -> Result<serde_json::Value, String> {
    let candidates: &[&str] = if cfg!(windows) {
        &["python", "py"]
    } else {
        &["python3", "python"]
    };
    let mut last_start_error = String::new();
    let mut selected_python = String::new();
    let mut output = None;
    let mut module_name = String::new();
    let mut lang_value = String::new();
    for idx in 0..args.len() {
        if args[idx] == "-m" {
            if let Some(m) = args.get(idx + 1) {
                module_name = m.clone();
            }
        }
        if args[idx] == "--lang" {
            if let Some(v) = args.get(idx + 1) {
                lang_value = v.clone();
            }
        }
    }
    if !module_name.trim().is_empty() {
        validate_ocr_env(
            &module_name,
            if lang_value.trim().is_empty() {
                None
            } else {
                Some(lang_value.as_str())
            },
        )?;
    }
    for candidate in candidates {
        let mut command = Command::new(candidate);
        command.env("PYTHONUTF8", "1");
        command.env("PYTHONIOENCODING", "utf-8");
        if *candidate == "py" {
            command.arg("-3");
        }
        match command.args(args).current_dir(python_dir()).output() {
            Ok(value) => {
                output = Some(value);
                selected_python = (*candidate).to_string();
                break;
            }
            Err(e) => {
                last_start_error = format!("{candidate}: {e}");
            }
        }
    }
    let output = output.ok_or_else(|| {
        format!(
            "OCR_PYTHON_START_FAILED: python_path=none; script_path={}; lang={}; stderr_snippet={}; exit_status=not_started",
            if module_name.is_empty() { "unknown".to_string() } else { script_path(&module_name).to_string_lossy().to_string() },
            if lang_value.is_empty() { "n/a".to_string() } else { lang_value.clone() },
            last_start_error
        )
    })?;
    if !output.status.success() {
        let stderr_snippet = String::from_utf8_lossy(&output.stderr)
            .chars()
            .take(400)
            .collect::<String>();
        if stderr_snippet.to_lowercase().contains("timed out")
            || stderr_snippet.to_lowercase().contains("timeout")
        {
            warn!(
                "ocr-env python_path={} script_path={} lang={} stderr_snippet={} exit_status={}",
                selected_python,
                if module_name.is_empty() {
                    "unknown".to_string()
                } else {
                    script_path(&module_name).to_string_lossy().to_string()
                },
                if lang_value.is_empty() {
                    "n/a".to_string()
                } else {
                    lang_value.clone()
                },
                stderr_snippet,
                output.status
            );
        }
        return Err(format!(
            "OCR_PYTHON_FAILED: python_path={}; script_path={}; lang={}; stderr_snippet={}; exit_status={}; stdout={}",
            selected_python,
            if module_name.is_empty() { "unknown".to_string() } else { script_path(&module_name).to_string_lossy().to_string() },
            if lang_value.is_empty() { "n/a".to_string() } else { lang_value.clone() },
            stderr_snippet,
            output.status,
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    let stdout_text = String::from_utf8_lossy(&output.stdout).to_string();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&stdout_text) {
        return Ok(value);
    }

    let first_obj = stdout_text.find('{');
    let last_obj = stdout_text.rfind('}');
    if let (Some(start), Some(end)) = (first_obj, last_obj) {
        if start < end {
            let candidate = &stdout_text[start..=end];
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(candidate) {
                return Ok(value);
            }
        }
    }

    serde_json::from_slice::<serde_json::Value>(&output.stdout).map_err(|e| {
        format!(
            "OCR_JSON_PARSE_FAILED: python_path={}; script_path={}; lang={}; stderr_snippet={}; exit_status={}; parse_error={e}; stdout={}",
            selected_python,
            if module_name.is_empty() { "unknown".to_string() } else { script_path(&module_name).to_string_lossy().to_string() },
            if lang_value.is_empty() { "n/a".to_string() } else { lang_value.clone() },
            String::from_utf8_lossy(&output.stderr).chars().take(240).collect::<String>(),
            output.status,
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn upsert_ocr_text(
    conn: &rusqlite::Connection,
    document_id: &str,
    page_index: i32,
    image_path: Option<&str>,
    ocr_text: &str,
    ocr_formatted_text: &str,
    ocr_layout_blocks: &str,
    confidence: f64,
    engine: &str,
    has_handwriting: bool,
    regions_json: &str,
    handwriting_regions_json: &str,
) -> Result<String, String> {
    let page_id: String = match conn.query_row(
        "SELECT page_id FROM pages WHERE document_id = ?1 AND page_index = ?2",
        params![document_id, page_index],
        |row| row.get(0),
    ) {
        Ok(existing) => existing,
        Err(_) => {
            let page_id = generate_document_id().replace("doc-", "page-");
            conn.execute(
                "INSERT INTO pages (page_id, document_id, page_index, image_path, created_at)
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                params![page_id, document_id, page_index, image_path],
            )
            .map_err(|e| format!("OCR_PAGE_INSERT_FAILED: {e}"))?;
            page_id
        }
    };

    conn.execute(
        "UPDATE pages
         SET image_path = COALESCE(?1, image_path),
             ocr_text = ?2,
             ocr_formatted_text = ?3,
             ocr_layout_blocks = ?4,
             ocr_preview = substr(COALESCE(NULLIF(?3, ''), ?2), 1, 500),
             confidence = ?5,
             has_handwriting = ?6,
             handwriting_regions = ?7,
             regions = ?8,
             transcription_state = CASE WHEN ?6 = 1 THEN 'interpolated_pending_review' ELSE 'direct' END
         WHERE page_id = ?9",
        params![
            image_path,
            ocr_text,
            ocr_formatted_text,
            ocr_layout_blocks,
            confidence,
            if has_handwriting { 1 } else { 0 },
            handwriting_regions_json,
            regions_json,
            page_id
        ],
    )
    .map_err(|e| format!("OCR_PAGE_UPDATE_FAILED: {e}"))?;

    let ocr_result_id = generate_ocr_result_id();
    conn.execute(
        "INSERT INTO ocr_results (
            ocr_result_id, page_id, engine, raw_text, confidence,
            is_handwriting, transcription_state, created_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6,
            CASE WHEN ?6 = 1 THEN 'interpolated_pending_review' ELSE 'direct' END,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            ocr_result_id,
            page_id,
            engine,
            ocr_text,
            confidence,
            if has_handwriting { 1 } else { 0 }
        ],
    )
    .map_err(|e| format!("OCR_RESULT_INSERT_FAILED: {e}"))?;
    Ok(page_id)
}

fn bbox_to_xywh(value: &serde_json::Value) -> (f64, f64, f64, f64) {
    let arr = value.as_array().cloned().unwrap_or_default();
    let x1 = arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y1 = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let x2 = arr.get(2).and_then(|v| v.as_f64()).unwrap_or(x1);
    let y2 = arr.get(3).and_then(|v| v.as_f64()).unwrap_or(y1);
    (x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
}

fn normalize_block_type(value: &str) -> String {
    match value {
        "text"
        | "title"
        | "header"
        | "footer"
        | "stamp"
        | "signature"
        | "but_luc"
        | "document_number"
        | "date"
        | "agency"
        | "person_name"
        | "decision_type"
        | "table"
        | "unknown"
        | "low_confidence"
        | "possible_handwriting"
        | "unreadable_stamp"
        | "unreadable_signature"
        | "blurred_area" => value.to_string(),
        "handwriting_candidate" => "possible_handwriting".to_string(),
        "warning" => "unknown".to_string(),
        _ => "unknown".to_string(),
    }
}

fn insert_layout_block(
    conn: &rusqlite::Connection,
    document_id: &str,
    page_id: &str,
    page_number: i32,
    engine: &str,
    source: &str,
    raw: &serde_json::Value,
    fallback_order: i32,
) -> Result<(), String> {
    let block_id = generate_layout_block_id();
    let block_type = normalize_block_type(
        raw.get("block_type")
            .or_else(|| raw.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown"),
    );
    let text = raw
        .get("text")
        .or_else(|| raw.get("label"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let confidence = raw
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let reading_order = raw
        .get("reading_order")
        .and_then(|v| v.as_i64())
        .unwrap_or(fallback_order as i64) as i32;
    let (x, y, width, height) = bbox_to_xywh(raw.get("bbox").unwrap_or(&serde_json::Value::Null));
    let page_width = raw.get("page_width").and_then(|v| v.as_f64());
    let page_height = raw.get("page_height").and_then(|v| v.as_f64());
    let normalized_text = raw
        .get("normalized_text")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let unicode_form = raw
        .get("unicode_form")
        .and_then(|v| v.as_str())
        .unwrap_or("NFC");

    conn.execute(
        "INSERT INTO page_layout_blocks (
            id, document_id, page_id, page_number, block_type, text,
            normalized_text, unicode_form,
            x, y, width, height, page_width, page_height, confidence,
            reading_order, engine, source, raw_json, created_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6,
            ?7, ?8,
            ?9, ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            block_id,
            document_id,
            page_id,
            page_number,
            block_type,
            text,
            normalized_text,
            unicode_form,
            x,
            y,
            width,
            height,
            page_width,
            page_height,
            confidence,
            reading_order,
            engine,
            source,
            raw.to_string()
        ],
    )
    .map_err(|e| format!("LAYOUT_BLOCK_INSERT_FAILED: {e}"))?;
    Ok(())
}

fn insert_extracted_field(
    conn: &rusqlite::Connection,
    document_id: &str,
    page_number: i32,
    raw: &serde_json::Value,
) -> Result<(), String> {
    let field_id = generate_extracted_field_id();
    let field_name = raw
        .get("field_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let field_value = raw
        .get("field_value")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim();
    if field_value.is_empty() {
        return Ok(());
    }
    let (x, y, width, height) = bbox_to_xywh(raw.get("bbox").unwrap_or(&serde_json::Value::Null));
    let confidence = raw
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let source = raw
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("ocr_rule");

    conn.execute(
        "INSERT INTO document_extracted_fields (
            id, document_id, field_name, field_value, page_number,
            x, y, width, height, confidence, source, raw_json, created_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            field_id,
            document_id,
            field_name,
            field_value,
            page_number,
            x,
            y,
            width,
            height,
            confidence,
            source,
            raw.to_string()
        ],
    )
    .map_err(|e| format!("EXTRACTED_FIELD_INSERT_FAILED: {e}"))?;
    Ok(())
}

fn persist_page_analysis(
    conn: &rusqlite::Connection,
    document_id: &str,
    page_id: &str,
    page_number: i32,
    engine: &str,
    blocks_json: &str,
    regions_json: &str,
    handwriting_regions_json: &str,
    extracted_fields_json: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM page_layout_blocks WHERE document_id = ?1 AND page_number = ?2",
        params![document_id, page_number],
    )
    .map_err(|e| format!("LAYOUT_BLOCK_CLEAR_FAILED: {e}"))?;
    conn.execute(
        "DELETE FROM document_extracted_fields WHERE document_id = ?1 AND page_number = ?2",
        params![document_id, page_number],
    )
    .map_err(|e| format!("EXTRACTED_FIELD_CLEAR_FAILED: {e}"))?;

    for (source, json_text) in [
        ("ocr_block", blocks_json),
        ("opencv_region", regions_json),
        ("ocr_handwriting", handwriting_regions_json),
    ] {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_text) {
            if let Some(items) = value.as_array() {
                for (idx, item) in items.iter().enumerate() {
                    insert_layout_block(
                        conn,
                        document_id,
                        page_id,
                        page_number,
                        engine,
                        source,
                        item,
                        idx as i32,
                    )?;
                }
            }
        }
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(extracted_fields_json) {
        if let Some(items) = value.as_array() {
            for item in items {
                insert_extracted_field(conn, document_id, page_number, item)?;
            }
        }
    }
    Ok(())
}

fn non_empty_or(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn fold_vietnamese(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| match ch {
            'à' | 'á' | 'ạ' | 'ả' | 'ã' | 'â' | 'ầ' | 'ấ' | 'ậ' | 'ẩ' | 'ẫ' | 'ă' | 'ằ' | 'ắ'
            | 'ặ' | 'ẳ' | 'ẵ' => 'a',
            'è' | 'é' | 'ẹ' | 'ẻ' | 'ẽ' | 'ê' | 'ề' | 'ế' | 'ệ' | 'ể' | 'ễ' => {
                'e'
            }
            'ì' | 'í' | 'ị' | 'ỉ' | 'ĩ' => 'i',
            'ò' | 'ó' | 'ọ' | 'ỏ' | 'õ' | 'ô' | 'ồ' | 'ố' | 'ộ' | 'ổ' | 'ỗ' | 'ơ' | 'ờ' | 'ớ'
            | 'ợ' | 'ở' | 'ỡ' => 'o',
            'ù' | 'ú' | 'ụ' | 'ủ' | 'ũ' | 'ư' | 'ừ' | 'ứ' | 'ự' | 'ử' | 'ữ' => {
                'u'
            }
            'ỳ' | 'ý' | 'ỵ' | 'ỷ' | 'ỹ' => 'y',
            'đ' => 'd',
            _ => ch,
        })
        .collect()
}

fn count_document_pages(path: &Path) -> i32 {
    let ext = path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_lowercase();
    if ext == "pdf" {
        lopdf::Document::load(path)
            .map(|doc| doc.get_pages().len() as i32)
            .unwrap_or(0)
    } else {
        1
    }
}

fn normalize_base_name(file_name: &str) -> String {
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or(file_name)
        .to_string();
    stem.replace('_', " ").replace('-', " ").trim().to_string()
}

fn classify_document_type_from_name(file_name: &str) -> String {
    let n = file_name.to_lowercase();
    let folded = fold_vietnamese(file_name);
    if n.contains("to khai")
        || n.contains("lời khai")
        || n.contains("loi khai")
        || folded.contains("loi khai")
    {
        "to_khai".to_string()
    } else if n.contains("bien ban") || n.contains("biên bản") || folded.contains("bien ban") {
        "bien_ban".to_string()
    } else if n.contains("quyet dinh")
        || n.contains("quyết định")
        || n.contains("qd")
        || folded.contains("quyet dinh")
    {
        "quyet_dinh".to_string()
    } else if n.contains("ket luan") || n.contains("kết luận") || folded.contains("ket luan") {
        "ket_luan".to_string()
    } else if n.contains("giay") || n.contains("phieu") {
        "phieu".to_string()
    } else {
        "khong_xac_dinh".to_string()
    }
}

fn build_display_name(file_name: &str, document_type: &str) -> String {
    let base = normalize_base_name(file_name);
    if base.is_empty() {
        format!("Tài liệu {}", document_type)
    } else {
        base
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

fn sanitize_filename_part(value: &str, fallback: &str) -> String {
    let raw = if value.trim().is_empty() {
        fallback
    } else {
        value.trim()
    };
    let mut out = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '_'
            } else {
                '_'
            }
        })
        .collect::<String>();
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

fn get_document_by_id(
    conn: &rusqlite::Connection,
    doc_id: &str,
) -> Result<DocumentSummary, String> {
    conn.query_row(
        "SELECT
            document_id,
            case_id,
            display_name,
            original_filename,
            file_path,
            document_type,
            page_count,
            status,
            created_at
         FROM documents
         WHERE document_id = ?1",
        [doc_id],
        |row| {
            let file_path: String = row.get(4)?;
            let file_missing = !Path::new(&file_path).exists();
            Ok(DocumentSummary {
                document_id: row.get(0)?,
                case_id: row.get(1)?,
                display_name: row.get(2)?,
                original_filename: row.get(3)?,
                file_path,
                document_type: row.get(5)?,
                page_count: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                file_missing,
            })
        },
    )
    .map_err(|e| format!("DOC_QUERY_FAILED: {e}"))
}

#[tauri::command]
pub fn list_documents(
    db: State<'_, DbState>,
    case_id: Option<String>,
) -> Result<Vec<DocumentSummary>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut documents: Vec<DocumentSummary> = Vec::new();

    if let Some(case_id_filter) = case_id {
        let mut stmt = conn
            .prepare(
                "SELECT
                    document_id,
                    case_id,
                    display_name,
                    original_filename,
                    file_path,
                    document_type,
                    page_count,
                    status,
                    created_at
                 FROM documents
                 WHERE case_id = ?1
                 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Prepare failed: {e}"))?;

        let rows = stmt
            .query_map([case_id_filter], |row| {
                let file_path: String = row.get(4)?;
                let file_missing = !Path::new(&file_path).exists();
                Ok(DocumentSummary {
                    document_id: row.get(0)?,
                    case_id: row.get(1)?,
                    display_name: row.get(2)?,
                    original_filename: row.get(3)?,
                    file_path,
                    document_type: row.get(5)?,
                    page_count: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    file_missing,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            documents.push(row.map_err(|e| format!("Row read failed: {e}"))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT
                    document_id,
                    case_id,
                    display_name,
                    original_filename,
                    file_path,
                    document_type,
                    page_count,
                    status,
                    created_at
                 FROM documents
                 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Prepare failed: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                let file_path: String = row.get(4)?;
                let file_missing = !Path::new(&file_path).exists();
                Ok(DocumentSummary {
                    document_id: row.get(0)?,
                    case_id: row.get(1)?,
                    display_name: row.get(2)?,
                    original_filename: row.get(3)?,
                    file_path,
                    document_type: row.get(5)?,
                    page_count: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    file_missing,
                })
            })
            .map_err(|e| format!("Query failed: {e}"))?;

        for row in rows {
            documents.push(row.map_err(|e| format!("Row read failed: {e}"))?);
        }
    }

    Ok(documents)
}

#[tauri::command]
pub fn get_document(
    db: State<'_, DbState>,
    document_id: String,
) -> Result<DocumentSummary, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    get_document_by_id(&conn, &document_id)
}

#[tauri::command]
pub fn update_document_status(
    db: State<'_, DbState>,
    document_id: String,
    status: String,
    reason_note: Option<String>,
) -> Result<DocumentSummary, String> {
    let valid = matches!(
        status.as_str(),
        "pending" | "processed" | "reviewed" | "error"
    );
    if !valid {
        return Err(format!("DOC_STATUS_INVALID: {status}"));
    }

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let before_status: String = conn
        .query_row(
            "SELECT status FROM documents WHERE document_id = ?1",
            [document_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("DOC_STATUS_QUERY_FAILED: {e}"))?;

    conn.execute(
        "UPDATE documents
         SET status = ?1,
             needs_review = CASE WHEN ?1 = 'reviewed' THEN 0 ELSE 1 END,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE document_id = ?2",
        params![&status, &document_id],
    )
    .map_err(|e| format!("DOC_STATUS_UPDATE_FAILED: {e}"))?;

    let action_type = if status == "reviewed" {
        "approve"
    } else if status == "error" {
        "reject"
    } else {
        "update"
    };
    let approval_status = if status == "reviewed" {
        "approved"
    } else if status == "error" {
        "rejected"
    } else {
        "pending_review"
    };
    let audit_id = generate_audit_event_id();
    let _ = conn.execute(
        "INSERT INTO audit_events (
            audit_event_id,
            object_type,
            object_id,
            field_name,
            action_type,
            before_value,
            after_value,
            actor_type,
            actor_id,
            reason_code,
            reason_note,
            approval_status,
            created_at
         ) VALUES (
            ?1, 'document', ?2, 'status', ?3, ?4, ?5,
            'user', 'desktop-ui', 'review_queue_action', ?6, ?7,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            audit_id,
            &document_id,
            action_type,
            before_status,
            &status,
            reason_note.unwrap_or_default(),
            approval_status,
        ],
    );

    get_document_by_id(&conn, &document_id)
}

#[tauri::command]
pub fn import_document(
    db: State<'_, DbState>,
    input: ImportDocumentInput,
) -> Result<DocumentSummary, String> {
    let source_path = Path::new(&input.file_path);
    if !source_path.exists() {
        return Err("DOC_IMPORT_FILE_NOT_FOUND".to_string());
    }
    if !source_path.is_file() {
        return Err("DOC_IMPORT_PATH_NOT_FILE".to_string());
    }
    if !is_supported_import_file(source_path) {
        return Err(
            "DOC_IMPORT_UNSUPPORTED_FILE: chỉ nhận PDF, ảnh scan hoặc file Office".to_string(),
        );
    }

    let file_name = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("tai_lieu")
        .to_string();

    let document_type = non_empty_or(
        input.document_type.as_deref().unwrap_or_default(),
        &classify_document_type_from_name(&file_name),
    );
    let display_name = build_display_name(&file_name, &document_type);
    let summary_short = build_summary_short(&display_name, &document_type);
    let file_size = std::fs::metadata(source_path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);
    let page_count = count_document_pages(source_path).max(1);

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let case_id = input.case_id.clone();

    let case_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cases WHERE case_id = ?1",
            [&input.case_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("DOC_IMPORT_CASE_CHECK_FAILED: {e}"))?;
    if case_exists == 0 {
        return Err("DOC_IMPORT_CASE_NOT_FOUND".to_string());
    }
    let case_code: String = conn
        .query_row(
            "SELECT case_code FROM cases WHERE case_id = ?1",
            [&input.case_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("DOC_IMPORT_CASE_CODE_QUERY_FAILED: {e}"))?;

    let document_id = generate_document_id();
    let is_office = source_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "rtf"))
        .unwrap_or(false);

    if is_office {
        warn!("import_office_file_warning: path={} OCR support pending (Phase 2)", source_path.display());
    }

    let managed_path = storage::copy_to_originals(source_path, &case_code, &document_id, &file_name)?;
    let managed_file_path = managed_path.to_string_lossy().to_string();
    let summary_detail = build_summary_detail(&display_name, &document_type, &managed_file_path);
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
            case_id.clone(),
            file_name,
            managed_file_path,
            file_size,
            page_count,
            display_name,
            document_type,
            summary_short,
            summary_detail,
        ],
    )
    .map_err(|e| format!("DOC_IMPORT_INSERT_FAILED: {e}"))?;

    for page_index in 1..=page_count {
        let page_id = generate_document_id().replace("doc-", "page-");
        conn.execute(
            "INSERT INTO pages (page_id, document_id, page_index, image_path, created_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![
                page_id,
                document_id,
                page_index,
                if page_count == 1
                    && is_image_import_file(&managed_path)
                {
                    Some(managed_path.to_string_lossy().to_string())
                } else {
                    None
                },
            ],
        )
        .map_err(|e| format!("DOC_IMPORT_INSERT_PAGE_FAILED: {e}"))?;

        let ocr_result_id = generate_ocr_result_id();
        conn.execute(
            "INSERT INTO ocr_results (
                ocr_result_id, page_id, engine, raw_text, confidence,
                transcription_state, created_at
             ) VALUES (
                ?1, ?2, 'pending', '', 0.0,
                'candidate_only', strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             )",
            params![ocr_result_id, page_id],
        )
        .map_err(|e| format!("DOC_IMPORT_INSERT_OCR_PLACEHOLDER_FAILED: {e}"))?;
    }

    let _ = conn.execute(
        "INSERT INTO fts_documents (rowid, document_id, display_name, document_title, summary_short, summary_detail)
         SELECT rowid, document_id, display_name, document_title, summary_short, summary_detail
         FROM documents WHERE document_id = ?1",
        params![document_id.clone()],
    );

    conn.execute(
        "UPDATE cases
         SET document_count = (
                SELECT COUNT(*)
                FROM documents
                WHERE documents.case_id = cases.case_id
             ),
             total_pages = (
                SELECT COALESCE(SUM(page_count), 0)
                FROM documents
                WHERE documents.case_id = cases.case_id
             ),
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE case_id = ?1",
        params![case_id],
    )
    .map_err(|e| format!("DOC_IMPORT_CASE_COUNTER_UPDATE_FAILED: {e}"))?;

    get_document_by_id(&conn, &document_id)
}

#[tauri::command]
pub fn enrich_document_metadata(
    db: State<'_, DbState>,
    document_id: String,
) -> Result<DocumentSummary, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let (original_filename, file_path): (String, String) = conn
        .query_row(
            "SELECT original_filename, file_path FROM documents WHERE document_id = ?1",
            [document_id.clone()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("DOC_ENRICH_QUERY_FAILED: {e}"))?;

    let document_type = classify_document_type_from_name(&original_filename);
    let display_name = build_display_name(&original_filename, &document_type);
    let summary_short = build_summary_short(&display_name, &document_type);
    let summary_detail = build_summary_detail(&display_name, &document_type, &file_path);

    conn.execute(
        "UPDATE documents
         SET display_name = ?1,
             document_title = ?1,
             document_type = ?2,
             summary_short = ?3,
             summary_detail = ?4,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE document_id = ?5",
        params![
            display_name,
            document_type,
            summary_short,
            summary_detail,
            document_id,
        ],
    )
    .map_err(|e| format!("DOC_ENRICH_UPDATE_FAILED: {e}"))?;

    get_document_by_id(&conn, &document_id)
}

#[tauri::command]
pub fn bulk_enrich_documents(
    db: State<'_, DbState>,
    case_id: Option<String>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let mut ids: Vec<String> = Vec::new();
    if let Some(case_filter) = case_id {
        let mut stmt = conn
            .prepare("SELECT document_id FROM documents WHERE case_id = ?1")
            .map_err(|e| format!("DOC_BULK_PREPARE_FAILED: {e}"))?;
        let rows = stmt
            .query_map([case_filter], |row| row.get::<_, String>(0))
            .map_err(|e| format!("DOC_BULK_QUERY_FAILED: {e}"))?;
        for row in rows {
            ids.push(row.map_err(|e| format!("DOC_BULK_ROW_FAILED: {e}"))?);
        }
    } else {
        let mut stmt = conn
            .prepare("SELECT document_id FROM documents")
            .map_err(|e| format!("DOC_BULK_PREPARE_FAILED: {e}"))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("DOC_BULK_QUERY_FAILED: {e}"))?;
        for row in rows {
            ids.push(row.map_err(|e| format!("DOC_BULK_ROW_FAILED: {e}"))?);
        }
    }

    let mut updated = 0_i64;
    for id in ids {
        let (original_filename, file_path): (String, String) = conn
            .query_row(
                "SELECT original_filename, file_path FROM documents WHERE document_id = ?1",
                [id.clone()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| format!("DOC_BULK_QUERY_ONE_FAILED: {e}"))?;

        let document_type = classify_document_type_from_name(&original_filename);
        let display_name = build_display_name(&original_filename, &document_type);
        let summary_short = build_summary_short(&display_name, &document_type);
        let summary_detail = build_summary_detail(&display_name, &document_type, &file_path);

        let changed = conn
            .execute(
                "UPDATE documents
                 SET display_name = ?1,
                     document_title = ?1,
                     document_type = ?2,
                     summary_short = ?3,
                     summary_detail = ?4,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE document_id = ?5",
                params![
                    display_name,
                    document_type,
                    summary_short,
                    summary_detail,
                    id
                ],
            )
            .map_err(|e| format!("DOC_BULK_UPDATE_FAILED: {e}"))?;
        updated += changed as i64;
    }

    Ok(updated)
}

#[tauri::command]
pub fn get_document_groups(
    db: State<'_, DbState>,
    case_id: Option<String>,
) -> Result<Vec<DocumentGroup>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut groups: Vec<DocumentGroup> = Vec::new();

    if let Some(case_filter) = case_id {
        let mut stmt = conn
            .prepare(
                "SELECT document_type, COUNT(*)
                 FROM documents
                 WHERE case_id = ?1
                 GROUP BY document_type
                 ORDER BY COUNT(*) DESC, document_type ASC",
            )
            .map_err(|e| format!("DOC_GROUP_PREPARE_FAILED: {e}"))?;
        let rows = stmt
            .query_map([case_filter], |row| {
                Ok(DocumentGroup {
                    group_key: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| format!("DOC_GROUP_QUERY_FAILED: {e}"))?;
        for row in rows {
            groups.push(row.map_err(|e| format!("DOC_GROUP_ROW_FAILED: {e}"))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT document_type, COUNT(*)
                 FROM documents
                 GROUP BY document_type
                 ORDER BY COUNT(*) DESC, document_type ASC",
            )
            .map_err(|e| format!("DOC_GROUP_PREPARE_FAILED: {e}"))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(DocumentGroup {
                    group_key: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| format!("DOC_GROUP_QUERY_FAILED: {e}"))?;
        for row in rows {
            groups.push(row.map_err(|e| format!("DOC_GROUP_ROW_FAILED: {e}"))?);
        }
    }

    Ok(groups)
}

#[tauri::command]
pub fn rebuild_text_index(db: State<'_, DbState>) -> Result<IndexRebuildResult, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    conn.execute(
        "INSERT INTO fts_documents(fts_documents) VALUES('rebuild')",
        [],
    )
    .map_err(|e| format!("FTS_REBUILD_DOCUMENTS_FAILED: {e}"))?;
    conn.execute("INSERT INTO fts_pages(fts_pages) VALUES('rebuild')", [])
        .map_err(|e| format!("FTS_REBUILD_PAGES_FAILED: {e}"))?;
    conn.execute(
        "INSERT INTO fts_entities(fts_entities) VALUES('rebuild')",
        [],
    )
    .map_err(|e| format!("FTS_REBUILD_ENTITIES_FAILED: {e}"))?;
    conn.execute(
        "INSERT INTO fts_layout_blocks(fts_layout_blocks) VALUES('rebuild')",
        [],
    )
    .map_err(|e| format!("FTS_REBUILD_LAYOUT_BLOCKS_FAILED: {e}"))?;
    conn.execute(
        "INSERT INTO fts_extracted_fields(fts_extracted_fields) VALUES('rebuild')",
        [],
    )
    .map_err(|e| format!("FTS_REBUILD_EXTRACTED_FIELDS_FAILED: {e}"))?;

    let indexed_documents: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
        .map_err(|e| format!("FTS_COUNT_DOCUMENTS_FAILED: {e}"))?;
    let indexed_pages: i64 = conn
        .query_row("SELECT COUNT(*) FROM pages", [], |row| row.get(0))
        .map_err(|e| format!("FTS_COUNT_PAGES_FAILED: {e}"))?;
    let indexed_entities: i64 = conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .map_err(|e| format!("FTS_COUNT_ENTITIES_FAILED: {e}"))?;

    Ok(IndexRebuildResult {
        indexed_documents,
        indexed_pages,
        indexed_entities,
    })
}

#[tauri::command]
pub fn get_page_ocr(
    db: State<'_, DbState>,
    document_id: String,
    page_index: i32,
) -> Result<PageOcrResult, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT
            p.page_id,
            COALESCE(o.raw_text, p.ocr_text, ''),
            COALESCE(NULLIF(p.ocr_formatted_text, ''), p.ocr_text, ''),
            COALESCE(p.ocr_layout_blocks, '[]'),
            COALESCE(o.confidence, p.confidence, 0.0),
            COALESCE(o.engine, 'pending'),
            p.but_luc,
            p.transcription_state
         FROM pages p
         LEFT JOIN ocr_results o ON o.page_id = p.page_id
         WHERE p.document_id = ?1 AND p.page_index = ?2
         ORDER BY o.created_at DESC
         LIMIT 1",
        )
        .map_err(|e| format!("PAGE_OCR_PREPARE_FAILED: {e}"))?;

    let mut rows = stmt
        .query(params![document_id.clone(), page_index])
        .map_err(|e| format!("PAGE_OCR_QUERY_FAILED: {e}"))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| format!("PAGE_OCR_ROW_NEXT_FAILED: {e}"))?
    {
        return Ok(PageOcrResult {
            page_id: row
                .get(0)
                .map_err(|e| format!("PAGE_OCR_GET_PAGE_ID_FAILED: {e}"))?,
            document_id,
            page_index,
            ocr_text: row
                .get(1)
                .map_err(|e| format!("PAGE_OCR_GET_TEXT_FAILED: {e}"))?,
            ocr_formatted_text: row
                .get(2)
                .map_err(|e| format!("PAGE_OCR_GET_FORMATTED_TEXT_FAILED: {e}"))?,
            ocr_layout_blocks: row
                .get(3)
                .map_err(|e| format!("PAGE_OCR_GET_LAYOUT_BLOCKS_FAILED: {e}"))?,
            confidence: row
                .get(4)
                .map_err(|e| format!("PAGE_OCR_GET_CONFIDENCE_FAILED: {e}"))?,
            engine: row
                .get(5)
                .map_err(|e| format!("PAGE_OCR_GET_ENGINE_FAILED: {e}"))?,
            but_luc: row.get(6).ok(),
            transcription_state: row.get(7).unwrap_or_else(|_| "pending".to_string()),
        });
    }

    Ok(PageOcrResult {
        page_id: String::new(),
        document_id,
        page_index,
        ocr_text: String::new(),
        ocr_formatted_text: String::new(),
        ocr_layout_blocks: "[]".to_string(),
        confidence: 0.0,
        engine: "pending".to_string(),
        but_luc: None,
        transcription_state: "pending".to_string(),
    })
}

#[tauri::command]
pub fn update_page_ocr(
    db: State<'_, DbState>,
    document_id: String,
    page_index: i32,
    ocr_text: String,
) -> Result<PageOcrResult, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let page_id: String = conn
        .query_row(
            "SELECT page_id FROM pages WHERE document_id = ?1 AND page_index = ?2",
            params![document_id.clone(), page_index],
            |row| row.get(0),
        )
        .map_err(|e| format!("PAGE_OCR_UPDATE_PAGE_NOT_FOUND: {e}"))?;

    conn.execute(
        "UPDATE pages
         SET ocr_text = ?1,
             ocr_formatted_text = ?1,
             ocr_preview = substr(?1, 1, 500),
             transcription_state = 'approved_manual'
         WHERE page_id = ?2",
        params![ocr_text, page_id],
    )
    .map_err(|e| format!("PAGE_OCR_UPDATE_PAGE_FAILED: {e}"))?;

    let existing_ocr: Option<String> = conn
        .query_row(
            "SELECT ocr_result_id
             FROM ocr_results
             WHERE page_id = ?1
             ORDER BY created_at DESC
             LIMIT 1",
            params![page_id.clone()],
            |row| row.get(0),
        )
        .ok();

    if let Some(ocr_result_id) = existing_ocr {
        conn.execute(
            "UPDATE ocr_results
             SET raw_text = ?1,
                 confidence = 1.0,
                 engine = 'manual_edit',
                 transcription_state = 'approved_manual'
             WHERE ocr_result_id = ?2",
            params![ocr_text, ocr_result_id],
        )
        .map_err(|e| format!("PAGE_OCR_UPDATE_RESULT_FAILED: {e}"))?;
    } else {
        let ocr_result_id = generate_ocr_result_id();
        conn.execute(
            "INSERT INTO ocr_results (
                ocr_result_id,
                page_id,
                engine,
                raw_text,
                confidence,
                transcription_state,
                created_at
             ) VALUES (
                ?1, ?2, 'manual_edit', ?3, 1.0, 'approved_manual',
                strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             )",
            params![ocr_result_id, page_id, ocr_text],
        )
        .map_err(|e| format!("PAGE_OCR_INSERT_RESULT_FAILED: {e}"))?;
    }

    let _ = conn.execute("INSERT INTO fts_pages(fts_pages) VALUES('rebuild')", []);

    drop(conn);
    get_page_ocr(db, document_id, page_index)
}

#[tauri::command]
pub fn run_ocr_for_document(
    app: AppHandle,
    db: State<'_, DbState>,
    document_id: String,
) -> Result<OcrRunResult, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    run_ocr_for_document_with_conn(&app, &conn, &document_id)
}

pub(crate) fn run_ocr_for_document_with_conn(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    document_id: &str,
) -> Result<OcrRunResult, String> {
    run_ocr_for_document_scope_with_conn(app, conn, document_id, None, 300, "auto")
}

fn run_ocr_for_document_scope_with_conn(
    _app: &AppHandle,
    conn: &rusqlite::Connection,
    document_id: &str,
    page_filter: Option<i32>,
    dpi: i32,
    mode: &str,
) -> Result<OcrRunResult, String> {
    let (file_path, page_count): (String, i32) = conn
        .query_row(
            "SELECT file_path, page_count FROM documents WHERE document_id = ?1",
            params![document_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("OCR_DOC_NOT_FOUND: {e}"))?;

    conn.execute(
        "UPDATE documents
         SET status = 'pending',
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE document_id = ?1",
        params![document_id],
    )
    .map_err(|e| format!("OCR_STATUS_PREP_FAILED: {e}"))?;

    let source = PathBuf::from(&file_path);
    if !source.exists() {
        return Err("OCR_SOURCE_FILE_NOT_FOUND".to_string());
    }
    if !is_ocr_source_file(&source) {
        return Err("OCR_UNSUPPORTED_FILE_TYPE: OCR chỉ chạy với PDF hoặc ảnh scan; file Office được lưu gốc nhưng không OCR trực tiếp".to_string());
    }

    let mut image_pages = Vec::<(i32, String, Option<String>)>::new();
    let ext = source
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_lowercase();

    if ext == "pdf" {
        let output_dir = storage::ocr_pages_dir(document_id)?;
        std::fs::create_dir_all(&output_dir).map_err(|e| format!("OCR_OUTPUT_DIR_FAILED: {e}"))?;
        let mut args = vec![
            "-m".to_string(),
            "ocr.pdf_to_images".to_string(),
            "--pdf".to_string(),
            file_path.clone(),
            "--output-dir".to_string(),
            output_dir.to_string_lossy().to_string(),
            "--dpi".to_string(),
            dpi.to_string(),
        ];
        if let Some(page) = page_filter {
            args.push("--page".to_string());
            args.push(page.to_string());
        }
        args.push("--json".to_string());
        let extracted = run_python_json(&args)?;
        if let Some(pages) = extracted.get("pages").and_then(|v| v.as_array()) {
            for page in pages {
                let page_num = page.get("page_num").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                if let Some(path) = page.get("path").and_then(|v| v.as_str()) {
                    let text_layer = page
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(|v| v.trim().to_string())
                        .filter(|v| !v.is_empty());
                    image_pages.push((page_num, path.to_string(), text_layer));
                }
            }
        }
    } else {
        image_pages.push((1, file_path.clone(), None));
    }

    if image_pages.is_empty() && page_count > 0 {
        conn.execute(
            "UPDATE documents SET status = 'error', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE document_id = ?1",
            params![document_id],
        )
        .ok();
        return Err("OCR_NO_PAGES_EXTRACTED".to_string());
    }

    let mut processed = 0_i64;
    let mut failed = 0_i64;
    let mut handwritten_pages = 0_i64;
    let mut conf_total = 0.0_f64;
    let mut page_results = Vec::<PageOcrPayload>::new();
    let mut page_errors = Vec::<String>::new();
    for (page_num, image_path, text_layer) in image_pages {
        if let Some(text) = text_layer {
            page_results.push(PageOcrPayload {
                page_num,
                image_path: Some(image_path),
                text,
                formatted_text: String::new(),
                layout_blocks_json: "[]".to_string(),
                confidence: 1.0,
                engine: "pymupdf_text_layer".to_string(),
                has_handwriting: false,
                regions_json: "[]".to_string(),
                handwriting_regions_json: "[]".to_string(),
                warning: None,
                extracted_fields_json: "[]".to_string(),
            });
            processed += 1;
            conf_total += 1.0;
            continue;
        }
        let args = vec![
            "-m".to_string(),
            "ocr.ocr_pipeline".to_string(),
            "--image".to_string(),
            image_path.clone(),
            "--lang".to_string(),
            "vie".to_string(),
            "--mode".to_string(),
            mode.to_string(),
            "--json".to_string(),
        ];
        match run_python_json(&args) {
            Ok(value) => {
                let text = value
                    .get("ocr_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                if text.trim().is_empty() {
                    failed += 1;
                    page_errors.push(format!("trang {page_num}: OCR_EMPTY_TEXT"));
                    continue;
                }
                let confidence = value
                    .get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let engine = value
                    .get("engine")
                    .and_then(|v| v.as_str())
                    .unwrap_or("python_ocr")
                    .to_string();
                let has_handwriting = value
                    .get("has_handwriting")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let regions_json = value
                    .get("regions")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "[]".to_string());
                let handwriting_regions_json = value
                    .get("handwriting_regions")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "[]".to_string());
                let ocr_formatted_text = value
                    .get("ocr_formatted_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let layout_blocks_json = value
                    .get("blocks")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "[]".to_string());
                let warning = value
                    .get("warnings")
                    .and_then(|v| v.as_array())
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str())
                            .collect::<Vec<_>>()
                            .join(" | ")
                    })
                    .filter(|v| !v.is_empty());
                let extracted_fields_json = value
                    .get("extracted_fields")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "[]".to_string());
                if has_handwriting {
                    handwritten_pages += 1;
                }
                page_results.push(PageOcrPayload {
                    page_num,
                    image_path: Some(image_path),
                    text,
                    formatted_text: ocr_formatted_text,
                    layout_blocks_json,
                    confidence,
                    engine,
                    has_handwriting,
                    regions_json,
                    handwriting_regions_json,
                    warning,
                    extracted_fields_json,
                });
                processed += 1;
                conf_total += confidence;
            }
            Err(e) => {
                failed += 1;
                page_errors.push(format!("trang {page_num}: {e}"));
            }
        }
    }

    let average = if processed > 0 {
        conf_total / processed as f64
    } else {
        0.0
    };
    for item in &page_results {
        let page_id = upsert_ocr_text(
            conn,
            document_id,
            item.page_num,
            item.image_path.as_deref(),
            &item.text,
            if item.formatted_text.trim().is_empty() {
                &item.text
            } else {
                &item.formatted_text
            },
            &item.layout_blocks_json,
            item.confidence,
            &item.engine,
            item.has_handwriting,
            &item.regions_json,
            &item.handwriting_regions_json,
        )?;
        persist_page_analysis(
            conn,
            document_id,
            &page_id,
            item.page_num,
            &item.engine,
            &item.layout_blocks_json,
            &item.regions_json,
            &item.handwriting_regions_json,
            &item.extracted_fields_json,
        )?;
    }
    let status = if processed > 0 && failed == 0 {
        "processed"
    } else if processed > 0 {
        "processed"
    } else {
        "error"
    };
    conn.execute(
        "UPDATE documents
         SET status = ?1,
             ocr_confidence_avg = ?2,
             needs_review = CASE WHEN ?3 > 0 OR ?5 > 0 OR ?2 < 0.65 THEN 1 ELSE needs_review END,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE document_id = ?4",
        params![status, average, failed, document_id, handwritten_pages],
    )
    .map_err(|e| format!("OCR_DOC_UPDATE_FAILED: {e}"))?;
    let _ = conn.execute("INSERT INTO fts_pages(fts_pages) VALUES('rebuild')", []);
    let _ = conn.execute(
        "INSERT INTO fts_layout_blocks(fts_layout_blocks) VALUES('rebuild')",
        [],
    );
    let _ = conn.execute(
        "INSERT INTO fts_extracted_fields(fts_extracted_fields) VALUES('rebuild')",
        [],
    );

    Ok(OcrRunResult {
        document_id: document_id.to_string(),
        processed_pages: processed,
        failed_pages: failed,
        handwritten_pages,
        average_confidence: average,
        status: status.to_string(),
        message: if processed > 0 {
            let warning_count = page_results
                .iter()
                .filter(|item| item.warning.is_some())
                .count();
            format!(
                "OCR hoàn tất: {processed} trang, {failed} lỗi, {handwritten_pages} trang nghi chữ viết tay, {warning_count} cảnh báo."
            )
        } else {
            let detail = page_errors
                .first()
                .cloned()
                .unwrap_or_else(|| "Kiểm tra Python/RapidOCR/PaddleOCR/PyMuPDF.".to_string());
            format!("OCR thất bại. {detail}")
        },
    })
}

#[tauri::command]
pub fn rescan_document_ocr(
    app: AppHandle,
    db: State<'_, DbState>,
    document_id: String,
    page_number: Option<i32>,
    options: RescanOcrOptions,
) -> Result<OcrRunResult, String> {
    let quality = options.quality.unwrap_or_else(|| "fast".to_string());
    let dpi = match quality.as_str() {
        "high" | "accurate" | "ocr_chinh_xac_cao" => 400,
        "max" | "600" => 600,
        _ => 300,
    };
    let mode = options.mode.unwrap_or_else(|| "auto".to_string());
    let scope = options.scope.unwrap_or_else(|| {
        if page_number.is_some() {
            "current".to_string()
        } else {
            "all".to_string()
        }
    });
    let page_filter = if scope == "current" {
        page_number
    } else {
        None
    };

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let result =
        run_ocr_for_document_scope_with_conn(&app, &conn, &document_id, page_filter, dpi, &mode)?;

    if options.rebuild_index.unwrap_or(true) {
        let _ = conn.execute(
            "INSERT INTO fts_documents(fts_documents) VALUES('rebuild')",
            [],
        );
        let _ = conn.execute("INSERT INTO fts_pages(fts_pages) VALUES('rebuild')", []);
        let _ = conn.execute(
            "INSERT INTO fts_layout_blocks(fts_layout_blocks) VALUES('rebuild')",
            [],
        );
        let _ = conn.execute(
            "INSERT INTO fts_extracted_fields(fts_extracted_fields) VALUES('rebuild')",
            [],
        );
    }
    Ok(result)
}

#[tauri::command]
pub fn list_page_layout_blocks(
    db: State<'_, DbState>,
    document_id: String,
    page_number: i32,
) -> Result<Vec<PageLayoutBlock>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT
                id, document_id, page_id, page_number, block_type, COALESCE(text, ''),
                x, y, width, height, page_width, page_height, confidence,
                reading_order, engine, source, reviewed
             FROM page_layout_blocks
             WHERE document_id = ?1 AND page_number = ?2
             ORDER BY reading_order ASC, y ASC, x ASC",
        )
        .map_err(|e| format!("LAYOUT_BLOCK_LIST_PREPARE_FAILED: {e}"))?;
    let rows = stmt
        .query_map(params![document_id, page_number], |row| {
            let reviewed: i64 = row.get(16)?;
            Ok(PageLayoutBlock {
                id: row.get(0)?,
                document_id: row.get(1)?,
                page_id: row.get(2)?,
                page_number: row.get(3)?,
                block_type: row.get(4)?,
                text: row.get(5)?,
                x: row.get(6)?,
                y: row.get(7)?,
                width: row.get(8)?,
                height: row.get(9)?,
                page_width: row.get(10)?,
                page_height: row.get(11)?,
                confidence: row.get(12)?,
                reading_order: row.get(13)?,
                engine: row.get(14)?,
                source: row.get(15)?,
                reviewed: reviewed != 0,
            })
        })
        .map_err(|e| format!("LAYOUT_BLOCK_LIST_QUERY_FAILED: {e}"))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("LAYOUT_BLOCK_LIST_ROW_FAILED: {e}"))?);
    }
    Ok(out)
}

#[tauri::command]
pub fn list_document_extracted_fields(
    db: State<'_, DbState>,
    document_id: String,
) -> Result<Vec<ExtractedField>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT
                id, document_id, field_name, field_value, page_number,
                x, y, width, height, confidence, source, created_at
             FROM document_extracted_fields
             WHERE document_id = ?1
             ORDER BY page_number ASC, confidence DESC, field_name ASC",
        )
        .map_err(|e| format!("EXTRACTED_FIELD_LIST_PREPARE_FAILED: {e}"))?;
    let rows = stmt
        .query_map(params![document_id], |row| {
            Ok(ExtractedField {
                id: row.get(0)?,
                document_id: row.get(1)?,
                field_name: row.get(2)?,
                field_value: row.get(3)?,
                page_number: row.get(4)?,
                x: row.get(5)?,
                y: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                confidence: row.get(9)?,
                source: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| format!("EXTRACTED_FIELD_LIST_QUERY_FAILED: {e}"))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("EXTRACTED_FIELD_LIST_ROW_FAILED: {e}"))?);
    }
    Ok(out)
}

#[tauri::command]
pub fn analyze_document_with_ai_agent(
    db: State<'_, DbState>,
    document_id: String,
) -> Result<WorkProductSummary, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let (case_id, display_name): (String, String) = conn
        .query_row(
            "SELECT case_id, display_name FROM documents WHERE document_id = ?1",
            params![document_id.clone()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("AI_DOC_ANALYZE_DOC_NOT_FOUND: {e}"))?;
    let ocr_text: String = conn
        .query_row(
            "SELECT COALESCE(group_concat(COALESCE(NULLIF(ocr_formatted_text, ''), ocr_text, ''), char(10) || char(10)), '')
             FROM pages WHERE document_id = ?1 ORDER BY page_index",
            params![document_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or_default();
    let mut field_stmt = conn
        .prepare(
            "SELECT field_name, field_value, page_number, confidence, source
             FROM document_extracted_fields
             WHERE document_id = ?1
             ORDER BY confidence DESC
             LIMIT 40",
        )
        .map_err(|e| format!("AI_DOC_ANALYZE_FIELDS_PREPARE_FAILED: {e}"))?;
    let field_rows = field_stmt
        .query_map(params![document_id.clone()], |row| {
            Ok(format!(
                "- {}: {} (trang {}, {}%, {})",
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                (row.get::<_, f64>(3)? * 100.0).round(),
                row.get::<_, String>(4)?
            ))
        })
        .map_err(|e| format!("AI_DOC_ANALYZE_FIELDS_QUERY_FAILED: {e}"))?;
    let mut fields = Vec::new();
    for row in field_rows {
        fields.push(row.map_err(|e| format!("AI_DOC_ANALYZE_FIELDS_ROW_FAILED: {e}"))?);
    }
    let summary_excerpt = ocr_text
        .split_whitespace()
        .take(180)
        .collect::<Vec<_>>()
        .join(" ");
    let warnings = if fields.is_empty() {
        "- Chưa có trường trích xuất; cần chạy Rescan PDF/OCR + phát hiện layout trước.".to_string()
    } else {
        "- Đã phân tích dựa trên OCR/layout/field offline; cần kiểm sát viên xác nhận trường có confidence thấp.".to_string()
    };
    let body = format!(
        "AI Agent offline phân tích tài liệu: {display_name}\n\nTóm tắt trích xuất:\n{summary_excerpt}\n\nThông tin quan trọng:\n{}\n\nCảnh báo:\n{warnings}",
        if fields.is_empty() { "- Chưa có dữ liệu field.".to_string() } else { fields.join("\n") }
    );
    let wp_id = generate_work_product_id();
    conn.execute(
        "INSERT INTO work_products (
            work_product_id, case_id, work_product_type, status, title, body,
            linked_citation_ids, linked_entity_ids, created_at, updated_at
         ) VALUES (
            ?1, ?2, 'note', 'draft', ?3, ?4,
            '[]', '[]', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            wp_id,
            case_id,
            format!("Phân tích AI offline - {display_name}"),
            body
        ],
    )
    .map_err(|e| format!("AI_DOC_ANALYZE_INSERT_WORK_PRODUCT_FAILED: {e}"))?;
    conn.execute(
        "UPDATE documents
         SET summary_short = COALESCE(NULLIF(summary_short, ''), ?1),
             summary_detail = ?2,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE document_id = ?3",
        params![
            summary_excerpt.chars().take(240).collect::<String>(),
            body,
            document_id
        ],
    )
    .map_err(|e| format!("AI_DOC_ANALYZE_UPDATE_DOC_FAILED: {e}"))?;
    let _ = conn.execute(
        "INSERT INTO fts_documents(fts_documents) VALUES('rebuild')",
        [],
    );

    Ok(WorkProductSummary {
        work_product_id: wp_id,
        case_id,
        title: format!("Phân tích AI offline - {display_name}"),
        body,
        created_at: chrono_like_now(),
    })
}

#[tauri::command]
pub fn suggest_document_filename(
    db: State<'_, DbState>,
    document_id: String,
) -> Result<FilenameSuggestion, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let (
        original_filename,
        document_type,
        issued_date,
        issued_by,
        case_code,
        primary_person_name,
        import_sequence,
    ): (
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        Option<i32>,
    ) = conn
        .query_row(
            "SELECT
                d.original_filename,
                d.document_type,
                d.issued_date,
                d.issued_by,
                c.case_code,
                c.primary_person_name,
                d.import_sequence
             FROM documents d
             JOIN cases c ON c.case_id = d.case_id
             WHERE d.document_id = ?1",
            params![document_id.clone()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .map_err(|e| format!("SUGGEST_FILENAME_QUERY_FAILED: {e}"))?;

    let mut missing_fields = Vec::<String>::new();
    if issued_date.as_deref().unwrap_or_default().trim().is_empty() {
        missing_fields.push("Ngày văn bản".to_string());
    }
    if issued_by.as_deref().unwrap_or_default().trim().is_empty() {
        missing_fields.push("Cơ quan ban hành".to_string());
    }

    let stt = import_sequence.unwrap_or(1).max(1);
    let date = issued_date.unwrap_or_else(|| "Chua_ro_ngay".to_string());
    let authority = issued_by.unwrap_or_else(|| "Chua_ro_co_quan".to_string());
    let suggested_filename = format!(
        "{:03}_{}_{}_{}_{}_{}.pdf",
        stt,
        sanitize_filename_part(&document_type, "khong_xac_dinh"),
        sanitize_filename_part(&date, "Chua_ro_ngay"),
        sanitize_filename_part(&authority, "Chua_ro_co_quan"),
        sanitize_filename_part(&primary_person_name, "Chua_ro_nguoi_lien_quan"),
        sanitize_filename_part(&case_code, "HS")
    );

    Ok(FilenameSuggestion {
        document_id,
        suggested_filename,
        current_filename: original_filename,
        missing_fields,
    })
}

#[tauri::command]
pub fn rename_document_file(
    db: State<'_, DbState>,
    document_id: String,
    new_filename: String,
) -> Result<DocumentSummary, String> {
    let clean_name = sanitize_filename_part(&new_filename, "");
    if clean_name.is_empty() {
        return Err("RENAME_FILENAME_EMPTY".to_string());
    }

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let (case_id, file_path): (String, String) = conn
        .query_row(
            "SELECT case_id, file_path FROM documents WHERE document_id = ?1",
            params![document_id.clone()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("RENAME_DOC_QUERY_FAILED: {e}"))?;

    let current = Path::new(&file_path);
    let extension = current
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("pdf");
    let final_name = if clean_name
        .to_lowercase()
        .ends_with(&format!(".{extension}"))
    {
        clean_name
    } else {
        format!("{clean_name}.{extension}")
    };
    let target = current
        .parent()
        .ok_or_else(|| "RENAME_PARENT_NOT_FOUND".to_string())?
        .join(&final_name);

    if target.exists() {
        return Err("RENAME_TARGET_EXISTS".to_string());
    }

    run_rename_with_retry(current, &target)?;

    let display_name = target
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or(&final_name)
        .replace('_', " ");
    conn.execute(
        "UPDATE documents
         SET file_path = ?1,
             original_filename = ?2,
             display_name = ?3,
             document_title = ?3,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE document_id = ?4",
        params![
            target.to_string_lossy().to_string(),
            final_name,
            display_name,
            document_id,
        ],
    )
    .map_err(|e| format!("RENAME_DB_UPDATE_FAILED: {e}"))?;

    get_document_by_id(&conn, &document_id).or_else(|_| {
        conn.query_row(
            "SELECT
                document_id, case_id, display_name, original_filename, file_path,
                document_type, page_count, status, created_at
             FROM documents
             WHERE case_id = ?1
             ORDER BY updated_at DESC
             LIMIT 1",
            params![case_id],
            |row| {
                let file_path: String = row.get(4)?;
                let file_missing = !Path::new(&file_path).exists();
                Ok(DocumentSummary {
                    document_id: row.get(0)?,
                    case_id: row.get(1)?,
                    display_name: row.get(2)?,
                    original_filename: row.get(3)?,
                    file_path,
                    document_type: row.get(5)?,
                    page_count: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    file_missing,
                })
            },
        )
        .map_err(|e| format!("RENAME_REQUERY_FAILED: {e}"))
    })
}

#[tauri::command]
pub fn create_note_from_selection(
    db: State<'_, DbState>,
    case_id: String,
    document_id: String,
    page_number: i32,
    selected_text: String,
) -> Result<WorkProductSummary, String> {
    let text = selected_text.trim();
    if text.is_empty() {
        return Err("NOTE_SELECTION_EMPTY".to_string());
    }

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let display_name: String = conn
        .query_row(
            "SELECT display_name FROM documents WHERE document_id = ?1 AND case_id = ?2",
            params![document_id.clone(), case_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("NOTE_DOCUMENT_NOT_FOUND: {e}"))?;

    let work_product_id = generate_work_product_id();
    let title = format!("Ghi chú trang {} - {}", page_number.max(1), display_name);
    let source = json!({
        "document_id": document_id,
        "document_name": display_name,
        "page_number": page_number.max(1),
        "selected_text": text,
    });
    let body = format!(
        "Nguồn: {} - trang {}\n\n{}",
        source["document_name"].as_str().unwrap_or("Tài liệu"),
        page_number.max(1),
        text
    );

    conn.execute(
        "INSERT INTO work_products (
            work_product_id, case_id, work_product_type, status, title, body,
            linked_citation_ids, created_at, updated_at
         ) VALUES (
            ?1, ?2, 'note', 'draft', ?3, ?4, ?5,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            work_product_id,
            case_id,
            title,
            body,
            serde_json::to_string(&source).unwrap_or_else(|_| "{}".to_string())
        ],
    )
    .map_err(|e| format!("NOTE_INSERT_FAILED: {e}"))?;

    let created_at: String = conn
        .query_row(
            "SELECT created_at FROM work_products WHERE work_product_id = ?1",
            params![work_product_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or_default();

    Ok(WorkProductSummary {
        work_product_id,
        case_id,
        title,
        body,
        created_at,
    })
}

#[tauri::command]
pub fn move_document_to_case(
    db: State<'_, DbState>,
    document_id: String,
    target_case_id: String,
) -> Result<DocumentSummary, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let old_case_id: String = conn
        .query_row(
            "SELECT case_id FROM documents WHERE document_id = ?1",
            params![document_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("MOVE_DOC_NOT_FOUND: {e}"))?;

    let target_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cases WHERE case_id = ?1",
            params![target_case_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("MOVE_CASE_CHECK_FAILED: {e}"))?;
    if target_exists == 0 {
        return Err("MOVE_TARGET_CASE_NOT_FOUND".to_string());
    }

    conn.execute(
        "UPDATE documents
         SET case_id = ?1,
             needs_review = 1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE document_id = ?2",
        params![target_case_id.clone(), document_id.clone()],
    )
    .map_err(|e| format!("MOVE_DOC_UPDATE_FAILED: {e}"))?;

    for case_id in [old_case_id.clone(), target_case_id.clone()] {
        let _ = conn.execute(
            "UPDATE cases
             SET document_count = (SELECT COUNT(*) FROM documents WHERE case_id = ?1),
                 total_pages = (SELECT COALESCE(SUM(page_count), 0) FROM documents WHERE case_id = ?1),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE case_id = ?1",
            params![case_id],
        );
    }

    let audit_id = generate_audit_event_id();
    let _ = conn.execute(
        "INSERT INTO audit_events (
            audit_event_id, object_type, object_id, field_name, action_type,
            before_value, after_value, actor_type, actor_id, reason_code,
            reason_note, approval_status, created_at
         ) VALUES (
            ?1, 'document', ?2, 'case_id', 'update',
            ?3, ?4, 'user', 'desktop-ui', 'move_document',
            'Di chuyển tài liệu sang hồ sơ khác', 'not_required',
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![audit_id, document_id, old_case_id, target_case_id],
    );

    get_document_by_id(&conn, &document_id)
}
