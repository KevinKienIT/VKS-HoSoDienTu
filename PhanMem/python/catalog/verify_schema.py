import sqlite3
from pathlib import Path

conn = sqlite3.connect(":memory:")

repo_root = Path(__file__).resolve().parents[3]
schema_path = repo_root / "PhanMem" / "src-tauri" / "migrations" / "001_init_schema.sql"

with schema_path.open("r", encoding="utf-8") as f:
    sql = f.read()

conn.executescript(sql)

tables = conn.execute(
    "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
).fetchall()

print(f"Tables created: {len(tables)}")
for t in tables:
    print(f"  - {t[0]}")

# Count indexes
indexes = conn.execute(
    "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY name"
).fetchall()
print(f"\nIndexes created: {len(indexes)}")

conn.close()
print("\nSQL schema validation: OK")
