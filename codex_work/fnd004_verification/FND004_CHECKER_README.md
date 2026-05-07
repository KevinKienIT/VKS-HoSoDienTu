# FND-004 Checker README

## `check_e2e_runtime_result.py`

Command:

```powershell
cd PhanMem
python scripts/check_e2e_runtime_result.py
```

Default DB path:
- `%APPDATA%\com.vks.ecms\vks-ecms.db`
- Current resolved path: `C:\Users\KKIT\AppData\Roaming\com.vks.ecms\vks-ecms.db`

This checker reports:
- DB existence and size.
- Table counts.
- Governed event counts.
- Export JSON evidence.
- Page image extraction status.
- OCR terminal status.
- Pending/queued page rows.

Current Codex run:
- Exit code: 0.
- Final status: `PARTIAL`.
- Cause: four page rows remain pending and no export package exists.

## `check_scan_diagram_document.py`

Command:

```powershell
cd PhanMem
python scripts/check_scan_diagram_document.py --pdf test_pdfs/test_doc_1.pdf
```

This checker:
- Finds a document by `document_id`, `case_id`, or imported PDF path/name.
- Verifies page count, PNG file existence, page quality, OCR terminal status, AI summary, suggested filename, and review requirement.
- Does not import or mutate data.

Current Codex run:
- Exit code: 1.
- Result: FAIL.
- Cause: the runtime DB did not contain a document matching the fixture path `D:\JOBS\VKS-HoSoDienTu\PhanMem\test_pdfs\test_doc_1.pdf`.

## Interpretation Rule

Do not mark FND-004 PASS unless:
- The checker reads the same runtime DB as backend import logs.
- The target imported document has all page PNGs on disk.
- `extract_status` is terminal.
- OCR is terminal or has a clear failure event.
- No target pages remain pending/queued/running.

