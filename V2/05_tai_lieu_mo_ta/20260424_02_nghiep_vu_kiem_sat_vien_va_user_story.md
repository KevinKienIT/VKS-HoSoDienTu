# Nghiep Vu Kiem Sat Vien Va User Story

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dinh nghia ro nhu cau nghiep vu, user story va acceptance criteria de agent khong di lac sang huong ky thuat thuần tuy.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`
Nguon corpus: Toan bo file PDF trong `TaiLieu`

## Muc Luc

1. Vai tro nguoi dung
2. Muc tieu nghiep vu cot loi
3. User story uu tien cao
4. Acceptance criteria

## 1. Vai Tro Nguoi Dung

- KSV chinh: doc, tra cuu, tong hop, phan tich ho so.
- Nguoi review: xac nhan OCR, but luc, loai van ban, metadata.
- Quan tri noi bo: import case, quan ly bo du lieu, dong goi ban offline.

## 2. Muc Tieu Nghiep Vu Cot Loi

- Biet tung file trong case la gi.
- Tim tai lieu theo dung y do nghiep vu, khong chi theo filename.
- Sap xep danh sach tai lieu theo trinh tu can doc.
- Gan doi tuong, vat chung, su kien vao tai lieu.
- Hoi AI offline va nhan cau tra loi co trich dan.

## 3. User Story Uu Tien Cao

1. La KSV, toi muon mo mot case va thay ngay danh sach file cung ten hien thi de khong phai mo tung PDF.
2. La KSV, toi muon tim theo but luc, tu khoa OCR, loai van ban va ten doi tuong trong cung mot o tim kiem.
3. La KSV, toi muon sap xep tai lieu theo but luc, ngay van ban hoac thu tu import.
4. La KSV, toi muon click vao ket qua tim kiem va nhay den dung trang, dung but luc.
5. La KSV, toi muon AI offline tom tat mot tai lieu hoac tra loi cau hoi ve case, kem trich dan file/trang/but luc.
6. La reviewer, toi muon thay danh sach item nghi ngo de xac nhan nhanh.
7. La KSV, toi muon xem dossier doi tuong gom ly lich, tai lieu lien quan va vat chung lien ket.

## 4. Acceptance Criteria

- Khong co file nao chi hien filename timestamp neu he thong da OCR duoc.
- Ket qua tim kiem phai co preview va context.
- Moi thao tac sap xep phai hoat dong tren list lon.
- Moi cau tra loi AI phai co citation list.
- Moi item OCR/classification confidence thap phai vao review queue.

