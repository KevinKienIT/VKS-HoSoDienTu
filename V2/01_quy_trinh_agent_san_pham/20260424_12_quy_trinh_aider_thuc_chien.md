# ⚡ QUY TRINH AIDER THUC CHIEN (AIDER-STYLE WORKFLOW)

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Ap dung ky thuat "Repo Map" va "Edit Blocks" cua Aider vao dự án ECMS VKS để đảm bảo độ chính xác tuyệt đối.

---

## 1. Nguyen Tac "Repo Map" (Map duong huong)

Moi Agent khi bắt đầu làm việc PHẢI đọc file này để biết cấu trúc dự án.
- **Diem vao (Entry Point):** `V2/20260423_00_tong_quan_v2.md`
- **Core Logic tai lieu:** `V2/05_tai_lieu_mo_ta/`
- **Output code khi bat dau xay dung:** `PhanMem/` (chi danh cho source code, asset UI, cau hinh runtime, script build/run, test code)

---

## 2. Quy Chuan Chinh Sua Code (Edit Blocks - Chuan Aider)

Agent KHÔNG ĐƯỢC viết lại toàn bộ file. Khi cần sửa code, BẮT BUỘC dùng định dạng SEARCH/REPLACE block chính xác như sau:

1. Block `SEARCH` phải chứa đoạn code **y hệt 100%** so với file gốc (bao gồm cả khoảng trắng, thụt lề).
2. Đoạn code cần thay thế sẽ nằm trong block `REPLACE`.
3. Chỉ sử dụng 1 cặp SEARCH/REPLACE cho mỗi khối thay đổi liên tục.

**Cú pháp bắt buộc:**
```python
<<<<<<< SEARCH
[Đoạn code cũ cần thay đổi, copy y hệt từ file]
=======
[Đoạn code mới hoàn thiện]
>>>>>>> REPLACE
```

**Ví dụ:**
```python
<<<<<<< SEARCH
    def calculate_tax(amount):
        return amount * 0.1
=======
    def calculate_tax(amount):
        # Updated tax rate for 2026
        return amount * 0.15
>>>>>>> REPLACE
```

**Lợi ích:** Tránh làm mất các đoạn code quan trọng khác và giúp áp dụng patch chính xác tuyệt đối như công cụ `diff`.

---

## 3. Quy Tac Git & Commit (Lich su thong minh)

Mỗi khi Agent hoàn thành 1 Task:
1. **Verify:** Chạy lệnh `npm run test` hoặc kiểm tra thủ công.
2. **Commit:** Tự động tạo commit message theo chuẩn:
   - `feat: [Mô tả tính năng mới]`
   - `fix: [Mô tả lỗi đã sửa]`
   - `docs: [Cập nhật tài liệu]`
3. **Log:** Ghi nhận vào `V2/01_quy_trinh_agent_san_pham/20260423_03_nhat_ky_agent_v2.md`.

---

## 4. Cau hinh Aider (`.aider.conf.yml`)

Dự án ưu tiên các thiết lập sau:
- `model: openai-codex` (hoặc `anthropic/claude-3-5-sonnet`)
- `edit-format: diff`
- `auto-commits: true`
- `map-tokens: 1024` (Giới hạn bản đồ repo để tiết kiệm token mà vẫn đủ ý)

---

## 5. Danh sach "Chat Files" (File Agent can doc dau tien)

Trước khi thực hiện bất kỳ lệnh nào, Agent phải "Add" các file sau vào context:
1. `V2/05_tai_lieu_mo_ta/20260423_10_dac_ta_hop_nhat_va_chien_luoc_luu_tru.md`
2. `V2/01_quy_trinh_agent_san_pham/20260423_04_quy_chuan_lam_viec_agent_v2.md`

---
**TEAM AI STATUS: AIDER WORKFLOW INITIALIZED. READY TO CODE WITH PRECISION.**
