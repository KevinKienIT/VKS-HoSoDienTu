# 00 SPEC INDEX — Mục lục đặc tả sản phẩm

> Cập nhật: 2026-05-04

## Tổng hợp chính

| File | Vai trò | Trạng thái |
|------|---------|------------|
| `01_tonghop.md` | ★ Spec tổng hợp nhất (57KB) — UI/workflow/schema/nghiệp vụ | **Active** |
| `02_UI_UX_IMPROVEMENT_PROPOSAL.md` | Đề xuất cải thiện UI/UX | Draft |
| `03_schema.json` | JSON schema lớn (4.3MB) — **không paste nguyên file**, dùng query/parse theo khóa | Reference |

## Đặc tả tổng quan

| File | Module | Trạng thái |
|------|--------|------------|
| `20260423_00_tong_quan_v2.md` | Tổng quan V2 (cũ) | Superseded bởi `01_tonghop.md` |
| `20260423_01_mo_ta_he_thong_tong_hop.md` | Mô tả hệ thống tổng hợp | Active |

## Đặc tả theo module

| File | Module | Trạng thái |
|------|--------|------------|
| `20260424_02_nghiep_vu_kiem_sat_vien_va_user_story.md` | User stories | Active |
| `20260424_03_ban_do_module_va_luong_du_lieu.md` | Module map + data flow | Active |
| `20260424_04_dac_ta_tim_kiem_sap_xep_va_trich_dan.md` | Search/Sort/Citation | Active |
| `20260424_05_ho_so_nguon_tai_lieu_va_chien_luoc_ingest.md` | Import/Ingest | Active |
| `20260424_06_dac_ta_ai_offline_phan_tich_tai_lieu.md` | AI Offline | Active |
| `20260424_07_dac_ta_case_lifecycle_va_work_product.md` | Case Lifecycle | Active |
| `20260424_08_dac_ta_audit_trail_va_manual_override.md` | Audit/Override | Active |
| `20260424_09_dac_ta_citation_anchor_va_ocr_revision.md` | Citation/OCR Revision | Active |
| `20260424_10_tu_dien_document_type_va_quy_tac_phan_loai.md` | Doc Type Dictionary | Active |
| `20260424_11_dac_ta_import_job_backup_restore.md` | Import/Backup | Active |
| `20260424_12_dac_ta_workspace_ho_tro_kiem_sat_vien.md` | Workspace KSV | Active |

## Đặc tả kế hoạch & kiến trúc

| File | Module | Trạng thái |
|------|--------|------------|
| `20260424_13_ke_hoach_phan_ra_phase_va_gate_trien_khai.md` | Phase plan (cũ) | Superseded bởi `08_execution_phases/` |
| `20260424_14_ban_do_thuoc_tinh_metadata_va_schema_nghiep_vu.md` | Metadata schema | Active |
| `20260426_01_quyet_dinh_nen_tang_va_cach_chay.md` | Quyết định nền tảng Tauri | **Active — bắt buộc đọc** |

## Đặc tả UI/UX

| File | Module | Trạng thái |
|------|--------|------------|
| `20260424_15_ban_do_giao_dien_tong_the_va_dieu_huong_chinh.md` | UI Map tổng | Active |
| `20260424_16_dac_ta_man_hinh_quan_ly_trang_va_viewer.md` | Page/Viewer UI | Active |
| `20260424_17_dac_ta_man_hinh_quan_ly_ho_so_va_khoi_ho_so.md` | Case/Dossier UI | Active |
| `20260424_18_dac_ta_man_hinh_doc_text_citation_va_review.md` | Text/Citation/Review UI | Active |
| `20260424_19_dac_ta_quan_ly_tai_khoan_nguoi_dung_va_phan_quyen.md` | User/Permission UI | Active |

## Đặc tả workflow end-to-end

| File | Module | Trạng thái |
|------|--------|------------|
| `20260424_20_ban_do_luong_chuc_nang_end_to_end.md` | E2E flow | Active |
| `20260424_21_dac_ta_luong_scan_ocr_ai_danh_gia_chat_luong.md` | Scan/OCR/AI pipeline | Active |
| `20260424_22_dac_ta_tach_file_ingest_db_metadata.md` | File split/ingest | Active |

## Đặc tả nghiệp vụ chuyên sâu

| File | Module | Trạng thái |
|------|--------|------------|
| `20260424_23_dac_ta_profile_bi_can_luoc_su_va_xet_hoi.md` | Profile bị can | Active |
| `20260424_24_ban_do_lien_ket_giua_file_thong_tin_thuc_the_su_kien.md` | Entity linking | Active |
| `20260424_25_ban_do_tri_thuc_man_hinh_tom_tat_bi_can.md` | Knowledge map | Active |

## Quy ước đọc spec

1. Đọc `01_tonghop.md` trước khi đọc file chi tiết.
2. File đánh dấu **Superseded** → chỉ tham khảo, không dùng làm source of truth.
3. `03_schema.json` → query theo key, KHÔNG paste nguyên file.
4. Khi mâu thuẫn giữa spec và standards → standards thắng.
