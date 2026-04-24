# Ban Do Tai Lieu Va Lenh V2

Ngay tao: 2026-04-23
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Map tai lieu duoc dua vao V2 va xac dinh file nao la nguon lenh chinh.
Nguon prompt: `V2/00_prompt_tho/20260423_02_len_v2.md`

## Muc Luc

1. Thu tu uu tien lenh
2. Ban do prompt tho
3. Ban do tai lieu mo ta
4. Ban do tai lieu dieu phoi
5. Ghi chu van hanh

## 1. Thu Tu Uu Tien Lenh

Khi co nhieu file lenh, thu tu uu tien nhu sau:

1. Prompt tho moi nhat trong `V2/00_prompt_tho`
2. Quy chuan agent V2
3. Ke hoach V2
4. Ban do V2
5. Nhat ky agent V2

## 2. Ban Do Prompt Tho

| Nguon V1 | Dich V2 | Vai tro |
| --- | --- | --- |
| `V1/00PromptTho/260423_BuildMoTa.md` | `V2/00_prompt_tho/20260423_01_build_mo_ta.md` | Prompt nen ve san pham va quy chuan agent |
| `V1/00PromptTho/260423_LenV2.md` | `V2/00_prompt_tho/20260423_02_len_v2.md` | Lenh tao khong gian V2 va bo quy dinh van hanh |
| (nhap truc tiep) | `V2/00_prompt_tho/20260423_03_lenh_quy_chuan_ai_thong_minh.md` | Lenh xay dung quy trinh chuan day du cho team AI tu dong |
| (nhap truc tiep) | `V2/00_prompt_tho/20260424_04_lenh_viet_lai_nghiep_vu_va_quan_ly_aider_cho_model_manh.md` | Lenh manh de dua cho model ngoai viet lai bo mo ta nghiep vu va bo dieu phoi Aider theo vai tro |

## 3. Ban Do Tai Lieu Mo Ta

5 file cu da duoc hop nhat thanh 1 file chuan duy nhat:

| Nguon goc (da xoa) | Dich V2 hien tai | Ghi chu |
| --- | --- | --- |
| `he_thong_quan_ly_ho_so.md` (root) | DA GOP | File goc da xoa (trung 100% voi ban V2) |
| `20260423_01_he_thong_quan_ly_ho_so.md` | DA GOP | |
| `20260423_02_kien_truc_he_thong.md` | DA GOP | |
| `20260423_03_quy_tac_phan_loai.md` | DA GOP | |
| `20260423_04_huong_dan_su_dung.md` | DA GOP | |
| `20260423_05_tong_hop_setup_va_su_dung.md` | DA GOP | |

File hien tai:

| File V2 | Vai tro |
| --- | --- |
| `V2/05_tai_lieu_mo_ta/20260423_01_mo_ta_he_thong_tong_hop.md` | Mo ta day du he thong, san sang cho thiet ke giao dien |
| `V2/05_tai_lieu_mo_ta/20260423_10_dac_ta_hop_nhat_va_chien_luoc_luu_tru.md` | Chien luoc luu tru, hop nhat du lieu va tang tim kiem |
| `V2/05_tai_lieu_mo_ta/20260424_02_nghiep_vu_kiem_sat_vien_va_user_story.md` | Nghiep vu nguoi dung, user story va muc tieu thao tac |
| `V2/05_tai_lieu_mo_ta/20260424_03_ban_do_module_va_luong_du_lieu.md` | Map module, luong du lieu va ranh gioi thanh phan |
| `V2/05_tai_lieu_mo_ta/20260424_04_dac_ta_tim_kiem_sap_xep_va_trich_dan.md` | Nghiep vu tim kiem, sap xep, citation va ket qua tra cuu |
| `V2/05_tai_lieu_mo_ta/20260424_05_ho_so_nguon_tai_lieu_va_chien_luoc_ingest.md` | Ho so nguon du lieu thuc te va chien luoc ingest |
| `V2/05_tai_lieu_mo_ta/20260424_06_dac_ta_ai_offline_phan_tich_tai_lieu.md` | Dac ta AI offline, phan tich tai lieu va tra loi co trich dan |

## 4. Ban Do Tai Lieu Dieu Phoi

| Tai lieu V2 | Vai tro |
| --- | --- |
| `20260423_01_ke_hoach_chuyen_len_v2.md` | Ke hoach tao V2 |
| `20260423_02_ban_do_tai_lieu_va_lenh_v2.md` | Ban do nguon lenh va tai lieu |
| `20260423_03_nhat_ky_agent_v2.md` | Log cac thay doi cua agent |
| `20260423_04_quy_chuan_lam_viec_agent_v2.md` | Quy tac lam viec chinh thuc cua agent trong V2 |

## 5. Ghi Chu Van Hanh

- Tai lieu trong `05_tai_lieu_mo_ta` duoc giu de tham chieu, khong phai noi de phat lenh.
- Moi lenh moi cua nguoi dung phai duoc dua vao `00_prompt_tho` truoc khi agent xu ly theo V2.
- Neu co prompt moi ma chua duoc map, agent phai cap nhat file nay va nhat ky.
- `PhanMem` khong con la noi chua tai lieu. Tat ca prompt, SOP, mo ta, dac ta, report va log deu phai dat trong `V2`.

