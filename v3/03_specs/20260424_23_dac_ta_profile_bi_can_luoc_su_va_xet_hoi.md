# ĐẶC TẢ PROFILE BỊ CAN LỘC SỬ VÀ XÉT HỎI

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa profile bị can đầy đủ, lược sử, xét hỏi và mind-map màn hình cho việc quản lý thông tin bị can.
Nguon prompt: Tu yeu cau ve suspect profile va interrogation.

---

## 1. Tong Quan Suspect Profile

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal suspect profile**
- **KHÔNG có interrogation record**
- **KHÔNG cótimeline**

### 1.2 Muc Tieu Cua File Nay

- Suspect profile đầy đủ
- Interrogation records
- History tracking
- Timeline view

---

## 2. Suspect Profile

### 2.1 Profile Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│                    PROFILE: NGUYEN VAN A                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│    ┌─────────────────┐    ┌────────────────────────────────────┐    │
│    │                 │    │  THÔNG TIN CƠ BẢN                    │    │
│    │   [AVATAR]     │    │  ─────────────────────               │    │
│    │   200x200      │    │  Ho ten: Nguyen Van A               │    │
│    │                 │    │  Nam sinh: 1985                    │    │
│    │                 │    │  Gioi tinh: Nam                     │    │
│    └─────────────────┘    │  Noi sinh: Tp.HCM                  │    │
│                           │  CMND: 012345678                   │    │
│                           │  Ngay cap: 15/01/2010              │    │
│                           │  Noi cap: Tp.HCM                   │    │
│                           │  Noi o: Q.1, Tp.HCM               │    │
│                           └────────────────────────────────────┘    │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│    THÔNG TIN NGHỀ NGHIỆP                    │  TÌNH TRẠNG           │
│    ──────────────────────                    │  ────────────          │
│    Nghe nghiep: Lao dong                    │  Hon nhan: That       │
│    Noi lam viec: Cty X                     │  Con: 2               │
│    Thu nhap: 10tr/thang                   │  Thuong benh: Khong   │
│    Chuc vu: Nhan vien                     │  Nghiện: Binh thuong │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      ���
│    LIÊN HỆ                               │  LỊCH SỬ PHÁP LUẬT     │
│    ─────────────────────               │  ────────────────         │
│    Dien thoai: 0912 xxx xxx          │  Tien su: Chua          │
│    Email: nva@email.com             │  Toi pham: Diep am    │
│    Facebook: nva.fb                  │  Tuy phat: 12 thang  │
│    Nguoi thanh: [Link]              │  Chuong trinh: ....   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Profile Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | Text | ✅ | Ho ten |
| alias | Text | | Ten goi khac |
| birth_year | Number | ✅ | Nam sinh |
| gender | Select | ✅ | Gioi tinh |
| birth_place | Text | | Noi sinh |
| identification | Text | | So CMND/CCCD |
| id_issue_date | Date | | Ngay cap CMND |
| id_issue_place | Text | | Noi cap CMND |
| address | Text | | Dia chi hien tai |
| occupation | Text | | Nghe nghiep |
| workplace | Text | | Noi lam viec |
| income | Text | | Thu nhap |
| phone | Text | | Dien thoai |
| email | Text | | Email |
| marital_status | Select | | Tinh trang hon nhan |
| children | Number | | So con |
| medical_history | Text | | Benh su |
| severity | Select | | Muc do nghiem |

---

## 3. Interrogation Records (Xet Hoi)

### 3.1 Record Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                   XET HOI LAN 1                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                      │
│  Thong tin co ban                                       │
│  ──────────────────────                              │
│  So: 001        |  Ngay: 20/01/2026              │
│  Gio: 14:00    |  Dia diem: Tru so VKS            │
│  Nguoi phỏng: KS Le Thi B                            │
│  Nguoi ghi: Le Thi B                              │
│                                                      │
├─────────────────────────────────────────────────────────────────────┤
│  Noi dung                                         │
│  ────────────────────────────────────────────────    │
│  Hoi: Tai sao anh bi cho lai cong ty?                │
│                                                      │
│  TL: To di lam, bi o giam lai.                     │
│                                                      │
│  Hoi: Anh co biet vi sao bi goi khong?              │
│                                                      │
│  TL: To khong biet. To chi biet la bi goi den day.  │
│                                                      │
│  ...                                                │
│                                                      │
├─────────────────────────────────────────────────────────────────────┤
│  Ky                                                 │
│  [Chu ky nguoi hoi]           [Chu ky nguoi bi hoi]    │
│                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Interrogation Fields

| Field | Type | Description |
|-------|------|-------------|
| sequence_no | Number | So thu tu |
| session_date | DateTime | Ngay gio |
| location | Text | Dia diem |
| interviewer | Text | Nguoi hoi |
| recorder | Text | Nguoi ghi |
| questions | Array | Cau hoi |
| answers | Array | Cau tra loi |
| signature_interviewer | Image | Chu ky nguoi hoi |
| signature_interrogee | Image | Chu ky bi hoi |

### 3.3 List View

```
┌─────────────────────────────────────────────────────────────────────┐
│                   DANH SACH XET HOI (5)                         │
├─────────────────────────────────────────────────────────────────────┤
│  # │ Ngay      │ Gio │ Nguoi hoi │ Tom tat    │ Trang thai      │
│  1 │ 20/01/26 │ 14:00│ Le Thi B  │ "Gioi thieu" │ Đa duyet    │
│  2 │ 21/01/26 │ 09:00│ Le Thi B  │ "Loi khai" │ Cho duyet │
│  3 │ 22/01/26 │ 14:00│ Tran C   │ "Bo sung" │ Cho duyet │
│  4 │ 23/01/26 │ 09:00│ Tran C   │ "Tai khai"│ Cho duyet │
│  5 │ 24/01/26 │ 14:00│ Tran C   │ "Ket thuc"│ Cho duyet │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. History (Luoc Su)

### 4.1 History Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                     LICH SU BI CAN                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  GIA DINH                                                   │
│  ────────────────────────────────────────────────────────────────          │
│  Cha: Nguyen Van B (da mất)                                  │
│  Me: Tran Thi C                                              │
│  Anh chi: [Link to profiles]                                  │
│  Con: 2                                                    │
│                                                                      │
│  DAO TAO                                                    │
│  ────────────────────────────────────────────────────────────────          │
│  1992-2000: Truong Tieu hoc A                               │
│  2000-2003: Truong THCS B                                    │
│  2003-2006: Truong THPT C                                    │
│                                                                      │
│  NGHE NGHIEP                                              │
│  ────────────────────────────────────────────────────────────────          │
│  2006-2010: Cong nhan Cty X                                  │
│  2010-2015: Truong phong Cty Y                               │
│  2015-Hien tai: Tu do                                       │
│                                                                      │
│  TIEN SU PHAP LY                                           │
│  ────────────────────────────────────────────────────────────────          │
│  2015: Vi pham TTAT (xu phat 5 trieu)                      │
│  2018: To pham (treo 6 thang)                              │
│  2020: Gian lan (treo 12 thang)                             │
│                                                                      │
│  TIEU SU                                              │
│  ────────────────────────────────────────────────────────────────          │
│  Thuong tich: Khong                                          │
│  Nghiện: Khong                                           │
│  Nghiện: Khong                                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 History Sections

| Section | Fields |
|---------|--------|
| **Gia dinh** | parents, siblings, spouse, children |
| **Dao tao** | education[] |
| **Nghe nghiep** | employment[] |
| **Tien su phap ly** | criminal_record[] |
| **Tieu su** | characteristics, habits |

---

## 5. Timeline View

### 5.1 Timeline Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                     TIMELINE                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  2025-12  ●────────────────────────────────────────● 2026-04         │
│          │                                                    │         │
│          │                                                    ▼         │
│     events:                                               events:          │
│  ┌──────────────┐                                      ┌──────────┐  │
│  │ 30/12/2025 │                                      │20/01/26 │  │
│  │ Bi bat     │                                      │Xet hoi 1│  │
│  │     ●─────┼──────────────────────────────────────│────●    │  │
│  │          │                                      │         │  │
│  │          │                                      │         │  │
│  │          │                           ┌───────────┤         │  │
│  │          │                           │26/01/26  │         │  │
│  │          │                           │Qd khoi to │────●    │  │
│  │          │                           └───────────┤         │  │
│  │          │                                       │         │  │
│  │          │                                       │27/01/26 │  │
│  │          │                                       │Qd tam giam──●    │
│  └──────────────┘                                       │         │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 Event Types

| Type | Icon | Color |
|------|------|-------|
| arrest | 🔒 | Red |
| interrogation | ❓ | Blue |
| decision | ⚖️ | Purple |
| detention | 🏢 | Orange |
| evidence | 📄 | Green |
| transfer | ➡ | Gray |
| release | 🔓 | Green |

---

## 6. Evidence Links

### 6.1 Link Structure

```
Profile: Nguyen Van A
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     EVIDENCE LINKS                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Documents (15)                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ [✓] To khai 01 - "To pham..." | 20/01/2026 | to_khai       │   │
│  │ [✓] To khai 02 - "Khai bay..."    | 21/01/2026 | to_khai   │   │
│  │ [✓] Bien ban hop 01              | 22/01/2026 | bien_ban   │   │
│  │ ...                                                        │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  Citations (23)                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ "To pham theo..." | Tr.1 | admission | Level: high          │   │
│  │ "Khai bay..."    | Tr.3 | evidence | Level: medium         │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  Devices (2)                                                        │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ iPhone 13 - [Link]                                         │   │
│  │ Laptop Dell - [Link]                                       │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 7. State Management

### 7.1 Dossier Store

```typescript
interface DossierState {
  profiles: Entity[];
  currentProfile: Entity | null;
  interrogations: Interrogation[];
  history: HistoryEntry[];
  timeline: TimelineEvent[];
  
  // Actions
  fetchProfile: (id: string) => Promise<Entity>;
  createProfile: (data: CreateEntityInput) => Promise<Entity>;
  addInterrogation: (data: AddInterrogationInput) => Promise<void>;
  addHistory: (data: AddHistoryInput) => Promise<void>;
}
```

---

## 8. Acceptance Criteria

- [ ] Profile hien thi day du thong tin
- [ ] Interrogation records day du
- [ ] History tracking day du
- [ ] Timeline view chinh xac
- [ ] Evidence links work

---

## 9. Khong Duoc Hieu Sai

- **Profile KHONG phai la database row** - chi display, database rieng
- **Interrogation KHONG phai la case note** - structured record
- **Timeline KHONG phai la calendar** - event-based view

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom E - Nghiep vu*
*Tiep theo: File 12 - Ban do lien ket*