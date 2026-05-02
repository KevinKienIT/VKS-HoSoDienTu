// VKS ECMS — Review Queue commands
use crate::commands::module_cmd::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

static REVIEW_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewItem {
    pub review_id: String,
    pub case_id: String,
    pub document_id: String,
    pub page_id: Option<String>,
    pub display_name: String,
    pub document_type: String,
    pub review_type: String,
    pub status: String,
    pub confidence: f64,
    pub reviewer_note: String,
    pub created_at: String,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn generate_audit_event_id() -> String {
    let n = REVIEW_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("audit-{}-{}", now_millis(), n)
}

#[tauri::command]
pub fn list_review_queue(
    db: State<'_, DbState>,
    status_filter: Option<String>,
) -> Result<Vec<ReviewItem>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let sql = "SELECT
            rq.review_id,
            COALESCE(d_doc.case_id, d_page.case_id, ''),
            COALESCE(d_doc.document_id, d_page.document_id, ''),
            p.page_id,
            COALESCE(d_doc.display_name, d_page.display_name, rq.object_id),
            COALESCE(d_doc.document_type, d_page.document_type, 'khong_xac_dinh'),
            rq.reason,
            rq.status,
            COALESCE(p.confidence, 0.0),
            COALESCE(rq.reviewer_note, ''),
            rq.created_at
         FROM review_queue rq
         LEFT JOIN documents d_doc
            ON rq.object_type = 'document' AND d_doc.document_id = rq.object_id
         LEFT JOIN pages p
            ON rq.object_type = 'page' AND p.page_id = rq.object_id
         LEFT JOIN documents d_page
            ON p.document_id = d_page.document_id
         WHERE (
            (?1 IS NULL AND rq.status = 'pending')
            OR (?1 IS NOT NULL AND rq.status = ?1)
         )
         ORDER BY rq.created_at DESC";

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("REVIEW_LIST_PREPARE_FAILED: {e}"))?;

    let rows = stmt
        .query_map(params![status_filter], |row| {
            Ok(ReviewItem {
                review_id: row.get(0)?,
                case_id: row.get(1)?,
                document_id: row.get(2)?,
                page_id: row.get(3)?,
                display_name: row.get(4)?,
                document_type: row.get(5)?,
                review_type: row.get(6)?,
                status: row.get(7)?,
                confidence: row.get(8)?,
                reviewer_note: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(|e| format!("REVIEW_LIST_QUERY_FAILED: {e}"))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("REVIEW_LIST_ROW_FAILED: {e}"))?);
    }
    Ok(out)
}

#[tauri::command]
pub fn review_action(
    db: State<'_, DbState>,
    review_id: String,
    action: String,
    reviewer_note: Option<String>,
) -> Result<(), String> {
    let mapped_status = match action.as_str() {
        "approved" => "approved",
        "rejected" => "rejected",
        "skipped" => "deferred",
        _ => return Err(format!("REVIEW_ACTION_INVALID: {action}")),
    };

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM review_queue WHERE review_id = ?1",
            params![review_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("REVIEW_ACTION_CHECK_FAILED: {e}"))?;
    if exists == 0 {
        return Err("REVIEW_ACTION_NOT_FOUND".to_string());
    }

    let note = reviewer_note.unwrap_or_default();
    conn.execute(
        "UPDATE review_queue
         SET status = ?1,
             reviewer_note = ?2,
             resolved_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE review_id = ?3",
        params![mapped_status, note, review_id],
    )
    .map_err(|e| format!("REVIEW_ACTION_UPDATE_FAILED: {e}"))?;

    let action_type = if action == "approved" {
        "approve"
    } else if action == "rejected" {
        "reject"
    } else {
        "update"
    };
    let approval_status = if action == "approved" {
        "approved"
    } else if action == "rejected" {
        "rejected"
    } else {
        "pending_review"
    };

    let payload = json!({
        "action": action,
        "status": mapped_status,
        "note": note,
    })
    .to_string();

    let audit_event_id = generate_audit_event_id();
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
            ?1,
            'review_queue',
            ?2,
            'status',
            ?3,
            NULL,
            ?4,
            'user',
            'desktop-ui',
            'review_action',
            ?5,
            ?6,
            strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         )",
        params![
            audit_event_id,
            review_id,
            action_type,
            mapped_status,
            payload,
            approval_status,
        ],
    );

    Ok(())
}
