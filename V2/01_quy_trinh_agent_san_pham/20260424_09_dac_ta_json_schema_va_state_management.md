# DAC TA JSON SCHEMA VA STATE MANAGEMENT

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dinh nghia JSON schemas, state management patterns, va data flow cho ung dung VKS ECMS.
Nguon prompt: Tu yeu cau nguoi dung ve xu ly JSON thong minh, luu trang thai.

---

## 1. JSON Schema Definitions

### 1.1 Root Config Schema

```json
// config.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["app", "database", "ai", "ui"],
  "properties": {
    "app": {
      "type": "object",
      "required": ["name", "version", "mode"],
      "properties": {
        "name": { "type": "string", "const": "VKS ECMS" },
        "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
        "mode": { "type": "string", "enum": ["development", "production"] },
        "singleInstance": { "type": "boolean", "default": true }
      }
    },
    "database": {
      "type": "object",
      "properties": {
        "path": { "type": "string" },
        "filename": { "type": "string" }
      }
    },
    "ai": {
      "type": "object",
      "properties": {
        "provider": { "type": "string", "enum": ["ollama", "openclaw"] },
        "model": { "type": "string" },
        "endpoint": { "type": "string" }
      }
    },
    "ui": {
      "type": "object",
      "properties": {
        "theme": { "type": "string", "enum": ["light", "dark", "system"] },
        "fontSize": { "type": "number", "minimum": 12, "maximum": 24 },
        "animation": { "type": "boolean" }
      }
    }
  }
}
```

### 1.2 User Preferences Schema

```json
// preferences.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "user_id": { "type": "string", "format": "uuid" },
    "display": {
      "type": "object",
      "properties": {
        "fontSize": { "type": "number", "default": 14 },
        "theme": { "type": "string", "default": "light" },
        "language": { "type": "string", "default": "vi" },
        "sidebarWidth": { "type": "number", "default": 280 }
      }
    },
    "viewer": {
      "type": "object",
      "properties": {
        "defaultZoom": { "type": "string", "default": "fit-width" },
        "autoRotate": { "type": "boolean", "default": true },
        "showPageNumbers": { "type": "boolean", "default": true }
      }
    },
    "search": {
      "type": "object",
      "properties": {
        "defaultFilters": {
          "type": "object",
          "properties": {
            "documentType": { "type": "array", "items": { "type": "string" } },
            "dateRange": { "type": "object" }
          }
        },
        "recentQueries": {
          "type": "array",
          "items": { "type": "string" },
          "maxItems": 20
        }
      }
    },
    "recentCases": {
      "type": "array",
      "items": { "type": "string" },
      "maxItems": 10
    },
    "lastOpenedCase": { "type": "string" },
    "lastOpenedDocument": { "type": "string" },
    "updated_at": { "type": "string", "format": "date-time" }
  }
}
```

### 1.3 Case Schema

```json
// Case entity
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["case_id", "case_code", "case_display_name", "status"],
  "properties": {
    "case_id": { "type": "string", "format": "uuid" },
    "case_code": { "type": "string" },
    "case_display_name": { "type": "string" },
    "source_folder_name": { "type": "string" },
    "case_sequence_no": { "type": "number" },
    "primary_person_name": { "type": "string" },
    "case_group_label": { "type": "string" },
    "status": { "type": "string", "enum": ["active", "archived"] },
    "document_count": { "type": "number" },
    "created_at": { "type": "string", "format": "date-time" },
    "updated_at": { "type": "string", "format": "date-time" }
  }
}
```

### 1.4 Document Schema

```json
// Document entity
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["document_id", "case_id", "original_filename", "status"],
  "properties": {
    "document_id": { "type": "string", "format": "uuid" },
    "case_id": { "type": "string", "format": "uuid" },
    "original_filename": { "type": "string" },
    "import_sequence": { "type": "number" },
    "file_path": { "type": "string" },
    "file_hash": { "type": "string" },
    "file_size": { "type": "number" },
    "page_count": { "type": "number" },
    "display_name": { "type": "string" },
    "document_title": { "type": "string" },
    "document_type": { "type": "string" },
    "issued_date": { "type": ["string", "null"] },
    "summary_short": { "type": "string" },
    "summary_detail": { "type": "string" },
    "ocr_confidence_avg": { "type": "number" },
    "classification_confidence": { "type": "number" },
    "needs_review": { "type": "boolean" },
    "status": { "type": "string", "enum": ["pending", "processed", "reviewed", "error"] },
    "created_at": { "type": "string", "format": "date-time" },
    "updated_at": { "type": "string", "format": "date-time" }
  }
}
```

### 1.5 Page Schema

```json
// Page entity
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["page_id", "document_id", "page_index"],
  "properties": {
    "page_id": { "type": "string", "format": "uuid" },
    "document_id": { "type": "string", "format": "uuid" },
    "page_index": { "type": "number" },
    "image_path": { "type": "string" },
    "ocr_text": { "type": "string" },
    "ocr_preview": { "type": "string" },
    "but_luc": { "type": ["string", "null"] },
    "confidence": { "type": "number" },
    "regions": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string" },
          "bbox": { "type": "array", "items": { "type": "number" } }
        }
      }
    }
  }
}
```

### 1.6 Entity Schema (Person, Device, Evidence, Event)

```json
// Entity
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["entity_id", "entity_type", "name"],
  "properties": {
    "entity_id": { "type": "string", "format": "uuid" },
    "entity_type": { "type": "string", "enum": ["person", "device", "evidence", "event"] },
    "name": { "type": "string" },
    "description": { "type": "string" },
    "metadata": { "type": "object" },
    "linked_documents": {
      "type": "array",
      "items": { "type": "string", "format": "uuid" }
    },
    "linked_pages": {
      "type": "array",
      "items": { "type": "string", "format": "uuid" }
    },
    "created_at": { "type": "string", "format": "date-time" }
  }
}
```

### 1.7 Citation Schema

```json
// Citation
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["citation_id", "source_document_id", "quote_excerpt"],
  "properties": {
    "citation_id": { "type": "string", "format": "uuid" },
    "source_document_id": { "type": "string", "format": "uuid" },
    "source_page_id": { "type": "string", "format": "uuid" },
    "source_but_luc": { "type": "string" },
    "quote_excerpt": { "type": "string" },
    "confidence": { "type": "number" },
    "entity_links": {
      "type": "array",
      "items": { "type": "string", "format": "uuid" }
    },
    "created_at": { "type": "string", "format": "date-time" }
  }
}
```

---

## 2. State Management Patterns

### 2.1 Zustand Store Structure

```
/src
  /store
    index.ts          # Root store
    caseStore.ts      # Case state
    documentStore.ts  # Document state
    viewerStore.ts    # Viewer state
    searchStore.ts    # Search state
    uiStore.ts        # UI state
    aiStore.ts       # AI state
```

### 2.2 Store Definitions

#### caseStore.ts
```typescript
interface CaseState {
  // State
  cases: Case[];
  currentCase: Case | null;
  loading: boolean;
  error: string | null;
  
  // Actions
  fetchCases: () => Promise<void>;
  getCaseById: (id: string) => Case | undefined;
  createCase: (data: CreateCaseInput) => Promise<Case>;
  updateCase: (id: string, data: UpdateCaseInput) => Promise<Case>;
  deleteCase: (id: string) => Promise<void>;
  setCurrentCase: (caseId: string | null) => void;
}

export const useCaseStore = create<CaseState>((set, get) => ({
  cases: [],
  currentCase: null,
  loading: false,
  error: null,
  
  fetchCases: async () => {
    set({ loading: true, error: null });
    try {
      const cases = await caseService.getAll();
      set({ cases, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },
  
  getCaseById: (id) => get().cases.find(c => c.case_id === id),
  
  createCase: async (data) => {
    const newCase = await caseService.create(data);
    set(s => ({ cases: [...s.cases, newCase] }));
    return newCase;
  },
  
  setCurrentCase: (caseId) => {
    const c = caseId ? get().cases.find(x => x.case_id === caseId) : null;
    set({ currentCase: c });
  }
}));
```

#### viewerStore.ts
```typescript
interface ViewerState {
  // State
  currentDocumentId: string | null;
  currentPageIndex: number;
  zoom: number;
  rotation: number;
  mode: 'fit-width' | 'fit-page' | 'custom';
  sidebarOpen: boolean;
  
  // Actions
  openDocument: (docId: string, pageIndex?: number) => void;
  closeDocument: () => void;
  goToPage: (pageIndex: number) => void;
  nextPage: () => void;
  prevPage: () => void;
  setZoom: (zoom: number) => void;
  setRotation: (rotation: number) => void;
  toggleSidebar: () => void;
}
```

#### searchStore.ts
```typescript
interface SearchState {
  // State
  query: string;
  filters: SearchFilters;
  results: SearchResult[];
  totalResults: number;
  loading: boolean;
  
  // Actions
  setQuery: (query: string) => void;
  setFilters: (filters: SearchFilters) => void;
  search: () => Promise<void>;
  clearResults: () => void;
}

interface SearchFilters {
  documentTypes: string[];
  dateRange: { start: string; end: string } | null;
  confidence: { min: number; max: number };
  hasEntities: boolean;
  needsReview: boolean;
}
```

---

## 3. JSON Transformer Patterns

### 3.1 Import Transformer

```typescript
// JSON -> SQLite
class ImportTransformer {
  async transform(jsonPath: string): Promise<void> {
    const data = await fs.readJson(jsonPath);
    
    // Validate against schema
    validate(data, importSchema);
    
    // Transform and insert
    for (const caseData of data.cases) {
      await caseService.create(transformCase(caseData));
    }
  }
  
  private transformCase(input: any): CreateCaseInput {
    return {
      case_code: input.case_code,
      case_display_name: input.case_display_name,
      source_folder_name: input.source_folder_name,
      primary_person_name: input.primary_person_name
    };
  }
}
```

### 3.2 Export Transformer

```typescript
// SQLite -> JSON
class ExportTransformer {
  async transformCase(caseId: string): Promise<string> {
    const case = await caseService.getById(caseId);
    const documents = await documentService.getByCase(caseId);
    
    const output = {
      case: transformToJson(case),
      documents: documents.map(transformToJson),
      exported_at: new Date().toISOString()
    };
    
    return JSON.stringify(output, null, 2);
  }
}
```

---

## 4. JSON-RPC API Pattern

### 4.1 Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "case.getById",
  "params": { "case_id": "uuid" },
  "id": 1
}
```

### 4.2 Response Format

```json
{
  "jsonrpc": "2.0",
  "result": { ... },
  "id": 1
}
```

### 4.3 Error Response

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": { ... }
  },
  "id": 1
}
```

---

## 5. Persistence Strategy

### 5.1 Auto-save

- Preferences: save after 2 seconds of inactivity (debounced)
- Viewer state: save on document change
- Search history: save after each search
- Last opened: save immediately on navigate

### 5.2 Storage Location

```typescript
const STORAGE = {
  config: path.join(app.getPath('userData'), 'config.json'),
  preferences: path.join(app.getPath('userData'), 'preferences.json'),
  db: path.join(app.getPath('userData'), 'vks-ecms.db'),
  cache: path.join(app.getPath('cache'), 'vks-ecms')
};
```

---

## 6. Acceptance Criteria

- [ ] Tat ca entity co JSON schema ro rang
- [ ] Schema validation khi import/export
- [ ] State management dung Zustand
- [ ] Auto-save preferences trong 2s
- [ ] JSON-RPC cho IPC communication
- [ ] Type-safe stores voi TypeScript

---

**STATUS: SPECIFICATION DEFINED. WAITING FOR IMPLEMENTATION.**