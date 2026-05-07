# FND-006 Viewer PNG-First Code Review

## Task
- Task ID: FND-006
- Title: Verify Viewer PNG-first
- Owner: Codex GPT-5.5
- Date: 2026-05-07

## V4 Context Loaded
- README_V4.md: yes
- PROJECT_SUMMARY.md: yes
- AGENT_RULES.md: yes
- CURRENT_ARCHITECTURE.md: yes
- ACTIVE_TASKS.md: yes

## Grapuco Extension Status
- Running in Antigravity/Gravity: no direct extension access in this non-interactive terminal session
- Repository sync status: unknown from this session
- CLI required: NO - Grapuco is a VS Code extension, not CLI
- Code graph reviewed: GRAPUCO_EXTENSION_UNAVAILABLE_IN_SESSION
- Fallback used: MANUAL_CODE_GRAPH_FALLBACK procedure with `rg` and read-only source inspection

## FND-004 / FND-005 Runtime Context
- FND-004 report says `test_doc_1.pdf` has 2 extracted PNG pages with non-empty `pages.image_path`, non-empty `thumbnail_path`, existing PNG files, and `PAGE_IMAGE_EXTRACT_DONE`.
- FND-005 report says target OCR is terminal as `review_pending` for both target pages and `OCR_DONE` payload records `ocr_source=stored_page_image`.
- Viewer review does not depend on FND-005. OCR terminal state is recorded only as context; PNG rendering should depend on page image availability, not OCR text completion.
- Global DB still has stale pending rows for another document, but that is out of scope for this viewer review.

## Grapuco / Manual Findings
- Feature query: viewer PNG-first, PDF fallback removal, thumbnail source, page image DB flow.
- Related files:
  - `PhanMem/src/modules/phantichtailieu/TrinhXemTaiLieu.tsx`
  - `PhanMem/src/modules/phantichtailieu/AnhThuNho.tsx`
  - `PhanMem/src/modules/phantichtailieu/ChiTietXemTaiLieuPage.tsx`
  - `PhanMem/src/modules/quantailieu/quantailieu.service.ts`
  - `PhanMem/src/components/DocumentViewer.tsx`
  - `PhanMem/src/components/PdfPageThumbnail.tsx`
  - `PhanMem/src/modules/hosovuan/ChiTietVuAnPage.tsx`
  - `PhanMem/src-tauri/src/commands/doc_cmd.rs`
- Related functions:
  - `DocumentViewer` in `TrinhXemTaiLieu.tsx`
  - `PdfPageThumbnail` in `AnhThuNho.tsx`
  - `getPageOcr()` and `readImageBase64()` in `quantailieu.service.ts`
  - `get_page_ocr()` and `read_image_base64()` in `doc_cmd.rs`
  - Legacy `DocumentViewer` in `src/components/DocumentViewer.tsx`
  - Legacy `PdfPageThumbnail` in `src/components/PdfPageThumbnail.tsx`
- Callers:
  - Main viewer route: `App.tsx` routes `/cases/:caseId/docs/:docId` to `ChiTietXemTaiLieuPage`.
  - `ChiTietXemTaiLieuPage` passes `document.file_path`, `document.file_path` extension, and `document.document_id` into `TrinhXemTaiLieu.DocumentViewer`.
  - `DanhSachTaiLieuPage`, `KhongGianAIPage`, and `TimKiemPage` use the PNG-first module viewer/thumbnail.
  - `ChiTietVuAnPage` still imports `PdfPageThumbnail` from `../../components/PdfPageThumbnail`, which is legacy PDF.js based.
  - Backup pages under `src/components/_backup_pages_v2/` still reference legacy components but are not active route targets in current `App.tsx`.
- Callees:
  - `TrinhXemTaiLieu.tsx` calls `documentService.getPageOcr(documentId, page)` and `documentService.readImageBase64(image_path)`.
  - `AnhThuNho.tsx` calls `getPageOcr(documentId, pageNumber)`, reads `thumbnail_path || image_path`, then calls `readImageBase64`.
  - Legacy components call `readFile()` and render `react-pdf` `<Document>` / `<Page>`.
- DB tables touched:
  - Frontend does not write DB in viewer path.
  - Backend `get_page_ocr()` reads `pages.image_path`, `pages.thumbnail_path`, `pages.source_pdf_path`, `pages.source_page_number`, `pages.current_order`, OCR fields, and joins `ocr_results`.
- Governed events touched:
  - Viewer path does not emit governed events.
- UI modules affected:
  - `phantichtailieu`, `quantailieu`, `phantichai`, `timkiem`, `hosovuan`.
- Scripts/checkers affected:
  - No checker change required for this review.
  - Suggested future verification should include visual/manual Tauri viewer check after code changes.

## Impact Analysis
- Files expected to change:
  - `PhanMem/src/modules/phantichtailieu/TrinhXemTaiLieu.tsx`
  - `PhanMem/src/modules/phantichtailieu/ChiTietXemTaiLieuPage.tsx`
  - `PhanMem/src/modules/phantichai/KhongGianAIPage.tsx`
  - `PhanMem/src/modules/quantailieu/DanhSachTaiLieuPage.tsx`
  - `PhanMem/src/modules/hosovuan/ChiTietVuAnPage.tsx`
  - `PhanMem/src/modules/phantichtailieu/AnhThuNho.tsx`
  - Optional cleanup or quarantine: `PhanMem/src/components/DocumentViewer.tsx`, `PhanMem/src/components/PdfPageThumbnail.tsx`
- Functions expected to change:
  - `DocumentViewer` props/data loading in `TrinhXemTaiLieu.tsx`
  - `PdfPageThumbnail` import/call sites for case detail hover preview
  - Optional legacy component removal or conversion
- Callers impacted:
  - Viewer route, document preview modal, AI workspace preview, case detail hover preview.
- Callees impacted:
  - Existing `getPageOcr` and `readImageBase64` APIs are sufficient; no backend API change required for a minimal fix.
- DB impact:
  - None expected. Read-only use of existing `pages.image_path` and `pages.thumbnail_path`.
- UI impact:
  - Main PDF viewer should render stored PNG images only.
  - Thumbnail previews should use `thumbnail_path` first, then `image_path`.
  - Missing PNG should show explicit PAGE_IMAGE_REQUIRED/PAGE_IMAGE_MISSING state, not PDF.js fallback.
- Export impact:
  - None.
- Test impact:
  - TypeScript compile must pass.
  - Tauri dev manual verification required with document `doc-1778086172251-2`.
- Risk level: MEDIUM

## Current Behavior Observed

### Main `TrinhXemTaiLieu.tsx`
- This file is already mostly PNG-first.
- It does not import `react-pdf`, `readFile`, or `convertFileSrc`.
- For PDFs, it enters loading while page image rows are fetched.
- If no page image row is found, it displays `PAGE_IMAGE_REQUIRED` or `PAGE_IMAGE_MISSING` and explicitly says it does not render PDF fallback.
- If `hasPageImage` is true, it reads `currentPageImage.image_path` through `readImageBase64()` and renders an `<img>`.
- Thumbnail rendering delegates to `./AnhThuNho`, passing `documentId`.

### Conditions That Still Use PDF Bytes
- `PhanMem/src/components/DocumentViewer.tsx` uses PDF bytes when `isPdf` is true:
  - `isPdf = documentType.includes("pdf") || documentPath.endsWith(".pdf")`
  - then `readFile(documentPath)` loads bytes into `pdfData`
  - then `react-pdf` renders `<Document file={{ data: pdfData }}>` and `<Page>`.
- `PhanMem/src/components/PdfPageThumbnail.tsx` uses PDF bytes when `isPdf` is true:
  - `readFile(filePath)` loads bytes into `pdfData`
  - then `react-pdf` renders thumbnail `<Document>` and `<Page>`.
  - It also references remote PDF.js cMap/font URLs, which conflicts with offline runtime rules if this path runs.
- Active current-module users mostly switched to `../phantichtailieu/TrinhXemTaiLieu` and `../phantichtailieu/AnhThuNho`.
- `PhanMem/src/modules/hosovuan/ChiTietVuAnPage.tsx` still imports the legacy `../../components/PdfPageThumbnail`, so case-detail hover preview can still read PDF bytes instead of stored PNG.

### How `image_path` And `thumbnail_path` Are Loaded
- Backend `doc_cmd::get_page_ocr()` selects:
  - `p.image_path`
  - `p.thumbnail_path`
  - `p.source_pdf_path`
  - `p.source_page_number`
  - `p.current_order`
  - OCR/page quality fields
- Frontend `quantailieu.service.ts` exposes those fields in `PageOcrResult`.
- `quantailieu.service.getPageOcr(documentId, pageIndex)` invokes Tauri command `get_page_ocr`.
- `quantailieu.service.readImageBase64(imagePath)` invokes Tauri command `read_image_base64`.
- Backend `read_image_base64()` reads a local image path and returns a MIME data URL. This bypasses disabled asset protocol.

### `pageImages` Data Flow
- `ChiTietXemTaiLieuPage` passes `documentId={document.document_id}` into `TrinhXemTaiLieu.DocumentViewer`.
- `TrinhXemTaiLieu` stores `pageImages` as `Record<number, PageOcrResult>`.
- On load, it computes `maxPage = Math.max(numPages ?? initialPage, initialPage, 1)`.
- It calls `getPageOcr(documentId, page)` for pages `1..maxPage`.
- It keeps only rows where `item?.image_path` is truthy.
- It stores rows keyed by `item.page_index`.
- `currentPageImage = pageImages[pageNumber]`.
- `hasPageImage = Boolean(currentPageImage?.image_path)`.
- It reads `currentPageImage.image_path` with `readImageBase64()` into `imageDataUrls[pageNumber]`.
- It preloads adjacent page images from the same `pageImages` map.
- It renders `<img src={imageDataUrls[pageNumber]}>` for the main page.
- `AnhThuNho` independently calls `getPageOcr(documentId, pageNumber)` and uses `thumbnail_path || image_path` for the thumbnail, with hover loading from `image_path`.

### Current Risk / Bug
- `TrinhXemTaiLieu` does not receive `document.page_count`.
- On first load with `initialPage=1` and `numPages` undefined, `maxPage` is 1, so it can load only page 1 for a multi-page PDF.
- It then sets `numPages` to the max loaded key, which remains 1, preventing normal navigation to page 2+ unless the URL initial page is already higher.
- FND-004 proves two PNG pages exist for `test_doc_1.pdf`, so viewer verification should check whether both pages are reachable from default page 1.

## Target Behavior (V4 Spec)
- PDF viewer never reads original PDF bytes in production.
- Main PDF page display uses stored PNG from `pages.image_path`.
- Thumbnail display uses `pages.thumbnail_path` first and `pages.image_path` as fallback.
- Missing page image is a terminal UI state requiring extraction, not a PDF.js fallback.
- Viewer should use all pages for the document, including multi-page docs opened at page 1.
- OCR completion is not required for PNG rendering. `review_pending`, `done`, `error`, or still-running OCR should not block PNG display if `image_path` exists.

## Expected Diff (Describe)
```diff
// PhanMem/src/modules/phantichtailieu/TrinhXemTaiLieu.tsx
+ add optional prop pageCount?: number
+ compute maxPage from pageCount before falling back to numPages/initialPage
+ set numPages from pageCount when available, not only from loaded image rows
+ keep PAGE_IMAGE_REQUIRED/PAGE_IMAGE_MISSING behavior for missing image_path
+ keep PNG <img> rendering through readImageBase64

// PhanMem/src/modules/phantichtailieu/ChiTietXemTaiLieuPage.tsx
+ pass pageCount={document.page_count} into DocumentViewer

// PhanMem/src/modules/quantailieu/DanhSachTaiLieuPage.tsx
+ pass pageCount={previewDoc.page_count} into DocumentViewer preview

// PhanMem/src/modules/phantichai/KhongGianAIPage.tsx
+ pass pageCount={selectedDoc.page_count} into DocumentViewer preview

// PhanMem/src/modules/hosovuan/ChiTietVuAnPage.tsx
- import { PdfPageThumbnail } from "../../components/PdfPageThumbnail";
+ import { PdfPageThumbnail } from "../phantichtailieu/AnhThuNho";
+ pass documentId={doc.document_id} to thumbnail preview

// Optional cleanup
- retire or quarantine src/components/DocumentViewer.tsx and src/components/PdfPageThumbnail.tsx from runtime imports
+ if kept, make them explicit legacy/backup only or convert them to PNG-first with documentId
```

## Forbidden Changes (Checklist)
- [x] No unrelated refactor
- [x] No runtime model download
- [x] No assetProtocol
- [x] No system Python fallback
- [x] No Docling runtime integration
- [x] No deletion outside scope
- [x] No include python_embedded/** in Tauri resources

## Test Gate
- Commands:
  - `cd PhanMem && npx tsc --noEmit`
  - `cd PhanMem && npm run tauri:dev`
- Expected result:
  - TypeScript compiles.
  - Opening `doc-1778086172251-2` shows PNG page 1 and page 2 from `pages.image_path`.
  - No `readFile(documentPath)` call is needed for PDF viewing in active viewer path.
  - Thumbnail preview in viewer and case detail uses `thumbnail_path` or `image_path`.
  - Missing PNG shows `PAGE_IMAGE_REQUIRED` or `PAGE_IMAGE_MISSING`, not PDF.js fallback.
- Manual verification required:
  - Open `/cases/:caseId/docs/doc-1778086172251-2`.
  - Navigate page 1 to page 2 from default page 1.
  - Confirm visible image is PNG content.
  - Confirm no UI path opens PDF via legacy `src/components/DocumentViewer.tsx`.

## Decision
- APPROVED_TO_EDIT: no
- BLOCKED_REASON: Review-only task; user explicitly requested no code edit in this step.
- HANDOFF_REQUIRED: Implement expected diff in FND-006 runtime edit step after approval.

## Files Created By This Review
- `v4/07_REPORTS/FND_006_VIEWER_PNG_FIRST_REVIEW.md`

## Runtime Code Touched
- NO
