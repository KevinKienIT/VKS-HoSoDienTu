import { invoke } from "@tauri-apps/api/core";

export interface ReviewItem {
    review_id: string;
    case_id: string;
    document_id: string;
    page_id: string | null;
    display_name: string;
    document_type: string;
    review_type: string;
    status: string;
    confidence: number;
    reviewer_note: string;
    created_at: string;
}

async function safeInvoke<T>(
    command: string,
    args?: Record<string, unknown>
): Promise<T | null> {
    try {
        return await invoke<T>(command, args);
    } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        console.warn(`[reviewService] Tauri invoke failed: ${command} — ${msg}`);
        return null;
    }
}

export async function listReviewQueue(statusFilter?: string): Promise<ReviewItem[]> {
    const result = await safeInvoke<ReviewItem[]>("list_review_queue", {
        statusFilter,
    });
    return result ?? [];
}

export async function reviewAction(
    reviewId: string,
    action: "approved" | "rejected" | "skipped",
    note?: string
): Promise<boolean> {
    try {
        await invoke("review_action", {
            reviewId,
            action,
            reviewerNote: note,
        });
        return true;
    } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        console.warn(`[reviewService] review_action failed: ${msg}`);
        return false;
    }
}

