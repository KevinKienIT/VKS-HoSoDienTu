import { invoke } from "@tauri-apps/api/core";

export interface ExportPdfInput {
  document_ids: string[];
  output_path: string;
  cover_page: boolean;
  table_of_contents: boolean;
  page_numbers: boolean;
  include_ocr_text: boolean;
}

export interface ExportPdfResult {
  output_path: string;
  merged_documents: number;
  skipped_documents: string[];
}

async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T | null> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.warn(`[exportService] Tauri invoke failed: ${command} — ${msg}`);
    return null;
  }
}

export async function exportPdfBundle(input: ExportPdfInput): Promise<ExportPdfResult | null> {
  return await safeInvoke<ExportPdfResult>("export_pdf_bundle", { input });
}
