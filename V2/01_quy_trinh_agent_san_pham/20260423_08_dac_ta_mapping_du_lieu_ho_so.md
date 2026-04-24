# 📑 DAC TA MAPPING DU LIEU HO SO

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-23
Sua boi agent: Antigravity
Muc dich: Quy dinh cach anh xa tu file PDF tho sang du lieu nghiep vu VKS.

---

## 1. Quy Trinh Mapping Tu Dong (Automation Pipeline)

1. **Scan Folder:** Agent quet thu muc `TaiLieu`.
2. **Identity Verification:** Trich xuat ten Bican (VD: Le Thanh Cong) tu ten thu muc.
3. **Deep Scan (OCR):** Quet trang 1 cua tung file PDF de tim tu khoa:
   - "BIEN BAN GHI LOI KHAI" → Type: `LOI_KHAI`
   - "KET LUAN DIEU TRA" → Type: `KET_LUAN`
   - "QUYET DINH KHOI TO" → Type: `KHOI_TO`
4. **But Luc Mapping:** Tim so "But luc" o goc tren ben phai (thuong ghi tay hoac dong dau).
5. **Timeline Extraction:** Tim ngay thang nam ban hanh van ban.

---

## 2. Cau Truc Metadata Schema (SQLite)

```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    file_path TEXT,
    but_luc_so INTEGER,
    document_type TEXT,
    issued_date DATE,
    summary TEXT,
    entities TEXT -- JSON list: ["Le Thanh Cong", "Nguyen Van A"]
);
```

---

## 3. Mapping Logic cho Agent

| Field | Nguon trich xuat | Do tin cay yeu cau |
| --- | --- | --- |
| **But luc so** | OCR vung Top-Right | 100% (Manual check neu mo) |
| **Loai van ban** | Tieu de van ban (in hoa, giua trang) | 95% |
| **Ngay thang** | Cuoi van ban hoac dong dau "Hoi... ngay..." | 90% |

---

## 4. Yeu cau Nâng cap tiep theo

- Xay dung script Python tu dong quet va update file `openclaw.json` khi co file moi duoc them vao.
- Tao giao dien "Review mapping" de KSV xac nhan lai cac thong tin AI da trich xuat.

---
**TEAM AI STATUS: MAPPING STRATEGY DEFINED.**
