-- VKS ECMS - OCR layout preservation extension
-- Add formatted OCR text and structured layout blocks while keeping legacy columns.

ALTER TABLE pages ADD COLUMN ocr_formatted_text TEXT;
ALTER TABLE pages ADD COLUMN ocr_layout_blocks TEXT; -- JSON array of block/line geometry

