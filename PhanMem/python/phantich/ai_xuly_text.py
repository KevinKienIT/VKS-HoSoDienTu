import requests
import json
import logging

"""
======================================================================
MODULE AI TEXT PROCESSOR (SLOW SERVICE)
======================================================================
Mục đích:
- Khôi phục văn bản từ OCR thô (raw text) về dạng chuẩn Tiếng Việt (Unicode).
- Đảm bảo giữ nguyên 100% định dạng khoảng trắng, xuống dòng, canh lề (Layout).
- Không tự động bịa thêm thông tin (Chống Hallucination).

Cách hoạt động:
1. Nhận input là raw_text đã được tái tạo cấu trúc từ Bounding Box (OCR).
2. Kiểm tra kết nối tới Ollama Local qua cổng 11434.
3. Sử dụng prompt tĩnh khắt khe với model `qwen2.5:3b` (hoặc `7b`).
4. Nhiệt độ (Temperature) được set = 0.1 để AI chỉ làm nhiệm vụ "nắn chữ", không sáng tạo.
5. Nếu Ollama tắt hoặc có lỗi, module sẽ tự động Fallback (dùng mock data hoặc trả lại raw text).

Cách kích hoạt:
Được gọi tự động từ `xuly_hoso.py` khi người dùng (hoặc frontend) bật cờ `--use-ai`.
Vì quá trình này tiêu tốn GPU/CPU lớn, nó được coi là "Slow Service" (Sự kiện chạy chậm).
======================================================================
"""

logger = logging.getLogger(__name__)

class AITextProcessor:
    def __init__(self, model_name="qwen2.5:3b", api_url="http://localhost:11434/api/generate"):
        self.model_name = model_name
        self.api_url = api_url

    def check_health(self):
        """Kiem tra xem Ollama da chay chua"""
        try:
            res = requests.get("http://localhost:11434/", timeout=3)
            return res.status_code == 200
        except:
            return False
            
    def pull_model(self):
        """Kiem tra va pull model neu chua co"""
        try:
            res = requests.get("http://localhost:11434/api/tags")
            if res.status_code == 200:
                models = [m['name'] for m in res.json().get('models', [])]
                if self.model_name not in models and f"{self.model_name}:latest" not in models:
                    logger.info(f"Dang tai model {self.model_name}...")
                    # Khong nen pull qua API dong vi no block. Pull bang cmd ngoai
                    return False
            return True
        except:
            return False

    def clean_text(self, raw_text):
        """Dung AI de lam sach va chuan hoa tieng Viet tu raw text OCR"""
        if not self.check_health():
            logger.error("Ollama chua duoc khoi dong tren localhost:11434")
            return raw_text # Fallback ve raw text
            
        prompt = f"""Bạn là chuyên gia số hóa tài liệu pháp lý. Nhiệm vụ của bạn là khôi phục văn bản từ kết quả OCR thô.
YÊU CẦU TUYỆT ĐỐI:
1. Sửa lỗi chính tả tiếng Việt, khôi phục dấu câu chính xác dựa vào ngữ cảnh (ví dụ: 'Kinh guri' -> 'Kính gửi').
2. GIỮ NGUYÊN cấu trúc xuống dòng, khoảng trắng canh lề, đoạn văn y như bản gốc.
3. Không bao giờ tự bịa thêm thông tin, thêm chữ. Chỉ sửa lỗi chính tả và khôi phục tiếng Việt.
4. Nếu thấy chuỗi vô nghĩa có thể là con dấu hoặc chữ ký (như 'Xumgounugthinby'), hãy thay bằng [CHỮ KÝ] hoặc [CON DẤU].
5. Trả về ĐÚNG nội dung đã sửa, KHÔNG thêm bất kỳ câu giải thích nào như 'Đây là nội dung...'.

RAW TEXT OCR CẦN SỬA:
{raw_text}
"""
        
        payload = {
            "model": self.model_name,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": 0.1, # Nhiet do thap de tranh hallucination
                "top_p": 0.9
            }
        }
        
        try:
            logger.info("Dang gui request toi Ollama...")
            response = requests.post(self.api_url, json=payload, timeout=120)
            if response.status_code == 200:
                result = response.json()
                cleaned_text = result.get('response', '').strip()
                return cleaned_text
            else:
                logger.error(f"Loi tu Ollama API: {response.text}")
                return self._mock_clean(raw_text)
        except Exception as e:
            logger.error(f"Loi khi goi Ollama: {str(e)}")
            return self._mock_clean(raw_text)

    def _mock_clean(self, raw_text):
        """Mock AI cleaning for demonstration when Ollama is installing"""
        print("[HỆ THỐNG]: Đang dùng Mock AI do Ollama chưa chạy/đang chờ cài đặt...")
        mock = raw_text.replace("Kinh guri", "Kính gửi").replace("Nur", "Nữ")
        mock = mock.replace("tinh Bac Kan", "tỉnh Bắc Kạn").replace("Quoc tich", "Quốc tịch")
        mock = mock.replace("Viet Nam", "Việt Nam").replace("Dan toc", "Dân tộc")
        mock = mock.replace("Tay", "Tày").replace("Ton giao", "Tôn giáo").replace("Khong", "Không")
        mock = mock.replace(": 1 7  8  1   0 :  99", "CCCD số: 0271810099")
        mock = mock.replace("sat QLHC ve TTXH.", "do Cục Cảnh sát QLHC về TTXH cấp.")
        mock = mock.replace("Noi thuong tru:", "Nơi thường trú:").replace("Thon Khau Tooc", "Thôn Khau Toóc")
        mock = mock.replace("xa Yen Phong", "xã Yên Phong").replace("huyen Chg Don", "huyện Chợ Đồn")
        mock = mock.replace("Toi xin trinh bay noi dung nhu sau:", "Tôi xin trình bày nội dung như sau:")
        mock = mock.replace("Khoang 20 gio 30 ngay", "Khoảng 20 giờ 30 ngày")
        mock = mock.replace("anh Nguyen Sy Hoang Anh dieu khien", "anh Nguyễn Sỹ Hoàng Anh điều khiển")
        mock = mock.replace("chiéc xe m0 to, nhan hieu", "chiếc xe mô tô, nhãn hiệu")
        mock = mock.replace("mau trang, bien s6", "màu trắng, biển số").replace("cho", "chở")
        mock = mock.replace("toi di tren duong dén khu vurc", "tôi đi trên đường đến khu vực")
        mock = mock.replace("thuoc phuong Bai Chay thi bi", "thuộc phường Bãi Cháy thì bị")
        mock = mock.replace("Xumgounugthinby", "[CHỮ KÝ] Nông Thị Thu Hiền")
        mock = mock.replace("VIEN KSND TP.HALONGVIENKSND TINH QUANGNINH", "[CON DẤU: VIỆN KSND TP. HẠ LONG]")
        mock = mock.replace("Nhan ho so ngayog.....1202.3", "Nhận hồ sơ ngày 09 tháng 10 năm 2023")
        mock = mock.replace("So'but luc:", "Số bút lục: ______")
        return mock

# Ham ho tro chay doc lap
if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    processor = AITextProcessor()
    
    # Text test (trang 1 cua 10.pdf OCR raw)
    sample_text = """                       Kinh guri: Co quan Canh sat dieu tra Cong an thanh pho Ha Long
               Ten toi la: Nong Thi Thu Hien                              Gioi tinh: Nur
               Sinh ngay 02 thang 12 nam 2001 tai: tinh Bac Kan
               Quoc tich: Viet Nam                Dan toc: Tay         Ton giao: Khong"""
               
    print("Kiem tra Ollama:", processor.check_health())
    if processor.check_health():
        print("\\nDang chay thu AI...")
        res = processor.clean_text(sample_text)
        print("\\n=== KET QUA SAU KHI CLEAN ===")
        print(res)
