# 04 Orchestration Protocol

## Mục tiêu

Chuẩn hóa điều phối: mọi agent đọc cùng một file lệnh tổng trong ngày, tự nhận phần việc được phân công, và xử lý an toàn khi chưa có phân công trực tiếp.

## Quy tắc bắt buộc cho tất cả agent

1. Mỗi phiên làm việc, agent phải đọc file lệnh tổng trong ngày:
   - `v3/05_logs/YYYYMMDD_phase-<phase>.md`
2. Agent phải kiểm tra rõ mục `Assigned agents` để xác định lệnh dành cho mình.
3. Nếu chưa có điều phối trực tiếp cho agent:
   - Tự đánh giá khả năng xử lý.
   - Ghi đề xuất phân loại task vào log.
   - Không tự ý code ngoài luồng phê duyệt.
4. Chỉ thực hiện hành động nằm trong phạm vi đã điều phối hoặc đã được leader ghi bổ sung vào lệnh tổng.

## Nhiệm vụ bắt buộc của Leader

1. Đọc toàn bộ prompt thô từ người dùng.
2. Gom thành một `prompt tổng` duy nhất (không để prompt rời rạc).
3. Diễn giải chi tiết prompt tổng thành các mục:
   - Mục tiêu
   - Phạm vi
   - Ràng buộc
   - Đầu ra mong muốn
   - Agent phụ trách
4. Mỗi lần prompt thay đổi, phải cập nhật lại file lệnh tổng theo thời gian thực.

## Quy tắc ghi log lệnh

1. Tất cả câu lệnh phải ghi file theo thời gian/ngày tháng.
2. File log dùng chuẩn duy nhất:
   - `v3/05_logs/YYYYMMDD_phase-<phase>.md`
3. Mọi bản cập nhật phải có timestamp và người thực hiện.
4. Trên màn hình cho người dùng chỉ hiển thị **lệnh điều phối hiện tại** để giao việc tiếp cho agent.

## Cổng an toàn chống tự ý triển khai

1. Nếu chưa có mục điều phối cho agent trong log ngày: trạng thái `HOLD`.
2. Nếu task vượt năng lực agent: ghi `ESCALATE` về leader.
3. Nếu task mâu thuẫn ràng buộc hệ thống: ghi `BLOCKED` và chờ lệnh mới.
