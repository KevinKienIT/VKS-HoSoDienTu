import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import * as documentService from "../../services/documentService";
import * as reviewService from "../../services/reviewService";
import { PageSection, TypeIcon } from "./common";

type AnalyzeTab = "ocr" | "classify" | "review";

const TABS: { key: AnalyzeTab; label: string; icon: string }[] = [
    { key: "ocr", label: "OCR & Nhận diện", icon: "🔍" },
    { key: "classify", label: "Phân loại & Đặt tên", icon: "🏷️" },
    { key: "review", label: "Review Queue", icon: "☑️" },
];

function statusBadge(status: string) {
    const s = status.toLowerCase();
    if (s === "reviewed" || s === "approved") return "badge badge-success";
    if (s === "error" || s === "rejected" || s === "failed") return "badge badge-danger";
    if (s === "processed" || s === "in_review") return "badge badge-info";
    if (s === "pending" || s === "scanning") return "badge badge-warning";
    return "badge badge-neutral";
}

export function AnalyzePage() {
    const navigate = useNavigate();
    const [params, setParams] = useSearchParams();
    const activeTab = (params.get("tab") as AnalyzeTab) || "ocr";
    const [documents, setDocuments] = useState<documentService.DocumentSummary[]>([]);
    const [reviewItems, setReviewItems] = useState<reviewService.ReviewItem[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [notice, setNotice] = useState<string | null>(null);
    const [ocrBusyId, setOcrBusyId] = useState<string | null>(null);

    const setTab = (tab: AnalyzeTab) => {
        const next = new URLSearchParams(params);
        next.set("tab", tab);
        setParams(next);
    };

    useEffect(() => {
        setLoading(true);
        setError(null);
        Promise.all([
            documentService.listDocuments(),
            reviewService.listReviewQueue(),
        ])
            .then(([docs, reviews]) => {
                setDocuments(docs);
                setReviewItems(reviews);
            })
            .catch((e) => setError((e as Error).message))
            .finally(() => setLoading(false));
    }, []);

    const pendingOcr = useMemo(() => documents.filter((d) => d.status === "pending"), [documents]);
    const processedDocs = useMemo(() => documents.filter((d) => d.status === "processed"), [documents]);
    const errorDocs = useMemo(() => documents.filter((d) => d.status === "error"), [documents]);

    const handleBulkEnrich = useCallback(async () => {
        const changed = await documentService.bulkEnrichDocuments();
        setError(null);
        const docs = await documentService.listDocuments();
        setDocuments(docs);
        setNotice(`Đã phân tích metadata cho ${changed} tài liệu.`);
    }, []);

    const handleRebuildIndex = useCallback(async () => {
        const result = await documentService.rebuildTextIndex();
        if (result) {
            setNotice(`Rebuild xong: ${result.indexed_documents} docs, ${result.indexed_pages} pages, ${result.indexed_entities} entities`);
        }
    }, []);

    const runOcr = useCallback(async (doc: documentService.DocumentSummary) => {
        setOcrBusyId(doc.document_id);
        setError(null);
        setNotice(null);
        try {
            const result = await documentService.runOcrForDocument(doc.document_id);
            if (!result) {
                setError("OCR helper không trả kết quả. Kiểm tra Tauri log.");
                return;
            }
            setNotice(`${result.message} Trang xử lý: ${result.processed_pages}, lỗi: ${result.failed_pages}, nghi viết tay: ${result.handwritten_pages ?? 0}, confidence: ${Math.round(result.average_confidence * 100)}%.`);
            const [docs, reviews] = await Promise.all([
                documentService.listDocuments(),
                reviewService.listReviewQueue(),
            ]);
            setDocuments(docs);
            setReviewItems(reviews);
        } catch (e) {
            setError((e as Error).message || "Không thể chạy OCR.");
        } finally {
            setOcrBusyId(null);
        }
    }, []);

    const runReviewAction = async (item: reviewService.ReviewItem, action: "approved" | "rejected" | "skipped") => {
        setError(null);
        const ok = await reviewService.reviewAction(item.review_id, action, `Review: ${action}`);
        if (!ok) {
            setError("Không thể cập nhật trạng thái review.");
            return;
        }
        const reviews = await reviewService.listReviewQueue();
        setReviewItems(reviews);
    };

    return (
        <PageSection
            title="Phân tích Tài liệu"
            description="OCR nhận diện, phân loại, đặt tên tự động và kiểm duyệt"
            right={
                <div style={{ display: "flex", gap: "var(--space-2)" }}>
                    <button className="btn" onClick={handleBulkEnrich}>
                        🧠 Phân tích hàng loạt
                    </button>
                    <button className="btn" onClick={handleRebuildIndex}>
                        📚 Rebuild FTS
                    </button>
                </div>
            }
        >
            {error ? <div className="module-error mb-3">⚠️ {error}</div> : null}
            {notice ? <div className="toast-inline mb-3">{notice}</div> : null}

            <div className="tab-bar">
                {TABS.map((t) => (
                    <button
                        key={t.key}
                        className={`tab-item ${activeTab === t.key ? "active" : ""}`}
                        onClick={() => setTab(t.key)}
                    >
                        {t.icon} {t.label}
                        {t.key === "review" && reviewItems.length > 0 ? (
                            <span className="badge badge-warning" style={{ marginLeft: 6, fontSize: 10 }}>
                                {reviewItems.length}
                            </span>
                        ) : null}
                    </button>
                ))}
            </div>

            {loading ? <div className="card">Đang tải dữ liệu...</div> : null}

            {/* OCR Tab */}
            {activeTab === "ocr" && !loading ? (
                <div>
                    <div className="analyze-stat-row">
                        <div className="card analyze-stat-card">
                            <div className="stat-label">Chờ OCR</div>
                            <div className="stat-value" style={{ color: pendingOcr.length > 0 ? "var(--color-warning)" : "var(--color-success)" }}>
                                {pendingOcr.length}
                            </div>
                        </div>
                        <div className="card analyze-stat-card">
                            <div className="stat-label">Đã xử lý</div>
                            <div className="stat-value" style={{ color: "var(--color-success)" }}>{processedDocs.length}</div>
                        </div>
                        <div className="card analyze-stat-card">
                            <div className="stat-label">Lỗi OCR</div>
                            <div className="stat-value" style={{ color: errorDocs.length > 0 ? "var(--color-danger)" : "var(--color-success)" }}>
                                {errorDocs.length}
                            </div>
                        </div>
                        <div className="card analyze-stat-card">
                            <div className="stat-label">Tổng</div>
                            <div className="stat-value">{documents.length}</div>
                        </div>
                    </div>

                    {pendingOcr.length > 0 ? (
                        <div className="card">
                            <div className="card-title">Tài liệu chờ OCR</div>
                            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                                {pendingOcr.slice(0, 20).map((doc) => (
                                    <div
                                        key={doc.document_id}
                                        className="analyze-doc-row card-interactive"
                                        onClick={() => navigate(`/cases/${doc.case_id}/docs/${doc.document_id}`)}
                                    >
                                        <TypeIcon docType={doc.document_type} />
                                        <span className="truncate" style={{ flex: 1 }}>{doc.display_name}</span>
                                        <span className="text-muted text-xs">{doc.page_count} trang</span>
                                        <span className={statusBadge(doc.status)}>{doc.status}</span>
                                        <button
                                            className="btn btn-sm"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                runOcr(doc);
                                            }}
                                            disabled={ocrBusyId === doc.document_id}
                                        >
                                            {ocrBusyId === doc.document_id ? "Đang OCR" : "Chạy OCR"}
                                        </button>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ) : (
                        <div className="card">
                            <div className="empty-state-text">Tất cả tài liệu đã được xử lý OCR.</div>
                        </div>
                    )}

                    {errorDocs.length > 0 ? (
                        <div className="card" style={{ marginTop: "var(--space-3)" }}>
                            <div className="card-title" style={{ color: "var(--color-danger)" }}>⚠️ Tài liệu lỗi OCR</div>
                            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                                {errorDocs.map((doc) => (
                                    <div
                                        key={doc.document_id}
                                        className="analyze-doc-row card-interactive"
                                        onClick={() => navigate(`/cases/${doc.case_id}/docs/${doc.document_id}`)}
                                    >
                                        <TypeIcon docType={doc.document_type} />
                                        <span className="truncate" style={{ flex: 1 }}>{doc.display_name}</span>
                                        <span className="badge badge-danger">error</span>
                                        <button
                                            className="btn btn-sm"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                runOcr(doc);
                                            }}
                                            disabled={ocrBusyId === doc.document_id}
                                        >
                                            Chạy lại OCR
                                        </button>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ) : null}
                </div>
            ) : null}

            {/* Classify Tab */}
            {activeTab === "classify" && !loading ? (
                <div className="card">
                    <div className="card-title">Phân loại & Đặt tên tự động</div>
                    <div className="empty-state" style={{ padding: "var(--space-6)" }}>
                        <div className="empty-state-icon">🏷️</div>
                        <div className="empty-state-title">Đặt tên file tự động</div>
                        <div className="empty-state-text" style={{ maxWidth: 500 }}>
                            Hệ thống sẽ phân tích nội dung OCR để gợi ý tên file theo cấu trúc:
                            <br />
                            <code style={{ fontSize: "var(--text-xs)", marginTop: 8, display: "inline-block" }}>
                                [STT]_[Loại]_[Ngày]_[Cơ quan]_[Người]_[Mã HS].pdf
                            </code>
                        </div>
                        <button className="btn btn-primary" style={{ marginTop: "var(--space-4)" }} onClick={handleBulkEnrich}>
                            🧠 Phân tích & gợi ý tên hàng loạt
                        </button>
                    </div>
                </div>
            ) : null}

            {/* Review Tab */}
            {activeTab === "review" && !loading ? (
                <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                    <table className="data-table">
                        <thead>
                            <tr>
                                <th style={{ width: 40 }}>Loại</th>
                                <th>Tài liệu</th>
                                <th style={{ width: 80 }}>Trang</th>
                                <th style={{ width: 100 }}>Trạng thái</th>
                                <th style={{ width: 100 }}>Confidence</th>
                                <th style={{ width: 260 }}>Hành động</th>
                            </tr>
                        </thead>
                        <tbody>
                            {reviewItems.length === 0 ? (
                                <tr>
                                    <td colSpan={6} style={{ textAlign: "center", color: "var(--color-text-muted)", padding: "var(--space-6)" }}>
                                        ☑ Không có item cần review.
                                    </td>
                                </tr>
                            ) : (
                                reviewItems.map((item) => (
                                    <tr key={item.review_id ?? item.document_id}>
                                        <td><TypeIcon docType={item.document_type} /></td>
                                        <td>
                                            <div className="font-semibold">{item.display_name}</div>
                                            <div className="text-muted text-xs">{item.document_type} · {item.review_type}</div>
                                        </td>
                                        <td style={{ textAlign: "center" }}>{item.page_id ? "1" : "-"}</td>
                                        <td><span className={statusBadge(item.status)}>{item.status}</span></td>
                                        <td>
                                            <span className={`badge ${item.confidence >= 0.8 ? "badge-success" : item.confidence >= 0.6 ? "badge-warning" : "badge-danger"}`}>
                                                {(item.confidence * 100).toFixed(0)}%
                                            </span>
                                        </td>
                                        <td>
                                            <div style={{ display: "flex", gap: "var(--space-1)" }}>
                                                <button className="btn btn-sm" onClick={() => navigate(`/cases/${item.case_id}/docs/${item.document_id}`)}>Mở</button>
                                                <button className="btn btn-sm btn-success" onClick={() => runReviewAction(item, "approved")}>Duyệt</button>
                                                <button className="btn btn-sm btn-danger" onClick={() => runReviewAction(item, "rejected")}>Từ chối</button>
                                                <button className="btn btn-sm btn-ghost" onClick={() => runReviewAction(item, "skipped")}>Bỏ qua</button>
                                            </div>
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>
            ) : null}
        </PageSection>
    );
}
