import type { Document } from "../lib/catalog";
import { invoke } from "@tauri-apps/api/core";

export interface DocumentSummary {
  document_id: string;
  case_id: string;
  display_name: string;
  original_filename: string;
  file_path: string;
  document_type: string;
  page_count: number;
  status: string;
  created_at: string;
  file_missing: boolean;
}

export interface ImportDocumentInput {
  case_id: string;
  file_path: string;
  document_type?: string;
}

export interface DocumentGroup {
  group_key: string;
  count: number;
}

export interface IndexRebuildResult {
  indexed_documents: number;
  indexed_pages: number;
  indexed_entities: number;
}

export interface PageOcrResult {
  page_id: string;
  document_id: string;
  page_index: number;
  ocr_text: string;
  ocr_formatted_text: string;
  ocr_layout_blocks: string;
  confidence: number;
  engine: string;
  but_luc: string | null;
  transcription_state: string;
}

export interface FilenameSuggestion {
  document_id: string;
  suggested_filename: string;
  current_filename: string;
  missing_fields: string[];
}

export interface WorkProductSummary {
  work_product_id: string;
  case_id: string;
  title: string;
  body: string;
  created_at: string;
}

export interface OcrRunResult {
  document_id: string;
  processed_pages: number;
  failed_pages: number;
  handwritten_pages: number;
  average_confidence: number;
  status: string;
  message: string;
}

export interface RescanOcrOptions {
  scope?: "current" | "all";
  quality?: "fast" | "high" | "max";
  mode?: "auto" | "printed" | "handwritten";
  detectLayout?: boolean;
  detectMarks?: boolean;
  rebuildIndex?: boolean;
}

export interface PageLayoutBlock {
  id: string;
  document_id: string;
  page_id: string;
  page_number: number;
  block_type: string;
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
  page_width: number | null;
  page_height: number | null;
  confidence: number;
  reading_order: number;
  engine: string;
  source: string;
  reviewed: boolean;
}

export interface ExtractedField {
  id: string;
  document_id: string;
  field_name: string;
  field_value: string;
  page_number: number;
  x: number;
  y: number;
  width: number;
  height: number;
  confidence: number;
  source: string;
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
    console.warn(`[documentService] Tauri invoke failed: ${command} — ${msg}`);
    return null;
  }
}

export async function listDocuments(caseId?: string): Promise<DocumentSummary[]> {
  const result = await safeInvoke<DocumentSummary[]>("list_documents", {
    caseId,
  });
  return result ?? [];
}

export async function getDocument(documentId: string): Promise<DocumentSummary | null> {
  const result = await safeInvoke<DocumentSummary>("get_document", {
    documentId,
  });
  return result ?? null;
}

export async function updateDocumentStatus(
  documentId: string,
  status: "pending" | "processed" | "reviewed" | "error",
  reasonNote?: string
): Promise<DocumentSummary | null> {
  const result = await safeInvoke<DocumentSummary>("update_document_status", {
    documentId,
    status,
    reasonNote,
  });
  return result ?? null;
}

export async function getPageOcr(
  documentId: string,
  pageIndex: number
): Promise<PageOcrResult | null> {
  const result = await safeInvoke<PageOcrResult | null>("get_page_ocr", {
    documentId,
    pageIndex,
  });
  return result ?? null;
}

export async function updatePageOcr(
  documentId: string,
  pageIndex: number,
  ocrText: string
): Promise<PageOcrResult | null> {
  const result = await safeInvoke<PageOcrResult>("update_page_ocr", {
    documentId,
    pageIndex,
    ocrText,
  });
  return result ?? null;
}

export async function runOcrForDocument(
  documentId: string
): Promise<OcrRunResult | null> {
  return await safeInvoke<OcrRunResult>("run_ocr_for_document", {
    documentId,
  });
}

export async function rescanDocumentOcr(
  documentId: string,
  pageNumber?: number,
  options: RescanOcrOptions = {}
): Promise<OcrRunResult | null> {
  return await safeInvoke<OcrRunResult>("rescan_document_ocr", {
    documentId,
    pageNumber,
    options,
  });
}

export async function listPageLayoutBlocks(
  documentId: string,
  pageNumber: number
): Promise<PageLayoutBlock[]> {
  const result = await safeInvoke<PageLayoutBlock[]>("list_page_layout_blocks", {
    documentId,
    pageNumber,
  });
  return result ?? [];
}

export async function listDocumentExtractedFields(
  documentId: string
): Promise<ExtractedField[]> {
  const result = await safeInvoke<ExtractedField[]>("list_document_extracted_fields", {
    documentId,
  });
  return result ?? [];
}

export async function analyzeDocumentWithAiAgent(
  documentId: string
): Promise<WorkProductSummary | null> {
  return await safeInvoke<WorkProductSummary>("analyze_document_with_ai_agent", {
    documentId,
  });
}

export async function importDocument(
  input: ImportDocumentInput
): Promise<DocumentSummary | null> {
  const result = await safeInvoke<DocumentSummary>("import_document", { input });
  return result ?? null;
}

export async function enrichDocumentMetadata(
  documentId: string
): Promise<DocumentSummary | null> {
  const result = await safeInvoke<DocumentSummary>("enrich_document_metadata", {
    documentId,
  });
  return result ?? null;
}

export async function bulkEnrichDocuments(caseId?: string): Promise<number> {
  const result = await safeInvoke<number>("bulk_enrich_documents", {
    caseId,
  });
  return result ?? 0;
}

export async function getDocumentGroups(caseId?: string): Promise<DocumentGroup[]> {
  const result = await safeInvoke<DocumentGroup[]>("get_document_groups", {
    caseId,
  });
  return result ?? [];
}

export async function rebuildTextIndex(): Promise<IndexRebuildResult | null> {
  return await safeInvoke<IndexRebuildResult>("rebuild_text_index");
}

export async function suggestDocumentFilename(
  documentId: string
): Promise<FilenameSuggestion | null> {
  return await safeInvoke<FilenameSuggestion>("suggest_document_filename", {
    documentId,
  });
}

export async function renameDocumentFile(
  documentId: string,
  newFilename: string
): Promise<DocumentSummary | null> {
  return await safeInvoke<DocumentSummary>("rename_document_file", {
    documentId,
    newFilename,
  });
}

export async function createNoteFromSelection(
  caseId: string,
  documentId: string,
  pageNumber: number,
  selectedText: string
): Promise<WorkProductSummary | null> {
  return await safeInvoke<WorkProductSummary>("create_note_from_selection", {
    caseId,
    documentId,
    pageNumber,
    selectedText,
  });
}

export async function moveDocumentToCase(
  documentId: string,
  targetCaseId: string
): Promise<DocumentSummary | null> {
  return await safeInvoke<DocumentSummary>("move_document_to_case", {
    documentId,
    targetCaseId,
  });
}

export function toDocument(summary: DocumentSummary): Document {
  return {
    document_id: summary.document_id,
    case_id: summary.case_id,
    original_filename: summary.original_filename,
    import_sequence: null,
    file_path: "",
    file_hash: "",
    file_size: null,
    page_count: summary.page_count,
    display_name: summary.display_name,
    document_title: summary.display_name,
    document_type: summary.document_type,
    document_subtype: null,
    issued_date: null,
    summary_short: null,
    ocr_confidence_avg: null,
    has_handwriting: false,
    classification_confidence: null,
    needs_review: false,
    status: summary.status as "pending" | "processed" | "reviewed" | "error",
    created_at: summary.created_at,
    updated_at: summary.created_at,
  };
}
