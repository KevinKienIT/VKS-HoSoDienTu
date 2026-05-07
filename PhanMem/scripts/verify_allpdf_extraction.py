import sqlite3
import os
import sys

def verify_extractions():
    db = os.path.join(os.environ.get("APPDATA", ""), "com.vks.ecms", "vks-ecms.db")
    if not os.path.exists(db):
        print(f"ERROR: Database not found at {db}")
        sys.exit(1)
        
    conn = sqlite3.connect(db)
    conn.row_factory = sqlite3.Row
    
    docs = conn.execute("SELECT document_id, original_filename, page_count FROM documents WHERE original_filename LIKE '%.pdf'").fetchall()
    
    if not docs:
        print("No PDFs found in the database. Please import the test_pdfs folder via the application UI first.")
        sys.exit(0)
    
    all_ok = True
    missing_pages_docs = []
    failed_extract_docs = []

    for doc in docs:
        doc_id = doc['document_id']
        expected_pages = doc['page_count']
        
        # Check column names in pages table
        try:
            pages = conn.execute("SELECT page_id, page_index, extract_status, image_path FROM pages WHERE document_id = ?", (doc_id,)).fetchall()
        except sqlite3.OperationalError as e:
            if "no such column: extract_status" in str(e):
                pages = conn.execute("SELECT page_id, page_index, 'extracted' as extract_status, image_path FROM pages WHERE document_id = ?", (doc_id,)).fetchall()
            else:
                raise e
        
        if len(pages) != expected_pages:
            print(f"FAIL: {doc['original_filename']} - expected {expected_pages} pages, but DB has {len(pages)} records.")
            all_ok = False
            missing_pages_docs.append(doc['original_filename'])
            continue
            
        doc_failed = False
        for page in pages:
            # Check extract_status
            if page['extract_status'] != 'extracted':
                print(f"FAIL: {doc['original_filename']} page {page['page_index']} has extract_status='{page['extract_status']}'")
                doc_failed = True
                
            # Check image exists
            if not page['image_path'] or not os.path.exists(page['image_path']):
                print(f"FAIL: {doc['original_filename']} page {page['page_index']} image_path is invalid or file missing: {page['image_path']}")
                doc_failed = True
        
        if doc_failed:
            failed_extract_docs.append(doc['original_filename'])
            all_ok = False
        else:
            print(f"OK: {doc['original_filename']} ({expected_pages} pages)")
                
    if all_ok:
        print("\nPASS: All PDFs have correctly extracted pages.")
    else:
        print("\nSUMMARY OF FAILURES:")
        if missing_pages_docs:
            print("Documents with missing page records:", missing_pages_docs)
        if failed_extract_docs:
            print("Documents with missing PNG files or bad extract_status:", failed_extract_docs)
        sys.exit(1)

if __name__ == "__main__":
    verify_extractions()
