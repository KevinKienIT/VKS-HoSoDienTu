# ĐẶC TẢ LUỒNG SCAN OCR AI ĐÁNH GIÁ CHẤT LƯỢNG

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa luồng scan tài liệu, xử lý OCR, AI đánh giá chất lượng và quản lý review queue.
Nguon prompt: Tu yeu cau ve scan OCR AI pipeline.

---

## 1. Tong Quan Scan/OCR/AI Pipeline

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal scan pipeline**
- **KHÔNG có AI evaluation**
- **KHÔNG có quality metrics**

### 1.2 Muc Tieu Cua File Nay

- Scan/import pipeline
- OCR processing pipeline
- AI quality evaluation
- Review queue management

---

## 2. Scan Pipeline

### 2.1 Import Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SCAN/IMPORT PIPELINE                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐               │
│  │ User      │───►│ Validate   │───►│ Extract   │               │
│  │ Select   │    │ Files     │    │ Pages    │               │
│  │ Files    │    │           │    │ (PDF→PNG)│               │
│  └─────────────┘    └─────────────┘    └─────────────┘               │
│        │                  │                  │                        │
│        ▼                  ▼                  ▼                        │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐               │
│  │ Extension │    │ File Size │    │ Image    │               │
│  │ Check    │    │ Check    │    │ Quality │               │
│  │ (.pdf,.jpg│    │ <50MB    │    │ Assessment│               │
│  │ .png)    │    │          │    │          │               │
│  └─────────────┘    └─────────────┘    └─────────────┘               │
│                                                                      │
│        │                  │                  │                        │
│        ▼                  ▼                  ▼                        │
│  ┌─────────────────────────────────────────────────────────┐          │
│  │                    OCR PROCESS                        │          │
│  ├─────────────────────────────────────────────────────────┤          │
│  │                                                  │          │
│  │  ┌─────────┐   ┌─────────┐   ┌─────────┐         │          │
│  │  │Preproc  │──►│ Paddle  │──►│ Post    │         │          │
│  │  │(denoise)│   │  OCR   │   │ process │         │          │
│  │  └─────────┘   └─────────┘   └─────────┘         │          │
│  │                                              │          │
│  │  Output:                                    │          │
│  │  - ocr_text                             │          │
│  │  - confidence                             │          │
│  │  - regions                                │          │
│  │  - has_handwriting                        │          │
│  │  - has_seal                             │          │
│  │                                                  │          │
│  └─────────────────────────────────────────────────────────┘          │
│        │                  │                  │                        │
│        ▼                  ▼                  ▼                        │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐               │
│  │ Quality    │───►│ Store to   │───►│ Create    │               │
│  │ Assessment│    │ Database   │    │ Citation  │               │
│  │            │    │            │    │ Anchors   │               │
│  └─────────────┘    └─────────────┘    └─────────────┘               │
│        │                                                     │
│        ▼                                                     │
│  ┌─────────────┐                                             │
│  │ Review     │───► Review Queue (if needed)                  │
│  │ Queue    │                                              │
│  │ Decision │                                              │
│  └─────────────┘                                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Validation Rules

| Check | Rule | Action |
|-------|------|--------|
| Extension | pdf, jpg, png | Accept/Reject |
| File size | < 50MB | Accept/Warn |
| Image DPI | > 150 DPI | Accept/Warn |
| PDF pages | < 500 pages | Accept/Warn |

---

## 3. OCR Processing Pipeline

### 3.1 OCR Steps

| Step | Description | Output |
|------|-------------|--------|
| 1. Image Preprocessing | Denoise, contrast, deskew | Enhanced image |
| 2. Layout Analysis | Detect regions, columns | Region boxes |
| 3. Text Detection | Bounding boxes | Text boxes |
| 4. Text Recognition | Per line | Raw text |
| 5. Seal Detection | Detect stamps/seals | Seal info |
| 6. Handwriting Detection | Identify handwriting | HW regions |
| 7. Post-processing | Vietnamese normalization | Clean text |

### 3.2 OCR Configuration

| Document Type | OCR Mode | Preprocessing |
|--------------|---------|--------------|
| Van ban may | Standard | Basic |
| To khai | Enhanced | Heavy |
| Bien ban | Standard | Basic |
| Hinh su (dau/ky) | Seal detection | Enhanced |
| Ban ghi tay | Handwriting | Heavy + HW assist |

### 3.3 Confidence Calculation

```
Overall Confidence = (Σ line_confidence) / total_lines
     + bonus_for_seal (+0.05 if seal detected)
     + penalty_for_handwriting (-0.1 if handwritten)
     + bonus_for_low_var (+0.05 if consistent)
```

| Score | Level | Action |
|-------|-------|--------|
| ≥ 0.90 | High | Auto-accept |
| 0.70-0.90 | Medium | Queue for review |
| 0.50-0.70 | Low | Require review |
| < 0.50 | Very Low | Manual required |

---

## 4. AI Quality Evaluation

### 4.1 Evaluation Metrics

| Metric | Description | Weight |
|--------|-------------|--------|
| OCR Accuracy | Correct character rate | 40% |
| Layout Preservation | Structure preserved | 20% |
| Seal Detection | Stamp detection success | 15% |
| Handwriting Detection | HW identification | 15% |
| Vietnamese Quality | Diacritics/encoding | 10% |

### 4.2 AI Evaluation Flow

```
OCR Result + Original Image
         │
         ▼
┌─────────────────────────┐
│ AI Evaluation (Ollama) │
├─────────────────────────┤
│ Prompt:                │
│ "Evaluate OCR quality:  │
│  - Accuracy            │
│  - Issues             │
│  - Confidence"        │
└─────────────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Evaluation Result      │
├─────────────────────────┤
│ Score: 0.85          │
│ Issues: ["unclear "b"] │
│ Recommendation: "review"│
└─────────────────────────┘
         │
         ├───────────────────────┐
         ▼                       ▼
    Accept                 Review Queue
```

### 4.3 AI Evaluation Criteria

| Check | Question | Score Impact |
|-------|----------|-------------|
| Character accuracy | "Is text readable?" | ±20% |
| Number accuracy | "Are numbers correct?" | ±15% |
| Date accuracy | "Are dates correct?" | ±10% |
| Name accuracy | "Are names correct?" | ±10% |
| Layout preservation | "Structure intact?" | ±15% |

---

## 5. Review Queue Management

### 5.1 Queue Entry Criteria

| Condition | Queue Entry | Priority |
|-----------|------------|----------|
| confidence < 0.7 | ✅ | High |
| has_handwriting = true | ✅ | Medium |
| has_seal detected = false | ✅ | Low |
| uncertain_spans exists | ✅ | High |
| OCR failed | ✅ | Critical |

### 5.2 Queue Priority

```
Priority = 
  (confidence < 0.5 ? 100 : 0) +
  (has_handwriting ? 50 : 0) +
  (uncertain_spans > 0 ? 30 : 0) +
  (document_type == to_khai ? 20 : 0)
```

| Priority | Score | SLA |
|----------|-------|-----|
| Critical | ≥ 100 | < 1 hour |
| High | 50-99 | < 4 hours |
| Medium | 20-49 | < 24 hours |
| Low | < 20 | < 7 days |

### 5.3 Queue Processing

```
[Queue Item Available]
         │
         ▼
┌─────────────────────────┐
│ Fetch Next by Priority │
│ (highest priority)    │
└─────────────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Display in Review UI   │
│ (side-by-side)        │
└─────────────────────────┘
         │
         ▼
┌─────────────────────────┐
│ User Action           │
├──────────────────────┤
│ Approve → Update DB │
│ Edit → Update + Review │
│ Reject → Mark failed │
│ Skip → Next item    │
└─────────────────────────┘
```

---

## 6. Handwriting Special Handling

### 6.1 Handwriting Detection

```
Image Input
         │
         ▼
┌─────────────────────────┐
│ Detect Handwriting     │
│ Regions               │
├──────────────────────┤
│ 1. Edge detection   │
│ 2. Shape analysis  │
│ 3. Stroke analysis │
│ 4. AI classification│
└─────────────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Output: HW Regions    │
├──────────────────────┤
│ [{bbox: [x,y,w,h],   │
│  confidence: 0.75,  │
│  type: "handwritten"}]│
└─────────────────────────┘
```

### 6.2 Handwriting Processing

| Step | Action | Confidence |
|------|--------|-------------|
| 1. Region isolation | Crop HW region | - |
| 2. Preprocessing | Enhance | - |
| 3. HW model OCR | PaddleOCR-HW | 0.65-0.80 |
| 4. AI assist | Ollama candidate | 0.70-0.85 |
| 5. Bundle candidates | Top-3 readings | - |
| 6. Manual review | Required | - |

### 6.3 Candidate Generation

```json
{
  "candidates": [
    { "text": "To pham thanh 500", "confidence": 0.72 },
    { "text": "To pham Dinh 500", "confidence": 0.45 },
    { "text": "To pham tang 500", "confidence": 0.35 }
  ],
  "uncertain_spans": [
    { "start": 3, "end": 6, "reason": "unclear_digit" }
  ]
}
```

---

## 7. Quality Metrics Dashboard

### 7.1 Metrics Display

```
┌─────────────────────────────────────────────────┐
│           OCR QUALITY METRICS                   │
├─────────────────────────────────────────────────┤
│                                                 │
│  Overall Score     ████████████░░░░  82%          │
│                                                 │
│  By Type:                                      │
│  ├─Van ban may   ██████████████░░  95%         │
│  ├─To khai      ██████████░░░░░░░░  70%         │
│  ├─Bien ban    ██████████████░░░░  92%         │
│  └─Ban ghi tay ████████░░░░░░░░░░░  55%        │
│                                                 │
│  By Day:                                       │
│  ████████████████████░░░░░░░░░░░░░  2026-04-24│
│  ██████████████████░░░░░░░░░░░░░░░  2026-04-23│
│                                                 │
└─────────────────────────────────────────────────┘
```

### 7.2 Metrics Stored

| Metric | Calculation | Storage |
|--------|-------------|----------|
| avg_confidence | Mean of all | Per document |
| high_conf_rate | % with >0.9 | Daily |
| review_rate | % queued | Daily |
| ocr_success_rate | % no error | Daily |

---

## 8. State Management

### 8.1 OCR Store

```typescript
interface OCRState {
  jobs: OCRJob[];
  currentJob: OCRJob | null;
  queue: ReviewItem[];
  
  // Actions
  startOCRJob: (files: File[]) => Promise<void>;
  processPage: (pageId: string) => Promise<OCRResult>;
  addToQueue: (item: ReviewItem) => Promise<void>;
  removeFromQueue: (itemId: string) => Promise<void>;
}
```

### 8.2 Review Store

```typescript
interface ReviewQueueState {
  items: ReviewItem[];
  currentItem: ReviewItem | null;
  loading: boolean;
  
  // Actions
  fetchNext: () => Promise<ReviewItem>;
  approve: (itemId: string) => Promise<void>;
  reject: (itemId: string, reason: string) => Promise<void>;
}
```

---

## 9. Acceptance Criteria

- [ ] Scan/import pipeline hoan chinh
- [ ] OCR processing day du
- [ ] AI evaluation tich hop
- [ ] Review queue management
- [ ] Handwriting special handling
- [ ] Quality metrics hien thi

---

## 10. Khong Duoc Hieu Sai

- **OCR KHONG phai la recognition engine** - chi pipeline, engine rieng
- **AI evaluation KHONG phai la model** - chi wrapper cho Ollama
- **Review queue KHONG phai la task queue** - chi review-specific

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom D - Pipeline*
*Tiep theo: File 10 - Dac ta tach file ingest*