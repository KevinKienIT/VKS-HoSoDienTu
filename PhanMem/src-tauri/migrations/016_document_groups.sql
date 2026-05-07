-- ============================================================
-- VKS ECMS - Document Groups / Folder Hierarchy
-- Version: 016
-- Date: 2026-05-07
-- ============================================================

CREATE TABLE IF NOT EXISTS document_groups (
    group_id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
    parent_group_id TEXT REFERENCES document_groups(group_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(case_id, relative_path)
);

CREATE INDEX IF NOT EXISTS idx_document_groups_case_parent_sort
ON document_groups(case_id, parent_group_id, sort_order, name);

CREATE INDEX IF NOT EXISTS idx_document_groups_case_path
ON document_groups(case_id, relative_path);

ALTER TABLE documents ADD COLUMN group_id TEXT REFERENCES document_groups(group_id) ON DELETE SET NULL;
ALTER TABLE documents ADD COLUMN relative_path TEXT;

CREATE INDEX IF NOT EXISTS idx_documents_case_group
ON documents(case_id, group_id);

CREATE INDEX IF NOT EXISTS idx_documents_case_relative_path
ON documents(case_id, relative_path);
