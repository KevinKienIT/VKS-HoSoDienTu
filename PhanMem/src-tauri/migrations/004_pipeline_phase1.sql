-- ============================================================
-- VKS ECMS — Phase 1 Pipeline Foundation
-- Version: 004
-- Date: 2026-04-28
-- ============================================================

CREATE TABLE IF NOT EXISTS pipeline_jobs (
    job_id               TEXT PRIMARY KEY,
    source_type          TEXT NOT NULL DEFAULT 'ricoh_folder',
    status               TEXT NOT NULL CHECK (status IN (
                            'created',
                            'running',
                            'paused',
                            'completed',
                            'cancelled',
                            'failed'
                         )),
    total_tasks          INTEGER NOT NULL DEFAULT 0,
    completed_tasks      INTEGER NOT NULL DEFAULT 0,
    failed_tasks         INTEGER NOT NULL DEFAULT 0,
    cancelled_tasks      INTEGER NOT NULL DEFAULT 0,
    pause_requested      INTEGER NOT NULL DEFAULT 0,
    cancel_requested     INTEGER NOT NULL DEFAULT 0,
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    started_at           TEXT,
    paused_at            TEXT,
    resumed_at           TEXT,
    completed_at         TEXT,
    cancelled_at         TEXT,
    updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    last_error           TEXT
);

CREATE TABLE IF NOT EXISTS pipeline_tasks (
    task_id              TEXT PRIMARY KEY,
    job_id               TEXT NOT NULL REFERENCES pipeline_jobs(job_id) ON DELETE CASCADE,
    task_type            TEXT NOT NULL,
    status               TEXT NOT NULL CHECK (status IN (
                            'queued',
                            'running',
                            'paused',
                            'completed',
                            'cancelled',
                            'failed'
                         )),
    attempt              INTEGER NOT NULL DEFAULT 0,
    payload_json         TEXT,
    result_json          TEXT,
    error_message        TEXT,
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    started_at           TEXT,
    finished_at          TEXT,
    updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS pipeline_events (
    event_id             TEXT PRIMARY KEY,
    job_id               TEXT NOT NULL REFERENCES pipeline_jobs(job_id) ON DELETE CASCADE,
    task_id              TEXT REFERENCES pipeline_tasks(task_id) ON DELETE SET NULL,
    level                TEXT NOT NULL CHECK (level IN ('info', 'warn', 'error')),
    event_type           TEXT NOT NULL,
    message              TEXT NOT NULL,
    from_status          TEXT,
    to_status            TEXT,
    meta_json            TEXT,
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS pipeline_checkpoints (
    checkpoint_id        TEXT PRIMARY KEY,
    job_id               TEXT NOT NULL REFERENCES pipeline_jobs(job_id) ON DELETE CASCADE,
    checkpoint_key       TEXT NOT NULL,
    checkpoint_value     TEXT,
    created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(job_id, checkpoint_key)
);

CREATE INDEX IF NOT EXISTS idx_pipeline_jobs_status ON pipeline_jobs(status);
CREATE INDEX IF NOT EXISTS idx_pipeline_jobs_updated_at ON pipeline_jobs(updated_at);
CREATE INDEX IF NOT EXISTS idx_pipeline_tasks_job_id ON pipeline_tasks(job_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_tasks_status ON pipeline_tasks(status);
CREATE INDEX IF NOT EXISTS idx_pipeline_events_job_id ON pipeline_events(job_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_events_created_at ON pipeline_events(created_at);
CREATE INDEX IF NOT EXISTS idx_pipeline_checkpoints_job_id ON pipeline_checkpoints(job_id);
