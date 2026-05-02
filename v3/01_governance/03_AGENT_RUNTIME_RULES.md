# 03 Agent Runtime Rules

## Mục tiêu

Chuẩn hóa vận hành agent theo luật nghiêm: mọi lệnh người dùng phải được ghi log, mọi phản hồi agent phải lưu vết, và điều phối + phản hồi nằm chung một file theo ngày + phase.

## Quy tắc bắt buộc

1. Mọi lệnh từ người dùng phải ghi vào log hợp nhất.
2. Mọi kết quả của tất cả agent phải ghi vào log hợp nhất.
3. Log điều phối và phản hồi agent để chung một file theo mẫu:
   - `v3/05_logs/YYYYMMDD_phase-<phase>.md`
4. Mọi file `.md` và `.json` cấp gốc phải đưa vào thư mục con trong `v3/` và có tiền tố số thứ tự.
5. Chỉ được xóa plan/phase khi đã hoàn tất đủ 3 bước:
   - code
   - check test
   - check bug

## Điều kiện xóa plan/phase

Một plan/phase chỉ được phép xóa khi trong log hợp nhất có đủ:
1. Biên bản hoàn thành code.
2. Kết quả test có trạng thái PASS.
3. Kết quả check bug có trạng thái PASS hoặc đã đóng toàn bộ bug blocker.

Nếu thiếu một trong ba điều kiện trên thì không được xóa.
