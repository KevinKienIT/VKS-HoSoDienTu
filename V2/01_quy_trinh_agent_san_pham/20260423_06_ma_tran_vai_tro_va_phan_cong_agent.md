# Ma Tran Vai Tro Va Phan Cong Agent

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-23
Sua boi agent: Codex
Muc dich: Dinh nghia vai tro, trach nhiem, dau vao, dau ra va ranh gioi cua tung agent trong team AI.
Nguon prompt: `V2/00_prompt_tho/20260423_02_len_v2.md`

## Muc Luc

1. Nguyen tac phan cong
2. Danh sach vai tro
3. Ma tran ban giao
4. Quy tac doi vai

## 1. Nguyen Tac Phan Cong

- Moi task chi co 1 owner chinh.
- Co the co nhieu agent phoi hop, nhung owner chinh chiu trach nhiem chot.
- Khong giao task ma khong co dau vao va dau ra.
- Neu 1 agent kiem nhieu vai tro, phai ghi ro role hien tai trong log.

## 2. Danh Sach Vai Tro

| Vai tro | Trach nhiem chinh | Dau vao | Dau ra | Khong duoc bo qua |
| --- | --- | --- | --- | --- |
| agent_leader | dieu phoi, uu tien, gate review | prompt, plan, log | task board, quyet dinh gate | khong bo qua check scope |
| agent_tiep_nhan_lenh | ghi nhan menh lenh va tao case | prompt tho | file prompt, note tiep nhan | khong lam mat prompt goc |
| agent_phan_tich_lenh | tach muc tieu, rang buoc, acceptance criteria | prompt, docs | phan tich, danh sach can doc | khong duoc doan y scope |
| agent_phan_tich_tai_lieu | doc va tom tat tai lieu lien quan | docs, case | context map, note tai lieu | khong bo sot tai lieu cot loi |
| agent_kien_truc | thiet ke giai phap tong the | plan, context | dac ta API/UI/data/AI | khong de implementer tu quyet dinh thay |
| agent_frontend | code UI, state, interaction | specs UI | thay doi frontend, self test | khong bo qua test flow UI |
| agent_backend | code logic, API, workflow | specs backend | thay doi backend, self test | khong bo qua test function |
| agent_data_ai | schema, data flow, AI logic | specs data/AI | thay doi data/AI, validation | khong bo qua nguon du lieu |
| agent_test_function | test function, API, integration | build, specs | test report function | khong test mot cach hinh thuc |
| agent_test_giao_dien | test UI, layout, interaction | UI build, flows | test report UI | khong bo qua empty/loading/error |
| agent_qa_doc_lap | cross-check doc lap, check vong | tat ca report | bug list, ket luan QA | khong duoc du de qua khi chua co bang chung |
| agent_fix_bug | sua bug va retest loop | bug report | fix report | khong dong bug thay tester |
| agent_bao_cao_cleanup | tong hop, don dep, luu chuyen | report, log, case | tong hop, cleanup, archive move | khong de file da xong trong luong active |

## 3. Ma Tran Ban Giao

| Tu vai tro | Sang vai tro | Dieu kien ban giao | Tai lieu bat buoc |
| --- | --- | --- | --- |
| tiep_nhan_lenh | leader | prompt da luu vao `00_prompt_tho` | prompt file + note tiep nhan |
| phan_tich_lenh | kien_truc | da ro muc tieu, pham vi, rang buoc | phan tich + docs can doc |
| kien_truc | implementer | specs da ro, khong mo ho | dac ta + acceptance criteria |
| implementer | test_function/test_giao_dien | da self check va co note thay doi | change note + self test |
| tester | qa_doc_lap | da co report va bug list so bo | test report |
| qa_doc_lap | fix_bug | bug da duoc triage | bug report |
| fix_bug | tester | da co fix report | fix report |
| tester/qa | bao_cao_cleanup | bug da dong hoac ghi trang thai | final reports |

## 4. Quy Tac Doi Vai

- 1 agent co the dong nhieu vai, nhung khong duoc tu dong vua implement vua la QA doc lap cho chinh thay doi do.
- Neu thieu nguoi, co the gom vai tro gan nhau, nhung van phai giu it nhat 1 vong check doc lap.
- Moi lan doi vai tro phai ghi vao nhat ky de agent sau biet ngu canh.

