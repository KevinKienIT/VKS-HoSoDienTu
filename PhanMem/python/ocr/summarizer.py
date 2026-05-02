"""
VKS ECMS — Document Summarizer
Tom tat noi dung ho so tu ket qua OCR.
Tier 1: Rule-based extractive summary.
Tier 2: Ollama local LLM (optional).

Usage:
    python -m ocr.summarizer --text-file <path> [--json]
"""

import json
import os
import sys
import re
from pathlib import Path
from typing import Optional, Any

def extractive_summary(text: str) -> dict:
    """
    Tier 1: Rule-based summary using regex and simple extraction.
    """
    lines = [l.strip() for l in text.split('\n') if l.strip()]
    
    # 1. Summary short/detail (first few lines)
    summary_short = " ".join(lines[:2]) if lines else "Empty document"
    summary_detail = " ".join(lines[:5]) if lines else "Empty document"
    
    # 2. Extract keywords (Names, Dates, Locations)
    # Basic regex for Vietnamese names (Capitalized words)
    # This is a very simple heuristic
    name_pattern = r'[A-ZÀÁẢÃẠÂẦẤẨẪẬĂẰẮẲẴẶÈÉẺẼẸÊỀẾỂỄỆÌÍỈĨỊÒÓỎÕỌÔỒỐỔỖỘƠỜỚỞỠỢÙÚỦŨỤƯỪỨỬỮỰỲÝỶỸỴ][a-zàáảãạâầấẩẫậăằắẳẵặèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵ]*(\s+[A-ZÀÁẢÃẠÂẦẤẨẪẬĂẰẮẲẴẶÈÉẺẼẸÊỀẾỂỄỆÌÍỈĨỊÒÓỎÕỌÔỒỐỔỖỘƠỜỚỞỠỢÙÚỦŨỤƯỪỨỬỮỰỲÝỶỸỴ][a-zàáảãạâầấẩẫậăằắẳẵặèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵ]*)+'
    names = list(set(re.findall(name_pattern, text)))
    
    date_pattern = r'\d{1,2}[/-]\d{1,2}[/-]\d{2,4}|ngày\s+\d+\s+tháng\s+\d+\s+năm\s+\d+'
    dates = list(set(re.findall(date_pattern, text, re.IGNORECASE)))
    
    # 3. Guess document type
    doc_types = {
        "bien_ban": ["biên bản", "lời khai", "hỏi cung", "đối chất"],
        "quyet_dinh": ["quyết định", "khởi tố", "phê chuẩn"],
        "to_khai": ["tờ khai", "lý lịch"],
        "ket_luan": ["kết luận", "điều tra"],
        "cao_trang": ["cáo trạng"]
    }
    
    found_type = "unknown"
    for dtype, keywords in doc_types.items():
        if any(k in text.lower() for k in keywords):
            found_type = dtype
            break
            
    # 4. Suggested name
    suggested_name = f"{found_type.upper()} "
    if dates:
        suggested_name += f"ngày {dates[0]} "
    if names:
        suggested_name += f"liên quan {names[0]}"
    
    return {
        "summary_short": summary_short,
        "summary_detail": summary_detail,
        "keywords": names[:5] + dates[:3],
        "document_type_guess": found_type,
        "suggested_name": suggested_name.strip(),
        "method": "rule-based"
    }

def ollama_summary(text: str, model: str = "qwen2.5:0.5b") -> Optional[dict]:
    """
    Tier 2: Summarize using Ollama if available.
    """
    try:
        import requests
        
        # We only send a chunk of text to avoid context limits
        truncated_text = text[:4000]
        
        prompt = f"""Tóm tắt văn bản pháp luật sau đây bằng tiếng Việt.
Trả về định dạng JSON với các trường: summary_short, summary_detail, keywords (mảng), document_type_guess, suggested_name.
Văn bản:
{truncated_text}
"""
        
        response = requests.post(
            "http://localhost:11434/api/generate",
            json={
                "model": model,
                "prompt": prompt,
                "stream": False,
                "format": "json"
            },
            timeout=30
        )
        
        if response.status_code == 200:
            data = json.loads(response.json()["response"])
            data["method"] = f"ollama ({model})"
            return data
    except Exception:
        # Graceful degrade
        pass
    return None

def process_text(text_file: Path, try_ollama: bool = True) -> dict:
    if not text_file.exists():
        raise FileNotFoundError(f"File not found: {text_file}")
        
    with open(text_file, 'r', encoding='utf-8') as f:
        text = f.read()
        
    if not text.strip():
        return {
            "summary_short": "Empty file",
            "summary_detail": "Empty file",
            "keywords": [],
            "document_type_guess": "unknown",
            "suggested_name": text_file.name,
            "method": "none"
        }
    
    result = None
    if try_ollama:
        result = ollama_summary(text)
        
    if not result:
        result = extractive_summary(text)
        
    return result

def main(args: Optional[list[str]] = None) -> None:
    if args is None:
        args = sys.argv[1:]

    if not args or "--help" in args:
        print(__doc__)
        sys.exit(0)

    text_file: Optional[Path] = None
    output_json = False

    i = 0
    while i < len(args):
        if args[i] == "--text-file" and i + 1 < len(args):
            text_file = Path(args[i + 1])
            i += 2
        elif args[i] == "--json":
            output_json = True
            i += 1
        else:
            print(f"Unknown option: {args[i]}", file=sys.stderr)
            sys.exit(1)

    if text_file is None:
        print("ERROR: --text-file is required", file=sys.stderr)
        sys.exit(1)

    try:
        result = process_text(text_file)
        if output_json:
            print(json.dumps(result, indent=2, ensure_ascii=False))
        else:
            print(f"Summary for: {text_file.name}")
            print(f"Method: {result['method']}")
            print(f"Type: {result['document_type_guess']}")
            print("-" * 20)
            print(f"Short: {result['summary_short']}")
            print(f"Detail: {result['summary_detail']}")
            print(f"Keywords: {', '.join(result['keywords'])}")
            print(f"Suggested Name: {result['suggested_name']}")
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
