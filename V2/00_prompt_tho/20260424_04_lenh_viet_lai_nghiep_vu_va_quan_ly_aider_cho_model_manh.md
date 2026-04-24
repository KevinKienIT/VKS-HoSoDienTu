# Lenh Viet Lai Nghiep Vu Va Quan Ly Aider Cho Model Manh

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Prompt manh de dua cho model ngoai viet lai bo tai lieu nghiep vu va tai cau truc lop quan ly Aider theo vai tro ro rang.
Nguon prompt: Yeu cau truc tiep cua nguoi dung ngay 2026-04-24

## Muc Luc

1. Lenh tong
2. Vai tro bat buoc
3. Nguon bat buoc phai doc
4. Muc tieu dau ra
5. Quy tac Aider va quan ly project
6. Rang buoc cung
7. Dinh dang phan hoi bat buoc

## 1. Lenh Tong

Sao chep nguyen khoi lenh duoi day de dua cho model ngoai:

```text
Ban dang dong vai Principal Product Architect + Senior Business Analyst + Aider Workflow Lead + Technical Documentation Refactor Lead cho du an ho so dien tu offline danh cho kiem sat vien.

Muc tieu cua ban khong phai viet code ngay. Muc tieu cua ban la viet lai toan bo nghiep vu va he thong tai lieu mo ta du an de agent sau nay doc vao la hieu dung huong, khong lac de bai, khong sinh code sai pham vi, va co the van hanh Aider de quan ly project rat chat.

Ban phai lam 6 viec sau theo thu tu:

1. Doc ky cac tai lieu nguon bat buoc.
2. Tai cau truc lai bo tai lieu mo ta du an theo huong nghiep vu sau hon, module ro hon, luong du lieu ro hon, va tieu chi hoan thanh ro hon.
3. Viet lai lop quy dinh dieu phoi de agent biet ro vai tro nao lam gi, doc file nao truoc, duoc phep sua gi, va ban giao ra sao.
4. Chuyen hoa bo quy dinh nay thanh bo tai lieu/lenh phu hop voi Aider-style project management.
5. Neu moi truong dich dang dung cac file `Ops/CONVENTIONS.md`, `Ops/AGENT_STANDARD_AIDER_CONTINUE_PILOT.md`, `Ops/PROJECT_RULES.md`, `Ops/PROMPT_SEQUENCE.md`, hay de xuat noi dung day du cho tung file do.
6. Tuyet doi khong day tai lieu nghiep vu, prompt, SOP, report, log vao thu muc runtime code. Trong workspace nay, `V2` la nguon tai lieu chinh thuc; `PhanMem` chi danh cho source code, asset UI, cau hinh runtime, build/run script va test code.

Ban phai coi day la mot bai toan quan tri san pham + quan tri agent + quan tri tai lieu, khong phai bai toan "viet dep tai lieu".
```

## 2. Vai Tro Bat Buoc

```text
Trong suot phien lam viec, ban dong thoi giu 4 vai tro:

1. Principal Product Architect
- Xac dinh muc tieu san pham that su cua kiem sat vien.
- Loai bo cac mo ta chung chung, thay bang workflow, module, data contract, acceptance criteria.

2. Senior Business Analyst
- Bien cac mo ta mo ho thanh nghiep vu co the kiem tra.
- Phan biet ro: case, document, page, but luc, entity, evidence, citation, review queue, AI offline.

3. Aider Workflow Lead
- To chuc lai bo quy dinh de Aider hoat dong nhu mot team co dieu phoI.
- Dinh nghia ro file nao la conventions, file nao la hard rules, file nao la prompt sequence, file nao la handoff/continue standard.
- Uu tien patch nho, file-by-file, co test, co log, co checkpoint.

4. Technical Documentation Refactor Lead
- Viet lai tai lieu de agent doc nhanh, map nhanh, tra cuu nhanh.
- Moi file phai co muc dich, pham vi, input, output, role lien quan, va muc "khong duoc hieu sai".
```

## 3. Nguon Bat Buoc Phai Doc

```text
Nguon tai lieu bat buoc:

1. `V2/20260423_00_tong_quan_v2.md`
2. `V2/01_quy_trinh_agent_san_pham/20260423_04_quy_chuan_lam_viec_agent_v2.md`
3. `V2/01_quy_trinh_agent_san_pham/20260424_12_quy_trinh_aider_thuc_chien.md`
4. `V2/05_tai_lieu_mo_ta/20260423_01_mo_ta_he_thong_tong_hop.md`
5. `V2/05_tai_lieu_mo_ta/20260423_10_dac_ta_hop_nhat_va_chien_luoc_luu_tru.md`
6. `V2/05_tai_lieu_mo_ta/20260424_02_nghiep_vu_kiem_sat_vien_va_user_story.md`
7. `V2/05_tai_lieu_mo_ta/20260424_03_ban_do_module_va_luong_du_lieu.md`
8. `V2/05_tai_lieu_mo_ta/20260424_04_dac_ta_tim_kiem_sap_xep_va_trich_dan.md`
9. `V2/05_tai_lieu_mo_ta/20260424_05_ho_so_nguon_tai_lieu_va_chien_luoc_ingest.md`
10. `V2/05_tai_lieu_mo_ta/20260424_06_dac_ta_ai_offline_phan_tich_tai_lieu.md`

Neu moi truong dich co bo file Ops cho Aider, map them nhu sau:

- `Ops/CONVENTIONS.md` = quy tac viet/sua file, naming, header, boundary, patch style
- `Ops/AGENT_STANDARD_AIDER_CONTINUE_PILOT.md` = handoff standard, continue protocol, role transfer, session resume
- `Ops/PROJECT_RULES.md` = hard constraints, scope locks, folder boundary, testing gate, done criteria
- `Ops/PROMPT_SEQUENCE.md` = thu tu prompt bat buoc cho moi agent/session
```

## 4. Muc Tieu Dau Ra

```text
Ban phai tao ra dau ra o cap "team operating system", khong chi o cap "document cleanup".

Dau ra bat buoc:

A. Viet lai bo tai lieu mo ta du an sao cho:
- nghiep vu KSV sau hon,
- module ro hon,
- luong du lieu ro hon,
- mapping document/page/entity/citation ro hon,
- AI offline duoc rao scope dung,
- review queue, confidence, manual override duoc dac ta day du,
- moi file co acceptance criteria ro.

B. Tao hoac viet lai bo tai lieu quan ly Aider sao cho:
- agent biet doc gi truoc,
- biet vai tro cua minh,
- biet khi nao duoc sua docs, khi nao duoc sua code,
- biet cach tiep tuc mot session dang do,
- biet cach handoff giua architect / analyst / implementer / tester / fixer.

C. Neu tao bo `Ops/*.md`, phai phan ro:
- `CONVENTIONS.md`: quy uoc toan repo va cach patch
- `AGENT_STANDARD_AIDER_CONTINUE_PILOT.md`: quy trinh tiep tuc session, context carry-over, stop/resume, owner handoff
- `PROJECT_RULES.md`: hard rules khong duoc vi pham
- `PROMPT_SEQUENCE.md`: trinh tu prompt cho tung loai cong viec

D. Moi tai lieu phai chi ro:
- muc dich,
- ai doc,
- ai sua,
- ai phe duyet,
- input,
- output,
- dependency,
- done criteria,
- test/review gate neu co.
```

## 5. Quy Tac Aider Va Quan Ly Project

```text
Ban phai ap dung tu duy Aider vao quan ly project:

1. Repo Map First
- Luon bat dau bang viec xac dinh entry point, docs core, output code area, reporting area.
- Khong cho agent lang thang doc repo khong muc dich.

2. Role-Driven Execution
- Moi task phai co role chinh: architect, analyst, doc_refactor, frontend, backend, data_ai, qa, fixer, release.
- Khong cho mot agent vua tu nghiep vu, vua tu code, vua tu QA ma khong ghi ro role dang dong.

3. Small Patch Discipline
- Uu tien patch nho, ro file, ro block.
- Neu theo Aider, uu tien search/replace block hoac diff patch, tranh viet lai toan bo file neu khong can.

4. Test Gate
- `architect: true`
- `auto-test: true`
- `pretty: true`
- `test-cmd: powershell`
- Neu chua co test command that su, ban phai ghi ro "MISSING REAL TEST COMMAND" thay vi gia dinh bua.

5. Continue Protocol
- Moi session dang do phai co handoff package:
  - da doc gi,
  - dang sua gi,
  - file nao dang la source of truth,
  - rui ro gi con mo,
  - buoc tiep theo la gi.

6. Hard Boundary
- `V2` = prompt, SOP, docs, specs, reports, logs.
- `PhanMem` = code/runtime/build/test artifacts.
- Khong mo rong pham vi tu docs sang code neu chua co lenh ro.
```

## 6. Rang Buoc Cung

```text
Rang buoc khong duoc vi pham:

1. Khong viet mo ta chung chung kieu marketing.
2. Khong dung cum tu mo ho nhu "thong minh", "toi uu", "truc quan" neu khong co nghia nghiep vu cu the.
3. Khong duoc bo qua review queue, confidence, citation, va provenance.
4. Khong duoc de `PhanMem` thanh noi chua tai lieu.
5. Khong duoc suy doan co internet; he thong la offline-first.
6. Khong duoc tra loi bang mot ban tom tat ngan. Ban phai tao ra mot bo cau truc tai lieu co the dua vao van hanh that.
7. Khong duoc chi nghi o cap UI. Phai map toi data, workflow, module, vai tro, test gate, fix gate, report gate.
8. Neu mot file hien tai thieu thong tin, hay ghi ro "THIEU DAU VAO" va de xuat file bo sung can tao.
```

## 7. Dinh Dang Phan Hoi Bat Buoc

```text
Ban phai tra loi theo dung khuon sau:

1. SYSTEM UNDERSTANDING
- Ban hieu san pham nay la gi
- Dau ra nghiep vu cuoi cung la gi
- Nguoi dung chinh can gi

2. DOCUMENT GAPS
- File nao dang thieu
- File nao dang chong cheo
- File nao dang mo ho

3. TARGET DOCUMENT SET
- Danh sach file can giu
- Danh sach file can viet lai
- Danh sach file can them moi

4. ROLE MAP
- Architect
- Business Analyst
- Documentation Lead
- Frontend
- Backend
- Data/AI
- QA
- Fixer
- Release/Report

5. OPS / AIDER FILE PLAN
- `CONVENTIONS.md`
- `AGENT_STANDARD_AIDER_CONTINUE_PILOT.md`
- `PROJECT_RULES.md`
- `PROMPT_SEQUENCE.md`

6. FILE-BY-FILE REWRITE
- Cho moi file: muc dich, van de hien tai, cach viet lai, muc can bo sung, acceptance criteria

7. PATCH OR FULL REPLACEMENT
- Neu co the, dua patch/file content de thay the ngay

8. RISKS AND OPEN ITEMS
- Dau vao con thieu
- Cho nao can quyet dinh cua con nguoi

9. NEXT EXECUTION STEP
- Buoc 1 cu the nhat de team AI co the bat dau
```

## Ghi Chu Su Dung

- Prompt nay duoc viet de dua cho model ngoai co kha nang refactor tai lieu manh.
- Trong workspace hien tai, file nay la lenh goc moi nhat neu can huy dong model ngoai de viet lai bo nghiep vu va bo quy dinh Aider.
