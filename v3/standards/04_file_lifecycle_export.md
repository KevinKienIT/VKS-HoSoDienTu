# 04 File Lifecycle Export

## Vòng đời file bắt buộc

Trạng thái:
`imported -> processing -> ocr_done -> review_pending -> reviewed -> managed_ready -> exported`

## Cấu trúc thư mục storage (flat layout)

`VKS_ECMS_Data/`
1. `originals/<CASE_CODE>/` (bất biến)
2. `processing/<CASE_CODE>/` (trung gian OCR)
3. `reviewed/<CASE_CODE>/` (sau chỉnh sửa user)
4. `managed/<CASE_CODE>/` (bản chuẩn nghiệp vụ)
5. `exports/<EXPORT_JOB_ID>_<TIMESTAMP>/` (gói bàn giao)
6. `logs/` (pipeline/audit global)

## Quy tắc dữ liệu

1. Không ghi đè file gốc.
2. Mọi sửa OCR/metadata tạo revision.
3. Export chỉ lấy từ `managed_ready` + citation hợp lệ.

## Gói export chuẩn

`exports/<EXPORT_JOB_ID>_<TIMESTAMP>/`
1. `00_index_trich_dan.docx`
2. `01_bao_cao_tong_hop.docx`
3. `02_so_do_vu_an.html`
4. `03_tai_lieu_quan_ly/`
5. `04_phu_luc_citation.json`
6. `05_audit_export.json`
