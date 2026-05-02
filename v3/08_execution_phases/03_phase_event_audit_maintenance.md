# 03 PHASE EVENT AUDIT MAINTENANCE

## Mục tiêu

Bổ sung event governance, transaction/audit boundary, maintenance cleanup/index refresh.

## Câu lệnh xử lý theo từng việc

1. `P3-01 Event contract`
   - Chuẩn hóa `event_id/event_type/source/phase/status/payload/created_at` ở layer command [`scan_cmd.rs`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1).

2. `P3-02 Persist before emit`
   - Ghi event vào DB trước khi emit UI trong [`scan_cmd.rs`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1).

3. `P3-03 Audit boundary`
   - Mọi hành động nghiệp vụ phải qua transaction + audit trong [`db/mod.rs`](PhanMem/src-tauri/src/db/mod.rs:1).

4. `P3-04 Maintenance jobs`
   - Thêm cleanup TTL cache/orphan và refresh index định kỳ, neo vào scheduler [`pipeline_execution_tick()`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1402).

