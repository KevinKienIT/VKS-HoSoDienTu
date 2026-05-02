# ĐẶC TẢ MÀN HÌNH QUẢN LÝ HỒ SƠ VÀ KHỐI HỒ SƠ

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa màn hình quản lý hồ sơ (dossier) và khối hồ sơ (case block) cho việc quản lý case và tài liệu.
Nguon prompt: Tu yeu cau ve case management va dossier.

---

## 1. Tong Quan Case & Dossier Management

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal case detail screen**
- **KHÔNG có dossier panel**
- **KHÔNG có case block organization**

### 1.2 Muc Tieu Cua File Nay

- Case detail screen layout
- Dossier panel (hồ sơ bị can)
- Case block organization
- Document grouping

---

## 2. Case Detail Screen

### 2.1 Full Layout

```
┌──────────────────────────────────────────────────────────────────────┐
│ [←Back]  Ho So: VK-2026-00123          [Edit] [Export] [Archive]   │
├──────────────────────────────────────────────┬───────────────────┤
│                                            │                   │
│  THONG TIN CASE                          │   DOSSIER PANEL    │
│  ─────────────────                      │   ────────────    │
│  Ma case: VK-2026-00123                 │                   │
│  Ten bi can: Nguyen Van A               │   [Avatar]        │
│  Loai: To Dieu Tra                     │   Nguyen Van A    │
│  Ngay tao: 2026-04-20                 │   Nam sinh: 1985   │
│  Trang thai: Dang xu ly                │   Noi sinh: TpHCM │
│                                        │                   │
├─────────────────────────────────────────┤   CMND: 012345678 │
│  TAI LIEU (15)                       │                   │
│  ─────────────────                     │   [Timeline]       │
│  ┌────────────────────────────────┐  │   [Xet hoi]      │
│  │ [✓] To khai ban 1     20/01      │  │   [Luoc su]      │
│  │ [✓] Bien ban hop   22/01        │  │                   │
│  │ [⚠] Qd khoi to     25/01       │  │   ────────────  │
│  │ [ ] Qd gia han    28/01        │  │                   │
│  └────────────────────────────────┘  │   CONNECTION     │
│                                        │   ────────────  │
│  [Timeline]  [Citations]  [Notes]      │   + Add link    │
│                                        │                   │
└──────────────────────────────────────────────┴───────────────────┘
```

### 2.2 Tabs

| Tab | Content | Route |
|-----|----------|-------|
| **Tong quan** | Case info + documents | `:id` |
| **Tai lieu** | Document list | `:id/documents` |
| **Timeline** | Event timeline | `:id/timeline` |
| **Trich dan** | Citations | `:id/citations` |
| **Ho so** | Dossier panel | `:id/dossier` |
| **Ghi chu** | Notes | `:id/notes` |

---

## 3. Case Information Panel

### 3.1 Information Fields

| Field | Type | Editable | Description |
|-------|------|----------|-------------|
| case_code | Text | Read-only | VK-2026-00123 |
| case_display_name | Text | Edit | Ten hien thi |
| primary_person_name | Text | Edit | Ten bi can |
| case_type | Select | Edit | Loai case |
| case_group_label | Select | Edit | Nhom case |
| investigation_level | Select | Edit | Cap do diều tra |
| prosecutor_office | Text | Edit | Vien KS |
| investigator_name | Text | Edit | KSV phu trach |
| severity | Select | Edit | Muc do nghiem trong |
| status | Toggle | Edit | Active/Archived |
| created_at | DateTime | Read-only | Ngay tao |
| updated_at | DateTime | Read-only | Ngay cap nhat |

### 3.2 Actions

| Action | Icon | Description |
|--------|------|-------------|
| Edit | ✏️ | Chinh sua case |
| Export | 📤 | Xuat bao cao |
| Archive | 📦 | Luu tru |
| Delete | 🗑️ | Xoa (confirm) |

---

## 4. Document List

### 4.1 List Layout

```
┌─────────────────────────────────────────────────────────────┐
│ [Tai lieu] [15]    [+ Them tai lieu]  [Sort ▾] [Filter ▾]│
├─────────────────────────────────────────────────────── │
│ ┌───────────────────────────────────────────────┐     │
│ │ □ │ [icon] │ Ten tai lieu      │ Loai │ Ngay │▼ │
│ │───│───────│──────────────────│──────│─────│──│
│ │ ✓ │ 📄   │ To khai ban 1     │ to  │ 20/ │  │
│ │ ✓ │ 📄   │ Bien ban hop     │ bb  │ 22/ │  │
│ │ ⚠ │ 📄   │ Qd khoi to      │ qd  │ 25/ │  │
│ │   │ 📄   │ Qd gia han      │ qd  │ 28/ │  │
│ └───────────────────────────────────────────────┘     │
├─────────────────────────────────────────────────────── │
│ Showing 1-4 of 15            [< Prev] [1] [2] [Next >]  │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Document Columns

| Column | Width | Sortable | Description |
|--------|-------|----------|-------------|
| Checkbox | 40px | | Selection |
| Icon | 40px | | Document type |
| Ten tai lieu | flex | ✅ | Document name |
| Loai | 120px | ✅ | Document type |
| Ngay | 100px | ✅ | Issued date |
| Trang thai | 100px | ✅ | Review status |
| Actions | 80px | | Row actions |

### 4.3 Document Row Actions

| Action | Shortcut | Description |
|--------|----------|-------------|
| View | Click | Open viewer |
| Edit | Click | Edit metadata |
| Delete | Click | Delete (confirm) |

### 4.4 Document Type Icons

| Type | Icon | Color |
|------|------|-------|
| to_khai | 📄 | Blue |
| bien_ban | 📋 | Green |
| quyet_dinh | ⚖️ | Purple |
| ket_luan | 📊 | Orange |
| phieu | 📋 | Gray |

---

## 5. Dossier Panel (Ho So Bi Can)

### 5.1 Layout

```
┌─────────────────────────────┐
│        HO SO BI CAN        │
│    ─────────────────    │
│      ┌─────────┐        │
│      │ [Img]  │        │
│      └─────────┘        │
│        Nguyen Van A      │
│    ─────────────────    │
│    Nam sinh: 1985    │
│    Gioi tinh: Nam      │
│    Noi sinh: TpHCM    │
│    CMND: 012345678    │
│    Noi o: Q.1, TpHCM  │
│    ─────────────────    │
│    [Xet hoi]          │
│    [Luoc su]          │
│    [Timeline]         │
│    ─────────────────    │
│    Cong ngiep: Lao dong │
│    Tinh trang: That   │
│    nghiem: Binh thuong │
└─────────────────────────────┘
```

### 5.2 Information Fields

| Field | Type | Description |
|-------|------|-------------|
| avatar | Image | Anh bi can |
| name | Text | Ho ten |
| alias | Text | Ten goi khac |
| birth_year | Number | Nam sinh |
| gender | Select | Gioi tinh |
| birth_place | Text | Noi sinh |
| identification | Text | So CMND/CCCD |
| address | Text | Dia chi |
| occupation | Text | Nghe nghiep |
| status | Text | Tinh trang hon nhan |
| severity | Text | Muc do nghiem trong |

### 5.3 Dossier Tabs

| Tab | Description |
|-----|-------------|
| **Thong tin** | Basic info |
| **Xet hoi** | Interrogation records |
| **Luoc su** | History |
| **Timeline** | Su kien |

### 5.4 Dossier Actions

| Action | Description |
|--------|-------------|
| Edit | Update info |
| Add note | Add note |
| Link citation | Link to citation |
| View history | View changes |

---

## 6. Case Block Organization

### 6.1 Block Structure

```
Case
├── Thong tin case
├── Tai lieu
│   ├── To khai (3)
│   │   ├── To khai ban 1
│   │   ├── To khai ban 2
│   │   └── To khai bo sung
│   ├── Bien ban (5)
│   └── Quyet dinh (7)
├── Trich dan
├── Ho so
├── Ghi chu
└── Timeline
```

### 6.2 Document Grouping

| Group | Types | Default Sort |
|-------|-------|-------------|
| To khai | to_khai | Ngay tang |
| Bien ban | bien_ban | Ngay tang |
| Quyet dinh | quyet_dinh | Ngay tang |
| Phieu | phieu, giay | Ngay tang |
| Khac | others | Ngay tang |

### 6.3 Filtering

| Filter | Options |
|--------|----------|
| Loai tai lieu | All, To khai, Bien ban, Quyet dinh, ... |
| Trang thai | All, Pending, Reviewed, Error |
| Ngay | Range picker |
| Nguoi ban hanh | Search |

---

## 7. Search & Filter

### 7.1 Case Search

```
┌──────────────────────────────────────────────────┐
│ [🔍 Tim kiem...]              [Tim] [Xoa]         │
├──────────────────────────────────────────────────┤
│ Loai:  [All ▼]                             │
│ Trang thai: [All ▼]                       │
│ Ngay tao: [Tu] ---- [Den]                 │
│ KSV:      [All ▼]                        │
└──────────────────────────────────────────────────┘
```

### 7.2 Quick Filters

| Filter | Description |
|--------|-------------|
| Tat ca | All cases |
| Dang xu ly | Active cases |
| Da dong | Archived |
| Cuu toi | My cases |
| Gan day | Recently opened |

---

## 8. State Management

### 8.1 Case Store

```typescript
interface CaseState {
  cases: Case[];
  currentCase: Case | null;
  loading: boolean;
  error: string | null;
  
  // Filters
  filters: CaseFilters;
  sort: CaseSort;
  
  // Actions
  fetchCases: () => Promise<void>;
  getCaseById: (id: string) => Case | undefined;
  createCase: (data: CreateCaseInput) => Promise<Case>;
  updateCase: (id: string, data: UpdateCaseInput) => Promise<Case>;
  deleteCase: (id: string) => Promise<void>;
}
```

### 8.2 Document Store

```typescript
interface DocumentState {
  documents: Document[];
  selectedIds: string[];
  loading: boolean;
  
  // Actions
  fetchByCase: (caseId: string) => Promise<void>;
  importDocuments: (caseId: string, files: File[]) => Promise<void>;
  updateDocument: (id: string, data: UpdateDocInput) => Promise<void>;
  deleteDocuments: (ids: string[]) => Promise<void>;
}
```

---

## 9. Data Flow

### 9.1 Load Case

```
User clicks case
         │
         ▼
┌─────────────────────┐
│ Fetch case data     │
│ from database      │
├─────────────────────┤
│ Fetch documents    │
│ for this case     │
├─────────────────────┤
│ Fetch dossier     │
│ (person entity)   │
├─────────────────────┤
│ Update stores     │
└─────────────────────┘
```

### 9.2 Import Documents

```
User clicks import
         │
         ▼
┌─────────────────────┐
│ Select folder     │
│ (folder picker)  │
├─────────────────────┤
│ Scan files        │
│ + OCR processing │
├─────────────────────┤
│ Create records   │
│ in database     │
├─────────────────────┤
│ Update case     │
│ document count │
└─────────────────────┘
```

---

## 10. Acceptance Criteria

- [ ] Case detail screen hien thi day du thong tin
- [ ] Document list voi sort/filter
- [ ] Document grouping theo loai
- [ ] Dossier panel hien thi bi can info
- [ ] Case block organization
- [ ] Search/filter work
- [ ] CRUD operations work

---

## 11. Khong Duoc Hieu Sai

- **Dossier KHONG phai la CRM** - chi thong tin bi can, khong customer
- **Document grouping KHONG phai la folder** - chi grouping logic, khong file system
- **Case KHONG phai la project** - case trong KSV context

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom C - UI & Man hinh*
*Tiep theo: File 6 - Dac ta man hinh doc text*