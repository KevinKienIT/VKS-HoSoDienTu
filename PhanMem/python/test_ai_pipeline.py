import sys
import os
import json
from rapidocr import RapidOCR
from phantich.ai_xuly_text import AITextProcessor

sys.stdout.reconfigure(encoding='utf-8')

def test_pipeline(img_path):
    print(f"=== [1] ĐANG TRÍCH XUẤT OCR LAYOUT: {os.path.basename(img_path)} ===")
    
    engine = RapidOCR()
    result = engine(img_path)
    
    if not result or getattr(result, 'txts', None) is None:
        print("KHÔNG TÌM THẤY TEXT!")
        return

    boxes = getattr(result, 'boxes', [])
    txts = getattr(result, 'txts', [])
    
    # Gom nhóm theo Y (Dòng)
    blocks = []
    for i in range(len(txts)):
        box = boxes[i]
        y_min = min(p[1] for p in box)
        y_max = max(p[1] for p in box)
        blocks.append({'text': txts[i], 'x': min(p[0] for p in box), 'y_center': (y_min + y_max)/2, 'h': y_max - y_min})
        
    blocks.sort(key=lambda b: b['y_center'])
    
    lines = []
    current_line = []
    if blocks:
        current_line = [blocks[0]]
        for b in blocks[1:]:
            if abs(b['y_center'] - current_line[-1]['y_center']) < b['h'] * 0.5:
                current_line.append(b)
            else:
                lines.append(current_line)
                current_line = [b]
        if current_line:
            lines.append(current_line)

    # Tái tạo raw text (giữ space)
    raw_text_lines = []
    for line in lines:
        line.sort(key=lambda b: b['x'])
        line_str = ""
        last_x = 0
        for b in line:
            space_count = min(max(0, int((b['x'] - last_x) / 25)), 30) 
            line_str += (" " * space_count) + b['text']
            last_x = b['x'] + len(b['text']) * 15
        raw_text_lines.append(line_str)
        
    raw_text = "\\n".join(raw_text_lines)
    print("\\n[RAW TEXT TỪ OCR]:")
    print(raw_text)
    
    print(f"\\n=== [2] KIỂM TRA OLLAMA AI AGENT ===")
    processor = AITextProcessor(model_name="qwen2.5:3b")
    
    if not processor.check_health():
        print("❌ CẢNH BÁO: Ollama chưa chạy ở localhost:11434.")
        print("Vui lòng xác nhận cài đặt Ollama trên màn hình (nếu có UAC) và chạy lệnh: 'ollama run qwen2.5:3b'")
        print("Đang chạy MOCK AI fallback để demo pipeline...")
    else:
        print("✅ Ollama đang chạy! Đang gửi text để làm mịn...")
        
    cleaned = processor.clean_text(raw_text)
    print("\\n=== KẾT QUẢ ĐÃ ĐƯỢC AI LÀM MỊN (95%+) ===")
    print(cleaned)

if __name__ == "__main__":
    sys.path.append(os.path.dirname(os.path.abspath(__file__)))
    test_img = "d:/JOBS/VKS-HoSoDienTu/PhanMem/_test_hoso/_output_vks/images/10/page_001.png"
    if os.path.exists(test_img):
        test_pipeline(test_img)
    else:
        print(f"Không tìm thấy ảnh: {test_img}")
