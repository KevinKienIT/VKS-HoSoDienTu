# KÊ HOẠCH PHÂN RÃ PHASE VÀ GATE TRIỂN KHAI

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghiã phase chia nhỏ cho dự án lớn, gate triển khai, và tiêu chí đánh giá qua từng giai đoạn.
Nguon prompt: Tu yeu cau tong quat ve Full Scope Docs-Only.

---

## 1. Tong Quan Phase Plan

### 1.1 Mau Thieu Đoạn Hien Tai

Du an chua co phase plan chinh thuc. Chi co:
- Kanban-style task tracking (khong co gate)
- Iteration descriptions (khong co scope boundaries)
- Feature lists (khong co phase gates)

### 1.2 Muc Tieu Cua File Nay

Tạo ra:
- Phase chia nho co milestone ro rang
- Gate kiem tra giữa các phase
- Acceptance criteria cho tung gate
- Dependency map giữa các module

---

## 2. Phase Tong The (5 Phase)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     VKS ECMS IMPLEMENTATION PhASES                        │
├─────────────────┬─────────────────┬─────────────────┬─────────────────┤
│    PHASE 1       │    PHASE 2      │    PHASE 3      │    PHASE 4      │
│   FOUNDATION     │   CORE BUILD    │   ENHANCEMENT   │   PRODUCTION     │
├─────────────────┼─────────────────┼─────────────────┴─────────────────┤
│                              PHASE 5                                    │
│                         OPTIMIZATION & SCALING                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Phase 1: Foundation (Tuần 1-2)

| Thành phần | Mo ta | Trạng thái |
|-----------|------|-----------|
| Project Setup | Tauri + React + SQLite scaffold | CAN_CUA_SANG_PHASE_2 |
| Layered Architecture | 4-layer setup | CAN_CUA_SANG_PHASE_2 |
| Base Components | Button, Input, Card, Modal | CAN_CUA_SANG_PHASE_2 |
| Data Layer | SQLite schema setup | CAN_CUA_SANG_PHASE_2 |

### Phase 2: Core Build (Tuần 3-6)

| Thành phần | Mo ta | Trạng thái |
|-----------|------|-----------|
| Case Management | CRUD case | CAN_CUA_SANG_PHASE_3 |
| Document Import | PDF scan import | CAN_CUA_SANG_PHASE_3 |
| OCR Pipeline | PaddleOCR integration | CAN_CUA_SANG_PHASE_3 |
| Search & Filter | Basic search | CAN_CUA_SANG_PHASE_3 |

### Phase 3: Enhancement (Tuần 7-10)

| Thành phần | Mo ta | Trạng thái |
|-----------|------|-----------|
| AI Assistant | Ollama integration | CAN_CUA_SANG_PHASE_4 |
| Citation System | Citation anchor + review | CAN_CUA_SANG_PHASE_4 |
| Audit Trail | Before/after tracking | CAN_CUA_SANG_PHASE_4 |
| User Management | Account + permission | CAN_CUA_SANG_PHASE_4 |

### Phase 4: Production (Tuần 11-14)

| Thành phần | Mo ta | Trạng thái |
|-----------|------|-----------|
| Dossier Panel | Suspect profile | CAN_CUA_SANG_PHASE_5 |
| Timeline View | Su kien timeline | CAN_CUA_SANG_PHASE_5 |
| Report Export | Export bao cao | CAN_CUA_SANG_PHASE_5 |
| Workspace Features | Bookmark, note, contradiction | CAN_CUA_SANG_PHASE_5 |

### Phase 5: Optimization & Scaling (Tuần 15+)

| Thành phần | Mo ta | Trạng thái |
|-----------|------|-----------|
| Performance | Cache, batch processing | DA_HOAN_THANH |
| Mobile Adapt | Responsive design | DA_HOAN_THANH |
| Backup/Restore | Import job resume | DA_HOAN_THANH |

---

## 3. Gate Reviews Va Handoff Criteria

### 3.1 Gate Types

| Gate | Vi tri | Kiem tra | Owner |
|------|-------|---------|-------|
| **Docs Gate** | Giữa spec va code | Canonical docs hoan chinh | **Documentation Lead** |
| **Code Gate** | Giữa code va test | Code review | **Senior Developer** |
| **Test Gate** | Giữa test va handoff | Acceptance test | **QA Lead** |
| **Handoff Gate** | Giữa phase | Phase completion | **Project Manager** |

### 3.2 Gate Criteria (Phase 1)

| Gate | Criteria | Checkpoint |
|------|----------|------------|
| **Docs Gate P1** | ✅ Toan bo 13 spec files hoan thanh | V2/05_tai_lieu_mo_ta/ |
| **Code Gate P1** | ✅ Project scaffold duoc build | `npm run build` |
| **Test Gate P1** | ✅ Unit tests pass | 80%+ coverage |
| **Handoff Gate P1** | ✅ Phase 1 checklist complete | Issue #P1_CLOSED |

### 3.3 Gate Criteria (Phase 2)

| Gate | Criteria | Checkpoint |
|------|----------|------------|
| **Docs Gate P2** | ✅ Data/UI/API specs hoan thanh | Full specification |
| **Code Gate P2** | ✅ CRUD + Import working | Manual test |
| **Test Gate P2** | ✅ Integration tests | 15 test cases |
| **Handoff Gate P2** | ✅ Phase 2 checklist | Issue #P2_CLOSED |

### 3.4 Gate Criteria (Phase 3)

| Gate | Criteria | Checkpoint |
|------|----------|------------|
| **Docs Gate P3** | ✅ AI/Citation specs hoan thanh | Full specification |
| **Code Gate P3** | ✅ AI + Citation working | Manual test |
| **Test Gate P3** | ✅ E2E tests | 30 test cases |
| **Handoff Gate P3** | ✅ Phase 3 checklist | Issue #P3_CLOSED |

### 3.5 Gate Criteria (Phase 4)

| Gate | Criteria | Checkpoint |
|------|----------|------------|
| **Docs Gate P4** | ✅ All features specs | Full specification |
| **Code Gate P4** | ✅ All features working | Manual test |
| **Test Gate P4** | ✅ All E2E tests | 50+ test cases |
| **Handoff Gate P4** | ✅ Phase 4 checklist | Issue #P4_CLOSED |

---

## 4. Phase Gate Review Process

### 4.1 Review Flow (per Phase)

```
PHASE START
    │
    ▼
DOCS GATE ──✗──► Fix specs ──► DOCS GATE ✓
    │                       │
    ▼                       │
CODE GATE ──✗──► Fix code ──► CODE GATE ✓
    │                       │
    ▼                       │
TEST GATE ──✗──► Fix test ──► TEST GATE ✓
    │                       │
    ▼                       │
HANDOFF GATE ──✗──► Fix ─────► HANDLE GATE ✓
    │                       │
    ▼                       │
NEXT PHASE ◄────────────────┘
```

### 4.2 Blocker Criteria

| Blocker Level | Mo ta | Action |
|---------------|------|--------|
| **BLOCKER** | System khong khoi dong | Fix ngay lap tuc |
| **CRITICAL** | Core feature khong work | Fix trong 24h |
| **MAJOR** | Feature gap | Fix trong 1 week |
| **MINOR** | UI/UX issue | Fix trong next phase |

### 4.3 Go/No-Go Decision

| Criteria | Go | No-Go |
|----------|-----|------|
| All Docs Gates | ✅ Pass | ❌ Failed |
| All Test Gates | ✅ Pass | ❌ Failed |
| Blocker Count | 0 | > 0 |
| Acceptance Rate | > 80% | < 80% |

---

## 5. Module Dependency Map

### 5.1 Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                    MODULE DEPENDENCIES                         │
├──────────────────────────────────────────────────────────────┤
│                                                             │
│  [Data Layer] ──────┬──────► [Case Service]                  │
│        │           │              │                          │
│        │           │              ├──────► [Document Service] │
│        │           │              │                          │
│        │           │              ├──────► [Search Service] │
│        │           │              │                          │
│        ▼           ▼              ▼                          │
│  [OCR Service] ──► [AI Service] ──► [Citation Service]      │
│                                                             │
├──────────────────────────────────────────────────────────────┤
│                    UI COMPONENTS                             │
├──────────────────────────────────────────────────────────────┤
│                                                             │
│  [App Layout] ◄──────┬─────── [Sidebar]                    │
│         │           │              │                          │
│         │           │              ├─────── [Case List]       │
│         │           │              │                          │
│         │           │              ├─────── [Document List]  │
│         │           │              │                          │
│         ▼           ▼              ▼                          │
│  [Viewer Panel] ◄───┼─────── [Dossier Panel]              │
│         │           │              │                          │
│         │           │              ├─────── [Timeline View]  │
│         │           │              │                          │
│         ▼           ▼              ▼                          │
│  [Review Queue] ◄──┴─────── [AI Assistant]               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Critical Path

| Order | Module | Phase | Blocking |
|-------|--------|-------|----------|
| 1 | Data Layer | 1 | ⭐ CRITICAL |
| 2 | Project Setup | 1 | ⭐ CRITICAL |
| 3 | Case Management | 2 | ⭐ CRITICAL |
| 4 | Document Import | 2 | HIGH |
| 5 | OCR Pipeline | 2 | HIGH |
| 6 | Search & Filter | 2 | MEDIUM |
| 7 | AI Assistant | 3 | MEDIUM |
| 8 | Citation System | 3 | MEDIUM |
| 9 | User Management | 3 | LOW |
| 10 | Dossier Panel | 4 | LOW |

---

## 6. Implementation Checklist Template

### Phase 1 Checklist

- [ ] Project scaffold (Tauri + React)
- [ ] SQLite database setup
- [ ] 4-Layer Architecture implemented
- [ ] Base components library
- [ ] Zustand store setup
- [ ] Routing setup
- [ ] Theme/Preferences system
- [ ] Development build working
- [ ] Production build working
- [ ] Unit tests (50+)
- [ ] Integration tests (20+)
- [ ] Documentation complete

### Phase 2 Checklist

- [ ] Case CRUD operations
- [ ] Document import (PDF)
- [ ] Page extraction
- [ ] OCR integration (PaddleOCR)
- [ ] Basic search
- [ ] Document viewer
- [ ] Sidebar navigation
- [ ] Filter/Sort functionality
- [ ] Manual tests complete
- [ ] E2E tests (50+)

### Phase 3 Checklist

- [ ] AI Assistant (Ollama)
- [ ] Citation anchor system
- [ ] OCR revision workflow
- [ ] Review queue
- [ ] Audit trail
- [ ] User accounts
- [ ] Permission system
- [ ] Export feature

### Phase 4 Checklist

- [ ] Dossier panel
- [ ] Suspect profile
- [ ] Timeline view
- [ ] Bookmark/Note system
- [ ] Contradiction tracking
- [ ] Report generation
- [ ] Workspace features

---

## 7. Bang Trang Thai Phase

| Phase | Ten | Bat Dau | Ket Thuc | Trang Thai |
|-------|-----|---------|----------|------------|
| 1 | Foundation | TBD | TBD | PENDING |
| 2 | Core Build | TBD | TBD | PENDING |
| 3 | Enhancement | TBD | TBD | PENDING |
| 4 | Production | TBD | TBD | PENDING |
| 5 | Optimization | TBD | TBD | PENDING |

---

## 8. Dependency Va Risk

### 8.1 Technical Dependencies

| Module | Phu thuoc vao | Risk Level |
|--------|---------------|------------|
| Case Management | Data Layer | HIGH |
| Document Import | File System | HIGH |
| OCR Pipeline | Python | HIGH |
| AI Assistant | Ollama | MEDIUM |
| Citation System | Case + Document | MEDIUM |

### 8.2 Resource Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| OCR quality | HIGH | Review queue + manual override |
| AI offline | MEDIUM | Fallback to keyword search |
| Performance | MEDIUM | Batch processing + cache |
| Data migration | LOW | Backup + restore |

### 8.3 Schedule Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scope creep | HIGH | Gate review process |
| Technical debt | MEDIUM | Refactor in Phase 5 |
| Testing gaps | MEDIUM | Minimum 80% coverage |

---

## 9. Acceptance Criteria

- [ ] 5 phase ro rang voi milestone
- [ ] Gate criteria cho tung phase
- [ ] Dependency map giua module
- [ ] Blocker tracking system
- [ ] Go/No-Go decision criteria
- [ ] Checklist template cho team

---

## 10. Khong Duoc Hieu Sai

- **Phase plan KHONG phai là timeline** - chi la cấu trúc mốc, ngày cụ thể cần update khi có resource
- **Gate KHONG phai là test** - gate là điểm review chính thức, test là verification method
- **Phase 1 KHÔNG bao gồm feature development** - chi setup và foundation
- **Dependency map KHÔNG thay thế architecture** - map chiển thị flow, không thay thế design

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuộc Nhom A - Phase & Gate*
*Tiep theo: File 2 - Ban do thuoc tinh metadata*