/**
 * VKS ECMS — Search Service
 * Goi Rust commands de tim kiem ho so bang FTS5.
 */

import { invoke } from "@tauri-apps/api/core";

export interface SearchResult {
  document_id: string;
  case_id: string;
  display_name: string;
  file_path: string;
  document_type: string;
  page_number: number;
  created_at: string;
  snippet: string;
  score: number;
  block_type?: string | null;
  bounding_box?: string | null;
  confidence?: number | null;
}

async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T | null> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.warn(`[searchService] Tauri invoke failed: ${command} — ${msg}`);
    return null;
  }
}

/**
 * Tim kiem toan van (FTS5).
 */
export async function ftsSearch(query: string, limit?: number): Promise<SearchResult[]> {
  const result = await safeInvoke<SearchResult[]>("fts_search", {
    query,
    limit: limit ?? 50
  });
  return result ?? [];
}
