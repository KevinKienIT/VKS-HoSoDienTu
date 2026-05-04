import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import * as reviewService from "../duyethotro/duyethotro.service";
import { PageSection, TypeIcon } from "./common";

type ConfidenceBand = "all" | "low" | "medium" | "high";

function statusBadge(status: string) {
    if (status === "rejected") return <span className="badge badge-danger">Từ chối</span>;
    if (status === "approved") return <span className="badge badge-success">Đã duyệt</span>;
    if (status === "in_review") return <span className="badge badge-info">Đang duyệt</span>;
    if (status === "deferred") return <span className="badge badge-neutral">Bỏ qua</span>;
    return <span className="badge badge-warning">Chờ xử lý</span>;
}

export function ReviewQueuePage() {
    const navigate = useNavigate();
    const [items, setItems] = useState<reviewService.ReviewItem[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [docTypeFilter, setDocTypeFilter] = useState("all");
    const [confidenceFilter, setConfidenceFilter] = useState<ConfidenceBand>("all");

    const loadQueue = (statusFilter?: string) => {
        setLoading(true);
        setError(null);
        reviewService
            .listReviewQueue(statusFilter)
            .then((rows) => setItems(rows))
            .catch((e) => setError((e as Error).message || "Lỗi tải review queue"))
            .finally(() => setLoading(false));
    };

    useEffect(() => {
        loadQueue();
    }, []);

    const runAction = async (item: reviewService.ReviewItem, action: "approved" | "rejected" | "skipped") => {
        setError(null);
        const ok = await reviewService.reviewAction(item.review_id, action, `Review action: ${action}`);
        if (!ok) {
            setError("Không thể cập nhật trạng thái review qua Tauri/SQLite.");
            return;
        }
        loadQueue();
    };

    const docTypes = useMemo(() => {
        return Array.from(new Set(items.map((i) => i.document_type))).sort((a, b) => a.localeCompare(b, "vi"));
    }, [items]);

    const filtered = useMemo(() => {
        return items.filter((i) => {
            if (docTypeFilter !== "all" && i.document_type !== docTypeFilter) {
                return false;
            }
            if (confidenceFilter === "all") {
                return true;
            }
            if (confidenceFilter === "low") {
                return i.confidence < 0.6;
            }
            if (confidenceFilter === "medium") {
                return i.confidence >= 0.6 && i.confidence < 0.8;
            }
            return i.confidence >= 0.8;
        });
    }, [items, docTypeFilter, confidenceFilter]);

    return (
        <PageSection title="Review Queue" description={`${filtered.length} tài liệu cần rà soát`}>
            {error ? <div className="module-error mb-3">⚠️ {error}</div> : null}

            <div className="toolbar">
                <select
                    className="form-select"
                    value={docTypeFilter}
                    onChange={(e) => setDocTypeFilter(e.target.value)}
                    style={{ width: 200 }}
                >
                    <option value="all">Tất cả loại tài liệu</option>
                    {docTypes.map((t) => (
                        <option key={t} value={t}>{t}</option>
                    ))}
                </select>
                <select
                    className="form-select"
                    value={confidenceFilter}
                    onChange={(e) => setConfidenceFilter(e.target.value as ConfidenceBand)}
                    style={{ width: 180 }}
                >
                    <option value="all">Tất cả confidence</option>
                    <option value="low">Confidence thấp</option>
                    <option value="medium">Confidence trung bình</option>
                    <option value="high">Confidence cao</option>
                </select>
                <div className="toolbar-spacer" />
                <span className="text-muted text-xs">{filtered.length}/{items.length} items</span>
            </div>

            <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                <table className="data-table">
                    <thead>
                        <tr>
                            <th style={{ width: 40 }}>Loại</th>
                            <th>Tài liệu</th>
                            <th style={{ width: 80 }}>Trang</th>
                            <th style={{ width: 100 }}>Trạng thái</th>
                            <th style={{ width: 120 }}>Confidence</th>
                            <th style={{ width: 280 }}>Hành động</th>
                        </tr>
                    </thead>
                    <tbody>
                        {loading ? (
                            <tr>
                                <td colSpan={6} style={{ textAlign: "center", color: "var(--color-text-muted)" }}>Đang tải...</td>
                            </tr>
                        ) : filtered.length === 0 ? (
                            <tr>
                                <td colSpan={6} style={{ textAlign: "center", color: "var(--color-text-muted)", padding: "var(--space-6)" }}>
                                    ☑ Không có item cần review.
                                </td>
                            </tr>
                        ) : (
                            filtered.map((item) => (
                                <tr key={item.document_id}>
                                    <td><TypeIcon docType={item.document_type} /></td>
                                    <td>
                                        <div className="font-semibold">{item.display_name}</div>
                                        <div className="text-muted text-xs">{item.document_type} · {item.review_type}</div>
                                    </td>
                                    <td style={{ textAlign: "center" }}>{item.page_id ? "1" : "-"}</td>
                                    <td>{statusBadge(item.status)}</td>
                                    <td>
                                        <span className={`badge ${item.confidence >= 0.8 ? "badge-success" : item.confidence >= 0.6 ? "badge-warning" : "badge-danger"}`}>
                                            {(item.confidence * 100).toFixed(0)}%
                                        </span>
                                    </td>
                                    <td>
                                        <div style={{ display: "flex", gap: "var(--space-1)" }}>
                                            <button
                                                className="btn btn-sm"
                                                onClick={() => navigate(`/cases/${item.case_id}/docs/${item.document_id}`)}
                                            >
                                                Mở
                                            </button>
                                            <button
                                                className="btn btn-sm btn-success"
                                                onClick={() => runAction(item, "approved")}
                                            >
                                                Duyệt
                                            </button>
                                            <button
                                                className="btn btn-sm btn-ghost"
                                                onClick={() => navigate(`/cases/${item.case_id}/docs/${item.document_id}`)}
                                            >
                                                Sửa
                                            </button>
                                            <button
                                                className="btn btn-sm btn-danger"
                                                onClick={() => runAction(item, "rejected")}
                                            >
                                                Từ chối
                                            </button>
                                            <button
                                                className="btn btn-sm btn-ghost"
                                                onClick={() => runAction(item, "skipped")}
                                            >
                                                Bỏ qua
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>
        </PageSection>
    );
}
