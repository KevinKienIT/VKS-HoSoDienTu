import { useEffect, useRef, useState } from "react";
import {
    HashRouter,
    Link,
    Navigate,
    Route,
    Routes,
    useLocation,
    useNavigate,
} from "react-router-dom";
import {
    LayoutDashboard,
    FolderInput,
    ScanLine,
    FileSearch,
    FolderKanban,
    BrainCircuit,
    Search,
    Settings,
    ChevronLeft,
    ChevronRight,
    ClipboardCheck,
    FileOutput,
} from "lucide-react";
import * as searchService from "./services/searchService";
import { useCatalogStore } from "./store/catalogStore";
import { useModuleStore } from "./store/moduleStore";
import { useUiStore } from "./store/uiStore";
import { DashboardPage } from "./modules/bangdieukhien/BangDieuKhienPage";
import { CaseListPage } from "./modules/hosovuan/DanhSachVuAnPage";
import { CaseDetailPage } from "./modules/hosovuan/ChiTietVuAnPage";
import { DocumentViewerPage } from "./modules/phantichtailieu/ChiTietXemTaiLieuPage";
import { SearchPage } from "./modules/timkiem/TimKiemPage";
import { ImportJobPage } from "./modules/duahosovao/DuaHoSoVaoPage";
import { SettingsPage } from "./modules/cauhinh/CauHinhPage";
import { DocumentListPage } from "./modules/quantailieu/DanhSachTaiLieuPage";
import { AnalyzePage } from "./modules/phantichtailieu/PhanTichTaiLieuPage";
import { AiWorkspacePage } from "./modules/phantichai/KhongGianAIPage";
import { ExportPage } from "./modules/xuatbangiao/XuatBanGiaoPage";
import { ScanPage } from "./modules/quettailieu/QuetTaiLieuPage";

// Main workflow navigation — ordered by dossier business flow
const NAV_MAIN = [
    { path: "/", icon: LayoutDashboard, label: "Dashboard", num: "01" },
    { path: "/import", icon: FolderInput, label: "Đưa hồ sơ vào", num: "02" },
    { path: "/scan", icon: ScanLine, label: "Scan tài liệu", num: "03" },
    { path: "/analyze", icon: FileSearch, label: "Phân tích tài liệu", num: "04" },
    { path: "/documents", icon: FolderKanban, label: "Quản lý tài liệu", num: "05" },
    { path: "/ai", icon: BrainCircuit, label: "Phân tích AI", num: "06" },
    { path: "/search", icon: Search, label: "Tìm kiếm", num: "07" },
];

function Sidebar() {
    const location = useLocation();
    const collapsed = useUiStore((s) => s.sidebarCollapsed);
    const toggleSidebar = useUiStore((s) => s.toggleSidebar);

    const isActive = (path: string) =>
        path === "/"
            ? location.pathname === "/"
            : location.pathname === path || location.pathname.startsWith(`${path}/`);

    return (
        <aside className={`app-sidebar ${collapsed ? "collapsed" : ""}`}>
            <div className="dossier-header">
                <span className="dossier-folder-icon">⬡</span>
                <span className="dossier-title">VKS ECMS</span>
            </div>

            <nav className="dossier-spine">
                {NAV_MAIN.map((item) => {
                    const Icon = item.icon;
                    return (
                        <Link
                            key={item.path}
                            to={item.path}
                            className={`dossier-tab ${isActive(item.path) ? "active" : ""}`}
                            title={collapsed ? item.label : undefined}
                        >
                            <span className="dossier-tab-icon">
                                <Icon />
                            </span>
                            <span className="dossier-tab-label">{item.label}</span>
                            <span className="dossier-tab-number">{item.num}</span>
                        </Link>
                    );
                })}
            </nav>

            <div className="sidebar-bottom">
                <div className="dossier-divider" />
                <Link
                    to="/reviews"
                    className={`sidebar-settings-btn ${isActive("/reviews") ? "active" : ""}`}
                    title="Review"
                >
                    <span className="dossier-tab-icon">
                        <ClipboardCheck />
                    </span>
                    <span className="dossier-tab-label">Review</span>
                </Link>
                <Link
                    to="/export"
                    className={`sidebar-settings-btn ${isActive("/export") ? "active" : ""}`}
                    title="Export PDF"
                >
                    <span className="dossier-tab-icon">
                        <FileOutput />
                    </span>
                    <span className="dossier-tab-label">Export PDF</span>
                </Link>
                <button className="sidebar-toggle" onClick={toggleSidebar} title={collapsed ? "Mở rộng" : "Thu gọn"}>
                    {collapsed ? <ChevronRight size={14} /> : <ChevronLeft size={14} />}
                </button>
                <Link
                    to="/settings"
                    className={`sidebar-settings-btn ${isActive("/settings") ? "active" : ""}`}
                    title="Cài đặt"
                >
                    <span className="dossier-tab-icon">
                        <Settings />
                    </span>
                    <span className="dossier-tab-label">Cài đặt</span>
                </Link>
            </div>
        </aside>
    );
}

function AppShell() {
    const navigate = useNavigate();
    const online = useUiStore((s) => s.online);
    const statusText = useUiStore((s) => s.statusText);
    const stats = useCatalogStore((s) => s.stats);
    const pipelineFooter = useUiStore((s) => s.pipelineFooter);
    const pipelineProgress = useUiStore((s) => s.pipelineProgress);
    const refreshPipelineStatus = useUiStore((s) => s.refreshPipelineStatus);
    const [globalQuery, setGlobalQuery] = useState("");
    const searchRef = useRef<HTMLInputElement | null>(null);

    useEffect(() => {
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.ctrlKey && e.key.toLowerCase() === "k") {
                e.preventDefault();
                searchRef.current?.focus();
            }
        };
        window.addEventListener("keydown", onKeyDown);
        return () => window.removeEventListener("keydown", onKeyDown);
    }, []);

    useEffect(() => {
        refreshPipelineStatus();
        const t = window.setInterval(() => {
            refreshPipelineStatus();
        }, 2500);
        return () => window.clearInterval(t);
    }, [refreshPipelineStatus]);

    const handleSearch = async () => {
        const q = globalQuery.trim();
        if (!q) return;
        const preview = await searchService.ftsSearch(q, 1);
        if (preview.length > 0) {
            navigate(
                `/cases/${preview[0].case_id}/docs/${preview[0].document_id}?page=${preview[0].page_number || 1}&q=${encodeURIComponent(q)}`
            );
            return;
        }
        navigate(`/search?q=${encodeURIComponent(q)}`);
    };

    return (
        <div className="app-shell">
            <header className="app-header">
                <div className="header-brand">
                    <div className="header-brand-icon">⬡</div>
                    <div>
                        <div className="header-title">VKS Hồ Sơ Điện Tử</div>
                        <div className="header-subtitle">Quản lý hồ sơ án</div>
                    </div>
                </div>

                <div className="header-spacer" />

                <div className="header-search">
                    <span className="header-search-icon">
                        <Search size={13} />
                    </span>
                    <input
                        ref={searchRef}
                        className="header-search-input"
                        type="text"
                        value={globalQuery}
                        onChange={(e) => setGlobalQuery(e.target.value)}
                        placeholder="Tìm kiếm tài liệu, hồ sơ..."
                        onKeyDown={(e) => {
                            if (e.key === "Enter") handleSearch();
                        }}
                    />
                    <span className="header-search-hint">Ctrl+K</span>
                </div>

                <div className="header-actions">
                    <button className="header-action-btn" title="Thông báo">🔔</button>
                </div>
            </header>

            <Sidebar />

            <main className="app-main">
                {!online ? <BrowserRuntimeWarning /> : null}
                <Routes>
                    <Route path="/" element={<DashboardPage />} />
                    <Route path="/import" element={<ImportJobPage />} />
                    <Route path="/scan" element={<ScanPage />} />
                    <Route path="/analyze" element={<AnalyzePage />} />
                    <Route path="/documents" element={<DocumentListPage />} />
                    <Route path="/ai" element={<AiWorkspacePage />} />
                    <Route path="/ai/:caseId" element={<AiWorkspacePage />} />
                    <Route path="/export" element={<ExportPage />} />
                    <Route path="/search" element={<SearchPage />} />
                    <Route path="/reviews" element={<Navigate to="/analyze?tab=review" replace />} />
                    <Route path="/settings" element={<SettingsPage />} />
                    <Route path="/cases" element={<CaseListPage />} />
                    <Route path="/cases/:caseId" element={<CaseDetailPage />} />
                    <Route path="/cases/:caseId/docs/:docId" element={<DocumentViewerPage />} />
                    <Route path="*" element={<Navigate to="/" replace />} />
                </Routes>
            </main>

            <footer className="app-footer">
                <div className="status-group">
                    <span>
                        <span className={`status-dot ${online ? "" : "offline"}`} />
                        {online ? "Desktop Runtime" : "Browser Preview"}
                    </span>
                    <span>{statusText}</span>
                </div>
                <div className="status-group">
                    <span>Tài liệu: {stats?.total_files ?? 0}</span>
                    <span>Imported: {stats?.by_status?.imported ?? 0}</span>
                    <span>Pipeline active: {pipelineFooter.active_jobs}</span>
                    <span>Paused: {pipelineFooter.paused_jobs}</span>
                    {pipelineProgress ? (
                        <span>
                            Pipeline {pipelineProgress.phase}: {pipelineProgress.processed_tasks}/{pipelineProgress.total_tasks} ({pipelineProgress.progress_percent.toFixed(0)}%)
                        </span>
                    ) : null}
                </div>
            </footer>
        </div>
    );
}

function BrowserRuntimeWarning() {
    return (
        <div className="runtime-warning">
            <div>
                <div className="runtime-warning-title">Sai hướng chạy: Browser Preview</div>
                <div className="runtime-warning-text">
                    Đây là ứng dụng Tauri desktop. Browser preview chỉ xem được layout; các chức năng SQLite, import, OCR,
                    search, review và AI offline phải chạy bằng <b>npm run tauri:dev</b>.
                </div>
            </div>
            <span className="badge badge-danger">No Tauri runtime</span>
        </div>
    );
}

function App() {
    const loadConfigs = useModuleStore((s) => s.loadConfigs);
    const setOnline = useUiStore((s) => s.setOnline);
    const setStatusText = useUiStore((s) => s.setStatusText);

    useEffect(() => {
        loadConfigs();
    }, [loadConfigs]);

    useEffect(() => {
        const hasTauriRuntime = "__TAURI_INTERNALS__" in window;
        setOnline(hasTauriRuntime);
        setStatusText(hasTauriRuntime ? "Tauri v2" : "Browser preview only - sai huong chuc nang");
    }, [setOnline, setStatusText]);

    return (
        <HashRouter>
            <AppShell />
        </HashRouter>
    );
}

export default App;
