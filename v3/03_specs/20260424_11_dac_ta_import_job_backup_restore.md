# Dac Ta Import Job Backup Restore

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dinh nghia import job, resume/pause/retry, backup va restore de khong mat du lieu khi xu ly bo ho so scan lon.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`
Nguon corpus: Toan bo file PDF trong `TaiLieu`

## Muc Luc

1. Nguyen tac import job
2. Trang thai job
3. Quy trinh backup
4. Quy trinh restore
5. Loi va recovery
6. Acceptance criteria

## 1. Nguyen Tac Import Job

Moi lan nhap mot thu muc ho so phai tao `import_job`, khong xu ly am tham.

Moi job phai ghi:

- case nao,
- folder nao,
- bao nhieu file,
- dang xu ly den file nao,
- bao nhieu file da thanh cong,
- bao nhieu file loi,
- co the resume duoc tu dau.

## 2. Trang Thai Job

Trang thai job chuan:

- `created`
- `scanning`
- `importing`
- `ocr_processing`
- `indexing`
- `paused`
- `failed`
- `completed`
- `cancelled`

Truong du lieu toi thieu:

- `import_job_id`
- `case_id`
- `source_folder`
- `job_status`
- `total_files`
- `processed_files`
- `failed_files`
- `current_file`
- `last_success_document_id`
- `started_at`
- `updated_at`
- `completed_at`

## 3. Quy Trinh Backup

Bat buoc backup trong cac truong hop:

- truoc restore,
- truoc migrate schema,
- truoc import job lon neu user bat che do an toan,
- truoc thao tac overwrite hang loat.

Backup package toi thieu gom:

- SQLite DB,
- preferences,
- work products,
- audit trail,
- review queue,
- import job status,
- metadata lien ket file goc.

Khong can copy lai PDF goc neu che do backup chi du lieu chi muc. Neu can backup day du, dung `archive_full_case`.

## 4. Quy Trinh Restore

Restore khong duoc ghi de truc tiep len du lieu dang song ma khong canh bao.

Quy trinh:

1. Tao snapshot hien tai.
2. Validate backup package.
3. Hien thi preview:
   - case nao se duoc khoi phuc,
   - du lieu nao se bi ghi de,
   - co mismatch schema hay khong.
4. Xac nhan user.
5. Restore.
6. Rebuild search/citation index neu can.
7. Ghi audit event.

## 5. Loi Va Recovery

He thong phai ho tro:

- `resume_import_from_last_success`
- `retry_failed_documents_only`
- `skip_corrupt_file_and_continue`
- `quarantine_problem_files`
- `restore_after_crash`

File loi phai vao danh sach rieng, khong duoc lam job mat dau vet.

## 6. Acceptance Criteria

- Import bo ho so 68 PDF khong bi mat dau vet khi dung giua chung.
- Job co the pause/resume.
- Backup/restore co preview va snapshot an toan.
- Corrupt/encrypted PDF khong lam sap toan job.
- Sau restore, search/citation/work products van con day du.
