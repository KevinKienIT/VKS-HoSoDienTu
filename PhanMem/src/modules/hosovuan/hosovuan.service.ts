import type { Case } from "../lib/catalog";
import { invoke } from "@tauri-apps/api/core";

export interface CaseSummary {
  case_id: string;
  case_code: string;
  case_display_name: string;
  primary_person_name: string;
  case_type: string;
  dossier_type: string;
  but_luc: string | null;
  ocr_state: "none" | "pending" | "done" | string;
  status: string;
  document_count: number;
  missing_document_count: number;
  total_pages: number;
  created_at: string;
}

export interface CreateCaseInput {
  case_display_name: string;
  primary_person_name?: string;
  source_folder_name?: string;
  case_type?: string;
  prosecutor_office?: string;
  investigator_name?: string;
}

export interface PurgeImportedDossierInput {
  case_code: string;
  expected_case_identity: string;
  confirm_token: "ok";
}

export interface PurgeImportedDossierSummary {
  case_id: string;
  case_code: string;
  case_display_name: string;
  deleted_rows_by_table: Record<string, number>;
  deleted_files_count: number;
  deleted_cache_dirs_count: number;
  errors: string[];
}

async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T | null> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.warn(`[caseService] Tauri invoke failed: ${command} — ${msg}`);
    return null;
  }
}

export async function listCases(): Promise<CaseSummary[]> {
  const result = await safeInvoke<CaseSummary[]>("list_cases");
  return result ?? [];
}

export async function getCase(caseId: string): Promise<CaseSummary | null> {
  const result = await safeInvoke<CaseSummary>("get_case", {
    caseId,
  });
  return result ?? null;
}

export async function createCase(input: CreateCaseInput): Promise<CaseSummary | null> {
  const result = await safeInvoke<CaseSummary>("create_case", { input });
  return result ?? null;
}

export async function purgeImportedDossier(
  input: PurgeImportedDossierInput
): Promise<PurgeImportedDossierSummary | null> {
  const result = await safeInvoke<PurgeImportedDossierSummary>("purge_imported_dossier", { input });
  return result ?? null;
}

export function toCase(summary: CaseSummary): Case {
  return {
    case_id: summary.case_id,
    case_code: summary.case_code,
    case_display_name: summary.case_display_name,
    source_folder_name: "",
    case_sequence_no: null,
    primary_person_name: summary.primary_person_name,
    case_type: summary.case_type || "to_dieu_tra",
    status: summary.status as "active" | "archived" | "closed",
    document_count: summary.document_count,
    total_pages: summary.total_pages,
    created_at: summary.created_at,
    updated_at: summary.created_at,
  };
}
