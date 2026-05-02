# 💎 DAC TA HOP NHAT: ANTIGRAVITY MANAGER STYLE & STORAGE STRATEGY

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-23
Sua boi agent: Antigravity
Muc dich: Hop nhat cac yeu cau giao dien va lam ro chien luoc luu tru du lieu quy mo lon de cac agent khac danh gia.

---

## 1. So Sanh & Hop Nhat Giao Dien (UI/UX)

| Dac diem | Phong cach "AntigravityManager" (Chon) | Ly do uu tien |
| --- | --- | --- |
| **Tong quan** | **Premium Dashboard / Command Center** | Tao cam giac quyen luc, tap trung cao do cho KSV. |
| **Layout** | **Dossier-style (Ho so dac vu)** | Trinh bay bi can, vat chung nhu mot tap ho so hinh su chuyen nghiep. |
| **Hieu ung** | **Glassmorphism & Smooth Transitions** | Giam stress khi lam viec lau, tang tinh hien dai. |
| **Tuong tac** | **Context-aware Menus (An hien thong minh)** | Chi hien thi nhung gi can thiet tai thoi diem do, khong gay roi. |
| **Luu tru** | **Zero-click Auto-save** | Luu trang thai ngay lap tuc (Toggle/Switch), khong can nut Save. |

---

## 2. Chien Luoc Luu Tru Quy Mo Lon (Scalable Storage)

De giai quyet bai toan "nhieu ho so, nhieu file" ma van chay muot tren SQLite:

### 2.1 Phan chia luu tru (Hybrid Storage)
- **SQLite (Metadata & Index):**
  - Luu thong tin bi can, ma vu an, loai van ban.
  - Luu text da OCR (Toi uu bang FTS5).
  - Luu link mapping giua cac doi tuong.
- **Filesystem (Binary Data):**
  - PDF goc va Anh HD duoc luu trong thu muc `/data/storage/` co cau truc.
  - Database chi giu `file_path`.
- **Cache (Performance):**
  - Luu anh thumbnail vao thu muc tam de load list nhanh.

### 2.2 Toi uu hoa SQLite cho nganh VKS
- Su dung **WAL mode** (Write-Ahead Logging) de doc/ghi dong thoi khong bi block.
- **Indexing:** Danh index cho cac truong `case_id`, `but_luc_so`, `document_type`.
- **FTS5:** Cho phep tim kiem toan van (Full-text search) tren hang trieu trang giay voi toc do cuc nhanh.

---

## 3. Mo Hinh Mapping Da Doi Tuong (Multi-Object Mapping)

Doi voi cac vu an phuc tap, he thong se map theo kien truc:
- **Vu an <-> Doi tuong:** Mot doi tuong co the co mat trong nhieu vu an.
- **Doi tuong <-> Vat chung:** Lien ket anh vat chung voi tung doi tuong cu the.
- **Bút lục <-> Su kien:** Trich xuat timeline tu noi dung but luc.

---

## 4. Acceptance Criteria (Danh cho Agent Test)

1. **Giao dien:** Phai dat chuan "Premium", khong lag khi zoom PDF.
2. **Du lieu:** SQLite file khong duoc phinh to qua muc (do khong luu BLOB).
3. **Offline:** 100% tinh nang phai hoat dong khi rut day mang.

---
**TEAM AI STATUS: CONSOLIDATED SPECIFICATION PUBLISHED. READY FOR CROSS-AGENT EVALUATION.**

