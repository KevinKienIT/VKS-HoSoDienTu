# AGENTS.md — Bootstrap

> ⚠️ **ĐÂY KHÔNG PHẢI SOURCE OF TRUTH.**
> File này chỉ là bootstrap pointer. Mọi quy tắc chính thức nằm tại:

## → [`v3/01_governance/01_AGENTS.md`](v3/01_governance/01_AGENTS.md)

Đọc file đó TRƯỚC KHI làm bất cứ gì. Nó chứa toàn bộ:
- Kiến trúc hệ thống (Tauri desktop, KHÔNG phải web app)
- Quy chuẩn code (Rust + TypeScript + Python patterns)
- Quy chuẩn UI/UX design
- Quy chuẩn database & storage
- Quy trình làm việc bắt buộc
- Tham chiếu nhanh

## Luồng khởi động agent

```
1. Đọc v3/01_governance/01_AGENTS.md     ← LUẬT TỔNG
2. Đọc v3/NEXT_ACTION.md                 ← PHASE ĐANG ACTIVE
3. Đọc v3/05_logs/YYYYMMDD_*.md          ← LOG NGÀY
4. Nhận task → code → test → update log
```

## Không chạy lệnh nào khác ngoài

```powershell
cd PhanMem && npm run tauri:dev     # Development
cd PhanMem && npm run tauri:build   # Build .exe
```
