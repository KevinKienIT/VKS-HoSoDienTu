# NEXT ACTION — Bản đồ điều hướng toàn dự án VKS ECMS v3

> Cập nhật: 2026-04-29 · Đây là file duy nhất agent đọc đầu tiên khi mở dự án.

---

## ⛔ RÀNG BUỘC TUYỆT ĐỐI

| # | Quy tắc | Lý do |
|---|---------|-------|
| 1 | **Chỉ test bằng `npm run tauri:dev`** | Tauri API không hoạt động trong browser thuần |
| 2 | **Không dùng `localStorage`/`fetch` cho dữ liệu nghiệp vụ** | Chỉ SQLite qua Tauri invoke |
| 3 | **Không tự ý code ngoài phase đang active** | Phải đọc log điều phối ngày để nhận task |
| 4 | **AI là lớp hỗ trợ, không phải điều kiện chạy tối thiểu** | Máy không có Ollama vẫn phải hoạt động |

---

## 1. TRẠNG THÁI HIỆN TẠI CỦA PHẦN MỀM

### ✅ Đã hoạt động (code + build PASS)

| Module | File chính | Tình trạng |
|--------|-----------|------------|
| Import folder/files | `import_cmd.rs` · `importService.ts` | Hoạt động, import nhiều file vào case |
| OCR pipeline | `ocr_pipeline.py` · `doc_cmd.rs` | RapidOCR + PaddleOCR + handwriting preprocess |
| Scan inbox | `scan_cmd.rs` · `scanService.ts` | Watch folder + pipeline job lifecycle |
| Pipeline scheduler | `pipeline_execution_tick()` in `scan_cmd.rs` | State machine: job/task/event/checkpoint |
| Document viewer | `DocumentViewerPage.tsx` | PDF viewer + OCR panel + page navigation |
| Search FTS5 | `search_cmd.rs` · `searchService.ts` | Full-text search có highlight |
| Case management | `case_cmd.rs` · `caseService.ts` | CRUD + sort + OCR metadata (dossier_type, bút lục) |
| Export PDF | `export_cmd.rs` · `exportService.ts` | Export bundle cơ bản |
| AI workspace | `ai_cmd.rs` · `aiService.ts` | Extractive fallback + Ollama probe |
| Review queue | `review_cmd.rs` · `reviewService.ts` | Pending/approve/reject items |
| Purge hồ sơ | `case_cmd.rs` | Xóa an toàn + retry file-lock + deferred marker |
| File-lock hardening | `case_cmd.rs` · `doc_cmd.rs` · `export_cmd.rs` | Retry-backoff cho Windows lock |
| OCR env hardening | `doc_cmd.rs` | UTF-8 enforce + script/lang validation |
| Managed storage | `storage.rs` | `originals/` + `processed/` structure |
| DB migrations | `001` → `006` | Schema stable, layout blocks + extracted fields |

### 🔲 Chưa có (cần làm theo phase)

| Gap | Phase | Ghi chú |
|-----|-------|---------|
| Startup Environment Gate | P0-01 | Không có check RAM/OCR model/pdfium khi khởi động |
| File lifecycle đầy đủ | P0-02 | Chưa có `reviewed/` `managed/` `exports/` per-case |
| OCR dual-text (raw + normalized) | P0-03 | `ocr_pipeline.py` đã có `_normalize_text()` nhưng DB chưa lưu tách |
| Citation validator | P0-04 | AI output chưa bị chặn khi thiếu citation |
| Export package chuẩn | P0-05 | Chưa có cấu trúc 6-file bàn giao |
| DOCX/HTML parser | Phase 2 | Chưa parse mẫu Word/HTML reference |
| Event governance | Phase 3 | Event rời rạc, chưa có contract chuẩn |
| Hierarchical summary | Phase 4 | Chưa có page→doc→group→case |
| Adaptive performance | Phase 5 | Chưa đo hardware profile |
| Master dossier compiler | Phase 6 | Chưa sinh DOCX master tổng hợp |

---

## 2. BẢN ĐỒ THƯ MỤC V3

```
v3/
├── NEXT_ACTION.md              ← BẠN ĐANG Ở ĐÂY
├── README.md                   ← Giới thiệu cấu trúc v3
│
├── 01_governance/              ← ★ LUẬT TỔNG — đọc đầu tiên
│   ├── 01_AGENTS.md            ← Quy chuẩn bắt buộc toàn dự án
│   ├── 03_AGENT_RUNTIME_RULES.md ← Log bắt buộc + gate xóa phase
│   └── 04_ORCHESTRATION_PROTOCOL.md ← Luồng điều phối leader→agent
│
├── 03_specs/                   ← Đặc tả nghiệp vụ
│   ├── 00_SPEC_INDEX.md        ← Mục lục 31 file specs + trạng thái
│   ├── 01_tonghop.md           ← ★ Spec tổng hợp nhất (57KB)
│   └── 20260424_*.md           ← Đặc tả chi tiết từng module
│
├── standards/                  ← Luật kỹ thuật cứng
│   ├── 01_architecture.md      ← Phân tầng + evidence-first
│   ├── 02_runtime_offline_requirements.md ← Gate + installer
│   ├── 03_pipeline_ocr_ai.md   ← OCR schema + AI policy
│   ├── 04_file_lifecycle_export.md ← Vòng đời file
│   └── 05_backlog_implementation.md ← ★ Backlog chi tiết P0
│
├── 08_execution_phases/        ← Kế hoạch thực thi chia nhỏ
│   ├── 00_PHASE_INDEX.md       ← Mục lục + gap analysis
│   ├── 01_phase_p0_foundation.md ← ★ ĐANG ACTIVE
│   ├── 02 → 06                 ← Các phase tiếp theo
│
├── 05_logs/                    ← Log điều phối theo ngày
│   ├── 01_LOG_TEMPLATE.md      ← Template chuẩn
│   └── 20260504_phase-mvp-p0.md ← Log ngày mới nhất
│
└── 06_reference_samples/       ← Mẫu đầu ra đích (DOCX/HTML)
```

---

## 3. LUỒNG LÀM VIỆC BẮT BUỘC

```
┌─────────────────────────────────────────────────────┐
│  Agent mở dự án                                     │
│  ↓                                                  │
│  1. Đọc v3/NEXT_ACTION.md (file này)                │
│  ↓                                                  │
│  2. Xác định phase đang active (xem mục 4 bên dưới)│
│  ↓                                                  │
│  3. Đọc file phase chi tiết trong 08_execution_phases│
│  ↓                                                  │
│  4. Đọc log điều phối ngày: v3/05_logs/YYYYMMDD_*   │
│     → Tìm mục "Assigned agents" có tên mình không   │
│     → Nếu KHÔNG có: ghi HOLD, đề xuất task          │
│     → Nếu CÓ: thực hiện đúng scope được giao        │
│  ↓                                                  │
│  5. Code → Test (`npm run tauri:dev`) → Cập nhật log │
│  ↓                                                  │
│  6. Khi xong: cập nhật NEXT_ACTION + log ngày        │
└─────────────────────────────────────────────────────┘
```

---

## 4. PHASE ĐANG ACTIVE VÀ LỘ TRÌNH

### ★ ĐANG LÀM: P0 — Foundation

> **Quy ước đánh số:** P1 bỏ qua cố ý. Chuỗi phase: P0 → P2 → P3 → P4 → P5 → P6.
> **Phân biệt:** P0-04 = citation **warning**. P4-01 = citation **hard-stop** (chặn cứng). Hai task khác scope.

File chi tiết: [`01_phase_p0_foundation.md`](v3/08_execution_phases/01_phase_p0_foundation.md)

| Task | Mô tả | File cần sửa | Trạng thái |
|------|--------|--------------|-----------|
| **P0-01** | Startup Environment Gate | `main.rs` · `commands/mod.rs` · new service | 🔲 TODO |
| **P0-02** | File lifecycle (original → managed) | `import_cmd.rs` · `case_cmd.rs` · migrations | 🔲 TODO |
| **P0-03** | OCR block schema Unicode | `ocr_pipeline.py` · `scan_cmd.rs` · `schema.rs` | 🔲 TODO |
| **P0-04** | Citation validator | `ai_cmd.rs` · `aiService.ts` | 🔲 TODO |
| **P0-05** | Export package chuẩn | `export_cmd.rs` | 🔲 TODO |

### Lộ trình tiếp theo (chưa bắt đầu)

| Phase | Tên | Phụ thuộc | File |
|-------|-----|-----------|------|
| 2 | Parser DOCX/HTML + schema trung gian | P0 xong | [`02_phase_parser_and_schema.md`](v3/08_execution_phases/02_phase_parser_and_schema.md) |
| 3 | Event governance + audit + maintenance | P0 xong | [`03_phase_event_audit_maintenance.md`](v3/08_execution_phases/03_phase_event_audit_maintenance.md) |
| 4 | AI citation hard-stop + hierarchical summary | Phase 2,3 xong | [`04_phase_ai_citation_and_summary.md`](v3/08_execution_phases/04_phase_ai_citation_and_summary.md) |
| 5 | Adaptive performance profiles | Phase 3 xong | [`05_phase_performance_profiles.md`](v3/08_execution_phases/05_phase_performance_profiles.md) |
| 6 | Export master + offline installer + release | Phase 4,5 xong | [`06_phase_export_master_and_release.md`](v3/08_execution_phases/06_phase_export_master_and_release.md) |

---

## 5. DIỄN GIẢI CHI TIẾT TỪNG CÂU LỆNH P0

### P0-01 — Startup Environment Gate

**Mục đích:** Trước khi vào Dashboard, kiểm tra RAM, disk, OCR model, pdfium, SQLite. Nếu fail → chặn cứng, hiện lý do + hướng dẫn.

**Cách làm cụ thể:**
1. Tạo file `PhanMem/src-tauri/src/commands/system_cmd.rs` — command `run_startup_self_check`.
2. Check: RAM ≥ 4GB, disk free ≥ 10GB, workspace writable, SQLite OK, pdfium present, OCR models present.
3. Trả payload JSON `{ status: "pass"|"warning"|"fail", checks: [...], fatal_errors: [...] }`.
4. Đăng ký command trong `main.rs` → `invoke_handler`.
5. Frontend: tạo `StartupGatePage.tsx` render trước `AppShell`. Nếu `fail` → chặn navigation vào routes chính.
6. Tạo service `systemService.ts` gọi command.

**Đối chiếu code hiện tại:** `main.rs` setup có `db::init()` + `storage::ensure_managed_dirs()` nhưng không check hardware/runtime. App chạy thẳng vào Dashboard.

**Lưu ý:** Không cần check Ollama/AI model ở gate này — AI thiếu chỉ là warning.

---

### P0-02 — File Lifecycle

**Mục đích:** Tách rõ `original/processing/reviewed/managed/exports` per case. Không bao giờ ghi đè file gốc.

**Cách làm cụ thể:**
1. Mở rộng `storage.rs`: thêm `case_originals_dir(case_code)`, `case_processing_dir(case_code)`, `case_reviewed_dir(case_code)`, `case_managed_dir(case_code)`, `case_exports_dir(case_code)`.
2. `import_cmd.rs`: khi import, copy file vào `original/` (hiện đang copy vào `originals/<case_code>/`  — cần verify path chuẩn hóa).
3. `doc_cmd.rs`: khi sửa OCR/metadata, tạo revision record thay vì ghi đè.
4. Migration `007_file_lifecycle.sql`: thêm cột `file_status` (imported/processing/ocr_done/review_pending/reviewed/managed_ready/exported), `original_path`, `managed_path`, `revision_no`.
5. `export_cmd.rs`: chỉ export file có `file_status = managed_ready`.

**Đối chiếu code hiện tại:** `storage.rs` đã có `originals/` + `processed/` nhưng chưa có per-case `reviewed/managed/exports`. `document_status` trong `schema.rs` chỉ có `pending/processed/reviewed/error`.

---

### P0-03 — OCR Block Schema Unicode

**Mục đích:** Chuẩn dữ liệu OCR: lưu cả `raw_text` (giữ dấu tiếng Việt) và `normalized_text` (không dấu, tìm kiếm). Mỗi block có `bbox`, `confidence`, `block_type`, `reading_order`.

**Cách làm cụ thể:**
1. `ocr_pipeline.py`: output đã có `_normalize_text()`, `_classify_block()`, `_build_layout()` trả `blocks[]` với `block_type/reading_order/confidence`. → **Phần Python đã gần đủ.** Cần thêm `normalized_text` vào mỗi block và `unicode_form: "NFC"`.
2. `scan_cmd.rs` hoặc `doc_cmd.rs`: khi nhận kết quả OCR từ Python, parse và lưu `page_layout_blocks` + `document_extracted_fields` — **migration 006 đã tạo bảng này.**
3. `schema.rs`: bổ sung constant cho bảng `page_layout_blocks`, `document_extracted_fields`.
4. FTS5: rebuild index từ `normalized_text` để search không dấu.

**Đối chiếu code hiện tại:** `ocr_pipeline.py` đã có `_normalize_text()`, `_classify_block()`, `_extract_fields()`. Migration 006 đã tạo `page_layout_blocks` + `document_extracted_fields`. → **Phần này gần hoàn thành**, chỉ cần sync schema.rs + verify FTS5 index rebuild.

---

### P0-04 — Citation Validator

**Mục đích:** Mọi kết luận AI phải đi kèm citation (document_id, page_number, quote). Câu không có citation → reject hoặc `review_required`.

**Cách làm cụ thể:**
1. `ai_cmd.rs`: sau mỗi lần `ai_summarize_case()` hoặc `ai_ask_case()`, parse output tìm citation markers.
2. Nếu câu trả lời không chứa anchor đến nguồn: set `review_required = true` + append warning.
3. `aiService.ts`: hiện kết quả cho user với badge warning nếu citation thiếu.

**Đối chiếu code hiện tại:** `ai_cmd.rs` có `load_case_context()` trả chunks text + score, nhưng output LLM không bị validate citation. `score_text()` + `keyword_terms()` chỉ ranking, không enforce citation anchor.

---

### P0-05 — Export Package Chuẩn

**Mục đích:** Export sinh đúng cấu trúc bàn giao: `00_index.docx`, `01_bao_cao.docx`, `02_so_do.html`, `03_tai_lieu/`, `04_citation.json`, `05_audit.json`.

**Cách làm cụ thể:**
1. `export_cmd.rs`: refactor `export_pdf_bundle` thành `export_dossier_package` tạo thư mục chuẩn.
2. Sử dụng template engine (Rust `docx-rs` hoặc Python `python-docx`) để render DOCX từ data.
3. Sinh `citation.json` từ `document_extracted_fields` + `page_layout_blocks`.
4. Sinh `audit.json` từ log pipeline + revision history.

**Đối chiếu code hiện tại:** `export_cmd.rs` chỉ export PDF bundle đơn giản. Chưa có DOCX template renderer. Chưa có cấu trúc multi-file export.

---

## 6. CỔNG KIỂM TRA BẮT BUỘC (BUILD GATE)

Sau mỗi task hoàn thành, agent **phải** chạy:

```
# 1. Rust check
cargo check -j 1          # cwd: PhanMem/src-tauri

# 2. Full build
npm run tauri:build        # cwd: PhanMem

# 3. Cập nhật log
→ v3/05_logs/YYYYMMDD_phase-*.md
→ v3/NEXT_ACTION.md (đổi trạng thái task)
```

---

## 7. QUY TẮC ĐIỀU PHỐI (TÓM TẮT)

| Tình huống | Hành động |
|-----------|----------|
| Agent mở dự án, chưa có lệnh cụ thể | Đọc `NEXT_ACTION.md` → đọc phase active → đọc log ngày |
| Log ngày có "Assigned agents" ghi tên mình | Thực hiện đúng scope task được giao |
| Log ngày chưa có tên mình | Ghi `HOLD` + đề xuất task phù hợp vào log |
| Task vượt năng lực | Ghi `ESCALATE` trong log |
| Task mâu thuẫn ràng buộc hệ thống | Ghi `BLOCKED` + lý do trong log |
| Xong task | Cập nhật log + NEXT_ACTION + chạy build gate |
| Muốn xóa phase cũ | Phải có đủ 3 gate: Code PASS + Test PASS + Bug PASS |

---

## 8. THAM CHIẾU NHANH

| Cần gì | Đọc ở đâu |
|--------|-----------|
| Kiến trúc tổng | [`standards/01_architecture.md`](v3/standards/01_architecture.md) |
| Yêu cầu offline | [`standards/02_runtime_offline_requirements.md`](v3/standards/02_runtime_offline_requirements.md) |
| OCR + AI policy | [`standards/03_pipeline_ocr_ai.md`](v3/standards/03_pipeline_ocr_ai.md) |
| Vòng đời file | [`standards/04_file_lifecycle_export.md`](v3/standards/04_file_lifecycle_export.md) |
| Backlog chi tiết P0 | [`standards/05_backlog_implementation.md`](v3/standards/05_backlog_implementation.md) |
| Spec tổng hợp nhất | [`03_specs/01_tonghop.md`](v3/03_specs/01_tonghop.md) |
| Quy tắc agent | [`01_governance/03_AGENT_RUNTIME_RULES.md`](v3/01_governance/03_AGENT_RUNTIME_RULES.md) |
| Điều phối | [`01_governance/04_ORCHESTRATION_PROTOCOL.md`](v3/01_governance/04_ORCHESTRATION_PROTOCOL.md) |
| Mẫu đầu ra đích | [`06_reference_samples/`](v3/06_reference_samples/) |
