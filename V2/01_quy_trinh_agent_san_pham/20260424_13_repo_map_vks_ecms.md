# 🗺️ REPO MAP (AIDER-STYLE) - VKS ECMS

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Ban do thu nho giup Agent nhan dien nhanh cac thanh phan cot loi cua du an.

---

## 🏗️ CAU TRUC LÕI (CORE STRUCTURE)

### 1. Luong Dieu Phoi (Orchestration) - `V2/`
- `20260423_00_tong_quan_v2.md`: Entry point chinh.
- `01_quy_trinh_agent_san_pham/`: SOP, Plan, Role Matrix, Aider Workflow.
- `00_prompt_tho/`: Yeu cau goc tu nguoi dung.

### 2. Dac Ta San Pham (Specifications) - `V2/05_tai_lieu_mo_ta/`
- `20260423_01_mo_ta_he_thong_tong_hop.md`: Tai lieu "kinh thanh" ve tinh nang.
- `20260424_06_dac_ta_ai_offline_phan_tich_tai_lieu.md`: Luat cho AI Offline.

### 3. Du Lieu Ho So (Case Data) - `TaiLieu/`
- `15. Le Thanh Cong.../`: Thu muc chua 68 file PDF vu an dang xu ly.

---

## 🛠️ CAC FILE QUAN TRONG (HIGH-PRIORITY FILES)

- `d:/JOBS/VKS-HoSoDienTu/install.ps1`: Script setup moi truong goc.
- `d:/JOBS/VKS-HoSoDienTu/V2/05_tai_lieu_mo_ta/20260423_10_dac_ta_hop_nhat_va_chien_luoc_luu_tru.md`: Chien luoc UI/UX va CSDL.

---

## 🔗 MOI QUAN HE LOGIC (LOGIC MAPPING)

- **Input:** `TaiLieu/*.pdf` -> **Process:** `OCR + AI Classifier` -> **Output:** `SQLite DB + Organization UI`.
- **UI Style:** AntigravityManager (React + Electron).

## 4. Ranh Gioi Thu Muc

- `V2/` la noi chua prompt, SOP, mo ta, dac ta, report, log.
- `PhanMem/` khong chua tai lieu mo ta; chi duoc dung khi bat dau tao code, asset, cau hinh runtime, script build/run va test code.

---
**HD CHO AGENT:** Hay doc file nay truoc khi thuc hien lenh `ls -R` de tiet kiem context.
