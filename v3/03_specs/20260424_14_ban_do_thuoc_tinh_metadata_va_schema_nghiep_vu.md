# BẢN ĐỒ THUỘC TÍNH, METADATA VÀ SCHEMA NGHIỆP VỤ

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa attribute map, metadata schema, và các nghiệp vụ liên quan đến dữ liệu hồ sơ.
Nguon prompt: Tu yeu cau tong quat ve metadata va attribute mapping.

---

## 1. Tong Quan Attribute Map

### 1.1 Mau Thieu Đoạn Hien Tai

Hệ thống hiện tại có:
- Basic entity definitions (Case, Document, Page)
- Simple metadata fields (name, date, type)
- **KHÔNG có comprehensive attribute map**
- **KHÔNG có cross-reference tracking**

### 1.2 Muc Tieu Cua File Nay

- Tạo attribute map đầy đủ cho tất cả entity
- Định nghĩa metadata schema mở rộng
- Xác định relationships giữa các entity
- Chuẩn bị cho citation và cross-reference system

---

## 2. Entity Attribute Definitions

### 2.1 Case Entity

| # | Attribute | Kieu | Bat buoc | Mo ta | Vi du |
|---|----------|------|---------|------|-------|
| 1 | case_id | UUID | ✅ | Unique identifier | `550e8400-e29b-41d4-a716-446655440000` |
| 2 | case_code | String | ✅ | Ma case chinh thuc | `VK-2026-00123` |
| 3 | case_display_name | String | ✅ | Ten hien thi | "Nguyen Van A - Vu an 123" |
| 4 | source_folder_name | String | ✅ | Ten folder nguon | `VK_00123_NguyenVanA` |
| 5 | case_sequence_no | Number | | STT tu dong | 123 |
| 6 | primary_person_name | String | ✅ | Ten bi can chinh | "Nguyen Van A" |
| 7 | primary_person_id | UUID | | Link to Entity | Entity ID |
| 8 | case_group_label | String | | Nhom case (VD: hinh su, dan su) | "hinh_su" |
| 9 | case_type | String | ✅ | Loai case | `to_dieu_tra`, `truy_to`, `ks_trava` |
| 10 | investigation_level | String | | Cap do dk | `cap_coi`, `cap_tinh`, `cap_trung_uong` |
| 11 | status | Enum | ✅ | Trang thai | `active`, `archived`, `closed` |
| 12 | severity | String | | Muc do nghiem trong | `binh_thuong`, `nghiem_trong`, `rat_nghiem` |
| 13 | prosecutor_office | String | | Viện KS nơi | "VKS Tp.HCM" |
| 14 | investigator_name | String | | KSV phu trach | "Le Thi B" |
| 15 | document_count | Number | | So luong tai lieu | 15 |
| 16 | total_pages | Number | | Tong so trang | 234 |
| 17 | created_at | DateTime | ✅ | Ngay tao | `2026-04-24T10:00:00Z` |
| 18 | updated_at | DateTime | ✅ | Ngay cap nhat | `2026-04-24T14:30:00Z` |

### 2.2 Document Entity

| # | Attribute | Kieu | Bat buoc | Mo ta | Vi du |
|---|----------|------|---------|------|-------|
| 1 | document_id | UUID | ✅ | Unique identifier | - |
| 2 | case_id | UUID | ✅ | Link to Case | - |
| 3 | original_filename | String | ✅ | Ten file goc | `001_to_khai.pdf` |
| 4 | import_sequence | Number | | Thu tu import | 1 |
| 5 | file_path | String | ✅ | Duong dan luu tru | `/storage/originals/...` |
| 6 | file_hash | String | ✅ | SHA-256 hash | `abc123...` |
| 7 | file_size | Number | | Kich thuoc (bytes) | 1048576 |
| 8 | page_count | Number | | So trang | 3 |
| 9 | display_name | String | ✅ | Ten hien thi | "To khai ban 1" |
| 10 | document_title | String | | Tieu de day du | "To khai Cau truc bien" |
| 11 | document_type | String | ✅ | Loai tai lieu | `to_khai`, `bien_ban`, `quyet_dinh` |
| 12 | document_subtype | String | | Phan loai chi tiet | `to_khai_ban_1`, `qd_khoi_to` |
| 13 | issued_date | DateTime | | Ngay ban hanh | `2026-01-15` |
| 14 | issued_by | String | | Co quan/can bo ban | "CQDT Tp.HCM" |
| 15 | received_date | DateTime | | Ngay nhan ho so | `2026-04-20` |
| 16 | summary_short | String | | Tom tat ngan | "Khai bay to pham..." |
| 17 | summary_detail | String | | Tom tat chi tiet | - |
| 18 | ocr_confidence_avg | Number | | DTB confidence | 0.85 |
| 19 | has_handwriting | Boolean | | Co chu viet tay | true |
| 20 | has_seal | Boolean | | Co dau/chu ky | true |
| 21 | classification_confidence | Number | | DTB phan loai | 0.92 |
| 22 | needs_review | Boolean | | Can review | false |
| 23 | status | Enum | ✅ | Trang thai | `pending`, `processed`, `reviewed`, `error` |
| 24 | created_at | DateTime | ✅ | Ngay tao | - |
| 25 | updated_at | DateTime | ✅ | Ngay cap nhat | - |

### 2.3 Page Entity

| # | Attribute | Kieu | Bat buoc | Mo ta | Vi du |
|---|----------|------|---------|------|-------|
| 1 | page_id | UUID | ✅ | Unique identifier | - |
| 2 | document_id | UUID | ✅ | Link to Document | - |
| 3 | page_index | Number | ✅ | Chi so trang (1-based) | 1 |
| 4 | image_path | String | ✅ | Duong dan anh | `/storage/pages/...` |
| 5 | image_hash | String | | SHA-256 hash | - |
| 6 | ocr_text | String | | Van ban OCR | "Noi dung trang..." |
| 7 | ocr_preview | String | | Preview (500 ky tu) | "Noi dung trang..." |
| 8 | ocr_revision_id | UUID | | Reference to OCR Revision | - |
| 9 | but_luc | String | | But luc trich xuat | "Nguyen Van A khai..." |
| 10 | confidence | Number | | DTB ocr (0-1) | 0.88 |
| 11 | has_handwriting | Boolean | | Co chu viet tay | false |
| 12 | handwriting_regions | JSON | | Vi tri viet tay | `[{"bbox": [...]}]` |
| 13 | transcription_state | Enum | ✅ | Trang thai trich xuat | `direct`, `candidate_only`, `interpolated_pending_review`, `approved_manual` |
| 14 | uncertain_spans | JSON | | Span chua xac dinh | `[{"start": 10, "end": 20}]` |
| 15 | candidate_texts | JSON | | Cac cach doc khac | `["text1", "text2"]` |
| 16 | regions | JSON | | Region phat hien | `[{"type": "table", "bbox": [...]}]` |
| 17 | created_at | DateTime | ✅ | Ngay tao | - |

### 2.4 Entity (Person, Device, Evidence, Event)

| # | Attribute | Kieu | Bat buoc | Mo ta | Vi du |
|---|----------|------|---------|------|-------|
| 1 | entity_id | UUID | ✅ | Unique identifier | - |
| 2 | entity_type | Enum | ✅ | Loai | `person`, `device`, `evidence`, `event` |
| 3 | name | String | ✅ | Ten | "Nguyen Van A" |
| 4 | alias | String | | Ten khac | "A Nam" |
| 5 | description | String | | Mo ta | "Bi can chinh..." |
| 6 | entity_role | String | | Vai tro trong case | `bi_can`, `nhan_chung`, `bi_hai` |
| 7 | identification | JSON | | Thong tin ID | `{"cmnd": "012345678"}` |
| 8 | metadata | JSON | | Them thong tin | `{"age": 35, "job": "..."}` |
| 9 | linked_documents | Array | | Link to Documents | `[doc_id1, doc_id2]` |
| 10 | linked_pages | Array | | Link to Pages | `[page_id1, page_id2]` |
| 11 | linked_entities | Array | | Link to Entities | `[entity_id1]` |
| 12 | created_at | DateTime | ✅ | Ngay tao | - |
| 13 | updated_at | DateTime | | Ngay cap nhat | - |

### 2.5 Citation Entity

| # | Attribute | Kieu | Bat buoc | Mo ta | Vi du |
|---|----------|------|---------|------|-------|
| 1 | citation_id | UUID | ✅ | Unique identifier | - |
| 2 | source_document_id | UUID | ✅ | Link to Document | - |
| 3 | source_page_id | UUID | ✅ | Link to Page | - |
| 4 | source_but_luc | String | | But luc trich dan | "Khai to pham..." |
| 5 | ocr_revision_id | UUID | | Link to OCR Revision | - |
| 6 | citation_anchor_id | UUID | | Anchor ID | - |
| 7 | quote_excerpt | String | ✅ | Trich dan | "To pham theo..." |
| 8 | quote_normalized | String | | Trich dan chuan hoa | "To pham theo..." |
| 9 | context_before | String | | Truoc trich dan | "Tai trang 1..." |
| 10 | context_after | String | | Sau trich dan | "...vao ngay..." |
| 11 | confidence | Number | ✅ | DTB (0-1) | 0.95 |
| 12 | transcription_state | Enum | ✅ | Trang thai | `direct`, `candidate_only`, `interpolated_pending_review`, `approved_manual` |
| 13 | entity_links | Array | | Link to Entities | `[entity_id1]` |
| 14 | citation_type | String | | Loai trich dan | `admission`, `evidence`, `question` |
| 15 | page_reference | String | | Tham chieu trang | "Tr. 1, 2" |
| 16 | created_at | DateTime | ✅ | Ngay tao | - |

### 2.6 User Entity

| # | Attribute | Kieu | Bat buoc | Mo ta | Vi du |
|---|----------|------|---------|------|-------|
| 1 | user_id | UUID | ✅ | Unique identifier | - |
| 2 | username | String | ✅ | Ten dang nhap | "ksv001" |
| 3 | display_name | String | ✅ | Ten hien thi | "Nguyen Van A" |
| 4 | email | String | | Email | "user@vks.gov.vn" |
| 5 | role | Enum | ✅ | Vai tro | `admin`, `ksv`, `viewer` |
| 6 | permission_level | Number | | Cap do quyen | 1-5 |
| 7 | permissions | Array | | Danh sach quyen | `["read", "write", "export"]` |
| 8 | office | String | | Don vi | "VKS Tp.HCM" |
| 9 | status | Enum | ✅ | Trang thai | `active`, `inactive`, `locked` |
| 10 | preferences | JSON | | Cau hinh nguoi dung | `{theme: "light"}` |
| 11 | last_login | DateTime | | Lan dang nhap cuoi | - |
| 12 | created_at | DateTime | ✅ | Ngay tao | - |
| 13 | updated_at | DateTime | | Ngay cap nhat | - |

---

## 3. Metadata Schema (JSON-LD Style)

### 3.1 Case Metadata

```json
{
  "@context": {
    "vks": "http://vks-schema.org/",
    "dc": "http://purl.org/dc/terms/"
  },
  "@type": "vks:Case",
  "vks:caseId": "uuid",
  "vks:caseCode": "string",
  "vks:primaryPerson": {
    "@type": "vks:Person",
    "vks:name": "string",
    "vks:identification": "object"
  },
  "dc:created": "dateTime",
  "dc:modified": "dateTime",
  "vks:status": "active",
  "vks:documents": {
    "@type": "@id",
    "@container": "@set"
  }
}
```

### 3.2 Document Metadata

```json
{
  "@context": {
    "vks": "http://vks-schema.org/",
    "dc": "http://purl.org/dc/terms/"
  },
  "@type": "vks:Document",
  "vks:documentId": "uuid",
  "vks:case": "@id",
  "vks:documentType": "to_khai",
  "vks:issuedDate": "date",
  "vks:issuedBy": "string",
  "dc:title": "string",
  "dc:description": "string",
  "vks:ocrConfidence": "number",
  "vks:needsReview": "boolean",
  "vks:classification": {
    "vks:documentType": "string",
    "vks:confidence": "number"
  }
}
```

### 3.3 Page Metadata

```json
{
  "@context": {
    "vks": "http://vks-schema.org/"
  },
  "@type": "vks:Page",
  "vks:pageId": "uuid",
  "vks:document": "@id",
  "vks:pageIndex": "number",
  "vks:ocrText": "string",
  "vks:transcriptionState": "direct",
  "vks:confidence": "number",
  "vks:hasHandwriting": "boolean",
  "vks:uncertainSpans": [{
    "vks:start": "number",
    "vks:end": "number",
    "vks:reason": "string"
  }]
}
```

---

## 4. Cross-Reference Relationships

### 4.1 Relationship Types

| From | Relationship | To | Cardinality | Examples |
|------|---------------|-----|------------|-----------|
| Case | hasDocument | Document | 1:N | Case → Documents |
| Case | hasPerson | Entity | 1:N | Person entities |
| Document | hasPage | Page | 1:N | Document → Pages |
| Document | references | Entity | N:N | Entity references |
| Page | contains | Entity | N:N | Entity mentions |
| Citation | quotesFrom | Page | N:1 | Citation → Page |
| Citation | supports | Entity | N:N | Citation → Entity |
| Entity | relatedTo | Entity | N:N | Entity → Entity |

### 4.2 Relationship Diagram

```
┌��─��────────────────────────────────────────────────────────────────┐
│                    RELATIONSHIP GRAPH                          │
├───────────────────────────────────────────────────────────────────┤
│                                                             │
│    ┌─────────┐      hasDocument      ┌─────────────────┐    │
│    │  Case   │────────────────────────►│   Document      │    │
│    └─────────┘                       └────────┬────────┘    │
│         │                                     │             │
│         │ hasPerson                     hasPage           │
│         ▼                                     ▼             │
│    ┌─────────┐                       ┌─────────────────┐    │
│    │ Entity  │◄──────────────┬────────│      Page       │    │
│    │(Person) │               │       └─────────────────┘    │
│    └─────────┘               │             │             │
│         │              contains             │             │
│         ▼                     ┌───────────▼──────────┐   │
│    ┌─────────┐                │    Citation           │   │
│    │ Entity  │◄───supports────│    (quoted text)      │   │
│    │(Device) │                └───────────────────────┘   │
│    └─────────┘                             │             │
│                                             │ quotesFrom  │
│                                             ▼             │
│                                       ┌──────────┐      │
│                                       │  Entity  │      │
│                                       │(Person)  │      │
│                                       └──────────┘      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 5. Index Definitions

### 5.1 Primary Indexes

| Table | Index | Columns | Unique |
|-------|-------|---------|--------|
| cases | cases_pkey | case_id | ✅ |
| documents | documents_pkey | document_id | ✅ |
| pages | pages_pkey | page_id | ✅ |
| entities | entities_pkey | entity_id | ✅ |
| citations | citations_pkey | citation_id | ✅ |
| users | users_pkey | user_id | ✅ |

### 5.2 Secondary Indexes

| Table | Index | Columns | Purpose |
|-------|-------|---------|---------|
| cases | idx_case_code | case_code | Search by code |
| cases | idx_case_status | status | Filter by status |
| cases | idx_primary_person | primary_person_name | Search by name |
| documents | idx_doc_case | case_id | Link to case |
| documents | idx_doc_type | document_type | Filter by type |
| documents | idx_doc_date | issued_date | Filter by date |
| pages | idx_page_doc | document_id | Link to doc |
| pages | idx_transcription | transcription_state | Review filter |
| entities | idx_entity_type | entity_type | Filter by type |
| entities | idx_entity_name | name | Search by name |
| citations | idx_cite_doc | source_document_id | Link to doc |
| citations | idx_cite_entity | entity_links | Entity lookup |

### 5.3 Full-Text Index

| Table | Column | Index Type |
|-------|--------|------------|
| documents | summary_short | FTS |
| documents | summary_detail | FTS |
| pages | ocr_text | FTS |
| entities | description | FTS |
| citations | quote_excerpt | FTS |

---

## 6. Validation Rules

### 6.1 Required Field Validation

| Entity | Condition | Validation |
|--------|-----------|------------|
| Case | ALWAYS | case_code NOT NULL |
| Document | ALWAYS | document_type IN (valid types) |
| Page | ALWAYS | page_index >= 1 |
| Citation | ALWAYS | confidence BETWEEN 0 AND 1 |
| User | ALWAYS | role IN (admin, ksv, viewer) |

### 6.2 Business Logic Validation

| Entity | Rule | Error Code |
|--------|------|-----------|
| Document | page_count = images extracted | DOC_PAGE_MISMATCH |
| Page | ocr_revision_id IF needs_review | PAGE_REVISION_MISSING |
| Citation | source_document_id exists | CITE_DOC_NOT_FOUND |
| Entity | linked_documents exist | ENTITY_DOC_NOT_FOUND |

### 6.3 State Transition Rules

| Entity | From State | To State | Allowed |
|--------|-----------|----------|---------|
| Document | pending | processed | ✅ |
| Document | processed | reviewed | ✅ |
| Document | pending | error | ✅ (on failure) |
| Page | direct | candidate_only | ✅ (on low confidence) |
| Page | candidate_only | approved_manual | ✅ (after review) |
| Citation | candidate_only | approved_manual | ✅ (after review) |
| User | active | locked | ✅ (admin only) |

---

## 7. Data Quality Metrics

### 7.1 Quality Dimensions

| Dimension | Metric | Target |
|------------|--------|--------|
| Completeness | Required fields filled | > 95% |
| Accuracy | OCR confidence avg | > 0.85 |
| Consistency | Cross-reference valid | 100% |
| Timeliness | Updated within 24h | > 90% |
| Uniqueness | No duplicate entities | 100% |

### 7.2 Quality Checks

| Check | Frequency | Action on Fail |
|-------|-----------|-----------------|
| Null check | Real-time | Reject save |
| Duplicate check | On import | Warn user |
| Consistency check | Daily | Log issue |
| Confidence check | On OCR | Queue review |

---

## 8. Acceptance Criteria

- [ ] Tat ca entity co day du attribute definitions
- [ ] Metadata schema theo JSON-LD style
- [ ] Relationship diagram ro rang
- [ ] Indexes duoc dinh nghia cho tat ca table
- [ ] Validation rules cho required fields
- [ ] State transition rules cho business logic
- [ ] Data quality metrics duoc dinh nghia

---

## 9. Khong Duoc Hieu Sai

- **Attribute map KHONG phai la database schema** - chi la logical definition, schema cu the o Layer 1
- **Metadata schema KHONG phai la JSON file** - chi la definition, implementation o config
- **Relationship map KHONG phai la foreign key** - relationship o application level
- **Index definitions KHONG phai yeu cau index tao ngay** - chi la design guidance

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom B - Data & Metadata*
*Tiep theo: File 3 - Ban do giao dien*