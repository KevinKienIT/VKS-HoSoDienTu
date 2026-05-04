"""
VKS ECMS — Test PDF Pipeline từ network share
Test: đọc PDF, trích xuất trang thành PNG, OCR text, hiển thị kết quả.
"""
import sys
import os
import json
import time

# Thêm thư mục python vào path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

def test_pdf_pipeline(pdf_path: str, output_dir: str):
    """Test toàn bộ pipeline: PDF → PNG → OCR → Text"""
    results = {
        "pdf_path": pdf_path,
        "file_exists": False,
        "file_size_kb": 0,
        "page_count": 0,
        "pages_extracted": 0,
        "ocr_results": [],
        "errors": [],
        "timings": {},
    }

    # 1. Kiểm tra file tồn tại
    if not os.path.exists(pdf_path):
        results["errors"].append(f"FILE_NOT_FOUND: {pdf_path}")
        return results

    results["file_exists"] = True
    results["file_size_kb"] = round(os.path.getsize(pdf_path) / 1024, 1)
    print(f"\n{'='*60}")
    print(f"  PDF: {os.path.basename(pdf_path)}")
    print(f"  Size: {results['file_size_kb']} KB")
    print(f"{'='*60}")

    # 2. Mở PDF với PyMuPDF
    t0 = time.time()
    try:
        import fitz  # PyMuPDF
        doc = fitz.open(pdf_path)
        results["page_count"] = len(doc)
        results["timings"]["pdf_open_ms"] = round((time.time() - t0) * 1000, 1)
        print(f"  [OK] PyMuPDF: {len(doc)} trang, mở trong {results['timings']['pdf_open_ms']}ms")
    except Exception as e:
        results["errors"].append(f"PYMUPDF_OPEN_FAILED: {e}")
        print(f"  [FAIL] PyMuPDF: {e}")
        return results

    # 3. Trích xuất PNG cho trang đầu (và trang cuối nếu > 1 trang)
    os.makedirs(output_dir, exist_ok=True)
    pages_to_test = [0]
    if len(doc) > 1:
        pages_to_test.append(len(doc) - 1)

    png_paths = []
    for page_idx in pages_to_test:
        t1 = time.time()
        try:
            page = doc[page_idx]
            # Render 300 DPI cho chất lượng OCR tốt
            mat = fitz.Matrix(300/72, 300/72)
            pix = page.get_pixmap(matrix=mat)
            png_name = f"page_{page_idx+1}.png"
            png_path = os.path.join(output_dir, png_name)
            pix.save(png_path)
            png_size_kb = round(os.path.getsize(png_path) / 1024, 1)
            results["pages_extracted"] += 1
            png_paths.append(png_path)
            ms = round((time.time() - t1) * 1000, 1)
            print(f"  [OK] PNG trang {page_idx+1}: {pix.width}x{pix.height}px, {png_size_kb}KB, {ms}ms")
        except Exception as e:
            results["errors"].append(f"PNG_EXTRACT_FAILED_PAGE_{page_idx}: {e}")
            print(f"  [FAIL] PNG trang {page_idx+1}: {e}")

    # 4. Trích xuất text nhúng (text-based PDF)
    t2 = time.time()
    try:
        embedded_text = ""
        for page in doc:
            embedded_text += page.get_text("text") + "\n"
        embedded_text = embedded_text.strip()
        ms = round((time.time() - t2) * 1000, 1)
        results["timings"]["text_extract_ms"] = ms
        if embedded_text:
            preview = embedded_text[:200].replace("\n", " ↵ ")
            print(f"  [OK] Embedded text: {len(embedded_text)} ký tự, {ms}ms")
            print(f"       Preview: {preview}...")
        else:
            print(f"  [INFO] Không có text nhúng (scanned PDF), cần OCR")
    except Exception as e:
        results["errors"].append(f"TEXT_EXTRACT_FAILED: {e}")

    doc.close()

    # 5. OCR trên PNG đã trích xuất
    for png_path in png_paths:
        page_num = os.path.basename(png_path).replace("page_", "").replace(".png", "")
        t3 = time.time()
        try:
            from ocr.ocr_pipeline import process_image
            from pathlib import Path

            ocr_result = process_image(Path(png_path), lang="vi", mode="auto")
            ms = round((time.time() - t3) * 1000, 1)

            lines = ocr_result.get("lines", [])
            blocks = ocr_result.get("layout", {}).get("blocks", [])
            fields = ocr_result.get("extracted_fields", [])
            full_text = ocr_result.get("text", "")
            engine = ocr_result.get("engine", "unknown")
            avg_conf = ocr_result.get("average_confidence", 0.0)

            page_result = {
                "page": int(page_num),
                "engine": engine,
                "line_count": len(lines),
                "block_count": len(blocks),
                "field_count": len(fields),
                "text_length": len(full_text),
                "avg_confidence": round(avg_conf, 3),
                "ocr_time_ms": ms,
                "text_preview": full_text[:300] if full_text else "",
                "fields": [{"name": f.get("field_name"), "value": f.get("field_value", "")[:80]} for f in fields[:5]],
                "has_normalized_text": any(l.get("normalized_text") for l in lines),
            }
            results["ocr_results"].append(page_result)

            print(f"  [OK] OCR trang {page_num} ({engine}): {len(lines)} lines, "
                  f"conf={avg_conf:.2f}, {ms}ms")
            if full_text:
                preview = full_text[:150].replace("\n", " ↵ ")
                print(f"       OCR text: {preview}...")
            if fields:
                print(f"       Fields: {[f['name'] + '=' + f['value'][:30] for f in fields[:3]]}")
            if page_result["has_normalized_text"]:
                print(f"       [OK] normalized_text có sẵn (tìm kiếm không dấu)")

        except Exception as e:
            results["errors"].append(f"OCR_FAILED_PAGE_{page_num}: {e}")
            print(f"  [FAIL] OCR trang {page_num}: {e}")

    return results


def main():
    NETWORK_DIR = r"\\192.168.100.1\DataShare$\TAI LIEU\test"
    OUTPUT_DIR = r"D:\JOBS\VKS-HoSoDienTu\PhanMem\_test_output"

    # Chọn 3 file test đa dạng: nhỏ, trung bình, lớn
    test_files = [
        os.path.join(NETWORK_DIR, "1.pdf"),   # ~434 KB (nhỏ)
        os.path.join(NETWORK_DIR, "8.pdf"),   # ~1.1 MB (trung bình)
        os.path.join(NETWORK_DIR, "5.pdf"),   # ~10 MB (lớn)
    ]

    all_results = []
    total_start = time.time()

    for pdf_path in test_files:
        fname = os.path.splitext(os.path.basename(pdf_path))[0]
        out_dir = os.path.join(OUTPUT_DIR, fname)
        result = test_pdf_pipeline(pdf_path, out_dir)
        all_results.append(result)

    total_ms = round((time.time() - total_start) * 1000, 1)

    print(f"\n{'='*60}")
    print(f"  TỔNG KẾT")
    print(f"{'='*60}")
    print(f"  Thời gian tổng: {total_ms}ms")
    total_errors = sum(len(r["errors"]) for r in all_results)
    total_pages = sum(r["pages_extracted"] for r in all_results)
    total_ocr = sum(len(r["ocr_results"]) for r in all_results)
    print(f"  Files test: {len(all_results)}")
    print(f"  Trang trích xuất PNG: {total_pages}")
    print(f"  Trang OCR thành công: {total_ocr}")
    print(f"  Lỗi: {total_errors}")

    # Ghi kết quả JSON
    json_path = os.path.join(OUTPUT_DIR, "test_results.json")
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump({"total_time_ms": total_ms, "results": all_results}, f, ensure_ascii=False, indent=2)
    print(f"  JSON results: {json_path}")


if __name__ == "__main__":
    main()
