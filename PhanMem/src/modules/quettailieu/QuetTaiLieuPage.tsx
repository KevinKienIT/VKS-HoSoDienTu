import { useCallback, useEffect, useMemo, useState } from "react";
import { FolderOpen, Play, Square, RefreshCw, FileCheck2, AlertTriangle } from "lucide-react";
import { DocumentViewer } from "../phantichtailieu/TrinhXemTaiLieu";
import * as caseService from "../hosovuan/hosovuan.service";
import * as documentService from "../quantailieu/quantailieu.service";
import * as scanService from "../quettailieu/quettailieu.service";
import { PageSection } from "./common";

function scanStatusBadge(status: string) {
    if (status === "ready" || status === "imported") return "badge badge-success";
    if (status === "waiting_for_stable" || status === "importing") return "badge badge-warning";
    if (status === "duplicate") return "badge badge-info";
    if (status === "error") return "badge badge-danger";
    return "badge badge-neutral";
}

function scanStatusLabel(status: string) {
    const labels: Record<string, string> = {
        waiting_for_stable: "Chờ ghi xong",
        ready: "Sẵn sàng import",
        duplicate: "File trùng",
        error: "File lỗi",
        importing: "Đang import",
        imported: "Đã vào hồ sơ",
    };
    return labels[status] ?? status;
}

export function ScanPage() {
    const [settings, setSettings] = useState<scanService.ScanSettings | null>(null);
    const [cases, setCases] = useState<caseService.CaseSummary[]>([]);
    const [selectedCaseId, setSelectedCaseId] = useState("");
    const [driverStatus, setDriverStatus] = useState<scanService.ScannerDriverStatus | null>(null);
    const [files, setFiles] = useState<scanService.ScanInboxFile[]>([]);
    const [selectedFile, setSelectedFile] = useState<scanService.ScanInboxFile | null>(null);
    const [watching, setWatching] = useState(false);
    const [busyPath, setBusyPath] = useState<string | null>(null);
    const [message, setMessage] = useState<string | null>(null);
    const [lastTick, setLastTick] = useState("");
    const [ocrMessage, setOcrMessage] = useState<string | null>(null);

    const loadInbox = useCallback(async (folder?: string) => {
        const inboxFiles = await scanService.listScanInboxFiles(folder);
        setFiles(inboxFiles);
        setSelectedFile((prev) => {
            if (prev && inboxFiles.some((item) => item.file_path === prev.file_path)) return prev;
            return inboxFiles[0] ?? null;
        });
        setLastTick(new Date().toLocaleTimeString("vi-VN"));
    }, []);

    useEffect(() => {
        Promise.all([
            scanService.getScanSettings(),
            caseService.listCases(),
            scanService.checkScannerDriverStatus(),
        ]).then(([scanSettings, caseItems, status]) => {
            setSettings(scanSettings);
            setCases(caseItems);
            setSelectedCaseId(scanSettings.default_case_id || caseItems[0]?.case_id || "");
            setDriverStatus(status);
            if (scanSettings.inbox_folder) {
                loadInbox(scanSettings.inbox_folder);
            }
        });
    }, [loadInbox]);

    useEffect(() => {
        if (!watching || !settings?.inbox_folder) return;
        const timer = window.setInterval(() => {
            loadInbox(settings.inbox_folder);
        }, 2200);
        return () => window.clearInterval(timer);
    }, [loadInbox, settings?.inbox_folder, watching]);

    const readyFiles = useMemo(() => files.filter((file) => file.status === "ready"), [files]);
    const waitingFiles = useMemo(() => files.filter((file) => file.status === "waiting_for_stable"), [files]);
    const duplicateFiles = useMemo(() => files.filter((file) => file.status === "duplicate"), [files]);

    const chooseFolder = async () => {
        const { open } = await import("@tauri-apps/plugin-dialog");
        const folder = await open({ directory: true, multiple: false, title: "Chọn thư mục Ricoh Scan Inbox" });
        if (!folder || !settings) return;
        const next = { ...settings, inbox_folder: folder };
        const saved = await scanService.saveScanSettings(next);
        setSettings(saved ?? next);
        await loadInbox(folder);
    };

    const saveSettings = async (patch: Partial<scanService.ScanSettings>) => {
        if (!settings) return;
        const next = { ...settings, ...patch };
        const saved = await scanService.saveScanSettings(next);
        setSettings(saved ?? next);
    };

    const startWatch = async () => {
        if (driverStatus && !driverStatus.scan_to_folder_supported) {
            setMessage("Scan to Folder chưa sẵn sàng trên máy này. Kiểm tra cấu hình SMB/Ricoh và quyền thư mục.");
            return;
        }
        if (!settings?.inbox_folder) {
            setMessage("Chưa chọn Scan Inbox Folder.");
            return;
        }
        console.info("[ScanPage] startWatch", { folder: settings.inbox_folder });
        const ok = await scanService.watchScanFolder(settings.inbox_folder);
        setWatching(ok);
        setMessage(ok ? "Đang theo dõi thư mục Ricoh Scan Inbox." : "Không thể bắt đầu theo dõi thư mục.");
        await loadInbox(settings.inbox_folder);
    };

    const stopWatch = async () => {
        await scanService.stopWatchScanFolder();
        setWatching(false);
        setMessage("Đã dừng theo dõi thư mục scan.");
    };

    const importFile = async (file: scanService.ScanInboxFile) => {
        if (!selectedCaseId) {
            setMessage("Chưa chọn hồ sơ đích.");
            return;
        }
        setBusyPath(file.file_path);
        console.info("[ScanPage] importFile", { filePath: file.file_path, caseId: selectedCaseId, ext: file.file_ext, status: file.status });
        const result = await scanService.importScannedFile(file.file_path, selectedCaseId);
        if (result?.status === "imported" && result.document_id && settings?.ocr_after_scan) {
            setOcrMessage(`Đang OCR offline: ${file.file_name}`);
            const ocr = await documentService.runOcrForDocument(result.document_id);
            setOcrMessage(
                ocr
                    ? `OCR ${file.file_name}: ${ocr.processed_pages} trang, ${ocr.failed_pages} lỗi, ${ocr.handwritten_pages ?? 0} trang nghi viết tay, confidence ${Math.round(ocr.average_confidence * 100)}%.`
                    : `OCR ${file.file_name} chưa khả dụng hoặc thất bại. Kiểm tra Python/PaddleOCR/PyMuPDF và log Tauri (run_ocr_for_document).`
            );
        }
        setBusyPath(null);
        setMessage(result?.message ?? "Import scan thất bại.");
        await loadInbox(settings?.inbox_folder);
    };

    const importReadyBatch = async () => {
        if (!selectedCaseId) {
            setMessage("Chưa chọn hồ sơ đích.");
            return;
        }
        const paths = readyFiles.map((file) => file.file_path);
        if (paths.length === 0) {
            setMessage("Không có file sẵn sàng import.");
            return;
        }
        setBusyPath("__batch__");
        const results = await scanService.importScannedBatch(paths, selectedCaseId);
        if (settings?.ocr_after_scan) {
            let ocrDone = 0;
            let ocrFailed = 0;
            for (const result of results) {
                if (result.status !== "imported" || !result.document_id) continue;
                setOcrMessage(`Đang OCR offline batch: ${ocrDone + 1}/${results.length}`);
                const ocr = await documentService.runOcrForDocument(result.document_id);
                ocrDone += 1;
                if (!ocr || ocr.failed_pages > 0 || ocr.status === "error") {
                    ocrFailed += 1;
                }
            }
            setOcrMessage(`OCR batch hoàn tất: ${ocrDone} tài liệu, ${ocrFailed} lỗi một phần/toàn bộ.`);
        }
        setBusyPath(null);
        setMessage(`Đã xử lý ${results.length} file scan.`);
        await loadInbox(settings?.inbox_folder);
    };

    useEffect(() => {
        if (!watching || settings?.import_mode !== "auto" || !selectedCaseId || readyFiles.length === 0 || busyPath) {
            return;
        }
        const timer = window.setTimeout(() => {
            importReadyBatch();
        }, 600);
        return () => window.clearTimeout(timer);
    }, [busyPath, readyFiles.length, selectedCaseId, settings?.import_mode, watching]);

    return (
        <PageSection
            title="Ricoh Scan Center"
            description="Ricoh Scan to Folder → app phát hiện file mới → preview → import vào hồ sơ → OCR/review queue"
            right={
                <div className="flex gap-2">
                    <button className="btn" onClick={() => loadInbox(settings?.inbox_folder)}>
                        <RefreshCw size={14} /> Làm mới
                    </button>
                    {watching ? (
                        <button className="btn btn-danger" onClick={stopWatch}>
                            <Square size={14} /> Dừng theo dõi
                        </button>
                    ) : (
                        <button className="btn btn-primary" onClick={startWatch}>
                            <Play size={14} /> Theo dõi thư mục
                        </button>
                    )}
                </div>
            }
        >
            {message ? <div className="module-error mb-3">{message}</div> : null}
            {ocrMessage ? <div className="toast-inline mb-3">{ocrMessage}</div> : null}

            <div className="scan-center-grid">
                <aside className="card scan-case-panel">
                    <div className="card-title">Hồ sơ đang nhận scan</div>
                    <label className="form-label">Hồ sơ đích</label>
                    <select
                        className="form-select"
                        value={selectedCaseId}
                        onChange={(e) => {
                            setSelectedCaseId(e.target.value);
                            saveSettings({ default_case_id: e.target.value });
                        }}
                    >
                        <option value="">-- Chọn hồ sơ --</option>
                        {cases.map((item) => (
                            <option key={item.case_id} value={item.case_id}>
                                {item.case_code} - {item.case_display_name}
                            </option>
                        ))}
                    </select>

                    <div className="scan-folder-box">
                        <div className="stat-label">Ricoh Scan Inbox Folder</div>
                        <div className="scan-folder-path">{settings?.inbox_folder || "Chưa cấu hình"}</div>
                        <button className="btn btn-sm" onClick={chooseFolder}>
                            <FolderOpen size={13} /> Chọn thư mục
                        </button>
                    </div>

                    <div className="scan-state-stack">
                        <span className={watching ? "badge badge-success" : "badge badge-neutral"}>
                            {watching ? "Đang theo dõi thư mục Ricoh" : "Chưa theo dõi"}
                        </span>
                        <span className={driverStatus?.scan_to_folder_supported ? "badge badge-success" : "badge badge-danger"}>
                            Scan to Folder {driverStatus?.scan_to_folder_supported ? "sẵn sàng" : "không sẵn sàng"}
                        </span>
                        <span className="badge badge-neutral">WIA: {driverStatus?.wia_service_status ?? "unknown"}</span>
                    </div>

                    <div className="scan-driver-note">
                        <AlertTriangle size={14} />
                        <span>{driverStatus?.recommendation ?? "Đang kiểm tra driver scan..."}</span>
                    </div>
                </aside>

                <section className="card scan-inbox-panel">
                    <div className="dashboard-card-header">
                        <div>
                            <div className="card-title">Scan Inbox</div>
                            <div className="card-subtitle">File Ricoh vừa scan. Cập nhật gần nhất: {lastTick || "-"}</div>
                        </div>
                        <div className="scan-summary-badges">
                            <span className="badge badge-success">{readyFiles.length} sẵn sàng</span>
                            <span className="badge badge-warning">{waitingFiles.length} đang ghi</span>
                            <span className="badge badge-info">{duplicateFiles.length} trùng</span>
                        </div>
                    </div>

                    <div className="scan-progress-strip">
                        <div style={{ width: `${files.length ? Math.round((readyFiles.length / files.length) * 100) : 0}%` }} />
                    </div>

                    <div className="scan-inbox-list">
                        {files.map((file) => (
                            <button
                                key={file.file_path}
                                className={`scan-file-row ${selectedFile?.file_path === file.file_path ? "active" : ""}`}
                                onClick={() => setSelectedFile(file)}
                            >
                                <FileCheck2 size={16} />
                                <span className="truncate">
                                    <b>{file.file_name}</b>
                                    <small>{(file.file_size / 1024).toFixed(0)} KB · {file.reason}</small>
                                </span>
                                <span className={scanStatusBadge(file.status)}>{scanStatusLabel(file.status)}</span>
                            </button>
                        ))}
                        {files.length === 0 ? (
                            <div className="empty-state-text">
                                Chưa có file scan trong inbox. Trên máy Ricoh, chọn Scan to Folder và trỏ đến thư mục đã cấu hình.
                            </div>
                        ) : null}
                    </div>
                </section>

                <aside className="card scan-preview-panel">
                    <div className="dashboard-card-header">
                        <div>
                            <div className="card-title">Preview & xử lý</div>
                            <div className="card-subtitle">Kiểm tra file trước khi đưa vào hồ sơ.</div>
                        </div>
                    </div>

                    <div className="scan-processing-options">
                        <label><input type="checkbox" checked={settings?.ocr_after_scan ?? true} onChange={(e) => saveSettings({ ocr_after_scan: e.target.checked })} /> OCR sau import</label>
                        <label><input type="checkbox" checked={settings?.classify_after_scan ?? true} onChange={(e) => saveSettings({ classify_after_scan: e.target.checked })} /> Phân loại sau import</label>
                        <label><input type="checkbox" checked={settings?.duplicate_detection ?? true} onChange={(e) => saveSettings({ duplicate_detection: e.target.checked })} /> Kiểm tra trùng hash</label>
                    </div>

                    <div className="scan-preview-box">
                        {selectedFile ? (
                            <DocumentViewer
                                documentPath={selectedFile.file_path}
                                documentType={selectedFile.file_ext}
                            />
                        ) : (
                            <div className="viewer-empty-state">Chọn file scan để preview.</div>
                        )}
                    </div>

                    <div className="scan-action-row">
                        <button
                            className="btn btn-primary"
                            disabled={!selectedFile || selectedFile.status !== "ready" || busyPath === selectedFile?.file_path}
                            onClick={() => selectedFile && importFile(selectedFile)}
                        >
                            {busyPath === selectedFile?.file_path ? "Đang import..." : "Import file này"}
                        </button>
                        <button
                            className="btn"
                            disabled={readyFiles.length === 0 || busyPath === "__batch__"}
                            onClick={importReadyBatch}
                        >
                            {busyPath === "__batch__" ? "Đang import batch..." : "Import tất cả sẵn sàng"}
                        </button>
                    </div>
                </aside>
            </div>
        </PageSection>
    );
}
