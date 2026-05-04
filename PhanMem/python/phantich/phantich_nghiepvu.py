"""
VKS ECMS - Phan tich nghiep vu ho so vu an
(Business Analysis Pipeline)

Script nay doc cac file tham chieu trong 06_reference_samples,
trich xuat cau truc nghiep vu, va tao luong phan tich chuyen nghiep.

Luong xu ly:
1. Doc DOCX/HTML bang PyMuPDF
2. Trich xuat cac truong nghiep vu (regex + rule)
3. Phan loai tai lieu theo nhom
4. Xay dung timeline su kien
5. Tao bao cao tong hop (extractive)
6. [Optional] Goi Ollama AI de tong hop, phan tich sau

Usage:
    python phantich_nghiepvu.py
"""

import json
import os
import re
import sys
import unicodedata
from datetime import datetime
from pathlib import Path
from typing import Any

sys.stdout.reconfigure(encoding="utf-8")

# ================================================================
# 1. DOC FILES
# ================================================================

def read_docx(path: str) -> dict:
    """Doc file DOCX bang PyMuPDF, tra ve text + metadata."""
    import fitz
    doc = fitz.open(path)
    pages = []
    full_text = ""
    for i, page in enumerate(doc):
        text = page.get_text()
        pages.append({
            "page_number": i + 1,
            "text": text,
            "char_count": len(text),
            "word_count": len(text.split()),
        })
        full_text += text + "\n"
    result = {
        "filename": os.path.basename(path),
        "page_count": len(doc),
        "total_chars": len(full_text),
        "total_words": len(full_text.split()),
        "pages": pages,
        "full_text": full_text,
    }
    doc.close()
    return result


def read_html(path: str) -> dict:
    """Doc file HTML, tra ve text + cau truc."""
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    # Strip HTML tags for plain text
    plain = re.sub(r"<[^>]+>", " ", content)
    plain = re.sub(r"\s+", " ", plain).strip()

    # Extract headings
    headings = []
    for m in re.finditer(r"<h([1-6])[^>]*>(.*?)</h\1>", content, re.DOTALL | re.IGNORECASE):
        level = int(m.group(1))
        text = re.sub(r"<[^>]+>", "", m.group(2)).strip()
        if text:
            headings.append({"level": level, "text": text})

    # Extract timeline/cards
    cards = []
    for m in re.finditer(
        r'class=["\'](?:timeline-item|card|event)["\'][^>]*>(.*?)</(?:div|section)>',
        content, re.DOTALL | re.IGNORECASE
    ):
        card_text = re.sub(r"<[^>]+>", " ", m.group(1)).strip()
        card_text = re.sub(r"\s+", " ", card_text)
        if len(card_text) > 10:
            cards.append(card_text[:300])

    return {
        "filename": os.path.basename(path),
        "size_bytes": len(content),
        "plain_text": plain,
        "headings": headings,
        "cards": cards,
        "total_chars": len(plain),
    }


# ================================================================
# 2. TRICH XUAT TRUONG NGHIEP VU
# ================================================================

def normalize_vn(text: str) -> str:
    """Normalize Vietnamese text (remove diacritics for matching)."""
    decomposed = unicodedata.normalize("NFD", text)
    stripped = "".join(ch for ch in decomposed if unicodedata.category(ch) != "Mn")
    return stripped.replace("Đ", "D").replace("đ", "d").lower()


def extract_case_info(text: str) -> dict:
    """Trich xuat thong tin vu an tu van ban."""
    info: dict[str, Any] = {}

    # So vu an
    m = re.search(r"(?:S[oố]\s*(?:v[uụ]\s*[aá]n|QĐ|quyết\s*định))[:\s]*([^\n\r,;]+)", text, re.IGNORECASE)
    if m:
        info["so_vu_an"] = m.group(1).strip()

    # Bi can
    m = re.search(r"[Bb][ịi]\s*can[:\s]*([A-ZĐ][A-ZĐÀÁẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬÈÉẺẼẸÊẾỀỂỄỆÌÍỈĨỊÒÓỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÙÚỦŨỤƯỨỪỬỮỰỲÝỶỸỴ\s]+)", text)
    if m:
        info["bi_can"] = m.group(1).strip()

    # Toi danh
    patterns_toi = [
        r"[Tt][ộo]i\s*(?:danh)?[:\s]*(.*?)(?:\n|–|\.|$)",
        r"(?:CỐ\s*Ý\s*GÂY\s*THƯƠNG\s*TÍCH|Cố\s*ý\s*gây\s*thương\s*tích)",
    ]
    for pat in patterns_toi:
        m = re.search(pat, text)
        if m:
            info["toi_danh"] = m.group(0).strip() if not m.groups() else m.group(1).strip()
            break

    # Dieu luat
    m = re.search(r"[Đđ]i[ềe]u\s*(\d+)\s*(?:khoản\s*(\d+))?\s*(?:BLHS|B[ộo]\s*lu[ậa]t\s*[Hh][ìi]nh\s*s[ựu])?", text)
    if m:
        info["dieu_luat"] = f"Dieu {m.group(1)}" + (f" khoan {m.group(2)}" if m.group(2) else "")

    # Co quan dieu tra
    m = re.search(r"[Cc][oơ]\s*quan\s*(?:điều\s*tra|CSĐT)[:\s]*([^\n\r]+)", text, re.IGNORECASE)
    if m:
        info["co_quan_dieu_tra"] = m.group(1).strip()

    # Ngay lap
    dates = re.findall(
        r"(?:ng[àa]y\s+)?(\d{1,2})[/-](\d{1,2})[/-](\d{4})", text
    )
    if dates:
        info["cac_ngay"] = [f"{d}/{m}/{y}" for d, m, y in dates[:10]]

    # Nguoi bi hai
    victims = []
    for m in re.finditer(
        r"(?:Người\s*bị\s*hại|Bị\s*hại)[:\s]*([A-ZĐ][a-zđàáảãạăắằẳẵặâấầẩẫậèéẻẽẹêếềểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữựỳýỷỹỵ\s]+)",
        text
    ):
        victims.append(m.group(1).strip())
    if victims:
        info["nguoi_bi_hai"] = victims

    return info


def classify_documents(text: str) -> list[dict]:
    """Phan loai tai lieu theo nhom nghiep vu."""
    categories = []
    patterns = {
        "quyet_dinh": r"[Qq]uy[ếe]t\s*[đd][ịi]nh",
        "bien_ban": r"[Bb]i[êe]n\s*b[aả]n",
        "ket_luan": r"[Kk][ếe]t\s*lu[ậa]n",
        "giam_dinh": r"[Gg]i[áa]m\s*[đd][ịi]nh",
        "trieu_tap": r"[Tt]ri[ệe]u\s*t[ậa]p",
        "cung": r"[Cc]ung|[Ll][ờo]i\s*khai",
        "cao_trang": r"[Cc]áo\s*tr[ạa]ng",
        "bao_cao": r"[Bb][áa]o\s*c[áa]o",
        "quyet_dinh_khoi_to": r"[Kk]h[ởo]i\s*t[ốo]",
        "lenh_bat": r"[Ll][ệe]nh\s*(?:b[ắa]t|t[ạa]m)",
    }

    for cat, pat in patterns.items():
        matches = re.findall(pat, text)
        if matches:
            categories.append({
                "category": cat,
                "count": len(matches),
                "label_vi": cat.replace("_", " ").title(),
            })

    return sorted(categories, key=lambda c: c["count"], reverse=True)


def extract_timeline(text: str) -> list[dict]:
    """Trich xuat timeline su kien tu van ban."""
    events = []

    # Pattern: ngay DD/MM/YYYY hoac DD thang MM nam YYYY + mo ta
    for m in re.finditer(
        r"(\d{1,2})[/-](\d{1,2})[/-](\d{4})\s*[:\-–]?\s*([^\n\r]{10,200})",
        text
    ):
        day, month, year = m.group(1), m.group(2), m.group(3)
        desc = m.group(4).strip()
        events.append({
            "date": f"{day}/{month}/{year}",
            "date_sort": f"{year}{month.zfill(2)}{day.zfill(2)}",
            "description": desc[:200],
        })

    # Pattern: ngay DD thang MM nam YYYY
    for m in re.finditer(
        r"ng[àa]y\s+(\d{1,2})\s+th[áa]ng\s+(\d{1,2})\s+n[aă]m\s+(\d{4})\s*[,.\-–]?\s*([^\n\r]{5,200})?",
        text, re.IGNORECASE
    ):
        day, month, year = m.group(1), m.group(2), m.group(3)
        desc = (m.group(4) or "").strip()
        events.append({
            "date": f"{day}/{month}/{year}",
            "date_sort": f"{year}{month.zfill(2)}{day.zfill(2)}",
            "description": desc[:200] if desc else f"Su kien ngay {day}/{month}/{year}",
        })

    # Deduplicate by date
    seen = set()
    unique = []
    for e in sorted(events, key=lambda x: x["date_sort"]):
        key = (e["date"], e["description"][:50])
        if key not in seen:
            seen.add(key)
            unique.append(e)

    return unique


# ================================================================
# 3. AI OFFLINE (OLLAMA)
# ================================================================

def ai_summarize(text: str, model: str = "qwen2.5:3b") -> str:
    """Goi Ollama de tom tat van ban. Fallback: extractive summary."""
    try:
        import ollama
        prompt = f"""Ban la tro ly phap ly Viet Nam. Hay tom tat noi dung chinh cua van ban sau.
Yeu cau:
- Viet bang tieng Viet
- Neu ro: So vu an, bi can, toi danh, dieu luat, co quan dieu tra
- Neu ro: Cac moc thoi gian quan trong
- Neu ro: Nguoi bi hai va thuong tich
- Tom tat ngan gon, chinh xac

VAN BAN:
{text[:3000]}

TOM TAT:"""

        response = ollama.chat(
            model=model,
            messages=[{"role": "user", "content": prompt}],
            options={"temperature": 0.3, "num_predict": 500},
        )
        return response["message"]["content"]
    except Exception as e:
        # Fallback: extractive summary
        return f"[AI unavailable: {type(e).__name__}] Extractive summary below."


def ai_analyze_case(case_info: dict, timeline: list, categories: list, model: str = "qwen2.5:3b") -> str:
    """AI phan tich vu an dua tren du lieu da trich xuat."""
    try:
        import ollama
        prompt = f"""Ban la kiem sat vien Viet Nam. Phan tich ho so vu an sau:

THONG TIN VU AN:
{json.dumps(case_info, ensure_ascii=False, indent=2)}

TIMELINE SU KIEN ({len(timeline)} moc):
{json.dumps(timeline[:15], ensure_ascii=False, indent=2)}

PHAN LOAI TAI LIEU ({len(categories)} nhom):
{json.dumps(categories, ensure_ascii=False, indent=2)}

YEU CAU PHAN TICH:
1. Danh gia tinh day du cua ho so
2. Nhan dien cac van de phap ly can luu y
3. Kiem tra timeline co lien tuc va hop ly khong
4. De xuat cac buoc tiep theo

PHAN TICH:"""

        response = ollama.chat(
            model=model,
            messages=[{"role": "user", "content": prompt}],
            options={"temperature": 0.3, "num_predict": 800},
        )
        return response["message"]["content"]
    except Exception as e:
        return f"[AI unavailable: {type(e).__name__}] Rule-based analysis below."


# ================================================================
# 4. BAO CAO TONG HOP
# ================================================================

def generate_report(
    case_info: dict,
    timeline: list,
    categories: list,
    doc_summaries: list,
    ai_summary: str,
    ai_analysis: str,
) -> str:
    """Tao bao cao tong hop chuyen nghiep."""

    report = []
    report.append("=" * 70)
    report.append("  BAO CAO PHAN TICH NGHIEP VU HO SO VU AN")
    report.append("  VKS ECMS - He thong xu ly ho so dien tu")
    report.append(f"  Ngay lap: {datetime.now().strftime('%d/%m/%Y %H:%M')}")
    report.append("=" * 70)

    # I. Thong tin vu an
    report.append("\n" + "-" * 50)
    report.append("PHAN I: THONG TIN VU AN")
    report.append("-" * 50)
    for key, value in case_info.items():
        label = key.replace("_", " ").upper()
        if isinstance(value, list):
            report.append(f"  {label}:")
            for v in value[:5]:
                report.append(f"    - {v}")
        else:
            report.append(f"  {label}: {value}")

    # II. Phan loai tai lieu
    report.append("\n" + "-" * 50)
    report.append("PHAN II: PHAN LOAI TAI LIEU")
    report.append("-" * 50)
    for cat in categories:
        report.append(f"  [{cat['count']:2d}x] {cat['label_vi']}")

    # III. Timeline su kien
    report.append("\n" + "-" * 50)
    report.append("PHAN III: TIMELINE SU KIEN")
    report.append("-" * 50)
    for event in timeline[:20]:
        report.append(f"  {event['date']:12s} | {event['description'][:80]}")

    # IV. Tai lieu da xu ly
    report.append("\n" + "-" * 50)
    report.append("PHAN IV: TAI LIEU DA XU LY")
    report.append("-" * 50)
    for ds in doc_summaries:
        report.append(f"  File: {ds['filename']}")
        report.append(f"    Pages: {ds.get('page_count', 'N/A')}, Words: {ds.get('total_words', 'N/A')}")

    # V. AI Summary
    report.append("\n" + "-" * 50)
    report.append("PHAN V: TOM TAT AI")
    report.append("-" * 50)
    for line in ai_summary.split("\n"):
        report.append(f"  {line}")

    # VI. AI Analysis
    report.append("\n" + "-" * 50)
    report.append("PHAN VI: PHAN TICH AI")
    report.append("-" * 50)
    for line in ai_analysis.split("\n"):
        report.append(f"  {line}")

    # VII. Ket luan
    report.append("\n" + "-" * 50)
    report.append("PHAN VII: KET LUAN")
    report.append("-" * 50)
    report.append(f"  Tong so tai lieu: {len(doc_summaries)} files")
    report.append(f"  Tong so su kien: {len(timeline)} moc thoi gian")
    report.append(f"  Nhom tai lieu: {len(categories)} loai")
    if case_info.get("bi_can"):
        report.append(f"  Bi can: {case_info['bi_can']}")
    if case_info.get("toi_danh"):
        report.append(f"  Toi danh: {case_info['toi_danh']}")
    report.append(f"\n  Trang thai: HOAN THANH PHAN TICH SO BO")
    report.append(f"  Can review thu cong: CO")

    report.append("\n" + "=" * 70)
    report.append("  KET THUC BAO CAO")
    report.append("=" * 70)

    return "\n".join(report)


# ================================================================
# 5. MAIN
# ================================================================

def main():
    ref_dir = Path("d:/JOBS/VKS-HoSoDienTu/v3/06_reference_samples")
    output_dir = Path("d:/JOBS/VKS-HoSoDienTu/PhanMem/python/phantich")

    print("=" * 60)
    print("  VKS ECMS - PHAN TICH NGHIEP VU HO SO")
    print("=" * 60)

    # Step 1: Read all reference files
    print("\n[1/6] Doc tai lieu tham chieu...")
    doc_summaries = []

    docx_files = list(ref_dir.glob("*.docx"))
    html_files = list(ref_dir.glob("*.html"))

    combined_text = ""
    for f in docx_files:
        print(f"  Reading: {f.name}")
        data = read_docx(str(f))
        doc_summaries.append(data)
        combined_text += data["full_text"] + "\n"

    for f in html_files:
        print(f"  Reading: {f.name}")
        data = read_html(str(f))
        doc_summaries.append(data)
        combined_text += data["plain_text"] + "\n"

    print(f"  Total: {len(doc_summaries)} files, {len(combined_text)} chars")

    # Step 2: Extract case info
    print("\n[2/6] Trich xuat thong tin vu an...")
    case_info = extract_case_info(combined_text)
    for k, v in case_info.items():
        if isinstance(v, list):
            print(f"  {k}: {v[:3]}")
        else:
            print(f"  {k}: {v}")

    # Step 3: Classify documents
    print("\n[3/6] Phan loai tai lieu...")
    categories = classify_documents(combined_text)
    for cat in categories[:8]:
        print(f"  [{cat['count']:2d}x] {cat['label_vi']}")

    # Step 4: Extract timeline
    print("\n[4/6] Xay dung timeline...")
    timeline = extract_timeline(combined_text)
    print(f"  Found {len(timeline)} events")
    for e in timeline[:5]:
        print(f"  {e['date']:12s} | {e['description'][:60]}")
    if len(timeline) > 5:
        print(f"  ... and {len(timeline) - 5} more")

    # Step 5: AI Summary (try Ollama, fallback extractive)
    print("\n[5/6] AI tom tat (Ollama)...")
    ai_summary = ai_summarize(combined_text[:4000])
    is_ai_available = not ai_summary.startswith("[AI unavailable")
    print(f"  AI available: {is_ai_available}")
    if not is_ai_available:
        # Extractive fallback
        sentences = [s.strip() for s in combined_text.split(".") if len(s.strip()) > 20]
        ai_summary += "\n" + "\n".join(sentences[:10])
        print("  Using extractive fallback (first 10 sentences)")

    # Step 6: AI Analysis
    print("\n[6/6] AI phan tich nghiep vu...")
    ai_analysis = ai_analyze_case(case_info, timeline, categories)
    if ai_analysis.startswith("[AI unavailable"):
        # Rule-based fallback
        ai_analysis += "\n\nPhan tich rule-based:"
        if len(timeline) < 5:
            ai_analysis += "\n- CANH BAO: Timeline qua it su kien, can bo sung"
        if not case_info.get("bi_can"):
            ai_analysis += "\n- CANH BAO: Chua xac dinh bi can"
        if not case_info.get("toi_danh"):
            ai_analysis += "\n- CANH BAO: Chua xac dinh toi danh"
        total_cats = sum(c["count"] for c in categories)
        if total_cats < 10:
            ai_analysis += "\n- CANH BAO: So tai lieu phan loai thap, can kiem tra"
        else:
            ai_analysis += f"\n- OK: {total_cats} tai lieu da phan loai"
        ai_analysis += "\n- KET LUAN: Ho so can review thu cong truoc khi xuat"

    # Generate report
    print("\n[DONE] Tao bao cao tong hop...")
    report = generate_report(
        case_info, timeline, categories, doc_summaries, ai_summary, ai_analysis
    )

    # Save report
    report_path = output_dir / "bao_cao_phan_tich.txt"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    with open(report_path, "w", encoding="utf-8") as f:
        f.write(report)
    print(f"  Saved: {report_path}")

    # Save structured JSON
    json_path = output_dir / "ket_qua_phan_tich.json"
    json_data = {
        "timestamp": datetime.now().isoformat(),
        "case_info": case_info,
        "timeline": timeline,
        "categories": categories,
        "documents": [
            {k: v for k, v in d.items() if k != "full_text" and k != "plain_text" and k != "pages"}
            for d in doc_summaries
        ],
        "ai_available": is_ai_available,
    }
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(json_data, f, ensure_ascii=False, indent=2)
    print(f"  Saved: {json_path}")

    # Print report
    print("\n" + report)


if __name__ == "__main__":
    main()
