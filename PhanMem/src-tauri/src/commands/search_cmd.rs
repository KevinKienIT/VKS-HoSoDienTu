// VKS ECMS — Search commands (FTS5)

use crate::commands::module_cmd::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub document_id: String,
    pub case_id: String,
    pub display_name: String,
    pub file_path: String,
    pub document_type: String,
    pub page_number: i32,
    pub created_at: String,
    pub snippet: String,
    pub score: f64,
    pub block_type: Option<String>,
    pub bounding_box: Option<String>,
    pub confidence: Option<f64>,
}

fn normalize_query(value: &str) -> String {
    value.trim().to_string()
}

#[tauri::command]
pub fn fts_search(
    db: State<'_, DbState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    let q = normalize_query(&query);
    if q.is_empty() {
        return Ok(Vec::new());
    }

    let capped_limit = limit.unwrap_or(20).clamp(1, 200);
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let doc_sql = "SELECT
            d.document_id,
            d.case_id,
            d.display_name,
            d.file_path,
            d.document_type,
            1 AS page_number,
            d.created_at,
            COALESCE(
                snippet(fts_documents, 1, '<b>', '</b>', ' … ', 24),
                snippet(fts_documents, 2, '<b>', '</b>', ' … ', 24),
                snippet(fts_documents, 3, '<b>', '</b>', ' … ', 24),
                ''
            ) AS snippet_text,
            bm25(fts_documents) AS rank_score
         FROM fts_documents
         JOIN documents d ON d.rowid = fts_documents.rowid
         WHERE fts_documents MATCH ?1
         ORDER BY rank_score ASC
         LIMIT ?2";

    let mut stmt = conn
        .prepare(doc_sql)
        .map_err(|e| format!("FTS_SEARCH_PREPARE_FAILED: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![q, capped_limit], |row| {
            let rank: f64 = row.get(8)?;
            let score = if rank.is_finite() { -rank } else { 0.0 };
            Ok(SearchResult {
                document_id: row.get(0)?,
                case_id: row.get(1)?,
                display_name: row.get(2)?,
                file_path: row.get(3)?,
                document_type: row.get(4)?,
                page_number: row.get(5)?,
                created_at: row.get(6)?,
                snippet: row.get(7)?,
                score,
                block_type: None,
                bounding_box: None,
                confidence: None,
            })
        })
        .map_err(|e| format!("FTS_SEARCH_QUERY_FAILED: {e}"))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("FTS_SEARCH_ROW_FAILED: {e}"))?);
    }

    let page_sql = "SELECT
            d.document_id,
            d.case_id,
            d.display_name,
            d.file_path,
            d.document_type,
            p.page_index,
            d.created_at,
            COALESCE(snippet(fts_pages, 1, '<b>', '</b>', ' … ', 32), '') AS snippet_text,
            bm25(fts_pages) AS rank_score
         FROM fts_pages
         JOIN pages p ON p.rowid = fts_pages.rowid
         JOIN documents d ON d.document_id = p.document_id
         WHERE fts_pages MATCH ?1
         ORDER BY rank_score ASC
         LIMIT ?2";

    let mut page_stmt = conn
        .prepare(page_sql)
        .map_err(|e| format!("FTS_PAGE_SEARCH_PREPARE_FAILED: {e}"))?;
    let page_rows = page_stmt
        .query_map(rusqlite::params![normalize_query(&query), capped_limit], |row| {
            let rank: f64 = row.get(8)?;
            let score = if rank.is_finite() { -rank + 1000.0 } else { 1000.0 };
            Ok(SearchResult {
                document_id: row.get(0)?,
                case_id: row.get(1)?,
                display_name: row.get(2)?,
                file_path: row.get(3)?,
                document_type: row.get(4)?,
                page_number: row.get(5)?,
                created_at: row.get(6)?,
                snippet: row.get(7)?,
                score,
                block_type: None,
                bounding_box: None,
                confidence: None,
            })
        })
        .map_err(|e| format!("FTS_PAGE_SEARCH_QUERY_FAILED: {e}"))?;

    for row in page_rows {
        out.push(row.map_err(|e| format!("FTS_PAGE_SEARCH_ROW_FAILED: {e}"))?);
    }

    let layout_sql = "SELECT
            d.document_id,
            d.case_id,
            d.display_name,
            d.file_path,
            d.document_type,
            lb.page_number,
            d.created_at,
            COALESCE(snippet(fts_layout_blocks, 4, '<b>', '</b>', ' … ', 32), '') AS snippet_text,
            bm25(fts_layout_blocks) AS rank_score,
            lb.block_type,
            printf('{\"x\":%g,\"y\":%g,\"width\":%g,\"height\":%g}', lb.x, lb.y, lb.width, lb.height) AS bbox,
            lb.confidence
         FROM fts_layout_blocks
         JOIN page_layout_blocks lb ON lb.rowid = fts_layout_blocks.rowid
         JOIN documents d ON d.document_id = lb.document_id
         WHERE fts_layout_blocks MATCH ?1
         ORDER BY rank_score ASC
         LIMIT ?2";

    let mut layout_stmt = conn
        .prepare(layout_sql)
        .map_err(|e| format!("FTS_LAYOUT_SEARCH_PREPARE_FAILED: {e}"))?;
    let layout_rows = layout_stmt
        .query_map(rusqlite::params![normalize_query(&query), capped_limit], |row| {
            let rank: f64 = row.get(8)?;
            let score = if rank.is_finite() { -rank + 2000.0 } else { 2000.0 };
            Ok(SearchResult {
                document_id: row.get(0)?,
                case_id: row.get(1)?,
                display_name: row.get(2)?,
                file_path: row.get(3)?,
                document_type: row.get(4)?,
                page_number: row.get(5)?,
                created_at: row.get(6)?,
                snippet: row.get(7)?,
                score,
                block_type: row.get(9)?,
                bounding_box: row.get(10)?,
                confidence: row.get(11)?,
            })
        })
        .map_err(|e| format!("FTS_LAYOUT_SEARCH_QUERY_FAILED: {e}"))?;
    for row in layout_rows {
        out.push(row.map_err(|e| format!("FTS_LAYOUT_SEARCH_ROW_FAILED: {e}"))?);
    }

    let field_sql = "SELECT
            d.document_id,
            d.case_id,
            d.display_name,
            d.file_path,
            d.document_type,
            f.page_number,
            d.created_at,
            COALESCE(snippet(fts_extracted_fields, 3, '<b>', '</b>', ' … ', 32), '') AS snippet_text,
            bm25(fts_extracted_fields) AS rank_score,
            f.field_name,
            printf('{\"x\":%g,\"y\":%g,\"width\":%g,\"height\":%g}', f.x, f.y, f.width, f.height) AS bbox,
            f.confidence
         FROM fts_extracted_fields
         JOIN document_extracted_fields f ON f.rowid = fts_extracted_fields.rowid
         JOIN documents d ON d.document_id = f.document_id
         WHERE fts_extracted_fields MATCH ?1
         ORDER BY rank_score ASC
         LIMIT ?2";

    let mut field_stmt = conn
        .prepare(field_sql)
        .map_err(|e| format!("FTS_FIELD_SEARCH_PREPARE_FAILED: {e}"))?;
    let field_rows = field_stmt
        .query_map(rusqlite::params![normalize_query(&query), capped_limit], |row| {
            let rank: f64 = row.get(8)?;
            let score = if rank.is_finite() { -rank + 3000.0 } else { 3000.0 };
            Ok(SearchResult {
                document_id: row.get(0)?,
                case_id: row.get(1)?,
                display_name: row.get(2)?,
                file_path: row.get(3)?,
                document_type: row.get(4)?,
                page_number: row.get(5)?,
                created_at: row.get(6)?,
                snippet: row.get(7)?,
                score,
                block_type: row.get(9)?,
                bounding_box: row.get(10)?,
                confidence: row.get(11)?,
            })
        })
        .map_err(|e| format!("FTS_FIELD_SEARCH_QUERY_FAILED: {e}"))?;
    for row in field_rows {
        out.push(row.map_err(|e| format!("FTS_FIELD_SEARCH_ROW_FAILED: {e}"))?);
    }

    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    out.truncate(capped_limit as usize);
    Ok(out)
}
