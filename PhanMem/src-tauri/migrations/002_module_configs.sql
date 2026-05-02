-- ============================================================
-- VKS ECMS — Module Configs + App Settings
-- Version: 002
-- Date: 2026-04-26
-- ============================================================

CREATE TABLE IF NOT EXISTS module_configs (
    module_id        TEXT PRIMARY KEY,
    enabled          INTEGER NOT NULL DEFAULT 0,
    selected_version TEXT NOT NULL DEFAULT 'v1',
    settings         TEXT DEFAULT '{}',   -- JSON
    updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS app_settings (
    key              TEXT PRIMARY KEY,
    value            TEXT NOT NULL,
    updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Default module entries
INSERT OR IGNORE INTO module_configs (module_id, enabled, selected_version) VALUES
    ('doc-scanner',    0, 'v2'),
    ('case-group',     0, 'v1'),
    ('image-enhance',  0, 'v1'),
    ('ai-notebook',    0, 'v1'),
    ('doc-viewer',     0, 'v1'),
    ('timeline',       0, 'v1');

-- Default app settings
INSERT OR IGNORE INTO app_settings (key, value) VALUES
    ('theme',         'light'),
    ('font_size',     '14'),
    ('sidebar_width', '260'),
    ('language',      'vi');
