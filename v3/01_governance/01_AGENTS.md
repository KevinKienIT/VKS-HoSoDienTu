# AGENTS.md — Quy chuẩn bắt buộc cho MỌI agent trong VKS ECMS

> **Source of Truth.** File này là luật tối thượng. Mọi agent (Antigravity, Codex, Roo, Claude, Cursor, v.v.) đều phải tuân theo TOÀN BỘ nội dung bên dưới trước khi thực hiện BẤT KỲ hành động nào.

> Cập nhật: 2026-05-04 · Phiên bản: 3.1

---

## ⛔ PHẦN 1 — NỀN TẢNG DỰ ÁN (ĐỌC TRƯỚC KHI LÀM BẤT CỨ GÌ)

> **VKS ECMS là ứng dụng TAURI DESKTOP (.exe). KHÔNG PHẢI WEB APP.**

### 1.1 Kiến trúc hệ thống

```
PhanMem/
├── src/                ← Frontend (React + TypeScript, render trong Tauri WebView)
│   ├── modules/        ← ★ 12 module nghiệp vụ (tên tiếng Việt không dấu)
│   │   ├── bangdieukhien/   ← Dashboard (01)
│   │   ├── duahosovao/      ← Import hồ sơ (02)
│   │   ├── quettailieu/     ← Scan tài liệu (03)
│   │   ├── phantichtailieu/ ← Phân tích OCR (04)
│   │   ├── quantailieu/     ← Quản tài liệu (05)
│   │   ├── timkiem/         ← Tìm kiếm FTS (06)
│   │   ├── phantichai/      ← AI Notebook (07)
│   │   ├── xuatbangiao/     ← Export/Xuất (08)
│   │   ├── duyethotro/      ← Review queue (09)
│   │   ├── hosovuan/        ← Hồ sơ vụ án (10)
│   │   ├── cauhinh/         ← Cài đặt (11)
│   │   └── loidung/         ← Core/Shared (12)
│   ├── store/          ← Zustand stores (catalogStore, moduleStore, uiStore)
│   ├── services/       ← [LEGACY] Service cũ — chỉ App.tsx/store dùng
│   └── App.tsx         ← Router, import từ modules/
├── src-tauri/          ← Backend (Rust, Tauri commands, SQLite)
│   ├── src/commands/   ← Rust command modules
│   │   ├── case_cmd.rs     ← Hồ sơ vụ án (CRUD + purge)
│   │   ├── import_cmd.rs   ← Import folder/files
│   │   ├── scan_cmd.rs     ← Scan + Pipeline scheduler
│   │   ├── doc_cmd.rs      ← OCR + Document management
│   │   ├── ai_cmd.rs       ← AI commands (extractive + Ollama)
│   │   ├── search_cmd.rs   ← Search FTS5
│   │   ├── export_cmd.rs   ← Export PDF bundle
│   │   ├── review_cmd.rs   ← Review queue
│   │   ├── module_cmd.rs   ← Module config
│   │   └── catalog_cmd.rs  ← Catalog scanner
│   ├── src/db/         ← Database schema + migrations
│   └── src/storage.rs  ← Managed storage directories
├── python/             ← OCR/AI scripts (gọi qua tauri-plugin-shell)
│   ├── ocr/            ← OCR pipeline cơ bản (RapidOCR)
│   ├── phantich/       ← Pipeline xử lý nghiệp vụ
│   │   ├── xuly_hoso.py      ← Core Pipeline (PDF→OCR→AI→DOCX)
│   │   └── ai_xuly_text.py   ← AI Slow Service (Ollama qwen2.5)
│   ├── danhmuc/        ← Folder catalog scanner
│   └── xuatfile/       ← Export helpers (Phase 2)
└── package.json

### 1.1.1 Luồng nghiệp vụ chính (Business Flow)
```
Bước 1: Import    → duahosovao  → Nạp folder/ZIP vào DB, tạo case
Bước 2: Scan      → quettailieu → Watch inbox, nhận file, pipeline OCR
Bước 3: Phân tích → phantichtailieu → Xem page, annotation, classify
Bước 4: AI        → phantichai  → Tóm tắt, hỏi đáp (offline LLM)
Bước 5: Review    → duyethotro  → Duyệt lỗi OCR, sửa text
Bước 6: Export    → xuatbangiao → Xuất Master Dossier (PDF/DOCX)
```

### 1.1.2 Quy trình OCR & AI Agent (Slow Service)
Mọi xử lý tài liệu PDF/Scan tuân thủ luồng:
1. **Trích xuất Layout**: Dùng RapidOCR lấy Bounding Box, tạo lại đoạn văn, giữ lề.
2. **Kích hoạt AI (on-demand)**: Cờ `--use-ai` → `ai_xuly_text.py` gọi Ollama Local.
3. **Kết xuất Word**: Render ra .docx chuyên nghiệp (Times New Roman 12, Table).
```

### 1.2 Phân tầng chuẩn

1. **Presentation:** UI hiển thị trạng thái, không giữ dữ liệu nghiệp vụ cốt lõi.
2. **Application:** Điều phối workflow/state machine.
3. **Domain:** Rule nghiệp vụ hồ sơ, citation, review.
4. **Infrastructure:** Rust commands, Python OCR worker, file system.
5. **Persistence:** SQLite + migration + audit + FTS5.

### 1.3 Nguyên tắc bất biến

| # | Quy tắc | Lý do |
|---|---------|-------|
| 1 | **Chỉ test bằng `npm run tauri:dev`** | Tauri API không hoạt động trong browser thuần |
| 2 | **Không dùng `localStorage`/`fetch` cho dữ liệu nghiệp vụ** | Chỉ SQLite qua Tauri invoke |
| 3 | **Không tự ý code ngoài phase đang active** | Phải đọc log điều phối ngày để nhận task |
| 4 | **AI là lớp hỗ trợ, không phải điều kiện chạy tối thiểu** | Máy không có Ollama vẫn phải hoạt động |
| 5 | **Evidence-first, citation-first** | Mọi kết luận AI phải truy vết về nguồn (document/page/quote) |
| 6 | **Không ghi đè file gốc** | Mọi sửa tạo revision, giữ original bất biến |

### 1.4 Cấm tuyệt đối

| Hành động | Lý do |
|-----------|-------|
| Chạy Vite standalone rồi mở browser | Tauri API không hoạt động ngoài Tauri runtime |
| Tạo script `npm run dev` | Đã xoá vĩnh viễn — KHÔNG được tạo lại |
| Tạo `dist/` bằng `npm run build` rồi deploy | `dist/` chỉ là bước trung gian |
| Dùng `localStorage`/`sessionStorage` cho nghiệp vụ | Phải dùng SQLite qua Tauri |
| Dùng `window.fetch` gọi server bên ngoài | Offline 100% |
| Tạo file HTML riêng ngoài `index.html` | Single-page app trong WebView |
| Test bằng browser DOM (Cypress, Playwright) | Tauri có test riêng |

### 1.5 Cách chạy — CHỈ CÓ HAI LỆNH

```powershell
# Development:
cd PhanMem && npm run tauri:dev

# Build sản phẩm:
cd PhanMem && npm run tauri:build
```

---

## 📐 PHẦN 2 — QUY CHUẨN CODE

### 2.1 Tư duy trước khi code

- Nêu rõ giả định. Nếu không chắc, hỏi.
- Nếu có nhiều cách hiểu, trình bày — không chọn im lặng.
- Nếu có cách đơn giản hơn, đề xuất. Phản biện khi cần.
- Nếu không rõ, dừng lại. Nêu điểm chưa rõ. Hỏi.

### 2.2 Đơn giản trước

- Không thêm feature ngoài yêu cầu.
- Không tạo abstraction cho code dùng 1 lần.
- Không thêm "flexibility" hay "configurability" không được yêu cầu.
- Không xử lý lỗi cho tình huống không thể xảy ra.
- Viết 200 dòng mà có thể 50? Viết lại.

### 2.3 Sửa chính xác

- Không "cải thiện" code lân cận, comment, hay formatting.
- Không refactor thứ không hỏng.
- Match style hiện tại, dù bạn muốn khác.
- Phát hiện dead code? Mention, không xóa.
- Chỉ xóa import/variable/function mà CHÍNH SỬA CỦA BẠN làm thừa.
- Mỗi dòng thay đổi phải trace trực tiếp về yêu cầu user.

### 2.4 Mục tiêu rõ ràng

Biến task thành mục tiêu kiểm chứng được:
```
1. [Bước] → verify: [check]
2. [Bước] → verify: [check]
3. [Bước] → verify: [check]
```

### 2.5 Pattern bắt buộc

**Frontend service:**
```typescript
import { invoke } from "@tauri-apps/api/core";

export async function someAction(input: SomeInput): Promise<SomeResult | null> {
  try {
    return await invoke<SomeResult>("some_command", { input });
  } catch (error) {
    console.warn("some_command failed", error);
    return null;
  }
}
```

**Rust command:**
```rust
#[tauri::command]
pub fn some_command(db: State<'_, DbState>, input: SomeInput) -> Result<SomeOutput, String> {
    let conn = db.0.lock().map_err(|_| "DB_LOCK_FAILED".to_string())?;
    // validate input → transaction if multi-table → audit event
    Ok(output)
}
```

**Quy tắc:**
- Frontend KHÔNG gọi `invoke` trực tiếp nếu đã có service.
- Mỗi module mới: Rust command + Frontend service + UI component.
- DB mutex: không gọi command khác khi đang giữ lock. `drop(conn)` trước nếu cần query lại.

---

## 🎨 PHẦN 3 — QUY CHUẨN GIAO DIỆN (UI/UX DESIGN)

> **Chi tiết đầy đủ:** [`v3/standards/06_design_system.md`](../standards/06_design_system.md)
> Bao gồm: design tokens, color palette, typography, component standards, layout patterns, micro-interactions, accessibility.
> Nguồn cảm hứng: Linear, Notion, Superhuman, Raycast (từ `07_external_refs/design-md/`).

### 3.1 Thiết kế tổng thể

```
AppShell
├── DossierSidebar
│   ├── Dashboard
│   ├── Đưa hồ sơ vào
│   ├── Scan tài liệu
│   ├── Phân tích tài liệu
│   ├── Quản lý tài liệu
│   ├── Phân tích AI
│   ├── Tìm kiếm
│   └── Settings (tách riêng dưới cùng)
├── Header / Global Search
└── Workspace route
    ├── DashboardPage
    ├── ImportJobPage
    ├── ScanPage
    ├── AnalyzePage
    ├── DocumentListPage / DocumentViewerPage
    ├── SearchPage
    ├── AiWorkspacePage
    ├── ExportPage
    └── SettingsPage
```

### 3.2 Nguyên tắc UI

1. **Dossier workspace** — không phải admin panel web. Cảm giác như ứng dụng desktop chuyên nghiệp.
2. **Status badge rõ ràng** — mỗi tài liệu phải hiện trạng thái: chưa OCR, đã OCR, lỗi, cần review, file missing.
3. **Responsive trong WebView** — layout phải hoạt động ở nhiều kích thước cửa sổ.
4. **Không placeholder image** — nếu cần ảnh, tạo ảnh thật.
5. **OCR overlay** — click block trên ảnh ↔ select text trong panel, 2 chiều.
6. **Citation click-through** — click source → mở đúng document/page.

### 3.3 CSS conventions

- Variables: `variables.css`
- Layout: `layout.css`
- Navigation: `dossier-nav.css`
- Components: `components.css`
- Không dùng TailwindCSS trừ khi user yêu cầu.

### 3.4 Component map

| Vùng | File chính |
|------|-----------|
| App shell/routes | `App.tsx` |
| Dashboard | `DashboardPage.tsx` |
| Import | `ImportJobPage.tsx` + `importService.ts` |
| Scan | `ScanPage.tsx` + `scanService.ts` |
| Documents | `DocumentListPage.tsx` + `DocumentViewerPage.tsx` + `DocumentViewer.tsx` |
| Search | `SearchPage.tsx` + `searchService.ts` |
| AI | `AiWorkspacePage.tsx` + `AiNotebookPanel.tsx` + `aiService.ts` |
| Export | `ExportPage.tsx` + `exportService.ts` |
| Stores | `uiStore.ts`, `moduleStore.ts`, `catalogStore.ts` |

---

## 🗄️ PHẦN 4 — QUY CHUẨN DATABASE & STORAGE

### 4.1 Vòng đời file

```
imported → processing → ocr_done → review_pending → reviewed → managed_ready → exported
```

### 4.2 Cấu trúc storage

```
VKS_ECMS_Data/
├── originals/<CASE_CODE>/     ← bất biến, không bao giờ sửa
├── processing/<CASE_CODE>/    ← trung gian OCR
├── reviewed/<CASE_CODE>/      ← sau chỉnh sửa user
├── managed/<CASE_CODE>/       ← bản chuẩn nghiệp vụ
├── exports/<EXPORT_JOB_ID>/   ← gói bàn giao
└── logs/                      ← pipeline/audit global
```

### 4.3 Migration pattern

```sql
-- File: migrations/NNN_description.sql
ALTER TABLE some_table ADD COLUMN new_col TEXT;
CREATE INDEX IF NOT EXISTS idx_new ON some_table(new_col);
```

Sau khi thêm migration:
1. Include trong `db/mod.rs`
2. Sync `schema.rs` nếu có constants liên quan
3. Test app khởi động với DB cũ VÀ DB mới

### 4.4 OCR block schema tối thiểu

```json
{
  "raw_text": "Quyết định khởi tố",
  "normalized_text": "quyet dinh khoi to",
  "unicode_form": "NFC",
  "bbox": { "x": 120, "y": 80, "width": 340, "height": 42 },
  "confidence": 0.92,
  "reading_order": 3,
  "block_type": "document_number",
  "engine": "paddleocr"
}
```

---

## 📋 PHẦN 5 — QUY TRÌNH LÀM VIỆC

### 5.1 Luồng bắt buộc khi nhận task

```
1. Đọc v3/NEXT_ACTION.md
2. Xác định phase đang active
3. Đọc file phase trong 08_execution_phases/
4. Đọc log điều phối ngày: v3/05_logs/YYYYMMDD_*
   → Có tên mình trong "Assigned agents"? → Thực hiện đúng scope
   → Không có? → Ghi HOLD, đề xuất task
5. Code → Test (npm run tauri:dev) → Cập nhật log
6. Chạy build gate → Cập nhật NEXT_ACTION + log
```

### 5.2 Gate trước khi code

| Gate | Câu hỏi | Pass khi |
|------|---------|----------|
| Compatibility | Runtime tương thích? | node, npm, cargo, python OK |
| Feasibility | Làm được offline trong Tauri? | Không cần internet/server |
| UI Map | Màn hình nào đổi? | Có tên page/component |
| Code Map | File nào đổi? | Có Rust cmd + service + UI + DB |
| Data Safety | Có đụng file gốc? | Không ghi đè original |
| Test Evidence | Lệnh nào chứng minh? | Có output pass rõ |

### 5.3 Build gate sau mỗi task

```powershell
# 1. Rust check
cargo check -j 1              # cwd: PhanMem/src-tauri

# 2. TypeScript check
npm exec tsc -- --noEmit       # cwd: PhanMem

# 3. Full build (nếu task lớn)
npm run tauri:build             # cwd: PhanMem

# 4. Cập nhật log + NEXT_ACTION
```

### 5.4 Trạng thái agent

| Tình huống | Hành động |
|-----------|----------|
| Chưa có lệnh cụ thể | Đọc NEXT_ACTION → phase → log ngày |
| Log có tên mình | Thực hiện đúng scope |
| Log chưa có tên mình | HOLD + đề xuất |
| Task vượt năng lực | ESCALATE |
| Task mâu thuẫn hệ thống | BLOCKED + lý do |
| Xong task | Cập nhật log + NEXT_ACTION + build gate |
| Muốn xóa phase cũ | Code PASS + Test PASS + Bug PASS |

---

## 🧠 PHẦN 6 — MEMORY & IDENTITY

### 6.1 Memory

- **Daily notes:** `memory/YYYY-MM-DD.md` — raw logs
- **Long-term:** `MEMORY.md` ở root — curated memories
- Không dùng "mental notes" — ghi vào file.
- MEMORY.md chỉ load trong main session, không load trong shared contexts.

### 6.2 Log điều phối

- Format: `v3/05_logs/YYYYMMDD_phase-<phase>.md`
- Template: `v3/05_logs/01_LOG_TEMPLATE.md`
- Mọi lệnh và kết quả phải ghi vào log
- Không xóa log khi chưa pass 3 gate (code + test + bug)

---

## 📎 PHẦN 7 — THAM CHIẾU NHANH

### 7.1 File map

| Cần gì | Đọc ở đâu |
|--------|-----------|
| **Luật agent (file này)** | `v3/01_governance/01_AGENTS.md` |
| Agent runtime rules | `v3/01_governance/03_AGENT_RUNTIME_RULES.md` |
| Orchestration protocol | `v3/01_governance/04_ORCHESTRATION_PROTOCOL.md` |
| Kiến trúc tổng | `v3/standards/01_architecture.md` |
| Yêu cầu offline | `v3/standards/02_runtime_offline_requirements.md` |
| OCR + AI policy | `v3/standards/03_pipeline_ocr_ai.md` |
| Vòng đời file | `v3/standards/04_file_lifecycle_export.md` |
| Backlog P0 | `v3/standards/05_backlog_implementation.md` |
| **★ Design system** | `v3/standards/06_design_system.md` |
| **★ Module map** | `v3/standards/07_module_map.md` |
| Karpathy coding rules | `v3/07_external_refs/SKILL.md` |
| Design refs (59 brands) | `v3/07_external_refs/design-md/` |
| Spec tổng hợp | `v3/03_specs/01_tonghop.md` |
| Spec index | `v3/03_specs/00_SPEC_INDEX.md` |
| UI/UX proposal | `v3/03_specs/02_UI_UX_IMPROVEMENT_PROPOSAL.md` |
| Phase đang active | `v3/NEXT_ACTION.md` |
| Log điều phối | `v3/05_logs/YYYYMMDD_*.md` |
| Mẫu đầu ra | `v3/06_reference_samples/` |

### 7.2 Thứ tự ưu tiên khi mâu thuẫn

```
AGENTS.md (file này) > standards/ > NEXT_ACTION/log > execution phases > specs > external refs
```

---

## ⚠️ PHẦN 8 — AN TOÀN

- Không exfiltrate dữ liệu riêng tư. Bao giờ cũng vậy.
- Không chạy lệnh phá hoại mà không hỏi.
- `trash` > `rm` (khôi phục được tốt hơn mất hẳn).
- Khi không chắc, hỏi.
- Không gửi email, tweet, bài public mà không hỏi trước.
