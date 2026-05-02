"""
VKS ECMS — PDF to Images Extractor

Tach cac trang PDF thanh file anh PNG de OCR xu ly tiep.
Dung PyMuPDF (fitz) — pure Python, khong can Poppler.

Usage:
    python -m ocr.pdf_to_images --pdf <path> --output-dir <dir> [--dpi 300] [--json]
    python -m ocr.pdf_to_images --folder <path> --output-dir <dir> [--dpi 300] [--json]

Output (--json):
    {
      "pdf_path": "...",
      "total_pages": 5,
      "output_dir": "...",
      "pages": [
        {"index": 0, "path": "page_001.png", "width": 2480, "height": 3508}
      ]
    }

Called from Tauri via tauri-plugin-shell.
"""

import json
import os
import sys
from pathlib import Path
from typing import Optional

try:
    import fitz  # PyMuPDF
except ImportError:
    print("ERROR: PyMuPDF not installed. Run: pip install PyMuPDF", file=sys.stderr)
    sys.exit(1)


def extract_pages(
    pdf_path: Path,
    output_dir: Path,
    dpi: int = 300,
    page_number: Optional[int] = None,
) -> dict:
    """
    Extract all pages from a PDF as PNG images.

    Args:
        pdf_path: Path to the PDF file
        output_dir: Directory to save page images
        dpi: Resolution for rendering (default 300 for OCR quality)

    Returns:
        dict with pdf_path, total_pages, output_dir, pages[]
    """
    pdf_path = pdf_path.resolve()
    output_dir = output_dir.resolve()

    if not pdf_path.exists():
        raise FileNotFoundError(f"PDF not found: {pdf_path}")
    if not pdf_path.suffix.lower() == ".pdf":
        raise ValueError(f"Not a PDF file: {pdf_path}")

    output_dir.mkdir(parents=True, exist_ok=True)

    doc = fitz.open(str(pdf_path))
    pages = []

    try:
        zoom = dpi / 72.0  # fitz default is 72 DPI
        mat = fitz.Matrix(zoom, zoom)

        page_indexes = range(len(doc))
        if page_number is not None:
            if page_number < 1 or page_number > len(doc):
                raise ValueError(f"Page out of range: {page_number}/{len(doc)}")
            page_indexes = range(page_number - 1, page_number)

        for page_idx in page_indexes:
            page = doc[page_idx]
            pix = page.get_pixmap(matrix=mat, alpha=False)
            text_layer = page.get_text("text").strip()

            page_num = page_idx + 1
            img_name = f"page_{page_num:03d}.png"
            img_path = output_dir / img_name

            pix.save(str(img_path))

            pages.append({
                "index": page_idx,
                "page_num": page_num,
                "path": str(img_path),
                "filename": img_name,
                "width": pix.width,
                "height": pix.height,
                "text": text_layer,
                "text_length": len(text_layer),
            })
    finally:
        doc.close()

    return {
        "pdf_path": str(pdf_path),
        "pdf_filename": pdf_path.name,
        "total_pages": len(pages),
        "output_dir": str(output_dir),
        "dpi": dpi,
        "requested_page": page_number,
        "pages": pages,
    }


def extract_folder(
    folder_path: Path,
    output_base_dir: Path,
    dpi: int = 300,
) -> list[dict]:
    """
    Extract pages from all PDFs in a folder (recursively).

    Each PDF gets its own subdirectory under output_base_dir,
    named by the PDF stem (filename without extension).

    Returns:
        list of extract_pages results
    """
    folder_path = folder_path.resolve()
    if not folder_path.is_dir():
        raise NotADirectoryError(f"Not a directory: {folder_path}")

    results = []
    pdf_files = sorted(folder_path.rglob("*.pdf"))

    for pdf_path in pdf_files:
        # Create output subdir named after the PDF
        relative = pdf_path.relative_to(folder_path)
        subdir_name = str(relative.with_suffix("")).replace(os.sep, "__")
        output_dir = output_base_dir / subdir_name

        try:
            result = extract_pages(pdf_path, output_dir, dpi)
            results.append(result)
            print(
                f"  OK: {pdf_path.name} → {result['total_pages']} pages",
                file=sys.stderr,
            )
        except Exception as exc:
            error_result = {
                "pdf_path": str(pdf_path),
                "pdf_filename": pdf_path.name,
                "total_pages": 0,
                "output_dir": str(output_dir),
                "dpi": dpi,
                "pages": [],
                "error": str(exc),
            }
            results.append(error_result)
            print(f"  FAIL: {pdf_path.name} — {exc}", file=sys.stderr)

    return results


def main(args: Optional[list[str]] = None) -> None:
    """CLI entry point."""
    if args is None:
        args = sys.argv[1:]

    if not args or "--help" in args:
        print(__doc__)
        sys.exit(0)

    pdf_path: Optional[Path] = None
    folder_path: Optional[Path] = None
    output_dir: Optional[Path] = None
    dpi = 300
    page_number: Optional[int] = None
    output_json = False

    i = 0
    while i < len(args):
        if args[i] == "--pdf" and i + 1 < len(args):
            pdf_path = Path(args[i + 1])
            i += 2
        elif args[i] == "--folder" and i + 1 < len(args):
            folder_path = Path(args[i + 1])
            i += 2
        elif args[i] == "--output-dir" and i + 1 < len(args):
            output_dir = Path(args[i + 1])
            i += 2
        elif args[i] == "--dpi" and i + 1 < len(args):
            dpi = int(args[i + 1])
            i += 2
        elif args[i] == "--page" and i + 1 < len(args):
            page_number = int(args[i + 1])
            i += 2
        elif args[i] == "--json":
            output_json = True
            i += 1
        else:
            print(f"Unknown option: {args[i]}", file=sys.stderr)
            sys.exit(1)

    if output_dir is None:
        print("ERROR: --output-dir is required", file=sys.stderr)
        sys.exit(1)

    if folder_path:
        print(f"Extracting all PDFs in: {folder_path}", file=sys.stderr)
        results = extract_folder(folder_path, output_dir, dpi)
        total_pdfs = len(results)
        total_pages = sum(r["total_pages"] for r in results)
        errors = sum(1 for r in results if "error" in r)
        print(
            f"Done: {total_pdfs} PDFs, {total_pages} pages, {errors} errors",
            file=sys.stderr,
        )
        if output_json:
            print(json.dumps(results, indent=2, ensure_ascii=False))
        else:
            for r in results:
                status = "ERROR" if "error" in r else "OK"
                print(f"  [{status}] {r['pdf_filename']}: {r['total_pages']} pages")

    elif pdf_path:
        print(f"Extracting: {pdf_path}", file=sys.stderr)
        result = extract_pages(pdf_path, output_dir, dpi, page_number=page_number)
        print(f"Done: {result['total_pages']} pages", file=sys.stderr)
        if output_json:
            print(json.dumps(result, indent=2, ensure_ascii=False))
        else:
            for p in result["pages"]:
                print(f"  page {p['page_num']}: {p['width']}x{p['height']} → {p['filename']}")

    else:
        print("ERROR: Either --pdf or --folder is required", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
