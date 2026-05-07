# All-PDF Extraction Runtime Report

## Scope

- Branch target: `agent/roo/fnd-allpdf-extraction-runtime`
- Runtime app command: `cd PhanMem && npm run tauri:dev`
- Prepared PDF folder: `PhanMem/test_pdfs/allpdf_runtime/`
- Verification script used: `PhanMem/scripts/verify_allpdf_extraction.py`
- Main evidence logs:
  - `roo_work/allpdf_extraction_verify.log`
  - `roo_work/allpdf_extraction_detailed_pages.json`
  - `roo_work/allpdf_extraction_latest_docs.json`

## Commands run

```cmd
dir /s /b *.pdf
```

Purpose: locate available PDF fixtures in the workspace.

```cmd
if not exist "PhanMem\test_pdfs\allpdf_runtime" mkdir "PhanMem\test_pdfs\allpdf_runtime" && copy /Y "TAI LIEU\test\1.pdf" "PhanMem\test_pdfs\allpdf_runtime\1.pdf" && copy /Y "TAI LIEU\test\2.pdf" "PhanMem\test_pdfs\allpdf_runtime\2.pdf" && copy /Y "TAI LIEU\test\3.pdf" "PhanMem\test_pdfs\allpdf_runtime\3.pdf" && copy /Y "TAI LIEU\test\4.pdf" "PhanMem\test_pdfs\allpdf_runtime\4.pdf" && copy /Y "TAI LIEU\test\5.pdf" "PhanMem\test_pdfs\allpdf_runtime\5.pdf" && dir "PhanMem\test_pdfs\allpdf_runtime"
```

Purpose: prepare a runtime folder containing 5 PDF fixtures.

Prepared files:

| File                                     |            Size |
| ---------------------------------------- | --------------: |
| `PhanMem/test_pdfs/allpdf_runtime/1.pdf` |   433,967 bytes |
| `PhanMem/test_pdfs/allpdf_runtime/2.pdf` | 7,574,645 bytes |
| `PhanMem/test_pdfs/allpdf_runtime/3.pdf` |   995,862 bytes |
| `PhanMem/test_pdfs/allpdf_runtime/4.pdf` | 4,189,216 bytes |
| `PhanMem/test_pdfs/allpdf_runtime/5.pdf` | 9,996,265 bytes |

```cmd
cd PhanMem && npm run tauri:dev
```

Purpose: launch the Tauri desktop app for runtime import verification.

```cmd
python scripts\verify_allpdf_extraction.py > ..\roo_work\allpdf_extraction_verify.log 2>&1 & type ..\roo_work\allpdf_extraction_verify.log
```

Purpose: verify all PDF documents currently present in the runtime SQLite database.

```cmd
python -c "import os,sqlite3,json; db=os.path.join(os.environ['APPDATA'],'com.vks.ecms','vks-ecms.db'); con=sqlite3.connect(db); con.row_factory=sqlite3.Row; rows=con.execute('select document_id,case_id,original_filename,page_count,status,created_at,updated_at from documents order by created_at desc, rowid desc limit 20').fetchall(); print(json.dumps([dict(r) for r in rows], ensure_ascii=False, indent=2))" > roo_latest_docs.json && type roo_latest_docs.json
```

Purpose: collect latest document metadata from runtime DB.

```cmd
python -c "import os,sqlite3,json; db=os.path.join(os.environ['APPDATA'],'com.vks.ecms','vks-ecms.db'); con=sqlite3.connect(db); con.row_factory=sqlite3.Row; rows=con.execute('''select d.document_id,d.original_filename,d.page_count,p.page_id,p.page_index,coalesce(p.extract_status,'') extract_status,p.image_path,case when p.image_path is not null and p.image_path <> '' then 1 else 0 end has_image_path from documents d left join pages p on p.document_id=d.document_id where lower(d.original_filename) like '%.pdf' order by d.created_at desc,d.document_id,p.page_index''').fetchall(); print(json.dumps([dict(r) for r in rows], ensure_ascii=False, indent=2))" > ..\roo_work\allpdf_extraction_detailed_pages.json && type ..\roo_work\allpdf_extraction_detailed_pages.json
```

Purpose: collect page-level status, `extract_status`, and PNG path evidence.

```cmd
python -c "import os,sqlite3; db=os.path.join(os.environ['APPDATA'],'com.vks.ecms','vks-ecms.db'); con=sqlite3.connect(db); print(con.execute(\"select count(*) from documents where file_path like '%allpdf_runtime%' or original_filename in ('1.pdf','2.pdf','3.pdf','4.pdf','5.pdf')\").fetchone()[0])"
```

Purpose: check whether the 5 prepared fixture filenames/folder path appeared in the runtime database.

```cmd
git switch -c agent/roo/fnd-allpdf-extraction-runtime
```

Purpose: create the requested working branch.

## Runtime observations

### Tauri startup

`npm run tauri:dev` compiled and launched `target\debug\vks-ecms.exe`, but the Vite frontend reported repeated dependency-resolution errors:

- `react-pdf` could not be resolved from `PhanMem/src/modules/phantichtailieu/AnhThuNho.tsx`.
- `react-pdf` could not be resolved from `PhanMem/src/modules/phantichtailieu/TrinhXemTaiLieu.tsx`.
- `pdfjs-dist/build/pdf.worker.min.mjs?url` and `react-pdf` CSS imports were also reported unresolved.

This is a runtime/frontend dependency issue observed during the requested Tauri run. The backend executable still launched and the runtime DB was available for verification queries.

### UI import note

Manual UI feedback reported that import/OCR completed successfully for all 5 PDFs in `PhanMem/test_pdfs/allpdf_runtime/`.

However, DB validation found **0 documents** whose `file_path` contained `allpdf_runtime` or whose `original_filename` matched `1.pdf` through `5.pdf`. Therefore, the runtime DB evidence does **not** confirm that the prepared 5-PDF folder was imported into SQLite during this run.

## Verification result

Overall result: **FAIL / inconclusive for requested all-PDF folder**.

Reason:

1. The checker found existing PDF documents in the runtime DB, but not the prepared 5 PDFs from `PhanMem/test_pdfs/allpdf_runtime/`.
2. Among existing runtime PDF documents, one document has all 4 pages stuck at `extract_status='pending'` with missing `image_path`.
3. The requirement “no page pending or extract_status other than extracted” is not satisfied by the current runtime DB.

## Failing document/page details

From `roo_work/allpdf_extraction_verify.log` and `roo_work/allpdf_extraction_detailed_pages.json`:

| Document ID           | Original filename                                                                                 | Page | Page ID                 | `extract_status` | `image_path` |
| --------------------- | ------------------------------------------------------------------------------------------------- | ---: | ----------------------- | ---------------- | ------------ |
| `doc-1778129491670-6` | `001_khong_xac_dinh_Chua_ro_ngay_Chua_ro_co_quan_Import_nhi_u_file_VK_1778129491582_0002_pdf.pdf` |    1 | `page-1778129550316-7`  | `pending`        | `null`       |
| `doc-1778129491670-6` | `001_khong_xac_dinh_Chua_ro_ngay_Chua_ro_co_quan_Import_nhi_u_file_VK_1778129491582_0002_pdf.pdf` |    2 | `page-1778129550321-9`  | `pending`        | `null`       |
| `doc-1778129491670-6` | `001_khong_xac_dinh_Chua_ro_ngay_Chua_ro_co_quan_Import_nhi_u_file_VK_1778129491582_0002_pdf.pdf` |    3 | `page-1778129550324-11` | `pending`        | `null`       |
| `doc-1778129491670-6` | `001_khong_xac_dinh_Chua_ro_ngay_Chua_ro_co_quan_Import_nhi_u_file_VK_1778129491582_0002_pdf.pdf` |    4 | `page-1778129550328-13` | `pending`        | `null`       |

## Passing document/page details

| Document ID           | Original filename                                                                                 | Expected pages | DB page records | Status                                                      |
| --------------------- | ------------------------------------------------------------------------------------------------- | -------------: | --------------: | ----------------------------------------------------------- |
| `doc-1778086172251-2` | `001_khong_xac_dinh_Chua_ro_ngay_Chua_ro_co_quan_Import_nhi_u_file_VK_1778086172222_0001_pdf.pdf` |              2 |               2 | PASS: pages have `extract_status='extracted'` and PNG paths |

## Conclusion

- The prepared multi-PDF folder exists at `PhanMem/test_pdfs/allpdf_runtime/` with 5 PDFs.
- The Tauri runtime was launched with `npm run tauri:dev`, but frontend dependency errors for `react-pdf` were observed.
- The runtime DB did not show records for the 5 prepared fixture PDFs.
- The checker failed on an existing 4-page PDF document because all pages remain pending and have no PNG path.
- Detailed failure logs were written under `roo_work/` for follow-up.
