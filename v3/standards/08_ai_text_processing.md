# AI Text Processing Standard (Quy chuẩn Xử lý Văn bản AI)

> **Mục tiêu**: Định nghĩa rõ ràng luồng xử lý văn bản bằng AI (Slow Service) trong hệ thống VKS ECMS để phục hồi tiếng Việt từ kết quả OCR thô, giữ nguyên 100% định dạng.

## 1. Vị trí trong hệ thống
- Code implementation: `PhanMem/python/phantich/ai_xuly_text.py`
- Được tích hợp vào luồng điều phối chính: `PhanMem/python/phantich/xuly_hoso.py`
- Tầng tương tác: Chạy ở Backend (Python) thông qua Tauri Shell Plugin.

## 2. Nguyên tắc hoạt động (Slow Service)
AI Text Processing KHÔNG CHẠY MẶC ĐỊNH cho mọi luồng (do tiêu hao tài nguyên CPU/GPU và thời gian).
Nó chỉ được kích hoạt (Triggered) khi hệ thống phát ra cờ `--use-ai`.

### Khi nào kích hoạt?
- Khi người dùng cần tổng hợp Hồ sơ vụ án (Master Dossier) để báo cáo.
- Khi chất lượng OCR của bản Scan quá mờ, bị rớt dấu tiếng Việt.
- Khi cần trích xuất nguyên vẹn đoạn văn (copy-paste) sang các phần mềm khác.

## 3. Luồng dữ liệu (Data Flow)

```text
[1] PDF/Scan -> [2] BBox Layout OCR -> [3] Event Trigger -> [4] Local AI Agent -> [5] Kết quả DOCX/JSON
```

1. **PDF/Scan**: Nguồn vào là file PDF (thường là scan).
2. **BBox Layout OCR**: Sử dụng RapidOCR để lấy Bounding Box (tọa độ). Dùng tọa độ Y để chia dòng, tọa độ X để tạo khoảng trắng thụt lề nhằm **mô phỏng chính xác cấu trúc văn bản vật lý**.
3. **Event Trigger**: Nếu có cờ `--use-ai`, đẩy raw_text (với đầy đủ khoảng trắng) vào queue của AI.
4. **Local AI Agent**: Gửi prompt tới Ollama (`qwen2.5:3b` hoặc `qwen2.5:7b`) ở `localhost:11434`.
   - **System Prompt**: Bắt buộc AI CHỈ sửa lỗi chính tả tiếng Việt, khôi phục dấu câu.
   - **Nhiệt độ (Temperature)**: Cố định ở mức **0.1** (Low Hallucination).
   - **Định dạng**: Không được làm mất bất kỳ dấu space hay dấu xuống dòng nào.
   - Nếu phát hiện rác OCR ở vùng chữ ký, tự động chuyển đổi thành nhãn `[CHỮ KÝ]`, `[CON DẤU]`.
5. **Kết quả**: Văn bản đã được làm mịn (95%+) được đưa vào parser để render ra file Word (DOCX).

## 4. Cơ chế Fallback
Hệ thống ECMS thiết kế theo kiến trúc **Offline First** và **Resilient**.
- Nếu Ollama chưa được bật.
- Nếu model chưa được tải (`ollama run qwen2.5`).
- Nếu hết timeout 120s.

=> Hệ thống tự động Bypass (bỏ qua) AI, chuyển về trả kết quả OCR thô (Raw Text) hoặc dùng Mock Data trong môi trường Test để không bao giờ làm gián đoạn luồng làm việc của Kiểm sát viên.

## 5. Quy ước dành cho Agent sau
Khi bảo trì hoặc nâng cấp module `ai_xuly_text.py`:
- **KHÔNG SỬ DỤNG API NGOÀI**: Tuyệt đối không tích hợp OpenAI/Gemini/Claude API. Mọi xử lý phải là Local qua Ollama để bảo mật hồ sơ pháp lý.
- **KHÔNG THAY ĐỔI CẤU TRÚC LỀ (SPACING)**: Thuật toán nhận diện khoảng trắng ở `xuly_hoso.py` phải được mapping 1-1 với Prompt của AI. Nếu làm mất cấu trúc, văn bản pháp lý sẽ mất tính pháp lý.
- Luôn phải check hàm `check_health()` trước khi gửi payload lên Local Model.
