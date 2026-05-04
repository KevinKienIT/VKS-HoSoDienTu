import { useState } from "react";
import { Link } from "react-router-dom";
import * as documentService from "../quantailieu/quantailieu.service";
import * as importService from "../duahosovao/duahosovao.service";
import { PageSection } from "./common";

type ImportStep = "select" | "scan" | "ocr" | "review" | "complete";

const STEPS: { key: ImportStep; label: string }[] = [
    { key: "select", label: "Chọn thư mục" },
    { key: "scan", label: "Quét file" },
    { key: "ocr", label: "OCR" },
    { key: "review", label: "Kiểm tra" },
    { key: "complete", label: "Hoàn tất" },
];

function StepIndicator({ current }: { current: ImportStep }) {
    const idx = STEPS.findIndex((s) => s.key === current);
    return (
        <div style={{ display: "flex", alignItems: "center", gap: 0, marginBottom: "var(--space-4)" }}>
            {STEPS.map((s, i) => {
                const done = i < idx;
                const active = i === idx;
                return (
                    <div key={s.key} style={{ display: "flex", alignItems: "center", flex: i < STEPS.length - 1 ? 1 : "none" }}>
                        <div style={{
                            width: 28,
                            height: 28,
                            borderRadius: "var(--radius-full)",
                            display: "grid",
                            placeItems: "center",
                            fontSize: "var(--text-xs)",
                            fontWeight: "var(--weight-bold)",
                            flexShrink: 0,
                            background: done ? "var(--color-success)" : active ? "var(--color-primary)" : "var(--color-bg-muted)",
                            color: done || active ? "#fff" : "var(--color-text-muted)",
                            border: active ? "2px solid var(--color-primary-light)" : "2px solid transparent",
                            transition: "all 200ms ease",
                        }}>
                            {done ? "✓" : i + 1}
                        </div>
                        <div style={{
                            marginLeft: "var(--space-1_5)",
                            fontSize: "var(--text-xs)",
                            fontWeight: active ? "var(--weight-semibold)" : "var(--weight-normal)",
                            color: active ? "var(--color-text)" : "var(--color-text-muted)",
                            whiteSpace: "nowrap",
                        }}>
                            {s.label}
                        </div>
                        {i < STEPS.length - 1 ? (
                            <div style={{
                                flex: 1,
                                height: 2,
                                margin: "0 var(--space-2)",
                                background: done ? "var(--color-success)" : "var(--color-border)",
                                borderRadius: 1,
                                transition: "background 200ms ease",
                            }} />
                        ) : null}
                    </div>
                );
            })}
        </div>
    );
}

export function ImportJobPage() {
    const [step, setStep] = useState<ImportStep>("select");
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [result, setResult] = useState<importService.ImportFolderResult | null>(null);
    const [multiResult, setMultiResult] = useState<importService.ImportMultipleFilesResult | null>(null);
    const [autoOcr, setAutoOcr] = useState(true);
    const [ocrProgress, setOcrProgress] = useState<{
        total: number;
        done: number;
        failed: number;
        current: string;
        results: documentService.OcrRunResult[];
    }>({ total: 0, done: 0, failed: 0, current: "", results: [] });

    const runOcrForImportedFiles = async (files: importService.ImportedFile[]) => {
        if (!autoOcr || files.length === 0) {
            setOcrProgress({ total: files.length, done: 0, failed: 0, current: "OCR được bỏ qua theo lựa chọn người dùng.", results: [] });
            return;
        }

        setOcrProgress({ total: files.length, done: 0, failed: 0, current: "Chuẩn bị OCR offline...", results: [] });
        const results: documentService.OcrRunResult[] = [];
        let failed = 0;
        for (const file of files) {
            setOcrProgress((prev) => ({ ...prev, current: file.file_name }));
            const ocr = await documentService.runOcrForDocument(file.document_id);
            if (!ocr || ocr.failed_pages > 0 || ocr.status === "error") {
                failed += 1;
            }
            if (ocr) {
                results.push(ocr);
            }
            setOcrProgress((prev) => ({
                ...prev,
                done: prev.done + 1,
                failed,
                results: ocr ? [...prev.results, ocr] : prev.results,
            }));
        }
        setOcrProgress((prev) => ({ ...prev, current: "OCR hoàn tất" }));
    };

    const runImport = async () => {
        try {
            const { open } = await import("@tauri-apps/plugin-dialog");
            const selected = await open({ directory: true, multiple: false, title: "Chọn thư mục hồ sơ để import" });
            if (!selected) return;

            setBusy(true);
            setError(null);
            setMultiResult(null);
            setStep("scan");
            const imported = await importService.importFolder(selected);
            console.info("[ImportJobPage] import_folder result", {
                selected,
                hasResult: Boolean(imported),
                files: imported?.files.length ?? 0,
            });
            setStep("ocr");
            if (!imported) {
                throw new Error(importService.getLastImportError() ?? "Import command trả về null.");
            }
            setResult(imported);
            await runOcrForImportedFiles(imported.files);
            setStep("review");
            setStep("complete");
        } catch (e) {
            setError((e as Error).message || "Lỗi import job.");
        } finally {
            setBusy(false);
        }
    };

    const runMultiImport = async () => {
        try {
            const { open } = await import("@tauri-apps/plugin-dialog");
            const selected = await open({
                directory: false,
                multiple: true,
                title: "Chọn nhiều file PDF/ảnh để import",
                filters: [{ name: "Tài liệu scan", extensions: ["pdf", "png", "jpg", "jpeg", "tif", "tiff"] }],
            });
            if (!selected) return;
            const paths = Array.isArray(selected) ? selected : [selected];
            if (paths.length === 0) return;

            setBusy(true);
            setError(null);
            setResult(null);
            setStep("scan");
            const imported = await importService.importMultipleFiles(paths);
            console.info("[ImportJobPage] import_multiple_files result", {
                fileCount: paths.length,
                hasResult: Boolean(imported),
                importedCount: imported?.imported.length ?? 0,
                duplicateCount: imported?.duplicates.length ?? 0,
                errorCount: imported?.errors.length ?? 0,
            });
            setStep("ocr");
            if (!imported) throw new Error(importService.getLastImportError() ?? "Import nhiều file trả về null.");
            setMultiResult(imported);
            await runOcrForImportedFiles(imported.imported);
            setStep("review");
            setStep("complete");
        } catch (e) {
            setError((e as Error).message || "Lỗi import nhiều file.");
        } finally {
            setBusy(false);
        }
    };

    return (
        <PageSection title="Đưa hồ sơ vào" description="Nạp PDF scan hoặc thư mục hồ sơ vào SQLite/Tauri để xử lý OCR và phân loại">
            {error ? <div className="module-error mb-3">⚠️ {error}</div> : null}

            <div className="card mb-4">
                <StepIndicator current={step} />

                {step === "select" ? (
                    <div className="import-dropzone">
                        <div className="import-dropzone-icon">⇩</div>
                        <div className="empty-state-title">Kéo thả PDF hoặc chọn thư mục hồ sơ đã scan</div>
                        <div className="empty-state-text">
                            Backend hiện hỗ trợ import thư mục qua Tauri command. Giao diện này giữ sẵn các trạng thái cần cho import hàng trăm file:
                            đã nhập, lỗi, chưa phân loại, nghi trùng và cần đổi tên.
                        </div>
                        <div className="import-action-row">
                            <button className="btn btn-primary btn-lg" onClick={runImport} disabled={busy}>
                                Chọn thư mục
                            </button>
                            <button className="btn btn-lg" onClick={runMultiImport} disabled={busy}>
                                Chọn nhiều PDF
                            </button>
                        </div>
                        <label className="import-ocr-option">
                            <input type="checkbox" checked={autoOcr} onChange={(e) => setAutoOcr(e.target.checked)} />
                            Chạy OCR offline thật sau import
                        </label>
                        <div className="import-check-grid">
                            <span className="badge badge-success">Kiểm tra trùng file</span>
                            <span className="badge badge-info">Gán vào hồ sơ/vụ án</span>
                            <span className="badge badge-warning">Gợi ý đổi tên</span>
                            <span className="badge badge-neutral">Không ghi đè file gốc</span>
                        </div>
                    </div>
                ) : null}

                {busy ? (
                    <div style={{ display: "grid", gap: "var(--space-3)", padding: "var(--space-5)" }}>
                        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: "var(--space-3)" }}>
                            <div className="module-spinner" />
                            <span className="text-secondary">{step === "ocr" ? "Đang OCR offline..." : "Đang import..."}</span>
                        </div>
                        {step === "ocr" && ocrProgress.total > 0 ? (
                            <div className="ocr-progress-panel">
                                <div className="scan-progress-strip">
                                    <div style={{ width: `${Math.round((ocrProgress.done / ocrProgress.total) * 100)}%` }} />
                                </div>
                                <div className="text-xs text-muted">
                                    OCR: {ocrProgress.done}/{ocrProgress.total} · lỗi {ocrProgress.failed} · {ocrProgress.current}
                                </div>
                            </div>
                        ) : null}
                    </div>
                ) : null}
            </div>

            {result ? (
                <div className="card">
                    <div className="card-title">Kết quả Import</div>
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "var(--space-3)", marginBottom: "var(--space-3)" }}>
                        <div>
                            <div className="stat-label">Job ID</div>
                            <div className="text-mono text-sm mt-2">{result.job_id}</div>
                        </div>
                        <div>
                            <div className="stat-label">Tổng file</div>
                            <div className="stat-value">{result.total_files}</div>
                        </div>
                        <div>
                            <div className="stat-label">Case ID</div>
                            <div className="text-mono text-sm mt-2">{result.case_id}</div>
                        </div>
                    </div>
                    {result.files.length > 0 ? (
                        <div style={{ padding: 0, overflow: "hidden", border: "1px solid var(--color-border)", borderRadius: "var(--radius-md)" }}>
                            <table className="data-table">
                                <thead>
                                    <tr>
                                        <th>Tên file</th>
                                        <th style={{ width: 60 }}>Ext</th>
                                        <th style={{ width: 80 }}>Size</th>
                                        <th style={{ width: 50 }}>Pages</th>
                                        <th style={{ width: 130 }}>Trạng thái</th>
                                        <th style={{ width: 150 }}>Gợi ý</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {result.files.map((f) => (
                                        <tr key={f.document_id}>
                                            <td className="truncate" style={{ maxWidth: 300 }}>{f.file_name}</td>
                                            <td><span className="badge badge-neutral">{f.file_ext}</span></td>
                                            <td className="text-muted text-xs">{(f.file_size / 1024).toFixed(0)} KB</td>
                                            <td style={{ textAlign: "center" }}>{f.page_count}</td>
                                            <td><span className="badge badge-success">Đã nhập</span></td>
                                            <td><span className={autoOcr ? "badge badge-info" : "badge badge-warning"}>{autoOcr ? "Đã đưa qua OCR" : "Chờ OCR"}</span></td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    ) : null}
                    <div className="mt-3">
                        <Link className="btn btn-primary" to={`/cases/${result.case_id}`}>
                            Mở hồ sơ vừa import
                        </Link>
                    </div>
                </div>
            ) : null}

            {multiResult ? (
                <div className="card mt-4">
                    <div className="card-title">Kết quả Import nhiều file</div>
                    <div className="import-result-grid mb-3">
                        <div>
                            <div className="stat-label">Tổng file</div>
                            <div className="stat-value">{multiResult.total_files}</div>
                        </div>
                        <div>
                            <div className="stat-label">Thành công</div>
                            <div className="stat-value">{multiResult.imported.length}</div>
                        </div>
                        <div>
                            <div className="stat-label">Trùng</div>
                            <div className="stat-value">{multiResult.duplicates.length}</div>
                        </div>
                        <div>
                            <div className="stat-label">Lỗi</div>
                            <div className="stat-value">{multiResult.errors.length}</div>
                        </div>
                    </div>
                    <div style={{ padding: 0, overflow: "hidden", border: "1px solid var(--color-border)", borderRadius: "var(--radius-md)" }}>
                        <table className="data-table">
                            <thead>
                                <tr>
                                    <th>Tên file</th>
                                    <th style={{ width: 80 }}>Trang</th>
                                    <th style={{ width: 130 }}>Trạng thái</th>
                                    <th>Ghi chú</th>
                                </tr>
                            </thead>
                            <tbody>
                                {multiResult.imported.map((f) => (
                                    <tr key={f.document_id}>
                                        <td className="font-semibold">{f.file_name}</td>
                                        <td>{f.page_count}</td>
                                        <td><span className="badge badge-success">imported</span></td>
                                        <td className="text-xs text-muted">{autoOcr ? "Đã chạy OCR offline nếu Python/PaddleOCR sẵn sàng" : "Chờ OCR/phân loại"}</td>
                                    </tr>
                                ))}
                                {multiResult.duplicates.map((name) => (
                                    <tr key={`dup-${name}`}>
                                        <td className="font-semibold">{name}</td>
                                        <td>-</td>
                                        <td><span className="badge badge-warning">duplicate</span></td>
                                        <td className="text-xs text-muted">Trùng hash hoặc tên file trong hồ sơ đích</td>
                                    </tr>
                                ))}
                                {multiResult.errors.map((message) => (
                                    <tr key={`err-${message}`}>
                                        <td className="font-semibold">{message.split(":")[0]}</td>
                                        <td>-</td>
                                        <td><span className="badge badge-danger">error</span></td>
                                        <td className="text-xs text-muted">{message}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                    <div className="mt-3">
                        <Link className="btn btn-primary" to={`/cases/${multiResult.case_id}`}>
                            Mở hồ sơ vừa import
                        </Link>
                    </div>
                </div>
            ) : null}

            {ocrProgress.results.length > 0 ? (
                <div className="card mt-4">
                    <div className="card-title">Kết quả OCR offline</div>
                    <div className="dashboard-chip-cloud">
                        <span className="badge badge-info">{ocrProgress.results.length} tài liệu đã xử lý</span>
                        <span className={ocrProgress.failed ? "badge badge-warning" : "badge badge-success"}>{ocrProgress.failed} tài liệu lỗi một phần/toàn bộ</span>
                    </div>
                    <div className="mt-3" style={{ display: "grid", gap: "var(--space-2)" }}>
                        {ocrProgress.results.map((item) => (
                            <div className="ocr-result-row" key={item.document_id}>
                                <span className="text-mono text-xs">{item.document_id.slice(0, 18)}</span>
                                <span>{item.processed_pages} trang OCR</span>
                                <span>{item.failed_pages} lỗi</span>
                                <span>{item.handwritten_pages ?? 0} viết tay</span>
                                <span>{Math.round(item.average_confidence * 100)}%</span>
                                <span className={item.status === "processed" ? "badge badge-success" : "badge badge-danger"}>{item.status}</span>
                                <span className="text-xs text-muted">{item.message}</span>
                            </div>
                        ))}
                    </div>
                </div>
            ) : null}
        </PageSection>
    );
}
