# Log 2026-05-04 — Phase MVP P0: Bug Audit & Migration

## Trang thai: DONE

## Muc tieu
Kiem tra toan bo specs vs code, phat hien bugs, sua loi, cap nhat tai lieu agent.

## Ket qua
15 bugs phat hien, 15/15 da xu ly (fix hoac cap nhat specs).

### Bugs da fix (code change)

| Bug | Mo ta | Hanh dong |
|-----|-------|-----------|
| BUG-001 | App.tsx import tu he cu components/pages | Chuyen sang import tu modules/ |
| BUG-002 | 2 he thong Pages ton tai song song | Doi ten components/pages → _backup_pages_v2 |
| BUG-003 | Rust files ten tieng Anh | Doi ten 10 file: *_cmd.rs → lenh_*.rs |
| BUG-004 | 29 import ../../services/ sai | Chuyen sang import tu module service cung cap |
| BUG-005 | ./common khong ton tai trong modules | Tao common.tsx re-export trong 11 module |
| BUG-006 | registry.ts module IDs tieng Anh | Doi sang tieng Viet khong dau |
| BUG-009 | Function name DashboardPage vs BangDieuKhienPage | Them alias export |
| BUG-010 | Thieu bangdieukhien.service.ts | Tao moi voi re-export |
| BUG-011 | Double-escaped \\n trong xuly_hoso.py | Fix \\\\n → \\n |
| BUG-013 | python/xuatfile/ rong | Them docstring |
| BUG-014 | File duplicate ModuleSlot, registry, useModuleConfig | Xoa ban o modules/ goc |

### Bugs da fix (cap nhat specs)

| Bug | Mo ta | Hanh dong |
|-----|-------|-----------|
| BUG-007 | quantailieu thieu ChiTietTaiLieuPage | Cap nhat specs: bo khoi danh sach (chua ton tai) |
| BUG-008 | ChiTietXemTaiLieuPage khong co trong specs | Cap nhat specs: them vao phantichtailieu |
| BUG-012 | python/danhmuc/ chua ro thuoc module nao | Ghi nhan trong AGENTS.md |
| BUG-015 | Spec cu trong thu muc active | Chap nhan (da danh dau Superseded trong INDEX) |

### Tai lieu da cap nhat
- `v3/01_governance/01_AGENTS.md` — Cay thu muc, luong nghiep vu, Rust files
- `v3/standards/07_module_map.md` — Bang module, pages match code
- `v3/standards/08_ai_text_processing.md` — Quy chuan AI Slow Service
- `src-tauri/src/commands/mod.rs` — Ten file moi
- `src/modules/loidung/registry.ts` — Module IDs tieng Viet
- `src/store/moduleStore.ts` — Fix import path

### File di chuyen
- `src/components/pages/` → `src/components/_backup_pages_v2/` (backup)
- 10 Rust files renamed: `*_cmd.rs` → `lenh_*.rs`

## Ghi chu
- Tauri command names (#[tauri::command]) KHONG DOI — chi doi ten file .rs
- services/ van ton tai (legacy) — App.tsx va store van dung truc tiep
- Modules service goi Tauri invoke truc tiep, khong qua services/
