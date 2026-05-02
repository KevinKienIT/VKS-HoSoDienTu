# BẢN ĐỒ LUỒNG CHỨC NĂNG END-TO-END

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa luồng chức năng end-to-end cho các tác vụ chính của hệ thống VKS ECMS.
Nguon prompt: Tu yeu cau ve functional flows.

---

## 1. Tong Quan Functional Flows

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal functional flow diagrams**
- **KHÔNG có user journey maps**
- **KHÔNG có use case specifications**

### 1.2 Muc Tieu Cua File Nay

- Tạo end-to-end flow diagrams
- User journey maps
- Use case specifications
- Integration points

---

## 2. Main User Journeys

### 2.1 Journey 1: Import Case

```
┌─────────────────────────────────────────────────────────────────────┐
│                JOURNEY: IMPORT NEW CASE                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Nguoi dung chon "Import Case"                      [Button]  │
│         │                                                      │
│         ▼                                                      │
│  2. Chon thu muc chua tai lieu                      [Folder]  │
│         │                                                      │
│         ▼                                                      │
│  3. Nhap thong tin case                          [Form]     │
│     - Ma case                                                │
│     - Ten bi can                                             │
│     - Loai                                                  │
│         │                                                      │
│         ▼                                                      │
│  4. Scan tai lieu + OCR                       [Progress]   │
│         │                                                      │
│         ▼                                                      │
│  5. Review tu dong (neu can)                [Review]    │
│         │                                                      │
│         ▼                                                      │
│  6. Hoan thanh                             [Success]  │
│                                                              │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Journey 2: Review OCR

```
┌─────────────────────────────────────────────────────────────────────┐
│              JOURNEY: REVIEW OCR                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Vao queue review                        [Menu]     │
│         │                                                      │
│         ▼                                                      │
│  2. Chon item can review                  [List]    │
│         │                                                      │
│         ▼                                                      │
│  3. Xem side-by-side (image + OCR)           [View]    │
│         │                                                      │
│         ▼                                                      │
│  4. Duyet hoac chinh sua                     [Action]  │
│         │                                                      │
│         ├──────────┬──────────┬──────────┐                    │
│         ▼         ▼         ▼        ▼                     │
│     [Approve] [Edit]  [Reject] [Skip]                    │
│         │                                                      │
│         ▼                                                      │
│  5. Move to next item                    [Loop]    │
│         │                                                      │
│         ▼                                                      │
│  6. Hoan thanh queue                    [Done]    │
│                                                              │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.3 Journey 3: Create Citation

```
┌─────────────────────────────────────────────────────────────────────┐
│            JOURNEY: CREATE CITATION                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Mo tai lieu can trich dan               [Viewer]  │
│         │                                                      │
│         ▼                                                      │
│  2. Chon van ban can trich                [Select] │
│         │                                                      │
│         ▼                                                      │
│  3. Right-click → Create Citation       [Menu]    │
│         │                                                      │
│         ▼                                                      │
│  4. Dien form trich dan                     [Form]  │
│     - Trich dan                                                 │
│     - Loai                                                    │
│     - Lien ket doi tuong                                        │
│         │                                                      │
│         ▼                                                      │
│  5. Luu trich dan                       [Save]   │
│         │                                                      │
│         ▼                                                      │
│  6. Hien thi trong citation panel        [Done]  │
│                                                              │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.4 Journey 4: Search & Filter

```
┌─────────────────────────────────────────────────────────────────────┐
│           JOURNEY: SEARCH DOCUMENTS                    │
├─────────��───────────────────────────────────────────────────────────┤
│                                                              │
│  1. Nhap tu khoa tim kiem                [Input]   │
│         │                                                      │
│         ▼                                                      │
│  2. (Optional) Ap dung bo loc          [Filter] │
│         │                                                      │
│         ▼                                                      │
│  3. Thuc hien tim kiem                   [Search] │
│         │                                                      │
│         ▼                                                      │
│  4. Hien thi ket qua                       [List]  │
│         │                                                      │
│         ▼                                                      │
│  5. Chon item xem chi tiet              [Click] │
│         │                                                      │
│         ▼                                                      │
│  6. Mo trong viewer                    [View]  │
│                                                              │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Data Flow Diagrams

### 3.1 Import Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                 IMPORT DATA FLOW                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                              │
│  User       Application      Domain         Data         File      │
│   │             │             │             │            │         │
│   ▼             ▼             ▼             ▼            ▼     │
│ ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌────────┐  ┌─────┐ │
│ │Select  │──│Import  │──│Validate │──│Insert │──│Save │ │
│ │Folder │  │Service│  │Service │  │Service│  │File │ │
│ └─────────┘  └─────────┘  └─────────┘  └────────┘  └─────┘ │
│    │                                                 │        │
│    ▼                                                 ▼        │
│ ┌─────────┐                                   ┌─────────┐   │
│ │Start   │                                   │Complete│   │
│ │Process │                                   │(OK)   │   │
│ └─────────┘                                   └─────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 OCR Processing Flow

```
┌───────────────────────────────────────────────────���─���───────────────┐
│              OCR PROCESSING FLOW                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                              │
│  Source File                                                │
│       │                                                     │
│       ▼                                                     │
│ ┌─────────────┐                                             │
│ │PDF Extract │──► Images (per page)                       │
│ └─────────────┘                                             │
│       │                                                     │
│       ▼                                                     │
│ ┌─────────────┐                                             │
│ │Preprocess │──► Enhanced images                       │
│ │(denoise, │                                             │
│ │ contrast)│                                             │
│ └─────────────┘                                             │
│       │                                                     │
│       ▼                                                     │
│ ┌─────────────┐                                             │
│ │PaddleOCR │──► Raw text + confidence                    │
│ │Engine   │                                             │
│ └─────────────┘                                             │
│       │                                                     │
│       ▼                                                     │
│ ┌─────────────┐                                             │
│ │Post       │──► Structured OCR results                  │
│ │Process   │                                             │
│ │(format,  │                                             │
│ │ Vietnamese)│                                           │
│ └─────────────┘                                             │
│       │                                                     │
│       ├────────────────────┐                              │
│       ▼                    ▼                              │
│  High Confidence      Low Confidence                       │
│       │                    │                              │
│       ▼                    ▼                              │
│  Store Directly    → Review Queue                        │
│                                                              │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.3 Search Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                SEARCH FLOW                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                              │
│  User Input Query                                             │
│       │                                                     │
│       ▼                                                     │
│ ┌─────────────┐                                             │
│ │Parse       │──► Keywords + filters                        │
│ │Query      │                                             │
│ └─────────────┘                                             │
│       │                                                     │
│       ▼                                                     │
│ ┌─────────────┐                                             │
│ │Build      │──► SQL/FTS query                            │
│ │SQL        │                                             │
│ └─────────────┘                                             │
│       │                                                     │
│       ▼                                                     │
│ ┌─────────────┐                                             │
│ │Execute    │──► Results (documents)                     │
│ │Query     │                                             │
│ └─────────────┘                                             │
│       │                                                     │
│       ▼                                                     │
│ ┌─────────────┐                                             │
│ │Rank &     │──► Sorted results                          │
│ │Sort      │                                             │
│ └─────────────┘                                             │
│       │                                                     │
│       ▼                                                     │
│ ┌─────────────┐                                             │
│ │Display    │──► Results list                           │
│ │Results   │                                             │
│ └─────────────┘                                             │
│                                                              │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. Use Case Specifications

### 4.1 UC-001: Import Case

| Field | Value |
|-------|-------|
| Use Case ID | UC-001 |
| Name | Import Case |
| Actor | KSV |
| Preconditions | Da dang nhap |
| Postconditions | Case da duoc tao, tai lieu da import |
| Basic Flow | 1. Chon folder → 2. Scan files → 3. OCR → 4. Review → 5. Complete |
| Alternative Flow | Folder empty → Error message |
| Business Rules | Extension: pdf, jpg, png; Size < 50MB each |

### 4.2 UC-002: View Case Detail

| Field | Value |
|-------|-------|
| Use Case ID | UC-002 |
| Name | View Case Detail |
| Actor | KSV, Viewer |
| Preconditions | Da dang nhap, case ton tai |
| Postconditions | Hien thi thong tin case |
| Basic Flow | 1. Tu dashboard → 2. Chon case → 3. Hien thi chi tiet |
| Alternative Flow | Case bi xoa → Redirect to list |

### 4.3 UC-003: Review OCR

| Field | Value |
|-------|-------|
| Use Case ID | UC-003 |
| Name | Review OCR Result |
| Actor | KSV, Editor |
| Preconditions | Da dang nhap, co item trong queue |
| Postconditions | Item da duoc review |
| Basic Flow | 1. Vao queue → 2. Chon item → 3. Review → 4. Duyet/Sua/Reject → 5. Next item |
| Alternative Flow | Skip → Next item |

### 4.4 UC-004: Create Citation

| Field | Value |
|-------|-------|
| Use Case ID | UC-004 |
| Name | Create Citation |
| Actor | KSV |
| Preconditions | Da dang nhap, tai lieu dang mo |
| Postconditions | Citation da duoc tao |
| Basic Flow | 1. Select text → 2. Create Citation → 3. Fill form → 4. Save |
| Alternative Flow | No text selected → Disable button |

### 4.5 UC-005: Search Documents

| Field | Value |
|-------|-------|
| Use Case ID | UC-005 |
| Name | Search Documents |
| Actor | All users |
| Preconditions | Da dang nhap |
| Postconditions | Hien thi ket qua |
| Basic Flow | 1. Nhap query → 2. Search → 3. Display results |
| Alternative Flow | No results → Show empty state |

### 4.6 UC-006: Export Report

| Field | Value |
|-------|-------|
| Use Case ID | UC-006 |
| Name | Export Case Report |
| Actor | KSV |
| Preconditions | Da dang nhap, case ton tai |
| Postconditions | Export file |
| Basic Flow | 1. Chon Export → 2. Chon format → 3. Generate → 4. Download |
| Alternative Flow | Empty case → Disable export |

---

## 5. Integration Points

### 5.1 External Integrations

| System | Integration | Type |
|--------|-------------|------|
| File System | Import/Export | File read/write |
| PDF Libraries | OCR | API call |
| Ollama | AI Assistant | HTTP API |
| Tauri | Desktop APIs | Rust IPC |

### 5.2 Internal Integration

| Component | Interface |
|-----------|----------|
| UI → Application | React Hooks |
| Application → Domain | Use Cases |
| Domain → Data | Repositories |
| Application → OCR | Service |
| Application → AI | Adapter |

---

## 6. Error Handling

### 6.1 Error Types

| Error | User Message | Action |
|-------|-------------|--------|
| File not found | "Khong tim thay file" | Return to list |
| OCR failed | "Khong doc duoc file" | Retry option |
| Network offline | "Khong co mang" | Queue operations |
| Permission denied | "Khong co quyen" | Show message |
| Session expired | "Het phiên lam viec" | Redirect login |

### 6.2 Recovery Flows

```
Error Occurred
       │
       ▼
┌─────────────────────┐
│ Log Error          │
│ + Show Message    │
├──────────────────┤
│ User Action?    │
├──────────────────┤
│ Retry → Retry  │───► Try again
│ Cancel → Exit│───► Return
│ Contact → Help│───► Show contact
└─────────────────────┘
```

---

## 7. Acceptance Criteria

- [ ] Import case flow hoan chinh
- [ ] OCR processing flow day du
- [ ] Search flow hoat dong
- [ ] All 6 use cases dinh nghia
- [ ] Integration points ro rang
- [ ] Error handling ro rang

---

## 8. Khong Duoc Hieu Sai

- **Flow KHONG phai la sequence diagram** - chi la user journey
- **Use case KHONG phai la user story** - chi tiet hon, co action
- **Integration KHONG phai la API spec** - chi la connection points

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom D - Pipeline*
*Tiep theo: File 9 - Dac ta luong scan OCR AI*