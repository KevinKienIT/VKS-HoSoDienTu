# 06 PHASE EXPORT MASTER AND RELEASE

## Mục tiêu

Đóng gói đầu ra chuẩn bàn giao + master dossier + release offline installer đủ manifest.

## Câu lệnh xử lý theo từng việc

1. `P6-01 Export studio hoàn chỉnh`
   - Thêm danh sách export jobs + open folder + copy path + cảnh báo `review_pending` ở UI export.

2. `P6-02 Master dossier compiler`
   - Tạo luồng tổng hợp DOCX master từ toàn bộ nguồn tham chiếu tại [`06_reference_samples`](v3/06_reference_samples).

3. `P6-03 Manifest verification`
   - Verify `installer_manifest.json/model_manifest.json/runtime_manifest.json` trong startup self-check.

4. `P6-04 Release gate`
   - Chỉ phát hành khi pass: Startup Gate + regression tauri + export package validation.

