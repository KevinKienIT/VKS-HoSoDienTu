# 07 Module Map — Ban do module nghiep vu

> Quy chuan dat ten thu muc va file theo tieng Viet khong dau.
> Cap nhat: 2026-05-04

---

## 1. Danh sach 12 module

| # | Ten module | Y nghia | Pages | Service | Rust cmd |
|---|-----------|---------|-------|---------|----------|
| 01 | `bangdieukhien` | Bang dieu khien (Dashboard) | BangDieuKhienPage | bangdieukhien.service | lenh_bangdieukhien |
| 02 | `duahosovao` | Dua ho so vao (Import) | DuaHoSoVaoPage | duahosovao.service | lenh_duahosovao |
| 03 | `quettailieu` | Quet tai lieu (Scan) | QuetTaiLieuPage | quettailieu.service | lenh_quettailieu |
| 04 | `phantichtailieu` | Phan tich tai lieu (OCR/Analyze) | PhanTichTaiLieuPage, ChiTietXemTaiLieuPage, TrinhXemTaiLieu, AnhThuNho | phantichtailieu.service | lenh_phantichtailieu |
| 05 | `quantailieu` | Quan tai lieu (Manage) | DanhSachTaiLieuPage | quantailieu.service | lenh_quantailieu |
| 06 | `timkiem` | Tim kiem (Search) | TimKiemPage | timkiem.service | lenh_timkiem |
| 07 | `phantichai` | Phan tich AI | KhongGianAIPage, SoTayAIPanel | phantichai.service | lenh_phantichai |
| 08 | `xuatbangiao` | Xuat ban giao (Export) | XuatBanGiaoPage | xuatbangiao.service | lenh_xuatbangiao |
| 09 | `duyethotro` | Duyet ho tro (Review) | DuyetHoTroPage | duyethotro.service | lenh_duyethotro |
| 10 | `hosovuan` | Ho so vu an (Case/Dossier) | DanhSachVuAnPage, ChiTietVuAnPage | hosovuan.service | lenh_hosovuan |
| 11 | `cauhinh` | Cau hinh (Settings) | CauHinhPage | cauhinh.service | lenh_cauhinh |
| 12 | `loidung` | Loi dung chung (Core/Shared) | common.tsx, ModuleSlot | registry, useModuleConfig | — |

---

## 2. Cau truc thu muc chuan

### Frontend (src/modules/)

```
modules/<tenmodule>/
├── <TenComponent>Page.tsx     ← UI chinh
├── <tenmodule>.service.ts     ← goi Tauri invoke
├── <tenmodule>.types.ts       ← types/interfaces (neu can)
└── index.ts                   ← barrel export
```

### Rust (src-tauri/src/lenh/)

```
lenh/
├── mod.rs                     ← re-export tat ca
├── lenh_<tenmodule>.rs        ← commands cho module
└── lenh_hethong.rs            ← startup gate, storage, system
```

### Python (python/)

```
python/
├── ocr/                       ← giu ten quoc te
├── danhmuc/                   ← catalog scanner
├── phantich/                  ← DOCX/HTML/PDF parsers
└── xuatfile/                  ← export helpers
```

### Database (src-tauri/)

```
csdl/
├── mod.rs                     ← init + migration runner
└── bangmau.rs                 ← schema constants
dichchuyen/
├── NNN_mo_ta_ngan.sql         ← migration files
```

---

## 3. Quy uoc dat ten

| Loai | Quy uoc | Vi du |
|------|---------|-------|
| Thu muc module | vietlien, khong dau, lowercase | `phantichtailieu` |
| Component TSX | PascalCase | `DuaHoSoVaoPage.tsx` |
| Service TS | lowercase.service.ts | `timkiem.service.ts` |
| Rust command file | lenh_tenmodule.rs | `lenh_phantichai.rs` |
| Python file | ten_hanh_dong.py | `phantich_docx.py` |
| SQL migration | NNN_mo_ta.sql | `007_vongdoi_file.sql` |

---

## 4. Tu dien doi chieu

| Tieng Viet khong dau | Tieng Viet co dau | English |
|----------------------|-------------------|---------|
| bangdieukhien | Bang dieu khien | Dashboard |
| duahosovao | Dua ho so vao | Import |
| quettailieu | Quet tai lieu | Scan |
| phantichtailieu | Phan tich tai lieu | Analyze/OCR |
| quantailieu | Quan tai lieu | Documents |
| timkiem | Tim kiem | Search |
| phantichai | Phan tich AI | AI Analysis |
| xuatbangiao | Xuat ban giao | Export |
| duyethotro | Duyet ho tro | Review |
| hosovuan | Ho so vu an | Case/Dossier |
| cauhinh | Cau hinh | Settings |
| loidung | Loi dung | Core/Shared |
| lenh | Lenh | Command |
| csdl | Co so du lieu | Database |
| luutru | Luu tru | Storage |
| dichchuyen | Dich chuyen | Migration |
| danhmuc | Danh muc | Catalog |
| phantich | Phan tich | Parser |
| xuatfile | Xuat file | Export helpers |
| bangmau | Bang mau | Schema |
| anhthunho | Anh thu nho | Thumbnail |
| trinhxem | Trinh xem | Viewer |
| sotay | So tay | Notebook |
| khonggian | Khong gian | Workspace |
| danhsach | Danh sach | List |
| chitiet | Chi tiet | Detail |

---

## 5. Mapping file cu → file moi

### Pages

| Cu | Moi (module/file) |
|---|---|
| DashboardPage.tsx | bangdieukhien/BangDieuKhienPage.tsx |
| ImportJobPage.tsx | duahosovao/DuaHoSoVaoPage.tsx |
| ScanPage.tsx | quettailieu/QuetTaiLieuPage.tsx |
| AnalyzePage.tsx | phantichtailieu/PhanTichTaiLieuPage.tsx |
| DocumentViewerPage.tsx | phantichtailieu/TrinhXemTaiLieu.tsx |
| DocumentListPage.tsx | quantailieu/DanhSachTaiLieuPage.tsx |
| CaseListPage.tsx | hosovuan/DanhSachVuAnPage.tsx |
| CaseDetailPage.tsx | hosovuan/ChiTietVuAnPage.tsx |
| SearchPage.tsx | timkiem/TimKiemPage.tsx |
| AiWorkspacePage.tsx | phantichai/KhongGianAIPage.tsx |
| AiNotebookPanel.tsx | phantichai/SoTayAIPanel.tsx |
| ExportPage.tsx | xuatbangiao/XuatBanGiaoPage.tsx |
| ReviewQueuePage.tsx | duyethotro/DuyetHoTroPage.tsx |
| SettingsPage.tsx | cauhinh/CauHinhPage.tsx |

### Services

| Cu | Moi |
|---|---|
| caseService.ts | hosovuan/hosovuan.service.ts |
| importService.ts | duahosovao/duahosovao.service.ts |
| scanService.ts | quettailieu/quettailieu.service.ts |
| documentService.ts | **TACH**: quantailieu.service.ts + phantichtailieu.service.ts |
| searchService.ts | timkiem/timkiem.service.ts |
| aiService.ts | phantichai/phantichai.service.ts |
| exportService.ts | xuatbangiao/xuatbangiao.service.ts |
| reviewService.ts | duyethotro/duyethotro.service.ts |
| catalogService.ts | merge vao duahosovao.service.ts |
| appSettingService.ts | cauhinh/cauhinh.service.ts |

### Rust Commands

| Cu (size) | Moi |
|---|---|
| case_cmd.rs (25KB) | lenh_hosovuan.rs |
| import_cmd.rs (28KB) | lenh_duahosovao.rs |
| scan_cmd.rs (78KB) | lenh_quettailieu.rs |
| doc_cmd.rs (83KB) | **TACH**: lenh_phantichtailieu.rs + lenh_quantailieu.rs |
| search_cmd.rs (8KB) | lenh_timkiem.rs |
| ai_cmd.rs (12KB) | lenh_phantichai.rs |
| export_cmd.rs (15KB) | lenh_xuatbangiao.rs |
| review_cmd.rs (6KB) | lenh_duyethotro.rs |
| module_cmd.rs (3.6KB) | lenh_cauhinh.rs |
| catalog_cmd.rs (14KB) | lenh_duahosovao.rs (merge) |

---

## 6. Luu y khi migration

1. **KHONG** doi ten Tauri command names (vi frontend goi theo ten command, khong phai ten file)
2. Tach `doc_cmd.rs` (83KB) thanh 2 file nhung giu nguyen `#[tauri::command]` signatures
3. Move tung module mot, test sau moi buoc
4. Cap nhat `mod.rs` re-exports sau moi lan di chuyen
5. `App.tsx` routes chi thay doi import paths, khong doi route paths
