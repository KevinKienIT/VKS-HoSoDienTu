/**
 * VKS ECMS — Offline AI Notebook service.
 *
 * No external server is required. Rust commands provide Tier 1 extractive
 * answers from SQLite and only report Ollama localhost availability.
 */

import { invoke } from "@tauri-apps/api/core";

export interface AiStatus {
  available: boolean;
  provider: string;
  mode: string;
  message: string;
}

export interface AiSource {
  source_type: string;
  source_id: string;
  document_id?: string | null;
  page_number?: number | null;
  label: string;
  excerpt: string;
}

export interface AiAnswer {
  mode: string;
  answer: string;
  sources: AiSource[];
}

async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T | null> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.warn(`[aiService] Tauri invoke failed: ${command} — ${msg}`);
    return null;
  }
}

export async function checkAiStatus(): Promise<AiStatus> {
  const result = await safeInvoke<AiStatus>("ai_check_status");
  return result ?? {
    available: false,
    provider: "offline",
    mode: "browser_preview_unavailable",
    message: "AI Notebook chi hoat dong day du trong Tauri runtime.",
  };
}

export async function summarizeCase(caseId: string): Promise<AiAnswer | null> {
  return await safeInvoke<AiAnswer>("ai_summarize_case", { caseId });
}

export async function askCase(caseId: string, question: string): Promise<AiAnswer | null> {
  return await safeInvoke<AiAnswer>("ai_ask_case", { caseId, question });
}
