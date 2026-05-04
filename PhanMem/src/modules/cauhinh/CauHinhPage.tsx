import { useEffect, useState } from "react";
import { useModuleStore } from "../../store/moduleStore";
import { getModuleDefinition } from "../loidung/registry";
import { PageSection } from "./common";
import * as caseService from "../hosovuan/hosovuan.service";
import * as scanService from "../quettailieu/quettailieu.service";

function ModuleToggle({
    enabled,
    onChange,
}: {
    enabled: boolean;
    onChange: (enabled: boolean) => void;
}) {
    return (
        <label style={{ position: "relative", width: "36px", height: "20px", flexShrink: 0 }}>
            <input
                type="checkbox"
                checked={enabled}
                onChange={(e) => onChange(e.target.checked)}
                style={{ opacity: 0, width: 0, height: 0, position: "absolute" }}
            />
            <span
                style={{
                    position: "absolute",
                    inset: 0,
                    backgroundColor: enabled ? "var(--color-primary)" : "var(--color-border)",
                    borderRadius: "var(--radius-full)",
                    cursor: "pointer",
                    transition: "background-color 0.2s",
                }}
            >
                <span
                    style={{
                        position: "absolute",
                        width: "16px",
                        height: "16px",
                        borderRadius: "50%",
                        background: "#fff",
                        top: "2px",
                        left: enabled ? "18px" : "2px",
                        transition: "left 0.2s",
                    }}
                />
            </span>
        </label>
    );
}

function VersionSelector({
    moduleId,
    currentVersion,
    versions,
    onChangeVersion,
}: {
    moduleId: string;
    currentVersion: string;
    versions: string[];
    onChangeVersion: (moduleId: string, version: string) => void;
}) {
    if (versions.length <= 1) return null;

    return (
        <select
            value={currentVersion}
            onChange={(e) => onChangeVersion(moduleId, e.target.value)}
            style={{
                fontSize: "var(--text-xs)",
                padding: "2px 6px",
                borderRadius: "var(--radius-sm)",
                border: "1px solid var(--color-border)",
                background: "var(--color-surface)",
                color: "var(--color-text)",
                cursor: "pointer",
            }}
        >
            {versions.map((v) => (
                <option key={v} value={v}>{v}</option>
            ))}
        </select>
    );
}

export function SettingsPage() {
    const { configs } = useModuleStore();
    const toggleModule = useModuleStore((s) => s.toggleModule);
    const setModuleVersion = useModuleStore((s) => s.setModuleVersion);
    const [scanSettings, setScanSettings] = useState<scanService.ScanSettings | null>(null);
    const [cases, setCases] = useState<caseService.CaseSummary[]>([]);
    const [scanMessage, setScanMessage] = useState<string | null>(null);
    const [processMode, setProcessMode] = useState<scanService.PipelineProcessModeSettings | null>(null);
    const [purgeCaseCode, setPurgeCaseCode] = useState("");
    const [purgeIdentity, setPurgeIdentity] = useState("");
    const [purgeConfirmToken, setPurgeConfirmToken] = useState("");
    const [purgeBusy, setPurgeBusy] = useState(false);
    const [purgeMessage, setPurgeMessage] = useState<string | null>(null);

    useEffect(() => {
        Promise.all([
            scanService.getScanSettings(),
            caseService.listCases(),
            scanService.getPipelineProcessModeSettings(),
        ]).then(([settings, caseItems, mode]) => {
            setScanSettings(settings);
            setCases(caseItems);
            setProcessMode(mode);
        });
    }, []);

    const saveProcessModePatch = async (patch: Partial<scanService.PipelineProcessModeSettings>) => {
        if (!processMode) return;
        const next = { ...processMode, ...patch };
        setProcessMode(next);
        const saved = await scanService.savePipelineProcessModeSettings(next);
        setProcessMode(saved ?? next);
        setScanMessage("Đã lưu Cài đặt Process Mode.");
    };

    const saveScanPatch = async (patch: Partial<scanService.ScanSettings>) => {
        if (!scanSettings) return;
        const next = { ...scanSettings, ...patch };
        setScanSettings(next);
        const saved = await scanService.saveScanSettings(next);
        setScanSettings(saved ?? next);
        setScanMessage("Đã lưu Cài đặt Scan Ricoh.");
    };

    const chooseScanFolder = async () => {
        const { open } = await import("@tauri-apps/plugin-dialog");
        const folder = await open({ directory: true, multiple: false, title: "Chọn Ricoh Scan Inbox Folder" });
        if (folder) {
            await saveScanPatch({ inbox_folder: folder });
        }
    };

    const purgeCase = async () => {
        if (purgeBusy) return;
        setPurgeMessage(null);
        setPurgeBusy(true);
        try {
            const summary = await caseService.purgeImportedDossier({
                case_code: purgeCaseCode.trim(),
                expected_case_identity: purgeIdentity,
                confirm_token: "ok",
            });
            if (!summary) {
                setPurgeMessage("Xóa hồ sơ thất bại: không nhận được phản hồi backend.");
                return;
            }

            const dbRows = Object.entries(summary.deleted_rows_by_table)
                .map(([table, count]) => `${table}: ${count}`)
                .join(", ");
            const errors = summary.errors.length > 0 ? ` | Cảnh báo: ${summary.errors.join(" | ")}` : "";
            setPurgeMessage(`Đã purge hồ sơ ${summary.case_code} - ${summary.case_display_name}. DB(${dbRows}) | Files: ${summary.deleted_files_count} | Cache dirs: ${summary.deleted_cache_dirs_count}${errors}`);
            setPurgeCaseCode("");
            setPurgeIdentity("");
            setPurgeConfirmToken("");
            const caseItems = await caseService.listCases();
            setCases(caseItems);
        } finally {
            setPurgeBusy(false);
        }
    };

    const selectedCase = cases.find((c) => c.case_code === purgeCaseCode.trim()) ?? null;
    const expectedIdentity = selectedCase ? `${selectedCase.case_code} - ${selectedCase.case_display_name}` : "";
    const canPurge =
        !!selectedCase &&
        purgeIdentity === expectedIdentity &&
        purgeConfirmToken === "ok" &&
        !purgeBusy;

    return (
        <PageSection title="Cài đặt" description="Quản lý module, giao diện và tùy chọn ứng dụng">
            <div className="card mb-4">
                <div className="dashboard-card-header">
                    <div>
                        <div className="card-title">Cài đặt Process Mode</div>
                        <div className="card-subtitle">Điều khiển Manual vs Full Auto và các cờ autoscan/autosave/autoname/autosummary/autoclassify.</div>
                    </div>
                </div>
                <div className="settings-scan-grid">
                    <div className="form-group">
                        <label className="form-label">Process mode</label>
                        <select
                            className="form-select"
                            value={processMode?.process_mode ?? "manual"}
                            onChange={(e) => saveProcessModePatch({ process_mode: e.target.value as scanService.PipelineProcessModeSettings["process_mode"] })}
                        >
                            <option value="manual">Manual</option>
                            <option value="full_auto">Full auto</option>
                        </select>
                    </div>
                </div>
                <div className="settings-toggle-grid mb-3">
                    <label><input type="checkbox" checked={processMode?.autoscan ?? true} onChange={(e) => saveProcessModePatch({ autoscan: e.target.checked })} /> autoscan</label>
                    <label><input type="checkbox" checked={processMode?.autosave ?? true} onChange={(e) => saveProcessModePatch({ autosave: e.target.checked })} /> autosave</label>
                    <label><input type="checkbox" checked={processMode?.autoname ?? false} onChange={(e) => saveProcessModePatch({ autoname: e.target.checked })} /> autoname</label>
                    <label><input type="checkbox" checked={processMode?.autosummary ?? false} onChange={(e) => saveProcessModePatch({ autosummary: e.target.checked })} /> autosummary</label>
                    <label><input type="checkbox" checked={processMode?.autoclassify ?? false} onChange={(e) => saveProcessModePatch({ autoclassify: e.target.checked })} /> autoclassify</label>
                </div>
            </div>

            <div className="card mb-4">
                <div className="dashboard-card-header">
                    <div>
                        <div className="card-title">Cài đặt Scan Ricoh</div>
                        <div className="card-subtitle">Cấu hình Ricoh Scan to Folder, import tự động và xử lý sau scan.</div>
                    </div>
                    <button className="btn btn-sm" onClick={chooseScanFolder}>Chọn Scan Inbox Folder</button>
                </div>
                {scanMessage ? <div className="badge badge-success mb-3">{scanMessage}</div> : null}
                <div className="settings-scan-grid">
                    <div className="form-group">
                        <label className="form-label">Scan inbox folder</label>
                        <input
                            className="form-input"
                            value={scanSettings?.inbox_folder ?? ""}
                            onChange={(e) => saveScanPatch({ inbox_folder: e.target.value })}
                            placeholder="VD: D:\\ScanInbox hoặc \\\\server\\scan"
                        />
                    </div>
                    <div className="form-group">
                        <label className="form-label">Chế độ import</label>
                        <select
                            className="form-select"
                            value={scanSettings?.import_mode ?? "ask"}
                            onChange={(e) => saveScanPatch({ import_mode: e.target.value as scanService.ScanSettings["import_mode"] })}
                        >
                            <option value="manual">Import thủ công</option>
                            <option value="ask">Hỏi trước khi import</option>
                            <option value="auto">Tự động import</option>
                        </select>
                    </div>
                    <div className="form-group">
                        <label className="form-label">Hồ sơ mặc định</label>
                        <select
                            className="form-select"
                            value={scanSettings?.default_case_id ?? ""}
                            onChange={(e) => saveScanPatch({ default_case_id: e.target.value })}
                        >
                            <option value="">-- Không chọn --</option>
                            {cases.map((item) => (
                                <option key={item.case_id} value={item.case_id}>
                                    {item.case_code} - {item.case_display_name}
                                </option>
                            ))}
                        </select>
                    </div>
                    <div className="form-group">
                        <label className="form-label">Chờ file ổn định (ms)</label>
                        <input
                            className="form-input"
                            type="number"
                            min={500}
                            step={500}
                            value={scanSettings?.stable_wait_ms ?? 2500}
                            onChange={(e) => saveScanPatch({ stable_wait_ms: Number(e.target.value) || 2500 })}
                        />
                    </div>
                </div>
                <div className="settings-toggle-grid">
                    <label><input type="checkbox" checked={scanSettings?.accept_pdf ?? true} onChange={(e) => saveScanPatch({ accept_pdf: e.target.checked })} /> PDF</label>
                    <label><input type="checkbox" checked={scanSettings?.accept_tiff ?? true} onChange={(e) => saveScanPatch({ accept_tiff: e.target.checked })} /> TIFF</label>
                    <label><input type="checkbox" checked={scanSettings?.accept_jpg ?? true} onChange={(e) => saveScanPatch({ accept_jpg: e.target.checked })} /> JPG</label>
                    <label><input type="checkbox" checked={scanSettings?.accept_png ?? true} onChange={(e) => saveScanPatch({ accept_png: e.target.checked })} /> PNG</label>
                    <label><input type="checkbox" checked={scanSettings?.duplicate_detection ?? true} onChange={(e) => saveScanPatch({ duplicate_detection: e.target.checked })} /> Duplicate detection</label>
                    <label><input type="checkbox" checked={scanSettings?.ocr_after_scan ?? true} onChange={(e) => saveScanPatch({ ocr_after_scan: e.target.checked })} /> OCR sau scan</label>
                    <label><input type="checkbox" checked={scanSettings?.classify_after_scan ?? true} onChange={(e) => saveScanPatch({ classify_after_scan: e.target.checked })} /> Phân loại sau scan</label>
                </div>
            </div>

            <div className="card mb-4" style={{ border: "1px solid #fecaca", background: "var(--color-danger-surface)" }}>
                <div className="dashboard-card-header">
                    <div>
                        <div className="card-title" style={{ color: "#991b1b" }}>Vùng nguy hiểm — Xóa sạch hồ sơ đã import</div>
                        <div className="card-subtitle" style={{ color: "#991b1b" }}>
                            Hành động này KHÔNG THỂ HOÀN TÁC. Hệ thống sẽ xóa toàn bộ dữ liệu hồ sơ đã chọn:
                            file gốc, ảnh đính kèm/trang, cache/derived artifacts và dữ liệu DB liên quan.
                        </div>
                    </div>
                </div>
                {purgeMessage ? <div className="badge badge-danger mb-3">{purgeMessage}</div> : null}
                <div className="settings-scan-grid">
                    <div className="form-group">
                        <label className="form-label">Mã hồ sơ (case_code) cần xóa</label>
                        <input
                            className="form-input"
                            value={purgeCaseCode}
                            onChange={(e) => setPurgeCaseCode(e.target.value)}
                            placeholder="Nhập đúng mã hồ sơ, ví dụ: VK-..."
                        />
                    </div>
                    <div className="form-group">
                        <label className="form-label">Nhập chính xác chuỗi xác nhận hồ sơ</label>
                        <input
                            className="form-input"
                            value={purgeIdentity}
                            onChange={(e) => setPurgeIdentity(e.target.value)}
                            placeholder={expectedIdentity || "Chọn đúng case_code để hiện chuỗi cần nhập"}
                        />
                        {expectedIdentity ? (
                            <div className="text-muted text-xs" style={{ marginTop: "var(--space-1)" }}>
                                Chuỗi bắt buộc: <strong>{expectedIdentity}</strong>
                            </div>
                        ) : null}
                    </div>
                    <div className="form-group">
                        <label className="form-label">Xác nhận cuối cùng</label>
                        <input
                            className="form-input"
                            value={purgeConfirmToken}
                            onChange={(e) => setPurgeConfirmToken(e.target.value)}
                            placeholder="Nhập chính xác ok"
                        />
                    </div>
                </div>
                <div className="text-xs" style={{ color: "#991b1b", marginBottom: "var(--space-3)" }}>
                    Điều kiện bắt buộc: (1) case_code hợp lệ, (2) nhập đúng chuỗi hồ sơ hiển thị, (3) nhập chính xác <strong>ok</strong>.
                </div>
                <button className="btn btn-danger" disabled={!canPurge} onClick={purgeCase}>
                    {purgeBusy ? "Đang purge..." : "XÓA SẠCH HỒ SƠ"}
                </button>
            </div>

            <h2 style={{ fontSize: "var(--text-lg)", fontWeight: "var(--weight-semibold)", marginBottom: "var(--space-4)" }}>
                Quản lý Module
            </h2>

            <div
                style={{
                    display: "grid",
                    gridTemplateColumns: "repeat(auto-fill, minmax(300px, 1fr))",
                    gap: "var(--space-4)",
                }}
            >
                {configs.map((config) => {
                    const def = getModuleDefinition(config.module_id);
                    const meta = def ?? {
                        name: config.module_id,
                        icon: "📦",
                        description: "",
                        versions: [config.selected_version],
                    };

                    return (
                        <div className="card" key={config.module_id}>
                            <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between" }}>
                                <div style={{ display: "flex", alignItems: "center", gap: "var(--space-3)" }}>
                                    <span style={{ fontSize: "24px" }}>{meta.icon}</span>
                                    <div>
                                        <div className="card-title">{meta.name}</div>
                                        <div className="text-muted text-xs">{meta.description}</div>
                                    </div>
                                </div>
                                <ModuleToggle
                                    enabled={config.enabled}
                                    onChange={(value) => toggleModule(config.module_id, value)}
                                />
                            </div>
                            <div style={{ marginTop: "var(--space-3)", display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
                                <span className={`badge ${config.enabled ? "badge-success" : "badge-neutral"}`}>
                                    {config.enabled ? "Đang bật" : "Đã tắt"}
                                </span>
                                <VersionSelector
                                    moduleId={config.module_id}
                                    currentVersion={config.selected_version}
                                    versions={meta.versions}
                                    onChangeVersion={setModuleVersion}
                                />
                            </div>
                        </div>
                    );
                })}
            </div>
        </PageSection>
    );
}

