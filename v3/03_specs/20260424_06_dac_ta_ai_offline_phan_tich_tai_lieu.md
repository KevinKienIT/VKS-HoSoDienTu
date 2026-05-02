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
- goi y doc chu viet tay mo tren to loi khai, don viet tay va ghi chu tay bang candidate reading bundle

AI offline khong duoc dung de:

- tu suy dien su that khi khong co bang chung
- bo qua tai lieu goc
- xoa nhu cau review cua con nguoi
- bien phan doan/noi suy chu viet tay thanh text chac chan ma khong canh bao

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

### Mode D - Handwriting Assist

- dung cho vung text viet tay kho doc
- nhan `page crop` + `ocr candidates` + `ocr_revision_id`
- tra ve 2-3 candidate cach doc, span nghi van, va de xuat noi suy neu co
- chi duoc xem la suggestion bundle, khong phai ket qua da duoc phe duyet

## 3. Input Va Output

Input toi thieu:

- query cua KSV
- case scope
- danh sach tai lieu lien quan
- citation-ready context
- page/image crop + region_bbox neu bai toan lien quan text viet tay
- OCR candidate bundle + uncertainty metadata neu co

Output toi thieu:

- answer
- summary
- citation list
- confidence / warning
- danh sach cho review neu context mo ho
- work product draft neu user yeu cau tong hop/timeline/dossier/report
- candidate readings / uncertain spans / interpolation suggestion neu query lien quan text viet tay
- review_required = true neu co span nghi van hoac noi suy

## 4. Rule An Toan Nghiep Vu

- Khong co citation -> khong dua ra ket luan manh.
- OCR confidence thap -> phai canh bao.
- Neu co xung dot giua cac tai lieu -> phai neu ro xung dot.
- Neu query vuot qua tai lieu da ingest -> phai noi khong du bang chung.
- AI khong duoc tu cap nhat metadata, dossier hay report chinh thuc ma khong qua review.
- Moi de xuat doc chu viet tay phai kem `source_page`, `region_bbox`, `ocr_revision_id`, `confidence`.
- Neu co nhieu cach doc hop ly, AI phai liet ke candidate thay vi chot 1 cach doc duy nhat.
- Noi suy text viet tay chi duoc tra ve duoi dang de xuat `pending_review`.

## 5. Tier Cong Nghe Khuyen Nghi

- Tier 1: rule + extractive + FTS5
- Tier 2: retrieval + rerank offline
- Tier 3: local LLM (Qwen / Gemma / Llama dang local, tuy theo hardware)

Ghi chu cho text viet tay:

- OCR engine van la lop xu ly chinh.
- Local AI qua Ollama adapter chi la lop ho tro de tao candidate reading bundle cho span mo.
- Khong duoc dung AI local de thay the OCR goc hay bo qua review queue.

Thu tu uu tien trien khai:

1. Baseline extractive co citation
2. Retrieval hoat dong on dinh
3. Local LLM chi them sau khi citation pipeline da chac

