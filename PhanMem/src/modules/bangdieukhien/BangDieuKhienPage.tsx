import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import * as aiService from "../phantichai/phantichai.service";
import * as caseService from "../hosovuan/hosovuan.service";
import * as documentService from "../quantailieu/quantailieu.service";
import { useCatalogStore } from "../../store/catalogStore";
import { PageSection } from "./common";

type Accent = "primary" | "success" | "warning" | "danger" | "info";

interface StatCardProps {
    label: string;
    value: number | string;
    accent: Accent;
    hint?: string;
    linkTo?: string;
}

function StatCard({ label, value, accent, hint, linkTo }: StatCardProps) {
    const content = (
        <div className={`card stat-card ${linkTo ? "card-interactive" : ""}`} data-accent={accent}>
            <div className="stat-label">{label}</div>
            <div className="stat-value">{value}</div>
            {hint ? <div className="stat-change">{hint}</div> : null}
        </div>
    );
    if (linkTo) {
        return (
            <Link to={linkTo} className="dashboard-link-reset">
                {content}
            </Link>
        );
    }
    return content;
}

function statusBadge(status: string) {
    const normalized = status.toLowerCase();
    if (normalized === "reviewed" || normalized === "active" || normalized === "imported") {
        return "badge badge-success";
    }
    if (normalized === "error" || normalized === "failed") {
        return "badge badge-danger";
    }
    if (normalized === "processed" || normalized === "cataloged") {
        return "badge badge-info";
    }
    if (normalized === "pending" || normalized === "scanning" || normalized === "importing") {
        return "badge badge-warning";
    }
    return "badge badge-neutral";
}

function formatDate(value: string) {
    if (!value) return "-";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value.slice(0, 10);
    return new Intl.DateTimeFormat("vi-VN", {
        day: "2-digit",
        month: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
    }).format(date);
}

function compareCreatedAt<T extends { created_at: string }>(a: T, b: T) {
    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
}

export function DashboardPage() {
    const [cases, setCases] = useState<caseService.CaseSummary[]>([]);
    const [documents, setDocuments] = useState<documentService.DocumentSummary[]>([]);
    const [groups, setGroups] = useState<documentService.DocumentGroup[]>([]);
    const [aiStatus, setAiStatus] = useState<aiService.AiStatus | null>(null);
    const [error, setError] = useState<string | null>(null);
    const stats = useCatalogStore((s) => s.stats);
    const fetchStats = useCatalogStore((s) => s.fetchStats);

    useEffect(() => {
        fetchStats();
        Promise.all([
            caseService.listCases(),
            documentService.listDocuments(),
            documentService.getDocumentGroups(),
            aiService.checkAiStatus(),
        ])
            .then(([caseItems, docItems, groupItems, ai]) => {
                setCases(caseItems);
                setDocuments(docItems);
                setGroups(groupItems);
                setAiStatus(ai);
            })
            .catch((e) => setError((e as Error).message || "Không thể tải dashboard."));
    }, [fetchStats]);

    const sortedCases = useMemo(() => [...cases].sort(compareCreatedAt), [cases]);
    const sortedDocs = useMemo(() => [...documents].sort(compareCreatedAt), [documents]);
    const reviewDocs = useMemo(
        () => documents.filter((doc) => doc.status !== "reviewed"),
        [documents]
    );
    const pendingOcrDocs = useMemo(() => documents.filter((doc) => doc.status === "pending"), [documents]);
    const ocrErrorDocs = useMemo(() => documents.filter((doc) => doc.status === "error"), [documents]);
    const unclassifiedDocs = useMemo(
        () => documents.filter((doc) => doc.document_type === "khong_xac_dinh" || !doc.document_type),
        [documents]
    );
    const urgentDocs = useMemo(() => {
        const priority: Record<string, number> = { error: 0, pending: 1, processed: 2, reviewed: 3 };
        return [...reviewDocs]
            .sort((a, b) => (priority[a.status] ?? 9) - (priority[b.status] ?? 9) || compareCreatedAt(a, b))
            .slice(0, 6);
    }, [reviewDocs]);

    const totalPages = useMemo(() => cases.reduce((sum, item) => sum + item.total_pages, 0), [cases]);
    const importedCatalog = stats?.by_status?.imported ?? 0;
    const catalogTotal = stats?.total_files ?? 0;
    const catalogBacklog = Math.max(catalogTotal - importedCatalog, 0);
    const readyDocs = documents.filter((doc) => doc.page_count > 0).length;
    const continueDoc = urgentDocs[0] ?? sortedDocs[0] ?? null;
    const activeCases = cases.filter((item) => item.status === "active").length;
    const topGroups = groups.slice(0, 8);
    const importInProgress = false;

    return (
        <PageSection
            title="Dashboard"
            description="Bảng điều khiển hồ sơ: import, OCR, phân loại, review và cảnh báo xử lý"
            right={
                <div className="dashboard-actions">
                    <Link className="btn btn-primary" to="/import">⇩ Import hồ sơ</Link>
                    <Link className="btn btn-ghost" to="/search">⌕ Tìm kiếm</Link>
                </div>
            }
        >
            {error ? <div className="module-error mb-3">! {error}</div> : null}

            <div className="dashboard-alert-row">
                <Link
                    className="dashboard-focus-card card card-interactive"
                    to={continueDoc ? `/cases/${continueDoc.case_id}/docs/${continueDoc.document_id}` : "/import"}
                >
                    <div>
                        <div className="stat-label">Tiếp tục xử lý</div>
                        <div className="dashboard-focus-title">
                            {continueDoc ? continueDoc.display_name : "Import hồ sơ đầu tiên"}
                        </div>
                        <div className="dashboard-focus-meta">
                            {continueDoc
                                ? `${continueDoc.document_type} · ${continueDoc.page_count} trang · ${continueDoc.status}`
                                : "Chưa có tài liệu trong SQLite"}
                        </div>
                    </div>
                    <span className="btn btn-sm btn-primary">{continueDoc ? "Mở tài liệu" : "Bắt đầu"}</span>
                </Link>

                <div className="card dashboard-signal-card">
                    <div className="stat-label">Tín hiệu hệ thống</div>
                    <div className="dashboard-signal-list">
                        <span className={reviewDocs.length ? "badge badge-warning" : "badge badge-success"}>
                            {reviewDocs.length} cần review
                        </span>
                        <span className={pendingOcrDocs.length ? "badge badge-warning" : "badge badge-success"}>
                            {pendingOcrDocs.length} chưa OCR
                        </span>
                        <span className={ocrErrorDocs.length ? "badge badge-danger" : "badge badge-success"}>
                            {ocrErrorDocs.length} OCR lỗi
                        </span>
                        <span className={unclassifiedDocs.length ? "badge badge-warning" : "badge badge-success"}>
                            {unclassifiedDocs.length} chưa phân loại
                        </span>
                        <span className={catalogBacklog ? "badge badge-info" : "badge badge-success"}>
                            {catalogBacklog} catalog chưa import
                        </span>
                        <span className={importInProgress ? "badge badge-info" : "badge badge-neutral"}>
                            {importInProgress ? "Đang chạy job" : "Không có job đang chạy"}
                        </span>
                        <span className={aiStatus?.available ? "badge badge-success" : "badge badge-neutral"}>
                            AI {aiStatus?.available ? "local LLM" : "extractive"}
                        </span>
                    </div>
                    <div className="text-xs text-muted truncate">{aiStatus?.message ?? "Đang kiểm tra AI offline..."}</div>
                </div>
            </div>

            <div className="dashboard-stat-grid">
                <StatCard label="Hồ sơ" value={cases.length} accent="primary" hint={`${activeCases} đang xử lý`} linkTo="/cases" />
                <StatCard label="Tài liệu" value={documents.length} accent="info" hint={`${readyDocs} có page rows`} linkTo="/documents" />
                <StatCard label="Chưa OCR" value={pendingOcrDocs.length} accent={pendingOcrDocs.length ? "warning" : "success"} hint="Đang chờ pipeline" linkTo="/analyze" />
                <StatCard label="OCR lỗi" value={ocrErrorDocs.length} accent={ocrErrorDocs.length ? "danger" : "success"} hint="Cần chạy lại" linkTo="/analyze" />
                <StatCard label="Chưa phân loại" value={unclassifiedDocs.length} accent={unclassifiedDocs.length ? "warning" : "success"} hint="khong_xac_dinh" linkTo="/analyze?tab=classify" />
                <StatCard label="Cần kiểm tra" value={reviewDocs.length} accent={reviewDocs.length ? "warning" : "success"} hint="Queue thủ công" linkTo="/reviews" />
                <StatCard label="Tổng trang" value={totalPages} accent="success" hint="Theo cases.total_pages" />
                <StatCard label="Backlog import" value={catalogBacklog} accent={catalogBacklog ? "danger" : "success"} hint={`${importedCatalog}/${catalogTotal} imported`} linkTo="/import" />
            </div>

            <div className="dashboard-main-grid">
                <div className="card dashboard-priority-panel">
                    <div className="dashboard-card-header">
                        <div>
                            <div className="card-title">Việc cần xử lý</div>
                            <div className="card-subtitle">Tự gom tài liệu chưa reviewed để giảm thao tác tìm kiếm.</div>
                        </div>
                        <Link className="btn btn-sm btn-ghost" to="/reviews">Mở queue</Link>
                    </div>
                    {urgentDocs.length === 0 ? (
                        <div className="empty-state-text">Không có tài liệu cần review. Có thể import hoặc tìm kiếm hồ sơ.</div>
                    ) : (
                        <div className="dashboard-worklist">
                            {urgentDocs.map((doc) => (
                                <Link
                                    key={doc.document_id}
                                    className="dashboard-workitem card-interactive"
                                    to={`/cases/${doc.case_id}/docs/${doc.document_id}`}
                                >
                                    <div className="dashboard-workitem-main">
                                        <div className="dashboard-workitem-title truncate">{doc.display_name}</div>
                                        <div className="dashboard-workitem-meta">
                                            <span>{doc.document_type}</span>
                                            <span>{doc.page_count} trang</span>
                                            <span>{formatDate(doc.created_at)}</span>
                                        </div>
                                    </div>
                                    <span className={statusBadge(doc.status)}>{doc.status}</span>
                                </Link>
                            ))}
                        </div>
                    )}
                </div>

                <div className="card">
                    <div className="dashboard-card-header">
                        <div>
                            <div className="card-title">Hành động nhanh</div>
                            <div className="card-subtitle">Các luồng hay dùng được đưa về một chỗ.</div>
                        </div>
                    </div>
                    <div className="dashboard-quick-grid">
                        <Link className="dashboard-quick-action" to="/import">
                            <span>⇩</span>
                            <div>
                                <b>Import</b>
                                <small>Nạp hồ sơ</small>
                            </div>
                        </Link>
                        <Link className="dashboard-quick-action" to="/cases">
                            <span>◰</span>
                            <div>
                                <b>Hồ sơ</b>
                                <small>Mở case</small>
                            </div>
                        </Link>
                        <Link className="dashboard-quick-action" to="/search">
                            <span>⌕</span>
                            <div>
                                <b>Tìm kiếm</b>
                                <small>FTS SQLite</small>
                            </div>
                        </Link>
                        <Link className="dashboard-quick-action" to="/reviews">
                            <span>☑</span>
                            <div>
                                <b>Review</b>
                                <small>Duyệt lỗi</small>
                            </div>
                        </Link>
                    </div>

                    <div className="dashboard-mini-section">
                        <div className="stat-label">Loại tài liệu</div>
                        <div className="dashboard-chip-cloud">
                            {topGroups.length === 0 ? (
                                <span className="text-xs text-muted">Chưa có nhóm tài liệu.</span>
                            ) : (
                                topGroups.map((group) => (
                                    <span className="badge badge-neutral" key={group.group_key}>
                                        {group.group_key}: {group.count}
                                    </span>
                                ))
                            )}
                        </div>
                    </div>
                </div>
            </div>

            <div className="dashboard-bottom-grid">
                <div className="card">
                    <div className="dashboard-card-header">
                        <div>
                            <div className="card-title">Hồ sơ gần đây</div>
                            <div className="card-subtitle">Mở thẳng vào case, không cần qua nhiều menu.</div>
                        </div>
                        <Link className="btn btn-sm btn-ghost" to="/cases">Tất cả</Link>
                    </div>
                    <table className="data-table dashboard-compact-table">
                        <thead>
                            <tr>
                                <th>Mã</th>
                                <th>Hồ sơ</th>
                                <th>TL</th>
                                <th>Trang</th>
                                <th>Trạng thái</th>
                            </tr>
                        </thead>
                        <tbody>
                            {sortedCases.slice(0, 5).map((item) => (
                                <tr key={item.case_id}>
                                    <td className="text-mono">{item.case_code}</td>
                                    <td>
                                        <Link className="dashboard-table-link" to={`/cases/${item.case_id}`}>
                                            {item.case_display_name}
                                        </Link>
                                    </td>
                                    <td>{item.document_count}</td>
                                    <td>{item.total_pages}</td>
                                    <td><span className={statusBadge(item.status)}>{item.status}</span></td>
                                </tr>
                            ))}
                            {sortedCases.length === 0 ? (
                                <tr>
                                    <td colSpan={5}>Chưa có hồ sơ.</td>
                                </tr>
                            ) : null}
                        </tbody>
                    </table>
                </div>

                <div className="card">
                    <div className="dashboard-card-header">
                        <div>
                            <div className="card-title">Tài liệu gần đây</div>
                            <div className="card-subtitle">Danh sách đọc nhanh theo thời gian nhập.</div>
                        </div>
                        <Link className="btn btn-sm btn-ghost" to="/documents">Catalog</Link>
                    </div>
                    <div className="dashboard-recent-list">
                        {sortedDocs.slice(0, 6).map((doc) => (
                            <Link key={doc.document_id} className="dashboard-recent-doc" to={`/cases/${doc.case_id}/docs/${doc.document_id}`}>
                                <div className="truncate">
                                    <div className="font-semibold truncate">{doc.display_name}</div>
                                    <div className="text-xs text-muted truncate">{doc.original_filename}</div>
                                </div>
                                <span className={statusBadge(doc.status)}>{doc.status}</span>
                            </Link>
                        ))}
                        {sortedDocs.length === 0 ? <div className="empty-state-text">Chưa có tài liệu.</div> : null}
                    </div>
                </div>
            </div>
        </PageSection>
    );
}
