# Quy Trinh Test Nhieu Vong

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-23
Sua boi agent: Codex
Muc dich: Dinh nghia cach test theo nhieu vong va check vong bat buoc cho team AI.
Nguon prompt: `V2/00_prompt_tho/20260423_02_len_v2.md`

## Muc Luc

1. Muc tieu
2. Dieu kien vao vong test
3. Cac vong test
4. Dieu kien fail va quay vong
5. Dieu kien dong

## 1. Muc Tieu

Bao dam mot thay doi khong chi duoc "test qua loa", ma phai qua nhieu vong check, trong do co it nhat 1 vong doc lap.

## 2. Dieu Kien Vao Vong Test

Chi vao test khi:

- task da co owner,
- specs da ro,
- implementer da self check,
- co note thay doi,
- co danh sach pham vi can test.

## 3. Cac Vong Test

### Vong 0 - Self Test

Owner: implementer

Bat buoc:

- check logic vua sua,
- chay unit test lien quan,
- check loi ro rang,
- note rui ro tu danh gia.

### Vong 1 - Peer Review Cung Nhom

Owner: peer cung domain

Bat buoc:

- doc specs,
- doc note thay doi,
- check nhanh dung/sai trong domain do,
- ghi lai diem nghi ngo neu co.

### Vong 2 - Function/Integration Test

Owner: agent_test_function

Bat buoc:

- test input/output chinh,
- test luong bien,
- test integration voi module lien quan,
- test regression khu vuc anh huong.

### Vong 3 - UI/User Flow Test

Owner: agent_test_giao_dien

Bat buoc:

- layout,
- interaction,
- state change,
- empty/loading/error,
- responsive neu co giao dien.

### Vong 4 - QA Doc Lap

Owner: agent_qa_doc_lap

Bat buoc:

- khong la nguoi implement thay doi do,
- doi chieu prompt, plan, report, test,
- tim bug bo sot,
- ket luan: cho qua, yeu cau fix, hoac mo lai scope.

### Vong 5 - Retest Sau Fix

Owner: tester da phat hien bug + implementer phoi hop

Bat buoc:

- test lai bug goc,
- test regression toi thieu,
- cap nhat trang thai bug.

## 4. Dieu Kien Fail Va Quay Vong

Cong viec phai quay ve fix loop neu:

- sai acceptance criteria,
- co bug chuc nang,
- co bug UI anh huong flow,
- test khong co bang chung,
- QA doc lap khong thong qua,
- fix moi lam phat sinh regression.

## 5. Dieu Kien Dong

Mot thay doi qua test khi:

- da qua cac vong can thiet cho pham vi cua no,
- co report test/fix ro rang,
- bug con lai da duoc ghi trang thai,
- leader chap nhan dong dot viec neu pham vi lon.

