-- VKS ECMS - Viewer OCR layout blocks and extracted fields
-- Stores positional OCR blocks/regions and normalized metadata extracted from scan pages.

CREATE TABLE IF NOT EXISTS page_layout_blocks (
    id              TEXT PRIMARY KEY,
    document_id     TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    page_id         TEXT NOT NULL REFERENCES pages(page_id) ON DELETE CASCADE,
    page_number     INTEGER NOT NULL,
    block_type      TEXT NOT NULL CHECK (block_type IN (
                        'text',
                        'title',
                        'header',
                        'footer',
                        'stamp',
                        'signature',
                        'but_luc',
                        'document_number',
                        'date',
                        'agency',
                        'person_name',
                        'decision_type',
                        'table',
                        'unknown',
                        'low_confidence',
                        'possible_handwriting',
                        'unreadable_stamp',
                        'unreadable_signature',
                        'blurred_area'
                     )),
    text            TEXT,
    x               REAL NOT NULL DEFAULT 0,
    y               REAL NOT NULL DEFAULT 0,
    width           REAL NOT NULL DEFAULT 0,
    height          REAL NOT NULL DEFAULT 0,
    page_width      REAL,
    page_height     REAL,
    confidence      REAL NOT NULL DEFAULT 0,
    reading_order   INTEGER NOT NULL DEFAULT 0,
    engine          TEXT NOT NULL,
    source          TEXT NOT NULL DEFAULT 'ocr',
    raw_json        TEXT,
    reviewed        INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_layout_doc_page ON page_layout_blocks(document_id, page_number);
CREATE INDEX IF NOT EXISTS idx_layout_page ON page_layout_blocks(page_id);
CREATE INDEX IF NOT EXISTS idx_layout_type ON page_layout_blocks(block_type);
CREATE INDEX IF NOT EXISTS idx_layout_review ON page_layout_blocks(reviewed);

CREATE TABLE IF NOT EXISTS document_extracted_fields (
    id              TEXT PRIMARY KEY,
    document_id     TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    field_name      TEXT NOT NULL,
    field_value     TEXT NOT NULL,
    page_number     INTEGER NOT NULL DEFAULT 1,
    x               REAL NOT NULL DEFAULT 0,
    y               REAL NOT NULL DEFAULT 0,
    width           REAL NOT NULL DEFAULT 0,
    height          REAL NOT NULL DEFAULT 0,
    confidence      REAL NOT NULL DEFAULT 0,
    source          TEXT NOT NULL DEFAULT 'ocr_rule',
    raw_json        TEXT,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_extracted_doc ON document_extracted_fields(document_id);
CREATE INDEX IF NOT EXISTS idx_extracted_name ON document_extracted_fields(field_name);
CREATE INDEX IF NOT EXISTS idx_extracted_doc_name ON document_extracted_fields(document_id, field_name);

CREATE VIRTUAL TABLE IF NOT EXISTS fts_layout_blocks USING fts5(
    id UNINDEXED,
    document_id UNINDEXED,
    page_number UNINDEXED,
    block_type,
    text,
    content=page_layout_blocks,
    content_rowid=rowid
);

CREATE VIRTUAL TABLE IF NOT EXISTS fts_extracted_fields USING fts5(
    id UNINDEXED,
    document_id UNINDEXED,
    field_name,
    field_value,
    source,
    content=document_extracted_fields,
    content_rowid=rowid
);
