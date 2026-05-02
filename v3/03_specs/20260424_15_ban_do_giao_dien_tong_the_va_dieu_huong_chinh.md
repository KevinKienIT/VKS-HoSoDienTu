# BẢN ĐỒ GIAO DIỆN TỔNG THỂ VÀ ĐIỀU HƯỚNG CHÍNH

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa giao diện tổng thể, điều hướng chính (main navigation), và cấu trúc layout của ứng dụng.
Nguon prompt: Tu yeu cau tong quat ve UI/UX design.

---

## 1. Tong Quan Giao Dien

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal navigation map**
- **KHÔNG có layout wireframe**
- **KHÔNG có screen flow diagram**

### 1.2 Muc Tieu Cua File Nay

- Tạo main navigation map
- Định nghĩa screen hierarchy
- Xác định navigation flows
- Chuẩn bị cho UI implementation

---

## 2. Screen Hierarchy

### 2.1 Level 1: App Shell

```
┌──────────────────────────────────────────────────────────────────────┐
│                      VK S ECMS                              │
│  ┌─────────────────────────────────────────────────────────┤
│  │ HEADER: Logo | Search | User Menu                       │
│  ├──────────┬──────────────────────────────────────────────┤
│  │          │                                              │
│  │ SIDEBAR  │              MAIN CONTENT                   │
│  │          │                                              │
│  │ Nav     │    (Screen area changes based on route)        │
│  │ Menu   │                                              │
│  │        │                                              │
│  │        │                                              │
│  │        │                                              │
│  ├──────────┴──────────────────────────────────────────────┤
│  │ FOOTER: Status Bar | Version                           │
│  └─────────────────────────────────────────────────────────┘
```

### 2.2 Level 2: Main Screens

| Screen | Route | Parent | Description |
|--------|-------|--------|----------|
| Dashboard | `/` | App Shell | Overview |
| Case List | `/cases` | App Shell | Case management main |
| Case Detail | `/cases/:id` | Case List | Individual case |
| Document Viewer | `/cases/:id/docs/:docId` | Case Detail | View document |
| Search | `/search` | App Shell | Global search |
| Review Queue | `/reviews` | App Shell | OCR review |
| User Management | `/admin/users` | App Shell (admin) | User admin |

### 2.3 Level 3: Detail Views

| View | Route | Parent | Description |
|------|-------|--------|----------|
| Dossier Panel | `:id/dossier` | Case Detail | Suspect profile |
| Timeline | `:id/timeline` | Case Detail | Event timeline |
| Citation Graph | `:id/citations` | Case Detail | Citation view |
| Document List | `:id/documents` | Case Detail | Case documents |
| Import Job | `/import` | App Shell | Import interface |

### 2.4 Level 4: Modal/Overlay

| Modal | Trigger | Description |
|-------|---------|-------------|
| Document Import | Click "Import" | Import files |
| Document Preview | Click document | Preview modal |
| Citation Create | Click "Add Citation" | Citation form |
| Note Edit | Click note | Note editor |
| User Edit | Click user | User form |

---

## 3. Main Navigation Map

### 3.1 Primary Navigation (Sidebar)

```
┌─────────────────────┐
│         VKS        │  ← Logo (click → Dashboard)
│        ECMS        │
├───────────────────┤
│                    │
│  📊 Dashboard     │  → / (home)
│                    │
│  📁 Ho So         │  → /cases
│     ├─ Tat ca     │
│     ├─ dang xu ly │  → /cases?status=active
│     ├─ da dong    │  → /cases?status=archived
│     └─ tim kiem   │
│                    │
│  📄 Tai Lieu      │  → /documents
│                    │
│  🔍 Tim Kiem      │  → /search
│                    │
│  📝 Review Queue │  → /reviews
│     ├─ OCR Review │
│     └─ Citation  │
│                    │
│  📥 Import Job   │  → /import
│                    │
├───────────────────┤
│                    │
│  👤 Tai Khoan     │  → /profile
│     ├─ Cau hinh  │
│     └─ Dang xuat │
│                    │
│  ⚙️ Admin         │  → /admin (admin only)
│     ├─ Nguoi dung│
│     └─ Cau hinh  │
└─────────────────────┘
```

### 3.2 Breadcrumb Navigation

```
Dashboard / Ho So / [Case Code] / Tai Lieu / [Document]
```

### 3.3 Context Navigation

| Screen | Context Actions |
|--------|------------------|
| Case Detail | New Document, Edit Case, Archive Case |
| Document Viewer | Add Citation, Add Note, Export |
| Search Results | Filter, Sort, Export Results |
| Review Queue | Approve, Reject, Skip |

---

## 4. Screen Flow Diagrams

### 4.1 Main User Flow

```
┌───────────────┐
│   Dashboard   │  ← Entry point (default)
└───────┬───────┘
        │
        ├──────────────────┐
        ▼                  ▼
┌───────────────┐    ┌───────────────┐
│ Case List    │    │  Search      │
│ (/cases)     │    │  (/search)   │
└───────┬───────┘    └───────┬───────┘
        │                  │
        ▼                  ▼
┌─────────────────────┐ ┌─────────────────────┐
│ Case Detail        │ │ Search Results      │
│ (/cases/:id)       │ │ (/search?q=...)    │
├─────────────────────┤ ├─────────────────────┤
│ • Dossier Panel   │ │ • Filter panel     │
│ • Document List  │ │ • Sort options     │
│ • Timeline      │ │ • Export         │
│ • Citations    │ │ • Click → Detail │
└─────────────────────┘ └─────────────────────┘
        │
        ├────┬────┬────┐
        ▼    ▼    ▼    ▼
┌─────────┐ ┌────┐ ┌────────┐ ┌─────────┐
│Document │ │    │ │Import  │ │Export   │
│Viewer   │ │M   │ │Job     │ │Report   │
└─────────┘ └────┘ └────────┘ └─────────┘
```

### 4.2 Document Import Flow

```
[Click Import]
       │
       ▼
┌───────────────┐
│ Import Modal │
└───────┬───────┘
        │
        ���
┌─────────────────────┐
│ 1. Select Folder    │
│ (folder picker)    │
└───────┬─────────────┘
        │
        ▼
┌─────────────────────┐
│ 2. Scan Files       │
│ (show progress)    │
└───────┬─────────────┘
        │
        ▼
┌─────────────────────┐
│ 3. OCR Processing   │
│ (show progress)    │
└───────┬─────────────┘
        │
        ▼
┌─────────────────────┐
│ 4. Review Items    │
│ (show issues)      │
└───────┬─────────────┘
        │
        ▼
┌─────────────────────┐
│ 5. Complete        │
│ (summary)         │
└─────────────────────┘
```

### 4.3 Review Flow

```
[Review Queue]
       │
       ▼
┌───────────────┐
│ Review List  │
└───────┬───────┘
        │
        ├────────────────────┐
        ▼                    ▼
┌───────────────┐    ┌───────────────┐
│ OCR Review   │    │ Citation Rev │
│ Item        │    │ Item         │
└───────┬───────┘    └───────┬───────┘
        │                  │
        ▼                  ▼
┌───────────────┐    ┌───────────────┐
│ Side-by-side  │    │ Edit Form    │
│ (OCR vs Img)  │    │ (Citation)  │
└───────┬───────┘    └───────┬───────┘
        │                  │
        ├────────────┐    │
        ▼            ▼    ▼
┌───────────────┐ ┌────┐ ┌───────────────┐
│ Approve       │ │Skip│ │ Reject        │
└───────────────┘ └────┘ └───────────────┘
```

---

## 5. Layout Specifications

### 5.1 Main Layout Dimensions

| Area | Size | Content |
|------|------|---------|
| Header | 48px height | Logo, Search, User |
| Sidebar | 240px (collapsed: 64px) | Navigation |
| Content | Flex (remaining) | Main screen |
| Footer | 32px height | Status, Version |

### 5.2 Responsive Breakpoints

| Breakpoint | Width | Sidebar | Content |
|------------|-------|---------|----------|
| Desktop | >= 1200px | Full | 3 columns |
| Laptop | 992-1199px | Collapsed | 2 columns |
| Tablet | 768-991px | Hidden | Full |
| Mobile | < 768px | Drawer | Single |

### 5.3 Component Spacing

| Token | Value | Usage |
|-------|-------|-------|
| xs | 4px | Icon padding |
| sm | 8px | Button padding |
| md | 16px | Card padding |
| lg | 24px | Section spacing |
| xl | 32px | Page margin |
| xxl | 48px | Major section |

---

## 6. Navigation Patterns

### 6.1 URL Structure

```
/                                  → Dashboard
/cases                             → Case list
/cases/:caseId                    → Case detail
/cases/:caseId/dossier             → Dossier panel
/cases/:caseId/timeline           → Timeline view
/cases/:caseId/documents          → Document list
/cases/:caseId/documents/:docId    → Document viewer
/cases/:caseId/documents/:docId?page=2  → Viewer at page 2
/cases/:caseId/citations          → Citation view
/search                           → Search
/search?q=keyword                → Search results
/search?type=to_khai              → Filtered search
/reviews                          → Review queue
/reviews/ocr                      → OCR review
/reviews/citation                 → Citation review
/admin/users                      → User management
/admin/settings                   → App settings
/profile                          → User profile
/import                           → Import job
```

### 6.2 Navigation Rules

| Rule | Description |
|------|-------------|
| Back button | Return to previous screen |
| Deep linking | Support direct URL access |
| Breadcrumb | Always show current path |
| Bookmark | Support case/doc permalink |
| State preservation | Keep filter/sort on back |

### 6.3 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Open search |
| `Ctrl+I` | Import |
| `Ctrl+S` | Save |
| `←/→` | Prev/Next page in viewer |
| `Escape` | Close modal |

---

## 7. Component Areas

### 7.1 Header Area

| Component | Position | Actions |
|-----------|-----------|---------|
| Logo | Left | Click → Dashboard |
| Global Search | Center | Input → Search |
| User Menu | Right | Click → Profile dropdown |
| Quick Actions | Right | Import, Notifications |

### 7.2 Sidebar Area

| Component | Description |
|-----------|-------------|
| Menu Items | Primary navigation |
| Collapsed Toggle | Collapse/expand |
| Case Counter | Active cases badge |
| Quick Filters | Recent cases |

### 7.3 Content Area

| Layout Mode | Usage |
|------------|-------|
| List | Case list, Document list |
| Detail | Case detail, Viewer |
| Split | Side-by-side review |
| Modal | Edit forms |

---

## 8. State Management

### 8.1 Navigation State

| State | Storage | Purpose |
|-------|---------|---------|
| currentRoute | URL | Current screen |
| history | Browser | Back navigation |
| filters | URL + Session | Search filters |
| sort | URL | Sort preferences |

### 8.2 UI State

| State | Storage | Purpose |
|-------|---------|---------|
| sidebarOpen | LocalStorage | Sidebar state |
| theme | Preferences | Theme setting |
| fontSize | Preferences | Font size |

### 8.3 App State

| State | Storage | Purpose |
|-------|---------|---------|
| currentCase | Zustand | Active case |
| currentDocument | Zustand | Active document |
| importJobs | Zustand | Import progress |

---

## 9. Error Handling

| Error | Display | Action |
|-------|---------|---------|
| 404 Not Found | "Khong tim thay" + Back button | Redirect to list |
| 500 Server Error | Error page + Retry | Log + Retry |
| Network Offline | Toast warning | Queue operations |
| Permission Denied | "Khong co quyen" + Contact admin | Show to admin |

---

## 10. Acceptance Criteria

- [ ] Screen hierarchy co 4 levels ro rang
- [ ] Main navigation map day du
- [ ] Screen flow diagrams cho user flows chinh
- [ ] URL structure dinh nghia
- [ ] Responsive breakpoints xac dinh
- [ ] Keyboard shortcuts xac dinh
- [ ] Error handling ro rang

---

## 11. Khong Duoc Hieu Sai

- **Screen hierarchy KHONG phai la wireframe** - chi la structure, implementation khac
- **Navigation map KHONG phai la routing code** - chi la definition
- **Layout dimensions KHONG phai la CSS** - chi la design guidance

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom C - UI & Man hinh*
*Tiep theo: File 4 - Dac ta man hinh quan ly trang*