# DAC TA LAYERED ARCHITECTURE

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dinh nghia cau truc layered architecture cho phan mem desktop VKS ECMS, cac layer, data flow, va service boundaries.
Nguon prompt: Tu yeu cau nguoi dung ve phan mem chay offline, nhieu lop, JSON thong minh, hieu ung chuyen canh muot.

---

## 1. Tong Quan Layered Architecture

Cau truc 4 layer hoan chinh:

```
┌─────────────────────────────────────────────────┐
│           LAYER 4: UI PRESENTATION          │
│   (React Components, Animations, Views)         │
├─────────────────────────────────────────────────┤
│           LAYER 3: APPLICATION                   │
│   (State Management, Services, Hooks)          │
├─────────────────────────────────────────────────┤
│           LAYER 2: DOMAIN                        │
│   (Business Logic, Entities, Use Cases)       │
├─────────────────────────────────────────────────┤
│           LAYER 1: DATA                         │
│   (SQLite, File System, JSON Storage)         │
└─────────────────────────────────────────────────┘
```

Nguyen tac:
- Moi layer chi biet den layer ngay ben duoi
- Khong co circular dependencies
- Data flow tu tren xuong, co the broadcast events

---

## 2. Layer 1: Data Layer

### 2.1 SQLite Database

Cac bang co ban:

| Table | Muc dich | Primary Key |
|------|---------|------------|
| `cases` | Luu tru thong tin case | `case_id` |
| `documents` | Luu tru metadata tai lieu | `document_id` |
| `pages` | Luu tru page-level data | `page_id` |
| `entities` | Luu tru doi tuong, vat chung | `entity_id` |
| `citations` | Luu tru trich dan | `citation_id` |
| `users` | Luu tru cau hinh nguoi dung | `user_id` |

### 2.2 File Storage

| Loai | Thu muc | Muc dich |
|------|-------|----------|
| PDF goc | `/storage/originals/` | Luu file scan goc |
| Anh page | `/storage/pages/` | Anh sau tach page |
| Cache | `/storage/cache/` | Cache OCR, thumbnails |
| Export | `/storage/exports/` | File xuat bao cao |

### 2.3 JSON Storage

| File | Vi tri | Noi dung |
|------|-------|----------|
| `config.json` | Root | Cau hinh ung dung |
| `preferences.json` | `/storage/` | Cau hinh nguoi dung |
| `search-index.json` | `/storage/cache/` | Index tim kiem |
| `citation-graph.json` | `/storage/cache/` | Do thi trich dan |

---

## 3. Layer 2: Domain Layer

### 3.1 Entities

```typescript
interface Case {
  case_id: string;
  case_code: string;
  case_display_name: string;
  source_folder_name: string;
  primary_person_name: string;
  status: 'active' | 'archived';
  created_at: string;
  updated_at: string;
}

interface Document {
  document_id: string;
  case_id: string;
  original_filename: string;
  file_path: string;
  display_name: string;
  document_type: string;
  issued_date: string | null;
  summary_short: string;
  ocr_confidence_avg: number;
  needs_review: boolean;
}

interface Page {
  page_id: string;
  document_id: string;
  page_index: number;
  ocr_text: string;
  but_luc: string | null;
  confidence: number;
}

interface Entity {
  entity_id: string;
  entity_type: 'person' | 'device' | 'evidence' | 'event';
  name: string;
  description: string;
  linked_documents: string[];
}

interface Citation {
  citation_id: string;
  source_document_id: string;
  source_page_id: string;
  quote_excerpt: string;
  confidence: number;
  entity_links: string[];
}
```

### 3.2 Use Cases

| Use Case | Layer | Muc dich |
|---------|-------|----------|
| `CreateCase` | Domain | Tao case moi tu folder scan |
| `ImportDocument` | Domain | Nhap tai lieu vao he thong |
| `RunOCR` | Domain | Chay OCR tren page |
| `ClassifyDocument` | Domain | Phan loai tai lieu |
| `ExtractMetadata` | Domain | Trich metadata |
| `SearchDocuments` | Domain | Tim kiem tai lieu |
| `GetDossier` | Domain | Lay ho so doi tuong |
| `AskAI` | Domain | Hoi AI offline |

---

## 4. Layer 3: Application Layer

### 4.1 Services

| Service | Thuoc Layer | Muc dich |
|---------|-------------|----------|
| `CaseService` | Application | Quan ly case CRUD |
| `DocumentService` | Application | Quan ly tai lieu |
| `OCRService` | Application | Dong goi OCR |
| `SearchService` | Application | Tim kiem, sap xep |
| `CitationService` | Application | Quan ly trich dan |
| `AIService` | Application | Tich hop Ollama |
| `ExportService` | Application | Xuat bao cao |
| `ConfigService` | Application | Cau hinh he thong |
| `UserPreferenceService` | Application | Luu cau hinh nguoi dung |

### 4.2 State Management

```typescript
// Global App State (Zustand)
interface AppState {
  // Current session
  currentCaseId: string | null;
  currentDocumentId: string | null;
  currentPageIndex: number | null;
  
  // View state
  sidebarOpen: boolean;
  viewerMode: 'pdf' | 'image' | 'split';
  searchQuery: string;
  filters: FilterState;
  
  // User preferences
  fontSize: number;
  theme: 'light' | 'dark';
  language: 'vi' | 'en';
  
  // Actions
  setCurrentCase: (caseId: string) => void;
  setCurrentDocument: (docId: string, pageIndex?: number) => void;
  toggleSidebar: () => void;
  setSearchQuery: (query: string) => void;
  updateFilters: (filters: Partial<FilterState>) => void;
}
```

### 4.3 Hooks

| Hook | Muc dich |
|------|--------|
| `useCase` | Lay va quan ly case hien tai |
| `useDocument` | Lay va quan ly tai lieu |
| `useSearch` | Tim kiem voi real-time |
| `useViewer` | Dieu khien viewer |
| `useAI` | Tich hop AI assistant |
| `usePreferences` | Doc/ghi preferences |
| `useCitation` | Tao va lay trich dan |

---

## 5. Layer 4: UI Layer

### 4.1 Components

| Component | Thuoc Layer | Muc dich |
|----------|-------------|----------|
| `AppLayout` | Presentation | Layout chinh |
| `Sidebar` | Presentation | Thanh ben |
| `CaseList` | Presentation | Danh sach case |
| `DocumentList` | Presentation | Danh sach tai lieu |
| `DocumentViewer` | Presentation | Xem PDF/anh |
| `SearchPanel` | Presentation | Tim kiem |
| `AIPanel` | Presentation | Tro ly AI |
| `DossierPanel` | Presentation | Ho so doi tuong |
| `ReviewQueue` | Presentation | Hang cho review |

### 4.2 Animations

| Animation | Component | Thu Vien |
|-----------|-----------|----------|
| Page transition | Views | Framer Motion |
| List stagger | CaseList, DocumentList | Framer Motion |
| Sidebar slide | Sidebar | Framer Motion |
| Modal fade | Dialogs | React Transition Group |
| Viewer zoom | DocumentViewer | CSS Transform |
| Skeleton load | All lists | Framer Motion |

---

## 6. Data Flow

### 6.1 Read Flow

```
User click
  -> UI Layer (Component)
  -> Application Layer (Hook/Service)
  -> Domain Layer (Use Case)
  -> Data Layer (SQLite/File)
  -> Return reverse path
```

### 6.2 Write Flow

```
User action
  -> UI Layer (Component)
  -> Application Layer (Service.validate)
  -> Domain Layer (Use Case.execute)
  -> Data Layer (SQLite.insert/update)
  -> Broadcast event
  -> UI Layer (State update)
```

### 6.3 Event Flow

```
Data changes
  -> Domain Layer emit event
  -> Application Layer handle event
  -> UI Layer subscribe to state
  -> Re-render
```

---

## 7. Service Boundaries

| From | To | Allowed |
|------|-----|---------|
| UI | Application | Direct |
| UI | Domain | Cam (use use cases) |
| Application | Domain | Direct |
| Application | Data | Direct |
| Domain | Data | Direct |
| Data | any | Cam |

---

## 8. Error Handling

| Layer | Xu ly |
|-------|-------|
| Data | Return null, throw DBError |
| Domain | Wrap in Result<T, Error> |
| Application | Log + notify UI |
| UI | Show error toast |

---

## 9. Acceptance Criteria

- [ ] 4 layer ro rang, khong circular dependencies
- [ ] Moi service trong Layer 3 chi goi Layer 2
- [ ] State management dung Zustand, reactive
- [ ] UI chi giao tiep qua Application Layer
- [ ] Co separation giua business logic va I/O
- [ ] Animation dung Framer Motion

---

**STATUS: SPECIFICATION DEFINED. WAITING FOR IMPLEMENTATION.**