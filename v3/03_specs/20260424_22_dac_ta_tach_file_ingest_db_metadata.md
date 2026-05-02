# ĐẶC TẢ TÁCH FILE VÀ INGEST VÀO DATABASE METADATA

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa luồng tách file, ingest vào database và metadata extraction.
Nguon prompt: Tu yeu cau ve file splitting va ingest.

---

## 1. Tong Quan File Splitting & Ingest

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal splitting spec**
- **KHÔNG có ingest pipeline**
- **KHÔNG có metadata extraction**

### 1.2 Muc Tieu Cua File Nay

- File splitting logic
- Database ingest pipeline
- Metadata extraction
- Data validation

---

## 2. File Splitting Logic

### 2.1 Splitting Options

| Option | Description | Use Case |
|--------|------------|---------|
| **Single** | One file = One document | Normal import |
| **Multi-page** | Split by pages | Large documents |
| **Detection** | Auto-detect documents | Multiple docs in 1 file |

### 2.2 Splitting Flow

```
Input File
    │
    ▼
┌─────────────────────┐
│ Detect Document    │
│ Type             │
├──────────────────┤
│ Single → Single  │
│ PDF → Continue  │
│ Multi → Split   │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ Auto-Detection   │
│ (Optional mode)  │
├──────────────────┤
│ Page markers    │
│ Blank pages   │
│ Metadata    │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ Output         │
│ [Document 1]  │
│ [Document 2]  │
│ ...           │
└─────────────────────┘
```

### 2.3 Page Splitting

```
Multi-page PDF (100 pages)
         │
         ▼
┌───────────────────────────────────────┐
│ Option 1: Single Document              │
│ All pages → One document              │
├───────────────────────────────────────┤
│ Option 2: Each Page = Document      │
│ 100 documents (1 page each)          │
├───────────────────────────────────────┤
│ Option 3: Batch Pages               │
│ 10 documents (10 pages each)       │
├───────────────────────────────────────┤
│ Option 4: Auto-detect               │
│ Break on: new page, blank,         │
│ page number jump, metadata change     │
└───────────────────────────────────────┘
```

---

## 3. Database Ingest Pipeline

### 3.1 Ingest Flow

```
File(s) Selected
         │
         ▼
┌─────────────────────┐
│ Preprocess         │
│ (normalize, dedup) │
└─────────┬──────────┘
          │
          ▼
┌─────────────────────┐
│ OCR Processing      │
│ (per page)        │
└─────────┬──────────┘
          │
          ▼
┌─────────────────────┐
│ Metadata Extract  │
│ (auto-detect)     │
└─────────┬──────────┘
          │
          ▼
┌─────────────────────┐
│ Validation       │
│ (check required) │
└─────────┬──────────┘
          │
          ├─────────────────┐
          ▼               ▼
    ┌──────────┐    ┌──────────┐
    │ Valid   │    │ Invalid │
    └──────────┘    └──────────┘
          │               │
          ▼               ▼
    ┌──────────┐    ┌──────────┐
    │Store DB │    │Review   │
    │+ Index │    │+ Fix    │
    └──────────┘    └──────────┘
```

### 3.2 Insert Sequence

| Step | Entity | Action |
|------|--------|--------|
| 1 | Case | Create if not exists |
| 2 | Document | Insert metadata |
| 3 | Pages | Insert page data |
| 4 | OCR | Store OCR results |
| 5 | Entities | Extract from text |
| 6 | Citations | Create anchors |
| 7 | Search | Update index |

### 3.3 Transaction Pattern

```sql
BEGIN TRANSACTION;

-- 1. Create document
INSERT INTO documents (document_id, case_id, ...)
VALUES (?, ?, ...);

-- 2. Create pages
INSERT INTO pages (page_id, document_id, page_index, ...)
VALUES ...;

-- 3. Update case count
UPDATE cases 
SET document_count = document_count + 1, 
    updated_at = NOW()
WHERE case_id = ?;

COMMIT;
```

---

## 4. Metadata Extraction

### 4.1 Auto-extracted Fields

| Field | Source | Method |
|-------|--------|--------|
| document_type | Content | Pattern matching |
| issued_date | Content | Date extraction |
| issued_by | Content | Org name database |
| document_title | Filename + Content | First line parsing |
| summary_short | Content | AI extraction (50 chars) |

### 4.2 Pattern Matching Rules

| Document Type | Patterns |
|--------------|----------|
| to_khai | "TO KHAI", "Khai", "Tờ khai" |
| bien_ban | "BIÊN BẢN", "BB", "Bien ban" |
| quyet_dinh | "QUYẾT ĐỊNH", "QD", "Quyet dinh" |
| ket_luan | "KẾT LUẬN", "Ket luan" |
| phieu | "PHIẾU", "Phieu", "Giấy" |

### 4.3 Date Extraction

```
Content: "Tai ngay 20 thang 01 nam 2026"
         │
         ▼
Regex: (\d{1,2})\s*(tháng|thang)\s*(\d{1,2})\s*(nam|năm)\s*(\d{4})
         │
         ▼
Parsed: 2026-01-20
```

### 4.4 Entity Extraction

```
OCR Text Input
         │
         ▼
┌─────────────────────┐
│ Named Entity     │
│ Recognition    │
├──────────────────────┤
│ Person names   │
│ Organization  │
│ Dates         │
│ Addresses    │
│ Numbers      │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ Create Entities   │
│ + Link to        │
│ Document        │
└─────────────────────┘
```

---

## 5. Data Validation

### 5.1 Required Fields

| Entity | Required Fields |
|--------|----------------|
| Case | case_code, primary_person_name |
| Document | document_type, display_name |
| Page | page_index |

### 5.2 Validation Rules

```
┌─────────────────────────────────────┐
│         VALIDATION LAYERS            │
├────────────────────────────────────���┤
│                                      │
│ Layer 1: Type Check                │
│ (field type correct)                 │
│                                      │
│ Layer 2: Required Check            │
│ (all required fields)               │
│                                      │
│ Layer 3: Format Check             │
│ (date format, number)               │
│                                      │
│ Layer 4: Reference Check          │
│ (foreign key exists)                 │
│                                      │
│ Layer 5: Business Check           │
│ (custom rules)                      │
│                                      │
└─────────────────────────────────────┘
```

### 5.3 Error Handling

| Validation Failed | Action |
|------------------|--------|
| Required field | Prompt user |
| Format error | Auto-fix if possible |
| Reference missing | Create or error |
| Business rule | Flag for review |

---

## 6. Search Indexing

### 6.1 Index Update Flow

```
Document Inserted
         │
         ▼
┌─────────────────────┐
│ Generate Index    │
│ (FTS tokens)    │
└─────────┬──────────┘
          │
          ▼
┌─────────────────────┐
│ Update Index      │
│ (search-index) │
└─────────┬──────────┘
          │
          ▼
┌─────────────────────┐
│ Update Citation │
│ Graph        │
└─────────┬──────────┘
          │
          ▼
┌─────────────────────┐
│ Search Ready   │
└─────────────────────┘
```

### 6.2 FTS Configuration

| Field | Indexed | Tokenized |
|-------|---------|----------|
| document_title | ✅ | ✅ |
| summary_short | ✅ | ✅ |
| ocr_text | ✅ | ✅ |
| but_luc | ✅ | ❌ |
| person_name | ✅ | ✅ |

---

## 7. Import Job Management

### 7.1 Job States

| State | Description |
|-------|------------|
| created | Job tao moi |
| scanning | Scanning files |
| importing | Dang import |
| ocr_processing | Dang OCR |
| indexing | Dang index |
| paused | Tam dung |
| failed | That bai |
| completed | Hoan thanh |
| cancelled | Huy |

### 7.2 Job Progress

```
┌─────────────────────────────────────┐
│ Import Job Progress                   │
├─────────────────────────────────────┤
│                                      │
│ Case: VK-2026-00123                │
│ Status: ocr_processing               │
│                                      │
│ Progress: ████████░░░░░░░ 67%    │
│ ───────────────────────────────────│
│ Files: 20/30                      │
│ Pages: 134/200                    │
│                                      │
│ Current: document_15.png           │
│                                      │
│ ───────────────────────────────────│
│ [Pause] [Cancel] [Continue]        │
└─────────────────────────────────────┘
```

### 7.3 Pause/Resume

```
User clicks Pause
         │
         ▼
┌─────────────────────┐
│ Save Current State │
│ (to database)   │
├──────────────────┤
│ Mark as paused │
│ + Resume point │
└─────────────────────┘

Resume Later
         │
         ▼
┌─────────────────────┐
│ Load State    │
│ + Continue │
│ from point │
└─────────────────────┘
```

---

## 8. State Management

### 8.1 Import Store

```typescript
interface ImportState {
  jobs: ImportJob[];
  currentJob: ImportJob | null;
  progress: ImportProgress;
  errors: ImportError[];
  
  // Actions
  createJob: (caseId: string, files: File[]) => Promise<Job>;
  resumeJob: (jobId: string) => Promise<void>;
  cancelJob: (jobId: string) => Promise<void>;
}
```

### 8.2 Progress Tracking

```typescript
interface ImportProgress {
  totalFiles: number;
  processedFiles: number;
  totalPages: number;
  processedPages: number;
  failedFiles: string[];
  currentFile: string;
}
```

---

## 9. Acceptance Criteria

- [ ] File splitting logic day du
- [ ] Database ingest pipeline hoan chinh
- [ ] Metadata extraction tu dong
- [ ] Data validation day du
- [ ] Search indexing tich hop
- [ ] Import job management

---

## 10. Khong Duoc Hieu Sai

- **Splitting KHONG phai la PDF library** - chi logic, library rieng
- **Ingest KHONG phai la database** - chi pipeline, DB rieng
- **Metadata KHONG phai la OCR** - chi extraction, OCR rieng

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom D - Pipeline*
*Tiep theo: File 11 - Dac ta profile bi can*