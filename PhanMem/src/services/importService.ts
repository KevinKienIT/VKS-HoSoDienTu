/**
 * VKS ECMS — Import Service
 * Goi Rust commands de import ho so.
 */

import { invoke } from "@tauri-apps/api/core";

let lastImportError: string | null = null;

export interface ImportedFile {
  document_id: string;
  file_path: string;
  file_name: string;
  file_ext: string;
  file_size: number;
  page_count: number;
  group_id?: string | null;
  relative_path?: string | null;
  import_order?: number | null;
}

export interface ImportFolderResult {
  job_id: string;
  case_id: string;
  total_files: number;
  files: ImportedFile[];
}

export interface ImportMultipleFilesResult {
  case_id: string;
  total_files: number;
  imported: ImportedFile[];
  duplicates: string[];
  errors: string[];
}

function normalizeInvokeError(e: unknown): string {
  if (e instanceof Error) {
    return e.message;
  }
  if (typeof e === "string") {
    return e;
  }
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}

async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T | null> {
  try {
    lastImportError = null;
    return await invoke<T>(command, args);
  } catch (e) {
    const msg = normalizeInvokeError(e);
    lastImportError = `${command}: ${msg}`;
    console.warn(`[importService] Tauri invoke failed: ${command} — ${msg}`);
    return null;
  }
}

export function getLastImportError(): string | null {
  return lastImportError;
}

/**
 * Import toan bo mot thu muc ho so.
 */
export async function importFolder(folderPath: string): Promise<ImportFolderResult | null> {
  return await safeInvoke<ImportFolderResult>("import_folder", {
    folderPath,
  });
}

export async function importMultipleFiles(
  paths: string[],
  caseId?: string
): Promise<ImportMultipleFilesResult | null> {
  return await safeInvoke<ImportMultipleFilesResult>("import_multiple_files", {
    paths,
    caseId,
  });
}
