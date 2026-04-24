# 🛠️ TECH STACK VA SETUP MOI TRUONG

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-23
Sua boi agent: Antigravity
Muc dich: Dinh nghia ha tang ky thuat de team AI bat dau code du an ECMS VKS.

---

## 1. Core Stack (Offline-First)

| Thanh phan | Cong nghe | Ly do chon |
| --- | --- | --- |
| **Frontend Framework** | **React.js (Vite)** | Hien dai, component-based, hot-reload nhanh, ho tro tot cho AI team. |
| **UI Library** | **Vanilla CSS + Lucide Icons** | Toi uu hieu nang, khong phu thuoc framework CSS nang ne, giao dien sach se. |
| **Desktop Wrapper** | **Electron** | Chuyen doi Web App thanh file `.exe` chay offline hoan toan tren Windows. |
| **Database** | **SQLite** | CSDL dang file, khong can cai dat server, bao mat va di dong. |
| **PDF Processing** | **PDF.js + pypdf** | Doc va hien thi PDF nang (20MB+) muot ma, ho tro annotation. |
| **AI Engine** | **Ollama (Local)** | Chay model Qwen2.5/Claude (qua OpenClaw) ma khong can Internet. |

---

## 2. Cau Truc Thu Muc Du An (Codebase)

Doi voi team AI, cau truc phai nhat quan de agent de tim file:
```
/src
  /assets          ← Icon, hinh anh
  /components      ← UI Components (Viewer, Sidebar, Timeline)
  /hooks           ← Logic xu ly state
  /services        ← Goi API OpenClaw, truy van SQLite
  /store           ← Quan ly du lieu ho so (Zustand/Context)
  /utils           ← Ham tro giup (OCR parser, date formatter)
/electron          ← Cau hinh main process cho Desktop
/sql               ← Cac file migration cho SQLite
```

---

## 3. Quy Trinh Setup Moi Truong (Cho Agent)

Agent khi nhan task code phai tu kiem tra:
1. `node -v` (Yeu cau v20+).
2. `npm install` de cai dat dependencies theo file `package.json`.
3. `openclaw gateway status` de dam bao bridge AI dang song.

---

## 4. Acceptance Criteria cho Framework

Giao dien phai dat cac tieu chi:
- **Zero-latency:** Chuyen giua cac file PDF khong duoc treo.
- **Persistent State:** Dong app mo lai phai quay dung trang dang doc.
- **Search-driven:** Moi du lieu phai duoc tim thay trong < 1 giay.

---
**TEAM AI STATUS: INFRASTRUCTURE DEFINED. WAITING FOR IMPLEMENTATION START.**
