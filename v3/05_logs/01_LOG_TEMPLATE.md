# 01 Log Template (Unified by Date + Phase)

Tên file log thực tế:
- `YYYYMMDD_phase-<phase>.md`

Ví dụ:
- `20260429_phase-mvp-p0.md`

---

## 1) User Commands

- Time:
- User command:
- Scope:
- Expected output:

## 2) Orchestrator Dispatch

- Time:
- Phase:
- Assigned agents:
- Dispatch instruction:

## 3) Agent Outputs

### Agent A
- Status:
- Summary:
- Files touched:
- Evidence:

### Agent B
- Status:
- Summary:
- Review findings:

### Agent C
- Status:
- Summary:
- Test results:

### Agent D (if used)
- Status:
- Summary:
- Critical decision:

## 4) 3-Step Gate (for plan/phase deletion)

- Code: PASS/FAIL
- Check test: PASS/FAIL
- Check bug: PASS/FAIL

> Chỉ khi cả 3 mục PASS mới được xóa plan/phase.

## 5) Final Decision

- Keep/Delete phase artifacts:
- Reason:
- Approver:
