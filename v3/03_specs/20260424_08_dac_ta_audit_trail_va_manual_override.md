# Dac Ta Audit Trail Va Manual Override

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dinh nghia co che ghi vet day du cho moi sua doi quan trong va quy trinh override thu cong de he thong van dat chuan nghiep vu co kiem chung.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`

## Muc Luc

1. Nguyen tac audit
2. Doi tuong phai ghi vet
3. Cau truc audit event
4. Quy trinh manual override
5. Hien thi va tra cuu audit
6. Acceptance criteria

## 1. Nguyen Tac Audit

- Moi chinh sua quan trong phai de lai vet.
- Khong co "silent overwrite".
- Du lieu do AI sinh ra va du lieu do con nguoi sua tay deu phai phan biet ro.
- KSV phai xem duoc:
  - ai/su kien nao da sua,
  - sua luc nao,
  - sua tu gia tri nao sang gia tri nao,
  - sua vi ly do gi,
  - sua bang tay hay do AI de xuat.

## 2. Doi Tuong Phai Ghi Vet

Bat buoc audit voi:

- OCR text da duoc sua tay,
- doan text viet tay do OCR/AI de xuat hoac noi suy,
- `document_type` bi override,
- `display_name`, `document_title`, `issued_date` bi sua,
- entity merge/split/rename,
- citation bi doi anchor hoac bo link,
- contradiction item / timeline item / dossier draft bi sua,
- report draft do AI tao va duoc con nguoi chinh,
- import job bi pause/resume/retry/cancel,
- backup/restore.

## 3. Cau Truc Audit Event

Audit event toi thieu phai co:

- `audit_event_id`
- `object_type`
- `object_id`
- `field_name`
- `action_type`
- `before_value`
- `after_value`
- `actor_type` (`user`, `agent`, `system`, `ai`)
- `actor_id`
- `session_id`
- `source_module`
- `reason_code`
- `reason_note`
- `approval_status`
- `created_at`

`action_type` can co:

- `create`
- `update`
- `override`
- `transcribe_handwriting`
- `approve_interpolation`
- `reject_interpolation`
- `merge`
- `split`
- `restore`
- `delete_soft`
- `approve`
- `reject`

## 4. Quy Trinh Manual Override

Quy trinh chuan:

1. He thong hien gia tri goc + gia tri de xuat + image crop neu do la span text viet tay.
2. Nguoi sua chon thao tac:
   - chap nhan,
   - sua tay,
   - tu choi,
   - hoan lai gia tri cu.
3. Neu sua tay hoac chap nhan text noi suy, bat buoc nhap ly do.
4. He thong ghi audit event.
5. Neu muc sua thuoc loai nhay cam, gan `approval_status = pending_review`.
6. Sau khi duoc review, moi cap nhat index/citation phu thuoc.

Nhung override nhay cam:

- doi `document_type`,
- doi `but_luc`,
- doi `issued_date`,
- doi entity chinh cua dossier,
- doi citation anchor,
- chap nhan text doan chu/noi suy tren tai lieu viet tay,
- nang cap AI draft thanh work product nghiep vu.

## 5. Hien Thi Va Tra Cuu Audit

UI phai co:

- `audit panel` tren document/page/entity/work product,
- bo loc theo:
  - actor,
  - loai sua,
  - khoang thoi gian,
  - object type,
  - trang thai approve.
- kha nang xem diff `before` / `after`.

Neu du lieu da bi sua nhieu lan, he thong phai cho xem lich su theo thu tu revision.

## 6. Acceptance Criteria

- Moi override quan trong deu co audit event.
- Khong co du lieu nghiep vu bi sua ma khong co `reason_code` hoac `reason_note`.
- KSV xem duoc lich su sua cua mot file/trang/entity/work product.
- AI output va user override khong bi tron lam mot.
- Citation/index duoc cap nhat co kiem soat sau override.
- Khong co text doan chu/noi suy nao tro thanh `approved_text` ma khong co audit event.
