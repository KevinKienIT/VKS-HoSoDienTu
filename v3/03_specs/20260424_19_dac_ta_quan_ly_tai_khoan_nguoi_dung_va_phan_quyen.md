# ĐẶC TẢ QUẢN LÝ TÀI KHOẢN NGƯỜI DÙNG VÀ PHÂN QUYỀN

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Định nghĩa quản lý tài khoản người dùng và hệ thống phân quyền cho ứng dụng VKS ECMS.
Nguon prompt: Tu yeu cau ve user management va permission.

---

## 1. Tong Quan User Management

### 1.1 Mau Thieu Đoạn Hien Tai

- **KHÔNG có formal user management**
- **KHÔNG có permission system**
- **KHÔNG có role-based access**

### 1.2 Muc Tieu Cua File Nay

- User account management
- Role and permission system
- Access control
- Session management

---

## 2. User Roles

### 2.1 Role Hierarchy

```
┌─────────────────────────────────────┐
│          ADMIN (Level 5)             │
│      ────────────────────────        │
│     tat ca quyen + quan tri he thong     │
├─────────────────────────────────────┤
│          KSV (Level 4)              │
│      ────────────────────────        │
│      doc/ghi/duyet + import/export   │
├─────────────────────────────────────┤
│          EDITOR (Level 3)            │
│      ────────────────────────        │
│      doc/ghi/duyet + export         │
├─────────────────────────────────────┤
│          VIEWER (Level 2)           │
│      ────────────────────────        │
│      chi doc + tim kiem             │
└─────────────────────────────────────┘
```

### 2.2 Role Definitions

| Role | Level | Description | Use Case |
|------|-------|------------|---------|
| **Admin** | 5 | Full access | IT Admin |
| **KSV** | 4 | Kiểm sát viên | Normal users |
| **Editor** | 3 | Edit approved | Reviewers |
| **Viewer** | 2 | Read only | Guests |

### 2.3 Default Permissions

| Permission | Admin | KSV | Editor | Viewer |
|------------|------|-----|--------|--------|
| case.read | ✅ | ✅ | ✅ | ✅ |
| case.write | ✅ | ✅ | ❌ | ❌ |
| case.delete | ✅ | ❌ | ❌ | ❌ |
| document.read | ✅ | ✅ | ✅ | ✅ |
| document.write | ✅ | ✅ | ❌ | ❌ |
| document.import | ✅ | ✅ | ❌ | ❌ |
| document.export | ✅ | ✅ | ✅ | ❌ |
| citation.read | ✅ | ✅ | ✅ | ✅ |
| citation.write | ✅ | ✅ | ✅ | ❌ |
| review.ocr | ✅ | ✅ | ✅ | ❌ |
| review.approve | ✅ | ✅ | ❌ | ❌ |
| user.read | ✅ | ❌ | ❌ | ❌ |
| user.write | ✅ | ❌ | ❌ | ❌ |
| settings | ✅ | ❌ | ❌ | ❌ |
| audit.read | ✅ | ✅ | ❌ | ❌ |

---

## 3. User Account Management

### 3.1 User List Screen

```
┌──────────────────────────────────────────────────────────────────────┐
│ Quan Ly Tai Khoan                    [+ Them] [Export] [Filter ▾]      │
├──────────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌────────��──────────────────────────────────────────────────────┐    │
│  │ □ │ [Avatar] │ Ten          │ Vai tro │ Don vi     │ Trang Thai │   │
│  │───│─────────│─────────────│───────│─────────│───────────│    │
│  │ ✓ │ 👤     │ Nguyen A   │ KSV   │ VKS TPHM │ Active   │    │
│  │ ✓ │ 👤     │ Nguyen B   │ Editor│ VKS TPHM │ Active   │    │
│  │ ✓ │ 👤     │ Le C      │ Admin │ VKS HN  │ Active   │    │
│  │ ○ │ 👤     │ Tran D    │ Viewer│ VKS DN │ Inactive │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                                                                 │
├──────────────────────────────────────────────────────────────────────┤
│ Showing 1-4 of 45            [< Prev] [1] [2] [3] [Next >]         │
└──────────────────────────────────────────────────────────────────────┘
```

### 3.2 User Form

```
┌─────────────────────────────────────────────────────┐
│ Them Nguoi Dung                  [Cancel] [Save] │
├─────────────────────────────────────────────────────┤
│                                             │
│ Ten dang nhap: [________________]            │
│                                             │
│ Mat khau:    [________________]            │
│ (neu tao moi)                               │
│                                             │
│ Ten hien thi: [________________]             │
│                                             │
│ Email:      [________________]            │
│                                             │
│ Vai tro:    [dropdown ▼]                   │
│ [Admin] [KSV] [Editor] [Viewer]              │
│                                             │
│ Don vi:    [________________]            │
│                                             │
│ Quyen:     [ ] Doc                      │
│        [ ] Ghi                        │
│        [ ] Import                      │
│        [ ] Export                      │
│        [ ] Duyet                       │
│                                             │
│ Trang thai: (•) Active  ( ) Inactive       │
│                                             │
└─────────────────────────────────────────────────────┘
```

### 3.3 User Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| username | Text | ✅ | Unique login name |
| password | Password | ✅ | Initial password |
| display_name | Text | ✅ | Display name |
| email | Email | | Email address |
| role | Select | ✅ | User role |
| permissions | Array | | Custom permissions |
| office | Text | | Work unit |
| status | Enum | ✅ | Active/Inactive |
| preferences | JSON | | User settings |

---

## 4. Permissions System

### 4.1 Permission Matrix

| Permission | Description | Component |
|------------|-------------|----------|
| `case.read` | Xem case | CaseList, CaseDetail |
| `case.write` | Tao/sua case | CaseForm |
| `case.delete` | Xoa case | CaseList |
| `document.read` | Xem tai lieu | DocumentList |
| `document.write` | Sua tai lieu | DocumentForm |
| `document.import` | Import tai lieu | Import |
| `document.export` | Xuat tai lieu | Export |
| `citation.read` | Xem trich dan | Citation |
| `citation.write` | Tao trich dan | CitationForm |
| `review.ocr` | Review OCR | ReviewQueue |
| `review.approve` | Duyet review | ReviewItem |
| `user.read` | Xem nguoi dung | UserList |
| `user.write` | Tao/sua nguoi dung | UserForm |
| `settings` | Cau hinh he thong | Settings |
| `audit.read` | Xem audit trail | AuditLog |

### 4.2 Permission Check

```typescript
// Permission check pattern
function hasPermission(user: User, permission: string): boolean {
  // 1. Check role level
  const roleLevels = { admin: 5, ksv: 4, editor: 3, viewer: 2 };
  const requiredLevel = getRequiredLevel(permission);
  if (roleLevels[user.role] >= requiredLevel) return true;
  
  // 2. Check explicit permission
  if (user.permissions?.includes(permission)) return true;
  
  return false;
}
```

### 4.3 Conditional Permissions

| Permission | Condition |
|-------------|------------|
| case.delete | Created by current user |
| document.export | document.case_id in user.assigned_cases |
| review.approve | review.reviewer_id == user.id |

---

## 5. Authentication

### 5.1 Login Flow

```
┌─────────────────────────┐
│     VKS ECMS           │
│   ─────────────────   │
│     [Logo]            │
│                       │
│ User: [____________]  │
│ Pass: [____________]  │
│                       │
│    [Dang nhap]       │
│                       │
│ [Quen mat khau]      │
└─────────────────────────┘
```

### 5.2 Session Management

| Feature | Implementation |
|---------|---------------|
| Login | Username + password |
| Token | JWT, 8 hours |
| Refresh | Refresh token |
| Logout | Invalidate token |

### 5.3 Security Rules

| Rule | Description |
|------|-------------|
| Password minimum | 8 characters |
| Lockout | 5 failed attempts, 15 min |
| Session timeout | 8 hours |
| Password expiry | 90 days |

---

## 6. User Preferences

### 6.1 Preferences Storage

```json
{
  "user_id": "uuid",
  "display": {
    "theme": "light",
    "fontSize": 14,
    "sidebarWidth": 280
  },
  "viewer": {
    "defaultZoom": "fit-width",
    "showPageNumbers": true,
    "autoRotate": true
  },
  "search": {
    "recentQueries": [],
    "defaultFilters": {}
  },
  "notifications": {
    "email": true,
    "inApp": true
  }
}
```

### 6.2 Preferences Screen

```
┌─────────────────────────────────────────────┐
│ Cau hinh                 [Luu] [Mac dinh]   │
├─────────────────────────────────────────────┤
│                                             │
│  Cau hinh hien thi                          │
│  ──────────────────                        │
│  Giao dien: (•) Sang  ( ) Toi  ( ) He thong  │
│  Font size: [12] ----O---- [24]             │
│  Sidebar width: 200 ----O---- 400           │
│                                             │
│  Cau hinh viewer                           │
│  ──────────────────                        │
│  Zoom mac dinh: [Fit Width ▼]              │
│  Hien so trang: (•) Co  ( ) Khong          │
│  Xoay tu dong: (•) Co  ( ) Khong          │
│                                             │
│  Thong bao                                │
│  ──────────────────                        │
│  Email: (•) Co  ( ) Khong                   │
│  Trong ung dung: (•) Co  ( ) Khong         │
└─────────────────────────────────────────────┘
```

---

## 7. Audit Trail for Users

### 7.1 User Actions Logged

| Action | Logged |
|--------|-------|
| Login | ✅ |
| Logout | ✅ |
| Failed login | ✅ |
| Password change | ✅ |
| Role change | ✅ |
| Permission change | ✅ |
| User create | ✅ |
| User update | ✅ |
| User deactivate | ✅ |

### 7.2 Audit Fields

| Field | Description |
|-------|-------------|
| user_id | Target user |
| actor_id | Who performed action |
| action | Action type |
| before_value | Previous value |
| after_value | New value |
| timestamp | When |
| ip_address | Where |

---

## 8. State Management

### 8.1 User Store

```typescript
interface UserState {
  currentUser: User | null;
  users: User[];
  loading: boolean;
  error: string | null;
  
  // Actions
  login: (username: string, password: string) => Promise<User>;
  logout: () => Promise<void>;
  fetchUsers: () => Promise<void>;
  createUser: (data: CreateUserInput) => Promise<User>;
  updateUser: (id: string, data: UpdateUserInput) => Promise<User>;
  deleteUser: (id: string) => Promise<void>;
}
```

### 8.2 Auth Store

```typescript
interface AuthState {
  token: string | null;
  refreshToken: string | null;
  expiresAt: Date | null;
  isAuthenticated: boolean;
  
  // Actions
  setToken: (token: string) => void;
  clearToken: () => void;
  refreshToken: () => Promise<void>;
}
```

---

## 9. Data Flow

### 9.1 Login Flow

```
User submits credentials
         │
         ▼
┌─────────────────────┐
│ Validate          │
│ username/password │
├──────────────────┤
│ Generate JWT     │
│ token           │
├──────────────────┤
│ Store in local  │
│ storage         │
├──────────────────┤
│ Redirect to     │
│ dashboard      │
└─────────────────────┘
```

### 9.2 Permission Check Flow

```
User performs action
         │
         ▼
┌─────────────────────┐
│ Check permission │
│ in user store  │
├──────────────────┤
│ Has permission? │───► Yes → Allow
│ No → Deny   │         │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ Show error      │
│ "Khong co quyen"│
└─────────────────────┘
```

---

## 10. Acceptance Criteria

- [ ] User list hien thi danh sach
- [ ] User form create/sua/dung
- [ ] Role system 4 cap
- [ ] Permission matrix day du
- [ ] Login/logout work
- [ ] Permission check work
- [ ] Preferences save/load
- [ ] Audit trail for user actions

---

## 11. Khong Duoc Hieu Sai

- **Admin KHONG phai la superuser cua OS** - chi trong app context
- **Permission KHONG phai la ACL phan tan** - chi local app
- **User preferences KHONG phai la profile** - chi settings

---

**STATUS: SPECIFICATION DEFINED. READY FOR IMPLEMENTATION.**

*File thuoc Nhom C - UI & Man hinh*
*Tiep theo: File 8 - Ban do luong chuc nang*