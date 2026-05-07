# FND-004 Runtime Commands

Run from repository root unless noted.

## Standard Command Sequence

```powershell
cd PhanMem
cargo test
npx tsc --noEmit
npm run tauri:dev
```

Manual runtime action:
- In the Tauri app, import `test_pdfs/test_doc_1.pdf`.
- Wait for import/page extraction/OCR to reach terminal status.

Then run:

```powershell
python scripts/check_e2e_runtime_result.py
python scripts/check_scan_diagram_document.py --pdf test_pdfs/test_doc_1.pdf
```

## Local Note

If `cargo test` from `PhanMem/` reports that no `Cargo.toml` exists, use the Rust manifest path:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Do not run Vite standalone for runtime verification.

