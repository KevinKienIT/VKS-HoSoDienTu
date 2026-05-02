import sys
try:
    from paddleocr import PaddleOCR
    print("Downloading PaddleOCR models...")
    # This will download the models to ~/.paddleocr if not already present
    ocr = PaddleOCR(use_angle_cls=True, lang='vie', use_gpu=False)
    print("Models downloaded successfully.")
except Exception as e:
    print(f"Error: {e}")
    sys.exit(1)
