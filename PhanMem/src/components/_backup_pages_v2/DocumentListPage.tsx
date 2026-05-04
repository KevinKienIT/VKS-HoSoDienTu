import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import type { CatalogEntry } from "../../lib/catalog";
import * as caseService from "../../services/caseService";
import * as catalogService from "../../services/catalogService";
import * as documentService from "../../services/documentService";
import { useCatalogStore } from "../../store/catalogStore";
import { DocumentViewer } from "../DocumentViewer";
import { PdfPageThumbnail } from "../PdfPageThumbnail";
import { PageSection, TypeIcon } from "./common";

type ViewMode = "grid" | "list" | "tree";

function statusClass(status: string) {
    if (status === "reviewed" || status === "imported") return "badge badge-success";
    if (status === "processed" || status === "cataloged") return "badge badge-info";
    if (status === "error" || status === "failed") return "badge badge-danger";
    if (status === "skipped") return "badge badge-neutral";
    return "badge badge-warning";
}

function statusLabel(status: string) {
    const labels: Record<string, string> = {
        pending: "Chưa OCR",
        processed: "Đã OCR",
        reviewed: "Đã duyệt",
        error: "OCR lỗi",
        imported: "Đã nhập",
        cataloged: "Catalog",
        skipped: "Bỏ qua",
        discovered: "Phát hiện",
    };
    return labels[status] ?? status;
}

function formatDate(value: string) {
    if (!value) return "-";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value.slice(0, 10);
    return new Intl.DateTimeFormat("vi-VN", { day: "2-digit", month: "2-digit", year: "numeric" }).format(date);
}

export function DocumentListPage() {
    const navigate = useNavigate();
    const entries = useCatalogStore((s) => s.entries);
    const stats = useCatalogStore((s) => s.stats);
    const loading = useCatalogStore((s) => s.loading);
    const scanning = useCatalogStore((s) => s.scanning);
    const error = useCatalogStore((s) => s.error);
    const fetchEntries = useCatalogStore((s) => s.fetchEntries);
    const fetchStats = useCatalogStore((s) => s.fetchStats);
    const scanFolder = useCatalogStore((s) => s.scanFolder);
    const clearError = useCatalogStore((s) => s.clearError);
    const [keyword, setKeyword] = useState("");
    const [viewMode, setViewMode] = useState<ViewMode>("grid");
    const [statusFilter, setStatusFilter] = useState("all");
    const [documents, setDocuments] = useState<documentService.DocumentSummary[]>([]);
    const [selectedDoc, setSelectedDoc] = useState<documentService.DocumentSummary | null>(null);
    const [previewDoc, setPreviewDoc] = useState<documentService.DocumentSummary | null>(null);
    const [groups, setGroups] = useState<documentService.DocumentGroup[]>([]);
    const [indexResult, setIndexResult] = useState<documentService.IndexRebuildResult | null>(null);
    const [cases, setCases] = useState<caseService.CaseSummary[]>([]);
    const [notice, setNotice] = useState<string | null>(null);
    const [renameState, setRenameState] = useState<{
        doc: documentService.DocumentSummary;
        suggestion: documentService.FilenameSuggestion;
        draft: string;
    } | null>(null);
    const [moveState, setMoveState] = useState<{
        doc: documentService.DocumentSummary;
        targetCaseId: string;
    } | null>(null);

    const reload = useCallback(async () => {
        await Promise.all([fetchEntries(), fetchStats()]);
        const [docs, groupItems, caseItems] = await Promise.all([
            documentService.listDocuments(),
            documentService.getDocumentGroups(),
            caseService.listCases(),
        ]);
        setDocuments(docs);
        setGroups(groupItems);
        setCases(caseItems);
    }, [fetchEntries, fetchStats]);

    useEffect(() => {
        reload();
    }, [reload]);

    useEffect(() => {
        const onKeyDown = (event: KeyboardEvent) => {
            if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) return;
            if (event.code === "Space" && selectedDoc) {
                event.preventDefault();
                setPreviewDoc(selectedDoc);
            }
            if (event.key === "Escape") {
                setPreviewDoc(null);
            }
        };
        window.addEventListener("keydown", onKeyDown);
        return () => window.removeEventListener("keydown", onKeyDown);
    }, [selectedDoc]);

    const filteredDocuments = useMemo(() => {
        const q = keyword.trim().toLowerCase();
        return documents.filter((doc) => {
            const matchesKeyword =
                !q ||
                doc.display_name.toLowerCase().includes(q) ||
                doc.original_filename.toLowerCase().includes(q) ||
                doc.document_type.toLowerCase().includes(q) ||
                doc.case_id.toLowerCase().includes(q);
            const matchesStatus = statusFilter === "all" || doc.status === statusFilter;
            return matchesKeyword && matchesStatus;
        });
    }, [documents, keyword, statusFilter]);

    const documentsByCase = useMemo(() => {
        const map = new Map<string, documentService.DocumentSummary[]>();
        for (const doc of filteredDocuments) {
            const bucket = map.get(doc.case_id) ?? [];
            bucket.push(doc);
            map.set(doc.case_id, bucket);
        }
        return Array.from(map.entries()).sort((a, b) => a[0].localeCompare(b[0]));
    }, [filteredDocuments]);

    const catalogIssues = useMemo(() => {
        const q = keyword.trim().toLowerCase();
        return entries
            .filter((item) => item.scan_status !== "imported")
            .filter((item) => !q || item.file_name.toLowerCase().includes(q) || item.parent_folder.toLowerCase().includes(q))
            .slice(0, 12);
    }, [entries, keyword]);

    const handleScanFolder = useCallback(async () => {
        const folderPath = window.prompt("Nhập đường dẫn thư mục cần quét catalog:", "d:\\JOBS\\VKS-HoSoDienTu\\TaiLieu");
        if (!folderPath) return;
        const count = await scanFolder(folderPath);
        await reload();
        setNotice(`Đã xử lý ${count} bản ghi catalog.`);
    }, [reload, scanFolder]);

    const setCatalogStatus = useCallback(
        async (entry: CatalogEntry, status: CatalogEntry["scan_status"]) => {
            try {
                await catalogService.updateEntryStatus(entry.catalog_entry_id, status);
                await reload();
            } catch (e) {
                setNotice(`Cập nhật trạng thái thất bại: ${(e as Error).message || "Lỗi không xác định"}`);
            }
        },
        [reload]
    );

    const handleBulkEnrich = useCallback(async () => {
        const changed = await documentService.bulkEnrichDocuments();
        await reload();
        setNotice(`Đã cập nhật metadata cho ${changed} tài liệu.`);
    }, [reload]);

    const handleRebuildIndex = useCallback(async () => {
        const result = await documentService.rebuildTextIndex();
        setIndexResult(result);
        if (result) {
            setNotice(`Đã rebuild chỉ mục: docs=${result.indexed_documents}, pages=${result.indexed_pages}, entities=${result.indexed_entities}`);
        }
    }, []);

    const handleRenameSuggestion = useCallback(
        async (doc: documentService.DocumentSummary) => {
            const suggestion = await documentService.suggestDocumentFilename(doc.document_id);
            if (!suggestion) return;
            setRenameState({ doc, suggestion, draft: suggestion.suggested_filename });
        },
        []
    );

    const confirmRename = useCallback(async () => {
        if (!renameState) return;
        const next = renameState.draft.trim();
        if (!next || next === renameState.suggestion.current_filename) {
            setRenameState(null);
            return;
        }
        const renamed = await documentService.renameDocumentFile(renameState.doc.document_id, next);
        if (renamed) {
            setNotice(`Đã đổi tên: ${renamed.original_filename}`);
            setRenameState(null);
            await reload();
        } else {
            setNotice("Không thể đổi tên. Kiểm tra tên trùng hoặc quyền ghi file.");
        }
    }, [reload, renameState]);

    const confirmMove = useCallback(async () => {
        if (!moveState?.targetCaseId) return;
        const moved = await documentService.moveDocumentToCase(moveState.doc.document_id, moveState.targetCaseId);
        if (moved) {
            setNotice(`Đã di chuyển tài liệu sang hồ sơ mới.`);
            setMoveState(null);
            await reload();
        } else {
            setNotice("Không thể di chuyển tài liệu.");
        }
    }, [moveState, reload]);

    const openDoc = (doc: documentService.DocumentSummary) => {
        navigate(`/cases/${doc.case_id}/docs/${doc.document_id}`);
    };

    return (
        <PageSection
            title="Quản lý tài liệu"
            description="Workspace hồ sơ tài liệu: thẻ tài liệu, danh sách compact, cây theo hồ sơ và catalog chưa nhập"
            right={
                <button className="btn btn-primary" onClick={handleScanFolder} disabled={scanning}>
                    {scanning ? "Đang quét..." : "Quét thư mục"}
                </button>
            }
        >
            <div className="document-command-strip mb-4">
                <div className="document-command-card">
                    <div className="stat-label">Tài liệu trong SQLite</div>
                    <div className="stat-value">{documents.length}</div>
                    <div className="text-xs text-muted">Đã import vào hồ sơ</div>
                </div>
                <div className="document-command-card">
                    <div className="stat-label">Catalog file</div>
                    <div className="stat-value">{stats?.total_files ?? 0}</div>
                    <div className="text-xs text-muted">{stats?.by_status?.imported ?? 0} đã đánh dấu imported</div>
                </div>
                <div className="document-command-card">
                    <div className="stat-label">Chưa OCR / lỗi</div>
                    <div className="stat-value">
                        {documents.filter((d) => d.status === "pending" || d.status === "error").length}
                    </div>
                    <div className="text-xs text-muted">Cần xử lý hoặc kiểm tra thủ công</div>
                </div>
                <div className="document-command-card">
                    <div className="stat-label">Chưa phân loại</div>
                    <div className="stat-value">{documents.filter((d) => d.document_type === "khong_xac_dinh").length}</div>
                    <div className="text-xs text-muted">Cần gợi ý tên/loại tài liệu</div>
                </div>
            </div>

            <div className="toolbar document-toolbar">
                <input
                    type="text"
                    className="form-input"
                    value={keyword}
                    onChange={(e) => setKeyword(e.target.value)}
                    placeholder="Lọc theo tên tài liệu, hồ sơ, loại văn bản..."
                />
                <select className="form-select" value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}>
                    <option value="all">Tất cả trạng thái</option>
                    <option value="pending">Chưa OCR</option>
                    <option value="processed">Đã OCR</option>
                    <option value="reviewed">Đã duyệt</option>
                    <option value="error">OCR lỗi</option>
                </select>
                <div className="doc-view-toggle" aria-label="Chế độ xem">
                    <button className={viewMode === "grid" ? "active" : ""} onClick={() => setViewMode("grid")}>Grid</button>
                    <button className={viewMode === "list" ? "active" : ""} onClick={() => setViewMode("list")}>List</button>
                    <button className={viewMode === "tree" ? "active" : ""} onClick={() => setViewMode("tree")}>Tree</button>
                </div>
                <button className="btn" onClick={handleBulkEnrich}>Gợi ý tên</button>
                <button className="btn" onClick={handleRebuildIndex}>Rebuild FTS</button>
            </div>

            {indexResult ? (
                <div className="card mb-4">
                    <span className="badge badge-info">docs={indexResult.indexed_documents}</span>{" "}
                    <span className="badge badge-info">pages={indexResult.indexed_pages}</span>{" "}
                    <span className="badge badge-info">entities={indexResult.indexed_entities}</span>
                </div>
            ) : null}

            {error ? (
                <div className="module-error mb-4">
                    <div className="mb-2">{error}</div>
                    <button className="btn btn-sm" onClick={clearError}>Đóng lỗi</button>
                </div>
            ) : null}
            {notice ? <div className="toast-inline mb-4">{notice}</div> : null}

            <div className="card mb-4">
                <div className="dashboard-card-header">
                    <div>
                        <div className="card-title">Nhóm loại tài liệu</div>
                        <div className="card-subtitle">Dùng để kiểm soát phân loại và đặt tên theo quy tắc hồ sơ.</div>
                    </div>
                </div>
                <div className="dashboard-chip-cloud">
                    {groups.length === 0 ? (
                        <span className="text-xs text-muted">Chưa có dữ liệu nhóm.</span>
                    ) : (
                        groups.map((g) => (
                            <span className="badge badge-neutral" key={g.group_key}>
                                {g.group_key}: {g.count}
                            </span>
                        ))
                    )}
                </div>
            </div>

            {loading ? <div className="card mb-4">Đang tải catalog...</div> : null}

            {viewMode === "grid" ? (
                <div className="doc-card-grid">
                    {filteredDocuments.map((doc, index) => (
                        <article
                            key={doc.document_id}
                            className={`doc-card ${selectedDoc?.document_id === doc.document_id ? "selected" : ""}`}
                            onMouseEnter={() => setSelectedDoc(doc)}
                            tabIndex={0}
                            onFocus={() => setSelectedDoc(doc)}
                            onClick={() => openDoc(doc)}
                        >
                            <div className="doc-card-preview">
                                <PdfPageThumbnail
                                    filePath={doc.file_path}
                                    fileType={doc.file_path.split(".").pop() || "pdf"}
                                    pageNumber={1}
                                    width={110}
                                    label="Trang đầu"
                                />
                            </div>
                            <div className="doc-card-body">
                                <div className="doc-card-title" title={doc.display_name}>{doc.display_name}</div>
                                <div className="doc-card-meta">
                                    <span className="text-mono">{String(index + 1).padStart(3, "0")}</span>
                                    <span>{doc.page_count} trang</span>
                                    <span>{formatDate(doc.created_at)}</span>
                                </div>
                                <div className="doc-card-tags">
                                    <span className="badge badge-neutral">{doc.document_type}</span>
                                    <span className={statusClass(doc.status)}>{statusLabel(doc.status)}</span>
                                    {doc.file_missing ? <span className="badge badge-danger">Mất file</span> : null}
                                    <span className="badge badge-primary">HS: {doc.case_id.slice(0, 10)}</span>
                                </div>
                                <div className="doc-card-actions" onClick={(e) => e.stopPropagation()}>
                                    <button className="btn btn-sm btn-primary" onClick={() => openDoc(doc)}>Mở</button>
                                    <button className="btn btn-sm" onClick={() => handleRenameSuggestion(doc)}>Đổi tên</button>
                                    <button className="btn btn-sm" onClick={() => setMoveState({ doc, targetCaseId: doc.case_id })}>Chuyển</button>
                                    <button className="btn btn-sm" onClick={() => navigate("/export")}>Xuất PDF</button>
                                </div>
                            </div>
                        </article>
                    ))}
                </div>
            ) : null}

            {viewMode === "list" ? (
                <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                    <table className="data-table">
                        <thead>
                            <tr>
                                <th>Tài liệu</th>
                                <th style={{ width: 150 }}>Loại</th>
                                <th style={{ width: 120 }}>Hồ sơ</th>
                                <th style={{ width: 80 }}>Trang</th>
                                <th style={{ width: 120 }}>OCR</th>
                                <th style={{ width: 120 }}>Ngày nhập</th>
                                <th style={{ width: 190 }}>Thao tác</th>
                            </tr>
                        </thead>
                        <tbody>
                            {filteredDocuments.map((doc) => (
                                <tr
                                    key={doc.document_id}
                                    className={selectedDoc?.document_id === doc.document_id ? "selected" : ""}
                                    style={{ background: doc.file_missing ? "var(--color-danger-surface)" : undefined }}
                                    onMouseEnter={() => setSelectedDoc(doc)}
                                    onFocus={() => setSelectedDoc(doc)}
                                    tabIndex={0}
                                >
                                    <td>
                                        <div className="font-semibold truncate">{doc.display_name}</div>
                                        <div className="text-xs text-muted truncate">{doc.original_filename}</div>
                                    </td>
                                    <td><span className="badge badge-neutral">{doc.document_type}</span></td>
                                    <td className="text-mono text-xs">{doc.case_id.slice(0, 12)}</td>
                                    <td>{doc.page_count}</td>
                                    <td>
                                        <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                                            <span className={statusClass(doc.status)}>{statusLabel(doc.status)}</span>
                                            {doc.file_missing ? <span className="badge badge-danger">Mất file</span> : null}
                                        </div>
                                    </td>
                                    <td>{formatDate(doc.created_at)}</td>
                                    <td>
                                        <div className="flex gap-2">
                                            <button className="btn btn-sm btn-primary" onClick={() => openDoc(doc)}>Mở</button>
                                            <button className="btn btn-sm" onClick={() => handleRenameSuggestion(doc)}>Đổi tên</button>
                                            <button className="btn btn-sm" onClick={() => setMoveState({ doc, targetCaseId: doc.case_id })}>Chuyển</button>
                                            <button className="btn btn-sm" onClick={() => navigate("/export")}>PDF</button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            ) : null}

            {viewMode === "tree" ? (
                <div className="document-tree">
                    {documentsByCase.map(([caseId, docs]) => (
                        <div className="card document-tree-case" key={caseId}>
                            <div className="document-tree-case-header">
                                <div>
                                    <div className="font-semibold">Hồ sơ {caseId}</div>
                                    <div className="text-xs text-muted">{docs.length} tài liệu</div>
                                </div>
                                <button className="btn btn-sm" onClick={() => navigate(`/cases/${caseId}`)}>Mở hồ sơ</button>
                            </div>
                            <div className="document-tree-list">
                                {docs.map((doc) => (
                                    <button
                                        className="document-tree-item"
                                        key={doc.document_id}
                                        onMouseEnter={() => setSelectedDoc(doc)}
                                        onFocus={() => setSelectedDoc(doc)}
                                        onClick={() => openDoc(doc)}
                                    >
                                        <TypeIcon docType={doc.document_type} />
                                        <span className="truncate">{doc.display_name}</span>
                                        <span className={statusClass(doc.status)}>{statusLabel(doc.status)}</span>
                                        {doc.file_missing ? <span className="badge badge-danger">Mất file</span> : null}
                                    </button>
                                ))}
                            </div>
                        </div>
                    ))}
                </div>
            ) : null}

            {filteredDocuments.length === 0 ? (
                <div className="card mt-4">
                    <div className="empty-state-text">Không có tài liệu khớp bộ lọc.</div>
                </div>
            ) : null}

            {catalogIssues.length > 0 ? (
                <div className="card mt-4" style={{ padding: 0, overflow: "hidden" }}>
                    <div className="document-catalog-header">
                        <div>
                            <div className="card-title">File catalog cần xử lý</div>
                            <div className="card-subtitle">Các file đã phát hiện nhưng chưa được đánh dấu imported.</div>
                        </div>
                    </div>
                    <table className="data-table">
                        <thead>
                            <tr>
                                <th>Tên file</th>
                                <th>Thư mục</th>
                                <th style={{ width: 80 }}>Đuôi</th>
                                <th style={{ width: 120 }}>Trạng thái</th>
                                <th style={{ width: 190 }}>Đánh dấu</th>
                            </tr>
                        </thead>
                        <tbody>
                            {catalogIssues.map((entry) => (
                                <tr key={entry.catalog_entry_id}>
                                    <td className="font-semibold">{entry.file_name}</td>
                                    <td className="text-xs text-muted">{entry.parent_folder}</td>
                                    <td className="text-mono text-xs">{entry.file_ext}</td>
                                    <td><span className={statusClass(entry.scan_status)}>{statusLabel(entry.scan_status)}</span></td>
                                    <td>
                                        <div className="flex gap-2">
                                            <button className="btn btn-sm" onClick={() => setCatalogStatus(entry, "cataloged")}>Catalog</button>
                                            <button className="btn btn-sm btn-success" onClick={() => setCatalogStatus(entry, "imported")}>Imported</button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            ) : null}

            {previewDoc ? (
                <div className="quick-preview-backdrop" onClick={() => setPreviewDoc(null)}>
                    <div className="quick-preview-modal" onClick={(e) => e.stopPropagation()}>
                        <div className="quick-preview-header">
                            <div>
                                <div className="font-semibold truncate">{previewDoc.display_name}</div>
                                <div className="text-xs text-muted">Space preview · ESC đóng · Enter/mở tài liệu bằng nút bên phải</div>
                            </div>
                            <div className="flex gap-2">
                                <button className="btn btn-sm btn-primary" onClick={() => openDoc(previewDoc)}>Mở tài liệu</button>
                                <button className="btn btn-sm" onClick={() => setPreviewDoc(null)}>Đóng</button>
                            </div>
                        </div>
                        <div className="quick-preview-body">
                            <DocumentViewer
                                documentPath={previewDoc.file_path}
                                documentType={previewDoc.file_path.split(".").pop() || "unknown"}
                            />
                        </div>
                    </div>
                </div>
            ) : null}

            {renameState ? (
                <div className="quick-preview-backdrop" onClick={() => setRenameState(null)}>
                    <div className="workflow-modal" onClick={(e) => e.stopPropagation()}>
                        <div className="quick-preview-header">
                            <div>
                                <div className="font-semibold">Đổi tên theo quy tắc hồ sơ</div>
                                <div className="text-xs text-muted">Không ghi đè file gốc nếu chưa xác nhận.</div>
                            </div>
                            <button className="btn btn-sm" onClick={() => setRenameState(null)}>Đóng</button>
                        </div>
                        <div className="workflow-modal-body">
                            <label className="form-label">Tên đề xuất</label>
                            <input
                                className="form-input"
                                value={renameState.draft}
                                onChange={(e) => setRenameState({ ...renameState, draft: e.target.value })}
                            />
                            {renameState.suggestion.missing_fields.length > 0 ? (
                                <div className="rename-missing-fields">
                                    {renameState.suggestion.missing_fields.map((field) => (
                                        <span className="badge badge-warning" key={field}>Thiếu {field}</span>
                                    ))}
                                </div>
                            ) : (
                                <div className="badge badge-success mt-3">Đủ metadata đặt tên</div>
                            )}
                            <div className="workflow-modal-actions">
                                <button className="btn" onClick={() => setRenameState(null)}>Bỏ qua</button>
                                <button className="btn btn-primary" onClick={confirmRename}>Xác nhận đổi tên</button>
                            </div>
                        </div>
                    </div>
                </div>
            ) : null}

            {moveState ? (
                <div className="quick-preview-backdrop" onClick={() => setMoveState(null)}>
                    <div className="workflow-modal" onClick={(e) => e.stopPropagation()}>
                        <div className="quick-preview-header">
                            <div>
                                <div className="font-semibold">Di chuyển tài liệu</div>
                                <div className="text-xs text-muted">{moveState.doc.display_name}</div>
                            </div>
                            <button className="btn btn-sm" onClick={() => setMoveState(null)}>Đóng</button>
                        </div>
                        <div className="workflow-modal-body">
                            <label className="form-label">Hồ sơ đích</label>
                            <select
                                className="form-select"
                                value={moveState.targetCaseId}
                                onChange={(e) => setMoveState({ ...moveState, targetCaseId: e.target.value })}
                            >
                                {cases.map((item) => (
                                    <option key={item.case_id} value={item.case_id}>
                                        {item.case_code} - {item.case_display_name}
                                    </option>
                                ))}
                            </select>
                            <div className="workflow-modal-actions">
                                <button className="btn" onClick={() => setMoveState(null)}>Hủy</button>
                                <button className="btn btn-primary" onClick={confirmMove}>Di chuyển</button>
                            </div>
                        </div>
                    </div>
                </div>
            ) : null}
        </PageSection>
    );
}
