use crate::commands::module_cmd::DbState;
use crate::storage;
use log::{error, info, warn};
use lopdf::Document as PdfDocument;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State};

static SCAN_COUNTER: AtomicU64 = AtomicU64::new(0);
const SCAN_ALLOWED_EXTENSIONS: &[&str] = &["pdf", "png", "jpg", "jpeg", "tif", "tiff", "bmp"];

#[derive(Default)]
pub struct ScanWatchState {
    pub active: AtomicBool,
    pub folder: Mutex<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanSettings {
    pub inbox_folder: String,
    pub import_mode: String,
    pub default_case_id: String,
    pub accept_pdf: bool,
    pub accept_tiff: bool,
    pub accept_jpg: bool,
    pub accept_png: bool,
    pub stable_wait_ms: u64,
    pub duplicate_detection: bool,
    pub ocr_after_scan: bool,
    pub classify_after_scan: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanInboxFile {
    pub file_path: String,
    pub file_name: String,
    pub file_ext: String,
    pub file_size: i64,
    pub modified_at: String,
    pub status: String,
    pub reason: String,
    pub duplicate: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanImportResult {
    pub document_id: String,
    pub case_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScannerDriverStatus {
    pub scan_to_folder_supported: bool,
    pub direct_scan_supported: bool,
    pub wia_service_status: String,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineJob {
    pub job_id: String,
    pub source_type: String,
    pub status: String,
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub cancelled_tasks: i64,
    pub pause_requested: bool,
    pub cancel_requested: bool,
    pub created_at: String,
    pub started_at: Option<String>,
    pub paused_at: Option<String>,
    pub resumed_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub updated_at: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineJobListItem {
    pub job_id: String,
    pub source_type: String,
    pub status: String,
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub cancelled_tasks: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineFooterStatus {
    pub active_jobs: i64,
    pub paused_jobs: i64,
    pub failed_jobs: i64,
    pub completed_jobs: i64,
    pub cancelled_jobs: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineProgressStatus {
    pub job_id: String,
    pub status: String,
    pub source_type: String,
    pub phase: String,
    pub processed_tasks: i64,
    pub total_tasks: i64,
    pub running_tasks: i64,
    pub queued_tasks: i64,
    pub failed_tasks: i64,
    pub retrying_tasks: i64,
    pub progress_percent: f64,
    pub eta_seconds: Option<i64>,
    pub can_resume: bool,
    pub can_pause: bool,
    pub can_cancel: bool,
    pub updated_at: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineProcessModeSettings {
    pub process_mode: String,
    pub autoscan: bool,
    pub autosave: bool,
    pub autoname: bool,
    pub autosummary: bool,
    pub autoclassify: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineTickResult {
    pub job_id: String,
    pub worker_pool_size: i64,
    pub max_inflight: i64,
    pub queue_high_watermark: i64,
    pub queued_total: i64,
    pub running_total: i64,
    pub claimed_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub skipped_not_ready_count: i64,
    pub stopped_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineIntegrationCheckResult {
    pub job_id: String,
    pub discover_completed: bool,
    pub import_completed: bool,
    pub ocr_reached: bool,
    pub chain_ok: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineKpiSummary {
    pub total_jobs: i64,
    pub running_jobs: i64,
    pub completed_jobs: i64,
    pub failed_jobs: i64,
    pub cancelled_jobs: i64,
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub completion_rate_percent: f64,
    pub generated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineSafetyReport {
    pub queue_high_watermark: i64,
    pub max_worker_pool_cap: i64,
    pub max_inflight_cap: i64,
    pub blocked_jobs_by_watermark: i64,
    pub jobs_with_retry_pressure: i64,
    pub unresolved_risks: Vec<String>,
    pub rollout_guards: Vec<String>,
}

#[derive(Debug, Clone)]
struct PipelineTaskRow {
    task_id: String,
    task_type: String,
    status: String,
    attempt: i64,
    payload_json: Option<String>,
}

const PIPELINE_MAX_RETRY: i64 = 3;
const PIPELINE_RETRY_BASE_MS: u64 = 1500;
const PIPELINE_MAX_POOL_CAP: i64 = 6;
const PIPELINE_MAX_INFLIGHT_CAP: i64 = 8;
const PIPELINE_QUEUE_HIGH_WATERMARK: i64 = 2000;

fn pipeline_event(
    conn: &rusqlite::Connection,
    job_id: &str,
    task_id: Option<&str>,
    level: &str,
    event_type: &str,
    message: &str,
    from_status: Option<&str>,
    to_status: Option<&str>,
) {
    let event_id = generate_id("pipevent");
    if let Err(e) = conn.execute(
        "INSERT INTO pipeline_events (
            event_id, job_id, task_id, level, event_type,
            message, from_status, to_status, meta_json, created_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            event_id,
            job_id,
            task_id.map(|v| v.to_string()),
            level,
            event_type,
            message,
            from_status.map(|v| v.to_string()),
            to_status.map(|v| v.to_string()),
        ],
    ) {
        error!(
            "pipeline_events insert failed job_id={} event_type={} error={}",
            job_id, event_type, e
        );
    }
}

fn pipeline_event_with_meta(
    conn: &rusqlite::Connection,
    job_id: &str,
    task_id: Option<&str>,
    level: &str,
    event_type: &str,
    message: &str,
    from_status: Option<&str>,
    to_status: Option<&str>,
    meta_json: Option<String>,
) {
    let event_id = generate_id("pipevent");
    if let Err(e) = conn.execute(
        "INSERT INTO pipeline_events (
            event_id, job_id, task_id, level, event_type,
            message, from_status, to_status, meta_json, created_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            event_id,
            job_id,
            task_id.map(|v| v.to_string()),
            level,
            event_type,
            message,
            from_status.map(|v| v.to_string()),
            to_status.map(|v| v.to_string()),
            meta_json,
        ],
    ) {
        error!(
            "pipeline_events insert failed job_id={} event_type={} error={}",
            job_id, event_type, e
        );
    }
}

fn derive_worker_pool_size() -> i64 {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(2);
    cores.clamp(1, PIPELINE_MAX_POOL_CAP)
}

fn derive_max_inflight(worker_pool_size: i64) -> i64 {
    (worker_pool_size * 2).clamp(1, PIPELINE_MAX_INFLIGHT_CAP)
}

fn enqueue_pipeline_task(
    conn: &rusqlite::Connection,
    job_id: &str,
    task_type: &str,
    payload_json: Option<String>,
) -> Result<String, String> {
    let task_id = generate_id("pitask");
    conn.execute(
        "INSERT INTO pipeline_tasks (
            task_id, job_id, task_type, status, attempt, payload_json, created_at, updated_at
         ) VALUES (
            ?1, ?2, ?3, 'queued', 0, ?4,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![task_id, job_id, task_type, payload_json],
    )
    .map_err(|e| format!("PIPELINE_TASK_ENQUEUE_FAILED: {e}"))?;

    conn.execute(
        "UPDATE pipeline_jobs
         SET total_tasks = total_tasks + 1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE job_id = ?1",
        params![job_id],
    )
    .map_err(|e| format!("PIPELINE_JOB_TOTAL_TASKS_UPDATE_FAILED: {e}"))?;

    pipeline_event_with_meta(
        conn,
        job_id,
        Some(&task_id),
        "info",
        "task_queued",
        &format!("Task queued: {task_type}"),
        None,
        Some("queued"),
        Some(json!({ "task_type": task_type }).to_string()),
    );

    Ok(task_id)
}

fn parse_not_before_ms(payload_json: &Option<String>) -> u128 {
    payload_json
        .as_ref()
        .and_then(|v| serde_json::from_str::<serde_json::Value>(v).ok())
        .and_then(|v| v.get("not_before_epoch_ms").and_then(|n| n.as_u64()))
        .map(|v| v as u128)
        .unwrap_or(0)
}

fn execute_phase_placeholder(
    conn: &rusqlite::Connection,
    job_id: &str,
    task: &PipelineTaskRow,
) -> Result<Option<String>, String> {
    let next_phase = match task.task_type.as_str() {
        "discover" => Some("import"),
        "import" => Some("ocr"),
        "ocr" => Some("ai"),
        "ai" => Some("persist"),
        "persist" => None,
        other => {
            return Err(format!("PIPELINE_TASK_TYPE_UNSUPPORTED: {other}"));
        }
    };

    let result_json = json!({
        "phase": task.task_type,
        "status": "ok",
        "placeholder": true,
        "at_ms": now_millis(),
    })
    .to_string();

    conn.execute(
        "UPDATE pipeline_tasks
         SET status = 'completed',
             result_json = ?2,
             finished_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE task_id = ?1",
        params![task.task_id, result_json],
    )
    .map_err(|e| format!("PIPELINE_TASK_COMPLETE_FAILED: {e}"))?;

    conn.execute(
        "UPDATE pipeline_jobs
         SET completed_tasks = completed_tasks + 1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE job_id = ?1",
        params![job_id],
    )
    .map_err(|e| format!("PIPELINE_JOB_COMPLETED_UPDATE_FAILED: {e}"))?;

    pipeline_event_with_meta(
        conn,
        job_id,
        Some(&task.task_id),
        "info",
        "task_completed",
        &format!("Task completed: {}", task.task_type),
        Some(&task.status),
        Some("completed"),
        Some(json!({ "phase": task.task_type, "next_phase": next_phase }).to_string()),
    );

    Ok(next_phase.map(|v| v.to_string()))
}

fn parse_mode_settings(conn: &rusqlite::Connection) -> PipelineProcessModeSettings {
    PipelineProcessModeSettings {
        process_mode: setting(conn, "pipeline.process_mode", "manual"),
        autoscan: bool_setting(conn, "pipeline.autoscan", true),
        autosave: bool_setting(conn, "pipeline.autosave", true),
        autoname: bool_setting(conn, "pipeline.autoname", false),
        autosummary: bool_setting(conn, "pipeline.autosummary", false),
        autoclassify: bool_setting(conn, "pipeline.autoclassify", false),
    }
}

fn current_phase_name(task_type: &str) -> String {
    match task_type {
        "discover" => "discover",
        "import" => "import",
        "ocr" => "ocr",
        "ai" => "ai",
        "persist" => "persist",
        _ => "unknown",
    }
    .to_string()
}

fn complete_task_with_result(
    conn: &rusqlite::Connection,
    job_id: &str,
    task_id: &str,
    result_json: String,
) -> Result<(), String> {
    conn.execute(
        "UPDATE pipeline_tasks
         SET status = 'completed',
             result_json = ?2,
             finished_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE task_id = ?1",
        params![task_id, result_json],
    )
    .map_err(|e| format!("PIPELINE_TASK_COMPLETE_FAILED: {e}"))?;
    conn.execute(
        "UPDATE pipeline_jobs
         SET completed_tasks = completed_tasks + 1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE job_id = ?1",
        params![job_id],
    )
    .map_err(|e| format!("PIPELINE_JOB_COMPLETED_UPDATE_FAILED: {e}"))?;
    Ok(())
}

fn execute_phase_action(
    app: Option<&AppHandle>,
    conn: &rusqlite::Connection,
    job_id: &str,
    task: &PipelineTaskRow,
) -> Result<Option<String>, String> {
    let payload: serde_json::Value = task
        .payload_json
        .as_ref()
        .and_then(|v| serde_json::from_str(v).ok())
        .unwrap_or_else(|| json!({}));

    match task.task_type.as_str() {
        "discover" => {
            let ready_count = discover_ready_file_count(conn)?;
            let result_json = json!({
                "phase": "discover",
                "status": "ok",
                "real_action": true,
                "at_ms": now_millis(),
                "seed": payload.get("seed").and_then(|v| v.as_bool()).unwrap_or(false),
                "ready_count": ready_count,
            })
            .to_string();
            complete_task_with_result(conn, job_id, &task.task_id, result_json)?;
            Ok(Some("import".to_string()))
        }
        "import" => {
            let mut source_file = payload
                .get("source_file")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let mut case_id = payload
                .get("case_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            if source_file.trim().is_empty() {
                source_file = discover_first_ready_file(conn).unwrap_or_default();
            }
            if case_id.trim().is_empty() {
                case_id = resolve_case_id_for_autonomous_import(conn);
            }

            if source_file.trim().is_empty() || case_id.trim().is_empty() {
                let result_json = json!({
                    "phase": "import",
                    "status": "deferred",
                    "status_code": "IMPORT_INPUT_MISSING",
                    "todo": "Provide source_file/case_id or set scan.ricoh.default_case_id, and ensure inbox has ready files.",
                    "source_file": source_file,
                    "case_id": case_id,
                    "at_ms": now_millis(),
                })
                .to_string();
                complete_task_with_result(conn, job_id, &task.task_id, result_json)?;
                return Ok(None);
            }

            if !try_claim_ready_file(conn, &source_file, &case_id)? {
                let result_json = json!({
                    "phase": "import",
                    "status": "deferred",
                    "status_code": "IMPORT_FILE_ALREADY_CLAIMED",
                    "todo": "Another worker/job already claimed this ready file; skip current import tick.",
                    "source_file": source_file,
                    "case_id": case_id,
                    "at_ms": now_millis(),
                })
                .to_string();
                complete_task_with_result(conn, job_id, &task.task_id, result_json)?;
                return Ok(None);
            }

            let import_result = import_scanned_file_core(conn, &source_file, &case_id)?;
            let result_json = json!({
                "phase": "import",
                "status": "ok",
                "real_action": true,
                "import_status": import_result.status,
                "document_id": import_result.document_id,
                "message": import_result.message,
                "at_ms": now_millis(),
            })
            .to_string();
            complete_task_with_result(conn, job_id, &task.task_id, result_json)?;
            if import_result.document_id.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some("ocr".to_string()))
            }
        }
        "ocr" => {
            let document_id = payload
                .get("document_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let result_json = if document_id.trim().is_empty() {
                json!({
                    "phase": "ocr",
                    "status": "deferred",
                    "status_code": "OCR_DOCUMENT_ID_MISSING",
                    "todo": "Upstream import must provide document_id payload.",
                    "at_ms": now_millis(),
                })
                .to_string()
            } else {
                match app {
                    Some(app_handle) => {
                        match run_ocr_for_document_in_pipeline(app_handle, conn, &document_id) {
                            Ok(ocr) => json!({
                                "phase": "ocr",
                                "status": "ok",
                                "real_action": true,
                                "document_id": document_id,
                                "processed_pages": ocr.processed_pages,
                                "failed_pages": ocr.failed_pages,
                                "handwritten_pages": ocr.handwritten_pages,
                                "average_confidence": ocr.average_confidence,
                                "message": ocr.message,
                                "at_ms": now_millis(),
                            })
                            .to_string(),
                            Err(e) => json!({
                                "phase": "ocr",
                                "status": "error",
                                "status_code": "OCR_EXECUTION_FAILED",
                                "document_id": document_id,
                                "error": e,
                                "at_ms": now_millis(),
                            })
                            .to_string(),
                        }
                    }
                    None => json!({
                        "phase": "ocr",
                        "status": "deferred",
                        "status_code": "OCR_EXECUTION_REQUIRES_APPHANDLE",
                        "todo": "Use app-aware tick endpoint for OCR execution.",
                        "document_id": document_id,
                        "at_ms": now_millis(),
                    })
                    .to_string(),
                }
            };
            complete_task_with_result(conn, job_id, &task.task_id, result_json)?;
            Ok(Some("ai".to_string()))
        }
        _ => execute_phase_placeholder(conn, job_id, task),
    }
}

fn fail_or_retry_task(
    conn: &rusqlite::Connection,
    job_id: &str,
    task: &PipelineTaskRow,
    err_msg: &str,
) -> Result<(), String> {
    let next_attempt = task.attempt + 1;
    if next_attempt < PIPELINE_MAX_RETRY {
        let backoff_ms = PIPELINE_RETRY_BASE_MS * (1_u64 << (next_attempt as u32));
        let not_before_epoch_ms = now_millis() + backoff_ms as u128;
        let payload = json!({
            "retry": true,
            "backoff_ms": backoff_ms,
            "not_before_epoch_ms": not_before_epoch_ms,
            "prev_error": err_msg,
        })
        .to_string();

        conn.execute(
            "UPDATE pipeline_tasks
             SET status = 'queued',
                 attempt = ?2,
                 payload_json = ?3,
                 error_message = ?4,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE task_id = ?1",
            params![task.task_id, next_attempt, payload, err_msg],
        )
        .map_err(|e| format!("PIPELINE_TASK_RETRY_UPDATE_FAILED: {e}"))?;

        pipeline_event_with_meta(
            conn,
            job_id,
            Some(&task.task_id),
            "warn",
            "task_retry_scheduled",
            &format!("Task retry scheduled: {}", task.task_type),
            Some("running"),
            Some("queued"),
            Some(
                json!({ "attempt": next_attempt, "backoff_ms": backoff_ms, "error": err_msg })
                    .to_string(),
            ),
        );
        Ok(())
    } else {
        conn.execute(
            "UPDATE pipeline_tasks
             SET status = 'failed',
                 attempt = ?2,
                 error_message = ?3,
                 finished_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE task_id = ?1",
            params![task.task_id, next_attempt, err_msg],
        )
        .map_err(|e| format!("PIPELINE_TASK_FAIL_UPDATE_FAILED: {e}"))?;
        conn.execute(
            "UPDATE pipeline_jobs
             SET failed_tasks = failed_tasks + 1,
                 last_error = ?2,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE job_id = ?1",
            params![job_id, err_msg],
        )
        .map_err(|e| format!("PIPELINE_JOB_FAIL_UPDATE_FAILED: {e}"))?;

        pipeline_event_with_meta(
            conn,
            job_id,
            Some(&task.task_id),
            "error",
            "task_failed",
            &format!("Task failed: {}", task.task_type),
            Some("running"),
            Some("failed"),
            Some(json!({ "attempt": next_attempt, "error": err_msg }).to_string()),
        );
        Ok(())
    }
}

fn map_pipeline_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<PipelineJob> {
    Ok(PipelineJob {
        job_id: row.get(0)?,
        source_type: row.get(1)?,
        status: row.get(2)?,
        total_tasks: row.get(3)?,
        completed_tasks: row.get(4)?,
        failed_tasks: row.get(5)?,
        cancelled_tasks: row.get(6)?,
        pause_requested: row.get::<_, i64>(7)? == 1,
        cancel_requested: row.get::<_, i64>(8)? == 1,
        created_at: row.get(9)?,
        started_at: row.get(10)?,
        paused_at: row.get(11)?,
        resumed_at: row.get(12)?,
        completed_at: row.get(13)?,
        cancelled_at: row.get(14)?,
        updated_at: row.get(15)?,
        last_error: row.get(16)?,
    })
}

fn load_pipeline_job(conn: &rusqlite::Connection, job_id: &str) -> Result<PipelineJob, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                job_id, source_type, status, total_tasks, completed_tasks, failed_tasks,
                cancelled_tasks, pause_requested, cancel_requested, created_at, started_at,
                paused_at, resumed_at, completed_at, cancelled_at, updated_at, last_error
             FROM pipeline_jobs WHERE job_id = ?1",
        )
        .map_err(|e| format!("PIPELINE_GET_PREPARE_FAILED: {e}"))?;

    stmt.query_row(params![job_id], map_pipeline_job)
        .map_err(|e| format!("PIPELINE_JOB_NOT_FOUND: {e}"))
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn is_supported_scan_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SCAN_ALLOWED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn generate_id(prefix: &str) -> String {
    let n = SCAN_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{}-{n}", now_millis())
}

fn setting(conn: &rusqlite::Connection, key: &str, default: &str) -> String {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_else(|_| default.to_string())
}

fn bool_setting(conn: &rusqlite::Connection, key: &str, default: bool) -> bool {
    setting(conn, key, if default { "true" } else { "false" }) == "true"
}

fn parse_settings(conn: &rusqlite::Connection) -> ScanSettings {
    ScanSettings {
        inbox_folder: setting(conn, "scan.ricoh.inbox_folder", ""),
        import_mode: setting(conn, "scan.ricoh.import_mode", "ask"),
        default_case_id: setting(conn, "scan.ricoh.default_case_id", ""),
        accept_pdf: bool_setting(conn, "scan.ricoh.accept_pdf", true),
        accept_tiff: bool_setting(conn, "scan.ricoh.accept_tiff", true),
        accept_jpg: bool_setting(conn, "scan.ricoh.accept_jpg", true),
        accept_png: bool_setting(conn, "scan.ricoh.accept_png", true),
        stable_wait_ms: setting(conn, "scan.ricoh.stable_wait_ms", "2500")
            .parse()
            .unwrap_or(2500),
        duplicate_detection: bool_setting(conn, "scan.ricoh.duplicate_detection", true),
        ocr_after_scan: bool_setting(conn, "scan.ricoh.ocr_after_scan", true),
        classify_after_scan: bool_setting(conn, "scan.ricoh.classify_after_scan", true),
    }
}

fn upsert_setting(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = excluded.updated_at",
        params![key, value],
    )
    .map_err(|e| format!("SCAN_SETTING_UPSERT_FAILED: {e}"))?;
    Ok(())
}

fn is_allowed_ext(ext: &str, settings: &ScanSettings) -> bool {
    match ext {
        "pdf" => settings.accept_pdf,
        "tif" | "tiff" => settings.accept_tiff,
        "jpg" | "jpeg" => settings.accept_jpg,
        "png" => settings.accept_png,
        _ => false,
    }
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("SCAN_HASH_OPEN_FAILED {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("SCAN_HASH_READ_FAILED {}: {e}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn stable_status(metadata: &fs::Metadata, wait_ms: u64) -> (String, String) {
    let modified = match metadata.modified() {
        Ok(value) => value,
        Err(_) => {
            return (
                "error".to_string(),
                "Không đọc được modified time".to_string(),
            )
        }
    };
    let elapsed_ms = SystemTime::now()
        .duration_since(modified)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    if elapsed_ms < wait_ms {
        (
            "waiting_for_stable".to_string(),
            "Đang chờ máy Ricoh ghi xong file".to_string(),
        )
    } else {
        ("ready".to_string(), "Sẵn sàng import".to_string())
    }
}

fn count_pdf_pages(path: &Path) -> i32 {
    match PdfDocument::load(path) {
        Ok(doc) => doc.get_pages().len() as i32,
        Err(_) => 0,
    }
}

fn classify_document_type_from_name(file_name: &str) -> String {
    let n = file_name.to_lowercase();
    if n.contains("to khai") || n.contains("loi khai") || n.contains("lời khai") {
        "to_khai".to_string()
    } else if n.contains("bien ban") || n.contains("biên bản") {
        "bien_ban".to_string()
    } else if n.contains("quyet dinh") || n.contains("quyết định") || n.contains("qd") {
        "quyet_dinh".to_string()
    } else {
        "khong_xac_dinh".to_string()
    }
}

fn discover_ready_file_count(conn: &rusqlite::Connection) -> Result<i64, String> {
    let settings = parse_settings(conn);
    let folder = settings.inbox_folder.trim();
    if folder.is_empty() {
        return Ok(0);
    }
    let root = PathBuf::from(folder);
    if !root.exists() || !root.is_dir() {
        return Ok(0);
    }
    let mut count = 0_i64;
    for entry in fs::read_dir(root).map_err(|e| format!("SCAN_DISCOVER_READ_DIR_FAILED: {e}"))? {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if !is_allowed_ext(&ext, &settings) {
            continue;
        }
        let metadata = match fs::metadata(&path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let (status, _) = stable_status(&metadata, settings.stable_wait_ms);
        if status != "ready" {
            continue;
        }
        count += 1;
    }
    Ok(count)
}

fn discover_first_ready_file(conn: &rusqlite::Connection) -> Option<String> {
    let settings = parse_settings(conn);
    let folder = settings.inbox_folder.trim();
    if folder.is_empty() {
        return None;
    }
    let root = PathBuf::from(folder);
    if !root.exists() || !root.is_dir() {
        return None;
    }
    let mut candidates: Vec<(u64, String)> = Vec::new();
    let entries = fs::read_dir(root).ok()?;
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if !is_allowed_ext(&ext, &settings) {
            continue;
        }
        let metadata = match fs::metadata(&path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let (status, _) = stable_status(&metadata, settings.stable_wait_ms);
        if status != "ready" {
            continue;
        }
        let modified_secs = metadata
            .modified()
            .ok()
            .and_then(|v| v.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        candidates.push((modified_secs, path.to_string_lossy().to_string()));
    }
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.first().map(|(_, p)| p.clone())
}

fn resolve_case_id_for_autonomous_import(conn: &rusqlite::Connection) -> String {
    let settings = parse_settings(conn);
    if !settings.default_case_id.trim().is_empty() {
        let configured = settings.default_case_id.trim().to_string();
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cases WHERE case_id = ?1",
                params![configured.clone()],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if exists > 0 {
            return configured;
        }
    }
    let from_setting_alias: String = conn
        .query_row(
            "SELECT c.case_id
             FROM app_settings s
             JOIN cases c ON lower(trim(c.case_code)) = lower(trim(s.value))
             WHERE s.key = 'scan.ricoh.default_case_id' AND trim(s.value) <> ''
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();
    if !from_setting_alias.trim().is_empty() {
        return from_setting_alias;
    }
    conn.query_row(
        "SELECT case_id FROM cases ORDER BY updated_at DESC, created_at DESC LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_default()
}

fn try_claim_ready_file(
    conn: &rusqlite::Connection,
    source_file: &str,
    case_id: &str,
) -> Result<bool, String> {
    let source = source_file.trim();
    if source.is_empty() {
        return Ok(false);
    }
    let active_claims: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM scan_jobs
             WHERE original_scan_path = ?1
               AND scan_import_status IN ('importing', 'imported', 'duplicate')",
            params![source],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_CLAIM_CHECK_FAILED: {e}"))?;
    if active_claims > 0 {
        return Ok(false);
    }
    let claim_id = generate_id("scanclaim");
    conn.execute(
        "INSERT INTO scan_jobs (
            scan_job_id, source_type, scanner_model, original_scan_path,
            case_id, scan_import_status, detected_at
         ) VALUES (
            ?1, 'ricoh_folder', 'Pipeline claim lock', ?2,
            ?3, 'importing', strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![claim_id, source, case_id],
    )
    .map_err(|e| format!("PIPELINE_CLAIM_INSERT_FAILED: {e}"))?;
    Ok(true)
}

fn run_ocr_for_document_in_pipeline(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    document_id: &str,
) -> Result<crate::commands::doc_cmd::OcrRunResult, String> {
    crate::commands::doc_cmd::run_ocr_for_document_with_conn(app, conn, document_id)
}

fn display_name(file_name: &str) -> String {
    Path::new(file_name)
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or(file_name)
        .replace(['_', '-'], " ")
}

#[tauri::command]
pub fn get_scan_settings(db: State<'_, DbState>) -> Result<ScanSettings, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    Ok(parse_settings(&conn))
}

#[tauri::command]
pub fn save_scan_settings(
    db: State<'_, DbState>,
    settings: ScanSettings,
) -> Result<ScanSettings, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    upsert_setting(&conn, "scan.ricoh.inbox_folder", &settings.inbox_folder)?;
    upsert_setting(&conn, "scan.ricoh.import_mode", &settings.import_mode)?;
    upsert_setting(
        &conn,
        "scan.ricoh.default_case_id",
        &settings.default_case_id,
    )?;
    upsert_setting(
        &conn,
        "scan.ricoh.accept_pdf",
        &settings.accept_pdf.to_string(),
    )?;
    upsert_setting(
        &conn,
        "scan.ricoh.accept_tiff",
        &settings.accept_tiff.to_string(),
    )?;
    upsert_setting(
        &conn,
        "scan.ricoh.accept_jpg",
        &settings.accept_jpg.to_string(),
    )?;
    upsert_setting(
        &conn,
        "scan.ricoh.accept_png",
        &settings.accept_png.to_string(),
    )?;
    upsert_setting(
        &conn,
        "scan.ricoh.stable_wait_ms",
        &settings.stable_wait_ms.to_string(),
    )?;
    upsert_setting(
        &conn,
        "scan.ricoh.duplicate_detection",
        &settings.duplicate_detection.to_string(),
    )?;
    upsert_setting(
        &conn,
        "scan.ricoh.ocr_after_scan",
        &settings.ocr_after_scan.to_string(),
    )?;
    upsert_setting(
        &conn,
        "scan.ricoh.classify_after_scan",
        &settings.classify_after_scan.to_string(),
    )?;
    Ok(parse_settings(&conn))
}

#[tauri::command]
pub fn watch_scan_folder(
    state: State<'_, Arc<ScanWatchState>>,
    folder_path: String,
) -> Result<(), String> {
    let path = PathBuf::from(folder_path.trim());
    if !path.exists() || !path.is_dir() {
        return Err("SCAN_WATCH_FOLDER_INVALID".to_string());
    }
    state.active.store(true, Ordering::Relaxed);
    let mut folder = state
        .folder
        .lock()
        .map_err(|e| format!("SCAN_WATCH_LOCK_FAILED: {e}"))?;
    *folder = path.to_string_lossy().to_string();
    Ok(())
}

#[tauri::command]
pub fn stop_watch_scan_folder(state: State<'_, Arc<ScanWatchState>>) -> Result<(), String> {
    state.active.store(false, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn list_scan_inbox_files(
    db: State<'_, DbState>,
    folder_path: Option<String>,
) -> Result<Vec<ScanInboxFile>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let settings = parse_settings(&conn);
    let folder = folder_path
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(settings.inbox_folder.clone());
    if folder.trim().is_empty() {
        return Ok(Vec::new());
    }
    let root = PathBuf::from(folder);
    if !root.exists() || !root.is_dir() {
        return Err("SCAN_INBOX_FOLDER_INVALID".to_string());
    }

    let mut out = Vec::<ScanInboxFile>::new();
    for entry in fs::read_dir(root).map_err(|e| format!("SCAN_READ_DIR_FAILED: {e}"))? {
        let entry = match entry {
            Ok(value) => value,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if !is_allowed_ext(&ext, &settings) {
            continue;
        }
        let metadata = match fs::metadata(&path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let (mut status, mut reason) = stable_status(&metadata, settings.stable_wait_ms);
        let mut duplicate = false;
        if status == "ready" && settings.duplicate_detection {
            if let Ok(hash) = hash_file(&path) {
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM documents WHERE file_hash = ?1",
                        params![hash],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if count > 0 {
                    status = "duplicate".to_string();
                    reason = "Trùng hash với tài liệu đã import".to_string();
                    duplicate = true;
                }
            }
        }

        out.push(ScanInboxFile {
            file_path: path.to_string_lossy().to_string(),
            file_name: path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_string(),
            file_ext: ext,
            file_size: metadata.len() as i64,
            modified_at: metadata
                .modified()
                .ok()
                .and_then(|v| v.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default(),
            status,
            reason,
            duplicate,
        });
    }
    out.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    Ok(out)
}

#[tauri::command]
pub fn import_scanned_file(
    db: State<'_, DbState>,
    file_path: String,
    case_id: String,
) -> Result<ScanImportResult, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    import_scanned_file_core(&conn, &file_path, &case_id)
}

fn import_scanned_file_core(
    conn: &rusqlite::Connection,
    file_path: &str,
    case_id: &str,
) -> Result<ScanImportResult, String> {
    let source = PathBuf::from(file_path.trim());
    if !source.exists() || !source.is_file() {
        return Err("SCAN_IMPORT_FILE_NOT_FOUND".to_string());
    }
    if !is_supported_scan_file(&source) {
        return Err("SCAN_IMPORT_UNSUPPORTED_FILE: Ricoh scan chỉ nhận PDF hoặc ảnh scan".to_string());
    }
    let case_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cases WHERE case_id = ?1",
            params![case_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|e| format!("SCAN_IMPORT_CASE_CHECK_FAILED: {e}"))?;
    if case_exists == 0 {
        return Err("SCAN_IMPORT_CASE_NOT_FOUND".to_string());
    }
    let case_code: String = conn
        .query_row(
            "SELECT case_code FROM cases WHERE case_id = ?1",
            params![case_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|e| format!("SCAN_IMPORT_CASE_CODE_FAILED: {e}"))?;

    let file_hash = hash_file(&source)?;
    let duplicate_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE file_hash = ?1",
            params![file_hash.clone()],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if duplicate_count > 0 {
        return Ok(ScanImportResult {
            document_id: String::new(),
            case_id: case_id.to_string(),
            status: "duplicate".to_string(),
            message: "File trùng hash với tài liệu đã import".to_string(),
        });
    }

    let file_name = source
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("scan.pdf")
        .to_string();
    let ext = source
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_lowercase();
    let document_id = generate_id("doc");
    let document_type = classify_document_type_from_name(&file_name);
    let display_name = display_name(&file_name);
    let file_size = fs::metadata(&source).map(|m| m.len() as i64).unwrap_or(0);
    let page_count = if ext == "pdf" {
        count_pdf_pages(&source)
    } else {
        1
    };
    let source_path = source.to_string_lossy().to_string();
    let managed_path = storage::copy_to_originals(&source, &case_code, &document_id, &file_name)?;
    let managed_file_path = managed_path.to_string_lossy().to_string();

    conn.execute(
        "INSERT INTO documents (
            document_id, case_id, original_filename, file_path, file_hash,
            file_size, page_count, display_name, document_title, document_type,
            summary_short, summary_detail, status, created_at, updated_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9,
            ?10, ?11, 'pending', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            document_id,
            case_id,
            file_name,
            managed_file_path,
            file_hash,
            file_size,
            page_count,
            display_name,
            document_type,
            format!("{} ({})", display_name, document_type),
            "Import từ Ricoh Scan Inbox",
        ],
    )
    .map_err(|e| format!("SCAN_IMPORT_INSERT_DOC_FAILED: {e}"))?;

    for page in 1..=page_count.max(1) {
        let page_id = generate_id("page");
        conn.execute(
            "INSERT INTO pages (page_id, document_id, page_index, image_path, created_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![
                page_id,
                document_id,
                page,
                if ext == "pdf" {
                    None::<String>
                } else {
                    Some(managed_file_path.clone())
                },
            ],
        )
        .map_err(|e| format!("SCAN_IMPORT_INSERT_PAGE_FAILED: {e}"))?;
    }

    let scan_job_id = generate_id("scanjob");
    conn.execute(
        "INSERT INTO scan_jobs (
            scan_job_id, source_type, scanner_model, original_scan_path,
            file_hash, case_id, document_id, scan_import_status,
            detected_at, imported_at
         ) VALUES (
            ?1, 'ricoh_folder', 'Ricoh MFP Scan to Folder', ?2,
            ?3, ?4, ?5, 'imported',
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![scan_job_id, source_path, file_hash, case_id, document_id],
    )
    .map_err(|e| format!("SCAN_JOB_INSERT_FAILED: {e}"))?;

    let review_id = generate_id("review");
    let _ = conn.execute(
        "INSERT INTO review_queue (review_id, object_type, object_id, reason, priority, status, created_at)
         VALUES (?1, 'document', ?2, 'ricoh_scan_ocr_pending', 1, 'pending', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
        params![review_id, document_id],
    );
    let audit_id = generate_id("audit");
    let _ = conn.execute(
        "INSERT INTO audit_events (
            audit_event_id, object_type, object_id, field_name, action_type,
            before_value, after_value, actor_type, actor_id, reason_code,
            reason_note, approval_status, created_at
        ) VALUES (
            ?1, 'document', ?2, 'file_path', 'create',
            ?3, ?4, 'system', 'ricoh-scan-inbox',
            'ricoh_scan_import', 'Import từ Ricoh Scan to Folder',
            'not_required', strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![audit_id, document_id, source_path, managed_file_path],
    );
    let _ = conn.execute(
        "UPDATE cases
         SET document_count = (SELECT COUNT(*) FROM documents WHERE case_id = ?1),
             total_pages = (SELECT COALESCE(SUM(page_count), 0) FROM documents WHERE case_id = ?1),
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE case_id = ?1",
        params![case_id],
    );

    Ok(ScanImportResult {
        document_id,
        case_id: case_id.to_string(),
        status: "imported".to_string(),
        message: "Đã import file scan Ricoh vào hồ sơ".to_string(),
    })
}

#[tauri::command]
pub fn import_scanned_batch(
    db: State<'_, DbState>,
    file_paths: Vec<String>,
    case_id: String,
) -> Result<Vec<ScanImportResult>, String> {
    let mut out = Vec::<ScanImportResult>::new();
    for path in file_paths {
        match import_scanned_file(db.clone(), path, case_id.clone()) {
            Ok(result) => out.push(result),
            Err(e) => out.push(ScanImportResult {
                document_id: String::new(),
                case_id: case_id.clone(),
                status: "error".to_string(),
                message: e,
            }),
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn check_scanner_driver_status() -> Result<ScannerDriverStatus, String> {
    let wia_service_status = if cfg!(target_os = "windows") {
        Command::new("sc")
            .args(["query", "stisvc"])
            .output()
            .ok()
            .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
            .and_then(|text| {
                if text.contains("RUNNING") {
                    Some("running".to_string())
                } else if text.contains("STOPPED") {
                    Some("stopped".to_string())
                } else {
                    Some("unknown".to_string())
                }
            })
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "not_windows".to_string()
    };

    Ok(ScannerDriverStatus {
        scan_to_folder_supported: true,
        direct_scan_supported: false,
        wia_service_status,
        recommendation: "Ưu tiên Ricoh Scan to Folder qua SMB. TWAIN/WIA direct scan cần helper native/C# hoặc Python COM và driver Ricoh cụ thể.".to_string(),
    })
}

#[tauri::command]
pub fn start_pipeline_job(
    db: State<'_, DbState>,
    source_type: Option<String>,
) -> Result<PipelineJob, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let job_id = generate_id("pipejob");
    let source = source_type.unwrap_or_else(|| "ricoh_folder".to_string());
    let mode = parse_mode_settings(&conn);

    info!(
        "pipeline start requested job_id={} source_type={} transition=none->created",
        job_id, source
    );
    conn.execute(
        "INSERT INTO pipeline_jobs (
            job_id, source_type, status, total_tasks, completed_tasks, failed_tasks, cancelled_tasks,
            pause_requested, cancel_requested, created_at, started_at, updated_at
         ) VALUES (
            ?1, ?2, 'running', 0, 0, 0, 0,
            0, 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![job_id, source],
    )
    .map_err(|e| {
        error!("pipeline start failed job_id={} error={}", job_id, e);
        format!("PIPELINE_START_FAILED: {e}")
    })?;

    pipeline_event(
        &conn,
        &job_id,
        None,
        "info",
        "job_started",
        "Pipeline job started",
        Some("created"),
        Some("running"),
    );

    enqueue_pipeline_task(
        &conn,
        &job_id,
        "discover",
        Some(
            json!({
                "seed": true,
                "source_type": source,
                "process_mode": mode.process_mode,
            })
            .to_string(),
        ),
    )?;

    load_pipeline_job(&conn, &job_id)
}

#[tauri::command]
pub fn pipeline_execution_tick(
    app: AppHandle,
    db: State<'_, DbState>,
    job_id: String,
) -> Result<PipelineTickResult, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let job = load_pipeline_job(&conn, &job_id)?;

    let worker_pool_size = derive_worker_pool_size();
    let max_inflight = derive_max_inflight(worker_pool_size);

    if job.cancel_requested || job.status == "cancelled" {
        return Ok(PipelineTickResult {
            job_id,
            worker_pool_size,
            max_inflight,
            queue_high_watermark: PIPELINE_QUEUE_HIGH_WATERMARK,
            queued_total: 0,
            running_total: 0,
            claimed_count: 0,
            completed_count: 0,
            failed_count: 0,
            skipped_not_ready_count: 0,
            stopped_reason: "cancel_requested".to_string(),
        });
    }

    if job.pause_requested || job.status == "paused" {
        return Ok(PipelineTickResult {
            job_id,
            worker_pool_size,
            max_inflight,
            queue_high_watermark: PIPELINE_QUEUE_HIGH_WATERMARK,
            queued_total: 0,
            running_total: 0,
            claimed_count: 0,
            completed_count: 0,
            failed_count: 0,
            skipped_not_ready_count: 0,
            stopped_reason: "pause_requested".to_string(),
        });
    }

    let queued_total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND status = 'queued'",
            params![job_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_TICK_COUNT_QUEUED_FAILED: {e}"))?;

    let running_total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND status = 'running'",
            params![job_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_TICK_COUNT_RUNNING_FAILED: {e}"))?;

    if queued_total > PIPELINE_QUEUE_HIGH_WATERMARK {
        pipeline_event_with_meta(
            &conn,
            &job_id,
            None,
            "warn",
            "queue_high_watermark_block",
            "Queue high watermark reached; tick throttled",
            None,
            None,
            Some(json!({ "queued_total": queued_total }).to_string()),
        );
        return Ok(PipelineTickResult {
            job_id,
            worker_pool_size,
            max_inflight,
            queue_high_watermark: PIPELINE_QUEUE_HIGH_WATERMARK,
            queued_total,
            running_total,
            claimed_count: 0,
            completed_count: 0,
            failed_count: 0,
            skipped_not_ready_count: 0,
            stopped_reason: "queue_high_watermark".to_string(),
        });
    }

    if running_total >= max_inflight {
        return Ok(PipelineTickResult {
            job_id,
            worker_pool_size,
            max_inflight,
            queue_high_watermark: PIPELINE_QUEUE_HIGH_WATERMARK,
            queued_total,
            running_total,
            claimed_count: 0,
            completed_count: 0,
            failed_count: 0,
            skipped_not_ready_count: 0,
            stopped_reason: "max_inflight_reached".to_string(),
        });
    }

    let capacity = (max_inflight - running_total).min(worker_pool_size).max(0);
    let mut stmt = conn
        .prepare(
            "SELECT task_id, task_type, status, attempt, payload_json
             FROM pipeline_tasks
             WHERE job_id = ?1 AND status = 'queued'
             ORDER BY created_at ASC
             LIMIT ?2",
        )
        .map_err(|e| format!("PIPELINE_TICK_SELECT_PREPARE_FAILED: {e}"))?;

    let rows = stmt
        .query_map(params![job_id.clone(), capacity], |row| {
            Ok(PipelineTaskRow {
                task_id: row.get(0)?,
                task_type: row.get(1)?,
                status: row.get(2)?,
                attempt: row.get(3)?,
                payload_json: row.get(4)?,
            })
        })
        .map_err(|e| format!("PIPELINE_TICK_SELECT_QUERY_FAILED: {e}"))?;

    let mut tasks = Vec::<PipelineTaskRow>::new();
    for row in rows {
        tasks.push(row.map_err(|e| format!("PIPELINE_TICK_SELECT_ROW_FAILED: {e}"))?);
    }

    let mut claimed_count = 0_i64;
    let mut completed_count = 0_i64;
    let mut failed_count = 0_i64;
    let mut skipped_not_ready_count = 0_i64;

    for task in tasks {
        if load_pipeline_job(&conn, &job_id)?.cancel_requested {
            break;
        }
        if load_pipeline_job(&conn, &job_id)?.pause_requested {
            break;
        }

        let not_before = parse_not_before_ms(&task.payload_json);
        if not_before > now_millis() {
            skipped_not_ready_count += 1;
            continue;
        }

        conn.execute(
            "UPDATE pipeline_tasks
             SET status = 'running',
                 started_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE task_id = ?1 AND status = 'queued'",
            params![task.task_id.clone()],
        )
        .map_err(|e| format!("PIPELINE_TICK_CLAIM_FAILED: {e}"))?;

        claimed_count += 1;
        pipeline_event_with_meta(
            &conn,
            &job_id,
            Some(&task.task_id),
            "info",
            "task_claimed",
            &format!("Task claimed: {}", task.task_type),
            Some("queued"),
            Some("running"),
            Some(json!({ "attempt": task.attempt }).to_string()),
        );

        let mode = parse_mode_settings(&conn);
        if mode.process_mode == "manual"
            && ["import", "ocr", "ai", "persist"].contains(&task.task_type.as_str())
        {
            pipeline_event_with_meta(
                &conn,
                &job_id,
                Some(&task.task_id),
                "info",
                "manual_gate_pause",
                "Manual process mode gate reached; task kept queued",
                Some("queued"),
                Some("queued"),
                Some(
                    json!({"task_type": task.task_type, "process_mode": mode.process_mode})
                        .to_string(),
                ),
            );
            skipped_not_ready_count += 1;
            break;
        }

        match execute_phase_action(Some(&app), &conn, &job_id, &task) {
            Ok(next_phase) => {
                completed_count += 1;
                if let Some(next) = next_phase {
                    enqueue_pipeline_task(
                        &conn,
                        &job_id,
                        &next,
                        Some(json!({ "from_task_id": task.task_id }).to_string()),
                    )?;
                }
            }
            Err(e) => {
                fail_or_retry_task(&conn, &job_id, &task, &e)?;
                failed_count += 1;
            }
        }
    }

    let rem_queued: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND status = 'queued'",
            params![job_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_TICK_REMAINING_QUEUED_FAILED: {e}"))?;
    let rem_running: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND status = 'running'",
            params![job_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_TICK_REMAINING_RUNNING_FAILED: {e}"))?;

    if rem_queued == 0 && rem_running == 0 {
        conn.execute(
            "UPDATE pipeline_jobs
             SET status = CASE WHEN status IN ('running','created') THEN 'completed' ELSE status END,
                 completed_at = CASE WHEN status IN ('running','created') THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now') ELSE completed_at END,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE job_id = ?1",
            params![job_id.clone()],
        )
        .map_err(|e| format!("PIPELINE_TICK_MARK_COMPLETED_FAILED: {e}"))?;
        pipeline_event(
            &conn,
            &job_id,
            None,
            "info",
            "job_completed",
            "Pipeline job reached terminal completed state",
            Some("running"),
            Some("completed"),
        );
    }

    Ok(PipelineTickResult {
        job_id,
        worker_pool_size,
        max_inflight,
        queue_high_watermark: PIPELINE_QUEUE_HIGH_WATERMARK,
        queued_total,
        running_total,
        claimed_count,
        completed_count,
        failed_count,
        skipped_not_ready_count,
        stopped_reason: "ok".to_string(),
    })
}

#[tauri::command]
pub fn bootstrap_pipeline_background(
    app: AppHandle,
    db: State<'_, DbState>,
    job_id: String,
    ticks: Option<i64>,
) -> Result<Vec<PipelineTickResult>, String> {
    let mut out = Vec::new();
    let run_ticks = ticks.unwrap_or(1).clamp(1, 10);
    for _ in 0..run_ticks {
        let tick = pipeline_execution_tick(app.clone(), db.clone(), job_id.clone())?;
        let stop = tick.stopped_reason != "ok";
        out.push(tick);
        if stop {
            break;
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn get_pipeline_job(db: State<'_, DbState>, job_id: String) -> Result<PipelineJob, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    load_pipeline_job(&conn, &job_id).map_err(|e| {
        warn!("pipeline get not found job_id={} error={}", job_id, e);
        e
    })
}

#[tauri::command]
pub fn list_pipeline_jobs(
    db: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<PipelineJobListItem>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let take = limit.unwrap_or(50).clamp(1, 200);
    let mut stmt = conn
        .prepare(
            "SELECT
                job_id, source_type, status, total_tasks, completed_tasks,
                failed_tasks, cancelled_tasks, created_at, updated_at
             FROM pipeline_jobs
             ORDER BY created_at DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("PIPELINE_LIST_PREPARE_FAILED: {e}"))?;

    let rows = stmt
        .query_map(params![take], |row| {
            Ok(PipelineJobListItem {
                job_id: row.get(0)?,
                source_type: row.get(1)?,
                status: row.get(2)?,
                total_tasks: row.get(3)?,
                completed_tasks: row.get(4)?,
                failed_tasks: row.get(5)?,
                cancelled_tasks: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("PIPELINE_LIST_QUERY_FAILED: {e}"))?;

    let mut out = Vec::new();
    for item in rows {
        out.push(item.map_err(|e| format!("PIPELINE_LIST_ROW_FAILED: {e}"))?);
    }
    info!("pipeline list jobs limit={} count={}", take, out.len());
    Ok(out)
}

#[tauri::command]
pub fn pause_pipeline_job(db: State<'_, DbState>, job_id: String) -> Result<PipelineJob, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let before: String = conn
        .query_row(
            "SELECT status FROM pipeline_jobs WHERE job_id = ?1",
            params![job_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_PAUSE_NOT_FOUND: {e}"))?;

    if before != "running" {
        warn!(
            "pipeline pause ignored job_id={} transition={}=>{}",
            job_id, before, before
        );
    }

    conn.execute(
        "UPDATE pipeline_jobs
         SET status = CASE WHEN status = 'running' THEN 'paused' ELSE status END,
             pause_requested = 1,
             paused_at = CASE WHEN status = 'running' THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now') ELSE paused_at END,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE job_id = ?1",
        params![job_id.clone()],
    )
    .map_err(|e| format!("PIPELINE_PAUSE_FAILED: {e}"))?;

    pipeline_event(
        &conn,
        &job_id,
        None,
        "info",
        "job_paused",
        "Pipeline job paused",
        Some(&before),
        Some("paused"),
    );
    info!(
        "pipeline paused job_id={} transition={}=>paused",
        job_id, before
    );
    load_pipeline_job(&conn, &job_id)
}

#[tauri::command]
pub fn resume_pipeline_job(db: State<'_, DbState>, job_id: String) -> Result<PipelineJob, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let before: String = conn
        .query_row(
            "SELECT status FROM pipeline_jobs WHERE job_id = ?1",
            params![job_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_RESUME_NOT_FOUND: {e}"))?;

    if before != "paused" {
        warn!(
            "pipeline resume ignored job_id={} transition={}=>{}",
            job_id, before, before
        );
    }

    conn.execute(
        "UPDATE pipeline_jobs
         SET status = CASE WHEN status = 'paused' THEN 'running' ELSE status END,
             pause_requested = 0,
             resumed_at = CASE WHEN status = 'paused' THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now') ELSE resumed_at END,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE job_id = ?1",
        params![job_id.clone()],
    )
    .map_err(|e| format!("PIPELINE_RESUME_FAILED: {e}"))?;

    pipeline_event(
        &conn,
        &job_id,
        None,
        "info",
        "job_resumed",
        "Pipeline job resumed",
        Some(&before),
        Some("running"),
    );
    info!(
        "pipeline resumed job_id={} transition={}=>running",
        job_id, before
    );
    load_pipeline_job(&conn, &job_id)
}

#[tauri::command]
pub fn cancel_pipeline_job(db: State<'_, DbState>, job_id: String) -> Result<PipelineJob, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let before: String = conn
        .query_row(
            "SELECT status FROM pipeline_jobs WHERE job_id = ?1",
            params![job_id.clone()],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_CANCEL_NOT_FOUND: {e}"))?;

    conn.execute(
        "UPDATE pipeline_jobs
         SET status = CASE
                 WHEN status IN ('completed', 'failed', 'cancelled') THEN status
                 ELSE 'cancelled'
             END,
             cancel_requested = 1,
             cancelled_at = CASE
                 WHEN status IN ('completed', 'failed', 'cancelled') THEN cancelled_at
                 ELSE strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             END,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE job_id = ?1",
        params![job_id.clone()],
    )
    .map_err(|e| format!("PIPELINE_CANCEL_FAILED: {e}"))?;

    pipeline_event(
        &conn,
        &job_id,
        None,
        "warn",
        "job_cancelled",
        "Pipeline job cancelled",
        Some(&before),
        Some("cancelled"),
    );
    warn!(
        "pipeline cancelled job_id={} transition={}=>cancelled",
        job_id, before
    );
    load_pipeline_job(&conn, &job_id)
}

#[tauri::command]
pub fn get_pipeline_footer_status(db: State<'_, DbState>) -> Result<PipelineFooterStatus, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let active_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_jobs WHERE status IN ('created', 'running')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_FOOTER_ACTIVE_FAILED: {e}"))?;
    let paused_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_jobs WHERE status = 'paused'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_FOOTER_PAUSED_FAILED: {e}"))?;
    let failed_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_jobs WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_FOOTER_FAILED_FAILED: {e}"))?;
    let completed_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_jobs WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_FOOTER_COMPLETED_FAILED: {e}"))?;
    let cancelled_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_jobs WHERE status = 'cancelled'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("PIPELINE_FOOTER_CANCELLED_FAILED: {e}"))?;

    info!(
        "pipeline footer status active={} paused={} failed={} completed={} cancelled={}",
        active_jobs, paused_jobs, failed_jobs, completed_jobs, cancelled_jobs
    );

    Ok(PipelineFooterStatus {
        active_jobs,
        paused_jobs,
        failed_jobs,
        completed_jobs,
        cancelled_jobs,
    })
}

#[tauri::command]
pub fn get_pipeline_process_mode_settings(
    db: State<'_, DbState>,
) -> Result<PipelineProcessModeSettings, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    Ok(parse_mode_settings(&conn))
}

#[tauri::command]
pub fn save_pipeline_process_mode_settings(
    db: State<'_, DbState>,
    settings: PipelineProcessModeSettings,
) -> Result<PipelineProcessModeSettings, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    upsert_setting(&conn, "pipeline.process_mode", &settings.process_mode)?;
    upsert_setting(&conn, "pipeline.autoscan", &settings.autoscan.to_string())?;
    upsert_setting(&conn, "pipeline.autosave", &settings.autosave.to_string())?;
    upsert_setting(&conn, "pipeline.autoname", &settings.autoname.to_string())?;
    upsert_setting(
        &conn,
        "pipeline.autosummary",
        &settings.autosummary.to_string(),
    )?;
    upsert_setting(
        &conn,
        "pipeline.autoclassify",
        &settings.autoclassify.to_string(),
    )?;
    Ok(parse_mode_settings(&conn))
}

#[tauri::command]
pub fn get_pipeline_progress_status(
    db: State<'_, DbState>,
    job_id: Option<String>,
) -> Result<Option<PipelineProgressStatus>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let target_job_id = if let Some(id) = job_id {
        id
    } else {
        match conn.query_row(
            "SELECT job_id FROM pipeline_jobs ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        ) {
            Ok(id) => id,
            Err(_) => return Ok(None),
        }
    };

    let job = load_pipeline_job(&conn, &target_job_id)?;
    let running_tasks: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND status = 'running'",
            params![target_job_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let queued_tasks: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND status = 'queued'",
            params![target_job_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let retrying_tasks: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND status = 'queued' AND attempt > 0",
            params![target_job_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let phase: String = conn
        .query_row(
            "SELECT task_type FROM pipeline_tasks WHERE job_id = ?1 AND status IN ('running', 'queued') ORDER BY created_at ASC LIMIT 1",
            params![target_job_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "idle".to_string());

    let total = job.total_tasks.max(0);
    let processed = (job.completed_tasks + job.failed_tasks + job.cancelled_tasks).max(0);
    let progress_percent = if total > 0 {
        (processed as f64 / total as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    Ok(Some(PipelineProgressStatus {
        job_id: job.job_id,
        status: job.status.clone(),
        source_type: job.source_type,
        phase: current_phase_name(&phase),
        processed_tasks: processed,
        total_tasks: total,
        running_tasks,
        queued_tasks,
        failed_tasks: job.failed_tasks,
        retrying_tasks,
        progress_percent,
        eta_seconds: None,
        can_resume: job.status == "paused",
        can_pause: job.status == "running",
        can_cancel: !["completed", "cancelled", "failed"].contains(&job.status.as_str()),
        updated_at: job.updated_at,
        note: job.last_error,
    }))
}

#[tauri::command]
pub fn get_pipeline_integration_check(
    db: State<'_, DbState>,
    job_id: Option<String>,
) -> Result<Option<PipelineIntegrationCheckResult>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let target_job_id = if let Some(id) = job_id {
        id
    } else {
        match conn.query_row(
            "SELECT job_id FROM pipeline_jobs ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        ) {
            Ok(id) => id,
            Err(_) => return Ok(None),
        }
    };

    let discover_completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND task_type = 'discover' AND status = 'completed'",
            params![target_job_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let import_completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND task_type = 'import' AND status = 'completed'",
            params![target_job_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let ocr_reached: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_tasks WHERE job_id = ?1 AND task_type = 'ocr'",
            params![target_job_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut notes = Vec::<String>::new();
    if discover_completed == 0 {
        notes.push("discover phase not completed".to_string());
    }
    if import_completed == 0 {
        notes.push("import phase not completed".to_string());
    }
    if ocr_reached == 0 {
        notes.push("ocr phase not reached".to_string());
    }

    Ok(Some(PipelineIntegrationCheckResult {
        job_id: target_job_id,
        discover_completed: discover_completed > 0,
        import_completed: import_completed > 0,
        ocr_reached: ocr_reached > 0,
        chain_ok: discover_completed > 0 && import_completed > 0 && ocr_reached > 0,
        notes,
    }))
}

#[tauri::command]
pub fn get_pipeline_kpi_summary(db: State<'_, DbState>) -> Result<PipelineKpiSummary, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let total_jobs: i64 = conn
        .query_row("SELECT COUNT(*) FROM pipeline_jobs", [], |row| row.get(0))
        .unwrap_or(0);
    let running_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_jobs WHERE status IN ('created', 'running')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let completed_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_jobs WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let failed_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_jobs WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let cancelled_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pipeline_jobs WHERE status = 'cancelled'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let (total_tasks, completed_tasks, failed_tasks): (i64, i64, i64) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(total_tasks), 0),
                COALESCE(SUM(completed_tasks), 0),
                COALESCE(SUM(failed_tasks), 0)
             FROM pipeline_jobs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap_or((0, 0, 0));
    let completion_rate_percent = if total_tasks > 0 {
        ((completed_tasks as f64) / (total_tasks as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let generated_at: String = conn
        .query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ', 'now')", [], |row| {
            row.get(0)
        })
        .unwrap_or_else(|_| "".to_string());

    Ok(PipelineKpiSummary {
        total_jobs,
        running_jobs,
        completed_jobs,
        failed_jobs,
        cancelled_jobs,
        total_tasks,
        completed_tasks,
        failed_tasks,
        completion_rate_percent,
        generated_at,
    })
}

#[tauri::command]
pub fn get_pipeline_safety_report(db: State<'_, DbState>) -> Result<PipelineSafetyReport, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let blocked_jobs_by_watermark: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT job_id)
             FROM pipeline_events
             WHERE event_type = 'queue_high_watermark_block'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let jobs_with_retry_pressure: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT job_id)
             FROM pipeline_tasks
             WHERE status = 'queued' AND attempt > 0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(PipelineSafetyReport {
        queue_high_watermark: PIPELINE_QUEUE_HIGH_WATERMARK,
        max_worker_pool_cap: PIPELINE_MAX_POOL_CAP,
        max_inflight_cap: PIPELINE_MAX_INFLIGHT_CAP,
        blocked_jobs_by_watermark,
        jobs_with_retry_pressure,
        unresolved_risks: vec![
            "Ready-file claim uses DB-level lock metadata; does not lock filesystem rename semantics."
                .to_string(),
            "OCR runtime latency may still create long tails for oversized PDFs in constrained hosts."
                .to_string(),
        ],
        rollout_guards: vec![
            "Keep process_mode=manual for staged rollout and monitor integration check results before full_auto."
                .to_string(),
            "Track queue_high_watermark_block and retry pressure KPIs before increasing inflight caps."
                .to_string(),
        ],
    })
}
