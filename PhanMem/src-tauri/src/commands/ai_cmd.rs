// VKS ECMS — Offline AI assistant commands

use crate::commands::module_cmd::DbState;
use serde::{Deserialize, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiStatus {
    pub available: bool,
    pub provider: String,
    pub mode: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiSource {
    pub source_type: String,
    pub source_id: String,
    pub document_id: Option<String>,
    pub page_number: Option<i32>,
    pub label: String,
    pub excerpt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiAnswer {
    pub mode: String,
    pub answer: String,
    pub sources: Vec<AiSource>,
}

#[derive(Debug, Clone)]
struct CaseContext {
    case_display_name: String,
    case_code: String,
    notes: String,
    document_count: i64,
    page_count: i64,
    sources: Vec<AiSource>,
}

fn short_excerpt(value: &str, max_chars: usize) -> String {
    let text = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= max_chars {
        return text;
    }
    let mut out = text.chars().take(max_chars).collect::<String>();
    out.push_str("...");
    out
}

fn keyword_terms(question: &str) -> Vec<String> {
    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.chars().count() >= 3)
        .take(12)
        .map(ToString::to_string)
        .collect()
}

fn question_focus_bonus(question: &str, source: &AiSource) -> i32 {
    let q = question.to_lowercase();
    let mut bonus = 0;
    if q.contains("tom tat") || q.contains("tóm tắt") {
        if source.source_type == "document" || source.source_type == "case_note" {
            bonus += 2;
        }
    }
    if q.contains("trang") || q.contains("but luc") || q.contains("bút lục") {
        if source.source_type == "page" {
            bonus += 2;
        }
    }
    bonus
}

fn score_text(text: &str, terms: &[String]) -> i32 {
    let lower = text.to_lowercase();
    terms
        .iter()
        .map(|term| if lower.contains(term) { 1 } else { 0 })
        .sum()
}

fn load_case_context(
    conn: &rusqlite::Connection,
    case_id: &str,
    question: Option<&str>,
) -> Result<CaseContext, String> {
    let (case_code, case_display_name): (String, String) = conn
        .query_row(
            "SELECT case_code, case_display_name FROM cases WHERE case_id = ?1",
            [case_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("AI_CASE_QUERY_FAILED: {e}"))?;

    let notes_key = format!("case_notes::{case_id}");
    let notes = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [notes_key],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default();

    let document_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE case_id = ?1",
            [case_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let page_count: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM pages p
             JOIN documents d ON d.document_id = p.document_id
             WHERE d.case_id = ?1",
            [case_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let terms = question.map(keyword_terms).unwrap_or_default();
    let mut sources: Vec<(i32, AiSource)> = Vec::new();

    if !notes.trim().is_empty() {
        sources.push((
            score_text(&notes, &terms) + 2,
            AiSource {
                source_type: "case_note".to_string(),
                source_id: case_id.to_string(),
                document_id: None,
                page_number: None,
                label: "Ghi chú hồ sơ".to_string(),
                excerpt: short_excerpt(&notes, 360),
            },
        ));
    }

    let mut stmt = conn
        .prepare(
            "SELECT document_id, display_name, document_type,
                    COALESCE(summary_short, ''), COALESCE(summary_detail, '')
             FROM documents
             WHERE case_id = ?1
             ORDER BY created_at DESC
             LIMIT 12",
        )
        .map_err(|e| format!("AI_DOC_PREPARE_FAILED: {e}"))?;
    let rows = stmt
        .query_map([case_id], |row| {
            let doc_id: String = row.get(0)?;
            let display_name: String = row.get(1)?;
            let doc_type: String = row.get(2)?;
            let summary_short: String = row.get(3)?;
            let summary_detail: String = row.get(4)?;
            Ok((
                doc_id,
                display_name,
                doc_type,
                summary_short,
                summary_detail,
            ))
        })
        .map_err(|e| format!("AI_DOC_QUERY_FAILED: {e}"))?;
    for row in rows {
        let (doc_id, display_name, doc_type, summary_short, summary_detail) =
            row.map_err(|e| format!("AI_DOC_ROW_FAILED: {e}"))?;
        let text = format!("{display_name} {doc_type} {summary_short} {summary_detail}");
        sources.push((
            score_text(&text, &terms) + 1,
            AiSource {
                source_type: "document".to_string(),
                source_id: doc_id.clone(),
                document_id: Some(doc_id),
                page_number: Some(1),
                label: format!("{display_name} ({doc_type})"),
                excerpt: short_excerpt(&text, 300),
            },
        ));
    }

    let mut page_stmt = conn
        .prepare(
            "SELECT p.page_id, d.document_id, d.display_name, p.page_index, COALESCE(p.ocr_text, '')
             FROM pages p
             JOIN documents d ON d.document_id = p.document_id
             WHERE d.case_id = ?1 AND COALESCE(p.ocr_text, '') <> ''
             ORDER BY p.created_at DESC
             LIMIT 20",
        )
        .map_err(|e| format!("AI_PAGE_PREPARE_FAILED: {e}"))?;
    let page_rows = page_stmt
        .query_map([case_id], |row| {
            let page_id: String = row.get(0)?;
            let document_id: String = row.get(1)?;
            let display_name: String = row.get(2)?;
            let page_index: i32 = row.get(3)?;
            let ocr_text: String = row.get(4)?;
            Ok((page_id, document_id, display_name, page_index, ocr_text))
        })
        .map_err(|e| format!("AI_PAGE_QUERY_FAILED: {e}"))?;
    for row in page_rows {
        let (page_id, document_id, display_name, page_index, ocr_text) =
            row.map_err(|e| format!("AI_PAGE_ROW_FAILED: {e}"))?;
        let score = score_text(&ocr_text, &terms);
        if question.is_none() || score > 0 {
            sources.push((
                score,
                AiSource {
                    source_type: "page".to_string(),
                    source_id: page_id,
                    document_id: Some(document_id),
                    page_number: Some(page_index),
                    label: format!("{display_name} — trang {page_index}"),
                    excerpt: short_excerpt(&ocr_text, 420),
                },
            ));
        }
    }

    if let Some(q) = question {
        for (score, source) in &mut sources {
            *score += question_focus_bonus(q, source);
        }
    }

    sources.sort_by(|a, b| b.0.cmp(&a.0));
    let sources = sources
        .into_iter()
        .filter(|(score, _)| question.is_none() || *score > 0)
        .take(8)
        .map(|(_, source)| source)
        .collect();

    Ok(CaseContext {
        case_display_name,
        case_code,
        notes,
        document_count,
        page_count,
        sources,
    })
}

#[tauri::command]
pub fn ai_check_status() -> Result<AiStatus, String> {
    let addr = ("127.0.0.1", 11434)
        .to_socket_addrs()
        .map_err(|e| format!("AI_STATUS_ADDR_FAILED: {e}"))?
        .next()
        .ok_or_else(|| "AI_STATUS_ADDR_EMPTY".to_string())?;

    let available = TcpStream::connect_timeout(&addr, Duration::from_millis(350)).is_ok();
    Ok(AiStatus {
        available,
        provider: "ollama".to_string(),
        mode: if available {
            "local_llm_available"
        } else {
            "extractive_offline_fallback"
        }
        .to_string(),
        message: if available {
            "Ollama dang lang nghe tren localhost:11434; UI hien dung che do offline san sang."
                .to_string()
        } else {
            "Chua thay Ollama; AI Notebook dang chay Tier 1 extractive offline tu SQLite."
                .to_string()
        },
    })
}

#[tauri::command]
pub fn ai_summarize_case(db: State<'_, DbState>, case_id: String) -> Result<AiAnswer, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let context = load_case_context(&conn, &case_id, None)?;

    let note_line = if context.notes.trim().is_empty() {
        "Chua co ghi chu cua nguoi dung gan voi ho so nay.".to_string()
    } else {
        format!("Ghi chu hien co: {}", short_excerpt(&context.notes, 260))
    };

    let mut answer = format!(
        "Tom tat offline cho ho so {} - {}.\n\n- Tong tai lieu: {}\n- Tong trang OCR da ghi nhan: {}\n- {}\n",
        context.case_code, context.case_display_name, context.document_count, context.page_count, note_line
    );

    if context.sources.is_empty() {
        answer.push_str("\nChua co du OCR/summary de tong hop sau hon. Can chay OCR bridge va rebuild FTS de AI co them ngu canh.");
    } else {
        answer.push_str("\nNguon noi bat:\n");
        for (idx, source) in context.sources.iter().enumerate().take(5) {
            answer.push_str(&format!(
                "{}. {}: {}\n",
                idx + 1,
                source.label,
                source.excerpt
            ));
        }
    }

    Ok(AiAnswer {
        mode: "extractive_offline_sqlite".to_string(),
        answer,
        sources: context.sources,
    })
}

#[tauri::command]
pub fn ai_ask_case(
    db: State<'_, DbState>,
    case_id: String,
    question: String,
) -> Result<AiAnswer, String> {
    let q = question.trim();
    if q.is_empty() {
        return Err("AI_QUESTION_EMPTY".to_string());
    }

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let context = load_case_context(&conn, &case_id, Some(q))?;

    let mut answer = format!("Tra loi offline Tier 1 cho cau hoi: \"{}\".\n\n", q);
    if context.sources.is_empty() {
        answer.push_str(
            "Chua tim thay bang chung truc tiep trong ghi chu, metadata tai lieu hoac OCR text da ingest. Can chay OCR/index hoac bo sung citation/notes truoc khi ket luan.",
        );
    } else {
        answer.push_str("Cac manh bang chung lien quan nhat (uu tien OCR/page trich dan):\n");
        for (idx, source) in context.sources.iter().enumerate().take(6) {
            answer.push_str(&format!(
                "{}. [{}] {}{} — {}\n",
                idx + 1,
                source.source_type,
                source.label,
                source
                    .page_number
                    .map(|p| format!(" (trang {p})"))
                    .unwrap_or_default(),
                source.excerpt
            ));
        }
        answer.push_str("\nTra loi tren la tong hop co dan nguon offline. Neu can ket luan nghiep vu, doi chieu truc tiep tai lieu goc theo trang da neu.");
    }

    Ok(AiAnswer {
        mode: "extractive_offline_sqlite".to_string(),
        answer,
        sources: context.sources,
    })
}
