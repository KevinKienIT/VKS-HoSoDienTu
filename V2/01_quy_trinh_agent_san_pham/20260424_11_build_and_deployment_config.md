# BUILD AND DEPLOYMENT CONFIG

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dinh nghia cau hinh Electron, build script, single instance enforcement, va deployment.
Nguon prompt: Tu yeu cau nguoi dung ve file chay kieu exe, single instance.

---

## 1. Electron Configuration

### 1.1 Main Process (main.ts)

```typescript
// electron/main.ts
import { app, BrowserWindow, ipcMain, dialog, Menu, Tray, globalShortcut } from 'electron';
import * as path from 'path';
import * as fs from 'fs';

const isDev = process.env.NODE_ENV === 'development';

// Single instance lock
const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
  app.quit();
} else {
  app.on('second-instance', (event, commandLine, workingDirectory) => {
    // Focus existing window
    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore();
      mainWindow.focus();
    }
  });
}

let mainWindow: BrowserWindow | null = null;
let tray: Tray | null = null;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 1024,
    minHeight: 700,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'),
      sandbox: false
    },
    show: false,
    backgroundColor: '#ffffff'
  });

  // Load app
  if (isDev) {
    mainWindow.loadURL('http://localhost:5173');
    mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
  }

  // Show when ready
  mainWindow.once('ready-to-show', () => {
    mainWindow?.show();
  });

  // Handle close to tray
  mainWindow.on('close', (event) => {
    if (!app.isQuitting) {
      event.preventDefault();
      mainWindow?.hide();
    }
  });
}

// App ready
app.whenReady().then(() => {
  createWindow();
  createMenu();
  createTray();
  
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

// Quit when all windows closed
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// App before quit
app.on('before-quit', () => {
  (app as any).isQuitting = true;
});
```

### 1.2 Preload Script

```typescript
// electron/preload.ts
import { contextBridge, ipcRenderer } from 'electron';

// Expose protected methods
contextBridge.exposeInMainWorld('electronAPI', {
  // App
  getVersion: () => ipcRenderer.invoke('app:getVersion'),
  getPath: (name: string) => ipcRenderer.invoke('app:getPath', name),
  
  // Window
  minimize: () => ipcRenderer.invoke('window:minimize'),
  maximize: () => ipcRenderer.invoke('window:maximize'),
  close: () => ipcRenderer.invoke('window:close'),
  isMaximized: () => ipcRenderer.invoke('window:isMaximized'),
  
  // File operations
  openFile: (options: any) => ipcRenderer.invoke('dialog:openFile', options),
  saveFile: (options: any) => ipcRenderer.invoke('dialog:saveFile', options),
  
  // Database
  dbQuery: (sql: string, params: any[]) => ipcRenderer.invoke('db:query', sql, params),
  dbRun: (sql: string, params: any[]) => ipcRenderer.invoke('db:run', sql, params),
  
  // Events
  onMenuAction: (callback: (action: string) => void) => {
    ipcRenderer.on('menu:action', (_, action) => callback(action));
  }
});
```

### 1.3 IPC Handlers

```typescript
// electron/ipc.ts
import { ipcMain, app, dialog, BrowserWindow } from 'electron';
import * as path from 'path';
import * as fs from 'fs';
import Database from 'better-sqlite3';

const dbPath = path.join(app.getPath('userData'), 'vks-ecms.db');
const db = new Database(dbPath);

// App handlers
ipcMain.handle('app:getVersion', () => app.getVersion());
ipcMain.handle('app:getPath', (_, name: string) => app.getPath(name as any));

// Window handlers
ipcMain.handle('window:minimize', () => {
  BrowserWindow.getFocusedWindow()?.minimize();
});

ipcMain.handle('window:maximize', () => {
  const win = BrowserWindow.getFocusedWindow();
  if (win?.isMaximized()) {
    win.unmaximize();
  } else {
    win?.maximize();
  }
});

ipcMain.handle('window:close', () => {
  BrowserWindow.getFocusedWindow()?.close();
});

// File dialogs
ipcMain.handle('dialog:openFile', async (_, options) => {
  const result = await dialog.showOpenDialog(options);
  return result;
});

// Database handlers
ipcMain.handle('db:query', (_, sql: string, params: any[]) => {
  return db.prepare(sql).all(...params);
});

ipcMain.handle('db:run', (_, sql: string, params: any[]) => {
  return db.prepare(sql).run(...params);
});
```

---

## 2. Build Configuration

### 2.1 package.json

```json
{
  "name": "vks-ecms",
  "version": "1.0.0",
  "description": "VKS Ho So Dien Tu - Desktop Application",
  "main": "dist-electron/main.js",
  "author": "VKS Team",
  "license": "MIT",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build && electron-builder",
    "build:win": "tsc && vite build && electron-builder --win",
    "preview": "vite preview"
  },
  "dependencies": {
    "better-sqlite3": "^9.4.0",
    "electron-log": "^5.1.0",
    "pdfjs-dist": "^4.0.0",
    "zustand": "^4.5.0",
    "framer-motion": "^11.0.0",
    "uuid": "^9.0.0"
  },
  "devDependencies": {
    "@types/better-sqlite3": "^7.6.0",
    "@types/node": "^20.0.0",
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@types/uuid": "^9.0.0",
    "@vitejs/plugin-react": "^4.0.0",
    "electron": "^28.0.0",
    "electron-builder": "^24.0.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "vite-plugin-electron": "^0.28.0",
    "vite-plugin-electron-renderer": "^0.14.0"
  },
  "build": {
    "appId": "com.vks.ecms",
    "productName": "VKS ECMS",
    "directories": {
      "output": "release"
    },
    "files": [
      "dist/**/*",
      "dist-electron/**/*"
    ],
    "win": {
      "target": [
        {
          "target": "nsis",
          "arch": ["x64"]
        }
      ],
      "icon": "public/icon.ico"
    },
    "nsis": {
      "oneClick": false,
      "perMachine": false,
      "allowToChangeInstallationDirectory": true,
      "createDesktopShortcut": true,
      "createStartMenuShortcut": true
    }
  }
}
```

### 2.2 Vite Config

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import electron from 'vite-plugin-electron';
import renderer from 'vite-plugin-electron-renderer';
import * as path from 'path';

export default defineConfig({
  plugins: [
    react(),
    electron([
      {
        entry: 'electron/main.ts',
        onstart(options) {
          options.startup();
        },
        vite: {
          build: {
            outDir: 'dist-electron',
            rollupOptions: {
              external: ['better-sqlite3']
            }
          }
        }
      },
      {
        entry: 'electron/preload.ts',
        onstart(options) {
          options.reload();
        },
        vite: {
          build: {
            outDir: 'dist-electron'
          }
        }
      }
    ]),
    renderer()
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src')
    }
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true
  }
});
```

### 2.3 TypeScript Config

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  },
  "include": ["src", "electron"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

---

## 3. Application Menu

### 3.1 Menu Template

```typescript
function createMenu() {
  const template: Electron.MenuItemConstructorOptions[] = [
    {
      label: 'File',
      submenu: [
        {
          label: 'New Case',
          accelerator: 'CmdOrCtrl+N',
          click: () => mainWindow?.webContents.send('menu:action', 'new-case')
        },
        {
          label: 'Open Folder...',
          accelerator: 'CmdOrCtrl+O',
          click: () => openFolder()
        },
        { type: 'separator' },
        {
          label: 'Export...',
          accelerator: 'CmdOrCtrl+E',
          click: () => mainWindow?.webContents.send('menu:action', 'export')
        },
        { type: 'separator' },
        {
          label: 'Exit',
          accelerator: 'Alt+F4',
          click: () => {
            (app as any).isQuitting = true;
            app.quit();
          }
        }
      ]
    },
    {
      label: 'Edit',
      submenu: [
        { role: 'undo' },
        { role: 'redo' },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        { role: 'selectAll' }
      ]
    },
    {
      label: 'View',
      submenu: [
        {
          label: 'Sidebar',
          accelerator: 'CmdOrCtrl+B',
          click: () => mainWindow?.webContents.send('menu:action', 'toggle-sidebar')
        },
        { type: 'separator' },
        { role: 'reload' },
        { role: 'forceReload' },
        { role: 'toggleDevTools' },
        { type: 'separator' },
        { role: 'resetZoom' },
        { role: 'zoomIn' },
        { role: 'zoomOut' },
        { type: 'separator' },
        { role: 'togglefullscreen' }
      ]
    },
    {
      label: 'Window',
      submenu: [
        { role: 'minimize' },
        { role: 'zoom' },
        { type: 'separator' },
        { role: 'close' }
      ]
    },
    {
      label: 'Help',
      submenu: [
        {
          label: 'About',
          click: () => {
            dialog.showMessageBox({
              type: 'info',
              title: 'About VKS ECMS',
              message: 'VKS Ho So Dien Tu',
              detail: `Version: ${app.getVersion()}\nElectron: ${process.versions.electron}`
            });
          }
        }
      ]
    }
  ];

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
}
```

---

## 4. System Tray

### 4.1 Tray Configuration

```typescript
function createTray() {
  // Use default icon for now
  tray = new Tray(path.join(__dirname, '../public/icon.png'));
  
  const contextMenu = Menu.buildFromTemplate([
    {
      label: 'Open VKS ECMS',
      click: () => {
        mainWindow?.show();
      }
    },
    { type: 'separator' },
    {
      label: 'New Case',
      click: () => {
        mainWindow?.show();
        mainWindow?.webContents.send('menu:action', 'new-case');
      }
    },
    { type: 'separator' },
    {
      label: 'Exit',
      click: () => {
        (app as any).isQuitting = true;
        app.quit();
      }
    }
  ]);
  
  tray.setToolTip('VKS ECMS');
  tray.setContextMenu(contextMenu);
  
  tray.on('double-click', () => {
    mainWindow?.show();
  });
}
```

---

## 5. Global Shortcuts

```typescript
// Register global shortcuts
app.whenReady().then(() => {
  // Global: Show/Hide app
  globalShortcut.register('CommandOrControl+Shift+V', () => {
    if (mainWindow?.isVisible()) {
      mainWindow.hide();
    } else {
      mainWindow?.show();
    }
  });
});

app.on('will-quit', () => {
  globalShortcut.unregisterAll();
});
```

---

## 6. Logging

### 6.1 Logger Setup

```typescript
// electron/logger.ts
import log from 'electron-log';
import * as path from 'path';
import { app } from 'electron';

const logPath = path.join(app.getPath('userData'), 'logs');

// Configure logging
log.transports.file.resolvePathFn = () => path.join(logPath, 'main.log');
log.transports.file.level = 'info';
log.transports.file.maxSize = 10 * 1024 * 1024; // 10MB
log.transports.console.level = 'debug';

// Export for use in main process
export default log;
```

### 6.2 Error Handlers

```typescript
// Handle uncaught exceptions
process.on('uncaughtException', (error) => {
  log.error('Uncaught Exception:', error);
  dialog.showErrorBox('Error', `An unexpected error occurred: ${error.message}`);
  app.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  log.error('Unhandled Rejection at:', promise, 'reason:', reason);
});
```

---

## 7. File Structure After Build

```
/release
├── win-unpacked/
│   ├── VKS ECMS.exe
│   ├── resources/
│   └── locales/
└── VKS ECMS Setup 1.0.0.exe
```

---

## 8. Acceptance Criteria

- [ ] Single instance enforced
- [ ] Close to tray instead of quit
- [ ] System tray with context menu
- [ ] Application menu fully configured
- [ ] Global shortcut (Ctrl+Shift+V)
- [ ] Logging to file
- [ ] Error dialogs
- [ ] Build to .exe via electron-builder

---

**STATUS: SPECIFICATION DEFINED. WAITING FOR IMPLEMENTATION.**