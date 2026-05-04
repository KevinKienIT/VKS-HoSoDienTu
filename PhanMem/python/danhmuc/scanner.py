"""
VKS ECMS — Lightweight Catalog Scanner

Quet nhanh metadata cua cac file PDF/image trong thu muc ho so.
Khong thuc hien OCR, khong convert PDF.
Chi doc: path, filename, size, modified_at, hash (SHA-256).

Usage:
    python -m catalog.scanner <folder_path> [--db <sqlite_path>] [--json]

Output:
    - In danh sach file + metadata ra stdout (default)
    - Hoac ghi vao bang catalog_entries trong SQLite (--db)
    - Hoac xuat JSON (--json)
"""

import hashlib
import json
import os
import sqlite3
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional


# Extensions duoc coi la tai lieu hop le
VALID_EXTENSIONS = {".pdf", ".png", ".jpg", ".jpeg", ".tif", ".tiff", ".bmp"}


def compute_sha256(file_path: Path, chunk_size: int = 65536) -> str:
    """Tinh SHA-256 hash cua file."""
    h = hashlib.sha256()
    with open(file_path, "rb") as f:
        while True:
            chunk = f.read(chunk_size)
            if not chunk:
                break
            h.update(chunk)
    return h.hexdigest()


def scan_folder(folder: Path, compute_hash: bool = True) -> list[dict]:
    """
    Quet folder de lay metadata cua tat ca file hop le.

    Returns:
        list of dict voi cac truong:
        - catalog_entry_id (UUID)
        - file_path (absolute)
        - file_name
        - file_ext
        - file_size (bytes)
        - modified_at (ISO 8601)
        - file_hash (SHA-256, hoac None neu compute_hash=False)
        - parent_folder (ten folder cha truc tiep)
        - scanned_at (ISO 8601)
    """
    results = []
    folder = folder.resolve()

    if not folder.is_dir():
        print(f"ERROR: {folder} khong phai la thu muc", file=sys.stderr)
        return results

    for root, _dirs, files in os.walk(folder):
        for fname in sorted(files):
            fpath = Path(root) / fname
            ext = fpath.suffix.lower()

            if ext not in VALID_EXTENSIONS:
                continue

            try:
                stat = fpath.stat()
                mtime = datetime.fromtimestamp(stat.st_mtime, tz=timezone.utc)
                file_hash = compute_sha256(fpath) if compute_hash else None
            except OSError as exc:
                print(f"WARN: Khong doc duoc file {fpath}: {exc}", file=sys.stderr)
                continue

            entry = {
                "catalog_entry_id": str(uuid.uuid4()),
                "file_path": str(fpath),
                "file_name": fname,
                "file_ext": ext,
                "file_size": stat.st_size,
                "modified_at": mtime.isoformat(),
                "file_hash": file_hash,
                "parent_folder": Path(root).name,
                "scan_status": "discovered",
                "scan_note": None,
                "scanned_at": datetime.now(timezone.utc).isoformat(),
            }
            results.append(entry)

    return results


def save_to_db(entries: list[dict], db_path: Path) -> int:
    """
    Ghi catalog entries vao SQLite.
    Dung INSERT OR IGNORE de tranh duplicate theo file_path.

    Returns:
        So dong da insert thanh cong.
    """
    conn = sqlite3.connect(str(db_path))
    inserted = 0
    try:
        for entry in entries:
            try:
                cursor = conn.execute(
                    """INSERT OR IGNORE INTO catalog_entries
                       (catalog_entry_id, file_path, file_name, file_ext,
                        file_size, modified_at, file_hash, parent_folder,
                        scan_status, scan_note, scanned_at)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    (
                        entry["catalog_entry_id"],
                        entry["file_path"],
                        entry["file_name"],
                        entry["file_ext"],
                        entry["file_size"],
                        entry["modified_at"],
                        entry["file_hash"],
                        entry["parent_folder"],
                        entry["scan_status"],
                        entry["scan_note"],
                        entry["scanned_at"],
                    ),
                )
                inserted += cursor.rowcount
            except sqlite3.IntegrityError:
                pass
            except sqlite3.Error as exc:
                print(
                    f"WARN: Khong ghi duoc catalog entry {entry.get('file_path')}: {exc}",
                    file=sys.stderr,
                )
        conn.commit()
    finally:
        conn.close()
    return inserted


def main(args: Optional[list[str]] = None) -> None:
    """CLI entry point."""
    if args is None:
        args = sys.argv[1:]

    if not args:
        print(__doc__)
        sys.exit(1)

    folder = Path(args[0])
    db_path: Optional[Path] = None
    output_json = False

    i = 1
    while i < len(args):
        if args[i] == "--db" and i + 1 < len(args):
            db_path = Path(args[i + 1])
            i += 2
        elif args[i] == "--json":
            output_json = True
            i += 1
        else:
            print(f"Unknown option: {args[i]}", file=sys.stderr)
            sys.exit(1)

    print(f"Scanning: {folder}")
    entries = scan_folder(folder)
    print(f"Found {len(entries)} files")

    if db_path:
        inserted = save_to_db(entries, db_path)
        print(f"Inserted {inserted} entries into {db_path}")
    elif output_json:
        print(json.dumps(entries, indent=2, ensure_ascii=False))
    else:
        for entry in entries:
            size_kb = entry["file_size"] / 1024
            print(f"  {entry['file_name']:40s}  {size_kb:>8.1f} KB  {entry['parent_folder']}")


if __name__ == "__main__":
    main()
