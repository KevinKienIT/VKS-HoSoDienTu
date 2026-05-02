# 05 PHASE PERFORMANCE PROFILES

## Mục tiêu

Hoàn thiện adaptive profile theo 3 lớp: machine/runtime/task.

## Câu lệnh xử lý theo từng việc

1. `P5-01 Install-time profiling`
   - Đo RAM/CPU/GPU/disk và gán profile mặc định ở luồng khởi tạo app [`main.rs`](PhanMem/src-tauri/src/main.rs:1).

2. `P5-02 Runtime monitoring`
   - Theo dõi CPU/RAM/GPU/job queue và tự giảm tải ở scheduler [`scan_cmd.rs`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1).

3. `P5-03 Task-level mode switching`
   - Chọn `Fast/Accurate/Form/Handwriting` theo page chất lượng thực tế trong [`ocr_pipeline.py`](PhanMem/python/ocr/ocr_pipeline.py:1).

4. `P5-04 UI transparency`
   - Hiển thị rõ hành động degrade/profile hiện tại trong UI state store [`uiStore.ts`](PhanMem/src/store/uiStore.ts:1).

