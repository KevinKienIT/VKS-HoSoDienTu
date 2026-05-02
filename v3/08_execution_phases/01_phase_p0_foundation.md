# 01 PHASE P0 FOUNDATION — Chi tiết triển khai

> Phase đang ACTIVE. Khóa nền chạy thật trước khi làm bất kỳ tính năng nâng cao nào.

---

## P0-01 — Startup Environment Gate

### Mục tiêu
Chặn app vào Dashboard khi thiếu runtime tối thiểu. Hiện tại `main.rs` chạy thẳng vào UI mà không check gì.

### Đối chiếu code hiện tại

| Thành phần | Hiện trạng | Cần làm |
|-----------|-----------|---------|
| `main.rs:154-171` | Chỉ init DB + storage dirs | Thêm `run_startup_self_check` command |
| `commands/mod.rs` | 10 module, chưa có `system_cmd` | Thêm `pub mod system_cmd` |
| `App.tsx:280-284` | Check `__TAURI_INTERNALS__` chỉ set `online` flag | Gọi self-check trước khi render routes |
| Frontend services | Không có `systemService.ts` | Tạo mới |

### File cần tạo/sửa

**1. Tạo `PhanMem/src-tauri/src/commands/system_cmd.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub severity: String, // "fatal" | "warning" | "info"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartupCheckResult {
    pub status: String, // "pass" | "warning" | "fail"
    pub checks: Vec<EnvCheck>,
    pub fatal_errors: Vec<String>,
}

#[tauri::command]
pub fn run_startup_self_check() -> Result<StartupCheckResult, String> {
    let mut checks = Vec::new();
    let mut fatal = Vec::new();

    // Check 1: RAM >= 4GB
    let ram_gb = sysinfo::System::new_all().total_memory() / 1_073_741_824;
    let ram_ok = ram_gb >= 4;
    checks.push(EnvCheck {
        name: "RAM".into(),
        passed: ram_ok,
        message: format!("{ram_gb} GB"),
        severity: if ram_ok { "info" } else { "fatal" }.into(),
    });
    if !ram_ok { fatal.push("RAM < 4GB".into()); }

    // Check 2: Workspace writable
    let storage_ok = crate::storage::ensure_managed_dirs().is_ok();
    checks.push(EnvCheck {
        name: "Storage".into(),
        passed: storage_ok,
        message: if storage_ok { "Writable" } else { "Permission denied" }.into(),
        severity: if storage_ok { "info" } else { "fatal" }.into(),
    });
    if !storage_ok { fatal.push("Storage not writable".into()); }

    // Check 3: SQLite OK (DB đã init trước đó trong setup)
    checks.push(EnvCheck {
        name: "SQLite".into(), passed: true,
        message: "OK".into(), severity: "info".into(),
    });

    // Check 4: OCR models present
    let ocr_model_path = crate::storage::managed_root()
        .map(|r| r.join("models").join("ocr"));
    let ocr_ok = ocr_model_path.as_ref().map(|p| p.exists()).unwrap_or(false);
    checks.push(EnvCheck {
        name: "OCR Models".into(),
        passed: ocr_ok,
        message: if ocr_ok { "Found" } else { "Missing — OCR sẽ không hoạt động" }.into(),
        severity: if ocr_ok { "info" } else { "warning" }.into(),
    });

    let status = if !fatal.is_empty() { "fail" }
                 else if checks.iter().any(|c| !c.passed) { "warning" }
                 else { "pass" };

    Ok(StartupCheckResult {
        status: status.into(), checks, fatal_errors: fatal,
    })
}
```

**2. Đăng ký trong `commands/mod.rs`**
```rust
pub mod system_cmd;  // thêm dòng này
```

**3. Đăng ký trong `main.rs` invoke_handler**
```rust
commands::system_cmd::run_startup_self_check,  // thêm vào generate_handler!
```

**4. Tạo `PhanMem/src/services/systemService.ts`**
```typescript
import { invoke } from "@tauri-apps/api/core";

export interface EnvCheck {
  name: string;
  passed: boolean;
  message: string;
  severity: "fatal" | "warning" | "info";
}

export interface StartupCheckResult {
  status: "pass" | "warning" | "fail";
  checks: EnvCheck[];
  fatal_errors: string[];
}

export async function runStartupSelfCheck(): Promise<StartupCheckResult> {
  try {
    return await invoke<StartupCheckResult>("run_startup_self_check");
  } catch {
    return { status: "fail", checks: [], fatal_errors: ["Invoke failed"] };
  }
}
```

**5. UI: Tạo `StartupGatePage.tsx`**

```
┌─────────────────────────────────────────────┐
│  ⬡ VKS ECMS — Kiểm tra hệ thống           │
├─────────────────────────────────────────────┤
│                                             │
│  ✅ RAM: 16 GB                              │
│  ✅ Storage: Writable                       │
│  ✅ SQLite: OK                              │
│  ⚠️  OCR Models: Missing                    │
│                                             │
│  Trạng thái: ⚠️ WARNING                     │
│  OCR sẽ không hoạt động cho đến khi cài    │
│  model PaddleOCR vào VKS_ECMS_Data/models  │
│                                             │
│  [ Tiếp tục vào Dashboard ]  [ Copy Log ]   │
│  (nếu FAIL → nút Tiếp tục bị disable)      │
└─────────────────────────────────────────────┘
```

**6. Tích hợp vào `App.tsx`**
```typescript
// Trong App(), thêm state:
const [gateResult, setGateResult] = useState<StartupCheckResult | null>(null);

useEffect(() => {
  if (hasTauriRuntime) {
    runStartupSelfCheck().then(setGateResult);
  }
}, []);

// Trong AppShell render:
if (gateResult && gateResult.status === "fail") {
  return <StartupGatePage result={gateResult} />;
}
```

### Dependency cần thêm
```toml
# Cargo.toml — thêm sysinfo
[dependencies]
sysinfo = "0.30"
```

### Test bắt buộc
- `cargo check` PASS sau khi thêm `system_cmd.rs`
- `npm run tauri:dev` → thấy gate page trước Dashboard
- Giả lập fail: đổi ngưỡng RAM thành 999GB → gate chặn

---

## P0-02 — File Lifecycle

### Mục tiêu
Tách rõ 5 thư mục per-case: `original → processing → reviewed → managed → exports`. Không bao giờ ghi đè file gốc.

### Đối chiếu code hiện tại

| File | Hiện trạng | Cần làm |
|------|-----------|---------|
| `storage.rs:22-32` | Có `originals_dir()`, `processed_dir()`, `exports_dir()` global | Thêm per-case dirs |
| `storage.rs:71-96` | `copy_to_originals()` copy vào `originals/<case>/` | OK, giữ nguyên |
| `import_cmd.rs` | Import → copy to originals → insert DB | Thêm `file_status = 'imported'` |
| `schema.rs:35-40` | `document_status`: pending/processed/reviewed/error | Mở rộng thêm lifecycle states |

### Sửa `storage.rs` — thêm per-case lifecycle dirs

```rust
pub fn case_dir(case_code: &str, sub: &str) -> Result<PathBuf, String> {
    let dir = managed_root()?.join(sub).join(safe_component(case_code, "case"));
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("STORAGE_CASE_DIR_FAILED:{}:{e}", dir.display()))?;
    Ok(dir)
}

pub fn case_originals_dir(case_code: &str) -> Result<PathBuf, String> {
    case_dir(case_code, "originals")
}
pub fn case_processing_dir(case_code: &str) -> Result<PathBuf, String> {
    case_dir(case_code, "processing")
}
pub fn case_reviewed_dir(case_code: &str) -> Result<PathBuf, String> {
    case_dir(case_code, "reviewed")
}
pub fn case_managed_dir(case_code: &str) -> Result<PathBuf, String> {
    case_dir(case_code, "managed")
}
pub fn case_exports_dir(case_code: &str) -> Result<PathBuf, String> {
    case_dir(case_code, "exports")
}
```

### Migration `007_file_lifecycle.sql`

```sql
ALTER TABLE documents ADD COLUMN file_status TEXT NOT NULL DEFAULT 'imported'
  CHECK (file_status IN (
    'imported','processing','ocr_done','review_pending',
    'reviewed','managed_ready','exported'
  ));
ALTER TABLE documents ADD COLUMN original_path TEXT;
ALTER TABLE documents ADD COLUMN managed_path TEXT;
ALTER TABLE documents ADD COLUMN revision_no INTEGER NOT NULL DEFAULT 1;
```

### Sửa `schema.rs` — thêm constants

```rust
pub mod file_lifecycle {
    pub const IMPORTED: &str = "imported";
    pub const PROCESSING: &str = "processing";
    pub const OCR_DONE: &str = "ocr_done";
    pub const REVIEW_PENDING: &str = "review_pending";
    pub const REVIEWED: &str = "reviewed";
    pub const MANAGED_READY: &str = "managed_ready";
    pub const EXPORTED: &str = "exported";
}
```

### Luồng file lifecycle

```
Import PDF
  ↓
copy_to_originals() → originals/<case>/<doc_id>_file.pdf
  ↓  file_status = 'imported'
OCR pipeline chạy
  ↓  file_status = 'processing' → 'ocr_done'
User review / sửa OCR text
  ↓  file_status = 'review_pending' → 'reviewed'
  ↓  tạo revision mới (revision_no + 1), KHÔNG ghi đè original
Kiểm sát viên duyệt
  ↓  file_status = 'managed_ready'
  ↓  copy to managed/<case>/
Export
  ↓  file_status = 'exported'
  ↓  copy to exports/<case>/
```

### Test bắt buộc
- Import file → check `originals/` có file, `file_status = 'imported'`
- Chạy OCR → `file_status` chuyển sang `'ocr_done'`
- Export chỉ nhận `managed_ready` → reject nếu file chưa review

---

## P0-03 — OCR Block Schema Unicode

### Mục tiêu
Chuẩn hóa output OCR: lưu cả `raw_text` (dấu tiếng Việt) và `normalized_text` (không dấu, tìm kiếm).

### Đối chiếu code hiện tại

| File | Hiện trạng | Cần làm |
|------|-----------|---------|
| `ocr_pipeline.py` | Có `_normalize_text()`, `_classify_block()`, `_extract_fields()` | Thêm `normalized_text` vào mỗi block output |
| Migration 006 | Bảng `page_layout_blocks` có `text` column | Thêm `normalized_text` column |
| `schema.rs` | Chưa có constants cho `page_layout_blocks` | Thêm table/column names |

### Sửa `ocr_pipeline.py` — thêm normalized_text vào output block

```python
# Trong _build_layout(), mỗi block trả về thêm:
block = {
    "block_type": cls,
    "text": raw_text,                          # giữ nguyên dấu
    "normalized_text": _normalize_text(raw_text), # không dấu
    "unicode_form": "NFC",
    "x": x, "y": y, "width": w, "height": h,
    "confidence": conf,
    "reading_order": order,
    "engine": "rapidocr",
}
```

### Migration `008_ocr_normalized_text.sql`

```sql
ALTER TABLE page_layout_blocks
  ADD COLUMN normalized_text TEXT;

-- Rebuild FTS để index normalized_text
DROP TABLE IF EXISTS fts_layout_blocks;
CREATE VIRTUAL TABLE fts_layout_blocks USING fts5(
    id UNINDEXED,
    document_id UNINDEXED,
    page_number UNINDEXED,
    block_type,
    text,
    normalized_text,
    content=page_layout_blocks,
    content_rowid=rowid
);
```

### Sửa `schema.rs`

```rust
pub mod tables {
    // ... existing ...
    pub const PAGE_LAYOUT_BLOCKS: &str = "page_layout_blocks";
    pub const DOCUMENT_EXTRACTED_FIELDS: &str = "document_extracted_fields";
}
```

### Test bắt buộc
- OCR tài liệu tiếng Việt → `raw_text` giữ dấu: "Quyết định khởi tố"
- `normalized_text` = "quyet dinh khoi to"
- FTS search "quyet dinh" → tìm được document trên

---

## P0-04 — Citation Validator

### Mục tiêu
Mọi kết luận AI phải kèm citation (document_id, page, quote). Thiếu → `review_required`.

### Đối chiếu code hiện tại

| File | Hiện trạng | Cần làm |
|------|-----------|---------|
| `ai_cmd.rs:280-314` | `ai_summarize_case()` trả `AiAnswer { sources }` | Validate sources không rỗng |
| `ai_cmd.rs:317-359` | `ai_ask_case()` trả sources theo score | Thêm citation check |
| `AiSource` struct | Có `document_id`, `page_number`, `excerpt` | OK, đủ fields |

### Sửa `ai_cmd.rs` — thêm validation function

```rust
fn validate_citations(answer: &mut AiAnswer) {
    if answer.sources.is_empty() {
        answer.mode = format!("{}_NO_CITATION", answer.mode);
        answer.answer = format!(
            "⚠️ CẢNH BÁO: Kết luận chưa có trích dẫn nguồn.\n\
             Cần bổ sung citation trước khi sử dụng.\n\n{}",
            answer.answer
        );
    }
    // Check mỗi source phải có document_id + page_number
    let valid_count = answer.sources.iter()
        .filter(|s| s.document_id.is_some() && s.page_number.is_some())
        .count();
    if valid_count == 0 && !answer.sources.is_empty() {
        answer.answer = format!(
            "⚠️ Nguồn trích dẫn thiếu document_id/page_number.\n{}",
            answer.answer
        );
    }
}
```

Gọi `validate_citations(&mut result)` trước `Ok(result)` trong cả `ai_summarize_case` và `ai_ask_case`.

### Test bắt buộc
- Case rỗng (0 documents) → output có badge `⚠️ CẢNH BÁO`
- Case có documents → sources kèm `document_id` + `page_number`

---

## P0-05 — Export Package Chuẩn

### Mục tiêu
Export sinh cấu trúc bàn giao đầy đủ, không chỉ PDF đơn.

### Đối chiếu code hiện tại

| File | Hiện trạng | Cần làm |
|------|-----------|---------|
| `export_cmd.rs` | Chỉ `export_pdf_bundle` tạo 1 file PDF | Thêm `export_dossier_package` |
| `ExportPage.tsx` | UI chọn docs + export PDF | Thêm option "Gói bàn giao" |

### Cấu trúc export đích (thống nhất với standards/04_file_lifecycle_export.md)

```
exports/<EXPORT_JOB_ID>_<TIMESTAMP>/
├── 00_index_trich_dan.docx     ← mục lục trích dẫn tài liệu
├── 01_bao_cao_tong_hop.docx    ← báo cáo tổng hợp vụ án
├── 02_so_do_vu_an.html         ← sơ đồ quan hệ/timeline
├── 03_tai_lieu_quan_ly/        ← file gốc managed per document
│   ├── doc1_quyet_dinh.pdf
│   └── doc2_bien_ban.pdf
├── 04_phu_luc_citation.json    ← extracted fields + sources
└── 05_audit_export.json        ← pipeline events + revision log
```

### Sửa `export_cmd.rs` — thêm command mới

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct DossierPackageInput {
    pub case_id: String,
    pub output_dir: String,
    pub include_originals: bool,
    pub include_ocr_text: bool,
    pub include_audit: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DossierPackageResult {
    pub output_dir: String,
    pub file_count: usize,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub fn export_dossier_package(
    db: State<'_, DbState>,
    input: DossierPackageInput,
) -> Result<DossierPackageResult, String> {
    // 1. Tạo thư mục export
    // 2. Copy originals vào 02_tai_lieu/
    // 3. Dump OCR text vào 03_ocr_text/
    // 4. Build citation.json từ document_extracted_fields
    // 5. Build audit.json từ pipeline logs
    // 6. Sinh 00_muc_luc.html
    todo!("Implement full dossier export")
}
```

### UI Map — Export Page cập nhật

```
┌─────────────────────────────────────────────┐
│  Export Hồ Sơ                               │
├─────────────────────────────────────────────┤
│  Chọn hồ sơ: [Dropdown case]               │
│                                             │
│  ☐ Export PDF gộp (hiện có)                 │
│  ☑ Export Gói bàn giao đầy đủ              │
│    ├ ☑ File gốc                             │
│    ├ ☑ OCR text                             │
│    └ ☑ Audit trail                          │
│                                             │
│  ⚠️ 2 tài liệu chưa review                │
│                                             │
│  [ Chọn thư mục ]  [ Export ]               │
└─────────────────────────────────────────────┘
```

### Test bắt buộc
- Export case có 3 docs → thư mục output có đủ 6 items
- Export case có doc `review_pending` → warning hiển thị

---

## Thứ tự triển khai khuyến nghị

| Bước | Task | Lý do ưu tiên |
|------|------|---------------|
| 1 | P0-03 OCR schema | Gần xong 70%, giải phóng search/citation |
| 2 | P0-01 Startup Gate | Nền tảng an toàn, chặn lỗi sớm |
| 3 | P0-02 File lifecycle | Cần migration + storage refactor |
| 4 | P0-04 Citation validator | Nhỏ, sửa ít code |
| 5 | P0-05 Export package | Phức tạp nhất, cần P0-02 + P0-03 xong |
