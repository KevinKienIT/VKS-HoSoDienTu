import { invoke } from "@tauri-apps/api/core";

export interface ScanSettings {
  inbox_folder: string;
  import_mode: "manual" | "auto" | "ask";
  default_case_id: string;
  accept_pdf: boolean;
  accept_tiff: boolean;
  accept_jpg: boolean;
  accept_png: boolean;
  stable_wait_ms: number;
  duplicate_detection: boolean;
  ocr_after_scan: boolean;
  classify_after_scan: boolean;
}

export interface ScanInboxFile {
  file_path: string;
  file_name: string;
  file_ext: string;
  file_size: number;
  modified_at: string;
  status: "waiting_for_stable" | "ready" | "duplicate" | "error" | string;
  reason: string;
  duplicate: boolean;
}

export interface ScanImportResult {
  document_id: string;
  case_id: string;
  status: string;
  message: string;
}

export interface ScannerDriverStatus {
  scan_to_folder_supported: boolean;
  direct_scan_supported: boolean;
  wia_service_status: string;
  recommendation: string;
}

export interface PipelineJob {
  job_id: string;
  source_type: string;
  status: "created" | "running" | "paused" | "completed" | "cancelled" | "failed" | string;
  total_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  cancelled_tasks: number;
  pause_requested: boolean;
  cancel_requested: boolean;
  created_at: string;
  started_at: string | null;
  paused_at: string | null;
  resumed_at: string | null;
  completed_at: string | null;
  cancelled_at: string | null;
  updated_at: string;
  last_error: string | null;
}

export interface PipelineJobListItem {
  job_id: string;
  source_type: string;
  status: string;
  total_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  cancelled_tasks: number;
  created_at: string;
  updated_at: string;
}

export interface PipelineFooterStatus {
  active_jobs: number;
  paused_jobs: number;
  failed_jobs: number;
  completed_jobs: number;
  cancelled_jobs: number;
}

export interface PipelineTickResult {
  job_id: string;
  worker_pool_size: number;
  max_inflight: number;
  queue_high_watermark: number;
  queued_total: number;
  running_total: number;
  claimed_count: number;
  completed_count: number;
  failed_count: number;
  skipped_not_ready_count: number;
  stopped_reason: string;
}

export interface PipelineProgressStatus {
  job_id: string;
  status: string;
  source_type: string;
  phase: string;
  processed_tasks: number;
  total_tasks: number;
  running_tasks: number;
  queued_tasks: number;
  failed_tasks: number;
  retrying_tasks: number;
  progress_percent: number;
  eta_seconds: number | null;
  can_resume: boolean;
  can_pause: boolean;
  can_cancel: boolean;
  updated_at: string;
  note: string | null;
}

export interface PipelineIntegrationCheckResult {
  job_id: string;
  discover_completed: boolean;
  import_completed: boolean;
  ocr_reached: boolean;
  chain_ok: boolean;
  notes: string[];
}

export interface PipelineKpiSummary {
  total_jobs: number;
  running_jobs: number;
  completed_jobs: number;
  failed_jobs: number;
  cancelled_jobs: number;
  total_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  completion_rate_percent: number;
  generated_at: string;
}

export interface PipelineSafetyReport {
  queue_high_watermark: number;
  max_worker_pool_cap: number;
  max_inflight_cap: number;
  blocked_jobs_by_watermark: number;
  jobs_with_retry_pressure: number;
  unresolved_risks: string[];
  rollout_guards: string[];
}

export interface PipelineProcessModeSettings {
  process_mode: "manual" | "full_auto" | string;
  autoscan: boolean;
  autosave: boolean;
  autoname: boolean;
  autosummary: boolean;
  autoclassify: boolean;
}

async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T | null> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.warn(`[scanService] Tauri invoke failed: ${command} — ${msg}`);
    return null;
  }
}

export async function getScanSettings(): Promise<ScanSettings> {
  const result = await safeInvoke<ScanSettings>("get_scan_settings");
  return result ?? {
    inbox_folder: "",
    import_mode: "ask",
    default_case_id: "",
    accept_pdf: true,
    accept_tiff: true,
    accept_jpg: true,
    accept_png: true,
    stable_wait_ms: 2500,
    duplicate_detection: true,
    ocr_after_scan: true,
    classify_after_scan: true,
  };
}

export async function saveScanSettings(settings: ScanSettings): Promise<ScanSettings | null> {
  return await safeInvoke<ScanSettings>("save_scan_settings", { settings });
}

export async function watchScanFolder(folderPath: string): Promise<boolean> {
  const result = await safeInvoke<void>("watch_scan_folder", { folderPath });
  return result !== null;
}

export async function stopWatchScanFolder(): Promise<boolean> {
  const result = await safeInvoke<void>("stop_watch_scan_folder");
  return result !== null;
}

export async function listScanInboxFiles(folderPath?: string): Promise<ScanInboxFile[]> {
  const result = await safeInvoke<ScanInboxFile[]>("list_scan_inbox_files", { folderPath });
  return result ?? [];
}

export async function importScannedFile(
  filePath: string,
  caseId: string
): Promise<ScanImportResult | null> {
  return await safeInvoke<ScanImportResult>("import_scanned_file", { filePath, caseId });
}

export async function importScannedBatch(
  filePaths: string[],
  caseId: string
): Promise<ScanImportResult[]> {
  const result = await safeInvoke<ScanImportResult[]>("import_scanned_batch", { filePaths, caseId });
  return result ?? [];
}

export async function checkScannerDriverStatus(): Promise<ScannerDriverStatus> {
  const result = await safeInvoke<ScannerDriverStatus>("check_scanner_driver_status");
  return result ?? {
    scan_to_folder_supported: true,
    direct_scan_supported: false,
    wia_service_status: "browser_preview_unavailable",
    recommendation: "Chạy bằng Tauri để kiểm tra Windows WIA service.",
  };
}

export async function startPipelineJob(sourceType?: string): Promise<PipelineJob | null> {
  return await safeInvoke<PipelineJob>("start_pipeline_job", { sourceType });
}

export async function getPipelineJob(jobId: string): Promise<PipelineJob | null> {
  return await safeInvoke<PipelineJob>("get_pipeline_job", { jobId });
}

export async function listPipelineJobs(limit?: number): Promise<PipelineJobListItem[]> {
  const result = await safeInvoke<PipelineJobListItem[]>("list_pipeline_jobs", { limit });
  return result ?? [];
}

export async function pausePipelineJob(jobId: string): Promise<PipelineJob | null> {
  return await safeInvoke<PipelineJob>("pause_pipeline_job", { jobId });
}

export async function resumePipelineJob(jobId: string): Promise<PipelineJob | null> {
  return await safeInvoke<PipelineJob>("resume_pipeline_job", { jobId });
}

export async function cancelPipelineJob(jobId: string): Promise<PipelineJob | null> {
  return await safeInvoke<PipelineJob>("cancel_pipeline_job", { jobId });
}

export async function getPipelineFooterStatus(): Promise<PipelineFooterStatus> {
  const result = await safeInvoke<PipelineFooterStatus>("get_pipeline_footer_status");
  return (
    result ?? {
      active_jobs: 0,
      paused_jobs: 0,
      failed_jobs: 0,
      completed_jobs: 0,
      cancelled_jobs: 0,
    }
  );
}

export async function getPipelineProgressStatus(jobId?: string): Promise<PipelineProgressStatus | null> {
  return await safeInvoke<PipelineProgressStatus | null>("get_pipeline_progress_status", { jobId });
}

export async function getPipelineIntegrationCheck(
  jobId?: string
): Promise<PipelineIntegrationCheckResult | null> {
  return await safeInvoke<PipelineIntegrationCheckResult | null>("get_pipeline_integration_check", { jobId });
}

export async function getPipelineKpiSummary(): Promise<PipelineKpiSummary | null> {
  return await safeInvoke<PipelineKpiSummary>("get_pipeline_kpi_summary");
}

export async function getPipelineSafetyReport(): Promise<PipelineSafetyReport | null> {
  return await safeInvoke<PipelineSafetyReport>("get_pipeline_safety_report");
}

export async function getPipelineProcessModeSettings(): Promise<PipelineProcessModeSettings> {
  const result = await safeInvoke<PipelineProcessModeSettings>("get_pipeline_process_mode_settings");
  return (
    result ?? {
      process_mode: "manual",
      autoscan: true,
      autosave: true,
      autoname: false,
      autosummary: false,
      autoclassify: false,
    }
  );
}

export async function savePipelineProcessModeSettings(
  settings: PipelineProcessModeSettings
): Promise<PipelineProcessModeSettings | null> {
  return await safeInvoke<PipelineProcessModeSettings>("save_pipeline_process_mode_settings", { settings });
}

export async function pipelineExecutionTick(jobId: string): Promise<PipelineTickResult | null> {
  return await safeInvoke<PipelineTickResult>("pipeline_execution_tick", { jobId });
}

export async function bootstrapPipelineBackground(
  jobId: string,
  ticks?: number
): Promise<PipelineTickResult[]> {
  const result = await safeInvoke<PipelineTickResult[]>("bootstrap_pipeline_background", {
    jobId,
    ticks,
  });
  return result ?? [];
}
