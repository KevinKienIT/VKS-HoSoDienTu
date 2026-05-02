# 02 Runtime Offline Requirements

## Startup Environment Gate

Trước khi vào Dashboard, bắt buộc chạy self-check:
1. Windows 10/11 64-bit
2. RAM >= 4GB
3. CPU >= 2 cores
4. Disk free >= 10GB
5. Workspace read/write
6. SQLite available
7. PDF renderer (pdfium) available
8. OCR runtime available
9. Required OCR models available
10. Manifest/config hợp lệ

Nếu fail điều kiện fatal: chặn app chính, hiện lý do và hướng dẫn khắc phục.

## Runtime profile

- Minimum: RAM 4GB, CPU 2 core, OCR mobile CPU, AI off
- Recommended: RAM 8GB, CPU 4 core, AI Qwen 3B optional
- Advanced: RAM 16GB+, CPU 6+ core, AI Qwen 7B optional

## Full Offline Installer

Người dùng chỉ chạy `VKS_ECMS_Setup.exe`, không yêu cầu `node`, `npm`, `cargo`, `python`, `pip`, internet.

## Manifest bắt buộc

1. `installer_manifest.json`
2. `model_manifest.json`
3. `runtime_manifest.json`

Startup phải verify: presence + version + checksum + compatibility.
