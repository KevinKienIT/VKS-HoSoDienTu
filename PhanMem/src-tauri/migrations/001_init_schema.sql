-- ============================================================
-- VKS ECMS — Initial Schema Migration
-- Version: 001
-- Date: 2026-04-25
-- Source specs:
--   V2/01_.../20260424_08_dac_ta_layered_architecture.md §2.1
--   V2/01_.../20260424_09_dac_ta_json_schema_va_state_management.md §1.3-1.10
--   V2/05_.../20260424_14_ban_do_thuoc_tinh_metadata_va_schema_nghiep_vu.md §2
--   V2/01_.../20260423_08_dac_ta_mapping_du_lieu_ho_so.md §4
-- ============================================================

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ============================================================
-- 1. cases
-- ============================================================
CREATE TABLE IF NOT EXISTS cases (
    case_id              TEXT PRIMARY KEY,
    case_code            TEXT NOT NULL UNIQUE,
    case_display_name    TEXT NOT NULL,
    source_folder_name   TEXT NOT NULL,
    case_sequence_no     INTEGER,
    primary_person_name  TEXT NOT NULL,
    primary_person_id    TEXT,
    case_group_label     TEXT,
    case_type            TEXT NOT NULL DEFAULT 'to_dieu_tra',
    investigation_level  TEXT,
    status               TEXT NOT NULL DEFAULT 'active'
                             CHECK (status IN ('active', 'archived', 'closed')),
    severity             TEXT,
    prosecutor_office    TEXT,
    investigator_name    TEXT,
    document_count       INTEGER NOT NULL DEFAULT 0,
    total_pages          INTEGER NOT NULL DEFAULT 0,
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_case_code        ON cases(case_code);
CREATE INDEX IF NOT EXISTS idx_case_status       ON cases(status);
CREATE INDEX IF NOT EXISTS idx_primary_person    ON cases(primary_person_name);

-- ============================================================
-- 2. documents
-- ============================================================
CREATE TABLE IF NOT EXISTS documents (
    document_id              TEXT PRIMARY KEY,
    case_id                  TEXT NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
    original_filename        TEXT NOT NULL,
    import_sequence          INTEGER,
    file_path                TEXT NOT NULL,
    file_hash                TEXT NOT NULL,
    file_size                INTEGER,
    page_count               INTEGER NOT NULL DEFAULT 0,
    display_name             TEXT NOT NULL,
    document_title           TEXT,
    document_type            TEXT NOT NULL DEFAULT 'khong_xac_dinh',
    document_subtype         TEXT,
    issued_date              TEXT,
    issued_by                TEXT,
    received_date            TEXT,
    summary_short            TEXT,
    summary_detail           TEXT,
    ocr_confidence_avg       REAL,
    has_handwriting          INTEGER NOT NULL DEFAULT 0,
    has_seal                 INTEGER NOT NULL DEFAULT 0,
    classification_confidence REAL,
    needs_review             INTEGER NOT NULL DEFAULT 1,
    status                   TEXT NOT NULL DEFAULT 'pending'
                                 CHECK (status IN ('pending', 'processed', 'reviewed', 'error')),
    created_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_doc_case   ON documents(case_id);
CREATE INDEX IF NOT EXISTS idx_doc_type   ON documents(document_type);
CREATE INDEX IF NOT EXISTS idx_doc_date   ON documents(issued_date);
CREATE INDEX IF NOT EXISTS idx_doc_status ON documents(status);
CREATE INDEX IF NOT EXISTS idx_doc_review ON documents(needs_review);

-- ============================================================
-- 3. pages
-- ============================================================
CREATE TABLE IF NOT EXISTS pages (
    page_id              TEXT PRIMARY KEY,
    document_id          TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    page_index           INTEGER NOT NULL CHECK (page_index >= 1),
    image_path           TEXT,
    image_hash           TEXT,
    ocr_text             TEXT,
    ocr_preview          TEXT,
    ocr_revision_id      TEXT,
    but_luc              TEXT,
    confidence           REAL,
    has_handwriting      INTEGER NOT NULL DEFAULT 0,
    handwriting_regions  TEXT,   -- JSON array
    transcription_state  TEXT NOT NULL DEFAULT 'direct'
                             CHECK (transcription_state IN (
                                 'direct',
                                 'candidate_only',
                                 'interpolated_pending_review',
                                 'approved_manual'
                             )),
    uncertain_spans      TEXT,   -- JSON array
    candidate_texts      TEXT,   -- JSON array
    regions              TEXT,   -- JSON array
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_page_doc           ON pages(document_id);
CREATE INDEX IF NOT EXISTS idx_page_transcription  ON pages(transcription_state);
CREATE UNIQUE INDEX IF NOT EXISTS idx_page_doc_idx ON pages(document_id, page_index);

-- ============================================================
-- 4. ocr_results
-- ============================================================
CREATE TABLE IF NOT EXISTS ocr_results (
    ocr_result_id        TEXT PRIMARY KEY,
    page_id              TEXT NOT NULL REFERENCES pages(page_id) ON DELETE CASCADE,
    engine               TEXT NOT NULL DEFAULT 'paddleocr',
    raw_text             TEXT,
    confidence           REAL,
    processing_time_ms   INTEGER,
    region_bbox          TEXT,   -- JSON: [x, y, w, h]
    is_handwriting       INTEGER NOT NULL DEFAULT 0,
    candidate_texts      TEXT,   -- JSON array of alternative readings
    transcription_state  TEXT NOT NULL DEFAULT 'direct'
                             CHECK (transcription_state IN (
                                 'direct',
                                 'candidate_only',
                                 'interpolated_pending_review',
                                 'approved_manual'
                             )),
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_ocr_page   ON ocr_results(page_id);
CREATE INDEX IF NOT EXISTS idx_ocr_engine ON ocr_results(engine);

-- ============================================================
-- 5. review_queue
-- ============================================================
CREATE TABLE IF NOT EXISTS review_queue (
    review_id            TEXT PRIMARY KEY,
    object_type          TEXT NOT NULL
                             CHECK (object_type IN ('page', 'document', 'ocr_result', 'citation')),
    object_id            TEXT NOT NULL,
    reason               TEXT NOT NULL,
    priority             INTEGER NOT NULL DEFAULT 0,
    status               TEXT NOT NULL DEFAULT 'pending'
                             CHECK (status IN ('pending', 'in_review', 'approved', 'rejected', 'deferred')),
    reviewer_id          TEXT,
    reviewer_note        TEXT,
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    resolved_at          TEXT
);

CREATE INDEX IF NOT EXISTS idx_review_status ON review_queue(status);
CREATE INDEX IF NOT EXISTS idx_review_object ON review_queue(object_type, object_id);

-- ============================================================
-- 6. import_jobs
-- ============================================================
CREATE TABLE IF NOT EXISTS import_jobs (
    import_job_id        TEXT PRIMARY KEY,
    case_id              TEXT NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
    source_folder        TEXT NOT NULL,
    job_status           TEXT NOT NULL DEFAULT 'created'
                             CHECK (job_status IN (
                                 'created', 'scanning', 'importing',
                                 'ocr_processing', 'indexing',
                                 'paused', 'failed', 'completed', 'cancelled'
                             )),
    total_files          INTEGER NOT NULL DEFAULT 0,
    processed_files      INTEGER NOT NULL DEFAULT 0,
    failed_files         INTEGER NOT NULL DEFAULT 0,
    current_file         TEXT,
    last_success_document_id TEXT,
    error_log            TEXT,   -- JSON array of error entries
    started_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    completed_at         TEXT
);

CREATE INDEX IF NOT EXISTS idx_import_case   ON import_jobs(case_id);
CREATE INDEX IF NOT EXISTS idx_import_status ON import_jobs(job_status);

-- ============================================================
-- 7. audit_events
-- ============================================================
CREATE TABLE IF NOT EXISTS audit_events (
    audit_event_id       TEXT PRIMARY KEY,
    object_type          TEXT NOT NULL,
    object_id            TEXT NOT NULL,
    field_name           TEXT,
    action_type          TEXT NOT NULL
                             CHECK (action_type IN (
                                 'create', 'update', 'override', 'merge',
                                 'split', 'restore', 'approve', 'reject'
                             )),
    before_value         TEXT,
    after_value          TEXT,
    actor_type           TEXT NOT NULL
                             CHECK (actor_type IN ('user', 'agent', 'system', 'ai')),
    actor_id             TEXT,
    reason_code          TEXT,
    reason_note          TEXT,
    approval_status      TEXT NOT NULL DEFAULT 'not_required'
                             CHECK (approval_status IN (
                                 'not_required', 'pending_review', 'approved', 'rejected'
                             )),
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_object  ON audit_events(object_type, object_id);
CREATE INDEX IF NOT EXISTS idx_audit_action  ON audit_events(action_type);
CREATE INDEX IF NOT EXISTS idx_audit_actor   ON audit_events(actor_type, actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_events(created_at);

-- ============================================================
-- 8. entities (person, device, evidence, event)
-- ============================================================
CREATE TABLE IF NOT EXISTS entities (
    entity_id            TEXT PRIMARY KEY,
    entity_type          TEXT NOT NULL
                             CHECK (entity_type IN ('person', 'device', 'evidence', 'event')),
    name                 TEXT NOT NULL,
    alias                TEXT,
    description          TEXT,
    entity_role          TEXT,
    identification       TEXT,   -- JSON
    metadata             TEXT,   -- JSON
    linked_documents     TEXT,   -- JSON array of document_ids
    linked_pages         TEXT,   -- JSON array of page_ids
    linked_entities      TEXT,   -- JSON array of entity_ids
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at           TEXT
);

CREATE INDEX IF NOT EXISTS idx_entity_type ON entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entity_name ON entities(name);

-- ============================================================
-- 9. citations
-- ============================================================
CREATE TABLE IF NOT EXISTS citations (
    citation_id          TEXT PRIMARY KEY,
    source_document_id   TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    source_page_id       TEXT NOT NULL REFERENCES pages(page_id) ON DELETE CASCADE,
    source_but_luc       TEXT,
    ocr_revision_id      TEXT,
    citation_anchor_id   TEXT,
    quote_excerpt        TEXT NOT NULL,
    quote_normalized     TEXT,
    context_before       TEXT,
    context_after        TEXT,
    confidence           REAL NOT NULL CHECK (confidence BETWEEN 0 AND 1),
    transcription_state  TEXT NOT NULL DEFAULT 'direct'
                             CHECK (transcription_state IN (
                                 'direct',
                                 'candidate_only',
                                 'interpolated_pending_review',
                                 'approved_manual'
                             )),
    entity_links         TEXT,   -- JSON array of entity_ids
    citation_type        TEXT,
    page_reference       TEXT,
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_cite_doc    ON citations(source_document_id);
CREATE INDEX IF NOT EXISTS idx_cite_page   ON citations(source_page_id);

-- ============================================================
-- 10. users
-- ============================================================
CREATE TABLE IF NOT EXISTS users (
    user_id              TEXT PRIMARY KEY,
    username             TEXT NOT NULL UNIQUE,
    display_name         TEXT NOT NULL,
    email                TEXT,
    role                 TEXT NOT NULL DEFAULT 'ksv'
                             CHECK (role IN ('admin', 'ksv', 'viewer')),
    permission_level     INTEGER NOT NULL DEFAULT 1,
    permissions          TEXT,   -- JSON array
    office               TEXT,
    status               TEXT NOT NULL DEFAULT 'active'
                             CHECK (status IN ('active', 'inactive', 'locked')),
    preferences          TEXT,   -- JSON
    last_login           TEXT,
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at           TEXT
);

-- ============================================================
-- 11. work_products
-- ============================================================
CREATE TABLE IF NOT EXISTS work_products (
    work_product_id      TEXT PRIMARY KEY,
    case_id              TEXT NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
    owner_actor_id       TEXT,
    work_product_type    TEXT NOT NULL
                             CHECK (work_product_type IN (
                                 'reading_queue', 'bookmark', 'note', 'question',
                                 'issue', 'contradiction', 'timeline',
                                 'dossier_draft', 'report_draft'
                             )),
    status               TEXT NOT NULL DEFAULT 'draft'
                             CHECK (status IN ('draft', 'in_review', 'approved', 'rejected', 'archived')),
    title                TEXT NOT NULL,
    body                 TEXT,
    linked_citation_ids  TEXT,   -- JSON array
    linked_entity_ids    TEXT,   -- JSON array
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_wp_case ON work_products(case_id);
CREATE INDEX IF NOT EXISTS idx_wp_type ON work_products(work_product_type);

-- ============================================================
-- 12. catalog_entries (metadata nhe — Agent B addition)
-- Dung de quet nhanh corpus truoc khi import day du.
-- ============================================================
CREATE TABLE IF NOT EXISTS catalog_entries (
    catalog_entry_id     TEXT PRIMARY KEY,
    file_path            TEXT NOT NULL UNIQUE,
    file_name            TEXT NOT NULL,
    file_ext             TEXT NOT NULL,
    file_size            INTEGER NOT NULL,
    modified_at          TEXT NOT NULL,
    file_hash            TEXT,
    parent_folder        TEXT NOT NULL,
    case_id              TEXT REFERENCES cases(case_id),
    scan_status          TEXT NOT NULL DEFAULT 'discovered'
                             CHECK (scan_status IN ('discovered', 'cataloged', 'imported', 'skipped', 'error')),
    scan_note            TEXT,
    scanned_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_catalog_path   ON catalog_entries(file_path);
CREATE INDEX IF NOT EXISTS idx_catalog_folder ON catalog_entries(parent_folder);
CREATE INDEX IF NOT EXISTS idx_catalog_status ON catalog_entries(scan_status);
CREATE INDEX IF NOT EXISTS idx_catalog_case   ON catalog_entries(case_id);

-- ============================================================
-- FTS5 virtual tables for full-text search
-- ============================================================
CREATE VIRTUAL TABLE IF NOT EXISTS fts_documents USING fts5(
    document_id UNINDEXED,
    display_name,
    document_title,
    summary_short,
    summary_detail,
    content=documents,
    content_rowid=rowid
);

CREATE VIRTUAL TABLE IF NOT EXISTS fts_pages USING fts5(
    page_id UNINDEXED,
    ocr_text,
    but_luc,
    content=pages,
    content_rowid=rowid
);

CREATE VIRTUAL TABLE IF NOT EXISTS fts_entities USING fts5(
    entity_id UNINDEXED,
    name,
    description,
    content=entities,
    content_rowid=rowid
);
