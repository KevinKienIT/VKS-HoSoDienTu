# Quy Trinh Team AI Tu Dong

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-23
Sua boi agent: Codex
Muc dich: Dat quy trinh van hanh day du cho mot team AI manh, tu nhan menh lenh den dong viec va don dep tai lieu.
Nguon prompt: `V2/00_prompt_tho/20260423_02_len_v2.md`

## Muc Luc

1. Muc tieu cua quy trinh
2. Nguyen tac van hanh
3. Vong doi cong viec
4. Giai doan chi tiet
5. Vong test nhieu lop
6. Fix loop
7. Report va ban giao
8. Dong viec va don dep
9. Dinh nghia hoan thanh

## 1. Muc Tieu Cua Quy Trinh

Quy trinh nay dung cho mot team AI xay dung du an tu dong, nhung van co ky luat nhu mot team ky thuat manh:

- moi yeu cau vao deu duoc doc va phan tich,
- moi task deu co owner va ban giao,
- moi thay doi deu qua test va check vong,
- moi bug deu duoc ghi, fix, retest va report,
- moi file, task, plan da xong deu duoc day ra khoi luong dang lam viec.

## 2. Nguyen Tac Van Hanh

1. Prompt tho la dau vao chinh.
2. Team khong nhay vao code khi chua ro pham vi va tai lieu can doc.
3. Moi task phai co: owner, input, output, gate.
4. Moi vong test phai co bang chung.
5. Moi bug phai co report va trang thai.
6. Moi file da xong phai chuyen khoi luong active.
7. Khong de tri nho tac nghiep thay cho tai lieu.

## 3. Vong Doi Cong Viec

Vong doi chuan gom 12 buoc:

1. Nhan menh lenh
2. Tiep nhan va dieu phoi
3. Doc prompt va doc tai lieu
4. Phan tich day du
5. Lap ke hoach va chia task
6. Dac ta giai phap
7. Trien khai theo vai tro
8. Tu test va peer check
9. Test tich hop va test giao dien
10. QA doc lap va check vong
11. Fix bug va retest
12. Report, dong viec va don dep

## 4. Giai Doan Chi Tiet

| Giai doan | Owner chinh | Dau vao | Dau ra bat buoc | Gate de sang buoc sau |
| --- | --- | --- | --- | --- |
| 1. Nhan menh lenh | agent_tiep_nhan_lenh | prompt tho | ghi nhan prompt, muc tieu so bo | prompt da duoc dua vao `00_prompt_tho` |
| 2. Tiep nhan va dieu phoi | agent_leader | prompt, bo SOP | quyet dinh scope, do uu tien, case dang lam viec | case va owner da duoc gan |
| 3. Doc prompt va doc tai lieu | agent_phan_tich_lenh + agent_phan_tich_tai_lieu | prompt, docs, quy dinh | danh sach tai lieu da doc, context note | context da du de phan tich |
| 4. Phan tich day du | agent_phan_tich_lenh + agent_kien_truc | context, tai lieu | phan ra muc tieu, rang buoc, rui ro, pham vi | khong con diem mo ho lon |
| 5. Lap ke hoach va chia task | agent_leader | phan tich | plan, map, task board, owner | task co owner va dependencies |
| 6. Dac ta giai phap | agent_kien_truc + agent_san_pham | plan, map | dac ta API, UI, data, AI, acceptance criteria | implementer khong can doan y |
| 7. Trien khai theo vai tro | frontend/backend/data/ai | dac ta | code, note thay doi, log | code qua tu check |
| 8. Tu test va peer check | implementer + peer cung nhom | code, specs | unit test, self review, peer note | khong con loi muc co ban |
| 9. Test tich hop va test giao dien | agent_test_function + agent_test_giao_dien | build, case, test plan | bao cao test function, integration, UI | luong chinh chay duoc |
| 10. QA doc lap va check vong | agent_qa_doc_lap | tat ca report truoc | danh sach bug, risk, ket luan vong | chi con bug chap nhan duoc hoac da giao fix |
| 11. Fix bug va retest | agent_fix_bug + tester lien quan | bug report | fix report, ket qua retest | bug da dong hoac da downgrade co ly do |
| 12. Report, dong viec va don dep | agent_bao_cao + agent_cleanup + leader | report cac vong | tong hop dot viec, move file da xong, cap nhat log | luong dang lam viec gon, ro, khong ton file rac |

## 5. Vong Test Nhieu Lop

Mot team AI manh khong dung o 1 lan test. Can toi thieu 5 lop:

1. Lop 0 - self check cua implementer:
   - doc lai specs,
   - chay unit test lien quan,
   - check lint/log/coherence,
   - tu viet note rui ro.
2. Lop 1 - peer check cung nhom:
   - backend check backend,
   - frontend check frontend,
   - ai/data check ai/data.
3. Lop 2 - test function va integration:
   - input/output,
   - contract,
   - dependency,
   - regression vung bi anh huong.
4. Lop 3 - test giao dien va user flow:
   - layout,
   - interaction,
   - state,
   - responsive,
   - empty/loading/error states.
5. Lop 4 - QA doc lap va cross-check:
   - agent khong tham gia implement se check lai,
   - so voi prompt, plan, report, test,
   - tim bug bi bo sot,
   - ket luan cho phep dong hay yeu cau quay lai fix.

Neu that bai o bat ky lop nao, cong viec quay ve fix loop.

## 6. Fix Loop

Fix loop chuan:

1. Mo bug report.
2. Triage bug: muc do, owner, pham vi anh huong.
3. Sua bug trong pham vi da xac dinh.
4. Viet fix report.
5. Retest dung lop da phat hien bug.
6. Chay regression toi thieu tren luong lien quan.
7. Cap nhat tong hop.
8. Dong bug hoac mo lai bug neu that bai.

Khong duoc dong bug chi vi da sua code. Bat buoc phai co retest.

## 7. Report Va Ban Giao

Moi phase can report toi thieu nhu sau:

- sau phan tich: cap nhat plan/map/log;
- sau implement: note thay doi va owner;
- sau test: test report;
- sau fix: fix report;
- sau dot viec: tong hop dot viec.

Ban giao hop le phai tra loi duoc:

- da lam gi,
- da sua o dau,
- da test gi,
- con bug nao mo,
- rui ro nao chua dong,
- file nao can agent sau doc truoc.

## 8. Dong Viec Va Don Dep

Sau khi mot task hoac dot viec xong:

1. Cap nhat nhat ky.
2. Cap nhat report tong hop.
3. Di chuyen lenh, task, plan, report da xong sang `98_thung_rac` neu khong con active.
4. Giu `04_case_dang_lam_viec` chi con tai lieu va ban dang thao tac.
5. Kiem tra lai de khong con file mo ho, file tam, file duplicate trong luong active.

## 9. Dinh Nghia Hoan Thanh

Mot dot viec chi duoc xem la hoan thanh khi:

- prompt da duoc xu ly va map ro,
- task da co owner va da xong,
- code hoac tai lieu da qua test nhieu lop,
- bug mo da duoc dong hoac ghi ro ly do tri hoan,
- report da day du,
- file da xong da duoc day ra khoi luong active.

