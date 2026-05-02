# TỔNG HỢP HỢP NHẤT TOÀN BỘ Ý KIẾN VÀ HƯỚNG TRIỂN KHAI VKS ECMS

Ngày tạo: 2026-04-29  
Người tổng hợp: Roo (Code mode)

## 1) Mục tiêu của tài liệu này

Tài liệu này hợp nhất các nội dung rải rác trong nhiều tài liệu thành một nguồn duy nhất để triển khai. Cách viết đi theo đúng yêu cầu: xử lý tuần tự theo từng tệp, diễn giải cách làm, hành động cụ thể, cơ chế agent xử lý, cơ chế code xử lý, đề xuất agent online cho công việc lập trình, và đề xuất agent/model offline cho vận hành phần mềm desktop.

Toàn bộ định hướng tuân thủ nền tảng Tauri offline đã chốt trong [`20260426_01_quyet_dinh_nen_tang_va_cach_chay.md`](V2/05_tai_lieu_mo_ta/20260426_01_quyet_dinh_nen_tang_va_cach_chay.md).

---

## 2) Chuỗi tài liệu đã đối chiếu và cách xử lý tuần tự

### 2.1 Tệp nền tảng vận hành tài liệu V2

Nguồn: [`20260423_00_tong_quan_v2.md`](V2/20260423_00_tong_quan_v2.md)

Việc xử lý bắt đầu từ tệp này vì đây là entry point của toàn bộ V2, quy định rõ ranh giới giữa tài liệu điều phối và code runtime. Khi triển khai thực tế, mọi tác vụ phải đi theo trình tự đọc SOP rồi mới vào code, vì nếu đảo thứ tự sẽ dẫn đến sai phạm vi, ví dụ đưa nội dung điều phối sang khu vực runtime hoặc kiểm thử sai môi trường.

Hành động cụ thể cần làm:
1. Khóa quy tắc “V2 chứa điều phối, `PhanMem` chứa runtime” thành checklist bắt buộc trước mỗi phiên sửa code.
2. Mọi tài liệu mới phát sinh trong điều phối dự án đặt theo chuẩn đặt tên của V2, còn file tổng hợp hợp nhất này được đặt theo yêu cầu trực tiếp là [`tonghop.md`](tonghop.md).
3. Mọi thay đổi code chỉ được xem là hợp lệ sau khi đối chiếu lại quy tắc vận hành trong tệp trên.

### 2.2 Tệp quyết định nền tảng và cách chạy

Nguồn: [`20260426_01_quyet_dinh_nen_tang_va_cach_chay.md`](V2/05_tai_lieu_mo_ta/20260426_01_quyet_dinh_nen_tang_va_cach_chay.md)

Tệp này giải quyết dứt điểm nhầm lẫn web/browser so với desktop runtime. Cách xử lý ở mức kỹ thuật là mọi luồng nghiệp vụ có gọi Tauri API phải được kiểm chứng trong runtime Tauri, không dùng browser preview để xác nhận chức năng.

Hành động cụ thể cần làm:
1. Mọi test chức năng bắt buộc chạy bằng `npm run tauri:dev` hoặc build bằng `npm run tauri:build`.
2. Trong frontend, mọi chỗ gọi backend phải dùng pattern an toàn kiểu [`safeInvoke`](PhanMem/src/services/scanService.ts) (fallback + log rõ ngữ cảnh).
3. Thiết lập cổng kiểm soát CI nội bộ: nếu phát hiện luồng test dùng browser thuần cho chức năng dữ liệu thì đánh fail checklist.

### 2.3 Tệp đặc tả ingest nguồn tài liệu

Nguồn: [`20260424_05_ho_so_nguon_tai_lieu_va_chien_luoc_ingest.md`](V2/05_tai_lieu_mo_ta/20260424_05_ho_so_nguon_tai_lieu_va_chien_luoc_ingest.md)

Tệp này mô tả bản chất dữ liệu đầu vào: tên file không mang nghĩa nghiệp vụ, cần tạo metadata sau OCR/phân loại. Cách xử lý đúng là ingest phải lưu provenance đầy đủ, sau đó mới phân tích semantic và dựng tên hiển thị nghiệp vụ.

Hành động cụ thể cần làm:
1. Mở rộng logic phân loại sau ingest ở [`classify_document_type_from_name()`](PhanMem/src-tauri/src/commands/import_cmd.rs:128) bằng lớp phân loại nội dung (không chỉ tên file).
2. Bảo toàn trường bắt buộc khi ingest: hash, page_count, sequence, source folder.
3. Bổ sung cơ chế cảnh báo “dataset chưa sẵn sàng tra cứu” nếu thiếu trường cốt lõi.

### 2.4 Tệp đặc tả AI offline

Nguồn: [`20260424_06_dac_ta_ai_offline_phan_tich_tai_lieu.md`](V2/05_tai_lieu_mo_ta/20260424_06_dac_ta_ai_offline_phan_tich_tai_lieu.md)

Tệp này định nghĩa rõ: AI chỉ được phép kết luận trong biên chứng cứ có trích dẫn, và phải có chế độ fallback extractive. Đây là khung điều khiển bắt buộc cho agent offline.

Hành động cụ thể cần làm:
1. Giữ nguyên fallback hiện có trong [`ai_ask_case()`](PhanMem/src-tauri/src/commands/ai_cmd.rs:317) và [`ai_summarize_case()`](PhanMem/src-tauri/src/commands/ai_cmd.rs:281).
2. Bổ sung bước validator sau khi draft để phát hiện câu không có citation.
3. Đưa toàn bộ kết quả nghi vấn chữ viết tay vào luồng `pending_review`, không tự động hợp thức hóa.

### 2.5 Tệp mẫu đầu ra Word/HTML người dùng yêu cầu mô phỏng

Nguồn:
- [`Bang_chi_muc_trich_dan_tai_lieu.docx`](V2/tai lieu/Bang_chi_muc_trich_dan_tai_lieu.docx)
- [`Bao_cao_tong_hop_vu_an_LeThanhCong_D134.docx`](V2/tai lieu/Bao_cao_tong_hop_vu_an_LeThanhCong_D134.docx)
- [`So_do_vu_an_LeThanhCong_D134.html`](V2/tai lieu/So_do_vu_an_LeThanhCong_D134.html)

Ba tệp này được xử lý như “chuẩn đầu ra đích”. Về kỹ thuật, không thể chỉ dùng 1 prompt tổng quát để sinh đúng định dạng; cần pipeline tách cấu trúc rồi mới render bằng template compiler.

Hành động cụ thể cần làm:
1. Tạo parser DOCX trích heading/section/table-like row.
2. Tạo parser HTML trích cards/timeline/relation/table.
3. Chuẩn hóa thành schema trung gian, rồi render lại thành output chuẩn (DOCX/HTML/PDF).
4. Bổ sung lớp kiểm định trước export: thiếu phần mục lục, timeline, căn cứ pháp lý thì không cho đóng gói final.

### 2.6 Tệp kế hoạch kiến trúc đã lập

Nguồn: [`claude.md`](plans/claude.md)

Đây là bản thiết kế chi tiết đã gom gap analysis, model stack, module triển khai, và test gates. Cách xử lý tiếp theo là biến từng module thành milestone có đầu ra code đo được.

Hành động cụ thể cần làm:
1. Tách module A→F thành backlog kỹ thuật có owner và done criteria.
2. Ưu tiên theo trục: parser + citation trước, LLM orchestration sau.
3. Bắt buộc regression sau từng module theo bộ lệnh build Tauri.

### 2.7 Tệp nhật ký triển khai và checklist fix

Nguồn:
- [`20260428_09_phase1_pipeline_impl_log.md`](V2/03_bao_cao/03_fix/20260428_09_phase1_pipeline_impl_log.md)
- [`20260428_10_phase_next_task_checklist.md`](V2/03_bao_cao/03_fix/20260428_10_phase_next_task_checklist.md)

Hai tệp này cho biết pipeline đã có nền tảng bền vững: lifecycle job/task/event/checkpoint, hardening file-lock/OCR env, purge an toàn, và build pass nhiều vòng. Nghĩa là hệ thống không bắt đầu từ số 0; giai đoạn tới là nâng chiều sâu semantic và chất lượng report.

Hành động cụ thể cần làm:
1. Giữ nguyên các phần đã ổn định trong [`pipeline_execution_tick()`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1402) để tránh regression.
2. Chèn parser/LLM orchestration vào đúng điểm mở rộng thay vì viết luồng song song mới.
3. Duy trì kỷ luật log giống tệp phase log để truy hồi lỗi theo bước.

---

## 3) Cách vận hành kỹ thuật tổng thể (không tóm tắt đầu dòng)

Hệ thống vận hành theo chuỗi: ingest tài liệu gốc, bóc tách nội dung theo loại (PDF/ảnh qua OCR, DOCX qua parser cấu trúc, HTML qua parser DOM), chuẩn hóa vào SQLite thành bằng chứng có định danh, chạy truy hồi lai (keyword + embedding), sau đó mới giao cho lớp suy luận local LLM để viết bản nháp. Bản nháp không được đi thẳng ra kết quả cuối mà phải đi qua lớp validator citation. Nếu câu nào không chứng minh được bằng anchor nguồn thì buộc đánh cờ yêu cầu review. Khi toàn bộ section đã đạt điều kiện, template compiler mới render thành mẫu giống văn bản Word/HTML đích. Cuối cùng dữ liệu vào queue duyệt nghiệp vụ để người dùng xác nhận trước khi đóng gói xuất.

Ở tầng code, Rust command layer điều phối vòng đời job và transaction dữ liệu; Python xử lý OCR/parser semantic vì linh hoạt thư viện; frontend giữ vai trò hiển thị trạng thái, cấu hình model offline, và điều phối thao tác người dùng theo safe invoke. Nhờ vậy hệ thống giữ được tính offline tuyệt đối, nhưng vẫn có khả năng mở rộng chất lượng phân tích theo phần cứng.

---

## 4) Độ mạnh tác vụ và gợi ý agent AI online cho viết code

Lưu ý: phần mềm mục tiêu chạy offline, nhưng giai đoạn phát triển có thể dùng agent online để tăng tốc coding/review.

Đề xuất phân vai theo độ khó:

Tác vụ mức rất nặng (thiết kế kiến trúc, refactor xuyên Rust+TS+Python, migration, tối ưu pipeline):
- dùng model/agent coding mạnh cấp kiến trúc như GPT-5 class hoặc Claude Opus class để thiết kế và cắt milestone.

Tác vụ mức nặng vừa (viết module parser, service wrapper, test integration):
- dùng agent coding trung-cao như Claude Sonnet class hoặc GPT-4.1 class để triển khai khối lượng ổn định.

Tác vụ mức lặp/chuẩn hóa (rename, DTO sync, docs checklist, scaffold test):
- dùng agent code nhẹ hơn để tiết kiệm chi phí, sau đó bắt buộc review bằng agent mạnh trước merge.

Quy trình phối hợp khuyến nghị:
1. Agent mạnh tạo spec kỹ thuật + acceptance tests.
2. Agent trung bình triển khai code theo spec.
3. Agent mạnh chạy review diff và quét rủi ro hồi quy.

---

## 5) Đề xuất agent/model AI offline cho bản desktop vận hành thực tế

Trong runtime offline, đề xuất dùng Ollama làm bộ điều phối local model, giữ fallback extractive hiện có ở [`load_case_context()`](PhanMem/src-tauri/src/commands/ai_cmd.rs:88).

Lộ trình model:
1. Hồ sơ máy 32GB RAM: ưu tiên Qwen2.5 14B quantized để có chất lượng tổng hợp cao.
2. Hồ sơ máy 16GB RAM: dùng Qwen2.5 7B quantized, tăng cường retrieval/rerank để bù reasoning.
3. Máy thấp hơn: chạy extractive + citation strict, tắt chế độ sinh văn bản dài nếu thiếu tài nguyên.

Để “học sâu và luận text tốt hơn” trong ngữ cảnh offline pháp lý, trọng tâm không phải chỉ đổi model lớn hơn mà phải tăng chất lượng evidence index, chunking, rerank, và validator citation. Nếu thiếu lớp này, model mạnh vẫn dễ tạo câu khó kiểm chứng.

---

## 6) Kế hoạch hành động tiếp theo theo thứ tự triển khai

Bước 1: hoàn thiện parser DOCX/HTML và schema trung gian.  
Bước 2: dựng hybrid retrieval + rerank local và áp ràng buộc citation.  
Bước 3: bổ sung orchestration command cho local LLM có profile phần cứng.  
Bước 4: xây template compiler xuất đúng ba nhóm output chuẩn.  
Bước 5: thêm trang cấu hình model offline + trạng thái pipeline theo phase.  
Bước 6: chạy regression full Tauri, chốt bằng build artifact và báo cáo đối chiếu.

Các bước này phải bám vào log/checklist đã có trong [`20260428_09_phase1_pipeline_impl_log.md`](V2/03_bao_cao/03_fix/20260428_09_phase1_pipeline_impl_log.md) và [`20260428_10_phase_next_task_checklist.md`](V2/03_bao_cao/03_fix/20260428_10_phase_next_task_checklist.md), không tạo nhánh quy trình mới song song.

---

## 7) Kết luận vận hành

Hệ thống hiện đã có nền pipeline ổn định và cơ chế an toàn tốt. Phần còn lại để đạt đúng kỳ vọng người dùng là nâng cấp khả năng đọc hiểu đa định dạng và sinh đầu ra chuẩn hồ sơ tố tụng bằng kiến trúc evidence-first, citation-first. Khi triển khai đúng thứ tự ở tài liệu này, phần mềm sẽ vừa giữ offline tuyệt đối, vừa nâng đáng kể chất lượng phân tích và chất lượng báo cáo.

---

## 8) Tái tổng hợp theo nguồn `plans/` (đã cập nhật theo yêu cầu mới)

Phần này cập nhật trực tiếp theo yêu cầu “trong thư mục `V2/plan` là toàn bộ phần cần tổng hợp lại”, sau khi kiểm tra thực tế hệ thống thì thư mục có dữ liệu là [`plans/`](plans), và các tệp đã đọc gồm [`claude.md`](plans/claude.md), [`claude2.md`](plans/claude2.md), [`claude3`](plans/claude3), [`claude4.md`](plans/claude4.md). Hai đường dẫn [`plans/gemini`](plans/gemini) và [`plans/gpt`](plans/gpt) hiện chưa có tệp nội dung.

### 8.1 Nội dung bổ sung từ `claude2.md` và cách tích hợp

Từ [`claude2.md`](plans/claude2.md), hệ thống được đẩy sâu hơn vào tính thực chiến vận hành hồ sơ giấy: module Ricoh scan inbox, pipeline OCR đầy đủ (render, preprocess, OCR, layout, field extraction), contract dữ liệu OCR chi tiết theo block, chuẩn đặt tên file động theo template, và tiêu chí hoàn thành ở cả cấp tài liệu lẫn cấp hồ sơ.

Khi đưa vào triển khai kỹ thuật, phần giá trị lớn nhất của tài liệu này là định nghĩa “không giả lập thành công”. Điều đó nghĩa là UI phải phản ánh đúng trạng thái engine/model thật, không hiển thị kết quả AI/OCR hoàn tất khi chưa chạy engine thực. Trong code, yêu cầu này được ánh xạ thành cơ chế status rõ ràng ở pipeline và các cờ review bắt buộc khi confidence thấp.

Hành động cần làm ngay sau khi đồng bộ `claude2`:
1. Chuẩn hóa hợp đồng dữ liệu OCR block theo cấu trúc nêu trong [`claude2.md`](plans/claude2.md).
2. Bổ sung scan inbox workflow theo trạng thái Detected → Stable → Imported → OCR done.
3. Thêm template engine đặt tên file có sanitizer tiếng Việt và ký tự cấm hệ điều hành.
4. Chốt tiêu chí DONE theo checklist dữ liệu đầy đủ ở cấp document và case.

### 8.2 Nội dung bổ sung từ `claude3` và cách tích hợp

Từ [`claude3`](plans/claude3), tài liệu làm rõ kiến trúc state machine bất đồng bộ với từng phase agent tách vai rõ ràng: Intake/OCR, Classification, Evidence Extraction, Structuring/Timeline, Export. Điểm mạnh là nguyên tắc “citation-first hard stop” và “graceful degradation” được mô tả như cơ chế vận hành bắt buộc chứ không phải tùy chọn.

Khi tích hợp vào hệ thống hiện tại, phần này giúp khóa chặt tính đúng đắn pháp lý. Nếu một kết luận không có citation hợp lệ thì phải dừng lưu kết quả và phát sự kiện lỗi integrity. Đồng thời, nếu local LLM unavailable hoặc OOM thì tự rơi về minimal mode (FTS + template cứng), vẫn cho hệ thống chạy được nhưng giảm mức tự động sinh văn bản.

Hành động cần làm:
1. Tạo lớp citation integrity validator ở cuối mỗi agent output.
2. Chuẩn hóa payload `citations[]` bắt buộc với `document_id`, `page_number`, `quote`, `confidence`.
3. Ràng buộc chế độ degrade tự động bằng profile phần cứng và kiểm tra trạng thái model.
4. Tích hợp cờ low-confidence overlay cho viewer khi OCR < ngưỡng.

### 8.3 Nội dung bổ sung từ `claude4.md` và cách tích hợp

Từ [`claude4.md`](plans/claude4.md), nội dung tập trung vào tối ưu thực thi đa luồng, chống đơ UI, đồng bộ event từ backend lên frontend, xử lý vùng chữ viết tay (HTR branch), và tối ưu vòng đọc theo bbox để đảm bảo thứ tự văn bản logic.

Khi đưa vào codebase, các phần phù hợp nhất là: kiểm soát concurrency theo số core CPU, dùng event emitter để không block UI, và thêm pipeline handwriting riêng thay vì ép OCR in thường xử lý tất cả vùng chữ. Điều này đặc biệt quan trọng với bút lục/chữ ký/ghi chú tay.

Hành động cần làm:
1. Chuẩn hóa worker pool giới hạn theo CPU profile để giữ mượt UI.
2. Dùng event phát tiến độ theo file/page/phase thay vì polling mù.
3. Bổ sung nhánh HTR cho vùng handwriting sau layout detect.
4. Lưu confidence theo block để điều khiển ưu tiên review.

### 8.4 Hợp nhất `claude.md` + `claude2` + `claude3` + `claude4` thành bản triển khai cuối

Sau khi đối chiếu bốn tài liệu trong [`plans/`](plans), bản hợp nhất cuối cho VKS ECMS được chốt như sau: giữ khung kiến trúc trong [`claude.md`](plans/claude.md) làm xương sống; lấy contract dữ liệu và checklist vận hành thực chiến từ [`claude2.md`](plans/claude2.md); lấy guardrails pháp lý và state machine agent từ [`claude3`](plans/claude3); lấy kỹ thuật hiệu năng và handwriting pipeline từ [`claude4.md`](plans/claude4.md). Như vậy không chồng chéo, không mâu thuẫn, và mỗi tài liệu đóng một lớp chức năng rõ ràng.

Về phân công thực thi, cách tối ưu là giao cho agent online mạnh xử lý phần thiết kế liên tầng và refactor khó; agent online trung bình triển khai module rõ yêu cầu; còn trong runtime sản phẩm thì dùng agent/model offline theo profile máy với bắt buộc citation validator trước export chính thức. Đây là cách giữ cân bằng giữa tốc độ phát triển và an toàn vận hành pháp lý.

---

## 9) Bổ sung bắt buộc: chia nhân CPU để xử lý và cấu hình tự động theo máy mạnh/yếu

Phần này bổ sung trực tiếp theo phản hồi còn thiếu: cách chia nhân CPU cho pipeline và cấu hình agent offline theo sức máy.

### 9.1 Nguyên tắc chia nhân CPU cho pipeline không đơ UI

Luồng xử lý phải tách khỏi UI thread. Số worker xử lý nền không được lấy toàn bộ CPU, mà phải chừa tài nguyên cho hệ điều hành, Tauri runtime, và thao tác người dùng trong viewer/search.

Công thức khuyến nghị:
1. `cpu_total = số luồng logic (logical cores)`.
2. `cpu_reserved = max(2, round(cpu_total * 0.25))`.
3. `cpu_workers = clamp(cpu_total - cpu_reserved, min=1, max=12)`.

Ý nghĩa thực thi:
- Máy 4 luồng: reserved 2, workers 2.
- Máy 8 luồng: reserved 2, workers 6.
- Máy 16 luồng: reserved 4, workers 12.

### 9.2 Chia worker theo phase xử lý hồ sơ

Không cấp đều cho mọi phase, mà ưu tiên theo độ nặng:

`Intake/Hash/Copy`:
- nhẹ I/O; dùng 1–2 worker cố định.

`PDF render + preprocess + OCR`:
- nặng nhất; dùng 55–65% tổng `cpu_workers`.

`Layout + Field extraction`:
- trung bình; dùng 20–25% `cpu_workers`.

`Classification + Evidence extraction`:
- trung bình; dùng 10–15% `cpu_workers`.

`Export compile`:
- nhẹ hơn OCR; dùng 1–2 worker, chạy ưu tiên thấp để không giật UI.

### 9.3 Cơ chế tự động điều phối theo tải máy

Khi CPU > 85% liên tục 15 giây hoặc RAM còn trống < 15%, scheduler phải tự hạ concurrency một nấc. Khi CPU < 65% liên tục 30 giây và queue còn nhiều, scheduler mới tăng lại một nấc. Cơ chế này tránh dao động và giảm nguy cơ treo giao diện khi người dùng vừa làm việc vừa scan/OCR.

### 9.4 Gợi ý agent/model offline theo cấu hình máy (scan + phân tích hồ sơ)

#### Hồ sơ A — Máy yếu
- CPU: 4 luồng đến 8 luồng.
- RAM: 8–16 GB.
- GPU: không có hoặc VRAM < 4 GB.

Thiết lập đề xuất:
1. OCR: PaddleOCR CPU mode, batch nhỏ.
2. HTR: chỉ bật theo vùng nghi vấn (on-demand), không chạy toàn trang.
3. AI phân tích: chế độ Minimal/Desktop-lite (FTS + rule + template), hoặc Qwen 2.5 3B/7B quantized nếu đủ RAM.
4. Worker: 2–4 OCR workers tùy nhiệt độ/tải máy.
5. Export: ưu tiên template deterministic, giảm LLM sinh văn tự do.

#### Hồ sơ B — Máy trung bình
- CPU: 8–12 luồng.
- RAM: 16–24 GB.
- GPU: 4–8 GB VRAM (nếu có).

Thiết lập đề xuất:
1. OCR: PaddleOCR + preprocess đầy đủ.
2. HTR: bật theo trigger confidence thấp.
3. AI phân tích: Qwen 2.5 7B quantized + retrieval + citation validator.
4. Worker: 5–8 workers tổng, trong đó OCR chiếm đa số.
5. RAG: bật mặc định cho phần summary có kiểm chứng nguồn.

#### Hồ sơ C — Máy mạnh
- CPU: từ 16 luồng trở lên.
- RAM: 32 GB trở lên.
- GPU: >= 8 GB VRAM.

Thiết lập đề xuất:
1. OCR: pipeline đầy đủ, batch lớn hơn, render DPI cao hơn khi cần.
2. HTR: bật rộng hơn cho nhóm tài liệu có nhiều chữ viết tay.
3. AI phân tích: Qwen 2.5 14B quantized (hoặc tương đương) + embedding + rerank + citation hard-stop.
4. Worker: 10–12 workers tổng (giới hạn an toàn như mục 9.1).
5. Export: cho phép summary sâu hơn, nhưng vẫn khóa mọi kết luận bằng citation.

### 9.5 Agent offline nên chạy theo vai trò gì

Cho cả máy mạnh và máy yếu, vai trò agent giữ nguyên, chỉ khác mức tự động:
1. Intake/OCR Agent.
2. Classification Agent.
3. Evidence Extraction Agent.
4. Timeline/Structuring Agent.
5. Citation QA Agent (bắt buộc mọi profile máy).
6. Export Agent.

Khác biệt chính:
- Máy yếu: giảm hoặc tắt Free-form generation, ưu tiên extractive/template.
- Máy mạnh: bật phân tích sâu hơn nhưng không bỏ `Citation QA Agent`.

### 9.6 Bổ sung vào vận hành hiện tại của codebase

Phần chia CPU và profile máy cần được neo vào scheduler pipeline hiện có, không mở nhánh mới. Nghĩa là triển khai tại lớp điều phối đã dùng trong [`pipeline_execution_tick()`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1402), đồng thời mở cấu hình trong Settings để người dùng chọn profile `weak / balanced / strong` và chế độ `auto-tune`.

Khi `auto-tune` bật, app tự đo năng lực máy, chọn profile ban đầu, rồi điều chỉnh động theo tải tại mục 9.3. Khi `auto-tune` tắt, app tuân thủ profile người dùng chọn để ổn định hành vi trong môi trường nghiệp vụ yêu cầu dự đoán trước.

---

## 10) Bổ sung vòng đời lưu file hồ sơ: file gốc, file đang xử lý, file sau chỉnh sửa và hỗ trợ export

Phần này bổ sung trực tiếp theo yêu cầu còn thiếu: quy tắc lưu file trên máy người dùng theo từng trạng thái xử lý, và cách đóng gói export để người dùng nhận hồ sơ rõ ràng, dễ kiểm tra, dễ bàn giao.

### 10.1 Mục tiêu quản trị file

Hệ thống phải tách riêng ba lớp dữ liệu: bản gốc bất biến để làm chứng cứ, bản đang xử lý để phục vụ OCR/AI/index, và bản sau chỉnh sửa do người dùng xác nhận. Việc tách lớp này giúp tránh ghi đè dữ liệu gốc, giảm tranh chấp nguồn chứng cứ, và cho phép truy vết toàn bộ lịch sử chỉnh sửa khi xuất hồ sơ.

### 10.2 Cấu trúc thư mục đề xuất trên máy

Mỗi hồ sơ (case) dùng một thư mục riêng:

`workspace/cases/<CASE_ID>/`

Bên trong bắt buộc có:
1. `original/`: lưu file nhập ban đầu, không chỉnh sửa, không rename phá vết.
2. `processing/`: file trung gian đang chạy pipeline (ảnh render trang, OCR json, cache layout, cache preview).
3. `reviewed/`: file đã được người dùng duyệt/sửa metadata hoặc sửa OCR block.
4. `managed/`: file đã chuẩn hóa tên và nhóm theo quy tắc nghiệp vụ, dùng cho tra cứu vận hành.
5. `exports/`: các gói export đã phát hành cho người dùng.
6. `logs/`: nhật ký pipeline + audit thay đổi liên quan file.

### 10.3 Trạng thái file và quy tắc chuyển trạng thái

Mỗi file phải có lifecycle rõ ràng trong DB:

`imported` → `processing` → `ocr_done` → `review_pending` → `reviewed` → `managed_ready` → `exported`

Quy tắc vận hành:
1. Khi import xong, file được copy vào `original/`, gắn hash SHA-256 và nguồn nhập.
2. Khi vào pipeline, dữ liệu trung gian sinh trong `processing/`; không sửa file gốc.
3. Khi người dùng sửa OCR/metadata, lưu revision vào DB và ghi phiên bản vào `reviewed/` nếu có thay đổi nội dung xuất bản.
4. Khi duyệt xong, hệ thống tạo bản chuẩn trong `managed/` theo template đặt tên đã cấu hình.
5. Khi export, hệ thống chỉ lấy dữ liệu từ bản `managed_ready` + trích dẫn đã hợp lệ.

### 10.4 Đề xuất schema quản lý vòng đời file

Bổ sung/chuẩn hóa các trường trong bảng tài liệu:
1. `original_path`: đường dẫn bất biến trong `original/`.
2. `processing_path`: điểm vào dữ liệu trung gian chính.
3. `reviewed_path`: đường dẫn bản đã duyệt/sửa (nếu có).
4. `managed_path`: đường dẫn bản chuẩn để vận hành và export.
5. `file_status`: trạng thái lifecycle hiện tại.
6. `file_hash_sha256`: định danh toàn vẹn.
7. `revision_no`: số phiên bản chỉnh sửa.
8. `last_reviewed_by`, `last_reviewed_at`: truy vết người duyệt.

### 10.5 Chính sách file “đang xử lý”

File đang xử lý trong `processing/` cần TTL dọn dẹp và khóa an toàn:
1. Không xóa cache khi job còn chạy.
2. Nếu job fail, giữ cache để debug trong khoảng thời gian cấu hình (ví dụ 7 ngày).
3. Nếu job thành công và đã `managed_ready`, dọn bớt artefacts tạm theo chính sách tiết kiệm dung lượng.
4. Nếu gặp file lock Windows, áp dụng cơ chế retry/backoff và deferred marker theo hướng hardening đã có trong [`case_cmd.rs`](PhanMem/src-tauri/src/commands/case_cmd.rs:224).

### 10.6 File sau khi sửa và khả năng hoàn nguyên

Khi người dùng sửa OCR block hoặc metadata, hệ thống không ghi đè bản gốc mà lưu phiên bản mới:
1. Revision lưu ở DB (ai sửa gì, sửa lúc nào, thay đổi gì).
2. Có thể lưu snapshot JSON thay đổi để rollback.
3. Bản xuất mặc định dùng revision mới nhất đã duyệt.
4. Người dùng có quyền chọn export theo revision cụ thể nếu cần đối chiếu.

### 10.7 Cấu trúc gói export cho người dùng hồ sơ

Mỗi lần export tạo một thư mục riêng trong `exports/`:

`exports/<EXPORT_JOB_ID>_<TIMESTAMP>/`

Bên trong gồm:
1. `00_index_trich_dan.docx`
2. `01_bao_cao_tong_hop.docx`
3. `02_so_do_vu_an.html`
4. `03_tai_lieu_quan_ly/` (bản managed theo nhóm)
5. `04_phu_luc_citation.json` (toàn bộ citation máy đọc được)
6. `05_audit_export.json` (nguồn dữ liệu, revision, thời điểm export)

Tùy chọn đóng gói:
1. Xuất thư mục thường.
2. Xuất ZIP toàn bộ.
3. Xuất ZIP rút gọn (chỉ báo cáo + citation + danh mục file tham chiếu).

### 10.8 Hỗ trợ người dùng khi nhận hồ sơ export

Trong UI export studio cần có:
1. Danh sách các lần export với trạng thái thành công/thất bại.
2. Nút mở nhanh thư mục export tại máy.
3. Nút sao chép đường dẫn gói export để gửi nội bộ.
4. Cảnh báo nếu còn tài liệu `review_pending` mà người dùng vẫn export.
5. Màn hình đối chiếu “bản gốc ↔ bản managed ↔ bản report” để kiểm tra trước khi bàn giao.

### 10.9 Neo kỹ thuật vào code hiện có

Phần bổ sung này cần tích hợp vào command/export layer hiện tại, không làm nhánh kiến trúc mới. Về điểm chạm code, lifecycle job vẫn đi qua scheduler trong [`pipeline_execution_tick()`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1402), còn đóng gói xuất gắn vào command export hiện có trong [`export_cmd.rs`](PhanMem/src-tauri/src/commands/export_cmd.rs).

---

## 11) Mô hình phân tầng kế thừa, kiểm soát event/database, log toàn trình và tự làm sạch cache-index

Phần này bổ sung trực tiếp theo yêu cầu kiến trúc hệ thống phải phân tầng rõ ràng, có tính kế thừa, kiểm soát sự kiện và dữ liệu toàn cục, đồng thời có cơ chế làm tươi hệ thống theo thời gian sử dụng.

### 11.1 Mô hình phân tầng bắt buộc

Kiến trúc nên chuẩn hóa theo 5 tầng và quan hệ kế thừa giữa các service/agent:
1. **Presentation Layer**: UI pages/components, chỉ hiển thị trạng thái và phát hành intent.
2. **Application Layer**: workflow orchestration, state machine, policy quyết định.
3. **Domain Layer**: nghiệp vụ hồ sơ, phân loại, trích xuất, citation rules, review rules.
4. **Infrastructure Layer**: Rust commands, Python bridge, file system, OCR runtime, local LLM runtime.
5. **Persistence Layer**: SQLite schema, index FTS, migration, audit tables.

Mọi module mới phải tuân thủ nguyên tắc kế thừa:
- `BaseJobHandler` (khung job lifecycle chung) → `ScanJobHandler`, `OcrJobHandler`, `AiJobHandler`, `ExportJobHandler`.
- `BaseAgent` (khung execute/validate/emit_event chung) → `ClassificationAgent`, `EvidenceAgent`, `TimelineAgent`, `CitationQaAgent`.
- `BaseRepository` (CRUD + audit wrapper) → `CaseRepository`, `DocumentRepository`, `EventRepository`, `ExportRepository`.

Mục tiêu của kế thừa là giảm trùng logic retry, status transition, metrics và logging format.

### 11.2 Kiểm soát event toàn cục (Event Governance)

Toàn bộ pipeline phải đi theo event-driven contract chuẩn hóa thay vì log rời rạc. Mỗi event cần có cấu trúc tối thiểu:
1. `event_id`
2. `event_type`
3. `source_layer` và `source_component`
4. `case_id`, `document_id`, `job_id` (nếu có)
5. `phase`, `status_before`, `status_after`
6. `payload_json`
7. `created_at`

Các nhóm event bắt buộc:
- `INGEST_*`
- `OCR_*`
- `EXTRACT_*`
- `AI_*`
- `EXPORT_*`
- `CACHE_*`
- `INDEX_*`
- `DB_MAINTENANCE_*`

Event phải được ghi vào DB trước khi emit lên UI để đảm bảo truy hồi hậu kiểm.

### 11.3 Kiểm soát database và tính nhất quán

Mọi ghi dữ liệu nghiệp vụ đều phải có transaction boundary và audit boundary:
1. Transaction đảm bảo nguyên tử ở mức use-case.
2. Audit ghi “ai làm gì, khi nào, trước-sau ra sao”.
3. Nếu rollback, vẫn ghi event lỗi để điều tra.

Các bảng nên củng cố:
- `pipeline_events` cho vòng đời xử lý.
- `audit_events` cho hành động người dùng/agent.
- `maintenance_jobs` cho các tác vụ làm sạch định kỳ.
- `index_health` để theo dõi độ tươi FTS/index.

### 11.4 Logging toàn bộ quá trình để check về sau

Hệ thống log cần 3 lớp:
1. **Operational log**: tiến trình job theo phase.
2. **Diagnostic log**: lỗi kỹ thuật, timeout, lock file, OCR env.
3. **Compliance log**: thay đổi nghiệp vụ, chỉnh sửa dữ liệu, duyệt/xuất hồ sơ.

Định dạng log thống nhất JSON lines, mỗi log có:
- `trace_id`, `job_id`, `case_id`, `document_id`, `phase`, `level`, `message`, `meta`, `timestamp`.

Chính sách lưu trữ:
- hot log 30 ngày trong DB/file log quay vòng.
- cold log nén theo tháng để tra cứu khi cần kiểm tra lại hồ sơ cũ.

### 11.5 Tự động làm sạch cache và refresh index theo thời gian

Để phần mềm giữ độ tươi mới sau thời gian dài sử dụng và sau khi người dùng xóa hồ sơ, cần có `Maintenance Agent` chạy nền theo lịch:

Nhiệm vụ định kỳ:
1. Quét orphan files trong `processing/`, `previews/`, `ocr cache`.
2. Dọn cache quá hạn TTL.
3. Rebuild hoặc incremental refresh FTS index khi phát hiện lệch trạng thái.
4. `VACUUM`/`ANALYZE` SQLite theo chu kỳ thấp tải.
5. Verify integrity giữa DB record và file vật lý.

Trigger bắt buộc sau hành động xóa hồ sơ:
1. Phát `CASE_DELETED` event.
2. Chạy cleanup theo `case_id` (file + cache + preview + orphan index entries).
3. Chạy index refresh cho các bảng liên quan.
4. Ghi `DB_MAINTENANCE_COMPLETED` event cùng số lượng mục đã dọn.

### 11.6 Tự động tổng hợp Word chi tiết cho toàn bộ file trong `V2/tai lieu`

Yêu cầu “xuất 1 file Word tổng hợp chi tiết toàn bộ tài liệu trong `d:\JOBS\VKS-HoSoDienTu\V2\tai lieu\`” được mô hình hóa thành module `Master Dossier Compilation`.

Luồng thực hiện:
1. Đọc toàn bộ nguồn trong [`V2/tai lieu`](V2/tai lieu).
2. Parse DOCX/HTML/PDF/ảnh thành schema trung gian.
3. Chạy phân loại + trích xuất + timeline + citation mapping.
4. Dựng cấu trúc báo cáo tổng hợp hợp nhất.
5. Render một file Word master (DOCX) chứa đầy đủ mục lục, bảng chỉ mục, nội dung tổng hợp, timeline, sơ đồ tham chiếu, phụ lục citation.
6. Lưu artifact vào thư mục export của case và ghi audit đầy đủ.

Cấu trúc file Word master đề xuất:
1. Trang bìa hồ sơ tổng hợp.
2. Mục lục tự động.
3. Bảng chỉ mục tài liệu toàn tập.
4. Phần thông tin các bên/đối tượng.
5. Phần diễn biến và timeline.
6. Phần phân tích chứng cứ theo nhóm tài liệu.
7. Phần cảnh báo xung đột/cần review.
8. Phụ lục citation chi tiết (document_id, page, quote, bbox nếu có).

### 11.7 Điểm neo thực thi vào codebase hiện hữu

Để không phá kiến trúc đang chạy ổn định, các phần trên cần cấy vào lớp command/pipeline hiện có:
1. State machine và tick orchestration bám theo [`pipeline_execution_tick()`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1402).
2. Event persistence bám `pipeline_events` và mở rộng payload chuẩn.
3. Export master DOCX mở rộng từ command export hiện tại trong [`export_cmd.rs`](PhanMem/src-tauri/src/commands/export_cmd.rs).
4. Cleanup/index refresh chạy như maintenance job nền, có endpoint điều khiển và có log/audit tương ứng.

---

## 12) Chuẩn tổng hợp phân tầng theo page → tài liệu → nhóm → hồ sơ (hierarchical summarization)

Phần này chuẩn hóa lại hướng triển khai bạn nêu, để khóa thành quy tắc kỹ thuật chính thức trong hệ thống.

### 12.1 Tổng hợp từng page

Luồng chuẩn cho từng trang:

`Page image/PDF` → OCR (PaddleOCR) → rule extraction (số văn bản/ngày/tên người/bút lục) → tóm tắt page bằng local LLM → lưu `page_summary` vào DB.

Mỗi page phải lưu tối thiểu:
1. `page_number`
2. `main_content_summary`
3. `related_people_agencies`
4. `document_number_or_date` (nếu trích được)
5. `ocr_low_confidence_regions`
6. `citations` theo page

Điểm neo triển khai:
- OCR/lưu text theo pipeline hiện tại.
- Tạo thêm cột/bảng summary page để phục vụ tổng hợp tầng trên.

### 12.2 Tổng hợp từng tài liệu

Luồng chuẩn cho document:

`page_summaries` → `document_summary` + `document_type` + `suggested_filename` + `extracted_fields`

Nguyên tắc:
1. Không tóm tắt tài liệu trực tiếp từ toàn bộ OCR text thô khi có thể dùng `page_summary` đã chuẩn hóa.
2. Mọi kết luận document-level phải truy hồi ngược được về page citations.
3. Nếu thiếu chứng cứ, đẩy `review_required=true`.

Khuyến nghị model:
- Qwen 3B: đủ cho tóm tắt ngắn, tốc độ tốt trên máy vừa/yếu.
- Qwen 7B: tốt hơn rõ rệt cho phân loại và diễn đạt báo cáo mạch lạc.

### 12.3 Tổng hợp cả hồ sơ (không nạp một lần toàn bộ)

Bắt buộc áp dụng phân tầng:

`page_summary` → `document_summary` → `group_summary` → `case_summary`

Ví dụ thực thi hồ sơ lớn:
1. Tổng hợp từng page của 68 file.
2. Gom thành summary từng tài liệu.
3. Gom theo 7 nhóm hồ sơ.
4. Sinh báo cáo vụ án tổng hợp ở cấp case.

Lợi ích:
- Máy yếu vẫn chạy được vì mỗi lượt model chỉ xử lý khối nhỏ.
- Dễ kiểm tra sai lệch theo tầng.
- Giữ citation chain đầy đủ.

### 12.4 AI nên làm và không nên làm

AI nên làm:
1. Tóm tắt page.
2. Tóm tắt tài liệu.
3. Gợi ý loại tài liệu.
4. Gợi ý tên file.
5. Tạo timeline sơ bộ.
6. Phát hiện mâu thuẫn đơn giản.
7. Viết báo cáo nháp từ dữ liệu đã trích xuất.

AI không nên làm:
1. Thay OCR engine (không thay PaddleOCR bằng LLM).
2. Đọc ảnh scan trực tiếp end-to-end trên máy yếu.
3. Nạp toàn bộ hồ sơ lớn vào một lần suy luận.
4. Tự kết luận pháp lý cuối cùng.
5. Chạy nhiều agent nặng song song vượt profile máy.

### 12.5 Kiến trúc thực thi chuẩn theo hướng này

`PaddleOCR` → `page_text + bbox` → `rule extraction` → `SQLite FTS5` → `page_summary` (Qwen 3B/7B) → `document_summary` → `group_summary` → `case_summary` → `human review`.

Nguyên tắc vận hành cuối:
1. OCR chuẩn là nền tảng.
2. Dữ liệu có cấu trúc là điều kiện bắt buộc.
3. Tóm tắt phân tầng là cách duy nhất để xử lý hồ sơ lớn ổn định trên desktop.
4. Human review là chốt cuối trước export chính thức.

---

## 13) PaddleOCR Updated Strategy — Unicode, Form, Handwriting

Phần này cập nhật thiết kế OCR theo hướng mới hơn cho máy văn phòng 4–16GB RAM, VGA onboard hoặc GPU rời 2–8GB, theo nguyên tắc không dùng một cấu hình nặng cho tất cả.

### 13.1 Tầng 1 — Text OCR Unicode tiếng Việt

Engine chính:
1. PaddleOCR 3.x / PP-OCRv5.
2. Rec model ưu tiên: `latin_PP-OCRv5_mobile_rec`.
3. Detection model: PP-OCRv5 mobile/server tùy profile máy.

Yêu cầu bắt buộc:
1. Output giữ nguyên Unicode tiếng Việt.
2. Không tự strip dấu tiếng Việt ở bản gốc.
3. Lưu UTF-8 cho mọi text OCR.
4. Lưu đầy đủ `bbox`, `polygon`, `confidence`.
5. Normalize Unicode về NFC trước khi ghi DB.
6. Giữ nguyên quote gốc để dùng citation.

Nếu confidence thấp hoặc mất dấu tiếng Việt:
1. `review_required = true`.
2. Không dùng làm căn cứ kết luận pháp lý cuối cùng.

### 13.2 Chuẩn Unicode trong database

Mỗi block OCR cần lưu hai bản text:
1. `raw_text`: bản OCR gốc.
2. `normalized_text`: bản chuẩn hóa để search.

Quy tắc:
1. `raw_text` không uppercase toàn bộ, không bỏ dấu.
2. `normalized_text` có thể chuyển không dấu để tăng recall tìm kiếm.
3. Lưu cờ `unicode_form = NFC` và `language = vi`.

Ví dụ payload lưu:

```json
{
  "raw_text": "Cơ quan Cảnh sát điều tra",
  "normalized_text": "co quan canh sat dieu tra",
  "unicode_form": "NFC",
  "language": "vi",
  "confidence": 0.93
}
```

### 13.3 Tầng 2 — Form and Layout OCR

Dùng PaddleOCR-VL hoặc PP-Structure khi gặp trang cấu trúc phức tạp.

Không chạy mặc định cho toàn bộ page trên máy yếu. Chỉ chạy khi:
1. Page có bảng.
2. Page có form nhiều ô.
3. OCR thường confidence thấp.
4. Reading order rối.
5. Người dùng bấm “Phân tích form”.

Output layout bắt buộc:
1. `block_type`
2. `bbox`
3. `reading_order`
4. `table_cell` nếu là bảng
5. `field_name` nếu detect được trường

Danh mục `block_type` chuẩn:
1. `header`
2. `agency`
3. `national_motto`
4. `document_number`
5. `issue_date`
6. `title`
7. `body`
8. `form_label`
9. `form_value`
10. `table`
11. `table_cell`
12. `signature_area`
13. `stamp_area`
14. `but_luc`
15. `handwriting`
16. `unknown`

### 13.4 Tầng 3 — Handwriting and Bút lục handling

Handwriting không được coi là nguồn chắc chắn mặc định.

Luồng xử lý:
1. Detect vùng nghi handwriting bằng layout + contour + vị trí.
2. Nếu là số bút lục thì thử OCR/HTR nhẹ.
3. Nếu confidence >= 0.85 thì đưa dạng đề xuất.
4. Nếu confidence < 0.85 thì bắt buộc review.
5. Chỉ lưu final sau xác nhận người dùng.

Quy tắc chữ ký:
1. Chỉ detect vùng chữ ký.
2. Không suy luận danh tính người ký bằng AI vision/OCR.
3. Tên người ký chỉ lấy từ text in gần vùng ký khi có citation hợp lệ.

### 13.5 Re-OCR / Rescan modes trong Viewer

Viewer cần các nút thao tác:
1. OCR trang hiện tại.
2. OCR toàn bộ tài liệu.
3. OCR chính xác cao.
4. Phân tích form.
5. Phân tích chữ viết tay/bút lục.
6. Rebuild index.

Mode vận hành:
1. Fast OCR: PP-OCRv5 mobile, 200–300 DPI.
2. Accurate OCR: PP-OCRv5 server, 300–400 DPI.
3. Form OCR: PaddleOCR-VL / PP-Structure.
4. Handwriting Assist: detect + review, không auto-final.

### 13.6 Chuẩn output OCR đề xuất

```json
{
  "document_id": "doc_001",
  "page_number": 1,
  "ocr_engine": "paddleocr",
  "ocr_model": "PP-OCRv5",
  "rec_model": "latin_PP-OCRv5_mobile_rec",
  "language": "vi",
  "unicode_form": "NFC",
  "blocks": [
    {
      "block_id": "b001",
      "raw_text": "Số: 327/QĐ-CQCSĐT(ĐTTH)",
      "normalized_text": "so 327 qd cqcsdt dtth",
      "block_type": "document_number",
      "bbox": { "x": 120, "y": 180, "width": 260, "height": 28 },
      "polygon": [[120,180],[380,180],[380,208],[120,208]],
      "confidence": 0.94,
      "review_required": false
    }
  ]
}
```

### 13.7 Cấu hình thực tế theo profile máy

Máy 4GB RAM:
1. OCR: PaddleOCR mobile CPU.
2. DPI: 200–300.
3. AI: tắt mặc định.
4. Search: SQLite FTS5.

Máy 8GB RAM:
1. OCR: PP-OCRv5 mobile.
2. DPI: 300.
3. AI: Qwen 3B tùy chọn.
4. Form parsing: chỉ chạy khi user bấm.

Máy 16GB RAM:
1. OCR: PP-OCRv5 mobile/server tùy tốc độ.
2. DPI: 300–400.
3. AI: Qwen 7B tùy chọn.
4. PaddleOCR-VL: chỉ chạy theo batch nhỏ hoặc từng page có chọn lọc.

### 13.8 Quy tắc kỹ thuật chốt

1. PaddleOCR đọc chữ.
2. OpenCV cải thiện ảnh.
3. Rule extraction lấy trường.
4. SQLite FTS5 phục vụ tìm kiếm.
5. AI chỉ tổng hợp khi dữ liệu đã có nguồn.

Với máy văn phòng yếu:
1. OCR + rule + search là core.
2. AI là optional.
3. PaddleOCR-VL/form parsing chạy chọn lọc, không chạy toàn bộ hồ sơ mặc định.

### 13.9 Điểm neo triển khai vào hệ thống hiện tại

1. Tầng OCR và Re-OCR tích hợp vào command OCR/document đang có tại [`doc_cmd.rs`](PhanMem/src-tauri/src/commands/doc_cmd.rs).
2. Chọn mode theo profile máy nối với setting process mode hiện có tại [`scan_cmd.rs`](PhanMem/src-tauri/src/commands/scan_cmd.rs).
3. Cập nhật output OCR và metadata block để phục vụ viewer trong [`DocumentViewer.tsx`](PhanMem/src/components/DocumentViewer.tsx:53).
4. Luồng rebuild index và refresh dữ liệu tiếp tục đi qua scheduler nền ở [`pipeline_execution_tick()`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1402).

---

## 14) Adaptive Performance Profile — phân tầng hiệu năng theo 3 thời điểm

Phần này chốt cơ chế phân tầng hiệu năng theo đúng 3 thời điểm bắt buộc: lúc cài máy, lúc app đang chạy, và lúc OCR/AI xử lý theo page/tài liệu.

### 14.1 Install-time profiling (khi cài máy / lần mở đầu tiên)

Ngay sau cài đặt hoặc lần chạy đầu, hệ thống phải thực hiện `System Check` để gán profile mặc định.

Thông tin cần đo:
1. RAM tổng và RAM khả dụng.
2. CPU (số core/threads).
3. GPU có/không và VRAM khả dụng.
4. Dung lượng ổ đĩa còn trống.
5. Trạng thái OCR runtime (PaddleOCR đã sẵn sàng chưa).
6. Trạng thái model OCR đã tải chưa.
7. Trạng thái Ollama/local LLM (Qwen) đã sẵn sàng chưa.

Quy tắc gán profile mặc định:
1. `Low`: khoảng 4GB RAM, không GPU.
2. `Standard`: khoảng 8GB RAM.
3. `High`: khoảng 16GB RAM, GPU 4–8GB trở lên.

Mapping mặc định:
1. Máy 4GB: OCR mobile, AI tắt mặc định.
2. Máy 8GB: OCR mobile, AI 3B tùy chọn.
3. Máy 16GB: OCR nâng cao hơn, AI 7B tùy chọn.

### 14.2 Runtime monitoring (khi app hoạt động)

Trong khi app chạy, phải giám sát liên tục tình trạng tài nguyên để tránh đơ/crash:
1. RAM khả dụng.
2. CPU usage.
3. GPU/VRAM usage (nếu có).
4. Số job pipeline đang chạy.
5. Tín hiệu lag UI (thời gian phản hồi, queue UI event).

Khi tài nguyên xuống thấp, hệ thống tự giảm tải theo thứ tự:
1. Giảm số worker OCR.
2. Giảm batch size.
3. Tạm dừng job AI nền.
4. Chuyển xử lý sang từng page.
5. Ưu tiên giữ mượt Viewer và thao tác người dùng.

Các hành động giảm tải phải hiển thị rõ cho người dùng (không giảm ngầm không thông báo).

### 14.3 Task-level adaptive processing (khi OCR/AI chạy)

Đây là tầng quan trọng nhất vì chất lượng scan khác nhau theo từng page.

Chọn mode theo đặc điểm tài liệu:
1. Trang rõ, scan đẹp → `Fast OCR` (200–300 DPI, PP-OCR mobile).
2. Trang mờ/nghiêng/mất chữ → `Accurate OCR` (300–400 DPI + OpenCV preprocess).
3. Trang có bảng/form → `Form OCR` (PP-Structure hoặc PaddleOCR-VL nếu máy đủ).
4. Trang có bút lục/chữ viết tay → `Handwriting Assist` (detect vùng → OCR/HTR thử → bắt buộc review).

Nguyên tắc thực thi:
1. Không áp một mode cứng cho toàn bộ hồ sơ.
2. Mỗi page có thể có mode riêng.
3. Với máy bận, vẫn cho `Accurate OCR` nhưng chạy nhỏ lẻ, không batch nặng.

### 14.4 Mô hình profile 3 lớp cần dùng đồng thời

Hệ thống phải dùng đồng thời ba lớp quyết định:
1. `Machine Profile`: năng lực phần cứng nền của máy.
2. `Runtime Profile`: trạng thái bận/rảnh tức thời của máy khi app chạy.
3. `Task Profile`: mức xử lý cần thiết cho từng page/tài liệu.

Ví dụ vận hành:
1. Máy 8GB → `Machine Profile = Standard`.
2. Đang mở viewer + import nhiều file → `Runtime Profile = Busy`.
3. Gặp page scan mờ → `Task Profile = Accurate OCR`.
4. Kết quả: chạy accurate theo page, không batch lớn, giữ UI mượt.

### 14.5 Quy tắc an toàn bắt buộc

1. Máy yếu không được crash vì cấu hình mặc định quá nặng.
2. Không đủ tài nguyên thì hạ chế độ thay vì ném lỗi cứng.
3. Mọi lần hạ cấp phải ghi event + log + hiển thị rõ trên UI.
4. Cho phép người dùng `pause/resume` job OCR khi cần ưu tiên thao tác khác.

### 14.6 Điểm neo triển khai

1. Install-time profiling tích hợp vào luồng khởi tạo settings và kiểm tra runtime phụ thuộc.
2. Runtime monitoring tích hợp vào scheduler/tick và store trạng thái toàn cục.
3. Task-level adaptive mode tích hợp vào pipeline OCR theo page trong command layer hiện tại, bám luồng [`pipeline_execution_tick()`](PhanMem/src-tauri/src/commands/scan_cmd.rs:1402).

---

## 15) Startup Environment Gate + Full Offline Installer Bundle (bổ sung bắt buộc)

Phần này bổ sung trực tiếp theo yêu cầu mới để tài liệu đủ chuẩn triển khai offline thực tế. Trọng tâm là: kiểm tra điều kiện tối thiểu ngay khi khởi động và đóng gói bộ cài offline đầy đủ runtime/model/tool, không phụ thuộc cài đặt thủ công trên máy người dùng.

### 15.1 Bắt buộc có Startup Environment Gate

Ngay khi mở app, trước Dashboard, hệ thống phải chạy cổng kiểm tra môi trường khởi động (Startup Environment Gate) với các hạng mục:
1. RAM tối thiểu.
2. CPU tối thiểu.
3. Dung lượng ổ cứng còn trống.
4. Quyền đọc/ghi workspace.
5. SQLite hoạt động.
6. OCR engine có sẵn.
7. OCR model có đủ.
8. PDF renderer có sẵn.
9. Thư mục model tồn tại.
10. File config/manifest hợp lệ.

Nếu không đạt yêu cầu tối thiểu, app phải:
1. Báo `FAIL` ngay.
2. Chặn vào giao diện chính.
3. Hiển thị lý do rõ ràng, định lượng được.
4. Hiển thị hướng dẫn khắc phục cụ thể.

Ví dụ thông báo chuẩn:

`Không thể khởi động VKS ECMS`

`Lý do:`
1. `RAM khả dụng: 2.8GB, yêu cầu tối thiểu 4GB`.
2. `Thiếu PaddleOCR model: latin_PP-OCRv5_mobile_rec`.
3. `Không tìm thấy pdfium.dll`.

`Vui lòng cài lại bản Offline Full Installer.`

### 15.2 Hồ sơ yêu cầu tối thiểu/đề xuất/nâng cao

`Minimum` (chỉ chạy cơ bản):
1. RAM: 4GB.
2. CPU: 2 core.
3. Disk trống: 10GB.
4. GPU: không bắt buộc.
5. OS: Windows 10/11 64-bit.
6. OCR: PaddleOCR mobile CPU.
7. AI: tắt mặc định.

Chức năng cho phép ở `Minimum`:
1. Import file.
2. Xem PDF.
3. OCR từng trang/từng file nhỏ.
4. Search FTS5.
5. Quản lý hồ sơ.

Chức năng không bật mặc định ở `Minimum`:
1. Batch OCR lớn.
2. AI local.
3. Form OCR nặng.
4. Vision model.

`Recommended` (dùng ổn định):
1. RAM: 8GB.
2. CPU: 4 core.
3. Disk trống: 30GB.
4. GPU: optional.
5. OS: Windows 10/11 64-bit.
6. OCR: PaddleOCR mobile.
7. AI: Qwen 3B optional.

`Advanced` (dùng AI tốt hơn):
1. RAM: 16GB.
2. CPU: 6–8 core.
3. Disk trống: 50GB+.
4. GPU: optional 4–8GB VRAM.
5. OCR: PaddleOCR mobile/server.
6. AI: Qwen 7B optional.

### 15.3 Full Offline Installer là bắt buộc

Vì hệ thống vận hành offline, bộ cài phải chuẩn bị từ đầu đến cuối; không được để người dùng đang cài mới phát hiện thiếu thành phần như `Node.js`, `Python`, model, OCR engine, `pdfium`, Visual C++ Runtime.

Bộ cài mục tiêu: `VKS_ECMS_Setup.exe`.

Cấu trúc tối thiểu bên trong bộ cài:

```text
app/
  VKS_ECMS.exe

runtime/
  vc_redist/
  webview2/
  sqlite/
  pdfium/
  opencv/
  python_embedded/

ocr/
  paddleocr/
  models/
    det/
    rec/
    cls/
    latin_PP-OCRv5_mobile_rec/
    latin_PP-OCRv5_mobile_det/

ai/
  ollama_optional/
  models_optional/
    qwen_3b/
    qwen_7b/

tools/
  pdf_tools/
  image_tools/
  hash_tools/

templates/
  index_docx/
  summary_docx/
  case_diagram_html/

config/
  default_profiles.json
  model_manifest.json
  installer_manifest.json

test/
  self_check_files/
```

### 15.4 Không yêu cầu Node.js trên máy người dùng

`Node.js` chỉ dành cho máy dev. Máy người dùng cuối không được yêu cầu bất kỳ công cụ build nào.

Bản phát hành bắt buộc là `Tauri release bundle`, người dùng chỉ chạy `VKS_ECMS_Setup.exe`.

Không được yêu cầu người dùng cài/ chạy:
1. `npm install`.
2. `node`.
3. `cargo`.
4. Python system-wide.
5. `pip install`.
6. Internet.

### 15.5 Python/OCR phải là embedded runtime

Nếu OCR worker dùng Python (PaddleOCR/OpenCV), phải đóng gói embedded:

```text
python_embedded/
  python.exe
  Lib/
  site-packages/
  paddleocr/
  paddlepaddle/
  cv2/
  numpy/
  scripts/
    ocr_worker.py
```

Không dùng Python cài sẵn trên máy người dùng và không yêu cầu:
1. `pip install paddleocr`.
2. `pip install opencv-python`.

### 15.6 Bắt buộc có `model_manifest.json`

Thêm file manifest model: [`model_manifest.json`](model_manifest.json) để mô tả model bắt buộc/tùy chọn, đường dẫn, checksum.

Ví dụ:

```json
{
  "ocr": {
    "required": [
      {
        "name": "latin_PP-OCRv5_mobile_det",
        "path": "ocr/models/det",
        "required": true,
        "sha256": "..."
      },
      {
        "name": "latin_PP-OCRv5_mobile_rec",
        "path": "ocr/models/rec",
        "required": true,
        "sha256": "..."
      }
    ]
  },
  "ai": {
    "optional": [
      {
        "name": "qwen_3b",
        "path": "ai/models_optional/qwen_3b",
        "required": false,
        "sha256": "..."
      }
    ]
  }
}
```

Startup bắt buộc verify:
1. File/thư mục tồn tại.
2. Checksum đúng.
3. Version đúng.

### 15.7 Bắt buộc có `installer_manifest.json`

Thêm file [`installer_manifest.json`](installer_manifest.json) để xác nhận đây là full offline bundle và khai báo thành phần required/optional.

Ví dụ:

```json
{
  "app_version": "1.0.0",
  "offline_bundle": true,
  "required_components": [
    "pdfium",
    "sqlite",
    "paddleocr",
    "opencv",
    "python_embedded",
    "ocr_models"
  ],
  "optional_components": [
    "ollama",
    "qwen_3b",
    "qwen_7b"
  ]
}
```

### 15.8 Startup self-check command trong backend Rust

Backend Tauri cần command [`run_startup_self_check`](PhanMem/src-tauri/src/main.rs:1) (điểm neo tên command; khi triển khai thực tế nên đặt ở module command riêng rồi đăng ký invoke handler).

Payload trả về khi pass:

```json
{
  "status": "pass",
  "profile": "standard",
  "checks": [
    {
      "name": "RAM",
      "status": "pass",
      "required": "4GB",
      "actual": "7.6GB"
    },
    {
      "name": "PaddleOCR model",
      "status": "pass",
      "required": "latin_PP-OCRv5_mobile_rec",
      "actual": "found"
    }
  ],
  "fatal_errors": [],
  "warnings": []
}
```

Payload khi fail:

```json
{
  "status": "fail",
  "fatal_errors": [
    "Missing pdfium.dll",
    "RAM below 4GB",
    "OCR model not found"
  ]
}
```

### 15.9 UI khởi động trước Dashboard

Trước Dashboard phải có màn `System Startup Check`.

Hiển thị tiến trình:
1. `Đang kiểm tra môi trường...`
2. `✓ RAM`
3. `✓ CPU`
4. `✓ SQLite`
5. `✓ PDF renderer`
6. `✓ OCR engine`
7. `✓ OCR model`
8. `✓ Workspace permission`

Nếu fail:
1. `Không thể tiếp tục`.
2. `[Chi tiết lỗi]`.
3. `[Copy log]`.
4. `[Thử kiểm tra lại]`.
5. `[Thoát]`.

### 15.10 Phân loại lỗi Fatal vs Warning

Phân biệt bắt buộc để tránh fail sai mức:

`Fatal` (chặn app):
1. Thiếu OCR engine.
2. Thiếu PDF renderer.
3. Thiếu SQLite.
4. Không có quyền ghi workspace.
5. Thiếu OCR model bắt buộc.

`Warning` (app vẫn chạy):
1. Thiếu AI model.
2. Thiếu GPU (rơi về CPU mode).
3. Thiếu Ollama/local LLM.
4. RAM thấp cho AI nâng cao.

### 15.11 Nội dung cần bổ sung vào `claude.md`

Đoạn yêu cầu chuẩn triển khai runtime offline cần được thêm nguyên văn vào [`claude.md`](claude.md), tiêu đề:
`Offline Runtime Requirements & Full Installer Bundle`.

Nội dung gồm bốn cụm bắt buộc:
1. `Startup Environment Gate`.
2. `Minimum Runtime Profile`.
3. `Full Offline Installer`.
4. `Manifest Files` + `Startup Self Check Command` + quy tắc chặn UI chính khi fail.

### 15.12 Kết luận bắt buộc cho chuẩn offline thật

Để tài liệu đủ chuẩn triển khai offline thực tế, phải khóa cứng đúng hai điểm:
1. Không đủ tối thiểu thì fail ngay khi khởi động, chặn vào app chính.
2. Bộ cài offline phải đóng gói toàn bộ runtime/model/tool.

Nếu thiếu một trong hai, hệ thống rất dễ rơi vào trạng thái build chạy trên máy dev nhưng mang sang máy kiểm sát viên thì không vận hành được.
