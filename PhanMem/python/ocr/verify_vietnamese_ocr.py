"""
Quick verification script for Vietnamese OCR quality (diacritics + layout blocks).

Usage:
  python -m ocr.verify_vietnamese_ocr --json
"""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path


def _build_sample_image(path: Path) -> None:
    from PIL import Image, ImageDraw, ImageFont  # type: ignore

    img = Image.new("RGB", (1800, 1200), color="white")
    draw = ImageDraw.Draw(img)

    # Two separated text blocks to validate paragraph/layout grouping.
    lines = [
        "CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM",
        "Độc lập - Tự do - Hạnh phúc",
        "",
        "Biên bản ghi lời khai của bị can.",
        "Họ và tên: Nguyễn Văn Đặng; nơi cư trú: Thành phố Hồ Chí Minh.",
        "",
        "Điều 1: Bị can trình bày sự việc với đầy đủ chứng cứ và tài liệu.",
    ]

    try:
        font = ImageFont.truetype("arial.ttf", 42)
    except Exception:
        font = ImageFont.load_default()

    y = 90
    for line in lines:
        draw.text((120, y), line, fill="black", font=font)
        y += 70 if line else 110

    img.save(path)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    from ocr.ocr_pipeline import process_image

    with tempfile.TemporaryDirectory(prefix="vks_ocr_verify_") as tmp:
        image_path = Path(tmp) / "sample_vietnamese_layout.png"
        _build_sample_image(image_path)

        result = process_image(image_path, lang="vie", mode="auto")
        text = result.get("ocr_formatted_text") or result.get("ocr_text") or ""
        has_diacritic = any(ch in text for ch in "ăâêôơưđĂÂÊÔƠƯĐáàảãạéèẻẽẹíìỉĩịóòỏõọúùủũụýỳỷỹỵ")
        block_count = len(result.get("blocks") or [])

        report = {
            "engine": result.get("engine"),
            "lang": result.get("lang"),
            "has_diacritic_chars": has_diacritic,
            "block_count": block_count,
            "confidence": result.get("confidence"),
            "formatted_preview": text[:500],
            "pass": bool(has_diacritic and block_count >= 2),
        }

        if args.json:
            print(json.dumps(report, ensure_ascii=False, indent=2))
        else:
            print(report)


if __name__ == "__main__":
    main()

