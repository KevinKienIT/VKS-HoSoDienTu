import { useCallback, useEffect, useState } from "react";
import { useLocation, useNavigate, useParams } from "react-router-dom";
import { DocumentViewer } from "../DocumentViewer";
import * as caseService from "../../services/caseService";
import * as documentService from "../../services/documentService";
import * as aiService from "../../services/aiService";
import { TypeIcon } from "./common";

interface ChatMessage {
    role: "user" | "ai";
    text: string;
    sources?: aiService.AiSource[];
    mode?: string;
}

export function AiWorkspacePage() {
    const { caseId = "" } = useParams();
    const navigate = useNavigate();
    const location = useLocation();
    const [cases, setCases] = useState<caseService.CaseSummary[]>([]);
    const [caseInfo, setCaseInfo] = useState<caseService.CaseSummary | null>(null);
    const [documents, setDocuments] = useState<documentService.DocumentSummary[]>([]);
    const [selectedDoc, setSelectedDoc] = useState<documentService.DocumentSummary | null>(null);
    const [currentPage, setCurrentPage] = useState(1);
    const [messages, setMessages] = useState<ChatMessage[]>([]);
    const [question, setQuestion] = useState("");
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [notice, setNotice] = useState<string | null>(null);
    const [leftPanel, setLeftPanel] = useState<"full" | "compact" | "collapsed">("full");
    const [rightPanel, setRightPanel] = useState<"full" | "compact" | "collapsed">("full");

    useEffect(() => {
        if (!caseId) {
            caseService.listCases()
                .then(setCases)
                .catch((e) => setError((e as Error).message));
            return;
        }
        Promise.all([caseService.getCase(caseId), documentService.listDocuments(caseId)])
            .then(([c, docs]) => {
                setCaseInfo(c);
                setDocuments(docs);
                setSelectedDoc(docs[0] ?? null);
                setCurrentPage(1);
                setMessages([]);
            })
            .catch((e) => setError((e as Error).message));
    }, [caseId]);

    useEffect(() => {
        const state = location.state as { draftQuestion?: string; documentId?: string; pageNumber?: number } | null;
        if (!state?.draftQuestion) return;
        setQuestion(state.draftQuestion);
        if (state.documentId) {
            const doc = documents.find((item) => item.document_id === state.documentId);
            if (doc) {
                setSelectedDoc(doc);
                setCurrentPage(state.pageNumber || 1);
            }
        }
    }, [documents, location.state]);

    const handleAsk = useCallback(async () => {
        if (busy || !question.trim()) return;
        const q = question.trim();
        setQuestion("");
        setMessages((prev) => [...prev, { role: "user", text: q }]);
        setBusy(true);
        setError(null);
        try {
            const isSum = q.includes("tóm tắt") || q.includes("Tóm tắt");
            const result = isSum ? await aiService.summarizeCase(caseId) : await aiService.askCase(caseId, q);
            if (!result) throw new Error("AI trả về null");
            setMessages((prev) => [...prev, { role: "ai", text: result.answer, sources: result.sources, mode: result.mode }]);
        } catch (e) { setError((e as Error).message); }
        finally { setBusy(false); }
    }, [caseId, question]);

    const quickQs = ["Tóm tắt hồ sơ", "Trang nào có quyết định khởi tố?", "Liệt kê mốc thời gian quan trọng", "Ai liên quan trong hồ sơ?"];

    if (!caseId) {
        return (
            <div className="ai-case-picker">
                <div className="page-header">
                    <div>
                        <h1 className="page-title">Phân tích AI</h1>
                        <p className="page-description">Chọn một hồ sơ để mở workspace 3 vùng: tài liệu, viewer và AI Notebook có dẫn nguồn.</p>
                    </div>
                </div>
                {error ? <div className="module-error mb-3">{error}</div> : null}
                <div className="doc-card-grid">
                    {cases.map((item) => (
                        <button key={item.case_id} className="ai-case-card card card-interactive" onClick={() => navigate(`/ai/${item.case_id}`)}>
                            <div className="stat-label">{item.case_code}</div>
                            <div className="ai-case-title">{item.case_display_name}</div>
                            <div className="doc-card-meta">
                                <span>{item.document_count} tài liệu</span>
                                <span>{item.total_pages} trang</span>
                                <span className="badge badge-success">{item.status}</span>
                            </div>
                        </button>
                    ))}
                    {cases.length === 0 ? (
                        <div className="card">
                            <div className="empty-state-text">Chưa có hồ sơ. Import hồ sơ trước khi dùng AI Notebook.</div>
                        </div>
                    ) : null}
                </div>
            </div>
        );
    }

    return (
        <div className={`ai-workspace left-${leftPanel} right-${rightPanel}`}>
            {/* Left — doc list */}
            <div className="ai-workspace-left">
                <div className="ai-ws-header">
                    <span className="font-semibold truncate">{caseInfo?.case_display_name ?? "Hồ sơ"}</span>
                    <div style={{ display: "flex", gap: 6 }}>
                        <button className="btn btn-sm btn-ghost" onClick={() => setLeftPanel((s) => s === "full" ? "compact" : s === "compact" ? "collapsed" : "full")}>L</button>
                        <button className="btn btn-sm btn-ghost" onClick={() => navigate(-1)}>←</button>
                    </div>
                </div>
                <div className="ai-ws-doclist">
                    {documents.map((doc) => (
                        <div key={doc.document_id}
                            className={`ai-ws-docitem ${selectedDoc?.document_id === doc.document_id ? "active" : ""}`}
                            onClick={() => { setSelectedDoc(doc); setCurrentPage(1); }}>
                            <TypeIcon docType={doc.document_type} />
                            <div style={{ flex: 1, minWidth: 0 }}>
                                <div className="truncate text-sm font-semibold">{doc.display_name}</div>
                                <div className="text-xs text-muted">{doc.page_count} trang</div>
                            </div>
                        </div>
                    ))}
                    {documents.length === 0 ? <div className="text-muted text-xs p-4">Chưa có tài liệu.</div> : null}
                </div>
                <div className="ai-ws-toc">
                    <div className="stat-label" style={{ padding: "var(--space-2) var(--space-3)" }}>Mục lục</div>
                    {documents.map((doc, i) => (
                        <div key={doc.document_id} className="ai-ws-tocitem" onClick={() => { setSelectedDoc(doc); setCurrentPage(1); }}>
                            <span className="text-mono text-xs">{String(i + 1).padStart(2, "0")}.</span>
                            <span className="truncate text-xs">{doc.display_name}</span>
                        </div>
                    ))}
                </div>
            </div>

            {/* Center — PDF */}
            <div className="ai-workspace-center">
                {selectedDoc ? (
                    <DocumentViewer
                        documentPath={selectedDoc.file_path}
                        documentType={selectedDoc.file_path.split(".").pop() || "unknown"}
                        initialPage={currentPage}
                        onPageChange={setCurrentPage}
                        onAddSelectionToAi={(text, page) => {
                            setQuestion(`Phân tích đoạn trích ở trang ${page}:\n\n${text}`);
                            setNotice("Đã đưa đoạn chọn vào AI Notebook.");
                        }}
                        onCreateNoteFromSelection={async (text, page) => {
                            const note = await documentService.createNoteFromSelection(caseId, selectedDoc.document_id, page, text);
                            setNotice(note ? `Đã tạo ghi chú: ${note.title}` : "Không thể tạo ghi chú.");
                        }}
                    />
                ) : (
                    <div className="viewer-empty-state"><div className="empty-state-icon">📄</div><div>Chọn tài liệu bên trái</div></div>
                )}
            </div>

            {/* Right — AI chat */}
            <div className="ai-workspace-right">
                <div className="ai-ws-header">
                    <span className="font-semibold">🤖 AI Hỏi Đáp</span>
                    <button className="btn btn-sm btn-ghost" onClick={() => setRightPanel((s) => s === "full" ? "compact" : s === "compact" ? "collapsed" : "full")}>R</button>
                </div>
                {error ? <div className="module-error" style={{ margin: "var(--space-2)", fontSize: "var(--text-xs)" }}>⚠️ {error}</div> : null}
                {notice ? <div className="toast-inline" style={{ margin: "var(--space-2)", fontSize: "var(--text-xs)" }}>{notice}</div> : null}
                <div className="ai-ws-chat">
                    {messages.length === 0 ? (
                        <div style={{ padding: "var(--space-4)" }}>
                            <div className="text-muted text-sm mb-3">Hỏi AI về nội dung hồ sơ. Câu trả lời dẫn nguồn theo tài liệu và trang.</div>
                            <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--space-1)" }}>
                                {quickQs.map((q) => (
                                    <button key={q} className="btn btn-sm btn-ghost" style={{ fontSize: "var(--text-xs)" }} onClick={() => setQuestion(q)}>{q}</button>
                                ))}
                            </div>
                        </div>
                    ) : (
                        messages.map((msg, i) => (
                            <div key={i} className={`ai-msg ${msg.role}`}>
                                {msg.role === "user" ? (
                                    <div className="ai-msg-user">{msg.text}</div>
                                ) : (
                                    <div className="ai-msg-ai">
                                        {msg.mode ? <span className="badge badge-primary mb-1" style={{ fontSize: 10 }}>{msg.mode}</span> : null}
                                        <pre className="ai-msg-answer">{msg.text}</pre>
                                        {msg.sources && msg.sources.length > 0 ? (
                                            <div className="ai-msg-sources">
                                                <div className="stat-label mb-1">Nguồn</div>
                                                {msg.sources.map((src, si) => (
                                                    <div key={si} className="ai-msg-srcitem" onClick={() => {
                                                        const doc = documents.find((d) => d.document_id === src.document_id || d.document_id === src.source_id || d.display_name === src.label);
                                                        if (doc) {
                                                            setSelectedDoc(doc);
                                                            setCurrentPage(src.page_number || 1);
                                                        }
                                                    }}>
                                                        <div className="font-semibold text-xs">{src.label}</div>
                                                        {src.page_number ? <div className="text-xs text-muted">Trang {src.page_number}</div> : null}
                                                        {src.excerpt ? <div className="text-xs" style={{ opacity: 0.7 }}>"{src.excerpt.slice(0, 100)}"</div> : null}
                                                    </div>
                                                ))}
                                            </div>
                                        ) : null}
                                    </div>
                                )}
                            </div>
                        ))
                    )}
                </div>
                <div className="ai-ws-input">
                    <textarea className="form-textarea" value={question} onChange={(e) => setQuestion(e.target.value)}
                        placeholder="Hỏi về hồ sơ..." rows={2} style={{ resize: "none" }}
                        onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleAsk(); } }} />
                    <button className="btn btn-primary btn-sm" onClick={handleAsk} disabled={busy || !question.trim()}>{busy ? "..." : "Gửi"}</button>
                </div>
            </div>
        </div>
    );
}
