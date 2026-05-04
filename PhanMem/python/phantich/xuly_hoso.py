"""
VKS ECMS - Xu ly ho so vu an day du
Pipeline: PDF -> OCR -> Phan tich -> Xuat Word chuyen nghiep

Chay: .venv\\Scripts\\python.exe python/phantich/xuly_hoso.py --input "h:\\TAI LIEU\\test"
"""
import json, os, re, sys, unicodedata, traceback
from datetime import datetime
from pathlib import Path
from typing import Any

sys.stdout.reconfigure(encoding="utf-8")

# ============================================================
# HELPERS
# ============================================================
def nfc(text: str) -> str:
    return unicodedata.normalize("NFC", text)

def no_diacritics(text: str) -> str:
    d = unicodedata.normalize("NFD", text)
    s = "".join(ch for ch in d if unicodedata.category(ch) != "Mn")
    return s.replace("\u0110", "D").replace("\u0111", "d").lower()

# ============================================================
# 1. DOC PDF
# ============================================================
def process_pdf(pdf_path: str, output_dir: str, ai_processor=None) -> dict:
    """Doc PDF, trich xuat text + images, OCR tung page. Ket hop AI lam min neu co."""
    import fitz
    doc = fitz.open(pdf_path)
    fname = os.path.basename(pdf_path)
    pdf_out = os.path.join(output_dir, "images", Path(fname).stem)
    os.makedirs(pdf_out, exist_ok=True)

    pages = []
    all_text = ""
    all_fields = []
    for i, page in enumerate(doc):
        # Text extraction (digital PDF)
        text = nfc(page.get_text().strip())

        # Render page to image
        pix = page.get_pixmap(dpi=300)
        img_path = os.path.join(pdf_out, f"page_{i+1:03d}.png")
        pix.save(img_path)

        # OCR if text is empty/short (scanned PDF)
        ocr_result = None
        if len(text) < 30:
            try:
                sys.path.insert(0, str(Path(__file__).parent.parent))
                from ocr.ocr_pipeline import process_image
                ocr_result = process_image(Path(img_path), lang="vi", mode="auto")
                text = ocr_result.get("raw_text", "") or ""
                text = nfc(text)
                
                # CHẠY SLOW SERVICE: AI Agent làm mịn text nếu được yêu cầu
                if ai_processor and text.strip():
                    text = ai_processor.clean_text(text)
                
                for f in ocr_result.get("extracted_fields", []):
                    f["page"] = i + 1
                    f["source_file"] = fname
                    all_fields.append(f)
            except Exception as e:
                text = f"[OCR ERROR: {e}]"

        pages.append({
            "page_number": i + 1,
            "text": text[:2000],
            "char_count": len(text),
            "image_path": img_path,
            "ocr_applied": ocr_result is not None,
            "confidence": ocr_result.get("avg_confidence", 0) if ocr_result else None,
            "needs_review": ocr_result.get("needs_review", False) if ocr_result else False,
        })
        all_text += text + "\n"

    doc.close()
    return {
        "filename": fname,
        "path": pdf_path,
        "page_count": len(pages),
        "total_chars": len(all_text),
        "pages": pages,
        "full_text": all_text,
        "extracted_fields": all_fields,
    }

# ============================================================
# 2. PHAN LOAI TAI LIEU
# ============================================================
DOC_TYPES = {
    "quyet_dinh_khoi_to": (r"quy[eế]t\s*[dđ][iị]nh\s*kh[oở]i\s*t[oố]", "Quyet dinh khoi to"),
    "quyet_dinh": (r"quy[eế]t\s*[dđ][iị]nh", "Quyet dinh"),
    "lenh_bat": (r"l[eệ]nh\s*(?:b[aắ]t|t[aạ]m)", "Lenh bat/tam giu"),
    "bien_ban": (r"bi[eê]n\s*b[aả]n", "Bien ban"),
    "ket_luan_giam_dinh": (r"k[eế]t\s*lu[aậ]n\s*gi[aá]m\s*[dđ][iị]nh", "Ket luan giam dinh"),
    "giam_dinh": (r"gi[aá]m\s*[dđ][iị]nh", "Giam dinh"),
    "cao_trang": (r"c[aá]o\s*tr[aạ]ng", "Cao trang"),
    "loi_khai": (r"l[oờ]i\s*khai|cung", "Loi khai / Cung"),
    "bien_ban_hien_truong": (r"hi[eệ]n\s*tr[uưừ][oờ]ng", "Bien ban hien truong"),
    "ho_so_y_te": (r"y\s*t[eế]|b[eệ]nh\s*[aá]n|th[uư][oơ]ng\s*t[ií]ch|gi[aá]m\s*[dđ][iị]nh.*th[uư][oơ]ng", "Ho so y te"),
    "bao_cao": (r"b[aá]o\s*c[aá]o", "Bao cao"),
    "giay_trieu_tap": (r"tri[eệ]u\s*t[aậ]p", "Giay trieu tap"),
}

def classify_document(text: str) -> str:
    n = no_diacritics(text[:500])
    for key, (pat, _) in DOC_TYPES.items():
        if re.search(pat, n):
            return key
    return "khac"

def get_doc_label(key: str) -> str:
    return DOC_TYPES.get(key, ("", key.replace("_"," ").title()))[1]

# ============================================================
# 3. TRICH XUAT TIMELINE
# ============================================================
def extract_dates(text: str) -> list[dict]:
    events = []
    for m in re.finditer(r"(\d{1,2})[/\-](\d{1,2})[/\-](\d{4})", text):
        d, mo, y = m.group(1), m.group(2), m.group(3)
        ctx = text[max(0, m.start()-10):m.end()+120].strip()
        ctx = re.sub(r"\s+", " ", ctx)
        events.append({"date": f"{d}/{mo}/{y}", "sort": f"{y}{mo.zfill(2)}{d.zfill(2)}", "context": ctx[:150]})
    seen = set()
    unique = []
    for e in sorted(events, key=lambda x: x["sort"]):
        k = (e["date"], e["context"][:40])
        if k not in seen:
            seen.add(k)
            unique.append(e)
    return unique

# ============================================================
# 4. XUAT WORD CHUYEN NGHIEP
# ============================================================
def create_professional_docx(data: dict, output_path: str):
    """Tao file Word chuyen nghiep: Times New Roman 12, table, bold, background."""
    from docx import Document
    from docx.shared import Pt, RGBColor, Cm, Inches, Emu
    from docx.enum.text import WD_ALIGN_PARAGRAPH
    from docx.enum.table import WD_TABLE_ALIGNMENT
    from docx.oxml.ns import qn
    from docx.oxml import OxmlElement

    doc = Document()

    # --- Default style: Times New Roman 12 ---
    style = doc.styles["Normal"]
    font = style.font
    font.name = "Times New Roman"
    font.size = Pt(12)
    style.paragraph_format.space_after = Pt(3)
    style.paragraph_format.line_spacing = 1.15

    # Set East Asian font
    rFonts = style.element.rPr
    if rFonts is None:
        rFonts = OxmlElement("w:rPr")
        style.element.append(rFonts)

    def add_shading(paragraph, color="D9E2F3"):
        shd = OxmlElement("w:shd")
        shd.set(qn("w:fill"), color)
        shd.set(qn("w:val"), "clear")
        paragraph.paragraph_format.element.get_or_add_pPr().append(shd)

    def add_heading_styled(text, level=1):
        h = doc.add_heading(text, level=level)
        for run in h.runs:
            run.font.name = "Times New Roman"
            run.font.color.rgb = RGBColor(0x1B, 0x3A, 0x6B)
        return h

    def add_bold_para(text, size=12, color=None):
        p = doc.add_paragraph()
        run = p.add_run(text)
        run.bold = True
        run.font.name = "Times New Roman"
        run.font.size = Pt(size)
        if color:
            run.font.color.rgb = RGBColor(*color)
        return p

    # --- TRANG BIA ---
    for _ in range(4):
        doc.add_paragraph()
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    run = p.add_run("CONG HOA XA HOI CHU NGHIA VIET NAM")
    run.bold = True; run.font.size = Pt(14); run.font.name = "Times New Roman"
    p2 = doc.add_paragraph()
    p2.alignment = WD_ALIGN_PARAGRAPH.CENTER
    run2 = p2.add_run("Doc lap - Tu do - Hanh phuc")
    run2.font.size = Pt(13); run2.font.name = "Times New Roman"; run2.italic = True

    doc.add_paragraph()
    title = doc.add_paragraph()
    title.alignment = WD_ALIGN_PARAGRAPH.CENTER
    run_t = title.add_run("BAO CAO TONG HOP PHAN TICH HO SO VU AN")
    run_t.bold = True; run_t.font.size = Pt(18); run_t.font.name = "Times New Roman"
    run_t.font.color.rgb = RGBColor(0x1B, 0x3A, 0x6B)

    if data.get("case_info", {}).get("bi_can"):
        sub = doc.add_paragraph()
        sub.alignment = WD_ALIGN_PARAGRAPH.CENTER
        r = sub.add_run(f"Bi can: {data['case_info']['bi_can']}")
        r.font.size = Pt(14); r.font.name = "Times New Roman"; r.bold = True

    date_p = doc.add_paragraph()
    date_p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    r = date_p.add_run(f"Ngay lap: {datetime.now().strftime('%d/%m/%Y')}")
    r.font.size = Pt(12); r.font.name = "Times New Roman"; r.italic = True

    doc.add_page_break()

    # --- PHAN I: THONG TIN VU AN ---
    add_heading_styled("PHAN I: THONG TIN VU AN", 1)
    ci = data.get("case_info", {})
    info_table = doc.add_table(rows=0, cols=2)
    info_table.style = "Table Grid"
    info_table.alignment = WD_TABLE_ALIGNMENT.CENTER
    info_items = [
        ("So vu an", ci.get("so_vu_an", "Chua xac dinh")),
        ("Bi can", ci.get("bi_can", "Chua xac dinh")),
        ("Toi danh", ci.get("toi_danh", "Chua xac dinh")),
        ("Dieu luat", ci.get("dieu_luat", "")),
        ("Co quan dieu tra", ci.get("co_quan_dieu_tra", "")),
    ]
    for label, value in info_items:
        if value:
            row = info_table.add_row()
            c0 = row.cells[0]; c1 = row.cells[1]
            r0 = c0.paragraphs[0].add_run(label)
            r0.bold = True; r0.font.name = "Times New Roman"; r0.font.size = Pt(12)
            r1 = c1.paragraphs[0].add_run(str(value))
            r1.font.name = "Times New Roman"; r1.font.size = Pt(12)
            # Shading header column
            shd = OxmlElement("w:shd")
            shd.set(qn("w:fill"), "D9E2F3"); shd.set(qn("w:val"), "clear")
            c0._tc.get_or_add_tcPr().append(shd)
    for row in info_table.rows:
        for cell in row.cells:
            cell.width = Cm(8)

    doc.add_paragraph()

    # --- PHAN II: BANG CHI MUC TAI LIEU ---
    add_heading_styled("PHAN II: BANG CHI MUC TAI LIEU", 1)
    docs = data.get("documents", [])
    if docs:
        idx_table = doc.add_table(rows=1, cols=5)
        idx_table.style = "Table Grid"
        headers = ["STT", "Ten file", "Loai", "So trang", "Ghi chu"]
        for i, h in enumerate(headers):
            cell = idx_table.rows[0].cells[i]
            r = cell.paragraphs[0].add_run(h)
            r.bold = True; r.font.name = "Times New Roman"; r.font.size = Pt(11)
            cell.paragraphs[0].alignment = WD_ALIGN_PARAGRAPH.CENTER
            shd = OxmlElement("w:shd")
            shd.set(qn("w:fill"), "1B3A6B"); shd.set(qn("w:val"), "clear")
            cell._tc.get_or_add_tcPr().append(shd)
            r.font.color.rgb = RGBColor(0xFF, 0xFF, 0xFF)

        for i, d in enumerate(docs):
            row = idx_table.add_row()
            vals = [
                str(i + 1),
                d.get("filename", ""),
                get_doc_label(d.get("doc_type", "khac")),
                str(d.get("page_count", "")),
                "Can review" if d.get("has_review_pages") else "",
            ]
            for j, v in enumerate(vals):
                r = row.cells[j].paragraphs[0].add_run(v)
                r.font.name = "Times New Roman"; r.font.size = Pt(11)
                if i % 2 == 1:
                    shd = OxmlElement("w:shd")
                    shd.set(qn("w:fill"), "F2F2F2"); shd.set(qn("w:val"), "clear")
                    row.cells[j]._tc.get_or_add_tcPr().append(shd)

    doc.add_paragraph()

    # --- PHAN III: TIMELINE ---
    add_heading_styled("PHAN III: DIEN BIEN THOI GIAN", 1)
    timeline = data.get("timeline", [])
    if timeline:
        tl_table = doc.add_table(rows=1, cols=3)
        tl_table.style = "Table Grid"
        for i, h in enumerate(["STT", "Ngay", "Noi dung"]):
            cell = tl_table.rows[0].cells[i]
            r = cell.paragraphs[0].add_run(h)
            r.bold = True; r.font.name = "Times New Roman"; r.font.size = Pt(11)
            shd = OxmlElement("w:shd")
            shd.set(qn("w:fill"), "1B3A6B"); shd.set(qn("w:val"), "clear")
            cell._tc.get_or_add_tcPr().append(shd)
            r.font.color.rgb = RGBColor(0xFF, 0xFF, 0xFF)
        for i, ev in enumerate(timeline[:30]):
            row = tl_table.add_row()
            for j, v in enumerate([str(i+1), ev["date"], ev["context"][:120]]):
                r = row.cells[j].paragraphs[0].add_run(v)
                r.font.name = "Times New Roman"; r.font.size = Pt(11)

    doc.add_paragraph()

    # --- PHAN IV: CHI TIET TUNG TAI LIEU ---
    add_heading_styled("PHAN IV: CHI TIET TUNG TAI LIEU", 1)
    for i, d in enumerate(docs):
        add_heading_styled(f"{i+1}. {d.get('filename', '')}", 2)
        p = doc.add_paragraph()
        add_shading(p, "E8F0FE")
        r = p.add_run(f"Loai: {get_doc_label(d.get('doc_type','khac'))} | Trang: {d.get('page_count',0)} | Ky tu: {d.get('total_chars',0)}")
        r.font.name = "Times New Roman"; r.font.size = Pt(11); r.italic = True

        # Extracted fields
        fields = d.get("extracted_fields", [])
        if fields:
            add_bold_para("Truong trich xuat:", 11, (0x1B, 0x3A, 0x6B))
            for f in fields[:10]:
                p = doc.add_paragraph(style="List Bullet")
                r = p.add_run(f"{f.get('field_name','')}: ")
                r.bold = True; r.font.name = "Times New Roman"; r.font.size = Pt(11)
                r2 = p.add_run(f"{f.get('field_value','')} (conf: {f.get('confidence',0):.2f})")
                r2.font.name = "Times New Roman"; r2.font.size = Pt(11)

        # Page summary
        for pg in d.get("pages", [])[:3]:
            txt = pg.get("text", "")[:200]
            if txt.strip():
                p = doc.add_paragraph()
                r = p.add_run(f"Trang {pg['page_number']}: ")
                r.bold = True; r.font.name = "Times New Roman"; r.font.size = Pt(10)
                r2 = p.add_run(txt.replace("\n", " "))
                r2.font.name = "Times New Roman"; r2.font.size = Pt(10)
                r2.font.color.rgb = RGBColor(0x55, 0x55, 0x55)

    # --- PHAN V: KET LUAN ---
    doc.add_page_break()
    add_heading_styled("PHAN V: KET LUAN VA DE XUAT", 1)
    summary_items = [
        f"Tong so tai lieu: {len(docs)} files",
        f"Tong so trang: {sum(d.get('page_count',0) for d in docs)}",
        f"Tong so su kien: {len(timeline)} moc thoi gian",
        f"Tai lieu can review: {sum(1 for d in docs if d.get('has_review_pages'))}",
    ]
    for item in summary_items:
        p = doc.add_paragraph(style="List Bullet")
        r = p.add_run(item)
        r.font.name = "Times New Roman"; r.font.size = Pt(12)

    p = doc.add_paragraph()
    add_shading(p, "FFF3CD")
    r = p.add_run("LUU Y: Tat ca ket qua OCR va phan tich can duoc kiem tra thu cong truoc khi su dung chinh thuc.")
    r.bold = True; r.font.name = "Times New Roman"; r.font.size = Pt(12)
    r.font.color.rgb = RGBColor(0x85, 0x6A, 0x04)

    # Footer
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.RIGHT
    r = p.add_run(f"VKS ECMS - {datetime.now().strftime('%d/%m/%Y %H:%M')}")
    r.italic = True; r.font.name = "Times New Roman"; r.font.size = Pt(9)
    r.font.color.rgb = RGBColor(0x99, 0x99, 0x99)

    doc.save(output_path)
    return output_path

# ============================================================
# 5. MAIN PIPELINE
# ============================================================
def run_pipeline(input_dir: str, output_dir: str = None, use_ai: bool = False):
    input_path = Path(input_dir)
    if not input_path.exists():
        print(f"ERROR: Thu muc khong ton tai: {input_dir}")
        print("Vui long kiem tra duong dan va thu lai.")
        return

    if output_dir is None:
        output_dir = str(input_path / "_output_vks")
    os.makedirs(output_dir, exist_ok=True)
    os.makedirs(os.path.join(output_dir, "images"), exist_ok=True)

    pdf_files = sorted(input_path.glob("*.pdf"))
    if not pdf_files:
        print(f"Khong tim thay file PDF nao trong: {input_dir}")
        return
        
    ai_processor = None
    if use_ai:
        try:
            from phantich.ai_xuly_text import AITextProcessor
            ai_processor = AITextProcessor(model_name="qwen2.5:3b")
            print("  [AI] Chế độ làm mịn bằng AI Agent ĐÃ KÍCH HOẠT (Slow Service)")
        except ImportError:
            print("  [AI] Không tìm thấy module AI. Chạy chế độ thông thường.")

    print(f"\n{'='*60}")
    print(f"  VKS ECMS - XU LY HO SO VU AN")
    print(f"  Input: {input_dir}")
    print(f"  Files: {len(pdf_files)} PDFs")
    print(f"  Output: {output_dir}")
    print(f"{'='*60}")

    # Process each PDF
    all_docs = []
    all_text = ""
    for i, pdf in enumerate(pdf_files):
        print(f"\n[{i+1}/{len(pdf_files)}] {pdf.name}...")
        try:
            result = process_pdf(str(pdf), output_dir, ai_processor=ai_processor)
            result["doc_type"] = classify_document(result["full_text"])
            result["has_review_pages"] = any(p.get("needs_review") for p in result["pages"])
            all_docs.append(result)
            all_text += result["full_text"] + "\n"
            print(f"  -> {result['page_count']} pages, type={result['doc_type']}, {result['total_chars']} chars")
        except Exception as e:
            print(f"  ERROR: {e}")
            traceback.print_exc()

    # Extract case info
    print("\n[Phan tich] Trich xuat thong tin vu an...")
    case_info = {}
    m = re.search(r"[Bb][ịi]\s*can[:\s]*([A-Z\s\u0100-\u024F\u1E00-\u1EFF]+)", all_text)
    if m: case_info["bi_can"] = m.group(1).strip()
    m = re.search(r"(?:S[oố]|so).*?(\d+/Q[ĐD].*?)[\s\n,]", all_text)
    if m: case_info["so_vu_an"] = m.group(1).strip()
    m = re.search(r"[Đđ]i[ềe]u\s*(\d+)", all_text)
    if m: case_info["dieu_luat"] = f"Dieu {m.group(1)}"

    # Timeline
    timeline = extract_dates(all_text)
    print(f"  Timeline: {len(timeline)} su kien")

    # Aggregate fields
    all_fields = []
    for d in all_docs:
        all_fields.extend(d.get("extracted_fields", []))

    # Build data for Word
    report_data = {
        "case_info": case_info,
        "documents": [{k: v for k, v in d.items() if k not in ("full_text",)} for d in all_docs],
        "timeline": timeline,
        "extracted_fields": all_fields,
    }

    # Save JSON
    json_path = os.path.join(output_dir, "ket_qua_phan_tich.json")
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(report_data, f, ensure_ascii=False, indent=2, default=str)
    print(f"  JSON: {json_path}")

    # Create Word
    docx_path = os.path.join(output_dir, "Bao_cao_tong_hop_phan_tich.docx")
    create_professional_docx(report_data, docx_path)
    print(f"  DOCX: {docx_path}")

    print(f"\n{'='*60}")
    print(f"  HOAN THANH!")
    print(f"  Files: {len(all_docs)} PDFs xu ly")
    print(f"  Pages: {sum(d.get('page_count',0) for d in all_docs)}")
    print(f"  Output: {output_dir}")
    print(f"{'='*60}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="VKS ECMS - Xu ly ho so vu an")
    parser.add_argument("--input", required=True, help="Thu muc chua file PDF")
    parser.add_argument("--output", default=None, help="Thu muc xuat ket qua")
    parser.add_argument("--use-ai", action="store_true", help="Kich hoat AI Agent de lam min text (Slow Service)")
    args = parser.parse_args()
    run_pipeline(args.input, args.output, use_ai=args.use_ai)
