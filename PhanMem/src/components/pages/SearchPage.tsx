import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import * as searchService from "../../services/searchService";
import { PdfPageThumbnail } from "../PdfPageThumbnail";
import { PageSection } from "./common";

type SortBy = "relevance" | "date" | "name";

/** Escape HTML to prevent XSS, then wrap FTS highlight markers with <mark> */
function safeSnippet(raw: string): string {
    const escaped = raw
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
    // FTS5 snippet uses <b>...</b> markers — restore them as <mark>
    return escaped
        .replace(/&lt;b&gt;/g, "<mark>")
        .replace(/&lt;\/b&gt;/g, "</mark>");
}

export function SearchPage() {
    const navigate = useNavigate();
    const [params] = useSearchParams();
    const [keyword, setKeyword] = useState(params.get("q") ?? "");
    const [results, setResults] = useState<searchService.SearchResult[]>([]);
    const [documentTypeFilter, setDocumentTypeFilter] = useState("all");
    const [sortBy, setSortBy] = useState<SortBy>("relevance");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleSearch = useCallback(async () => {
        if (!keyword.trim()) {
            setResults([]);
            return;
        }
        setLoading(true);
        setError(null);
        try {
            const data = await searchService.ftsSearch(keyword);
            setResults(data);
        } catch (e) {
            setError((e as Error).message || "Lỗi khi tìm kiếm.");
        } finally {
            setLoading(false);
        }
    }, [keyword]);

    useEffect(() => {
        if (keyword.trim()) {
            handleSearch();
        }
    }, []); // intentional one-shot for initial query

    const normalizedTypes = useMemo(() => {
        const types = new Set<string>();
        for (const r of results) {
            if (r.document_type?.trim()) {
                types.add(r.document_type.trim());
            }
        }
        return Array.from(types).sort((a, b) => a.localeCompare(b, "vi"));
    }, [results]);

    const displayedResults = useMemo(() => {
        const filtered = results.filter((item) => {
            if (documentTypeFilter === "all") return true;
            return item.document_type === documentTypeFilter;
        });

        const sorted = [...filtered];
        if (sortBy === "name") {
            sorted.sort((a, b) => a.display_name.localeCompare(b.display_name, "vi"));
        } else if (sortBy === "date") {
            sorted.sort((a, b) => b.created_at.localeCompare(a.created_at));
        } else {
            sorted.sort((a, b) => b.score - a.score);
        }
        return sorted;
    }, [results, documentTypeFilter, sortBy]);

    return (
        <PageSection title="Tìm kiếm" description="Toàn văn FTS5 — tài liệu, OCR text, metadata">
            <div className="toolbar mb-4" style={{ gap: "var(--space-2)" }}>
                <input
                    className="form-input"
                    type="text"
                    value={keyword}
                    onChange={(e) => setKeyword(e.target.value)}
                    onKeyDown={(e) => {
                        if (e.key === "Enter") handleSearch();
                    }}
                    placeholder="Nhập nội dung cần tìm..."
                    style={{ flex: 1 }}
                />
                <button className="btn btn-primary" onClick={handleSearch} disabled={loading}>
                    {loading ? "Đang tìm..." : "⌕ Tìm kiếm"}
                </button>
                <div className="toolbar-separator" />
                <select
                    className="form-select"
                    value={documentTypeFilter}
                    onChange={(e) => setDocumentTypeFilter(e.target.value)}
                    style={{ width: 180 }}
                >
                    <option value="all">Tất cả loại</option>
                    {normalizedTypes.map((t) => (
                        <option key={t} value={t}>{t}</option>
                    ))}
                </select>
                <select
                    className="form-select"
                    value={sortBy}
                    onChange={(e) => setSortBy(e.target.value as SortBy)}
                    style={{ width: 170 }}
                >
                    <option value="relevance">Độ liên quan</option>
                    <option value="date">Mới nhất</option>
                    <option value="name">Tên A-Z</option>
                </select>
            </div>

            {error ? <div className="module-error mb-3">⚠️ {error}</div> : null}

            {displayedResults.length === 0 && !loading ? (
                <div className="card">
                    <div className="empty-state" style={{ padding: "var(--space-8)" }}>
                        <div className="empty-state-icon">⌕</div>
                        <div className="empty-state-title">
                            {keyword.trim() ? "Không tìm thấy kết quả" : "Bắt đầu tìm kiếm"}
                        </div>
                        <div className="empty-state-text">
                            {keyword.trim()
                                ? "Thử thay đổi từ khóa hoặc bỏ bộ lọc loại tài liệu."
                                : "Nhập từ khóa ở trên để tìm kiếm nội dung bên trong tài liệu."}
                        </div>
                    </div>
                </div>
            ) : null}

            {displayedResults.length > 0 ? (
                <>
                    <div className="text-muted text-xs mb-2">{displayedResults.length} kết quả</div>
                    <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
                        {displayedResults.map((item, index) => (
                            <div
                                key={`${item.document_id}-${item.page_number}-${item.block_type ?? "page"}-${index}`}
                                className="card card-interactive search-result-card"
                                onClick={() =>
                                    navigate(
                                        `/cases/${item.case_id}/docs/${item.document_id}?page=${item.page_number || 1}&q=${encodeURIComponent(keyword.trim())}`
                                    )
                                }
                                style={{ padding: "var(--space-3)" }}
                            >
                                <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", marginBottom: "var(--space-1)" }}>
                                    <span className="font-semibold" style={{ flex: 1 }}>{item.display_name}</span>
                                    <span className="badge badge-info">Trang {item.page_number || 1}</span>
                                    {item.block_type ? <span className="badge badge-warning">{item.block_type}</span> : null}
                                    {item.confidence != null ? <span className="badge badge-neutral">{Math.round(item.confidence * 100)}%</span> : null}
                                    <span className="badge badge-neutral">{item.document_type}</span>
                                    <span className="badge badge-primary">{item.score.toFixed(2)}</span>
                                </div>
                                <div className="text-muted text-xs mb-1">Hồ sơ: {item.case_id}</div>
                                <div
                                    className="text-secondary text-sm"
                                    style={{ lineHeight: "var(--leading-relaxed)" }}
                                    dangerouslySetInnerHTML={{ __html: safeSnippet(item.snippet) }}
                                />
                                <div className="text-muted text-xs mt-2">{item.created_at}</div>
                                <div className="search-page-preview">
                                    <PdfPageThumbnail
                                        filePath={item.file_path}
                                        fileType={item.file_path.split(".").pop() || "pdf"}
                                        pageNumber={item.page_number || 1}
                                        width={150}
                                        label={`Trang ${item.page_number || 1}`}
                                    />
                                </div>
                            </div>
                        ))}
                    </div>
                </>
            ) : null}
        </PageSection>
    );
}
