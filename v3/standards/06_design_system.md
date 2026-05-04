# DESIGN.md — Quy chuẩn thiết kế giao diện VKS ECMS

> Source of truth cho mọi agent khi tạo/sửa UI. File này được load bắt buộc qua AGENTS.md.
> Cập nhật: 2026-05-04

---

## 1. Triết lý thiết kế

VKS ECMS là **Document Workspace chuyên nghiệp** cho ngành kiểm sát, KHÔNG phải admin panel web.

### Nguồn cảm hứng (từ 07_external_refs/design-md)

| Nguồn | Áp dụng gì |
|-------|-----------|
| **Linear** | Ultra-minimal, sidebar navigation, keyboard shortcuts, dark sidebar |
| **Notion** | Warm workspace, split-pane, document-centric, serif headings option |
| **Superhuman** | Premium dark UI, keyboard-first, status badges, fast transitions |
| **Raycast** | Desktop-native feel, sleek chrome, vibrant accent, command palette |

### Nguyên tắc bất biến

1. **Desktop-native** — cảm giác như app desktop, không phải web page
2. **Document-centric** — UI phục vụ việc đọc/sửa/trích xuất tài liệu
3. **Split-pane workspace** — sidebar tree + center viewer + right panel
4. **Status-driven** — mọi item phải có status badge rõ ràng
5. **Keyboard-first** — Space=preview, Ctrl+K=command, Tab=navigate
6. **Evidence-first** — mọi hiển thị phải có citation/source link

---

## 2. Design Tokens (Source: `variables.css`)

### 2.1 Color Palette — Deep Indigo Enterprise

```css
/* Primary — Institutional authority */
--color-primary: #2d3a8c;          /* Main brand */
--color-primary-light: #4a5bc7;    /* Hover/active */
--color-primary-dark: #1a2566;     /* Headers */
--color-primary-surface: #eef0fb;  /* Light tint backgrounds */

/* Accent — Gold/amber for attention */
--color-accent: #e69500;           /* Active tabs, highlights */
--color-accent-light: #f5b731;     /* Hover states */

/* Status colors — PHẢI DÙNG ĐÚNG */
--color-success: #10b981;    /* OCR done, managed_ready, pass */
--color-warning: #f59e0b;    /* Review pending, low confidence */
--color-danger: #ef4444;     /* Error, fail, missing file */
--color-info: #3b82f6;       /* Processing, info, link */
```

### 2.2 Typography

```css
--font-sans: 'Inter', 'Segoe UI', system-ui, sans-serif;
--font-mono: 'JetBrains Mono', 'Cascadia Code', monospace;

/* Scale — compact cho desktop app */
--text-xs: 11px;    /* Metadata, timestamps */
--text-sm: 12px;    /* Secondary labels */
--text-base: 13px;  /* Body text — nhỏ hơn web */
--text-md: 14px;    /* Primary labels */
--text-lg: 16px;    /* Section headers */
--text-xl: 18px;    /* Page titles */
```

### 2.3 Spacing — 4px base grid

```css
--space-1: 4px;   --space-2: 8px;   --space-3: 12px;
--space-4: 16px;  --space-6: 24px;  --space-8: 32px;
```

### 2.4 Layout dimensions

```css
--sidebar-width: 232px;           /* Sidebar mở */
--sidebar-width-collapsed: 54px;  /* Sidebar thu */
--header-height: 44px;            /* Top header */
--footer-height: 26px;            /* Status bar */
```

---

## 3. Component Standards

### 3.1 Sidebar (Dossier Spine)

```css
/* Dark gradient background — cảm giác "gáy hồ sơ" */
--spine-bg: linear-gradient(180deg, #1a234e 0%, #16203f 40%, #121a33 100%);
```

- Icon + label cho mỗi mục
- Active item: accent bar bên trái + glow effect
- Hover: translate-X 7px nhẹ
- Số thứ tự nhỏ (`9px`) trước tên mục — giống bút lục
- Settings tách biệt dưới cùng

### 3.2 Cards & Surfaces

```css
--radius-md: 6px;                /* Card corners */
--shadow-sm: 0 1px 3px rgba(0,0,0,0.06);  /* Default elevation */
--shadow-md: 0 2px 6px rgba(0,0,0,0.08);  /* Hover elevation */
```

- Background: `--color-surface` (#fff)
- Border: `--color-border` (#dfe3ec)
- Hover: nâng shadow từ `sm` → `md`
- Transition: `--transition-base` (180ms)

### 3.3 Status Badges

| Trạng thái | Màu | CSS class |
|-----------|------|-----------|
| Chưa OCR | `--color-text-muted` | `.badge-pending` |
| Đang xử lý | `--color-info` | `.badge-processing` |
| OCR xong | `--color-success` | `.badge-ocr-done` |
| Cần review | `--color-warning` | `.badge-review` |
| Đã duyệt | `--color-success` | `.badge-reviewed` |
| Sẵn sàng bàn giao | `--color-primary` | `.badge-managed` |
| Lỗi | `--color-danger` | `.badge-error` |
| File missing | `--color-danger` | `.badge-missing` |

### 3.4 Buttons

- **Primary:** bg=`--color-primary`, text=white, radius=`--radius-md`
- **Secondary:** bg=transparent, border=`--color-border`, text=`--color-text`
- **Danger:** bg=`--color-danger`, text=white
- **Ghost:** bg=transparent, text=`--color-text-secondary`, hover=`--color-bg-hover`
- Min height: 32px, padding: `--space-2` `--space-4`

### 3.5 Tables & Lists

- Header: `--color-bg-muted`, text `--weight-semibold`
- Row hover: `--color-bg-hover`
- Selected: `--color-bg-selected`
- Stripe: không dùng zebra, dùng border-bottom thay

---

## 4. Layout Patterns

### 4.1 Workspace Layout (Split-pane)

```
┌─────────────────────────────────────────────────────────┐
│ Header (44px) — Search · Breadcrumb · User actions      │
├──────────┬──────────────────────────┬───────────────────┤
│ Sidebar  │ Center Pane              │ Right Panel       │
│ (232px)  │ (flex-1)                 │ (320px, toggle)   │
│          │                          │                   │
│ Tree     │ Document Viewer          │ OCR Text          │
│ View     │ PDF/Image/Table          │ Metadata          │
│          │                          │ AI Notebook       │
│          │                          │ Citations         │
├──────────┴──────────────────────────┴───────────────────┤
│ Footer/Status Bar (26px)                                 │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Viewer Modes

| Mode | Layout | Khi nào |
|------|--------|---------|
| PDF only | Center full | Đọc tài liệu gốc |
| OCR only | Center = text editor | Sửa text OCR |
| Split | Center = PDF bên trái, OCR bên phải | So sánh gốc vs OCR |
| AI Workspace | Center = Viewer + Right = AI Notebook | Phân tích AI |

### 4.3 Document Viewer

- Thumbnails panel bên trái (toggle)
- Toolbar: page navigation, zoom, fit, rotate, rescan OCR
- OCR overlay: blocks hiển thị trên ảnh, click = select
- Low confidence blocks: viền vàng/đỏ

---

## 5. Micro-interactions (Bắt buộc)

### 5.1 Transitions

```css
--transition-fast: 100ms ease;  /* Hover effects */
--transition-base: 180ms ease;  /* State changes */
--transition-slow: 280ms ease;  /* Layout shifts */
```

### 5.2 Interactions cần có

| Interaction | Hành vi |
|------------|---------|
| **Space preview** | Chọn file + Space → modal preview nhanh (macOS QuickLook style) |
| **Search → Viewer** | Click result → mở viewer, cuộn đúng trang, highlight keyword |
| **Sidebar hover** | Item dịch phải 7px, opacity tăng |
| **Card hover** | Shadow nâng sm→md, border sáng lên |
| **Status transition** | Badge đổi màu với fade 180ms |
| **Progress mini** | Import/OCR tiến trình ở footer, click = expand |
| **Right-click context** | Trên text: "Thêm ghi chú", "Giải thích", "Trích dẫn" |
| **Drag & drop** | Kéo thả tài liệu trong sidebar tree |

### 5.3 Loading states

- Skeleton loader cho cards/lists (không dùng spinner)
- Progress bar cho pipeline jobs
- Shimmer effect cho thumbnails đang render

---

## 6. Accessibility & i18n

- Vietnamese là ngôn ngữ chính
- Tất cả interactive elements phải có `id` duy nhất cho test
- Focus ring: `--shadow-ring` (2px primary border)
- Color contrast: ratio ≥ 4.5:1 cho text
- Không dùng color alone để truyền thông tin (kèm icon/text)

---

## 7. CSS File Map

| File | Vai trò | Khi nào sửa |
|------|---------|-------------|
| `variables.css` | Design tokens — SINGLE SOURCE | Khi đổi màu/font/spacing |
| `layout.css` | Grid, flex, responsive | Khi đổi layout structure |
| `dossier-nav.css` | Sidebar/navigation | Khi sửa sidebar |
| `components.css` | Cards, badges, buttons, forms | Khi thêm component |

### Quy tắc CSS

1. **Luôn dùng CSS variables** — không hardcode hex/px
2. **Không dùng TailwindCSS** trừ khi user yêu cầu
3. **BEM-lite naming**: `.card`, `.card-header`, `.card--active`
4. **Không !important** trừ utility override
5. **Mobile-last** — desktop mặc định, responsive khi cần

---

## 8. Nguồn tham khảo

| Nguồn | URL | Áp dụng |
|-------|-----|---------|
| Linear Design | `getdesign.md/linear.app/design-md` | Sidebar, navigation, minimal |
| Notion Design | `getdesign.md/notion/design-md` | Workspace, warm, document |
| Superhuman Design | `getdesign.md/superhuman/design-md` | Premium, keyboard, dark |
| Raycast Design | `getdesign.md/raycast/design-md` | Desktop-native, command |
| UI/UX Proposal | `v3/03_specs/02_UI_UX_IMPROVEMENT_PROPOSAL.md` | Split-pane, viewers |
| Karpathy Guidelines | `v3/07_external_refs/SKILL.md` | Coding behavior (đã merge vào AGENTS) |
