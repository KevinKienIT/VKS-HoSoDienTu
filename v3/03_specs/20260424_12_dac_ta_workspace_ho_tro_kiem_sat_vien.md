# Dac Ta Workspace Ho Tro Kiem Sat Vien

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dinh nghia cac tinh nang giup KSV lam viec thong tha hon voi bo ho so lon, giam tai nhan thuc va giu mach nghien cuu lien tuc.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`
Nguon corpus: Toan bo file PDF trong `TaiLieu`

## Muc Luc

1. Muc tieu workspace
2. Tinh nang giam tai nhan thuc
3. Tinh nang nghien cuu song song
4. Tinh nang tiep tuc cong viec dang do
5. Nguyen tac UI cho KSV
6. Acceptance criteria

## 1. Muc Tieu Workspace

Khi KSV dua vao mot bo ho so scan lon, he thong phai giup:

- khong bi ngop boi filename timestamp,
- biet dang doc den dau,
- biet file nao can doc tiep,
- ghi nho nhung diem quan trong,
- nhin thay cac mau thuan,
- tro lai dung vi tri lam viec cu,
- va tao duoc san pham lam viec ma khong can mo qua nhieu cua so roi rac.

## 2. Tinh Nang Giam Tai Nhan Thuc

Bat buoc co:

- `reading_queue`
- `recently_opened`
- `continue_reading`
- `pin_document`
- `bookmark_page`
- `quick_note`
- `needs_follow_up`
- `review_first`

Nhan dien mau sac/phu de UI phai giup:

- file can review,
- file da doc,
- file dang doc,
- file da pin,
- citation stale.

## 3. Tinh Nang Nghien Cuu Song Song

Phase 1 nen co:

- `split_view_compare`
  - mo 2 tai lieu canh nhau,
  - hoac 2 trang cua 2 tai lieu.
- `contradiction_board`
  - ghi 2 hay nhieu citation dang xung dot.
- `timeline_board`
  - sap xep su kien theo thoi gian.
- `dossier_workspace`
  - tap hop doi tuong, vat chung, citation lien quan.
- `report_draft_panel`
  - tong hop note, citation, issue item, contradiction item vao mot noi.

## 4. Tinh Nang Tiep Tuc Cong Viec Dang Do

He thong phai luu va khoi phuc:

- case dang mo,
- document dang doc,
- page dang doc,
- tab dang mo,
- sort/filter gan nhat,
- reading queue,
- bookmark,
- note ca nhan,
- contradiction item dang mo.

Co them:

- `session_resume_card`
- `last_work_product_opened`
- `last_ai_query_context`

## 5. Nguyen Tac UI Cho KSV

- Desktop first, Windows first.
- Khong ep mobile-first trong phase scaffold dau.
- Motion vua du, uu tien doc nhanh va khong lam mat tap trung.
- Mau canh bao dung cho review/stale/failed ro rang.
- Moi panel phai tra loi mot nhu cau nghiep vu ro, khong mo panel de trang tri.
- AI panel phai ho tro che do:
  - tra cuu nhanh,
  - tong hop co citation,
  - dua vao work product draft.

## 6. Acceptance Criteria

- KSV co the dong app va mo lai de tiep tuc dung diem dang lam.
- KSV co the pin, bookmark, note ma khong mat ngu canh.
- KSV co the so sanh hai tai lieu hoac hai citation.
- He thong co contradiction board/to-do nghiep vu co can cu citation.
- UI giup doc bo ho so lon ma khong can mo/ghi nho bang tay qua nhieu.
