# BẢN ĐỒ LIÊN KẾT GIỮA FILE THÔNG TIN, THỰC THỂ, SỰ KIỆN

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa bản đồ liên kết giữa file thông tin, thực thể (entities), sự kiện, vật chứng, và citation trong hệ thống.
Nguon prompt: Tu yeu cau ve entity relationship graph.

---

## 1. Tong Quan Entity Links

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal relationship graph**
- **KHÔNG có entity linking**
- **KHÔNG có cross-reference**

### 1.2 Muc Tieu Cua File Nay

- Entity relationship graph
- Cross-reference system
- Evidence linking
- Citation network

---

## 2. Entity Types & Relationships

### 2.1 Entity Types

| Entity Type | Icon | Color | Description |
|------------|------|-------|-------------|
| person | 👤 | Blue | Bi can, nhan chung |
| device | 📱 | Gray | Dien thoai, may tinh |
| evidence | 📦 | Green | Vat chung |
| event | 📅 | Purple | Su kien |
| location | 📍 | Orange | Dia diem |

### 2.2 Relationship Types

```
┌─────────────────────────────────────────────────────────────────────┐
│                   RELATIONSHIP TYPES                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Person → Person                                                    │
│  ├─ family:          Gia dinh                                     │
│  ├─ coworker:       Dong ngiep                                      │
│  ├─ associate:      Lien quan biet                               │
│  └─ co_defendant:   Dong pham                                      │
│                                                                      │
│  Person → Device                                                  │
│  ├─ owner:         Chu so huu                                        │
│  ├─ user:         Nguoi su dung                                   │
│  └─ previous_owner: Chu so cu                                      │
│                                                                      │
│  Person → Evidence                                               │
│  ├─ owns:          So huu                                          │
│  ├─ found:        Phat hien                                        │
│  └─ transferred: Chuyen nhuong                                    │
│                                                                      │
│  Event → Person                                                   │
│  ├─ participant:  Tham gia                                       │
│  ├─ witness:     Nhan chung                                        │
│  └─ victim:     Bi hai                                            │
│                                                                      │
│  Event → Location                                                 │
│  └─ occurred_at: Xay ra tai                                      │
│                                                                      │
│  Evidence → Event                                                │
│  └─ related_to: Lien quan voi                                    │
│                                                                      │
└─────��───────────────────────────────────────────────────────────────┘
```

---

## 3. Relationship Graph

### 3.1 Sample Graph

```
                                    ┌──────────────┐
                              ┌──────│   Event    │◄──────┐
                              │      │ Bi bat     │       │
                              │      └──────────────┘       │
                              │            │               │
                              │            │               │
                    ┌─────────┴──┐  ┌───┴────┐  ┌──────┴──────┐
                    │            │  │       │  │              │
                    ▼            ▼  ▼       ▼  ▼              ▼
              ┌──────────┐  ┌──────────┐  ┌──────┐  ┌─────────────┐
              │ Person A │  │ Person B │  │Person│  │  Location  │
              │ (Bi can) │──│(Nhan chg)│  │  C  │  │  Q.1,TpHCM │
              └──────────┘  └──────────┘  └──────┘  └─────────────┘
                    │            │
                    │            │                Device Links
                    │            │                ───────────────
                    │            │          ┌──────┐  ┌───────┐
                    │            └──────────│Phone │  │Laptop │
                    │                     │      │  │       │
                    │                     └──────┘  └───────┘
                    │
                    │            Evidence Links
                    │            ───────────────
                    │      ┌──────┐    ┌──────┐
                    └──────│Tien  │    │Dien  │
                         │1.5tr │    │thoai │
                         └──────┘    └──────┘
```

### 3.2 Graph Queries

| Query | Description | Example |
|-------|------------|----------|
| neighbors | All connected entities | "Ai lien quan toi A?" |
| path | Multi-hop relationships | "A → B → C" |
| common | Shared connections | "A va B cung lien quan toi nao?" |
| timeline | Events in sequence | "Su kien theo thoi gian" |

---

## 4. Cross-Reference System

### 4.1 Reference Types

| Reference | Symbol | Usage |
|-----------|--------|-------|
| document | 📄 | Link to document |
| page | 📄 # | Link to page |
| citation | "..." | Link to citation |
| entity | @person | Link to entity |
| event | *event | Link to event |
| location | @place | Link to location |

### 4.2 Auto-Detection

```
OCR Text: "To khai Nguyen Van A, CMND 012345678, ngay 20/01/2026 tai tru so VKS Q.1"
         │
         ▼
┌─────────────────────┐
│ NER Detection       │
├──────────────────────┤
│ Person: "Nguyen Van A"│
│ ID: "012345678"     │
│ Date: "20/01/2026"   │
│ Location: "VKS Q.1" │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ Create Links        │
├──────��───────────────┤
│ Link to person entity│
│ Link to date event   │
│ Link to location   │
└─────────────────────┘
```

---

## 5. Evidence Management

### 5.1 Evidence Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                    DANH SACH VAT CHUNG (15)                           │
├─────────────────────────────────────────────────────────────────────┤
│  # │ Ten           │ Loai     │ Nguon     │ Ngay     │ Lien ket      │
│  1 │ Tien mat 1.5t│ Tien    │ Bi can A  │ 20/01/26│ Person A    │
│  2 │ iPhone 13    │ Device  │ Kiem keat │ 20/01/26│ Person A    │
│  3 │ Laptop Dell  │ Device  │ Kiem keat │ 20/01/26│ Person A    │
│  4 │ Bao hiem    │ Document│ Bi can A  │ 15/01/26│ Person A    │
│  5 │ GPS data    │ Data    │ Phone    │ 20/01/26│ Device #2   │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 Evidence Fields

| Field | Type | Description |
|-------|------|-------------|
| evidence_id | UUID | Unique ID |
| name | Text | Ten vat chung |
| evidence_type | Enum | Loai (device, money, document, data) |
| source | Text | Nguon cap |
| acquisition_date | Date | Ngay thu thap |
| linked_persons | Array | Lien ket nguoi |
| linked_events | Array | Lien ket su kien |
| chain_of_custody | Array | Chuoi so huu |

### 5.3 Chain of Custody

```
Chain of Custody: iPhone 13
─────────────────────────────────────────
20/01/2026 09:00  │ Bien ban kiem keat
                   │ Nguoi giao: Bi can A
                   │ Nguoi nhan: KS Le Thi B
                   │
20/01/2026 14:00  │ Chuyen giao cho can bo ki nghiem
                   │ Nguoi giao: KS Le Thi B  
                   │ Nguoi nhan: KS Tran C
                   │
21/01/2026 10:00  │ Phan tich du lieu
                   │ Nguoi giao: KS Tran C
                   │ Nguoi nhan: Lab Tech D
                   │
─────────────────────────────────────────
```

---

## 6. Citation Network

### 6.1 Citation Graph

```
┌─────────────────────────────────────────────────────────────────────┐
│                     CITATION NETWORK                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Document 1 (To khai)                                                 │
│  ├─Citation 1: "To pham..." (page 1) ──┐                           │
│  │                                     │                           │
│  │                                     ▼                           │
│  │                              ┌─────────────────────┐              │
│  │                              │ Person: Nguyen Van A│              │
│  │                              └─────��─��─────────────┘              │
│  │                                     │                           │
│  │                                     │                           │
│  │                              ┌─────────────────────┐              │
│  │                              │ Evidence: Tien 1.5t  │              │
│  │                              └─────────────────────┘              │
│  │                                     │                           │
│  │                                     │                           │
│  │                              ┌─────────────────────┐              │
│  │                              │ Event: Bi bat 30/12  │              │
│  │                              └─────────────────────┘              │
│  │                                                                    │
│  └─Citation 2: "Khai bay..." (page 3)                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 6.2 Network Analysis

| Analysis | Description |
|----------|-------------|
| Person network | All persons connected to case |
| Evidence chain | Chain of evidence |
| Timeline flow | Events in order |
| Contradiction | Conflicting citations |

---

## 7. Location Tracking

### 7.1 Location Types

| Type | Example | GPS |
|------|---------|-----|
| address | 123 Nguyen Hue, Q.1 | Required |
| event_location |Bien ban tai nha may | Optional |
| workplace | Cong ty X | Optional |
| capture_location | Noi chup anh | Required |

### 7.2 Location Links

```
Location: Restaurant ABC
    │
    ├── Linked Events (5)
    │   ├── Event 1: An com (20/12/2025)
    │   ├── Event 2: Gap mat (25/12/2025)
    │   └── ...
    │
    ├── Linked Persons (10)
    │   ├── Person A - regular customer
    │   ├── Person B - owner
    │   └── ...
    │
    └── Evidence (2)
        └── Receipt #1
        └── CCTV footage
```

---

## 8. State Management

### 8.1 Entity Store

```typescript
interface EntityState {
  entities: Entity[];
  relationships: Relationship[];
  currentEntity: Entity | null;
  
  // Actions
  fetchByCase: (caseId: string) => Promise<void>;
  createEntity: (data: CreateEntityInput) => Promise<Entity>;
  linkEntities: (from: string, to: string, type: string) => Promise<void>;
  getNeighbors: (entityId: string) => Entity[];
}
```

---

## 9. Acceptance Criteria

- [ ] Entity types day du
- [ ] Relationships xac dinh
- [ ] Graph hien thi
- [ ] Cross-reference work
- [ ] Evidence management
- [ ] Citation network

---

## 10. Khong Duoc Hieu Sai

- **Entity KHONG phai la database row** - chi representation
- **Graph KHONG phai la social network** - chi case-specific
- **Evidence KHONG phai la file** - structured object

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom E - Nghiep vu*
*Tiep theo: File 13 - Ban do tri thuc man hinh*