# ĐẶC TẢ MÀN HÌNH ĐỌC TEXT VÀ HIỂN THỊ

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa màn hình đọc text, hiển thị OCR, citation và review queue để xem và xử lý văn bản trích xuất.
Nguon prompt: Tu yeu cau ve text reading va review queue.

---

## 1. Tong Quan Text Reading & Display

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal OCR display screen**
- **KHÔNG có review queue screen**
- **KHÔNG có citation management screen**

### 1.2 Muc Tieu Cua File Nay

- OCR text display screen
- Review queue screen
- Citation management
- Reading tools

---

## 2. OCR Text Display Screen

### 2.1 Full Layout

```
┌──────────────────────────────────────────────────────────────────────┐
│ [Tai lieu: To khai ban 1]                    [Edit] [Approve] [Reject] │
├──────────────────────────────────────────────┬───────────────────────┤
│                                              │                       │
│  VIEWER (PDF Image)                         │   OCR TEXT           │
│  ─────────────────                         │   ──────────────     │
│  ┌────────────────────────────────┐       │                       │
│  │                                │       │   To pham theo       │
│  │      [PDF Content]              │       │   Dieu 108 Binh      │
│  │                                │       │   Huong, To va      │
│  └────────────────────────────────┘       │   Dang su that ngay   │
│                                              │   20/01/2026     │
│                                              │   tai...           │
│                                              │                       │
├──────────────────────────────────────────────┼───────────────────────┤
│                                              │   CONFIDENCE       │
│                                              │   ──────────       │
│   PAGE 1/3  [<] [1] [2] [3] [>]      │   Avg: 0.85        │
│                                              │   ⚠ [5 uncertain]  │
└──────────────────────────────────────────────┴───────────────────────┘
```

### 2.2 Reading Modes

| Mode | Layout | Usage |
|------|--------|-------|
| **Side-by-side** | Left: PDF, Right: OCR | Review OCR |
| **PDF Only** | Full PDF view | Read document |
| **OCR Only** | Full text view | Read OCR |
| **Split** | Top: PDF, Bottom: OCR | Compare |

---

## 3. Text Display Features

### 3.1 Display Options

```
┌─────────────────────────────────┐
│ [Font A-] [A+] [Theme ▼] [Wrap]  │
└─────────────────────────────────┘
```

| Option | Values | Description |
|--------|--------|-------------|
| Font Size | 12-24px | Text size |
| Theme | Light/Dark/Sepia | Reading theme |
| Line Wrap | On/Off | Word wrap |
| Font | Sans/Serif/Mono | Font family |

### 3.2 Text Highlighting

```
Normal text:    To pham theo Dieu 108
Highlighted:   [To pham theo] Dieu 108  ← citation
Uncertain:     To [pham theo] Dang 108   ← uncertain span
```

| Type | Style | Description |
|------|-------|-------------|
| Normal | Default | Text binh thuong |
| Citation | Blue background | Text trich dan |
| Uncertain | Yellow background | Span chua xac nhan |
| Rejected | Red background | Text bi tu choi |

### 3.3 Confidence Indicators

| Level | Color | Badge | Action |
|-------|-------|-------|--------|
| High (≥0.9) | Green | ✓ | Auto display |
| Medium (0.7-0.9) | Yellow | ⚠ | Review suggested |
| Low (0.5-0.7) | Orange | ⚠ | Review required |
| Very Low (<0.5) | Red | ✗ | Manual review |

---

## 4. Review Queue Screen

### 4.1 Queue Layout

```
┌──────────────────────────────────────────────────────────────────────┐
│ Review Queue                              [Filter ▾] [Sort ▾]          │
├──────────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │ 📄 To khai - Tr. 2                                      │    │
│  │ Confidence: 0.65  ⚠ Low                               │    │
│  │ Issue: handwritten_uncertain                              │    │
│  │ ──────────────────────────────────────────────────────  │    │
│  │ "To [pham] [Dang] that ngay tai..."                    │    │
│  │        ↑ uncertain region                             │    │
│  ├──────────────────────────────────────────────────────┤    │
│  │ [Skip] [Edit] [Approve] [Reject]                    │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │ 📄 Bien ban - Tr. 5                                      │    │
│  │ Confidence: 0.72  ⚠ Medium                             │    │
│  │ Issue: unclear_text                                     │    │
│  │ ──────────────────────────────────────────────────────  │    │
│  │ "Nguoi bi can [khong thu nhan]..."                   │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                 │
├──────────────────────────────────────────────────────────────────────��
│ Items 1-10 of 45     [< Prev] [1] [2] [3] [4] [Next >]             │
└──────────────────────────────────────────────────────────────────────┘
```

### 4.2 Queue Filters

| Filter | Options |
|--------|----------|
| Issue Type | All, low_confidence, unclear_text, handwritten_uncertain, seal_not_detected |
| Confidence | All, <0.5, 0.5-0.7, 0.7-0.9 |
| Document Type | All, to_khai, bien_ban, quyet_dinh |
| Date | Today, Last 7 days, Custom |

### 4.3 Queue Item Actions

| Action | Result |
|--------|--------|
| Skip | Move to next item |
| Edit | Open editor |
| Approve | Mark as reviewed |
| Reject | Mark for manual |

---

## 5. Review Workflow

### 5.1 Review Process

```
[Queue Item]
    │
    ▼
┌─────────────────────┐
│ View Side-by-Side    │
│ (Image + OCR)      │
└───────┬─────────────┘
        │
        ▼
┌─────────────────────┐
│ Compare OCR        │
│ to original       │
└───────┬─────────────┘
        │
        ├──────┬──────┬──────┐
        ▼     ▼     ▼     ▼
    ┌─────┐ ┌─────┐ ┌─────┐ ┌────┐
    │Edit│ │Skip│ │ App │ │Rej │
    └─────┘ └─────┘ └─────┘ └────┘
```

### 5.2 Editor Interface

```
┌─────────────────────────────────────────────────────┐
│ Edit OCR Revision                    [Cancel] [Save] │
├─────────────────────────────────────────────────────┤
│                                             │
│ Original OCR:                                │
│ "To [pham] [Dang] thanh 500..."              │
│                                             │
│ ──────────────────────────────────────────── │
│                                             │
│ Proposed Edit:                               │
│ To pham thanh 500 trieu dong                 │
│                                             │
│ ──────────────────────────────────────────── │
│                                             │
│ Candidates:                                  │
│ 1. To pham thanh 500 (0.68)                │
│ 2. To pham Dang 500 (0.52)                  │
│ 3. To pham thanh 500.000 (0.45)              │
│                                             │
│ ──────────────────────────────────────────── │
│ Reason: [dropdown ▼]                         │
│ [typing_error] [missing_char] [wrong_word]    │
│                                             │
│ Note: [______________________________________] │
└─────────────────────────────────────────────────────┘
```

### 5.3 Review Actions

| Action | State Change | Audit |
|--------|-------------|-------|
| Approve | pending → reviewed | auto_approved |
| Reject | pending → rejected | manual_rejected |
| Edit | pending → in_review | manual_edited |
| Skip | - | skipped |

---

## 6. Citation Management

### 6.1 Citation List

```
┌─────────────────────────────────┐
│ Citations (23)     [+ Add] [Export]│
├─────────────────────────────────┤
│                                 │
│ ┌─────────────────────────────┐  │
│ │ "To pham theo..."  Tr.1     │  │
│ │ Type: admission  Confidence: │  │
│ │ 0.92                      │  │
│ │ Links: Nguyen Van A       │  │
│ └─────────────────────────────┘  │
│                                 │
│ ┌─────────────────────────────┐  │
│ │ "Khai bay..."  Tr.3        │  │
│ │ Type: evidence  Confidence:│  │
│ │ 0.88                      │  │
│ │ Links: Device #1           │  │
│ └─────────────────────────────┘  │
└─────────────────────────────────┘
```

### 6.2 Citation Types

| Type | Color | Description |
|------|-------|-------------|
| admission | Blue | Lời khai nhận tội |
| evidence | Green | Bằng chứng |
| question | Orange | Câu hỏi |
| statement | Purple | Lời khai |
| timeline | Yellow | Sự kiện |

### 6.3 Citation Search

```
┌─────────────────────────────────────────────┐
│ [Search citations...]      [Type ▾] [Entity ▾]  │
├─────────────────────────────────────────────┤
│ All (23) │ admission (8) │ evidence (10)    │
│         │ question (3) │ statement (2)    │
└─────────────────────────────────────────────┘
```

---

## 7. Text Selection Tools

### 7.1 Selection Actions

```
[Select text]
    │
    ▼
┌─────────────────────────────────────────┐
│ Create Citation  │  Add Note    │
│ Add Bookmark  │  Translate │
└─────────────────────────────────────────┘
```

### 7.2 Create Citation Flow

```
1. Select text in OCR view
2. Right-click → Create Citation
3. Fill form:
   - Quote (pre-filled)
   - Type (dropdown)
   - Entity links (search)
   - Note (optional)
4. Save → Citation created
```

### 7.3 Entity Linking

```
Search entities:
┌─────────────────────────────────────┐
│ [🔍 Tim doi tuong...]                │
├─────────────────────────────────────┤
│ ► Nguyen Van A  (bi_can) - CMND      │
│ ► Nguyen Van B  (nhan_chung)        │
│ ► #Device 1   (device)             │
└─────────────────────────────────────┘
```

---

## 8. State Management

### 8.1 Review Store

```typescript
interface ReviewState {
  queue: ReviewItem[];
  currentItem: ReviewItem | null;
  loading: boolean;
  
  // Actions
  fetchQueue: () => Promise<void>;
  approve: (itemId: string) => Promise<void>;
  reject: (itemId: string, reason: string) => Promise<void>;
  edit: (itemId: string, newText: string) => Promise<void>;
  skip: (itemId: string) => Promise<void>;
}
```

### 8.2 Citation Store

```typescript
interface CitationState {
  citations: Citation[];
  currentCitation: Citation | null;
  loading: boolean;
  
  // Actions
  fetchByCase: (caseId: string) => Promise<void>;
  create: (data: CreateCitation) => Promise<Citation>;
  update: (id: string, data: UpdateCitation) => Promise<void>;
  delete: (id: string) => Promise<void>;
}
```

---

## 9. Data Flow

### 9.1 Review Flow

```
OCR Processing Complete
         │
         ▼
┌─────────────────────┐
│ Confidence Check   │
│ (avg < 0.7?)     │
├──────────────────┤
│ Yes → Queue      │───► Review Queue
│ No → Indexed   │───► Done
└─────────────────────┘
```

### 9.2 Review Complete

```
Reviewer Action
         │
         ▼
┌─────────────────────┐
│ Update State       │
│ + Audit Trail    │
├──────────────────┤
│ Update page    │
│ content/ocr   │
│ state        │
├──────────────────┤
│ Re-index     │
│ if needed   │
└─────────────────────┘
```

---

## 10. Acceptance Criteria

- [ ] OCR text display correctly
- [ ] Side-by-side view work
- [ ] Review queue display items
- [ ] Review workflow complete
- [ ] Citation management
- [ ] Entity linking
- [ ] Confidence indicators
- [ ] Uncertain highlighting

---

## 11. Khong Duoc Hieu Sai

- **Review KHONG phải là edit document** - chi review OCR text
- **Citation KHONG phải là note** - citation co structured schema
- **OCR display KHONG phải là OCR engine** - chi display, engine rieng

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom C - UI & Man hinh*
*Tiep theo: File 7 - Dac ta tai khoan nguoi dung*