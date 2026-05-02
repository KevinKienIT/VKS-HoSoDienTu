# ĐẶC TẢ MÀN HÌNH QUẢN LÝ TRANG VÀ VIEWER

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa màn hình quản lý trang và viewer để xem tài liệu PDF/image với các tính năng như zoom, pan, citation.
Nguon prompt: Tu yeu cau ve page management va document viewer.

---

## 1. Tong Quan Page Management & Viewer

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal viewer spec**
- **KHÔNG có page navigation**
- **KHÔNG có annotation support**

### 1.2 Muc Tieu Cua File Nay

- Định nghĩa viewer requirements
- Page navigation features
- Zoom/pan controls
- Citation anchor integration

---

## 2. Viewer Screen Layout

### 2.1 Full Layout

```
┌──────────────────────────────────────────────────────────────────────┐
│ [←Back] [Case Name] / [Document Name]         [Zoom] [Fit] [Grid] │
├──────────────────────────────────────────────┬─────────────────────┤
│                                              │                    │
│                                              │  CITATION PANEL    │
│                                              │  ────────────────  │
│           DOCUMENT VIEWER                    │                    │
│           ─────────────────                   │  + Add Citation    │
│                                              │                    │
│    ┌────────────────────────────────┐       │  [Citation 1]     │
│    │                                │       │  "Tr. 1 - To pham.."│
│    │      [PDF/Image Content]       │       │  [Link to Entity]  │
│    │                                │       │                    │
│    │                                │       │  [Citation 2]     │
│    │                                │       │  "Tr. 2 - Khai.."  │
│    │                                │       │                    │
│    └────────────────────────────────┘       │  ────────────────  │
│                                              │                    │
│                                              │  [OCR Text]        │
│                                              │  ────────────────  │
├──────────────────────────────────────────────┤  "Van ban OCR..."   │
│ [Trang 1/15]  [< Prev]  [Trang]  [Next >]    │                    │
└──────────────────────────────────────────────┴─────────────────────┘
```

### 2.2 Modes

| Mode | Layout | Usage |
|------|--------|-------|
| **Full** | Viewer + Citation | Normal view |
| **Split** | Left: Viewer, Right: OCR text | OCR review |
| **Grid** | Multi-page thumbnail | Page navigation |
| **Focus** | Full-screen viewer | Read mode |

---

## 3. Viewer Components

### 3.1 Toolbar

| Component | Type | Description |
|-----------|------|-------------|
| Back Button | Button | Return to case |
| Document Title | Text | Current document name |
| Zoom In/Out | Button | Zoom controls |
| Fit Width/Page | Button | Fit options |
| Grid Toggle | Button | Grid mode |
| Export | Button | Export current page |

### 3.2 Page Navigation

| Component | Type | Description |
|-----------|------|-------------|
| Page Input | Number Input | Jump to page |
| Prev/Next | Button | Navigate pages |
| Page Slider | Slider | Scroll through pages |
| Thumbnails | Grid | Page overview |

### 3.3 Viewer Canvas

| Component | Type | Description |
|-----------|------|-------------|
| Image Container | Div | Holds PDF/image |
| OCR Overlay | Div | Text overlay |
| Annotation Layer | Canvas | Annotations |
| Selection Layer | Div | Text selection |

### 3.4 Citation Panel

| Component | Type | Description |
|-----------|------|-------------|
| Add Button | Button | Create citation |
| Citation List | List | Existing citations |
| Citation Item | Card | Individual citation |
| OCR Preview | Text | Current OCR text |

---

## 4. Page Navigation Features

### 4.1 Navigation Controls

```
┌────────────────────────────────────────┐
│ [First] [<]  [Page 5 / 15]  [>] [Last] │
├────────────────────────────────────────┤
│  [━━━━●────────────────────────]       │
│   Page 1       Slider       Page 15   │
└────────────────────────────────────────┘
```

| Control | Action | Shortcut |
|---------|--------|----------|
| First | Go to page 1 | `Ctrl+Home` |
| Prev | Previous page | `←` or `PageUp` |
| Next | Next page | `→` or `PageDown` |
| Last | Go to last page | `Ctrl+End` |
| Jump | Enter page number | `Ctrl+G` |

### 4.2 Page Thumbnails

```
┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐
│ [1] │ │ [2] │ │ [3] │ │ [4] │ │ [5] │
│     │ │  ✓  │ │  ⚠  │ │     │ │     │
└─────┘ └─────┘ └─────┘ └─────┘ └─────┘
  ✓ = reviewed, ⚠ = needs review
```

### 4.3 Page Indicators

| Indicator | Meaning |
|-----------|---------|
| ✓ Green | Reviewed, OK |
| ⚠ Yellow | Needs review |
| ✗ Red | Issue |
| ○ Empty | Not processed |

---

## 5. Zoom & Pan Controls

### 5.1 Zoom Options

| Mode | Zoom Level | Description |
|------|------------|-------------|
| Fit Width | Auto | Fit to width |
| Fit Page | Auto | Fit to page |
| Zoom In | 25%-400% | Manual |
| Custom | User-defined | User zoom |

### 5.2 Zoom Presets

| Preset | Zoom | Shortcut |
|--------|------|----------|
| Fit Width | Variable | `Ctrl+0` |
| Fit Page | Variable | `Ctrl+Shift+0` |
| 50% | 0.5 | - |
| 75% | 0.75 | - |
| 100% | 1.0 | `Ctrl+1` |
| 150% | 1.5 | - |
| 200% | 2.0 | `Ctrl+2` |

### 5.3 Pan Controls

| Action | Mouse | Keyboard |
|--------|-------|----------|
| Pan | Drag | Arrow keys |
| Reset | Double-click | `Escape` |

---

## 6. Text Selection & OCR

### 6.1 Text Selection

```
Selected text: "To pham theo Dieu 108"
                   └─[Create Citation]→
```

| Feature | Description |
|---------|-------------|
| Selection | Click and drag to select |
| Context Menu | Right-click for options |
| Copy | Copy selected text |
| Citation | Create from selection |

### 6.2 OCR Integration

| State | Display |
|-------|---------|
| **direct** | Text hiển thị trực tiếp |
| **candidate_only** | Text highlighted |
| **interpolated_pending_review** | Text with ⚠ badge |
| **approved_manual** | Text verified ✓ |

### 6.3 Uncertain Span Highlighting

```html
<!-- Example: uncertain span shown with background -->
<p>
  To <mark class="uncertain">pham theo</mark> Dieu 108
  <span class="tooltip">confidence: 0.65</span>
</p>
```

| Class | Background | Border |
|-------|------------|---------|
| uncertain | #FFF3CD | #FFC107 |
| candidate | #D1ECF1 | #17A2B8 |
| rejected | #F8D7DA | #DC3545 |

---

## 7. Citation Creation

### 7.1 Create Flow

```
[Select text] → [Right-click] → [Create Citation]
       │
       ▼
┌─────────────────────┐
│ Citation Form       │
├─────────────────────┤
│ Quote: [selected]   │
│ Type: [dropdown]   │
│ Entity: [search]   │
│ Note: [textarea]   │
├─────────────────────┤
│ [Cancel] [Save]   │
└─────────────────────┘
```

### 7.2 Citation Form Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| quote_excerpt | Text | ✅ | Selected text |
| citation_type | Select | ✅ | Type (admission, evidence, question) |
| entity_links | Multi-select | | Link to entities |
| context_before | Text | | Background |
| context_after | Text | | Following text |
| note | Textarea | | User notes |

### 7.3 Citation Actions

| Action | Result |
|--------|--------|
| Click citation | Scroll to position |
| Edit | Open edit form |
| Delete | Confirm + remove |
| Export | Export to report |

---

## 8. Keyboard Shortcuts

### 8.1 Navigation

| Shortcut | Action |
|----------|--------|
| `←` / `→` | Previous/Next page |
| `↑` / `↓` | Scroll vertical |
| `Home` / `End` | First/Last page |
| `Ctrl+G` | Go to page |

### 8.2 Viewing

| Shortcut | Action |
|----------|--------|
| `+` / `-` | Zoom in/out |
| `Ctrl+0` | Fit width |
| `Ctrl+Shift+0` | Fit page |
| `F` | Toggle fullscreen |
| `G` | Toggle grid |

### 8.3 Tools

| Shortcut | Action |
|----------|--------|
| `C` | Create citation |
| `Ctrl+C` | Copy text |
| `Ctrl+F` | Find in document |
| `Escape` | Close modal |

---

## 9. Data Flow

### 9.1 Document Load

```
User opens document
         │
         ▼
┌─────────────────────┐
│ Check cache        │
│ (image + OCR)      │
├─────────────────────┤
│ Cache HIT → Load   │───► Display
│ Cache MISS → OCR  │───► Display + Cache
└─────────────────────┘
```

### 9.2 Page Navigation

```
User clicks Next
         │
         ▼
┌─────────────────────┐
│ Update page_index  │
│ in viewerStore    │
├─────────────────────┤
│ Fetch page data   │
│ from cache/DB    │
├─────────────────────┤
│ Update viewer    │
│ + OCR overlay    │
└─────────────────────┘
```

### 9.3 State Management

```typescript
interface ViewerState {
  documentId: string | null;
  currentPage: number;
  totalPages: number;
  zoom: number;
  rotation: number;
  fitMode: 'width' | 'page' | 'custom';
  mode: 'full' | 'split' | 'grid' | 'focus';
  sidebarOpen: boolean;
  citations: Citation[];
  ocrOverlay: boolean;
}
```

---

## 10. Acceptance Criteria

- [ ] Viewer load PDF/image without delay
- [ ] Page navigation work correctly
- [ ] Zoom/pan smooth and intuitive
- [ ] Grid mode show thumbnails
- [ ] OCR text overlay toggleable
- [ ] Citation creation from selection
- [ ] Uncertain spans highlighted
- [ ] Keyboard shortcuts work

---

## 11. Khong Duoc Hieu Sai

- **Viewer KHÔNG phải là image editor** - chi display, không edit ảnh
- **OCR overlay KHÔNG phải là editor** - chi display, edit riêng
- **Citation KHÔNG phải là note** - citation có structured schema

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom C - UI & Man hinh*
*Tiep theo: File 5 - Dac ta man hinh quan ly ho so*