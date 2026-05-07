"""Deep query for FND-005 OCR Terminal evidence specifically for test_doc_1.pdf."""
import sqlite3
import json
import os

db = os.path.join(os.environ["APPDATA"], "com.vks.ecms", "vks-ecms.db")
conn = sqlite3.connect(db)
conn.row_factory = sqlite3.Row

# Find target document test_doc_1.pdf
print("=== TARGET DOCUMENT ===")
doc = conn.execute(
    "SELECT document_id, original_filename, managed_path, file_status FROM documents WHERE document_id = 'doc-1778086172251-2'"
).fetchone()

if not doc:
    print("ERROR: test_doc_1.pdf not found in documents.")
    conn.close()
    exit(1)

doc_id = doc["document_id"]
print(f"doc_id: {doc_id}")
print(f"filename: {doc['original_filename']}")
print(f"file_status: {doc['file_status']}")
print()

print("=== TARGET PAGES ===")
pages = conn.execute(
    "SELECT page_id, page_index, ocr_status, extract_status FROM pages WHERE document_id = ? ORDER BY page_index",
    (doc_id,)
).fetchall()

total_pages = len(pages)
pending_pages = 0
terminal_pages = 0

for p in pages:
    d = dict(p)
    status = d["ocr_status"]
    if status in ("pending", "queued", "running"):
        pending_pages += 1
    elif status in ("done", "review_pending", "error"):
        terminal_pages += 1
    print(f"page {d['page_index']}: ocr_status={status} (extract={d['extract_status']})")

print(f"\ntarget_total: {total_pages}, target_pending: {pending_pages}, target_terminal: {terminal_pages}")

print("\n=== TARGET OCR RESULTS ===")
for p in pages:
    page_id = p["page_id"]
    ocr_results = conn.execute(
        "SELECT engine, confidence, length(raw_text) as text_len FROM ocr_results WHERE page_id = ?",
        (page_id,)
    ).fetchall()
    if not ocr_results:
        print(f"page {p['page_index']}: NO OCR RESULTS")
    for r in ocr_results:
        print(f"page {p['page_index']} result: engine={r['engine']} conf={r['confidence']} text_len={r['text_len']}")

print("\n=== TARGET EVENTS ===")
events = conn.execute(
    "SELECT event_type, phase, payload_json, created_at FROM governed_events "
    "WHERE event_type LIKE 'OCR%' AND payload_json LIKE ? ORDER BY created_at ASC",
    (f"%{doc_id}%",)
).fetchall()

for e in events:
    d = dict(e)
    try:
        payload = json.loads(d["payload_json"])
        # Check if ocr_source exists
        ocr_source = payload.get("ocr_source", "N/A")
        print(f"event: {d['event_type']} phase={d['phase']} at {d['created_at']} | ocr_source={ocr_source}")
    except:
        print(f"event: {d['event_type']} phase={d['phase']} at {d['created_at']} | raw={d['payload_json'][:100]}")

conn.close()
print("\nDONE")
