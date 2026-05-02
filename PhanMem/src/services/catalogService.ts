/**
 * VKS ECMS — Catalog Service
 *
 * Service layer cho catalog operations.
 * Goi Tauri commands voi fallback an toan khi chay ngoai Tauri runtime.
 */

import { invoke } from "@tauri-apps/api/core";

import type {
  CatalogEntry,
  CatalogScanStatus,
  CatalogStats,
} from "../lib/catalog";

async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T | null> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.warn(`[catalogService] Tauri invoke failed: ${command} — ${msg}`);
    return null;
  }
}

/**
 * Lay tat ca catalog entries, co the loc theo status.
 */
export async function getCatalogEntries(
  status?: CatalogScanStatus
): Promise<CatalogEntry[]> {
  const result = await safeInvoke<CatalogEntry[]>("get_catalog_entries", { status });
  return result ?? [];
}

/**
 * Lay thong ke tong quan catalog.
 */
export async function getCatalogStats(): Promise<CatalogStats> {
  const fallback: CatalogStats = {
    total_files: 0,
    total_size_bytes: 0,
    by_extension: {},
    by_folder: {},
    by_status: { discovered: 0, cataloged: 0, imported: 0, skipped: 0, error: 0 },
  };

  const result = await safeInvoke<CatalogStats>("get_catalog_stats");
  return result ?? fallback;
}

/**
 * Trigger scan mot folder va ghi vao catalog.
 */
export async function scanFolder(folderPath: string): Promise<number> {
  const result = await safeInvoke<number>("scan_folder_catalog", { folderPath });
  return result ?? 0;
}

/**
 * Cap nhat trang thai cua mot catalog entry.
 */
export async function updateEntryStatus(
  entryId: string,
  status: CatalogScanStatus,
  note?: string
): Promise<void> {
  await safeInvoke<void>("update_catalog_entry_status", {
    entryId,
    status,
    note,
  });
}

/**
 * Xoa catalog entry (chi metadata, khong xoa file goc).
 */
export async function deleteEntry(entryId: string): Promise<void> {
  await safeInvoke<void>("delete_catalog_entry", { entryId });
}
