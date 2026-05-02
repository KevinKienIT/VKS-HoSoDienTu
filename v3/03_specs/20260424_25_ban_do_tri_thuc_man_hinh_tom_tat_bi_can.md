# BẢN ĐỒ TRI THỨC MÀN HÌNH TÓM TẮT BỊ CAN

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa bản đồ tri thức màn hình tóm tắt (mind map dashboard) cho việc xem nhanh thông tin bị can.
Nguon prompt: Tu yeu cau ve mind map va summary dashboard.

---

## 1. Tong Quan Mind Map Dashboard

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có summary dashboard**
- **KHÔNG có mind map view**
- **KHÔNG có quick overview**

### 1.2 Muc Tieu Cua File Nay

- Summary dashboard
- Mind map view
- Quick overview
- Key indicators

---

## 2. Summary Dashboard Layout

### 2.1 Overview Screen

```
┌─────────────────────────────────────────────────────────────────────┐
│                    DASHBOARD: HO SO VK-2026-00123                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐│
│  │                    KEY INFORMATION                           ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐      ││
│  │  │ BI CAN   │  │ TOI PHAM │  │ CHUNG CU │  │ KET LUAN │      ││
│  │  │ [Avatar]│  │ Diep am │  │ 30/12   │  │  Q.Dinh  │      ││
│  │  │ Nguyen  │  │  §134    │  │  15trieu│  │  Khoi to │      ││
│  │  │ Van A   │  │ BPC    │  │         │  │         │      ││
│  │  │ Nam:1985│  │        │  │         │  │         │      ││
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘      ││
│  └────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ┌───────────────────────┐    ┌───────────────────────┐              │
│  │  TIMELINE            │    │  EVIDENCE SUMMARY     │              │
│  │  ─────────────────  │    │  ──────────────────── │              │
│  │                     │    │                       │              │
│  │  30/12: Bi bat     │    │  Documents:    15    │              │
│  │  20/01: Xet hoi 1  │    │  Evidence:     8      │              │
│  │  26/01: Qd khoi to │    │  Citations:    23     │              │
│  │  27/01: Tam giam   │    │  Persons:       5      │              │
│  │                     │    │                       │              │
│  │  [+ View All]      │    │  [+ View All]      │              │
│  └───────────────────────┘    └───────────────────────┘              │
│                                                                      │
│  ┌───────────────────────┐    ┌───────────────────────┐              │
│  │  DOCUMENT SUMMARY   │    │  KEY CITATIONS         │              │
│  │  ─────────────────  │    │  ──────────────────── │              │
│  │                     │    │                       │              │
│  │  📄 To khai:  3    │    │  "To pham..."      ✓  │              │
│  │  📋 Bien ban: 5    │    │  "Khai bay..."    ✓  │              │
│  │  ⚖️ Qdinh:   7    │    │  "Bi giam..."    ✓  │              │
│  │  📊 Other:    0    │    │                       │              │
│  │                     │    │  [+ View All]      │              │
│  └───────────────────────┘    └───────────────────────┘              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Quick Stats

| Metric | Value | Status |
|--------|-------|--------|
| Case ID | VK-2026-00123 | - |
| Primary Person | Nguyen Van A | Active |
| Crime | Điều 134 BPC | Primary |
| Status | Đang điều tra | Active |
| Documents | 15 | - |
| Citations | 23 | 20 confirmed |
| Interrogations | 5 | - |
| Days in custody | 85 days | - |

---

## 3. Mind Map View

### 3.1 Mind Map Structure

```
                                    ┌─────────────────┐
                                    │    CASE TITLE   │
                                    │  VK-2026-00123  │
                                    │ Nguyen Van A    │
                                    └────────┬────────┘
                                             │
                     ┌─────────────────────────────┼─────────────────────────────┐
                     │                         │                         │
                     ▼                         ▼                         ▼
            ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
            │  PERSON         │    │  EVIDENCE        │    │  EVENTS         │
            │  ────────────    │    │  ──────────      │    │  ───────────    │
            │                 │    │                 │    │                 │
            │ ┌───────────┐  │    │ ┌───────────┐  │    │ ┌───────────┐  │
            │ │Bi can     │  │    │ │Tien mat   │  │    │ │Bi bat     │  │
            │ │Nguyen A  │  │    │ │1.5 trieu │  │    │ │30/12     │  │
            │ └───────────┘  │    │ └───────────┘  │    │ └───────────┘  │
            │ ┌───────────┐  │    │ ┌───────────┐  │    │ ┌───────────┐  │
            │ │Nhan chung │  │    │ │Dien thoai │  │    │ │Xet hoi   │  │
            │ │Nguyen B  │  │    │ │iPhone   │  │    │ │20/01    │  │
            │ └───────────┘  │    │ └───────────┘  │    │ └───────────┘  │
            │ ┌───────────┐  │    │ ┌───────────┐  │    │ ┌───────────┐  │
            │ │Dong pham │  │    │ │Laptop    │  │    │ │Qd khoi to│  │
            │ │Tran C   │  │    │ │Dell     │  │    │ │26/01    │  │
            │ └───────────┘  │    │ └───────────┘  │    │ └───────────┘  │
            └─────────────────┘    └─────────────────┘    └─────────────────┘
                                             │
                                             │
            ┌─────────────────────────────────────┼─────────────────────────────────────┐
            │                         ▼                         │
            │              ┌─────────────────┐    │
            │              │  DOCUMENTS     │    │
            │              │  ──────────    │    │
            │              │                 │    │
            │              │ ┌───────────┐  │    │
            │              │ │To khai    │  │    │
            │              │ │(3 docs) │  │    │
            │              │ └───────────┘  │    │
            │              │ ┌───────────┐  │    │
            │              │ │Bien ban   │  │    │
            │              │ │(5 docs) │  │    │
            │              │ └───────────┘  │    │
            │              │ ┌───────────┐  │    │
            │              │ │Qdinh     │  │    │
            │              │ │(7 docs) │  │    │
            │              │ └───────────┘  │    │
            │              └─────────────────┘    │
```

### 3.2 Mind Map Interactive

| Feature | Description |
|---------|-------------|
| Click node | Show detail |
| Expand/Collapse | Show/hide children |
| Filter by type | Filter nodes |
| Search | Find node |
| Zoom | Adjust view |
| Export | Export image |

---

## 4. Key Facts Panel

### 4.1 Critical Facts

```
┌─────────────────────────────────────────────────────────────────────┐
│                        KEY FACTS                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ⭐ CAUSE OF ACTION                                                  │
│  ────────────────────────────────────────────────────────              │
│  • Bi cáo buộc phạm tội theo Điều 134 BPC                         │
│    (Cướp tài sản, bạo lực)                                     ��
│                                                                      │
│  🕐 TIMELINE SUMMARY                                                │
│  ────────────────────────────────────────────────────────              │
│  • 30/12/2025: Xảy ra vụ việc tại Q.1, TP.HCM                  │
│  • 30/12/2025: Bị bắt tại hiện trường                       │
│  • 20/01/2026: Khai nhận tội phạm                                   │
│  • 26/01/2026: Quyết định khởi tố                                │
│  • 27/01/2026: Tạm giam                                               │
│                                                                      │
│  💰 EVIDENCE                                                       │
│  ────────────────────────────────────────────────────────              │
│  • Tiền mặt thu hồi: 1.5 triệu đồng                          │
│  • Điện thoại iPhone 13 - Dữ liệu GPS                                      │
│  • Laptop Dell - Email liên lạc                                        │
│  • CCTV - Video hiện trường                                        │
│                                                                      │
│  👥 KEY PERSONS                                                   │
│  ────────────────────────────────────────────────────────              │
│  • Nguyen Van A - Bị can (Primary)                                │
│  • Nguyen Van B - Nhân chứng                                      │
│  • Tran Thi C - Bị hại                                              │
│                                                                      │
│  📝 CURRENT STATUS                                                │
│  ────────────────────────────────────────────────────────              │
│  • Đang tạm giam (Ngày 85)                                      │
│  • Đợi xét xử                                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 Warning Indicators

| Warning | Icon | Description |
|---------|------|-------------|
| Evidence gap | ⚠️ | Missing key evidence |
| Timeline gap | ⚠️ | Unknown period |
| Contradiction | ⚠️ | Conflicting info |
| Review needed | ⚠️ | Some OCR pending |
| Expired custody | 🔴 | Near deadline |

---

## 5. Quick Actions

### 5.1 Action Buttons

```
┌─────────────────────────────────────────────────────────────────────┐
│                        QUICK ACTIONS                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  [+ Add Document]  [+ Add Citation]  [+ Add Note]  [+ Add Event]       │
│         │              │              │              │                       │
│         ▼              ���              ▼              ▼                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐                 │
│  │ Import  │  │ Create  │  │ Create   │  │ Add     │                 │
│  │ new    │  │ new     │  │ note    │  │ event   │                 │
│  │ file   │  │ citation│  │         │  │ timeline│                 │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘                 │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 Context Actions

| Action | Location | Result |
|--------|----------|--------|
| View in Viewer | Click document | Open viewer |
| Add to Timeline | Click event | Add event form |
| Link Entity | Click person | Add relationship |
| Export Case | Menu | Generate report |

---

## 6. Indicators & Alerts

### 6.1 Status Indicators

| Indicator | Color | Meaning |
|------------|-------|---------|
| 🟢 Active | Green | Case in progress |
| 🟡 Pending | Yellow | Awaiting action |
| 🔴 Expired | Red | Overdue |
| ⚪ Closed | Gray | Case closed |

### 6.2 Deadlines

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DEADLINES                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ⚠️ Tang giam:     Con 10 ngay (Han 05/04/2026)                    │
│  ⚠️ Dieu tra:      Con 30 ngay (Han 25/02/2026)                   │
│  ⚠️ Truy to:       None                                           │
│  ⚠️ Xet xu:       None                                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 7. Search & Filter

### 7.1 Dashboard Search

```
┌─────────────────────────────────────────────────────────────────────┐
│  [🔍 Search case, person, document...]   [Filter ▾] [Sort ▾] [Export ▾]  │
└─────────────────────────────────────────────────────────────────────┘
```

### 7.2 Quick Filters

| Filter | Description |
|--------|-------------|
| Active cases | Cases in progress |
| My assigned | Cases assigned to me |
| Near deadline | Expiring within 7 days |
| Has pending review | OCR review pending |
| High priority | Priority cases |

---

## 8. State Management

### 8.1 Dashboard Store

```typescript
interface DashboardState {
  currentCase: Case | null;
  summary: CaseSummary;
  timeline: TimelineEvent[];
  evidence: Evidence[];
  alerts: Alert[];
  loading: boolean;
  
  // Actions
  fetchSummary: (caseId: string) => Promise<Summary>;
  fetchTimeline: (caseId: string) => Promise<void>;
  fetchAlerts: (caseId: string) => Promise<void>;
}
```

---

## 9. Acceptance Criteria

- [ ] Summary dashboard day du
- [ ] Mind map interactive
- [ ] Key facts hien thi
- [ ] Deadlines tracking
- [ ] Alerts indicator
- [ ] Quick actions

---

## 10. Khong Duoc Hieu Sai

- **Dashboard KHONG phai la report** - summary view
- **Mind map KHONG phai la graph DB** - chi display
- **Key facts KHONG phai la profile** - subset

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom E - Nghiep vu*
*HOAN THANH - tat ca 13 files da tao*

---

**TONG KET 13 FILES:**

| # | File | Nhom | Status |
|---|------|-----|--------|
| 1 | 20260424_13_ke_hoach_phan_ra_phase_va_gate_trien_khai.md | A | ✅ |
| 2 | 20260424_14_ban_do_thuoc_tinh_metadata_va_schema_nghiep_vu.md | B | ✅ |
| 3 | 20260424_15_ban_do_giao_dien_tong_the_va_dieu_huong_chinh.md | C | ✅ |
| 4 | 20260424_16_dac_ta_man_hinh_quan_ly_trang_va_viewer.md | C | ✅ |
| 5 | 20260424_17_dac_ta_man_hinh_quan_ly_ho_so_va_khoi_ho_so.md | C | ✅ |
| 6 | 20260424_18_dac_ta_man_hinh_doc_text_citation_va_review.md | C | ✅ |
| 7 | 20260424_19_dac_ta_quan_ly_tai_khoan_nguoi_dung_va_phan_quyen.md | C | ✅ |
| 8 | 20260424_20_ban_do_luong_chuc_nang_end_to_end.md | D | ✅ |
| 9 | 20260424_21_dac_ta_luong_scan_ocr_ai_danh_gia_chat_luong.md | D | ✅ |
| 10 | 20260424_22_dac_ta_tach_file_ingest_db_metadata.md | D | ✅ |
| 11 | 20260424_23_dac_ta_profile_bi_can_luoc_su_va_xet_hoi.md | E | ✅ |
| 12 | 20260424_24_ban_do_lien_ket_giua_file_thong_tin_thuc_the_su_kien.md | E | ✅ |
| 13 | 20260424_25_ban_do_tri_thuc_man_hinh_tom_tat_bi_can.md | E | ✅ |