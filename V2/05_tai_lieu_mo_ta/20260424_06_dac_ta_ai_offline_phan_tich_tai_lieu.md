# Dac Ta AI Offline Phan Tich Tai Lieu

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Chot AI offline phai lam gi, khong duoc lam gi va phai tra ket qua nhu the nao.
Nguon prompt: `V2/00_prompt_tho/20260423_01_build_mo_ta.md`

## Muc Luc

1. Vai tro AI offline
2. Cac mode hoat dong
3. Input va output
4. Rule an toan nghiep vu
5. Tier cong nghe khuyen nghi

## 1. Vai Tro AI Offline

AI offline duoc dung de:

- tom tat document
- tom tat case
- hoi dap tren ho so
- trich xuat doi tuong, vat chung, su kien
- goi y dossier va timeline

AI offline khong duoc dung de:

- tu suy dien su that khi khong co bang chung
- bo qua tai lieu goc
- xoa nhu cau review cua con nguoi

## 2. Cac Mode Hoat Dong

### Mode A - Extractive Baseline

- dung cho may yeu
- lay cau quan trong, keyword, section heading
- luon co san

### Mode B - Retrieval + Citation

- tim trang lien quan
- rank context
- tra ve answer ngan + citation list

### Mode C - Local LLM Analysis

- tong hop dossier
- hoi dap da buoc
- so sanh nhieu tai lieu
- chi duoc tra ket qua neu co citation bundle

## 3. Input Va Output

Input toi thieu:

- query cua KSV
- case scope
- danh sach tai lieu lien quan
- citation-ready context

Output toi thieu:

- answer
- summary
- citation list
- confidence / warning
- danh sach cho review neu context mo ho

## 4. Rule An Toan Nghiep Vu

- Khong co citation -> khong dua ra ket luan manh.
- OCR confidence thap -> phai canh bao.
- Neu co xung dot giua cac tai lieu -> phai neu ro xung dot.
- Neu query vuot qua tai lieu da ingest -> phai noi khong du bang chung.

## 5. Tier Cong Nghe Khuyen Nghi

- Tier 1: rule + extractive + FTS5
- Tier 2: retrieval + rerank offline
- Tier 3: local LLM (Qwen / Gemma / Llama dang local, tuy theo hardware)

Thu tu uu tien trien khai:

1. Baseline extractive co citation
2. Retrieval hoat dong on dinh
3. Local LLM chi them sau khi citation pipeline da chac

