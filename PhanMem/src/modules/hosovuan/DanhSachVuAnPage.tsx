import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import * as caseService from "../hosovuan/hosovuan.service";
import * as importService from "../duahosovao/duahosovao.service";
import { PageSection } from "./common";

function statusBadge(status: string) {
    const cls = status === "active" ? "badge-success" : status === "closed" ? "badge-neutral" : "badge-warning";
    return <span className={`badge ${cls}`}>{status}</span>;
}

export function CaseListPage() {
    const [cases, setCases] = useState<caseService.CaseSummary[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);
    const [filter, setFilter] = useState<string>("all");
    const [purgeTarget, setPurgeTarget] = useState<caseService.CaseSummary | null>(null);
    const [purgeIdentity, setPurgeIdentity] = useState("");
    const [purgeToken, setPurgeToken] = useState("");
    const [purgeBusy, setPurgeBusy] = useState(false);
    const [sortKey, setSortKey] = useState<"name" | "dossier_type" | "but_luc">("name");
    const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
    const navigate = useNavigate();

    const loadCases = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const data = await caseService.listCases();
            setCases(data);
        } catch (e) {
            setError((e as Error).message);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        loadCases();
    }, [loadCases]);

    const createCase = useCallback(async () => {
        const title = window.prompt("Nhập tên hồ sơ mới:");
        if (!title?.trim()) return;

        try {
            setLoading(true);
            setError(null);
            setSuccessMessage(null);
            const created = await caseService.createCase({
                case_display_name: title.trim(),
                primary_person_name: title.trim(),
                source_folder_name: title.trim(),
            });

            if (created) {
                await loadCases();
                setSuccessMessage(`Đã tạo hồ sơ: ${created.case_code} — ${created.case_display_name}`);
                navigate(`/cases/${created.case_id}`);
            } else {
                setError("Không thể tạo hồ sơ. Kiểm tra kết nối Tauri.");
            }
        } catch (e) {
            setError((e as Error).message || "Lỗi không xác định khi tạo hồ sơ.");
        } finally {
            setLoading(false);
        }
    }, [loadCases, navigate]);

    const importFolder = useCallback(async () => {
        try {
            const { open } = await import("@tauri-apps/plugin-dialog");
            const selected = await open({ directory: true, multiple: false, title: "Chọn thư mục hồ sơ để import" });
            if (!selected) return;

            setLoading(true);
            setError(null);
            const result = await importService.importFolder(selected);
            if (result) {
                await loadCases();
                setSuccessMessage(`Import thành công ${result.files.length}/${result.total_files} file vào hồ sơ mới.`);
                navigate(`/cases/${result.case_id}`);
            } else {
                setError(importService.getLastImportError() ?? "Import thư mục thất bại.");
            }
        } catch (e) {
            setError((e as Error).message || "Lỗi khi import thư mục.");
        } finally {
            setLoading(false);
        }
    }, [loadCases, navigate]);

    const importFilesToCase = useCallback(async (targetCase: caseService.CaseSummary) => {
        try {
            const { open } = await import("@tauri-apps/plugin-dialog");
            const selected = await open({
                multiple: true,
                title: `Chọn file để import vào hồ sơ: ${targetCase.case_code}`,
                filters: [{ name: "Tài liệu scan", extensions: ["pdf", "png", "jpg", "jpeg", "tif", "tiff"] }],
            });
            if (!selected) return;
            const paths = Array.isArray(selected) ? selected : [selected];
            if (paths.length === 0) return;

            setLoading(true);
            setError(null);
            setSuccessMessage(null);
            const result = await importService.importMultipleFiles(paths, targetCase.case_id);
            if (!result) {
                setError(importService.getLastImportError() ?? "Import file vào hồ sơ thất bại.");
                return;
            }
            setSuccessMessage(
                `Đã import vào hồ sơ ${targetCase.case_code}: ${result.imported.length} thành công, ${result.duplicates.length} trùng, ${result.errors.length} lỗi.`
            );
            await loadCases();
        } catch (e) {
            setError((e as Error).message || "Lỗi khi import file vào hồ sơ.");
        } finally {
            setLoading(false);
        }
    }, [loadCases]);

    const openPurgeDialog = useCallback((item: caseService.CaseSummary) => {
        setError(null);
        setSuccessMessage(null);
        setPurgeTarget(item);
        setPurgeIdentity("");
        setPurgeToken("");
    }, []);

    const closePurgeDialog = useCallback(() => {
        if (purgeBusy) return;
        setPurgeTarget(null);
        setPurgeIdentity("");
        setPurgeToken("");
    }, [purgeBusy]);

    const purgeCase = useCallback(async () => {
        if (!purgeTarget || purgeBusy) return;

        setPurgeBusy(true);
        setError(null);
        setSuccessMessage(null);
        try {
            const summary = await caseService.purgeImportedDossier({
                case_code: purgeTarget.case_code,
                expected_case_identity: purgeIdentity,
                confirm_token: "ok",
            });

            if (!summary) {
                setError("Xóa hồ sơ thất bại: backend không trả phản hồi.");
                return;
            }

            const warnings = summary.errors.length > 0 ? ` Cảnh báo: ${summary.errors.join(" | ")}` : "";
            setSuccessMessage(
                `Đã xóa sạch hồ sơ ${summary.case_code}. Files: ${summary.deleted_files_count}, cache: ${summary.deleted_cache_dirs_count}.${warnings}`
            );
            setPurgeTarget(null);
            setPurgeIdentity("");
            setPurgeToken("");
            await loadCases();
        } catch (e) {
            setError((e as Error).message || "Lỗi không xác định khi xóa hồ sơ.");
        } finally {
            setPurgeBusy(false);
        }
    }, [loadCases, purgeBusy, purgeIdentity, purgeTarget]);

    const filtered = filter === "all" ? cases : cases.filter((c) => c.status === filter);
    const sortedCases = useMemo(() => {
        const toButLucNum = (v: string | null): number => {
            if (!v) return Number.MAX_SAFE_INTEGER;
            const m = v.match(/\d+/);
            return m ? Number(m[0]) : Number.MAX_SAFE_INTEGER;
        };

        const items = [...filtered].sort((a, b) => {
            let cmp = 0;
            if (sortKey === "name") {
                cmp = a.case_display_name.localeCompare(b.case_display_name, "vi", { sensitivity: "base" });
            } else if (sortKey === "dossier_type") {
                cmp = (a.dossier_type || "").localeCompare(b.dossier_type || "", "vi", { sensitivity: "base" });
            } else {
                cmp = toButLucNum(a.but_luc) - toButLucNum(b.but_luc);
                if (cmp === 0) cmp = (a.but_luc || "").localeCompare(b.but_luc || "", "vi", { sensitivity: "base" });
            }
            return sortDir === "asc" ? cmp : -cmp;
        });
        return items;
    }, [filtered, sortDir, sortKey]);

    const setSort = (key: "name" | "dossier_type" | "but_luc") => {
        if (sortKey === key) {
            setSortDir((prev) => (prev === "asc" ? "desc" : "asc"));
            return;
        }
        setSortKey(key);
        setSortDir("asc");
    };

    const purgeExpectedIdentity = purgeTarget ? `${purgeTarget.case_code} - ${purgeTarget.case_display_name}` : "";
    const canPurge =
        !!purgeTarget &&
        purgeIdentity === purgeExpectedIdentity &&
        purgeToken === "ok" &&
        !purgeBusy;

    return (
        <PageSection
            title="Danh sách Hồ sơ"
            description={`${cases.length} hồ sơ`}
            right={
                <div style={{ display: "flex", gap: "var(--space-2)" }}>
                    <button className="btn" onClick={importFolder} disabled={loading}>
                        {loading ? "Đang xử lý..." : "⇩ Import thư mục"}
                    </button>
                    <button className="btn btn-primary" onClick={createCase} disabled={loading}>
                        + Tạo hồ sơ
                    </button>
                </div>
            }
        >
            {error ? <div className="module-error mb-3">⚠️ {error}</div> : null}
            {successMessage ? <div className="badge badge-success mb-3">{successMessage}</div> : null}

            <div className="toolbar">
                <button className={`btn btn-sm ${filter === "all" ? "btn-primary" : "btn-ghost"}`} onClick={() => setFilter("all")}>
                    Tất cả ({cases.length})
                </button>
                <button className={`btn btn-sm ${filter === "active" ? "btn-primary" : "btn-ghost"}`} onClick={() => setFilter("active")}>
                    Đang xử lý
                </button>
                <button className={`btn btn-sm ${filter === "archived" ? "btn-primary" : "btn-ghost"}`} onClick={() => setFilter("archived")}>
                    Lưu trữ
                </button>
                <button className={`btn btn-sm ${filter === "closed" ? "btn-primary" : "btn-ghost"}`} onClick={() => setFilter("closed")}>
                    Đã đóng
                </button>
            </div>

            <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                <table className="data-table">
                    <thead>
                        <tr>
                            <th style={{ width: 120 }}>Mã</th>
                            <th>
                                <button className="btn btn-ghost btn-sm" onClick={() => setSort("name")}>Tên hồ sơ {sortKey === "name" ? (sortDir === "asc" ? "↑" : "↓") : ""}</button>
                            </th>
                            <th>
                                <button className="btn btn-ghost btn-sm" onClick={() => setSort("dossier_type")}>Loại hồ sơ {sortKey === "dossier_type" ? (sortDir === "asc" ? "↑" : "↓") : ""}</button>
                            </th>
                            <th>
                                <button className="btn btn-ghost btn-sm" onClick={() => setSort("but_luc")}>Bút lục {sortKey === "but_luc" ? (sortDir === "asc" ? "↑" : "↓") : ""}</button>
                            </th>
                            <th>Bị can</th>
                            <th style={{ width: 130 }}>Trạng thái</th>
                            <th style={{ width: 60 }}>Tài liệu</th>
                            <th style={{ width: 140 }}>Ngày tạo</th>
                            <th style={{ width: 220 }}>Thao tác</th>
                        </tr>
                    </thead>
                    <tbody>
                        {loading ? (
                            <tr>
                                <td colSpan={9} style={{ textAlign: "center", color: "var(--color-text-muted)" }}>Đang tải...</td>
                            </tr>
                        ) : sortedCases.length === 0 ? (
                            <tr>
                                <td colSpan={9} style={{ textAlign: "center", color: "var(--color-text-muted)" }}>
                                    {cases.length === 0 ? "Chưa có hồ sơ nào." : "Không có hồ sơ phù hợp bộ lọc."}
                                </td>
                            </tr>
                        ) : (
                            sortedCases.map((item) => (
                                <tr
                                    key={item.case_id}
                                    onClick={() => navigate(`/cases/${item.case_id}`)}
                                    style={{
                                        cursor: "pointer",
                                        background: item.missing_document_count > 0 ? "var(--color-danger-surface)" : undefined,
                                    }}
                                >
                                    <td className="text-mono text-xs">{item.case_code}</td>
                                    <td className="font-semibold">{item.case_display_name}</td>
                                    <td><span className="badge badge-neutral">{item.dossier_type || item.case_type || "khong_xac_dinh"}</span></td>
                                    <td>
                                        {item.but_luc ? (
                                            <span className="badge badge-primary">{item.but_luc}</span>
                                        ) : (
                                            <span className="text-xs text-muted">{item.ocr_state === "pending" ? "Đang OCR..." : "Chưa có"}</span>
                                        )}
                                    </td>
                                    <td className="text-secondary">{item.primary_person_name}</td>
                                    <td>
                                        <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                                            {statusBadge(item.status)}
                                            {item.missing_document_count > 0 ? (
                                                <span className="badge badge-danger">
                                                    Mất {item.missing_document_count} file
                                                </span>
                                            ) : null}
                                        </div>
                                    </td>
                                    <td style={{ textAlign: "center" }}>{item.document_count}</td>
                                    <td className="text-muted text-xs">{item.created_at}</td>
                                    <td>
                                        <button
                                            className="btn btn-sm"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                importFilesToCase(item);
                                            }}
                                            disabled={loading}
                                            style={{ marginRight: 6 }}
                                        >
                                            Import vào hồ sơ
                                        </button>
                                        <button
                                            className="btn btn-sm btn-danger"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                openPurgeDialog(item);
                                            }}
                                        >
                                            Xóa
                                        </button>
                                    </td>
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>

            {purgeTarget ? (
                <div className="modal-backdrop" onClick={closePurgeDialog}>
                    <div className="rescan-modal card" onClick={(e) => e.stopPropagation()}>
                        <div className="dashboard-card-header">
                            <div>
                                <div className="card-title" style={{ color: "var(--color-danger)" }}>
                                    Xóa sạch hồ sơ đã nhập
                                </div>
                                <div className="card-subtitle">
                                    Hành động này xóa dữ liệu DB, file gốc, ảnh scan/cache OCR và không thể hoàn tác.
                                </div>
                            </div>
                        </div>

                        <div className="card mb-3" style={{ background: "var(--color-danger-surface)", borderColor: "#fecaca" }}>
                            <div className="text-xs text-muted mb-1">Hồ sơ sẽ xóa</div>
                            <div className="font-semibold">{purgeTarget.case_display_name}</div>
                            <div className="text-mono text-xs">{purgeTarget.case_code}</div>
                            <div className="text-xs text-muted">{purgeTarget.document_count} tài liệu</div>
                        </div>

                        <div className="form-group mb-3">
                            <label className="form-label">Nhập lại đúng chuỗi hồ sơ</label>
                            <input
                                className="form-input"
                                value={purgeIdentity}
                                onChange={(e) => setPurgeIdentity(e.target.value)}
                                placeholder={purgeExpectedIdentity}
                            />
                            <div className="text-xs text-muted" style={{ marginTop: "var(--space-1)" }}>
                                Chuỗi bắt buộc: <strong>{purgeExpectedIdentity}</strong>
                            </div>
                        </div>

                        <div className="form-group mb-3">
                            <label className="form-label">Xác nhận cuối cùng</label>
                            <input
                                className="form-input"
                                value={purgeToken}
                                onChange={(e) => setPurgeToken(e.target.value)}
                                placeholder="Nhập chính xác ok"
                            />
                        </div>

                        <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--space-2)" }}>
                            <button className="btn" onClick={closePurgeDialog} disabled={purgeBusy}>
                                Hủy
                            </button>
                            <button className="btn btn-danger" onClick={purgeCase} disabled={!canPurge}>
                                {purgeBusy ? "Đang xóa..." : "Xóa sạch hồ sơ"}
                            </button>
                        </div>
                    </div>
                </div>
            ) : null}
        </PageSection>
    );
}
