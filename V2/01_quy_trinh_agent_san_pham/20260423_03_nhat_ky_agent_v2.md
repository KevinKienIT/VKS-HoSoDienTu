# Nhat Ky Agent V2

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Ghi lai lich su can chinh, dua tai lieu sang V2 va cac quyet dinh van hanh lien quan.
Nguon prompt: `V2/00_prompt_tho/20260423_02_len_v2.md`

## Muc Luc

1. Mau ghi nhat ky
2. Nhat ky hien tai
3. Diem agent sau phai ton trong

## 1. Mau Ghi Nhat Ky

```text
Ngay gio:
Agent:
Da doc:
Da tao:
Da sao chep:
Ket luan:
Rui ro:
Can review:
```

## 2. Nhat Ky Hien Tai

### 2026-04-23 - Codex

Ngay gio: 2026-04-23
Agent: Codex
Da doc:

- `V1/00PromptTho/260423_LenV2.md`
- `V1/00PromptTho/260423_BuildMoTa.md`
- `V1/01_quy_trinh_agent_san_pham/20260423_04_quy_chuan_lam_viec_agent.md`
- `he_thong_quan_ly_ho_so.md`
- `V1/docs/architecture.md`
- `V1/docs/classification_rules.md`
- `V1/docs/user_guide.md`
- `V1/README.md`

Da tao:

- `V2/20260423_00_tong_quan_v2.md`
- `V2/01_quy_trinh_agent_san_pham/20260423_01_ke_hoach_chuyen_len_v2.md`
- `V2/01_quy_trinh_agent_san_pham/20260423_02_ban_do_tai_lieu_va_lenh_v2.md`
- `V2/01_quy_trinh_agent_san_pham/20260423_03_nhat_ky_agent_v2.md`
- `V2/01_quy_trinh_agent_san_pham/20260423_04_quy_chuan_lam_viec_agent_v2.md`
- `V2/02_quy_dinh_bug_test_fix/20260423_01_quy_dinh_check_bug_ghi_bug_test_fix.md`
- `V2/02_quy_dinh_bug_test_fix/20260423_02_mau_bao_cao_bug_test_fix.md`
- `V2/03_bao_cao/20260423_01_quy_dinh_bao_cao.md`
- `V2/04_case_dang_lam_viec/20260423_01_quy_dinh_case_dang_lam_viec.md`
- `V2/98_thung_rac/20260423_01_quy_dinh_thung_rac.md`

Da sao chep:

- Prompt tho tu `V1/00PromptTho` sang `V2/00_prompt_tho`
- Tai lieu mo ta, kien truc, quy tac, huong dan, setup dang tai lieu sang `V2/05_tai_lieu_mo_ta`

Ket luan:

- `V2` da duoc dinh nghia thanh khong gian lam viec tai lieu doc lap.
- Trong `V2`, agent phai tu doc dieu lenh va prompt tho truoc khi xu ly.
- `V2` khong chua code hoac runtime data.
- Da tao luong rieng cho bug, test, fix, report, case dang lam viec va thung rac.

Rui ro:

- Tai lieu tham chieu da sao chep tu V1 co noi dung lich su, khong phai moi file deu la lenh hien hanh.
- Neu agent bo qua `01_quy_trinh_agent_san_pham`, V2 se bi dung nhu kho tai lieu thu dong thay vi luong xu ly.

Can review:

- Xac nhan cach dua prompt tho moi vao `V2/00_prompt_tho`.
- Xac nhan co can tach them nhieu nhom bao cao chi tiet hon trong tuong lai khong.

### 2026-04-23 - Antigravity

Ngay gio: 2026-04-23
Agent: Antigravity
Da doc:

- Toan bo 21 file trong V2 (00 den 98)
- `he_thong_quan_ly_ho_so.md` (file goc ngoai V2)
- Tat ca 3 prompt tho trong `00_prompt_tho`
- Tat ca 6 file SOP trong `01_quy_trinh_agent_san_pham`
- Tat ca 3 file quy dinh trong `02_quy_dinh_bug_test_fix`
- Tat ca file quy dinh trong `03_bao_cao`, `04_case`, `98_thung_rac`
- Tat ca 5 file tai lieu tham chieu trong `05_tai_lieu_mo_ta`

Da tao:

- He quy chuan V2 (artifact phan tich)

Da cap nhat:

- `20260423_00_tong_quan_v2.md` → nang cap thanh entry-point chinh thuc voi chuoi doc 8 buoc, quy chuan dat ten, header, trang thai task/bug/prompt, pham vi cho phep/cam
- `01.../20260423_02_ban_do_tai_lieu_va_lenh_v2.md` → them mapping cho prompt 03
- `00_prompt_tho/260423_03_...` → doi ten thanh `20260423_03_lenh_quy_chuan_ai_thong_minh.md` cho dung quy chuan
- File nhat ky nay → ghi phien lam viec hien tai

Ket luan:

- Nhan dien 7 lo hong trong bo SOP hien tai.
- Da bo sung vao entry-point: quy chuan dat ten file, header bat buoc, dinh nghia trang thai (task 7 trang thai, bug 6, prompt 4), pham vi cho phep/cam.
- Da sua loi dat ten prompt 03.
- Da map prompt 03 vao ban do tai lieu.
- Khong tao code hay file phan mem nao.

Rui ro:

- Tai lieu `05_tai_lieu_mo_ta` van dung lan tieng Viet co dau va khong dau. Can chuan hoa nhung doi pham vi lon (5 file).
- Cac file SOP con (04, 05, 06) chua duoc cap nhat de phan anh quy chuan trang thai moi. Can xem xet bo sung sau.

Can review:

- Xac nhan entry-point moi co du thong tin cho agent moi khong.
- ~~Quyet dinh chuan hoa ngon ngu trong 05_tai_lieu_mo_ta (co dau hay khong dau).~~ DA XONG: chuan hoa khong dau.

### 2026-04-23 - Antigravity (phien 2)

Ngay gio: 2026-04-23
Agent: Antigravity

Da thuc hien:

- So sanh `he_thong_quan_ly_ho_so.md` (root) voi `V2/05.../20260423_01_he_thong_quan_ly_ho_so.md` → trung 100% → xoa file root.
- Hop nhat 5 file trong `05_tai_lieu_mo_ta` thanh 1 file duy nhat: `20260423_01_mo_ta_he_thong_tong_hop.md`.
- Bo sung vao file tong hop: yeu cau giao dien (muc 10), yeu cau AI offline (muc 11), yeu cau da nguoi dung (muc 12), dong goi (muc 13) — lay tu prompt tho 01.
- Chuan hoa toan bo sang tieng Viet khong dau, co header V2 day du.
- Cap nhat ban do tai lieu: ghi ro 5 file da gop, chi con 1 file hien tai.
- Xoa 5 file cu trong `05_tai_lieu_mo_ta`.

Ket luan:

- `05_tai_lieu_mo_ta` gon lai con 1 file duy nhat, day du, khong dau, san sang lam co so cho thiet ke giao dien.
- Root khong con file tai lieu rac.
- Ban do tai lieu da cap nhat phan anh trang thai moi.

Rui ro:

- Khong con rui ro ngon ngu lon lon trong V2.
- File `install.ps1` va `onboard_help.txt` o root la file he thong, khong thuoc V2, de nguyen.

### 2026-04-24 - Codex

Ngay gio: 2026-04-24
Agent: Codex

Da doc:

- `V2/20260423_00_tong_quan_v2.md`
- `V2/01_quy_trinh_agent_san_pham/20260423_02_ban_do_tai_lieu_va_lenh_v2.md`
- `V2/01_quy_trinh_agent_san_pham/20260423_03_nhat_ky_agent_v2.md`
- `V2/01_quy_trinh_agent_san_pham/20260423_04_quy_chuan_lam_viec_agent_v2.md`
- Cac file mo ta va dac ta dang dat sai trong `PhanMem`

Da di chuyen:

- `PhanMem/01_mo_ta_san_pham/20260424_02_nghiep_vu_kiem_sat_vien_va_user_story.md` -> `V2/05_tai_lieu_mo_ta/20260424_02_nghiep_vu_kiem_sat_vien_va_user_story.md`
- `PhanMem/01_mo_ta_san_pham/20260424_03_ban_do_module_va_luong_du_lieu.md` -> `V2/05_tai_lieu_mo_ta/20260424_03_ban_do_module_va_luong_du_lieu.md`
- `PhanMem/01_mo_ta_san_pham/20260424_04_dac_ta_tim_kiem_sap_xep_va_trich_dan.md` -> `V2/05_tai_lieu_mo_ta/20260424_04_dac_ta_tim_kiem_sap_xep_va_trich_dan.md`
- `PhanMem/01_mo_ta_san_pham/20260424_05_ho_so_nguon_tai_lieu_va_chien_luoc_ingest.md` -> `V2/05_tai_lieu_mo_ta/20260424_05_ho_so_nguon_tai_lieu_va_chien_luoc_ingest.md`
- `PhanMem/01_mo_ta_san_pham/20260424_06_dac_ta_ai_offline_phan_tich_tai_lieu.md` -> `V2/05_tai_lieu_mo_ta/20260424_06_dac_ta_ai_offline_phan_tich_tai_lieu.md`

Da cap nhat:

- Chep noi dung `PhanMem/01_mo_ta_san_pham/20260423_01_mo_ta_he_thong_tong_hop.md` ve `V2/05_tai_lieu_mo_ta/20260423_01_mo_ta_he_thong_tong_hop.md`
- Xoa cac thu muc tai lieu trong `PhanMem`
- Sua lien ket `Nguon prompt` va tham chieu noi bo tu `PhanMem/...` sang `V2/...`
- Bo sung quy dinh ro rang: `PhanMem` chi duoc chua file truc tiep giup phan mem chay duoc; moi tai lieu prompt/SOP/mo ta/report/log phai o `V2`
- Sua cac file map/phuong phap lam viec trong `01_quy_trinh_agent_san_pham` de khong tro sai sang `PhanMem` nua

Ket luan:

- `PhanMem` khong duoc dung de chua tai lieu khong phuc vu phan mem chay.
- Toan bo tai lieu phan tich, quy trinh, dac ta va huong dan agent da duoc dua ve dung khu vuc `V2`.
- Tu thoi diem nay, agent nao dat nham tai lieu vao `PhanMem` la sai quy trinh.

Rui ro:

- `PhanMem` hien de trong cho den khi co code, script build/run, test code hoac cau hinh runtime that su.
- Cac file lich su cu co the van nhac den `V1` trong phan ngu canh; day la dau vet lich su, khong phai nguon van hanh hien tai.

Can review:

- Khi tao code thuc te, phai xac dinh cau truc con cua `PhanMem` truoc khi sinh file chay.

### 2026-04-24 - Codex (phien 2)

Ngay gio: 2026-04-24
Agent: Codex

Da doc:

- `V2/01_quy_trinh_agent_san_pham/20260424_12_quy_trinh_aider_thuc_chien.md`
- `V2/05_tai_lieu_mo_ta/20260423_01_mo_ta_he_thong_tong_hop.md`
- Mapping prompt hien tai trong `V2/00_prompt_tho`

Da tao:

- `V2/00_prompt_tho/20260424_04_lenh_viet_lai_nghiep_vu_va_quan_ly_aider_cho_model_manh.md`

Da cap nhat:

- `20260423_02_ban_do_tai_lieu_va_lenh_v2.md` de map prompt moi

Ket luan:

- Da tao mot lenh manh de dua cho model ngoai refactor lai nghiep vu va bo quan tri Aider theo vai tro ro rang.
- Prompt moi ep model doc dung nguon trong `V2`, giu ranh gioi `V2` va `PhanMem`, va neu can thi de xuat noi dung cho bo `Ops/*.md` ma khong bien `PhanMem` thanh noi chua tai lieu.

Rui ro:

- Cac file `Ops/CONVENTIONS.md`, `Ops/AGENT_STANDARD_AIDER_CONTINUE_PILOT.md`, `Ops/PROJECT_RULES.md`, `Ops/PROMPT_SEQUENCE.md` hien chi duoc nhac den nhu dich de viet lai cho moi truong Aider ben ngoai; chua duoc tao trong workspace nay.

Can review:

- Neu can, buoc tiep theo la tao thang bo `Ops/*.md` trong mot khu vuc rieng ngoai `PhanMem`, dong bo tu prompt moi nay.

## 3. Diem Agent Sau Phai Ton Trong

- V2 la ban dang lam viec, khong duoc xem la ban sao luu cu.
- Prompt tho moi phai duoc doc truoc, khong duoc chi dua vao mo ta mieng.
- Bao cao bug, test, fix phai vao dung thu muc.
- Task, plan, lenh da xong khong de lai trong luong chinh; phai dua sang `98_thung_rac` theo quy dinh.

