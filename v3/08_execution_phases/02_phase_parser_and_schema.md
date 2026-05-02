# 02 PHASE PARSER AND SCHEMA — Chi tiết triển khai

> Phụ thuộc: Phase 1 (P0) phải PASS trước. Đóng khoảng trống parser DOCX/HTML + schema trung gian.

---

## P2-01 — DOCX Parser

### Mục tiêu
Parse file Word (.docx) thành cấu trúc heading/section/table để hiển thị + index.

### Đối chiếu code hiện tại
- `ocr_pipeline.py` chỉ xử lý ảnh/PDF scan → chưa parse DOCX.
- Python đã có trong project, có thể dùng `python-docx` package.

### Tạo `PhanMem/python/parsers/docx_parser.py`

```python
import json, sys
from docx import Document

def parse_docx(file_path: str) -> dict:
    doc = Document(file_path)
    sections = []
    current = {"heading": None, "paragraphs": [], "tables": []}

    for para in doc.paragraphs:
        if para.style.name.startswith("Heading"):
            if current["paragraphs"] or current["tables"]:
                sections.append(current)
            current = {
                "heading": para.text,
                "heading_level": int(para.style.name[-1]) if para.style.name[-1].isdigit() else 1,
                "paragraphs": [],
                "tables": [],
            }
        else:
            current["paragraphs"].append(para.text)

    for table in doc.tables:
        rows = []
        for row in table.rows:
            rows.append([cell.text for cell in row.cells])
        current["tables"].append(rows)

    sections.append(current)
    return {
        "file_path": file_path,
        "section_count": len(sections),
        "sections": sections,
    }

if __name__ == "__main__":
    result = parse_docx(sys.argv[1])
    print(json.dumps(result, ensure_ascii=False, indent=2))
```

### Gọi từ Rust (pattern hiện có trong `scan_cmd.rs`)

```rust
// Trong scan_cmd.rs hoặc doc_cmd.rs — dùng Command::new("python")
let output = std::process::Command::new("python")
    .args([
        "python/parsers/docx_parser.py",
        &file_path,
    ])
    .output()
    .map_err(|e| format!("DOCX_PARSE_FAILED: {e}"))?;
let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
    .map_err(|e| format!("DOCX_PARSE_JSON_FAILED: {e}"))?;
```

### Test
- Parse `Bao_cao_tong_hop_vu_an_LeThanhCong_D134.docx` từ `06_reference_samples/`
- Output JSON có `sections[]` với heading + paragraphs

---

## P2-02 — HTML Parser

### Mục tiêu
Parse HTML mẫu (sơ đồ vụ án, timeline, quan hệ) thành structured data.

### Đối chiếu
- File mẫu: `v3/06_reference_samples/So_do_vu_an_LeThanhCong_D134.html` (25KB)

### Tạo `PhanMem/python/parsers/html_parser.py`

```python
import json, sys
from html.parser import HTMLParser

class VKSHTMLParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.cards = []
        self.tables = []
        self.timeline_events = []
        self._current = None

    def handle_starttag(self, tag, attrs):
        attrs_dict = dict(attrs)
        cls = attrs_dict.get("class", "")
        if "card" in cls or "timeline-item" in cls:
            self._current = {"tag": tag, "class": cls, "text": ""}

    def handle_data(self, data):
        if self._current:
            self._current["text"] += data.strip()

    def handle_endtag(self, tag):
        if self._current and self._current["tag"] == tag:
            if "timeline" in self._current["class"]:
                self.timeline_events.append(self._current)
            else:
                self.cards.append(self._current)
            self._current = None

def parse_html(file_path: str) -> dict:
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()
    parser = VKSHTMLParser()
    parser.feed(content)
    return {
        "file_path": file_path,
        "cards": parser.cards,
        "tables": parser.tables,
        "timeline_events": parser.timeline_events,
    }

if __name__ == "__main__":
    print(json.dumps(parse_html(sys.argv[1]), ensure_ascii=False, indent=2))
```

---

## P2-03 — Intermediate Schema

### Mục tiêu
Map output của DOCX/HTML parser vào DB dạng chuẩn.

### Migration `009_parsed_sections.sql`

```sql
CREATE TABLE IF NOT EXISTS document_sections (
    id              TEXT PRIMARY KEY,
    document_id     TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    section_type    TEXT NOT NULL CHECK (section_type IN (
        'heading','paragraph','table','timeline_event','card','relation'
    )),
    heading         TEXT,
    heading_level   INTEGER DEFAULT 1,
    content_text    TEXT,
    content_json    TEXT,  -- raw structured data
    section_order   INTEGER NOT NULL DEFAULT 0,
    source_parser   TEXT NOT NULL DEFAULT 'docx', -- 'docx' | 'html' | 'ocr'
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_sections_doc ON document_sections(document_id);
```

---

## P2-04 — Pre-export Validator

### Mục tiêu
Chặn export nếu thiếu mục lục/timeline/căn cứ pháp lý.

### Sửa `export_cmd.rs`

```rust
fn validate_before_export(conn: &Connection, case_id: &str) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();

    // Check 1: Có ít nhất 1 document managed_ready
    let managed_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE case_id=?1 AND file_status='managed_ready'",
        [case_id], |r| r.get(0)
    ).unwrap_or(0);
    if managed_count == 0 {
        warnings.push("Không có tài liệu nào ở trạng thái 'managed_ready'".into());
    }

    // Check 2: Có extracted fields (bút lục, ngày)
    let field_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM document_extracted_fields ef
         JOIN documents d ON d.document_id = ef.document_id
         WHERE d.case_id = ?1",
        [case_id], |r| r.get(0)
    ).unwrap_or(0);
    if field_count == 0 {
        warnings.push("Chưa có trường trích xuất (bút lục, ngày, số VB)".into());
    }

    Ok(warnings)
}
```

### Test
- Case rỗng → export bị chặn với warning
- Case đủ data → export chạy bình thường
