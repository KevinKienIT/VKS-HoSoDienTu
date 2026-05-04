# TONGHOP.md - He thong xu ly tep lenh V3 cho VKS ECMS

Cap nhat: 2026-04-29  
Pham vi: `v3/` + doi chieu nhanh voi source hien tai trong `PhanMem/`  
Loai san pham: Tauri Desktop App `.exe`, offline-first, khong phai web app doc lap.

Tai lieu nay la ban tong hop dieu phoi trung tam. Khi nhan mot lenh moi, agent doc file nay truoc de biet:

1. Doc tep nao trong `v3/` theo dung thu tu.
2. Luu lenh va ket qua vao dau.
3. Doi chieu UI/code/database/backend/python ra sao.
4. Chay lenh nao, khong chay lenh nao.
5. Khi nao duoc code, khi nao phai HOLD/BLOCKED/ESCALATE.

---

## 1. Nguyen tac nen tang khong duoc sai

VKS ECMS la ung dung desktop Tauri `.exe`.

Khong duoc bien thanh web app. Khong test nghiep vu bang browser doc lap. Khong dung `localStorage`/`sessionStorage` cho du lieu nghiep vu. Khong goi server ngoai bang `fetch` cho nghiep vu core. Du lieu nghiep vu phai di qua Tauri command + SQLite + filesystem duoc quan ly.

Lenh chay ung dung hop le:

```powershell
cd PhanMem
npm run tauri:dev
```

Lenh build san pham hop le:

```powershell
cd PhanMem
npm run tauri:build
```

Khong chay cac lenh nay de test chuc nang:

```powershell
npm run dev
npm run build
npm run preview
```

Ghi chu: `npm run build` neu ton tai chi la buoc noi bo do Tauri goi trong qua trinh `tauri:build`, agent khong goi rieng de ket luan app pass.

---

## 2. Ban do thu muc V3

```text
v3/
├── README.md
├── NEXT_ACTION.md
├── TONGHOP.md                         <- file dieu phoi trung tam nay
├── 01_governance/                     <- ★ LUAT TONG (AGENTS, runtime, orchestration)
├── 03_specs/                          <- dac ta san pham, UI, schema, nghiep vu
├── 05_logs/                           <- log dieu phoi theo ngay/phase
├── 06_reference_samples/              <- mau DOCX/HTML dau ra can mo phong
├── 07_external_refs/                  <- tham khao ngoai, khong phai source of truth
├── 08_execution_phases/               <- phase thuc thi P0-P6 + preplan
└── standards/                         <- luat kien truc, runtime, OCR, lifecycle
```

Quy tac doc nhanh:

| Nhom | Vai tro | Khi nao doc |
|---|---|---|
| `NEXT_ACTION.md` | Lenh hien hanh va phase dang active | Doc dau tien sau file nay |
| `05_logs/YYYYMMDD_phase-*.md` | Lenh trong ngay, agent duoc giao, ket qua | Doc khi tiep tuc phase hoac can biet trang thai |
| `08_execution_phases/` | Chi tiet viec can code theo phase | Doc theo ma task P0/P2/P3... |
| `standards/` | Dieu kien bat buoc khong duoc pha | Doc truoc khi sua logic lien quan |
| `03_specs/01_tonghop.md` | Dac ta tong san pham va ky vong dai han | Doc khi can hieu muc tieu lon |
| `03_specs/*.md` | Dac ta tung module/nghiep vu/UI | Doc co chon loc theo task |
| `03_specs/03_schema.json` | JSON schema lon | Dung query/parse theo khoa, khong paste nguyen file |
| `06_reference_samples/` | Mau Word/HTML dau ra | Doc khi lam export/master dossier |
| `07_external_refs/` | Tham khao phong cach, agent skill | Chi doc khi task yeu cau, khong quet het |

---

## 3. Thu tu doc lenh bat buoc cho moi task

Moi task phai di theo chuoi sau:

```text
User prompt
  -> TONGHOP.md
  -> NEXT_ACTION.md
  -> 05_logs/<today_phase>.md
  -> 08_execution_phases/<active_phase>.md
  -> standards/<relevant>.md
  -> 03_specs/<relevant>.md
  -> PhanMem/<code files>
  -> update log/result
```

Neu task la bug fix nho, van phai doi chieu:

1. No co dung Tauri desktop/offline khong.
2. No thuoc phase nao.
3. No cham vao UI nao, command nao, bang DB nao.
4. No can migration khong.
5. No co can rebuild FTS/cache/export/citation khong.

---

## 4. Phan loai tep lenh trong V3

### 4.1 Governance command

Nguon:

- `v3/01_governance/03_AGENT_RUNTIME_RULES.md`
- `v3/01_governance/04_ORCHESTRATION_PROTOCOL.md`

Dung de quyet dinh agent co duoc code hay chua.

Trang thai:

| Trang thai | Y nghia | Hanh dong |
|---|---|---|
| `READY` | Co lenh ro, nam trong phase | Tien hanh doc code va sua |
| `HOLD` | Chua co dieu phoi ro trong log ngay | Ghi de xuat, khong code ngoai luong |
| `BLOCKED` | Lenh mau thuan ranh gioi he thong | Bao ro ly do |
| `ESCALATE` | Can quyet dinh ngoai kha nang agent | Chuyen ve leader/user |
| `DONE` | Code + test + bug check da pass | Ghi log va ket qua |

### 4.2 Phase command

Nguon:

- `v3/08_execution_phases/00_PHASE_INDEX.md`
- `v3/08_execution_phases/01_phase_p0_foundation.md`
- `v3/08_execution_phases/02_phase_parser_and_schema.md`
- `v3/08_execution_phases/03_phase_event_audit_maintenance.md`
- `v3/08_execution_phases/04_phase_ai_citation_and_summary.md`
- `v3/08_execution_phases/05_phase_performance_profiles.md`
- `v3/08_execution_phases/06_phase_export_master_and_release.md`
- `v3/08_execution_phases/07_preplan_feasibility_and_command_playbook.md`

Dung de lay task code cu the. Moi phase task phai co:

- Muc tieu
- File can tao/sua
- UI map
- Code map
- Database/migration neu co
- Lenh test
- Doi chieu pass

### 4.3 Spec command

Nguon:

- `v3/03_specs/01_tonghop.md`
- `v3/03_specs/20260424_*.md`
- `v3/03_specs/20260426_01_quyet_dinh_nen_tang_va_cach_chay.md`

Dung de hieu ky vong san pham: workspace ho so, OCR, AI, citation, export, user stories.

Spec khong duoc code thang neu mau thuan voi standards. Thu tu uu tien:

```text
AGENTS/governance > standards > NEXT_ACTION/log active phase > execution phase > specs > external refs
```

### 4.4 Log command

Nguon:

- `v3/05_logs/20260429_phase-mvp-p0.md`
- Mau: `v3/05_logs/01_LOG_TEMPLATE.md`

Moi lenh va ket qua agent phai duoc ghi vao log phase. Khong xoa plan/phase neu chua co du 3 gate:

1. Code: PASS
2. Check test: PASS
3. Check bug: PASS hoac blocker da dong

### 4.5 Reference command

Nguon:

- `v3/06_reference_samples/Bang_chi_muc_trich_dan_tai_lieu.docx`
- `v3/06_reference_samples/Bao_cao_tong_hop_vu_an_LeThanhCong_D134.docx`
- `v3/06_reference_samples/So_do_vu_an_LeThanhCong_D134.html`
- `v3/06_reference_samples/01_export_templates/`

Dung khi lam export package, master dossier, chi muc trich dan, bao cao tong hop, so do vu an.

---

## 5. Gate bat buoc truoc khi code

Truoc khi sua source, lap bang gate ngan gon:

| Gate | Cau hoi | Pass khi |
|---|---|---|
| Compatibility | Version/runtime co tuong thich khong? | `node`, `npm`, `cargo`, `python`, OCR runtime dat yeu cau |
| Feasibility | Co the lam offline trong Tauri khong? | Khong can internet/server ngoai, co fallback |
| UI Map | Man hinh nao doi? | Co ten page/component va luong nguoi dung |
| Code Map | File nao doi? | Co Rust command + service + UI + DB neu can |
| Data Safety | Co dung file goc/SQLite khong? | Khong ghi de file goc, co audit/revision khi can |
| Test Evidence | Lenh nao chung minh? | Co output hoac ket qua pass ro |

Lenh minh hoa compatibility:

```powershell
node -v
npm -v
cargo -V
python --version
py -m pip show rapidocr onnxruntime paddleocr opencv-python numpy
```

Doi chieu pass/fail:

| Ket qua | Xu ly |
|---|---|
| Thieu Python/OCR package trong moi truong dev | Warning neu app co fallback; BLOCKED neu task can OCR that |
| Thieu Rust/Cargo | BLOCKED cho backend |
| Chi pass trong browser | FAIL, khong tinh |
| Pass trong Tauri runtime | Co the ghi PASS |

---

## 6. Ban do phase trien khai

### 6.1 Phase P0 Foundation - dang active

Nguon chinh:

- `v3/NEXT_ACTION.md`
- `v3/05_logs/20260429_phase-mvp-p0.md`
- `v3/08_execution_phases/01_phase_p0_foundation.md`
- `v3/standards/02_runtime_offline_requirements.md`
- `v3/standards/03_pipeline_ocr_ai.md`
- `v3/standards/04_file_lifecycle_export.md`

| Ma | Ten task | Muc tieu | Code map | UI map | Lenh/doi chieu |
|---|---|---|---|---|---|
| P0-01 | Startup Environment Gate | Self-check truoc Dashboard | `main.rs`, `commands/system_cmd.rs`, `services/systemService.ts` | `StartupGatePage`, `App.tsx` route/gate | `npm run tauri:dev`, fail fatal thi chan app |
| P0-02 | File lifecycle | Case co `original/processing/reviewed/managed/exports/logs` | `storage.rs`, migration `007`, import/export commands | Import, Case detail, Document list | Kiem tra file goc khong bi ghi de |
| P0-03 | OCR block schema Unicode | OCR co `raw_text`, `normalized_text`, bbox, confidence | `ocr_pipeline.py`, migration `008`, `schema.rs`, `doc_cmd.rs` | Viewer OCR overlay/panel | OCR tieng Viet NFC, FTS rebuild tim duoc text |
| P0-04 | Citation validator | AI/export khong ket luan neu citation sai | `ai_cmd.rs`, `search_cmd.rs`, `export_cmd.rs` | AI source cards, Review queue | Source click dung doc/page |
| P0-05 | Export package chuan | Tao goi export DOCX/HTML/JSON/audit | `export_cmd.rs`, `exportService.ts`, `ExportPage.tsx` | Export studio | Tao du folder export + manifest |

Thu tu uu tien:

```text
P0-01 -> P0-02 -> P0-03 -> P0-04 -> P0-05
```

Neu can chon task de lam truoc vi it rui ro: P0-03 co the di song song voi P0-01 neu chi sync schema/OCR output, nhung khong duoc pha startup gate.

### 6.2 Phase P2 Parser and Schema

Nguon:

- `v3/08_execution_phases/02_phase_parser_and_schema.md`

| Ma | Ten task | Muc tieu | Code map |
|---|---|---|---|
| P2-01 | DOCX Parser | Tach paragraph/table/heading/citation tu DOCX | `python/parsers/docx_parser.py`, Rust wrapper |
| P2-02 | HTML Parser | Tach node/section/link tu HTML | `python/parsers/html_parser.py`, Rust wrapper |
| P2-03 | Intermediate Schema | Luu parsed sections | migration `009_parsed_sections.sql`, `schema.rs` |
| P2-04 | Pre-export Validator | Chan export khi thieu citation/review | `export_cmd.rs` |

### 6.3 Phase P3 Event Audit Maintenance

Nguon:

- `v3/08_execution_phases/03_phase_event_audit_maintenance.md`

| Ma | Muc tieu | Code map |
|---|---|---|
| P3-01 | Event contract versioning | `main.rs`, event emit wrappers |
| P3-02 | Persist-before-emit | `scan_cmd.rs`, `doc_cmd.rs`, audit events |
| P3-03 | Audit boundary | DB audit tables, commands |
| P3-04 | Maintenance jobs | `pipeline_execution_tick`, cleanup cache/index |

### 6.4 Phase P4 AI Citation and Summary

Nguon:

- `v3/08_execution_phases/04_phase_ai_citation_and_summary.md`

| Ma | Muc tieu | Code map |
|---|---|---|
| P4-01 | Citation validator hard-stop | `ai_cmd.rs`, `export_cmd.rs` |
| P4-02 | Page-level summary | `ocr_pipeline.py`, `summarizer.py`, DB summaries |
| P4-03 | Document/group/case summary | `ai_cmd.rs`, extracted fields |
| P4-04 | Review loop | `review_cmd.rs`, `ReviewQueuePage.tsx` |

### 6.5 Phase P5 Performance Profiles

Nguon:

- `v3/08_execution_phases/05_phase_performance_profiles.md`

| Ma | Muc tieu | Code map |
|---|---|---|
| P5-01 | Install-time profiling | `main.rs`, startup self-check |
| P5-02 | Runtime monitoring | `scan_cmd.rs`, scheduler/job queue |
| P5-03 | Task mode switching | `ocr_pipeline.py`: Fast/Accurate/Form/Handwriting |
| P5-04 | UI transparency | `uiStore.ts`, footer/status badges |

### 6.6 Phase P6 Export Master and Release

Nguon:

- `v3/08_execution_phases/06_phase_export_master_and_release.md`

| Ma | Muc tieu | Code map |
|---|---|---|
| P6-01 | Export studio hoan chinh | `ExportPage.tsx`, `export_cmd.rs` |
| P6-02 | Master dossier compiler | `06_reference_samples`, parser/export pipeline |
| P6-03 | Manifest verification | startup self-check |
| P6-04 | Release gate | `npm run tauri:build`, installer manifest |

---

## 7. UI maps bat buoc

### 7.1 UI map tong the

```text
AppShell
├── DossierSidebar
│   ├── Dashboard
│   ├── Dua ho so vao
│   ├── Scan tai lieu
│   ├── Phan tich tai lieu
│   ├── Quan ly tai lieu
│   ├── Phan tich AI
│   ├── Tim kiem
│   └── Settings gear (tach rieng duoi cung)
├── Header / Global Search
└── Workspace route
    ├── DashboardPage
    ├── ImportJobPage
    ├── ScanPage
    ├── AnalyzePage
    ├── DocumentListPage
    ├── SearchPage
    ├── AiWorkspacePage
    ├── ExportPage
    └── SettingsPage
```

### 7.2 Startup Gate

```text
App launch
  -> run_startup_self_check()
      -> check OS/RAM/CPU/disk/workspace/SQLite/pdfium/OCR/models/manifests
  -> fatal fail?
      -> StartupGatePage shows blocker + fix instruction
  -> warning only?
      -> allow app, show warning badge
  -> pass?
      -> Dashboard
```

UI yeu cau:

- Khong hien Dashboard neu fatal gate fail.
- Moi check co status: `pass`, `warning`, `fail`.
- Fail phai co action: mo folder, mo settings, retry, copy diagnostics.

### 7.3 Import ho so

```text
Nguoi dung chon file/folder
  -> validate extension
  -> copy vao managed original folder
  -> create/import case if valid
  -> insert documents/pages placeholders
  -> OCR queue
  -> review queue neu loi/thieu du lieu
  -> DocumentList/Viewer
```

UI yeu cau:

- Khong luu case rong neu khong co file hop le.
- Chi nhan PDF/images/Office theo policy.
- File Office duoc luu goc, khong OCR truc tiep neu chua co parser/convert.
- File mat phai do tren case/document.

### 7.4 Ricoh Scan Center

```text
Ricoh MFP
  -> Scan to Folder
  -> Scan Inbox Folder
  -> watcher debounce + file stability check
  -> duplicate hash check
  -> preview
  -> import into selected/default case
  -> OCR/classify queue
```

UI yeu cau:

- Trang thai ro: dang theo doi, phat hien file, cho ghi xong, san sang import, dang OCR, duplicate, loi.
- Khong hien nhu scan truc tiep neu chua co WIA/TWAIN that.

### 7.5 Document Viewer

```text
Viewer
├── Document tabs
├── Toolbar: page, zoom, fit, rescan, OCR page/all, AI analyze, rebuild index
├── Left: thumbnails
├── Center: PDF/image canvas
│   └── OCR overlay blocks
└── Right: OCR text / extracted fields / notes / citations
```

Mode:

| Mode | Muc dich |
|---|---|
| PDF only | Doc ban scan goc |
| OCR only | Sua text OCR |
| Split | So sanh scan goc va OCR |

Tuong tac:

- Click thumbnail -> jump page.
- Click OCR block -> select text in panel.
- Click OCR text -> jump bbox tren page.
- Low confidence block -> vien vang/do, cho sua.
- Save OCR -> update DB + rebuild FTS.

### 7.6 Search -> Viewer -> Page/Block

```text
Global Search
  -> fts_search(query)
  -> result has document_id + page_number + snippet + bbox/block if available
  -> hover shows page preview
  -> click opens /cases/:caseId/docs/:documentId?page=N&block=B
  -> viewer jumps page and highlights match
```

Ket qua search phai co toi thieu:

- Ten ho so
- Ten tai lieu
- So trang
- Snippet highlight
- Loai block neu co
- Confidence neu co

### 7.7 AI Workspace

```text
AI Workspace
├── Left: case/document/page/source list
├── Center: PDF Viewer
└── Right: AI Notebook
    ├── summary
    ├── ask case/document
    ├── citations
    └── extracted facts
```

AI khong duoc la chatbot roi rac. Moi cau tra loi phai co source:

```json
{
  "document_id": "doc-...",
  "page_number": 12,
  "quote": "doan trich lien quan",
  "confidence": 0.91
}
```

Click source phai mo dung document/page.

### 7.8 Export Studio

```text
ExportPage
├── Case selector
├── Document picker
├── Reorder list
├── Validation panel
│   ├── review pending
│   ├── missing file
│   ├── invalid citation
│   └── OCR missing
├── Output options
│   ├── cover
│   ├── table of contents
│   ├── page number
│   ├── include OCR layer
│   └── include audit/citation manifest
└── Export jobs
```

Goi export chuan:

```text
exports/<EXPORT_JOB_ID>_<TIMESTAMP>/
├── 00_index_trich_dan.docx
├── 01_bao_cao_tong_hop.docx
├── 02_so_do_vu_an.html
├── 03_tai_lieu_quan_ly/
├── 04_phu_luc_citation.json
└── 05_audit_export.json
```

---

## 8. Code maps hien tai

### 8.1 Frontend map

| Vung | File chinh |
|---|---|
| App shell/routes/sidebar | `PhanMem/src/App.tsx` |
| Entry | `PhanMem/src/main.tsx` |
| Dashboard | `PhanMem/src/components/pages/DashboardPage.tsx` |
| Case list/detail | `CaseListPage.tsx`, `CaseDetailPage.tsx` |
| Import | `ImportJobPage.tsx`, `importService.ts` |
| Scan/Ricoh/pipeline | `ScanPage.tsx`, `scanService.ts` |
| Analyze/review | `AnalyzePage.tsx`, `ReviewQueuePage.tsx`, `reviewService.ts` |
| Documents | `DocumentListPage.tsx`, `DocumentViewerPage.tsx`, `DocumentViewer.tsx` |
| PDF thumbnails | `PdfPageThumbnail.tsx` |
| Search | `SearchPage.tsx`, `searchService.ts` |
| AI | `AiWorkspacePage.tsx`, `AiNotebookPanel.tsx`, `aiService.ts` |
| Export | `ExportPage.tsx`, `exportService.ts` |
| Settings | `SettingsPage.tsx`, `appSettingService.ts` |
| Stores | `uiStore.ts`, `moduleStore.ts`, `catalogStore.ts` |
| Styles | `variables.css`, `layout.css`, `dossier-nav.css`, `components.css` |

Frontend service pattern:

```ts
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

Neu command co nhieu tham so, dong bo casing JS/Rust ngay trong service. Khong de UI goi `invoke` truc tiep neu da co service.

### 8.2 Backend Rust command map

| Module | File | Vai tro |
|---|---|---|
| Case | `PhanMem/src-tauri/src/commands/case_cmd.rs` | list/create/delete/purge dossier |
| Document | `doc_cmd.rs` | import one doc, OCR, OCR edit, layout, fields, rename, notes |
| Import | `import_cmd.rs` | import folder, import multiple files |
| Scan | `scan_cmd.rs` | Ricoh settings, inbox, watcher, pipeline jobs |
| Search | `search_cmd.rs` | FTS search |
| AI | `ai_cmd.rs` | status, summarize, ask case |
| Export | `export_cmd.rs` | PDF/export bundle |
| Review | `review_cmd.rs` | review queue/action |
| Catalog | `catalog_cmd.rs` | folder catalog scan |
| Module/settings | `module_cmd.rs` | feature toggles, app settings |
| Registration | `main.rs`, `commands/mod.rs` | register commands/state |

Rust command pattern:

```rust
#[tauri::command]
pub fn some_command(db: State<'_, DbState>, input: SomeInput) -> Result<SomeOutput, String> {
    let conn = db.0.lock().map_err(|_| "DB_LOCK_FAILED".to_string())?;
    // validate input
    // transaction if writing many tables
    // audit event if business state changes
    Ok(output)
}
```

Quy tac lock:

- Khong goi command khac trong khi dang giu DB mutex.
- Neu can query lai sau update, `drop(conn)` truoc.
- Moi import/OCR/export nen ghi audit event.

### 8.3 Database/migration map

| File | Vai tro |
|---|---|
| `PhanMem/src-tauri/src/db/mod.rs` | load migrations |
| `PhanMem/src-tauri/src/db/schema.rs` | schema constants neu co |
| `001_init_schema.sql` | core cases/documents/pages/FTS |
| `002_module_configs.sql` | module/settings |
| `003_scan_ricoh.sql` | scan settings/jobs |
| `004_pipeline_phase1.sql` | pipeline jobs/states |
| `005_ocr_layout_utf8.sql` | OCR layout unicode phase |
| `006_page_layout_extracted_fields.sql` | layout blocks/extracted fields |
| `007_file_lifecycle.sql` | can them theo P0-02 neu chua co |
| `008_ocr_normalized_text.sql` | can them theo P0-03 neu chua co |
| `009_parsed_sections.sql` | can them theo P2-03 |

Migration pattern:

```sql
ALTER TABLE page_layout_blocks ADD COLUMN normalized_text TEXT;
ALTER TABLE page_layout_blocks ADD COLUMN unicode_form TEXT NOT NULL DEFAULT 'NFC';

CREATE INDEX IF NOT EXISTS idx_page_layout_blocks_normalized
ON page_layout_blocks(document_id, page_number, block_type);
```

Sau khi them migration:

1. Them include trong `db/mod.rs`.
2. Dong bo `schema.rs` neu file nay co constants lien quan.
3. Kiem tra app khoi dong voi DB cu va DB moi.

### 8.4 Python OCR/parser/AI map

| File | Vai tro |
|---|---|
| `PhanMem/python/ocr/ocr_pipeline.py` | OCR, normalize, layout, extract fields |
| `PhanMem/python/ocr/pdf_to_images.py` | render PDF page images |
| `PhanMem/python/ocr/summarizer.py` | summary fallback |
| `PhanMem/python/ocr/verify_vietnamese_ocr.py` | verify OCR tieng Viet |
| `PhanMem/python/download_models.py` | model setup dev |
| `PhanMem/python/requirements.txt` | Python dependencies |
| `PhanMem/python/catalog/scanner.py` | catalog source folders |
| `PhanMem/python/parsers/docx_parser.py` | can tao o P2 |
| `PhanMem/python/parsers/html_parser.py` | can tao o P2 |

OCR block output toi thieu:

```json
{
  "document_id": "doc-...",
  "page_number": 1,
  "raw_text": "Số: 986/QĐ-CSĐT",
  "normalized_text": "Số: 986/QĐ-CSĐT",
  "unicode_form": "NFC",
  "bbox": { "x": 120, "y": 80, "width": 340, "height": 42 },
  "confidence": 0.92,
  "reading_order": 3,
  "block_type": "document_number",
  "engine": "paddleocr"
}
```

Field extraction output:

```json
{
  "field_name": "so_van_ban",
  "field_value": "986/QĐ-CSĐT",
  "page_number": 1,
  "x": 120,
  "y": 80,
  "width": 340,
  "height": 42,
  "confidence": 0.92,
  "source": "OCR+Rule"
}
```

---

## 9. Command playbook theo tinh huong

### 9.1 Khi nhan task moi

```powershell
rg --files .\v3
Get-Content -Raw .\v3\NEXT_ACTION.md
Get-Content -Raw .\v3\05_logs\20260429_phase-mvp-p0.md
```

Doi chieu:

- Task co thuoc active phase khong?
- Neu khong thuoc, co phai bug blocker khong?
- Co can update log truoc khi code khong?

### 9.2 Khi can tim spec lien quan, khong quet het file lon

```powershell
rg -n "OCR|citation|export|startup|file lifecycle|scan|Notebook|viewer" .\v3\03_specs .\v3\standards .\v3\08_execution_phases
```

Doi chieu:

- Lay heading/file lien quan truoc.
- Mo dung file lien quan bang `Get-Content -Raw`.
- Khong paste `03_schema.json` nguyen file.

### 9.3 Khi can doi chieu code frontend/backend

```powershell
rg -n "run_ocr_for_document|rescan_document_ocr|fts_search|export_pdf_bundle|purge_imported_dossier" .\PhanMem\src .\PhanMem\src-tauri\src
rg --files .\PhanMem\src\components\pages .\PhanMem\src\services .\PhanMem\src-tauri\src\commands
```

Doi chieu:

- Moi UI action co service khong?
- Service co command Rust tuong ung khong?
- Command co dang ky trong `main.rs` khong?
- Command ghi DB/audit/index day du khong?

### 9.4 Khi test runtime app

```powershell
cd PhanMem
npm run tauri:dev
```

Doi chieu:

- Cua so desktop mo duoc.
- Tauri invoke hoat dong.
- Khong co warning "running outside Tauri" khi test chuc nang.
- Import/OCR/Search/Viewer/Export dung runtime that.

### 9.5 Khi build release

```powershell
cd PhanMem
npm run tauri:build
```

Doi chieu:

- Installer tao thanh cong trong `src-tauri/target/release/bundle/`.
- Startup Gate pass tren ban release.
- Du manifest/model/runtime neu phase yeu cau.

### 9.6 Static check ho tro, khong thay the Tauri runtime

Co the dung de bat loi code nhanh:

```powershell
cd PhanMem
npm exec tsc -- --noEmit
cd src-tauri
cargo check
```

Nhung khong duoc ket luan nghiep vu pass neu chua test trong Tauri runtime.

---

## 10. Doi chieu source hien tai voi V3

Bang nay la diem neo de agent khong doc lai toan bo source khi khong can.

| Yeu cau V3 | Hien trang doi chieu nhanh | Gap can theo P0/P2/P6 |
|---|---|---|
| Tauri desktop `.exe` | `PhanMem/src-tauri`, React/Vite trong WebView | Giu nguyen, khong tao web app |
| Sidebar dossier workspace | Co `App.tsx`, `dossier-nav.css` | Tiep tuc toi uu UX, khong bien thanh web admin |
| Import folder/multi-file | Co `import_cmd.rs`, `importService.ts` | Doi chieu lifecycle per-case theo P0-02 |
| Ricoh scan to folder | Co `scan_cmd.rs`, `ScanPage.tsx`, `scanService.ts` | Nang gate/status/profile theo P5/P7 sau |
| OCR offline | Co `ocr_pipeline.py`, `run_ocr_for_document`, `rescan_document_ocr` | P0-03 sync `normalized_text`, schema constants, verify FTS |
| Layout/extracted fields | Co migration `006_page_layout_extracted_fields.sql` va commands list blocks/fields | Can harden bbox/field review UI |
| Search FTS | Co `search_cmd.rs`, `searchService.ts` | Can bbox/block highlight day du theo phase search nang cao |
| Viewer | Co `DocumentViewer.tsx`, thumbnails, OCR panel | Can hoan thien overlay/edit-by-block neu thieu |
| AI Notebook | Co `AiWorkspacePage`, `AiNotebookPanel`, `ai_cmd.rs` | P0-04 citation validator hard-stop |
| Export | Co `export_cmd.rs`, `ExportPage.tsx` | P0-05/P6 export package chuan |
| Delete/purge dossier | Co `purge_imported_dossier` va UI settings/case list | Can giu confirm nghiem ngat, audit ro |
| Startup Gate | Chua thay command `system_cmd.rs` trong map hien tai | P0-01 la blocker nen lam dau |
| DOCX/HTML parser | Chua co `python/parsers` theo map | P2 |

---

## 11. Maps chi tiet theo module nghiep vu

### 11.1 Dashboard

Nguon spec:

- `03_specs/20260424_15_ban_do_giao_dien_tong_the_va_dieu_huong_chinh.md`
- `03_specs/20260424_25_ban_do_tri_thuc_man_hinh_tom_tat_bi_can.md`

Can hien:

- Tong ho so
- Tong tai lieu
- Tai lieu chua OCR
- OCR loi
- Chua phan loai
- Can review
- Import/OCR/export dang chay
- File missing
- Canh bao citation/export

Code map:

- UI: `DashboardPage.tsx`
- Data: `caseService.ts`, `documentService.ts`, `scanService.ts`
- Backend: `case_cmd.rs`, `doc_cmd.rs`, `scan_cmd.rs`

### 11.2 Dua ho so vao

Nguon spec:

- `03_specs/20260424_22_dac_ta_tach_file_ingest_db_metadata.md`
- `standards/04_file_lifecycle_export.md`

Code map:

- UI: `ImportJobPage.tsx`
- Service: `importService.ts`, `documentService.ts`
- Backend: `import_cmd.rs`, `doc_cmd.rs`, `storage.rs`
- DB: documents/pages/import jobs/audit/FTS

Pass khi:

- Import folder nhieu file co trang thai thanh cong/trung/loi.
- File goc copy vao managed storage, khong dung truc tiep thu muc nguoi dung.
- Khong tao case rong khi folder khong co file hop le.

### 11.3 Scan tai lieu

Nguon spec:

- Ricoh scan to folder la uu tien thuc te.
- Direct TWAIN/WIA chi la phase sau neu co driver/helper.

Code map:

- UI: `ScanPage.tsx`
- Service: `scanService.ts`
- Backend: `scan_cmd.rs`
- Storage: `storage.rs`

Pass khi:

- Chon scan inbox folder.
- Watch folder co debounce/file stability.
- Preview file moi scan.
- Import batch vao case dung.
- Duplicate/hash status ro.

### 11.4 Phan tich tai lieu/OCR

Nguon spec:

- `standards/03_pipeline_ocr_ai.md`
- `03_specs/20260424_21_dac_ta_luong_scan_ocr_ai_danh_gia_chat_luong.md`

Code map:

- UI: `AnalyzePage.tsx`, `ReviewQueuePage.tsx`, `DocumentViewer.tsx`
- Service: `documentService.ts`, `reviewService.ts`
- Backend: `doc_cmd.rs`, `review_cmd.rs`
- Python: `ocr_pipeline.py`, `pdf_to_images.py`

Pass khi:

- OCR co text theo page.
- Layout block co bbox/confidence.
- Low confidence vao review.
- Sua OCR rebuild index.

### 11.5 Quan ly tai lieu

Nguon spec:

- Document card/dossier workspace.

Code map:

- UI: `DocumentListPage.tsx`, `CaseDetailPage.tsx`
- Service: `documentService.ts`
- Backend: `doc_cmd.rs`

Pass khi:

- Card/list/tree view neu phase yeu cau.
- Status badges: chua OCR, da OCR, OCR loi, chua phan loai, da phan loai, can review, file missing.
- Quick open/rename/export/split-merge neu da vao phase.

### 11.6 Tim kiem

Nguon spec:

- Search toan cuc, click dung page, hover preview.

Code map:

- UI: `SearchPage.tsx`, `DocumentViewer.tsx`
- Service: `searchService.ts`
- Backend: `search_cmd.rs`
- DB: FTS tables + pages/layout blocks

Pass khi:

- Result co `document_id`, `page_number`, snippet.
- Neu co bbox/block thi viewer highlight dung vung.
- Search tim title, OCR text, extracted fields, summary.

### 11.7 AI Notebook

Nguon spec:

- AI gan chat voi document/citation, khong chatbot roi rac.

Code map:

- UI: `AiWorkspacePage.tsx`, `AiNotebookPanel.tsx`, `DocumentViewer.tsx`
- Service: `aiService.ts`
- Backend: `ai_cmd.rs`
- DB: summaries/citations/extracted fields/review queue

Pass khi:

- AI answer co citations.
- Citation validator pass.
- Click source mo dung document/page.
- Thieu citation thi chuyen review, khong xuat ket luan.

### 11.8 Export PDF/DOCX/HTML

Nguon spec:

- `standards/04_file_lifecycle_export.md`
- `06_reference_samples/`

Code map:

- UI: `ExportPage.tsx`
- Service: `exportService.ts`
- Backend: `export_cmd.rs`
- Python/parser/export helpers neu can

Pass khi:

- Chon tai lieu/nhom ho so.
- Reorder truoc khi export.
- Tao package day du.
- Co audit/citation manifest.
- Khong export neu missing file hoac citation fatal.

---

## 12. Vi du lenh dieu phoi ngan gon cho agent khac

Dung cac prompt ngan gon nay de khong bat agent khac quet ca repo.

### 12.1 P0-01 Startup Gate

```text
Doc chi cac file:
- v3/TONGHOP.md
- v3/NEXT_ACTION.md
- v3/08_execution_phases/01_phase_p0_foundation.md phan P0-01
- v3/standards/02_runtime_offline_requirements.md
- PhanMem/src-tauri/src/main.rs
- PhanMem/src-tauri/src/commands/mod.rs
- PhanMem/src/App.tsx

Hay implement Startup Environment Gate. Khong chay npm run dev. Test bang npm run tauri:dev hoac static check neu chua mo runtime.
```

### 12.2 P0-02 File Lifecycle

```text
Doc chi:
- v3/TONGHOP.md
- v3/standards/04_file_lifecycle_export.md
- v3/08_execution_phases/01_phase_p0_foundation.md phan P0-02
- PhanMem/src-tauri/src/storage.rs
- PhanMem/src-tauri/src/commands/import_cmd.rs
- PhanMem/src-tauri/src/commands/doc_cmd.rs
- PhanMem/src-tauri/src/commands/export_cmd.rs
- PhanMem/src-tauri/src/db/mod.rs

Bo sung lifecycle per-case original/processing/reviewed/managed/exports/logs. Khong ghi de file goc. Them migration neu can.
```

### 12.3 P0-03 OCR Unicode

```text
Doc chi:
- v3/TONGHOP.md
- v3/standards/03_pipeline_ocr_ai.md
- v3/08_execution_phases/01_phase_p0_foundation.md phan P0-03
- PhanMem/python/ocr/ocr_pipeline.py
- PhanMem/src-tauri/src/commands/doc_cmd.rs
- PhanMem/src-tauri/migrations/006_page_layout_extracted_fields.sql
- PhanMem/src-tauri/src/db/mod.rs
- PhanMem/src-tauri/src/db/schema.rs

Them normalized_text/unicode_form cho OCR block, sync DB/FTS, giu bbox/confidence. Khong quet thu muc anh test.
```

### 12.4 P0-04 Citation Validator

```text
Doc chi:
- v3/TONGHOP.md
- v3/standards/01_architecture.md
- v3/08_execution_phases/01_phase_p0_foundation.md phan P0-04
- PhanMem/src-tauri/src/commands/ai_cmd.rs
- PhanMem/src-tauri/src/commands/search_cmd.rs
- PhanMem/src/components/pages/AiWorkspacePage.tsx
- PhanMem/src/components/pages/AiNotebookPanel.tsx

Implement citation validation hard-stop: AI/export khong duoc tra ket luan neu source khong map duoc document_id/page/quote.
```

### 12.5 P0-05 Export Package

```text
Doc chi:
- v3/TONGHOP.md
- v3/standards/04_file_lifecycle_export.md
- v3/08_execution_phases/01_phase_p0_foundation.md phan P0-05
- v3/06_reference_samples danh sach file mau
- PhanMem/src-tauri/src/commands/export_cmd.rs
- PhanMem/src/services/exportService.ts
- PhanMem/src/components/pages/ExportPage.tsx

Tao export_dossier_package gom DOCX index, DOCX report, HTML graph, managed docs, citation JSON, audit JSON.
```

---

## 13. Mau ghi log ket qua

Them vao `v3/05_logs/YYYYMMDD_phase-<phase>.md`:

```md
### Agent: <name>
- Time: 2026-04-29 HH:mm
- Task: P0-03 OCR Unicode schema
- Status: DONE | BLOCKED | HOLD | ESCALATE
- Files touched:
  - `PhanMem/python/ocr/ocr_pipeline.py`
  - `PhanMem/src-tauri/migrations/008_ocr_normalized_text.sql`
- Evidence:
  - `npm exec tsc -- --noEmit`: PASS
  - `cargo check`: PASS
  - `npm run tauri:dev`: PASS/NOT RUN, reason
- Bugs found:
  - ...
- Next action:
  - ...
```

Khong duoc xoa phase/log khi chua co code + test + bug check PASS.

---

## 14. Mau doi chieu pass/fail theo task

### Import pass checklist

- [ ] Folder/file co file hop le.
- [ ] File goc duoc copy vao storage cua app, khong dung source folder lam noi luu chinh.
- [ ] Neu khong co file hop le, khong tao case rong.
- [ ] Duplicate hash duoc danh dau.
- [ ] PDF/images vao OCR queue.
- [ ] Office vao storage/parser queue, khong bao OCR thanh cong gia.
- [ ] UI hien thanh cong/trung/loi ro rang.

### OCR pass checklist

- [ ] Render page thanh image neu PDF scan.
- [ ] OCR text co dau tieng Viet dung Unicode NFC.
- [ ] Co `raw_text` + `normalized_text`.
- [ ] Co bbox/confidence/reading_order/page_number.
- [ ] Low confidence duoc danh dau review.
- [ ] `pages.ocr_text`, `ocr_results`, `page_layout_blocks`, FTS duoc sync.
- [ ] Viewer hien text/overlay dung page.

### Citation pass checklist

- [ ] Moi source co `document_id`.
- [ ] Moi source co `page_number`.
- [ ] Quote ton tai trong OCR/page/block.
- [ ] Click source mo dung document/page.
- [ ] Source sai thi khong export/khong ket luan AI.

### Export pass checklist

- [ ] Co pre-export validator.
- [ ] Khong co missing file fatal.
- [ ] Khong co citation invalid fatal.
- [ ] Output co du 6 thanh phan package.
- [ ] Audit export ghi input, order, options, time.
- [ ] User co duong dan folder/file export trong UI desktop.

---

## 15. Vi du code snippet theo huong V3

### 15.1 Startup self-check output

```rust
#[derive(serde::Serialize)]
pub struct StartupCheckItem {
    pub key: String,
    pub label: String,
    pub status: String, // pass | warning | fail
    pub message: String,
    pub fatal: bool,
}

#[derive(serde::Serialize)]
pub struct StartupSelfCheckResult {
    pub overall_status: String,
    pub can_enter_app: bool,
    pub items: Vec<StartupCheckItem>,
}
```

### 15.2 Frontend gate render

```tsx
if (startup && !startup.can_enter_app) {
  return <StartupGatePage result={startup} onRetry={runStartupCheck} />;
}

return <AppShell />;
```

### 15.3 Citation validator rule

```rust
fn validate_citation(conn: &Connection, citation: &AiSource) -> Result<(), String> {
    if citation.document_id.trim().is_empty() {
        return Err("CITATION_MISSING_DOCUMENT_ID".to_string());
    }
    if citation.page_number < 1 {
        return Err("CITATION_INVALID_PAGE_NUMBER".to_string());
    }
    // verify page exists and quote/snippet can be traced
    Ok(())
}
```

### 15.4 OCR normalized text in Python

```python
import unicodedata

def normalize_text(value: str) -> str:
    return unicodedata.normalize("NFC", value or "").strip()

block = {
    "raw_text": raw_text,
    "normalized_text": normalize_text(raw_text),
    "unicode_form": "NFC",
    "bbox": bbox,
    "confidence": confidence,
    "page_number": page_number,
}
```

---

## 16. Reference sample mapping

| Sample | Vai tro | Phase |
|---|---|---|
| `Bang_chi_muc_trich_dan_tai_lieu.docx` | Mau chi muc trich dan tai lieu | P0-05/P6 |
| `Bao_cao_tong_hop_vu_an_LeThanhCong_D134.docx` | Mau bao cao tong hop vu an | P6 |
| `So_do_vu_an_LeThanhCong_D134.html` | Mau so do vu an/quan he | P6 |
| `01_export_templates/` | Template export neu co | P6 |

Khi lam export/master dossier, can doc mau dau ra va map nguoc:

```text
OCR/pages/layout/extracted_fields
  -> citations
  -> index trich dan
  -> case summary
  -> relationship graph
  -> export package
```

---

## 17. Quy tac khong quet thua token

Neu giao task cho agent khac, chi dua:

1. File V3 lien quan.
2. File code lien quan.
3. Ket qua mong muon.
4. Lenh test hop le.

Khong dua:

- Duong dan anh/test file local neu agent khong co.
- Yeu cau "doc ca repo" neu task chi can 5 file.
- `03_schema.json` nguyen file.
- `07_external_refs` nguyen cay.

Vi du sai:

```text
Doc toan bo repo, sua OCR viet tay, day la duong dan anh test cua may toi...
```

Vi du dung:

```text
Sua OCR handwriting fallback. Doc:
- v3/TONGHOP.md
- v3/standards/03_pipeline_ocr_ai.md
- PhanMem/python/ocr/ocr_pipeline.py
- PhanMem/src-tauri/src/commands/doc_cmd.rs
Yeu cau: neu handwriting confidence thap, danh dau possible_handwriting + review_pending, khong bao OCR success gia.
```

---

## 18. Ket luan dieu phoi

Trang thai V3 hien tai:

- V3 da co he quy chieu ro: governance, standards, specs, phase, logs, samples.
- Active phase la MVP P0 Foundation.
- Viec can lam truoc la Startup Gate + lifecycle + OCR Unicode/citation/export.
- Moi task moi nen duoc chuyen thanh ma task, code map, UI map, DB map, test evidence.

Huong xu ly chuan:

```text
Nhan lenh
  -> map vao phase/task
  -> doc dung file lien quan
  -> doi chieu code map
  -> implement nho gon
  -> test trong Tauri runtime neu co chuc nang
  -> update log
  -> bao ket qua bang file/command/evidence
```

File nay khong thay the cac spec chi tiet. No la ban dieu phoi de giam viec doc lap lai va giu agent khong di lech kien truc desktop offline.
