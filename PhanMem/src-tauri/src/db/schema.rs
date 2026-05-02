// VKS ECMS — Schema constants
// Danh sach ten bang va cot de cac module khac tham chieu
// ma khong can hardcode string.

#![allow(dead_code)]

// Ghi chu: file nay dong vai tro "tu dien schema trung tam".
// Cac constant duoc giu san de module moi tham chieu dan,
// nen co the chua duoc dung het o phase hien tai.

/// Ten cac bang trong database.
pub mod tables {
    pub const CASES: &str = "cases";
    pub const DOCUMENTS: &str = "documents";
    pub const PAGES: &str = "pages";
    pub const OCR_RESULTS: &str = "ocr_results";
    pub const REVIEW_QUEUE: &str = "review_queue";
    pub const IMPORT_JOBS: &str = "import_jobs";
    pub const AUDIT_EVENTS: &str = "audit_events";
    pub const ENTITIES: &str = "entities";
    pub const CITATIONS: &str = "citations";
    pub const USERS: &str = "users";
    pub const WORK_PRODUCTS: &str = "work_products";
    pub const CATALOG_ENTRIES: &str = "catalog_entries";
    pub const PAGE_LAYOUT_BLOCKS: &str = "page_layout_blocks";
    pub const DOCUMENT_EXTRACTED_FIELDS: &str = "document_extracted_fields";
}

/// Trang thai hop le cua case.
pub mod case_status {
    pub const ACTIVE: &str = "active";
    pub const ARCHIVED: &str = "archived";
    pub const CLOSED: &str = "closed";
}

/// Trang thai hop le cua document (legacy, dung cho code hien tai).
/// Se duoc thay the boi file_lifecycle khi migration 007 hoan tat.
pub mod document_status {
    pub const PENDING: &str = "pending";
    pub const PROCESSED: &str = "processed";
    pub const REVIEWED: &str = "reviewed";
    pub const ERROR: &str = "error";
}

/// Vong doi file day du — ap dung tu P0-02 / migration 007.
pub mod file_lifecycle {
    pub const IMPORTED: &str = "imported";
    pub const PROCESSING: &str = "processing";
    pub const OCR_DONE: &str = "ocr_done";
    pub const REVIEW_PENDING: &str = "review_pending";
    pub const REVIEWED: &str = "reviewed";
    pub const MANAGED_READY: &str = "managed_ready";
    pub const EXPORTED: &str = "exported";
    pub const ERROR: &str = "error";
}

/// Trang thai transcription cua page/ocr_result/citation.
pub mod transcription_state {
    pub const DIRECT: &str = "direct";
    pub const CANDIDATE_ONLY: &str = "candidate_only";
    pub const INTERPOLATED_PENDING_REVIEW: &str = "interpolated_pending_review";
    pub const APPROVED_MANUAL: &str = "approved_manual";
}

/// Trang thai cua import job.
pub mod import_job_status {
    pub const CREATED: &str = "created";
    pub const SCANNING: &str = "scanning";
    pub const IMPORTING: &str = "importing";
    pub const OCR_PROCESSING: &str = "ocr_processing";
    pub const INDEXING: &str = "indexing";
    pub const PAUSED: &str = "paused";
    pub const FAILED: &str = "failed";
    pub const COMPLETED: &str = "completed";
    pub const CANCELLED: &str = "cancelled";
}

/// Trang thai cua review queue item.
pub mod review_status {
    pub const PENDING: &str = "pending";
    pub const IN_REVIEW: &str = "in_review";
    pub const APPROVED: &str = "approved";
    pub const REJECTED: &str = "rejected";
    pub const DEFERRED: &str = "deferred";
}

/// Loai doi tuong (entity_type).
pub mod entity_type {
    pub const PERSON: &str = "person";
    pub const DEVICE: &str = "device";
    pub const EVIDENCE: &str = "evidence";
    pub const EVENT: &str = "event";
}

/// Trang thai catalog entry.
pub mod catalog_scan_status {
    pub const DISCOVERED: &str = "discovered";
    pub const CATALOGED: &str = "cataloged";
    pub const IMPORTED: &str = "imported";
    pub const SKIPPED: &str = "skipped";
    pub const ERROR: &str = "error";
}
