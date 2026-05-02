import { useEffect, useMemo, useState } from "react";
import * as documentService from "../../services/documentService";
import * as exportService from "../../services/exportService";
import { PageSection } from "./common";

export function ExportPage() {
    const [documents, setDocuments] = useState<documentService.DocumentSummary[]>([]);
    const [selectedIds, setSelectedIds] = useState<string[]>([]);
    const [coverPage, setCoverPage] = useState(true);
    const [toc, setToc] = useState(true);
    const [pageNumbers, setPageNumbers] = useState(true);
    const [ocrLayer, setOcrLayer] = useState(false);
    const [busy, setBusy] = useState(false);
    const [message, setMessage] = useState<string | null>(null);
    const [result, setResult] = useState<exportService.ExportPdfResult | null>(null);

    useEffect(() => {
        documentService.listDocuments().then(setDocuments);
    }, []);

    const selectedDocs = useMemo(
        () => selectedIds
            .map((id) => documents.find((doc) => doc.document_id === id))
            .filter((doc): doc is documentService.DocumentSummary => Boolean(doc)),
        [documents, selectedIds]
    );

    const toggleDoc = (doc: documentService.DocumentSummary) => {
        setSelectedIds((prev) =>
            prev.includes(doc.document_id)
                ? prev.filter((id) => id !== doc.document_id)
                : [...prev, doc.document_id]
        );
    };

    const move = (docId: string, delta: -1 | 1) => {
        setSelectedIds((prev) => {
            const index = prev.indexOf(docId);
            const nextIndex = index + delta;
            if (index < 0 || nextIndex < 0 || nextIndex >= prev.length) return prev;
            const next = [...prev];
            [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
            return next;
        });
    };

    const runExport = async () => {
        if (selectedIds.length === 0) {
            setMessage("Chưa chọn tài liệu để export.");
            return;
        }

        const { save } = await import("@tauri-apps/plugin-dialog");
        const outputPath = await save({
            title: "Lưu bộ hồ sơ PDF",
            defaultPath: "bo_ho_so_export.pdf",
            filters: [{ name: "PDF", extensions: ["pdf"] }],
        });
        if (!outputPath) return;

        setBusy(true);
        setMessage(null);
        setResult(null);
        const exported = await exportService.exportPdfBundle({
            document_ids: selectedIds,
            output_path: outputPath,
            cover_page: coverPage,
            table_of_contents: toc,
            page_numbers: pageNumbers,
            include_ocr_text: ocrLayer,
        });
        setBusy(false);
        if (!exported) {
            setMessage("Export thất bại. Kiểm tra log Tauri để xem lỗi chi tiết.");
            return;
        }
        setResult(exported);
        setMessage(`Đã xuất PDF: ${exported.output_path}`);
    };

    return (
        <PageSection
            title="Export PDF"
            description="Chọn tài liệu, sắp xếp thứ tự và xuất một bộ hồ sơ PDF hoàn chỉnh"
            right={<button className="btn btn-primary" onClick={runExport} disabled={busy}>{busy ? "Đang xuất..." : "Xuất PDF"}</button>}
        >
            {message ? <div className="module-error mb-3">{message}</div> : null}

            <div className="export-workspace">
                <div className="card export-source-panel">
                    <div className="card-title">Tài liệu nguồn</div>
                    <div className="card-subtitle">Chọn nhiều tài liệu PDF để đưa vào bộ hồ sơ.</div>
                    <div className="export-doc-list">
                        {documents.map((doc) => (
                            <label className="export-doc-row" key={doc.document_id}>
                                <input
                                    type="checkbox"
                                    checked={selectedIds.includes(doc.document_id)}
                                    onChange={() => toggleDoc(doc)}
                                />
                                <span className="truncate">
                                    <b>{doc.display_name}</b>
                                    <small>{doc.document_type} · {doc.page_count} trang</small>
                                </span>
                                <span className="badge badge-neutral">{doc.file_path.toLowerCase().endsWith(".pdf") ? "PDF" : "skip"}</span>
                            </label>
                        ))}
                    </div>
                </div>

                <div className="card export-order-panel">
                    <div className="dashboard-card-header">
                        <div>
                            <div className="card-title">Thứ tự export</div>
                            <div className="card-subtitle">Dùng nút lên/xuống để reorder trước khi xuất.</div>
                        </div>
                        <span className="badge badge-primary">{selectedDocs.length} tài liệu</span>
                    </div>
                    <div className="export-selected-list">
                        {selectedDocs.map((doc, index) => (
                            <div className="export-selected-row" key={doc.document_id}>
                                <span className="text-mono">{String(index + 1).padStart(2, "0")}</span>
                                <span className="truncate">{doc.display_name}</span>
                                <button className="btn btn-sm btn-icon" onClick={() => move(doc.document_id, -1)}>↑</button>
                                <button className="btn btn-sm btn-icon" onClick={() => move(doc.document_id, 1)}>↓</button>
                            </div>
                        ))}
                        {selectedDocs.length === 0 ? <div className="empty-state-text">Chưa chọn tài liệu.</div> : null}
                    </div>
                </div>

                <div className="card export-options-panel">
                    <div className="card-title">Tùy chọn bộ hồ sơ</div>
                    <label className="export-option"><input type="checkbox" checked={coverPage} onChange={(e) => setCoverPage(e.target.checked)} /> Tạo bìa hồ sơ</label>
                    <label className="export-option"><input type="checkbox" checked={toc} onChange={(e) => setToc(e.target.checked)} /> Tạo mục lục/bookmark</label>
                    <label className="export-option"><input type="checkbox" checked={pageNumbers} onChange={(e) => setPageNumbers(e.target.checked)} /> Đánh số trang</label>
                    <label className="export-option"><input type="checkbox" checked={ocrLayer} onChange={(e) => setOcrLayer(e.target.checked)} /> Kèm OCR text layer</label>
                    <div className="text-xs text-muted mt-3">
                        Backend hiện merge PDF gốc và tạo outline mục lục. Bìa/số trang/OCR layer được giữ trong payload để mở rộng layout engine sau.
                    </div>
                    {result ? (
                        <div className="mt-4">
                            <div className="badge badge-success">Export hoàn tất</div>
                            <div className="text-xs text-muted mt-2">{result.output_path}</div>
                            {result.skipped_documents.length > 0 ? (
                                <div className="mt-2">
                                    {result.skipped_documents.map((item) => <div className="badge badge-warning" key={item}>{item}</div>)}
                                </div>
                            ) : null}
                        </div>
                    ) : null}
                </div>
            </div>
        </PageSection>
    );
}
