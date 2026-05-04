-- VKS ECMS — Migration 007: OCR Unicode normalized text
-- BUG-004 (audit): Thêm normalized_text cho tìm kiếm không dấu tiếng Việt
-- Spec: v3/standards/03_pipeline_ocr_ai.md §P0-03

-- Thêm cột normalized_text vào page_layout_blocks
ALTER TABLE page_layout_blocks ADD COLUMN normalized_text TEXT;

-- Thêm cột unicode_form để ghi nhận NFC/NFD
ALTER TABLE page_layout_blocks ADD COLUMN unicode_form TEXT NOT NULL DEFAULT 'NFC';

-- Rebuild FTS index để include normalized_text cho tìm kiếm không dấu
DROP TABLE IF EXISTS fts_layout_blocks;

CREATE VIRTUAL TABLE IF NOT EXISTS fts_layout_blocks USING fts5(
    id UNINDEXED,
    document_id UNINDEXED,
    page_number UNINDEXED,
    block_type,
    text,
    normalized_text,
    content=page_layout_blocks,
    content_rowid=rowid
);
