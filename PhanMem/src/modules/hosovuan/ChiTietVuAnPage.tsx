import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
import * as caseService from "../hosovuan/hosovuan.service";
import * as documentService from "../quantailieu/quantailieu.service";
import * as appSettingService from "../cauhinh/cauhinh.service";
import { AiNotebookPanel } from "../phantichai/SoTayAIPanel";
import { PageSection, TypeIcon } from "./common";

type CaseTab = "overview" | "documents" | "timeline" | "citations" | "dossier" | "notes";

function inferTab(value: string | null): CaseTab {
    const v = (value ?? "overview") as CaseTab;
    if (["overview", "documents", "timeline", "citations", "dossier", "notes"].includes(v)) {
        return v;
    }
    return "overview";
}

export function CaseDetailPage() {
    const { caseId = "" } = useParams();
    const navigate = useNavigate();
    const [params, setParams] = useSearchParams();
    const activeTab = inferTab(params.get("tab"));
    const [caseInfo, setCaseInfo] = useState<caseService.CaseSummary | null>(null);
    const [documents, setDocuments] = useState<documentService.DocumentSummary[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [notes, setNotes] = useState("");
    const noteKey = useMemo(() => `case_notes::${caseId}`, [caseId]);

    const load = useCallback(async () => {
        if (!caseId) return;
        setLoading(true);
        setError(null);
        try {
            const [c, docs, note] = await Promise.all([
                caseService.getCase(caseId),
                documentService.listDocuments(caseId),
                appSettingService.getAppSetting(noteKey),
            ]);
            setCaseInfo(c);
            setDocuments(docs);
            setNotes(note ?? "");
        } catch (e) {
            setError((e as Error).message || "Lỗi khi tải dữ liệu hồ sơ.");
        } finally {
            setLoading(false);
        }
    }, [caseId, noteKey]);

    useEffect(() => {
        load();
    }, [load]);

    const setTab = (tab: CaseTab) => {
        const next = new URLSearchParams(params);
        next.set("tab", tab);
        setParams(next);
    };

    const saveNotes = useCallback(async () => {
        const ok = await appSettingService.setAppSetting(noteKey, notes);
        if (!ok) {
            setError("Không thể lưu ghi chú vào SQLite/Tauri command.");
            return;
        }
        window.alert("Đã lưu ghi chú hồ sơ.");
    }, [noteKey, notes]);

    return (
        <PageSection
            title="Chi tiết hồ sơ"
            description={caseInfo ? `${caseInfo.case_code} — ${caseInfo.case_display_name}` : "Đang tải..."}
            right={
                <Link className="btn btn-sm" to="/cases">
                    ← Danh sách
                </Link>
            }
        >
            {error ? <div className="module-error mb-3">⚠️ {error}</div> : null}

            <div className="tab-bar">
                {([
                    ["overview", "Tổng quan"],
                    ["documents", "Tài liệu"],
                    ["timeline", "Timeline"],
                    ["citations", "Trích dẫn"],
                    ["dossier", "Hồ sơ bị can"],
                    ["notes", "Ghi chú & AI"],
                ] as [CaseTab, string][]).map(([tab, label]) => (
                    <button
                        key={tab}
                        className={`tab-item ${activeTab === tab ? "active" : ""}`}
                        onClick={() => setTab(tab)}
                    >
                        {label}
                    </button>
                ))}
            </div>

            {loading ? <div className="card">Đang tải dữ liệu hồ sơ...</div> : null}

            {activeTab === "overview" ? (
                <div className="card">
                    {caseInfo ? (
                        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "var(--space-4)" }}>
                            <div>
                                <div className="stat-label">Trạng thái</div>
                                <div className="font-semibold mt-2">
                                    <span className={`badge ${caseInfo.status === "active" ? "badge-success" : "badge-neutral"}`}>
                                        {caseInfo.status}
                                    </span>
                                </div>
                            </div>
                            <div>
                                <div className="stat-label">Tài liệu</div>
                                <div className="stat-value">{caseInfo.document_count}</div>
                            </div>
                            <div>
                                <div className="stat-label">Tổng trang</div>
                                <div className="stat-value">{caseInfo.total_pages}</div>
                            </div>
                        </div>
                    ) : (
                        <div className="empty-state-text">Không tìm thấy hồ sơ.</div>
                    )}
                </div>
            ) : null}

            {activeTab === "documents" ? (
                <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                    <table className="data-table">
                        <thead>
                            <tr>
                                <th style={{ width: 40 }}>Loại</th>
                                <th>Tên tài liệu</th>
                                <th style={{ width: 60 }}>Trang</th>
                                <th style={{ width: 90 }}>Trạng thái</th>
                                <th style={{ width: 140 }}>Ngày tạo</th>
                            </tr>
                        </thead>
                        <tbody>
                            {documents.length === 0 ? (
                                <tr>
                                    <td colSpan={5} style={{ textAlign: "center", color: "var(--color-text-muted)" }}>
                                        Chưa có tài liệu.
                                    </td>
                                </tr>
                            ) : (
                                documents.map((doc) => (
                                    <tr
                                        key={doc.document_id}
                                        style={{
                                            cursor: "pointer",
                                            background: doc.file_missing ? "var(--color-danger-surface)" : undefined,
                                        }}
                                        onClick={() => navigate(`/cases/${caseId}/docs/${doc.document_id}`)}
                                    >
                                        <td><TypeIcon docType={doc.document_type} /></td>
                                        <td>{doc.display_name}</td>
                                        <td>{doc.page_count}</td>
                                        <td>
                                            <span className={`badge ${doc.status === "reviewed" ? "badge-success" : doc.status === "error" ? "badge-danger" : "badge-neutral"}`}>
                                                {doc.status}
                                            </span>
                                            {doc.file_missing ? <span className="badge badge-danger" style={{ marginLeft: "var(--space-1)" }}>Mất file</span> : null}
                                        </td>
                                        <td className="text-muted text-xs">{doc.created_at}</td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>
            ) : null}

            {activeTab === "timeline" ? (
                <div className="card">
                    <div className="empty-state-text">Timeline board sẽ được mở rộng ở milestone M3/M5.</div>
                </div>
            ) : null}

            {activeTab === "citations" ? (
                <div className="card">
                    <div className="empty-state-text">Citation panel đang dùng fallback read-only.</div>
                </div>
            ) : null}

            {activeTab === "dossier" ? (
                <div className="card">
                    <div className="card-title">Hồ sơ bị can</div>
                    <div style={{ display: "grid", gridTemplateColumns: "100px 1fr", gap: "var(--space-4)" }}>
                        <div style={{ width: 100, height: 100, borderRadius: "var(--radius-lg)", background: "var(--color-bg-muted)", display: "grid", placeItems: "center", fontSize: 36, color: "var(--color-text-muted)" }}>
                            👤
                        </div>
                        <div style={{ display: "grid", gap: "var(--space-2)", fontSize: "var(--text-sm)" }}>
                            <div><span className="text-muted">Họ tên:</span> <strong>{caseInfo?.primary_person_name ?? "(chưa có)"}</strong></div>
                            <div><span className="text-muted">Năm sinh:</span> —</div>
                            <div><span className="text-muted">CMND/CCCD:</span> —</div>
                            <div><span className="text-muted">Nơi ở:</span> —</div>
                        </div>
                    </div>
                </div>
            ) : null}

            {activeTab === "notes" ? (
                <div style={{ display: "grid", gridTemplateColumns: "minmax(0, 1.1fr) minmax(320px, .9fr)", gap: "var(--space-4)" }}>
                    <div className="card">
                        <div className="card-title">Ghi chú hồ sơ</div>
                        <textarea
                            className="form-textarea"
                            value={notes}
                            onChange={(e) => setNotes(e.target.value)}
                            placeholder="Viết ghi chú cho hồ sơ này..."
                            style={{ minHeight: 340, resize: "vertical" }}
                        />
                        <div className="mt-3">
                            <button className="btn btn-primary" onClick={saveNotes}>
                                Lưu ghi chú
                            </button>
                        </div>
                    </div>
                    <AiNotebookPanel caseId={caseId} />
                </div>
            ) : null}
        </PageSection>
    );
}
