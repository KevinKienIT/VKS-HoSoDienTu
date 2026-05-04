import { useEffect, useState } from "react";
import * as aiService from "../phantichai/phantichai.service";

export function AiNotebookPanel({ caseId }: { caseId: string }) {
    const [status, setStatus] = useState<aiService.AiStatus | null>(null);
    const [question, setQuestion] = useState("");
    const [answer, setAnswer] = useState<aiService.AiAnswer | null>(null);
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        aiService.checkAiStatus().then(setStatus);
    }, []);

    const runSummary = async () => {
        setBusy(true);
        setError(null);
        try {
            const result = await aiService.summarizeCase(caseId);
            if (!result) {
                throw new Error("AI command trả về null.");
            }
            setAnswer(result);
        } catch (e) {
            setError((e as Error).message || "Không thể tóm tắt hồ sơ.");
        } finally {
            setBusy(false);
        }
    };

    const runAsk = async () => {
        if (!question.trim()) return;
        setBusy(true);
        setError(null);
        try {
            const result = await aiService.askCase(caseId, question);
            if (!result) {
                throw new Error("AI command trả về null.");
            }
            setAnswer(result);
        } catch (e) {
            setError((e as Error).message || "Không thể hỏi AI offline.");
        } finally {
            setBusy(false);
        }
    };

    return (
        <div className="card" style={{ height: "100%" }}>
            <div className="card-title">AI Notebook</div>
            <div className="mb-3" style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
                <span className={`badge ${status?.available ? "badge-success" : "badge-neutral"}`}>
                    {status?.mode ?? "checking"}
                </span>
                <span className="text-muted text-xs">{status?.message ?? "Kiểm tra..."}</span>
            </div>

            {error ? <div className="module-error mb-3">⚠️ {error}</div> : null}

            <div className="mb-3">
                <button className="btn btn-primary btn-sm" onClick={runSummary} disabled={busy}>
                    {busy ? "Đang chạy..." : "Tóm tắt hồ sơ"}
                </button>
            </div>

            <div className="mb-3" style={{ display: "grid", gap: "var(--space-2)" }}>
                <textarea
                    className="form-textarea"
                    value={question}
                    onChange={(e) => setQuestion(e.target.value)}
                    placeholder="Hỏi theo hồ sơ/notes/OCR đã ingest..."
                    style={{ minHeight: 80, resize: "vertical" }}
                />
                <button className="btn btn-sm" onClick={runAsk} disabled={busy || !question.trim()}>
                    Hỏi AI offline
                </button>
            </div>

            {!answer ? (
                <div className="text-muted text-xs" style={{ lineHeight: "var(--leading-relaxed)" }}>
                    AI đọc ghi chú, metadata, OCR text từ SQLite. Không có dữ liệu rời khỏi máy.
                </div>
            ) : (
                <div className="fade-in">
                    <div className="badge badge-primary mb-2">{answer.mode}</div>
                    <pre style={{
                        whiteSpace: "pre-wrap",
                        fontFamily: "var(--font-sans)",
                        fontSize: "var(--text-sm)",
                        lineHeight: "var(--leading-relaxed)",
                        background: "var(--color-bg-muted)",
                        padding: "var(--space-3)",
                        borderRadius: "var(--radius-md)",
                        border: "1px solid var(--color-border-light)",
                    }}>
                        {answer.answer}
                    </pre>
                    {answer.sources.length > 0 ? (
                        <div className="mt-3">
                            <div className="stat-label mb-2">Nguồn tham chiếu</div>
                            <div style={{ display: "grid", gap: "var(--space-1_5)" }}>
                                {answer.sources.map((source) => (
                                    <div
                                        key={`${source.source_type}:${source.source_id}`}
                                        style={{
                                            padding: "var(--space-2) var(--space-3)",
                                            background: "var(--color-bg-muted)",
                                            borderRadius: "var(--radius-sm)",
                                            border: "1px solid var(--color-border-light)",
                                        }}
                                    >
                                        <div className="font-semibold text-sm">{source.label}</div>
                                        <div className="text-muted text-xs">{source.source_type} · {source.source_id}</div>
                                        <div className="text-sm mt-2" style={{ lineHeight: "var(--leading-relaxed)" }}>
                                            {source.excerpt}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ) : null}
                </div>
            )}
        </div>
    );
}
