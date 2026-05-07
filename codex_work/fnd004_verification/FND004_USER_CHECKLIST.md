# FND-004 User Checklist

## Manual Tauri Import Required

1. Open terminal in repo root.
2. Run:

```powershell
cd PhanMem
npm run tauri:dev
```

3. In the Tauri UI, import:

```text
PhanMem/test_pdfs/test_doc_1.pdf
```

4. Wait until import/page extraction/OCR stops showing active work.
5. Confirm the app is using:

```text
C:\Users\KKIT\AppData\Roaming\com.vks.ecms\vks-ecms.db
```

6. Run:

```powershell
python scripts/check_e2e_runtime_result.py
python scripts/check_scan_diagram_document.py --pdf test_pdfs/test_doc_1.pdf
```

## PASS Checklist

- DB exists at `%APPDATA%\com.vks.ecms\vks-ecms.db`.
- Imported document for `test_doc_1.pdf` is found.
- `documents > 0`.
- `pages > 0`.
- Every target page has non-empty `pages.image_path`.
- Every target PNG exists and size is greater than zero.
- Target pages have `extract_status='extracted'`.
- `PAGE_IMAGE_EXTRACT_DONE > 0`.
- OCR has `OCR_DONE` or `OCR_FAILED`.
- Target OCR statuses are terminal: `done`, `review_pending`, `error`, or `skipped`.
- No target page remains `pending`, `queued`, or `running`.

