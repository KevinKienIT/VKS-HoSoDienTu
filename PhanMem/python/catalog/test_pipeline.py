"""Quick end-to-end test: schema → insert → query → cleanup."""
import os
import sqlite3
import tempfile
import uuid
from datetime import datetime, timezone
from pathlib import Path

schema_path = Path(__file__).resolve().parents[2] / "src-tauri" / "migrations" / "001_init_schema.sql"
sql = schema_path.read_text(encoding="utf-8")

tmp = os.path.join(tempfile.gettempdir(), "vks_test_agent_b.db")
conn = sqlite3.connect(tmp)
conn.executescript(sql)

required = [
    "cases", "documents", "pages", "ocr_results", "review_queue",
    "import_jobs", "audit_events", "entities", "citations",
    "users", "work_products", "catalog_entries",
]
tables = [r[0] for r in conn.execute(
    "SELECT name FROM sqlite_master WHERE type='table'"
).fetchall()]

all_ok = True
for t in required:
    ok = t in tables
    print(f"  {t}: {'OK' if ok else 'MISSING'}")
    if not ok:
        all_ok = False

idx_count = conn.execute(
    "SELECT count(1) FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'"
).fetchone()[0]
print(f"\nTotal custom indexes: {idx_count}")

# Insert test catalog entry
now = datetime.now(timezone.utc).isoformat()
conn.execute(
    """INSERT INTO catalog_entries
       (catalog_entry_id, file_path, file_name, file_ext,
        file_size, modified_at, file_hash, parent_folder,
        scan_status, scanned_at)
       VALUES (?,?,?,?,?,?,?,?,?,?)""",
    (str(uuid.uuid4()), "/test/path.pdf", "path.pdf", ".pdf",
     1024, now, "abc123hash", "test", "discovered", now),
)
conn.commit()
row_count = conn.execute("SELECT count(1) FROM catalog_entries").fetchone()[0]
print(f"Catalog insert test: {row_count} row(s)")

conn.close()
os.remove(tmp)

if all_ok and idx_count >= 30 and row_count == 1:
    print("\nFull pipeline test: PASS")
else:
    print("\nFull pipeline test: FAIL")
    raise SystemExit(1)
