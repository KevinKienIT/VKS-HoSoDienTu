"""
VKS ECMS - OCR Pipeline

Offline OCR cho anh scan. Pipeline uu tien:
1. RapidOCR/ONNXRuntime cho van ban danh may, on dinh tren Windows.
2. PaddleOCR neu moi truong Paddle hoat dong.

Chu viet tay:
- mode="handwritten": preprocess anh qua OpenCV (CLAHE, denoising, adaptive threshold)
  truoc khi goi OCR engine. Cac vung low-confidence duoc crop va OCR lai rieng.
- mode="auto": tu dong phat hien neu >30% lines la handwriting_candidate thi
  chay lai pipeline handwritten.
- Neu engine khong co model rieng cho handwriting (RapidOCR/PaddleOCR standard),
  pipeline se canh bao ro "NO_HANDWRITING_MODEL" trong warnings va dat
  needs_review=True. Ket qua chu viet tay LUON can review thu cong.

Usage:
    python -m ocr.ocr_pipeline --image <path> [--lang vi] [--mode auto|printed|handwritten] [--json]
"""

import contextlib
import io
import json
import os
import re
import sys
import tempfile
import unicodedata
from pathlib import Path
from typing import Any, Optional

os.environ.setdefault("PADDLE_PDX_DISABLE_MODEL_SOURCE_CHECK", "True")
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")


def _load_cv() -> tuple[Any, Any]:
    try:
        import cv2  # type: ignore
        import numpy as np  # type: ignore

        return cv2, np
    except Exception as exc:
        raise ImportError("Missing image dependencies. Install opencv-python numpy.") from exc


def _bbox_from_points(points: Any) -> list[int]:
    xs = [float(p[0]) for p in points]
    ys = [float(p[1]) for p in points]
    return [int(min(xs)), int(min(ys)), int(max(xs)), int(max(ys))]


def _line_kind(text: str, confidence: float, mode: str) -> str:
    if mode == "handwritten":
        return "handwritten"
    if mode == "printed":
        return "printed"
    alpha = sum(1 for ch in text if ch.isalpha())
    noisy = len(text.strip()) <= 2 or alpha == 0
    if confidence < 0.72 or noisy:
        return "handwritten"
    return "printed"


def _detect_but_luc(text: str) -> bool:
    patterns = [
        r"B[uú]t\s*l[uụ]c",
        r"BL[:\s]\s*\d+",
        r"^BL\s+\d+$",
        r"S[oố]\s*b[uú]t\s*l[uụ]c",
    ]
    return any(re.search(pattern, text, re.IGNORECASE) for pattern in patterns)


def _normalize_text(value: str) -> str:
    decomposed = unicodedata.normalize("NFD", value)
    stripped = "".join(ch for ch in decomposed if unicodedata.category(ch) != "Mn")
    return stripped.replace("Đ", "D").replace("đ", "d").lower()


def _looks_like_document_number(text: str) -> bool:
    return bool(re.search(r"\bS[oố]\s*[:：]\s*[\w\-/.()]+", text, re.IGNORECASE))


def _looks_like_date(text: str) -> bool:
    return bool(
        re.search(r"ng[aà]y\s+\d{1,2}\s+th[aá]ng\s+\d{1,2}\s+n[aă]m\s+\d{4}", text, re.IGNORECASE)
        or re.search(r"\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b", text)
    )


def _looks_like_agency(text: str) -> bool:
    n = _normalize_text(text)
    keywords = [
        "cong an",
        "co quan canh sat dieu tra",
        "vien kiem sat",
        "toa an",
        "bo cong an",
        "phong canh sat",
    ]
    return any(k in n for k in keywords)


def _looks_like_title(text: str) -> bool:
    n = _normalize_text(text)
    keywords = [
        "quyet dinh",
        "bien ban",
        "ket luan",
        "cao trang",
        "lenh",
        "cong van",
        "giay trieu tap",
        "ban yeu cau",
    ]
    letters = [ch for ch in text if ch.isalpha()]
    upper_ratio = sum(1 for ch in letters if ch.isupper()) / len(letters) if letters else 0
    return any(k in n for k in keywords) and (upper_ratio > 0.45 or len(text) < 80)


def _classify_block(text: str, bbox: list[int], confidence: float, page_height: int | None = None) -> str:
    if confidence < 0.62:
        return "low_confidence"
    if _detect_but_luc(text):
        return "but_luc"
    if _looks_like_document_number(text):
        return "document_number"
    if _looks_like_date(text):
        return "date"
    if _looks_like_agency(text):
        return "agency"
    if _looks_like_title(text):
        return "title"
    if page_height:
        y = bbox[1]
        if y < page_height * 0.16:
            return "header"
        if y > page_height * 0.88:
            return "footer"
    return "text"


def _preprocess_for_handwriting(img_path: Path) -> tuple[Path, list[str]]:
    """
    Preprocess anh de tang chat luong OCR chu viet tay.

    Cac buoc:
    1. Grayscale
    2. CLAHE (Contrast Limited Adaptive Histogram Equalization) — tang tuong phan cuc bo
    3. Gaussian denoising nhe
    4. Adaptive thresholding (Gaussian) — nhi phan hoa thich nghi theo vung
    5. Morphology opening nhe — loai nhieu diem nho
    6. Luu file tam thoi, tra ve Path den file da xu ly

    Tra ve (preprocessed_path, warnings_list).
    warnings_list co the chua "NO_HANDWRITING_MODEL" neu OpenCV khong kha dung.
    """
    warnings: list[str] = []
    try:
        cv2, np = _load_cv()
    except ImportError as exc:
        warnings.append(f"HANDWRITING_PREPROCESS_SKIP: opencv unavailable — {exc}")
        return img_path, warnings

    try:
        img = cv2.imread(str(img_path))
        if img is None:
            warnings.append(f"HANDWRITING_PREPROCESS_SKIP: cannot read image {img_path}")
            return img_path, warnings

        # 1. Grayscale
        gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

        # 2. CLAHE — tang tuong phan cuc bo, tot cho chu viet tay mo nhat
        clahe = cv2.createCLAHE(clipLimit=2.5, tileGridSize=(8, 8))
        enhanced = clahe.apply(gray)

        # 3. Gaussian blur nhe de giam nhieu truoc threshold
        blurred = cv2.GaussianBlur(enhanced, (3, 3), 0)

        # 4. Adaptive threshold Gaussian — phu hop nen khong dong deu (scan)
        binary = cv2.adaptiveThreshold(
            blurred, 255,
            cv2.ADAPTIVE_THRESH_GAUSSIAN_C,
            cv2.THRESH_BINARY,
            blockSize=31,
            C=10,
        )

        # 5. Morphology opening nhe — loai dot nhieu nho hon 1px
        kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (1, 1))
        cleaned = cv2.morphologyEx(binary, cv2.MORPH_OPEN, kernel)

        # Luu file tam thoi cung thu muc voi anh goc, tu dong don dep sau
        suffix = img_path.suffix or ".png"
        tmp_fd, tmp_path_str = tempfile.mkstemp(
            suffix=f"_hw_preproc{suffix}",
            dir=str(img_path.parent),
        )
        os.close(tmp_fd)
        cv2.imwrite(tmp_path_str, cleaned)
        warnings.append(
            "HANDWRITING_PREPROCESS_APPLIED: CLAHE+GaussianBlur+AdaptiveThresh"
            " — standard OCR engine used (no dedicated handwriting model)"
        )
        return Path(tmp_path_str), warnings

    except Exception as exc:
        warnings.append(f"HANDWRITING_PREPROCESS_ERROR: {exc}")
        return img_path, warnings


def _preprocess_region_for_handwriting(img_path: Path, bbox: list[int]) -> tuple[Path, list[str]]:
    """
    Crop va preprocess mot vung (bbox) de OCR lai vung chu viet tay cu the.
    bbox = [x1, y1, x2, y2].
    """
    warnings: list[str] = []
    try:
        cv2, np = _load_cv()
    except ImportError as exc:
        warnings.append(f"REGION_PREPROCESS_SKIP: {exc}")
        return img_path, warnings

    try:
        img = cv2.imread(str(img_path))
        if img is None:
            warnings.append(f"REGION_PREPROCESS_SKIP: cannot read {img_path}")
            return img_path, warnings

        x1, y1, x2, y2 = bbox
        # Them padding de OCR khong bi cat chu dau/cuoi
        pad = 8
        h, w = img.shape[:2]
        x1p = max(0, x1 - pad)
        y1p = max(0, y1 - pad)
        x2p = min(w, x2 + pad)
        y2p = min(h, y2 + pad)
        crop = img[y1p:y2p, x1p:x2p]
        if crop.size == 0:
            warnings.append("REGION_PREPROCESS_SKIP: empty crop")
            return img_path, warnings

        gray = cv2.cvtColor(crop, cv2.COLOR_BGR2GRAY)
        clahe = cv2.createCLAHE(clipLimit=3.0, tileGridSize=(4, 4))
        enhanced = clahe.apply(gray)
        # Upscale 2x de OCR nhan dien ro hon neu crop nho
        if (y2p - y1p) < 80 or (x2p - x1p) < 200:
            enhanced = cv2.resize(enhanced, None, fx=2.0, fy=2.0, interpolation=cv2.INTER_CUBIC)
        blurred = cv2.GaussianBlur(enhanced, (3, 3), 0)
        binary = cv2.adaptiveThreshold(
            blurred, 255,
            cv2.ADAPTIVE_THRESH_GAUSSIAN_C,
            cv2.THRESH_BINARY,
            blockSize=21,
            C=8,
        )
        suffix = img_path.suffix or ".png"
        tmp_fd, tmp_path_str = tempfile.mkstemp(
            suffix=f"_hw_region{suffix}",
            dir=str(img_path.parent),
        )
        os.close(tmp_fd)
        cv2.imwrite(tmp_path_str, binary)
        return Path(tmp_path_str), warnings

    except Exception as exc:
        warnings.append(f"REGION_PREPROCESS_ERROR: {exc}")
        return img_path, warnings


def _detect_stamp_regions(img_path: Path) -> list[dict]:
    regions: list[dict] = []
    try:
        cv2, np = _load_cv()
        img = cv2.imread(str(img_path))
        if img is None:
            return regions

        hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
        lower_red1 = np.array([0, 50, 50])
        upper_red1 = np.array([10, 255, 255])
        lower_red2 = np.array([160, 50, 50])
        upper_red2 = np.array([180, 255, 255])
        mask = cv2.bitwise_or(
            cv2.inRange(hsv, lower_red1, upper_red1),
            cv2.inRange(hsv, lower_red2, upper_red2),
        )
        contours, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        for cnt in contours:
            area = cv2.contourArea(cnt)
            if area < 1000:
                continue
            x, y, w, h = cv2.boundingRect(cnt)
            ratio = w / h if h > 0 else 0
            if 0.5 < ratio < 2.0:
                regions.append(
                    {
                        "type": "stamp",
                        "bbox": [int(x), int(y), int(x + w), int(y + h)],
                        "label": "Potential red stamp",
                        "confidence": 0.7,
                    }
                )
    except Exception as exc:
        regions.append({"type": "warning", "label": f"stamp_detection_failed: {exc}"})
    return regions


def _detect_signature_regions(img_path: Path, ocr_lines: list[dict]) -> list[dict]:
    """Heuristic signature detector. It marks likely ink-stroke regions, not signer identity."""
    regions: list[dict] = []
    try:
        cv2, np = _load_cv()
        img = cv2.imread(str(img_path))
        if img is None:
            return regions
        height, width = img.shape[:2]
        gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
        # Dark blue/black strokes, usually in lower/right half near signer labels.
        _, mask = cv2.threshold(gray, 115, 255, cv2.THRESH_BINARY_INV)
        kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (3, 3))
        mask = cv2.morphologyEx(mask, cv2.MORPH_OPEN, kernel)
        contours, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)

        signer_hint_boxes: list[list[int]] = []
        for line in ocr_lines:
            n = _normalize_text(str(line.get("text", "")))
            if any(k in n for k in ["thu truong", "nguoi ky", "kiem sat vien", "dieu tra vien", "pho thu truong"]):
                signer_hint_boxes.append(line.get("bbox", [0, 0, 0, 0]))

        for cnt in contours:
            x, y, w, h = cv2.boundingRect(cnt)
            area = cv2.contourArea(cnt)
            if area < 120 or area > width * height * 0.05:
                continue
            if y < height * 0.28:
                continue
            aspect = w / h if h else 0
            if not (1.4 <= aspect <= 12.0 and 8 <= h <= height * 0.16):
                continue
            near_signer_text = any(abs(y - box[3]) < height * 0.16 or y > box[3] for box in signer_hint_boxes)
            if near_signer_text or y > height * 0.55:
                regions.append(
                    {
                        "type": "signature",
                        "bbox": [int(x), int(y), int(x + w), int(y + h)],
                        "label": "Potential signature strokes",
                        "confidence": 0.62 if near_signer_text else 0.48,
                    }
                )
        # Keep largest candidates only to avoid noisy character contours.
        regions.sort(key=lambda item: (item["bbox"][2] - item["bbox"][0]) * (item["bbox"][3] - item["bbox"][1]), reverse=True)
        return regions[:4]
    except Exception as exc:
        return [{"type": "warning", "label": f"signature_detection_failed: {exc}"}]


def _sort_lines(lines: list[dict]) -> list[dict]:
    return sorted(lines, key=lambda ln: (int(ln.get("bbox", [0, 0, 0, 0])[1]), int(ln.get("bbox", [0, 0, 0, 0])[0])))


def _build_layout(lines: list[dict], page_width: int | None = None, page_height: int | None = None) -> dict:
    """Group OCR lines into page blocks to preserve layout-like reading order."""
    if not lines:
        return {"blocks": [], "formatted_text": ""}

    sorted_lines = _sort_lines(lines)
    heights = [max(1, int(ln["bbox"][3]) - int(ln["bbox"][1])) for ln in sorted_lines if ln.get("bbox")]
    median_h = sorted(heights)[len(heights) // 2] if heights else 20
    y_gap_threshold = max(10, int(median_h * 0.8))

    blocks: list[dict] = []
    current_block: dict[str, Any] | None = None
    block_idx = 0

    for ln in sorted_lines:
        bbox = ln.get("bbox", [0, 0, 0, 0])
        y_top = int(bbox[1])
        if current_block is None:
            current_block = {
                "block_index": block_idx,
                "bbox": [int(bbox[0]), int(bbox[1]), int(bbox[2]), int(bbox[3])],
                "lines": [],
            }
            block_idx += 1
        else:
            prev_bbox = current_block["lines"][-1].get("bbox", [0, 0, 0, 0]) if current_block["lines"] else current_block["bbox"]
            prev_bottom = int(prev_bbox[3])
            if y_top - prev_bottom > y_gap_threshold:
                blocks.append(current_block)
                current_block = {
                    "block_index": block_idx,
                    "bbox": [int(bbox[0]), int(bbox[1]), int(bbox[2]), int(bbox[3])],
                    "lines": [],
                }
                block_idx += 1

        current_block["bbox"][0] = min(current_block["bbox"][0], int(bbox[0]))
        current_block["bbox"][1] = min(current_block["bbox"][1], int(bbox[1]))
        current_block["bbox"][2] = max(current_block["bbox"][2], int(bbox[2]))
        current_block["bbox"][3] = max(current_block["bbox"][3], int(bbox[3]))
        reading_order = len(current_block["lines"])
        current_block["lines"].append(
            {
                "text": ln.get("text", ""),
                "bbox": bbox,
                "confidence": float(ln.get("confidence", 0.0)),
                "type": ln.get("type", "printed"),
                "reading_order": reading_order,
            }
        )

    if current_block is not None:
        blocks.append(current_block)

    block_texts = []
    for reading_order, block in enumerate(blocks):
        block_lines = sorted(block["lines"], key=lambda l: (int(l["bbox"][1]), int(l["bbox"][0])))
        block_text = "\n".join([str(ln.get("text", "")).strip() for ln in block_lines if str(ln.get("text", "")).strip()])
        block["text"] = block_text
        confidences = [float(ln.get("confidence", 0.0)) for ln in block_lines]
        avg_conf = sum(confidences) / len(confidences) if confidences else 0.0
        block["confidence"] = avg_conf
        block["block_type"] = _classify_block(block_text, block["bbox"], avg_conf, page_height)
        block["reading_order"] = reading_order
        if page_width:
            block["page_width"] = page_width
        if page_height:
            block["page_height"] = page_height
        block_texts.append(block_text)

    formatted_text = "\n\n".join([t for t in block_texts if t.strip()])
    return {"blocks": blocks, "formatted_text": formatted_text}


def _extract_fields(lines: list[dict], blocks: list[dict], regions: list[dict]) -> list[dict]:
    fields: list[dict] = []

    def add(field_name: str, field_value: str, bbox: list[int], confidence: float, source: str) -> None:
        value = field_value.strip()
        if not value:
            return
        fields.append(
            {
                "field_name": field_name,
                "field_value": value,
                "bbox": bbox,
                "confidence": float(confidence),
                "source": source,
            }
        )

    for block in blocks:
        text = str(block.get("text", ""))
        bbox = block.get("bbox", [0, 0, 0, 0])
        conf = float(block.get("confidence", 0.0))
        block_type = block.get("block_type")
        if block_type in {"title", "agency", "document_number", "date", "but_luc"}:
            add(block_type, text.replace("\n", " "), bbox, conf, "ocr_rule_layout")

        number_match = re.search(r"\bS[oố]\s*[:：]\s*([^\n\r]+)", text, re.IGNORECASE)
        if number_match:
            add("số văn bản", number_match.group(1), bbox, min(0.96, conf + 0.03), "ocr_regex")

        date_match = re.search(r"(ng[aà]y\s+\d{1,2}\s+th[aá]ng\s+\d{1,2}\s+n[aă]m\s+\d{4})", text, re.IGNORECASE)
        if not date_match:
            date_match = re.search(r"(\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b)", text)
        if date_match:
            add("ngày ban hành", date_match.group(1), bbox, min(0.95, conf + 0.02), "ocr_regex")

        if _detect_but_luc(text):
            bl_num = re.search(r"(?:BL|B\.L|B[uú]t\s*l[uụ]c)\D{0,8}(\d+)", text, re.IGNORECASE)
            add("số bút lục", bl_num.group(1) if bl_num else text, bbox, conf, "ocr_rule")

    title_blocks = [b for b in blocks if b.get("block_type") == "title"]
    if title_blocks:
        best = max(title_blocks, key=lambda b: float(b.get("confidence", 0.0)))
        add("loại văn bản", str(best.get("text", "")).replace("\n", " "), best.get("bbox", [0, 0, 0, 0]), float(best.get("confidence", 0.0)), "ocr_rule_title")

    agency_blocks = [b for b in blocks if b.get("block_type") == "agency"]
    if agency_blocks:
        text = " / ".join(str(b.get("text", "")).replace("\n", " ") for b in agency_blocks[:3])
        add("cơ quan ban hành", text, agency_blocks[0].get("bbox", [0, 0, 0, 0]), float(agency_blocks[0].get("confidence", 0.0)), "ocr_rule_agency")

    for region in regions:
        if region.get("type") == "stamp":
            add("dấu đỏ", region.get("label", "Có dấu đỏ"), region.get("bbox", [0, 0, 0, 0]), float(region.get("confidence", 0.0)), "opencv_red_mask")
        elif region.get("type") == "signature":
            add("chữ ký", region.get("label", "Có vùng chữ ký"), region.get("bbox", [0, 0, 0, 0]), float(region.get("confidence", 0.0)), "opencv_signature_heuristic")

    # De-duplicate by field/value while keeping first location.
    seen: set[tuple[str, str]] = set()
    unique: list[dict] = []
    for field in fields:
        key = (field["field_name"], field["field_value"].lower())
        if key in seen:
            continue
        seen.add(key)
        unique.append(field)
    return unique


def _parse_rapid_output(result: Any, mode: str) -> tuple[list[dict], list[str]]:
    lines: list[dict] = []
    warnings: list[str] = []
    boxes = getattr(result, "boxes", None)
    texts = getattr(result, "txts", None)
    scores = getattr(result, "scores", None)
    if boxes is None or texts is None or scores is None:
        return lines, ["rapidocr_empty_output"]

    for box, text, score in zip(boxes, texts, scores):
        text = str(text).strip()
        confidence = float(score or 0.0)
        if not text:
            continue
        bbox = _bbox_from_points(box)
        item = {
            "text": text,
            "normalized_text": _normalize_text(text),
            "unicode_form": "NFC",
            "bbox": bbox,
            "confidence": confidence,
            "type": _line_kind(text, confidence, mode),
        }
        if _detect_but_luc(text):
            item["is_but_luc"] = True
        lines.append(item)
    return lines, warnings


def _run_rapidocr(image_path: Path, mode: str) -> dict:
    try:
        from rapidocr import RapidOCR  # type: ignore
    except Exception as exc:
        raise ImportError("rapidocr_missing: install rapidocr onnxruntime") from exc

    stderr_buffer = io.StringIO()
    stdout_buffer = io.StringIO()
    with contextlib.redirect_stderr(stderr_buffer), contextlib.redirect_stdout(stdout_buffer):
        # Prefer Vietnamese model if available via lang='vie'; fallback to default constructor.
        try:
            engine = RapidOCR(lang="vie")
        except TypeError:
            engine = RapidOCR()
        result = engine(str(image_path))

    lines, warnings = _parse_rapid_output(result, mode)
    logs = "\n".join([stdout_buffer.getvalue().strip(), stderr_buffer.getvalue().strip()]).strip()
    if logs:
        warnings.append(logs[-1200:])
    return {"engine": "rapidocr_onnxruntime", "lines": lines, "warnings": warnings}


def _parse_paddle_v2(result: Any, mode: str) -> list[dict]:
    lines: list[dict] = []
    if not result:
        return lines
    rows = result[0] if isinstance(result, list) and result and isinstance(result[0], list) else result
    for row in rows:
        if not row or len(row) < 2:
            continue
        box = row[0]
        rec = row[1]
        if not rec or len(rec) < 2:
            continue
        text = str(rec[0]).strip()
        confidence = float(rec[1] or 0.0)
        if not text:
            continue
        item = {
            "text": text,
            "normalized_text": _normalize_text(text),
            "unicode_form": "NFC",
            "bbox": _bbox_from_points(box),
            "confidence": confidence,
            "type": _line_kind(text, confidence, mode),
        }
        if _detect_but_luc(text):
            item["is_but_luc"] = True
        lines.append(item)
    return lines


def _parse_paddle_v3(result: Any, mode: str) -> list[dict]:
    lines: list[dict] = []
    items = result if isinstance(result, list) else [result]
    for item in items:
        data = item
        if hasattr(item, "json"):
            with contextlib.suppress(Exception):
                data = item.json
        if hasattr(item, "to_json"):
            with contextlib.suppress(Exception):
                data = item.to_json()
        if isinstance(data, str):
            with contextlib.suppress(Exception):
                data = json.loads(data)
        if not isinstance(data, dict):
            continue
        res = data.get("res", data)
        texts = res.get("rec_texts") or res.get("texts") or []
        scores = res.get("rec_scores") or res.get("scores") or []
        boxes = res.get("rec_boxes") or res.get("dt_polys") or res.get("boxes") or []
        for idx, text in enumerate(texts):
            text = str(text).strip()
            if not text:
                continue
            confidence = float(scores[idx]) if idx < len(scores) else 0.0
            raw_box = boxes[idx] if idx < len(boxes) else [[0, 0], [0, 0]]
            if raw_box and isinstance(raw_box[0], (int, float)) and len(raw_box) >= 4:
                x1, y1, x2, y2 = raw_box[:4]
                bbox = [int(x1), int(y1), int(x2), int(y2)]
            else:
                bbox = _bbox_from_points(raw_box)
            line = {
                "text": text,
                "bbox": bbox,
                "confidence": confidence,
                "type": _line_kind(text, confidence, mode),
            }
            if _detect_but_luc(text):
                line["is_but_luc"] = True
            lines.append(line)
    return lines


def _run_paddleocr(image_path: Path, lang: str, mode: str) -> dict:
    try:
        from paddleocr import PaddleOCR  # type: ignore
    except Exception as exc:
        raise ImportError("paddleocr_missing: install paddleocr paddlepaddle") from exc

    stderr_buffer = io.StringIO()
    stdout_buffer = io.StringIO()
    with contextlib.redirect_stderr(stderr_buffer), contextlib.redirect_stdout(stdout_buffer):
        try:
            engine = PaddleOCR(
                lang=lang,
                use_doc_orientation_classify=False,
                use_doc_unwarping=False,
                use_textline_orientation=True,
            )
            result = engine.predict(str(image_path)) if hasattr(engine, "predict") else engine.ocr(str(image_path), cls=True)
            lines = _parse_paddle_v3(result, mode)
        except TypeError:
            engine = PaddleOCR(use_angle_cls=True, lang=lang)
            result = engine.ocr(str(image_path), cls=True)
            lines = _parse_paddle_v2(result, mode)

    warnings: list[str] = []
    logs = "\n".join([stdout_buffer.getvalue().strip(), stderr_buffer.getvalue().strip()]).strip()
    if logs:
        warnings.append(logs[-1200:])
    return {"engine": "paddleocr", "lines": lines, "warnings": warnings}


def _run_rapidocr_handwritten(image_path: Path, mode: str) -> dict:
    """
    Chay RapidOCR tren anh da preprocess cho handwriting.
    RapidOCR khong co model handwriting rieng — pipeline se preprocess truoc,
    chay OCR, va them warning "NO_HANDWRITING_MODEL".
    """
    preproc_path, preproc_warnings = _preprocess_for_handwriting(image_path)
    tmp_created = preproc_path != image_path
    try:
        result = _run_rapidocr(preproc_path, mode)
    finally:
        if tmp_created and preproc_path.exists():
            try:
                preproc_path.unlink()
            except OSError:
                pass
    result["warnings"] = preproc_warnings + list(result.get("warnings") or [])
    result["warnings"].append(
        "NO_HANDWRITING_MODEL: RapidOCR/ONNX khong co model handwriting "
        "chuyen biet. Ket qua duoc cai thien bang OpenCV preprocess "
        "(CLAHE+AdaptiveThresh) nhung van can review thu cong."
    )
    result["engine"] = "rapidocr_onnxruntime+hw_preproc"
    return result


def _run_paddleocr_handwritten(image_path: Path, lang: str, mode: str) -> dict:
    """
    Chay PaddleOCR tren anh da preprocess cho handwriting.
    PaddleOCR standard khong co model handwriting rieng — pipeline se preprocess
    truoc, chay OCR voi use_textline_orientation=True, them warning.
    """
    preproc_path, preproc_warnings = _preprocess_for_handwriting(image_path)
    tmp_created = preproc_path != image_path
    try:
        result = _run_paddleocr(preproc_path, lang, mode)
    finally:
        if tmp_created and preproc_path.exists():
            try:
                preproc_path.unlink()
            except OSError:
                pass
    result["warnings"] = preproc_warnings + list(result.get("warnings") or [])
    result["warnings"].append(
        "NO_HANDWRITING_MODEL: PaddleOCR khong co model handwriting "
        "chuyen biet. Ket qua duoc cai thien bang OpenCV preprocess "
        "(CLAHE+AdaptiveThresh) nhung van can review thu cong."
    )
    result["engine"] = "paddleocr+hw_preproc"
    return result


def _reocr_handwriting_regions(
    image_path: Path,
    handwriting_candidates: list[dict],
    lang: str,
    mode: str,
) -> list[dict]:
    """
    OCR lai tung vung handwriting_candidate bang cach crop + preprocess rieng.
    Tra ve list region voi text/confidence duoc cap nhat.
    Chi chay neu co it nhat 1 candidate va OpenCV kha dung.
    """
    if not handwriting_candidates:
        return handwriting_candidates

    updated: list[dict] = []
    for candidate in handwriting_candidates:
        bbox = candidate.get("bbox", [0, 0, 0, 0])
        # Bo qua bbox rong/khong hop le
        if bbox[2] <= bbox[0] or bbox[3] <= bbox[1]:
            updated.append(candidate)
            continue

        region_path, region_warnings = _preprocess_region_for_handwriting(image_path, bbox)
        tmp_created = region_path != image_path
        region_text = candidate.get("text", "")
        region_conf = candidate.get("confidence", 0.0)
        try:
            # Thu RapidOCR truoc
            try:
                r = _run_rapidocr(region_path, mode)
                if r["lines"]:
                    best = max(r["lines"], key=lambda l: float(l.get("confidence", 0.0)))
                    region_text = best["text"]
                    region_conf = float(best.get("confidence", 0.0))
            except Exception:
                # Fallback PaddleOCR
                try:
                    r2 = _run_paddleocr(region_path, lang, mode)
                    if r2["lines"]:
                        best2 = max(r2["lines"], key=lambda l: float(l.get("confidence", 0.0)))
                        region_text = best2["text"]
                        region_conf = float(best2.get("confidence", 0.0))
                except Exception:
                    pass
        finally:
            if tmp_created and region_path.exists():
                try:
                    region_path.unlink()
                except OSError:
                    pass

        updated_candidate = dict(candidate)
        updated_candidate["text"] = region_text
        updated_candidate["confidence"] = region_conf
        updated_candidate["reocr_applied"] = True
        if region_warnings:
            updated_candidate["preproc_notes"] = region_warnings
        updated.append(updated_candidate)

    return updated


def process_image(image_path: Path, lang: str = "vi", mode: str = "auto") -> dict:
    image_path = image_path.resolve()
    if not image_path.exists():
        raise FileNotFoundError(f"Image not found: {image_path}")

    attempts: list[str] = []
    result: Optional[dict] = None
    pipeline_warnings: list[str] = []

    # ------------------------------------------------------------------ #
    # HANDWRITTEN MODE: preprocess toan anh truoc, dung runner rieng      #
    # ------------------------------------------------------------------ #
    if mode == "handwritten":
        for hw_runner, extra_args in [
            (_run_rapidocr_handwritten, (mode,)),
            (_run_paddleocr_handwritten, (lang, mode)),
        ]:
            try:
                r_args: tuple = (image_path,) + extra_args
                result = hw_runner(*r_args)  # type: ignore[operator]
                if result["lines"]:
                    break
                attempts.append(f"{result['engine']}: no_text")
            except Exception as exc:
                attempts.append(f"{hw_runner.__name__}: {exc}")

    # ------------------------------------------------------------------ #
    # AUTO / PRINTED: dung runner tieu chuan                              #
    # ------------------------------------------------------------------ #
    if result is None or not result["lines"]:
        for runner in (_run_rapidocr,):
            try:
                result = runner(image_path, mode)
                if result["lines"]:
                    break
                attempts.append(f"{result['engine']}: no_text")
            except Exception as exc:
                attempts.append(f"{runner.__name__}: {exc}")

    if result is None or not result["lines"]:
        try:
            result = _run_paddleocr(image_path, lang, mode)
            if not result["lines"]:
                attempts.append("paddleocr: no_text")
        except Exception as exc:
            attempts.append(f"paddleocr: {exc}")

    if result is None or not result["lines"]:
        raise RuntimeError("OCR_ENGINE_FAILED: " + " | ".join(attempts))

    cv2_mod, _ = _load_cv()
    img = cv2_mod.imread(str(image_path))
    page_height = int(img.shape[0]) if img is not None else None
    page_width = int(img.shape[1]) if img is not None else None

    lines = _sort_lines(result["lines"])
    layout = _build_layout(lines, page_width=page_width, page_height=page_height)
    text_parts = [line["text"] for line in lines]
    confidence_values = [float(line.get("confidence", 0.0)) for line in lines]
    avg_conf = sum(confidence_values) / len(confidence_values) if confidence_values else 0.0
    regions = _detect_stamp_regions(image_path)
    regions.extend(_detect_signature_regions(image_path, lines))
    handwriting_regions: list[dict] = []
    for line in lines:
        if line.get("is_but_luc"):
            regions.append(
                {
                    "type": "but_luc",
                    "bbox": line["bbox"],
                    "text": line["text"],
                    "confidence": line["confidence"],
                }
            )
        if line.get("type") == "handwritten":
            handwriting_regions.append(
                {
                    "type": "handwriting_candidate",
                    "bbox": line["bbox"],
                    "text": line["text"],
                    "confidence": line["confidence"],
                }
            )

    # ------------------------------------------------------------------ #
    # AUTO-DETECT: trigger handwritten pipeline khi:                      #
    #   (a) >30% lines la handwriting_candidate, HOAC                     #
    #   (b) avg_conf < 0.60 (OCR in thay van ban kho/mo), HOAC           #
    #   (c) OCR tra ve empty lines (thuong gap khi chi co dau/chu viet tay#
    # ------------------------------------------------------------------ #
    _should_retry_hw = False
    _hw_retry_reason = ""
    if mode == "auto":
        if not lines:
            _should_retry_hw = True
            _hw_retry_reason = "AUTO_HANDWRITING_DETECTED: OCR returned empty lines — likely pure handwritten/stamp image."
        elif avg_conf < 0.60:
            _should_retry_hw = True
            _hw_retry_reason = (
                f"AUTO_HANDWRITING_DETECTED: avg_conf={avg_conf:.2f} < 0.60 — "
                "low confidence suggests handwritten or degraded scan."
            )
        else:
            hw_ratio = len(handwriting_regions) / len(lines)
            if hw_ratio > 0.30:
                _should_retry_hw = True
                _hw_retry_reason = (
                    f"AUTO_HANDWRITING_DETECTED: {hw_ratio:.0%} lines are handwriting_candidate. "
                    "Re-running with handwriting preprocess."
                )

    if _should_retry_hw:
        pipeline_warnings.append(_hw_retry_reason)
        try:
            hw_result: Optional[dict] = None
            try:
                hw_result = _run_rapidocr_handwritten(image_path, "handwritten")
                if not hw_result["lines"]:
                    hw_result = None
            except Exception as exc:
                pipeline_warnings.append(f"hw_rerun_rapidocr: {exc}")
            if hw_result is None:
                try:
                    hw_result = _run_paddleocr_handwritten(image_path, lang, "handwritten")
                except Exception as exc:
                    pipeline_warnings.append(f"hw_rerun_paddle: {exc}")

            if hw_result and hw_result["lines"]:
                result = hw_result
                lines = _sort_lines(result["lines"])
                layout = _build_layout(lines, page_width=page_width, page_height=page_height)
                text_parts = [line["text"] for line in lines]
                confidence_values = [float(line.get("confidence", 0.0)) for line in lines]
                avg_conf = sum(confidence_values) / len(confidence_values) if confidence_values else 0.0
                # Rebuild handwriting_regions tu result moi
                handwriting_regions = [
                    {
                        "type": "handwriting_candidate",
                        "bbox": ln["bbox"],
                        "text": ln["text"],
                        "confidence": ln["confidence"],
                    }
                    for ln in lines if ln.get("type") == "handwritten"
                ]
        except Exception as exc:
            pipeline_warnings.append(f"AUTO_HANDWRITING_RERUN_ERROR: {exc}")

    # ------------------------------------------------------------------ #
    # Re-OCR tung vung handwriting rieng neu co (handwritten/auto)        #
    # ------------------------------------------------------------------ #
    if handwriting_regions and mode in {"handwritten", "auto"}:
        handwriting_regions = _reocr_handwriting_regions(
            image_path, handwriting_regions, lang, mode
        )

    # ------------------------------------------------------------------ #
    # Re-OCR stamp regions neu OCR lines khong cover vung dau do           #
    # (so but luc viet tay trong dau thuong bi bo qua boi printed OCR)    #
    # ------------------------------------------------------------------ #
    stamp_regions = [r for r in regions if r.get("type") == "stamp"]
    if stamp_regions and mode in {"handwritten", "auto"}:
        for stamp in stamp_regions:
            stamp_bbox = stamp.get("bbox", [0, 0, 0, 0])
            # Kiem tra xem co OCR line nao nam trong vung dau khong
            lines_in_stamp = [
                ln for ln in lines
                if (
                    ln.get("bbox") and
                    int(ln["bbox"][0]) >= stamp_bbox[0] - 10 and
                    int(ln["bbox"][1]) >= stamp_bbox[1] - 10 and
                    int(ln["bbox"][2]) <= stamp_bbox[2] + 10 and
                    int(ln["bbox"][3]) <= stamp_bbox[3] + 10
                )
            ]
            # Neu khong co line nao hoac confidence thap, re-OCR vung dau
            stamp_has_good_ocr = any(
                float(ln.get("confidence", 0.0)) >= 0.55 for ln in lines_in_stamp
            )
            if not stamp_has_good_ocr:
                pipeline_warnings.append(
                    f"STAMP_REOCR: No reliable OCR text found in stamp region {stamp_bbox}. "
                    "Attempting handwritten re-OCR on stamp crop."
                )
                stamp_candidate = {
                    "type": "handwriting_candidate",
                    "bbox": stamp_bbox,
                    "text": "",
                    "confidence": 0.0,
                    "source": "stamp_reocr",
                }
                reocred = _reocr_handwriting_regions(
                    image_path, [stamp_candidate], lang, "handwritten"
                )
                if reocred and reocred[0].get("text"):
                    stamp["ocr_text"] = reocred[0]["text"]
                    stamp["ocr_confidence"] = reocred[0].get("confidence", 0.0)
                    stamp["label"] = f"Stamp text: {reocred[0]['text']}"
                    handwriting_regions.extend(reocred)

    warnings = list(result.get("warnings") or [])
    warnings.extend(attempts)
    warnings.extend(pipeline_warnings)

    has_handwriting = bool(handwriting_regions)
    extracted_fields = _extract_fields(lines, layout["blocks"], regions)

    # Canh bao ro: khong co model handwriting chuyen biet
    if has_handwriting and not any("NO_HANDWRITING_MODEL" in w for w in warnings):
        warnings.append(
            "NO_HANDWRITING_MODEL: Khong co engine/model OCR chuyen biet cho "
            "chu viet tay. Ket qua handwriting_regions can duoc review thu cong."
        )

    return {
        "image_path": str(image_path),
        "engine": result["engine"],
        "lang": lang,
        "mode": mode,
        "page_width": page_width,
        "page_height": page_height,
        "ocr_text": "\n".join(text_parts),
        "ocr_formatted_text": layout["formatted_text"],
        "confidence": float(avg_conf),
        "lines": lines,
        "blocks": layout["blocks"],
        "regions": regions,
        "has_handwriting": has_handwriting,
        "handwriting_regions": handwriting_regions,
        "extracted_fields": extracted_fields,
        "warnings": [w for w in warnings if w],
        "needs_review": has_handwriting or avg_conf < 0.72,
    }


def main(args: Optional[list[str]] = None) -> None:
    if args is None:
        args = sys.argv[1:]
    if not args or "--help" in args:
        print(__doc__)
        sys.exit(0)

    image_path: Optional[Path] = None
    lang = "vie"
    mode = "auto"
    output_json = False

    i = 0
    while i < len(args):
        if args[i] == "--image" and i + 1 < len(args):
            image_path = Path(args[i + 1])
            i += 2
        elif args[i] == "--lang" and i + 1 < len(args):
            lang = args[i + 1]
            i += 2
        elif args[i] == "--mode" and i + 1 < len(args):
            mode = args[i + 1]
            if mode not in {"auto", "printed", "handwritten"}:
                print(f"ERROR: invalid --mode {mode}", file=sys.stderr)
                sys.exit(1)
            i += 2
        elif args[i] == "--json":
            output_json = True
            i += 1
        else:
            print(f"Unknown option: {args[i]}", file=sys.stderr)
            sys.exit(1)

    if image_path is None:
        print("ERROR: --image is required", file=sys.stderr)
        sys.exit(1)

    try:
        result = process_image(image_path, lang=lang, mode=mode)
        if output_json:
            print(json.dumps(result, indent=2, ensure_ascii=False))
        else:
            print(f"OCR Result for: {image_path.name}")
            print(f"Engine: {result['engine']}")
            print(f"Confidence: {result['confidence']:.2f}")
            print("-" * 20)
            print(result["ocr_text"])
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
