# Quy Chuan Lam Viec Agent V2

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dat quy tac chinh thuc de agent tu hieu cach xu ly khi vao V2.
Nguon prompt: `V2/00_prompt_tho/20260423_02_len_v2.md`

## Muc Luc

1. Quy tac nhan dien V2
2. Quy trinh bat buoc
3. Quy tac xu ly prompt tho
4. Quy tac lam viec voi case
5. Quy tac bug, test, fix, report
6. Quy tac phan cong va ban giao
7. Quy tac dong task da xong
8. Dieu cam

## 1. Quy Tac Nhan Dien V2

Khi nguoi dung noi "lam o V2", "dua len V2", hoac khi agent dang lam viec trong thu muc `V2`, agent phai tu hieu:

- Day la khong gian lam viec chinh thuc theo prompt tho cua nguoi dung.
- Agent phai tu doc dieu lenh va prompt tho truoc khi lam.
- Agent khong cho giai thich lai bang tay neu prompt da co san trong `00_prompt_tho`.

## 2. Quy Trinh Bat Buoc

Moi agent phai lam theo thu tu:

1. Doc file moi nhat trong `V2/00_prompt_tho`.
2. Doc file nay.
3. Doc `20260423_05_quy_trinh_team_ai_tu_dong.md`.
4. Doc `20260423_06_ma_tran_vai_tro_va_phan_cong_agent.md`.
5. Doc ke hoach V2.
6. Doc ban do V2.
7. Doc nhat ky V2.
8. Neu prompt moi chua duoc map, cap nhat map va nhat ky truoc.
9. Truoc khi tao hoac di chuyen file, phai xac dinh file do thuoc `V2` hay `PhanMem`; neu file khong truc tiep giup phan mem chay duoc thi bat buoc dat trong `V2`.
10. Moi tac dong co thay doi phai de lai dau vet trong log hoac report.

## 3. Quy Tac Xu Ly Prompt Tho

- Prompt tho la nguon vao chinh.
- Agent phai tach prompt tho thanh: muc tieu, pham vi, rang buoc, tai lieu can doc, file can cap nhat.
- Neu prompt tho nhac den bug, test, fix hoac report, agent phai vao dung thu muc quy dinh.
- Neu prompt tho da xong nhiem vu, dua no sang `98_thung_rac/01_lenh_da_xong` thay vi de lai o luong dang lam viec.

## 4. Quy Tac Lam Viec Voi Case

- Moi case dang xu ly phai nam trong `V2/04_case_dang_lam_viec`.
- Trong case, chi giu ban dang su dung de lam viec.
- Ban cu, ban da thay the hoac ban da hoan thanh khong duoc de trong luong chinh.
- Neu can giu lich su, chuyen sang `V2/98_thung_rac` kem note ngan.

## 5. Quy Tac Bug, Test, Fix, Report

- Bao cao bug dat o `V2/03_bao_cao/01_bug`.
- Bao cao test dat o `V2/03_bao_cao/02_test`.
- Bao cao fix dat o `V2/03_bao_cao/03_fix`.
- Bao cao tong hop dat o `V2/03_bao_cao/04_tong_hop`.
- Quy dinh chi tiet nam o `V2/02_quy_dinh_bug_test_fix`.

## 6. Quy Tac Phan Cong Va Ban Giao

- Moi task phai co owner ro rang.
- Moi task phai co dau vao, dau ra va dieu kien hoan thanh.
- Agent khong nhan task khi chua ro scope.
- Ban giao giua cac agent phai co file log hoac report de agent sau khong doan y.
- Vong phat trien day du duoc mo ta trong `20260423_05_quy_trinh_team_ai_tu_dong.md`.
- Vai tro va ranh gioi cua tung agent duoc mo ta trong `20260423_06_ma_tran_vai_tro_va_phan_cong_agent.md`.

## 7. Quy Tac Dong Task Da Xong

- Task, plan, lenh va report da xong khong de lan voi doi dang xu ly.
- Phai dua ve:
  - `98_thung_rac/01_lenh_da_xong`
  - `98_thung_rac/02_task_da_xong`
  - `98_thung_rac/03_plan_da_xong`
  - `98_thung_rac/04_report_da_xong`

## 8. Dieu Cam

- Khong dua code va phan mem vao V2.
- Khong dua prompt, plan, SOP, mo ta nghiep vu, mo ta he thong, dac ta module, dac ta AI, report, log hoac tai lieu tham chieu vao `PhanMem`.
- `PhanMem` chi duoc chua source code, ma giao dien, asset UI, cau hinh runtime, manifest phu thuoc, script build/run va ma test.
- Neu chua co phan mem dang chay, de `PhanMem` trong; khong dung `PhanMem` lam noi dat tam tai lieu.
- Khong xu ly prompt tho theo tri nho neu da co file prompt.
- Khong lam viec tren ban cu khi da co ban dang lam viec.
- Khong ghi bug, test, fix lung tung ngoai cau truc V2.
- Khong xoa im lang tai lieu da xong; phai dua sang thung rac co cau truc.

