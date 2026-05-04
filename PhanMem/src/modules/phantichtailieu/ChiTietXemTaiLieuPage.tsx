import { useEffect, useMemo, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { DocumentViewer } from "./TrinhXemTaiLieu";
import * as documentService from "../quantailieu/quantailieu.service";
import { PageSection, SmallBackToCase } from "./common";

export function DocumentViewerPage() {
    const { caseId = "", docId = "" } = useParams();
    const [searchParams] = useSearchParams();
    const navigate = useNavigate();
    const [document, setDocument] = useState<documentService.DocumentSummary | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [ocrPanelMode, setOcrPanelMode] = useState<"split" | "viewer-only" | "ocr-only">("split");
    const initialPage = Math.max(1, Number(searchParams.get("page") ?? "1") || 1);
    const highlightQuery = searchParams.get("q") ?? "";
    const [currentPage, setCurrentPage] = useState(initialPage);
    const [ocrData, setOcrData] = useState<documentService.PageOcrResult | null>(null);
    const [ocrDraft, setOcrDraft] = useState("");
    const [ocrLoading, setOcrLoading] = useState(false);
    const [savingOcr, setSavingOcr] = useState(false);
    const [openedTabs, setOpenedTabs] = useState<documentService.DocumentSummary[]>([]);
    const [notice, setNotice] = useState<string | null>(null);
    const [showOcrOverlay, setShowOcrOverlay] = useState(true);
    const [ocrBlocks, setOcrBlocks] = useState<documentService.PageLayoutBlock[]>([]);
    const [activeBlockId, setActiveBlockId] = useState<string | null>(null);
    const [extractedFields, setExtractedFields] = useState<documentService.ExtractedField[]>([]);
    const [rightPanelTab, setRightPanelTab] = useState<"ocr" | "fields">("ocr");
    const [rescanModalOpen, setRescanModalOpen] = useState(false);
    const [processingAction, setProcessingAction] = useState<string | null>(null);
    const [rescanOptions, setRescanOptions] = useState<documentService.RescanOcrOptions>({
        scope: "current",
        quality: "fast",
        mode: "auto",
        detectLayout: true,
        detectMarks: true,
        rebuildIndex: true,
    });

    useEffect(() => {
        if (!docId) return;
        setLoading(true);
        setError(null);
        documentService
            .getDocument(docId)
            .then((data) => {
                setDocument(data);
                if (data) {
                    setOpenedTabs((prev) => prev.some((item) => item.document_id === data.document_id) ? prev : [...prev, data]);
                }
            })
            .catch((e) => setError((e as Error).message || "Lỗi khi tải tài liệu."))
            .finally(() => setLoading(false));
    }, [docId]);

    useEffect(() => {
        setCurrentPage(initialPage);
    }, [initialPage]);

    useEffect(() => {
        if (!docId || ocrPanelMode !== "split") return;
        setOcrLoading(true);
        documentService
            .getPageOcr(docId, currentPage)
            .then((data) => {
                setOcrData(data);
                setOcrDraft(data?.ocr_formatted_text || data?.ocr_text || "");
            })
            .catch((e) => setError((e as Error).message || "Lỗi tải OCR theo trang."))
            .finally(() => setOcrLoading(false));
    }, [currentPage, docId, ocrPanelMode]);

    const loadAnalysis = async () => {
        if (!docId) return;
        const [blocks, fields] = await Promise.all([
            documentService.listPageLayoutBlocks(docId, currentPage),
            documentService.listDocumentExtractedFields(docId),
        ]);
        setOcrBlocks(blocks);
        setExtractedFields(fields);
    };

    useEffect(() => {
        loadAnalysis().catch((e) => setError((e as Error).message || "Lỗi tải OCR layout."));
    }, [currentPage, docId]);

    useEffect(() => {
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

            if (e.key === "Escape") {
                navigate(`/cases/${caseId}`);
            }
            if (e.key === "g" || e.key === "G") {
                setOcrPanelMode((prev) => (prev === "split" ? "viewer-only" : "split"));
            }
        };
        window.addEventListener("keydown", onKeyDown);
        return () => window.removeEventListener("keydown", onKeyDown);
    }, [caseId, navigate]);

    const title = useMemo(() => {
        if (document) return `Viewer: ${document.display_name}`;
        return "Document Viewer";
    }, [document]);

    const saveOcr = async () => {
        if (!docId) return;
        setSavingOcr(true);
        setError(null);
        try {
            const saved = await documentService.updatePageOcr(docId, currentPage, ocrDraft);
            setOcrData(saved);
            setOcrDraft(saved?.ocr_formatted_text || saved?.ocr_text || ocrDraft);
        } catch (e) {
            setError((e as Error).message || "Không thể lưu OCR.");
        } finally {
            setSavingOcr(false);
        }
    };

    const reloadCurrentOcr = async () => {
        if (!docId) return;
        const data = await documentService.getPageOcr(docId, currentPage);
        setOcrData(data);
        setOcrDraft(data?.ocr_formatted_text || data?.ocr_text || "");
        await loadAnalysis();
    };

    const runRescan = async (options: documentService.RescanOcrOptions, page?: number) => {
        if (!docId) return;
        setProcessingAction("rescan");
        setError(null);
        setNotice(null);
        try {
            const result = await documentService.rescanDocumentOcr(docId, page, options);
            if (!result) {
                setError("Rescan OCR thất bại. Kiểm tra Python/RapidOCR/PaddleOCR/PyMuPDF.");
                return;
            }
            setNotice(`${result.message} Confidence ${Math.round(result.average_confidence * 100)}%.`);
            await reloadCurrentOcr();
        } catch (e) {
            setError((e as Error).message || "Không thể rescan PDF.");
        } finally {
            setProcessingAction(null);
            setRescanModalOpen(false);
        }
    };

    const runAiAnalysis = async () => {
        if (!docId) return;
        setProcessingAction("ai");
        setError(null);
        try {
            const result = await documentService.analyzeDocumentWithAiAgent(docId);
            setNotice(result ? `Đã lưu phân tích AI offline: ${result.title}` : "Phân tích AI offline thất bại.");
            await loadAnalysis();
        } catch (e) {
            setError((e as Error).message || "Không thể phân tích AI offline.");
        } finally {
            setProcessingAction(null);
        }
    };

    const rebuildIndex = async () => {
        setProcessingAction("index");
        setError(null);
        try {
            const result = await documentService.rebuildTextIndex();
            setNotice(result ? `Đã rebuild index: ${result.indexed_documents} tài liệu, ${result.indexed_pages} trang.` : "Rebuild index thất bại.");
        } catch (e) {
            setError((e as Error).message || "Không thể rebuild index.");
        } finally {
            setProcessingAction(null);
        }
    };

    const openFieldLocation = (field: documentService.ExtractedField) => {
        setCurrentPage(field.page_number);
        setShowOcrOverlay(true);
        const match = ocrBlocks.find((block) =>
            Math.abs(block.x - field.x) < 4 &&
            Math.abs(block.y - field.y) < 4 &&
            Math.abs(block.width - field.width) < 8
        );
        setActiveBlockId(match?.id ?? null);
    };

    const createNoteFromSelection = async (selectedText: string, page: number) => {
        if (!caseId || !docId) return;
        const note = await documentService.createNoteFromSelection(caseId, docId, page, selectedText);
        setNotice(note ? `Đã tạo ghi chú: ${note.title}` : "Không thể tạo ghi chú từ đoạn đã chọn.");
    };

    return (
        <PageSection title={title} description="Xem tài liệu + panel OCR/Citation theo chế độ side-by-side">
            <SmallBackToCase caseId={caseId} />

            <div className="toolbar">
                <div className="viewer-doc-tabs">
                    {openedTabs.map((tab) => (
                        <button
                            key={tab.document_id}
                            className={`viewer-doc-tab ${tab.document_id === docId ? "active" : ""}`}
                            onClick={() => navigate(`/cases/${tab.case_id}/docs/${tab.document_id}`)}
                        >
                            <span className="truncate">{tab.display_name}</span>
                            <span
                                className="viewer-doc-tab-close"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setOpenedTabs((prev) => prev.filter((item) => item.document_id !== tab.document_id));
                                }}
                            >
                                ×
                            </span>
                        </button>
                    ))}
                </div>
                <div className="toolbar-separator" />
                <button className="btn btn-sm btn-primary" onClick={() => setRescanModalOpen(true)} disabled={!!processingAction}>
                    Rescan PDF
                </button>
                <button
                    className="btn btn-sm"
                    onClick={() => runRescan({ scope: "current", quality: "fast", mode: "auto", detectLayout: true, detectMarks: true, rebuildIndex: true }, currentPage)}
                    disabled={!!processingAction}
                >
                    OCR trang hiện tại
                </button>
                <button
                    className="btn btn-sm"
                    onClick={() => runRescan({ scope: "all", quality: "fast", mode: "auto", detectLayout: true, detectMarks: true, rebuildIndex: true })}
                    disabled={!!processingAction}
                >
                    OCR toàn bộ
                </button>
                <button className="btn btn-sm" onClick={runAiAnalysis} disabled={!!processingAction}>
                    Phân tích AI offline
                </button>
                <button className={`btn btn-sm ${showOcrOverlay ? "btn-primary" : "btn-ghost"}`} onClick={() => setShowOcrOverlay((v) => !v)}>
                    Đánh dấu vùng lỗi
                </button>
                <button
                    className="btn btn-sm"
                    onClick={() => runRescan({ scope: "current", quality: "high", mode: "auto", detectLayout: true, detectMarks: true, rebuildIndex: true }, currentPage)}
                    disabled={!!processingAction}
                >
                    Trích xuất cấu trúc
                </button>
                <button className="btn btn-sm" onClick={rebuildIndex} disabled={!!processingAction}>
                    Rebuild Index
                </button>
                <div className="toolbar-separator" />
                <button
                    className={`btn btn-sm ${ocrPanelMode === "split" ? "btn-primary" : "btn-ghost"}`}
                    onClick={() => setOcrPanelMode("split")}
                >
                    Split
                </button>
                <button
                    className={`btn btn-sm ${ocrPanelMode === "viewer-only" ? "btn-primary" : "btn-ghost"}`}
                    onClick={() => setOcrPanelMode("viewer-only")}
                >
                    PDF only
                </button>
                <button
                    className={`btn btn-sm ${ocrPanelMode === "ocr-only" ? "btn-primary" : "btn-ghost"}`}
                    onClick={() => setOcrPanelMode("ocr-only")}
                >
                    OCR only
                </button>
                <div className="toolbar-separator" />
                <span className="text-muted text-xs">{processingAction ? "Đang xử lý..." : "Esc: back · G: toggle panel"}</span>
            </div>

            {rescanModalOpen ? (
                <div className="modal-backdrop" onClick={() => setRescanModalOpen(false)}>
                    <div className="rescan-modal card" onClick={(e) => e.stopPropagation()}>
                        <div className="dashboard-card-header">
                            <div>
                                <div className="card-title">Rescan PDF</div>
                                <div className="card-subtitle">OCR offline thật, lưu tọa độ, field và rebuild index.</div>
                            </div>
                            <button className="btn btn-icon" onClick={() => setRescanModalOpen(false)}>×</button>
                        </div>
                        <div className="rescan-options-grid">
                            <label><input type="radio" checked={rescanOptions.scope === "current"} onChange={() => setRescanOptions((p) => ({ ...p, scope: "current" }))} /> Rescan trang hiện tại</label>
                            <label><input type="radio" checked={rescanOptions.scope === "all"} onChange={() => setRescanOptions((p) => ({ ...p, scope: "all" }))} /> Rescan toàn bộ PDF</label>
                            <label><input type="radio" checked={rescanOptions.quality === "fast"} onChange={() => setRescanOptions((p) => ({ ...p, quality: "fast" }))} /> OCR nhanh 300 DPI</label>
                            <label><input type="radio" checked={rescanOptions.quality === "high"} onChange={() => setRescanOptions((p) => ({ ...p, quality: "high" }))} /> OCR chính xác cao 400 DPI</label>
                            <label><input type="checkbox" checked={rescanOptions.detectLayout ?? true} onChange={(e) => setRescanOptions((p) => ({ ...p, detectLayout: e.target.checked }))} /> OCR + phát hiện layout</label>
                            <label><input type="checkbox" checked={rescanOptions.detectMarks ?? true} onChange={(e) => setRescanOptions((p) => ({ ...p, detectMarks: e.target.checked }))} /> OCR + phân tích dấu/chữ ký/bút lục</label>
                            <label><input type="checkbox" checked={rescanOptions.rebuildIndex ?? true} onChange={(e) => setRescanOptions((p) => ({ ...p, rebuildIndex: e.target.checked }))} /> OCR + rebuild search index</label>
                            <label>
                                Mode
                                <select className="form-select mt-2" value={rescanOptions.mode ?? "auto"} onChange={(e) => setRescanOptions((p) => ({ ...p, mode: e.target.value as documentService.RescanOcrOptions["mode"] }))}>
                                    <option value="auto">Tự động</option>
                                    <option value="printed">Text đánh máy</option>
                                    <option value="handwritten">Ưu tiên viết tay</option>
                                </select>
                            </label>
                        </div>
                        <div className="viewer-ocr-actions">
                            <button
                                className="btn btn-primary"
                                onClick={() => runRescan(rescanOptions, rescanOptions.scope === "current" ? currentPage : undefined)}
                                disabled={!!processingAction}
                            >
                                {processingAction === "rescan" ? "Đang rescan..." : "Bắt đầu Rescan"}
                            </button>
                            <button className="btn" onClick={() => setRescanModalOpen(false)}>Hủy</button>
                        </div>
                    </div>
                </div>
            ) : null}

            {error ? <div className="module-error">⚠️ {error}</div> : null}
            {document?.file_missing ? (
                <div className="module-error mb-3">
                    ⚠️ Không tìm thấy file gốc trong thư mục dữ liệu phần mềm. Tài liệu này cần được khôi phục/import lại trước khi xem, OCR hoặc export.
                </div>
            ) : null}
            {notice ? <div className="toast-inline mb-3">{notice}</div> : null}
            {loading ? <div className="card">Đang tải tài liệu...</div> : null}

            {!loading && document && !document.file_missing ? (
                <div
                    style={{
                        display: "grid",
                        gridTemplateColumns: ocrPanelMode === "split" ? "minmax(0, 2fr) minmax(320px, 1fr)" : "1fr",
                        gap: "var(--space-3)",
                        minHeight: 520,
                    }}
                >
                    {ocrPanelMode !== "ocr-only" ? (
                        <div className="card" style={{ minHeight: 520, overflow: "hidden", padding: 0 }}>
                            <DocumentViewer
                                documentPath={document.file_path}
                                documentType={document.file_path.split('.').pop() || 'unknown'}
                                initialPage={initialPage}
                                highlightQuery={highlightQuery}
                                onPageChange={setCurrentPage}
                                ocrBlocks={ocrBlocks}
                                showOcrOverlay={showOcrOverlay}
                                activeBlockId={activeBlockId}
                                onOcrBlockSelect={(block) => {
                                    setActiveBlockId(block.id);
                                    setRightPanelTab("ocr");
                                    setOcrDraft(block.text || ocrDraft);
                                }}
                                onAddSelectionToAi={(text, page) => {
                                    navigate(`/ai/${caseId}`, {
                                        state: {
                                            draftQuestion: `Phân tích đoạn trích ở trang ${page}:\n\n${text}`,
                                            documentId: docId,
                                            pageNumber: page,
                                        },
                                    });
                                }}
                                onCreateNoteFromSelection={createNoteFromSelection}
                            />
                        </div>
                    ) : null}

                    {ocrPanelMode === "split" || ocrPanelMode === "ocr-only" ? (
                        <div className="card" style={{ minHeight: 520 }}>
                            <div className="dashboard-card-header">
                                <div>
                                    <div className="card-title">{rightPanelTab === "ocr" ? `OCR — Trang ${currentPage}` : "Thông tin trích xuất"}</div>
                                    <div className="card-subtitle">Trang {currentPage} · OCR engine: {ocrData?.engine ?? "pending"} · {ocrBlocks.length} vùng</div>
                                </div>
                                {ocrData?.confidence != null ? (
                                    <span className={`badge ${ocrData.confidence >= 0.8 ? "badge-success" : ocrData.confidence >= 0.6 ? "badge-warning" : "badge-danger"}`}>
                                        {Math.round(ocrData.confidence * 100)}%
                                    </span>
                                ) : (
                                    <span className="badge badge-neutral">No OCR</span>
                                )}
                            </div>
                            <div className="segmented-control mb-3">
                                <button className={rightPanelTab === "ocr" ? "active" : ""} onClick={() => setRightPanelTab("ocr")}>OCR text</button>
                                <button className={rightPanelTab === "fields" ? "active" : ""} onClick={() => setRightPanelTab("fields")}>Thông tin trích xuất</button>
                            </div>
                            {rightPanelTab === "ocr" ? (
                                <>
                                    {ocrLoading ? (
                                        <div className="viewer-empty-state">Đang tải OCR...</div>
                                    ) : (
                                        <textarea
                                            className="ocr-edit-textarea"
                                            value={ocrDraft}
                                            onChange={(e) => setOcrDraft(e.target.value)}
                                            placeholder="Chưa có OCR text cho trang này. Bấm Rescan PDF/OCR trang hiện tại để đọc lại."
                                        />
                                    )}
                                    <div className="viewer-ocr-actions">
                                        <button className="btn btn-primary btn-sm" onClick={saveOcr} disabled={savingOcr || ocrLoading}>
                                            {savingOcr ? "Đang lưu..." : "Lưu OCR"}
                                        </button>
                                        <span className="text-xs text-muted">Lưu vào pages/ocr_results và rebuild FTS pages.</span>
                                    </div>
                                    {ocrData?.but_luc ? (
                                        <div className="mt-3">
                                            <span className="badge badge-primary">Bút lục: {ocrData.but_luc}</span>
                                        </div>
                                    ) : null}
                                    <div className="mt-3 text-xs text-muted">
                                        Trạng thái transcription: {ocrData?.transcription_state ?? "chưa có page row"}
                                    </div>
                                </>
                            ) : (
                                <div className="extracted-field-table-wrap">
                                    <table className="data-table extracted-field-table">
                                        <thead>
                                            <tr>
                                                <th>Loại thông tin</th>
                                                <th>Giá trị</th>
                                                <th>Trang</th>
                                                <th>Độ tin cậy</th>
                                                <th>Nguồn</th>
                                                <th>Hành động</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {extractedFields.length === 0 ? (
                                                <tr><td colSpan={6} className="text-muted text-xs">Chưa có field. Bấm Rescan PDF hoặc Trích xuất cấu trúc.</td></tr>
                                            ) : extractedFields.map((field) => (
                                                <tr key={field.id}>
                                                    <td>{field.field_name}</td>
                                                    <td className="font-semibold">{field.field_value}</td>
                                                    <td>Trang {field.page_number}</td>
                                                    <td>{Math.round(field.confidence * 100)}%</td>
                                                    <td>{field.source}</td>
                                                    <td><button className="btn btn-sm" onClick={() => openFieldLocation(field)}>Mở vị trí</button></td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            )}
                        </div>
                    ) : null}
                </div>
            ) : null}
        </PageSection>
    );
}
