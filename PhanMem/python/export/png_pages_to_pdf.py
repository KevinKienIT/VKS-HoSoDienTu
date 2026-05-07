from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from PIL import Image


def _normalize_rotation(value: Any) -> int:
    try:
        rotation = int(value or 0) % 360
    except (TypeError, ValueError):
        rotation = 0
    if rotation not in (0, 90, 180, 270):
        return 0
    return rotation


def _load_page_image(image_path: Path, rotation: int) -> Image.Image:
    image = Image.open(image_path)
    image.load()
    if image.mode not in ("RGB", "L"):
        image = image.convert("RGB")
    elif image.mode == "L":
        image = image.convert("RGB")
    if rotation:
        image = image.rotate(-rotation, expand=True)
    return image


def export_png_pages(plan_path: Path, output_path: Path, result_path: Path | None = None) -> dict[str, Any]:
    plan = json.loads(plan_path.read_text(encoding="utf-8"))
    pages = plan.get("pages") or []
    included_pages = [page for page in pages if page.get("included")]
    if not included_pages:
        raise ValueError("EXPORT_PNG_NO_INCLUDED_PAGES")

    images: list[Image.Image] = []
    results: list[dict[str, Any]] = []
    try:
        for page in included_pages:
            image_path = Path(str(page.get("image_path") or ""))
            if not image_path.exists() or not image_path.is_file():
                raise FileNotFoundError(f"EXPORT_PNG_IMAGE_MISSING:{image_path}")
            rotation = _normalize_rotation(page.get("rotation"))
            image = _load_page_image(image_path, rotation)
            images.append(image)
            results.append(
                {
                    "page_id": page.get("page_id"),
                    "document_id": page.get("document_id"),
                    "image_path": str(image_path),
                    "rotation": rotation,
                    "width_px": image.width,
                    "height_px": image.height,
                    "dpi": image.info.get("dpi"),
                }
            )

        output_path.parent.mkdir(parents=True, exist_ok=True)
        first, rest = images[0], images[1:]
        dpi = first.info.get("dpi", (300, 300))
        first.save(output_path, "PDF", resolution=float(dpi[0] if isinstance(dpi, tuple) else dpi), save_all=True, append_images=rest)
    finally:
        for image in images:
            image.close()

    result = {
        "status": "ok",
        "output_path": str(output_path),
        "pages_written": len(included_pages),
        "pages": results,
    }
    if result_path is not None:
        result_path.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Export ordered PNG pages to a PDF file.")
    parser.add_argument("--plan", required=True, help="Path to JSON export plan")
    parser.add_argument("--output", required=True, help="Output PDF path")
    parser.add_argument("--result", required=False, help="Optional result JSON path")
    args = parser.parse_args()

    try:
        result = export_png_pages(
            Path(args.plan),
            Path(args.output),
            Path(args.result) if args.result else None,
        )
        print(json.dumps(result, ensure_ascii=False))
        return 0
    except Exception as exc:  # noqa: BLE001 - CLI must return structured error to Rust caller.
        print(json.dumps({"status": "error", "error": str(exc)}, ensure_ascii=False), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
