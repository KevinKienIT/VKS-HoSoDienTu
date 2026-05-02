# QUYET DINH NEN TANG VA CACH CHAY PHAN MEM

Ngay tao: 2026-04-26
Ngay cap nhat gan nhat: 2026-04-26
Sua boi agent: Antigravity
Muc dich: Thong nhat duy nhat mot nen tang chay, xoa bo su nham lan giua "web app" va "desktop app".

---

## 1. Quyet Dinh: TAURI DESKTOP APP (.exe)

**Phan mem nay la ung dung desktop**, khong phai web app.

| Tieu chi | Quyet dinh | Ly do |
|----------|-----------|-------|
| Nen tang | Tauri 2.x (Rust backend + WebView frontend) | Offline 100%, truy cap file system, SQLite local |
| Dau ra | File `.exe` cai dat tren Windows | Nguoi dung (KSV) khong can mo browser |
| Frontend | React + Vite (chi lam UI, **khong** chay doc lap) | Render trong Tauri WebView, khong phai browser |
| Database | SQLite qua `tauri-plugin-sql` + `rusqlite` | Offline, khong can server |
| File system | `tauri-plugin-fs` + Rust `std::fs` | Doc/ghi file scan truc tiep tren may |

---

## 2. Su Nham Lan Can Xoa Bo

### 2.1 Van de hien tai

| Lenh | Thuc te | Van de |
|------|---------|--------|
| `npm run dev` | Chay Vite dev server tren `localhost:5173`, mo trong browser | Frontend goi `@tauri-apps/api` nhung **khong co Tauri runtime** → moi Tauri API call deu **that bai im lang** |
| `npm run tauri dev` | Chay Rust backend + Vite cung luc, mo trong Tauri window | Day moi la cach chay dung |
| `npm run build` | Build frontend ra `dist/` | Chi la buoc trung gian, khong phai san pham |
| `npm run tauri build` | Build toan bo ra `.exe` | Day moi la san pham |

### 2.2 He qua cua viec chay `npm run dev` tren browser

- `useModuleStore` goi `get_module_configs` qua Tauri invoke → **fail** → dung fallback data
- Khong ket noi duoc SQLite → moi thao tac data deu **gia**
- File system API khong hoat dong → import/export **khong chay**
- Ung dung tren browser chi la **cai vo giao dien rong**, khong co chuc nang thuc

---

## 3. Quy Dinh Cach Chay Chinh Thuc

### 3.1 Khi phat trien (Development)

```powershell
# DUNG: Chay Tauri dev mode (Rust + Vite cung luc)
cd d:\JOBS\VKS-HoSoDienTu\PhanMem
npm run tauri dev

# SAI: Khong chay chi Vite
# npm run dev  ← CHI DUNG KHI SỬA CSS/LAYOUT THUAN TUY, KHONG TEST CHUC NANG
```

**Luu y**: `npm run tauri dev` se:
1. Chay `npm run dev` (Vite) tu dong (cau hinh trong `tauri.conf.json` > `build.beforeDevCommand`)
2. Build va chay Rust backend
3. Mo cua so Tauri voi WebView tro vao `localhost:5173`
4. Tat ca Tauri API (SQL, filesystem, dialog, shell) deu hoat dong

### 3.2 Khi build san pham (Production)

```powershell
cd d:\JOBS\VKS-HoSoDienTu\PhanMem
npm run tauri build
```

San pham dau ra: `src-tauri/target/release/bundle/nsis/VKS ECMS_1.0.0_x64-setup.exe`

### 3.3 Khi chi sửa giao dien (UI-only, khong can chuc nang)

```powershell
# Chi dung khi dang chinh CSS, layout, animation — khong test data/chuc nang
cd d:\JOBS\VKS-HoSoDienTu\PhanMem
npm run dev
```

Truong hop nay chap nhan duoc vi chi xem visual, khong test logic. Nhung **phai hieu rang tat ca data la gia/fallback**.

---

## 4. Cau Truc Thu Muc Chinh Thuc

```
PhanMem/
├── src/                    ← Frontend (React + TypeScript)
│   ├── App.tsx             ← App shell, routing, fallback UI
│   ├── main.tsx            ← Entry point
│   ├── components/         ← UI components dung chung
│   ├── modules/            ← Module slot system
│   ├── store/              ← Zustand state management
│   ├── services/           ← Goi Tauri API (invoke)
│   ├── hooks/              ← Custom React hooks
│   ├── styles/             ← CSS files
│   └── lib/                ← Utilities
│
├── src-tauri/              ← Backend (Rust + Tauri)
│   ├── src/
│   │   ├── main.rs         ← Tauri app entry, plugin setup
│   │   ├── commands/       ← Tauri commands (invoke handlers)
│   │   └── db/             ← SQLite schema, queries
│   ├── Cargo.toml          ← Rust dependencies
│   ├── tauri.conf.json     ← Tauri config (window, plugins, build)
│   ├── migrations/         ← SQL migration files
│   └── icons/              ← App icons
│
├── python/                 ← Python scripts (OCR, AI) — goi qua shell
├── scripts/                ← Build/dev scripts
├── package.json            ← Node dependencies (frontend)
├── vite.config.ts          ← Vite build config
└── index.html              ← HTML template cho WebView
```

---

## 5. Technology Stack Chinh Thuc

| Tang | Cong nghe | Phien ban | Vai tro |
|------|----------|----------|---------|
| **Runtime** | Tauri 2.x | ^2.0.0 | Native window, system API, plugin bridge |
| **Backend** | Rust | edition 2021 | Commands, DB, file processing |
| **Database** | SQLite (WAL mode) | via rusqlite 0.31 | Du lieu offline |
| **Frontend** | React 18 | ^18.2.0 | UI rendering |
| **State** | Zustand 4 | ^4.5.0 | Client state management |
| **Build** | Vite 8 | ^8.0.10 | Frontend bundling |
| **Routing** | react-router-dom 6 | ^6.30.1 | SPA navigation |
| **Animation** | framer-motion 11 | ^11.0.0 | UI transitions |
| **OCR** | PaddleOCR (Python) | TBD | Goi qua tauri-plugin-shell |
| **AI Offline** | Ollama (local LLM) | TBD | Phase 3 |
| **Installer** | NSIS | via Tauri bundle | Windows installer |

---

## 6. Quy Tac Cho Agent

1. **KHONG duoc chi chay `npm run dev` de test chuc nang.** Phai chay `npm run tauri dev`.
2. **KHONG duoc viet code frontend phu thuoc browser API** ma Tauri khong ho tro (vi du: `window.fetch` den server ngoai, `localStorage` thay vi SQLite).
3. **Moi Tauri invoke call phai co fallback** trong frontend, de khi chay `npm run dev` (UI-only) khong bi crash.
4. **Khi tao module moi**, phai tao ca:
   - Rust command trong `src-tauri/src/commands/`
   - Frontend service trong `src/services/`
   - UI component/page trong `src/components/` hoac `src/modules/`
5. **File CSS, asset UI, config Vite** nam trong `PhanMem/` goc hoac `src/`.
6. **Script Python** (OCR, AI) nam trong `PhanMem/python/`, goi qua `tauri-plugin-shell`.

---

## 7. Kiem Tra Nhanh Nen Tang

De xac nhan ung dung dang chay dung trong Tauri:

```typescript
// Trong bat ky component nao:
import { invoke } from '@tauri-apps/api/core';

// Neu invoke thanh cong → dang chay trong Tauri
// Neu invoke throw error → dang chay tren browser (sai)
```

Fallback pattern chuan:

```typescript
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    return await invoke<T>(cmd, args);
  } catch {
    console.warn(`[Fallback] Tauri invoke "${cmd}" khong kha dung — dang chay ngoai Tauri?`);
    return null;
  }
}
```

---

**TRANG THAI: DA QUYET DINH. BAT BUOC TUAN THU.**
