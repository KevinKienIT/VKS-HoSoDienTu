-- ============================================================
-- VKS ECMS — Ricoh Scan Inbox
-- Version: 003
-- Date: 2026-04-27
-- ============================================================

CREATE TABLE IF NOT EXISTS scan_jobs (
    scan_job_id        TEXT PRIMARY KEY,
    source_type        TEXT NOT NULL DEFAULT 'ricoh_folder'
                           CHECK (source_type IN ('ricoh_folder', 'manual_import', 'direct_scanner')),
    scanner_model      TEXT,
    original_scan_path TEXT NOT NULL,
    file_hash          TEXT,
    case_id            TEXT REFERENCES cases(case_id),
    document_id        TEXT REFERENCES documents(document_id),
    scan_import_status TEXT NOT NULL DEFAULT 'detected'
                           CHECK (scan_import_status IN (
                               'detected', 'waiting_for_stable', 'ready',
                               'importing', 'imported', 'duplicate', 'error'
                           )),
    detected_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    imported_at        TEXT,
    error_message      TEXT
);

CREATE INDEX IF NOT EXISTS idx_scan_jobs_status ON scan_jobs(scan_import_status);
CREATE INDEX IF NOT EXISTS idx_scan_jobs_case ON scan_jobs(case_id);
CREATE INDEX IF NOT EXISTS idx_scan_jobs_path ON scan_jobs(original_scan_path);

INSERT OR IGNORE INTO app_settings (key, value) VALUES
    ('scan.ricoh.inbox_folder', ''),
    ('scan.ricoh.import_mode', 'ask'),
    ('scan.ricoh.default_case_id', ''),
    ('scan.ricoh.accept_pdf', 'true'),
    ('scan.ricoh.accept_tiff', 'true'),
    ('scan.ricoh.accept_jpg', 'true'),
    ('scan.ricoh.accept_png', 'true'),
    ('scan.ricoh.stable_wait_ms', '2500'),
    ('scan.ricoh.duplicate_detection', 'true'),
    ('scan.ricoh.ocr_after_scan', 'true'),
    ('scan.ricoh.classify_after_scan', 'true');
