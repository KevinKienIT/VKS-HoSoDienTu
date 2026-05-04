import type { ReactNode } from "react";
import { Link } from "react-router-dom";

export function PageSection({
    title,
    description,
    right,
    children,
}: {
    title: string;
    description?: string;
    right?: ReactNode;
    children?: ReactNode;
}) {
    return (
        <div className="fade-in">
            <div className="page-header">
                <div>
                    <h1 className="page-title">{title}</h1>
                    {description ? <p className="page-description">{description}</p> : null}
                </div>
                {right}
            </div>
            {children}
        </div>
    );
}

export function EmptyCard({ text }: { text: string }) {
    return (
        <div className="card">
            <div className="empty-state-text">{text}</div>
        </div>
    );
}

export function TypeIcon({ docType }: { docType: string }) {
    const t = docType.toLowerCase();
    if (t.includes("to_khai") || t.includes("to khai")) {
        return <span title="Tờ khai">📄</span>;
    }
    if (t.includes("bien_ban") || t.includes("bien ban")) {
        return <span title="Biên bản">📋</span>;
    }
    if (t.includes("quyet_dinh") || t.includes("quyet dinh")) {
        return <span title="Quyết định">⚖️</span>;
    }
    if (t.includes("ket_luan") || t.includes("ket luan")) {
        return <span title="Kết luận">📊</span>;
    }
    if (t.includes("phieu")) {
        return <span title="Phiếu">📋</span>;
    }
    return <span title="Tài liệu">📄</span>;
}

export function SmallBackToCase({ caseId }: { caseId: string }) {
    return (
        <div style={{ marginBottom: "var(--space-3)" }}>
            <Link className="btn btn-sm" to={`/cases/${caseId}`}>
                ← Quay lại hồ sơ
            </Link>
        </div>
    );
}

export function Breadcrumb({ items }: { items: { label: string; to?: string }[] }) {
    return (
        <nav className="text-xs text-muted mb-3" style={{ display: "flex", gap: "var(--space-1)" }}>
            {items.map((item, i) => (
                <span key={`${item.label}-${i}`}>
                    {item.to ? (
                        <Link to={item.to} style={{ color: "var(--color-text-link)" }}>
                            {item.label}
                        </Link>
                    ) : (
                        item.label
                    )}
                    {i < items.length - 1 ? <span style={{ margin: "0 4px" }}>/</span> : null}
                </span>
            ))}
        </nav>
    );
}

