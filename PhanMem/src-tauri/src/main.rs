// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod storage;

use commands::module_cmd::DbState;
use commands::scan_cmd::ScanWatchState;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::Manager;

fn migrate_document_paths_to_managed_storage(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT d.document_id, c.case_code, d.original_filename, d.file_path
             FROM documents d
             JOIN cases c ON c.case_id = d.case_id",
        )
        .map_err(|e| format!("STORAGE_MIGRATE_PREPARE_FAILED: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("STORAGE_MIGRATE_QUERY_FAILED: {e}"))?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| format!("STORAGE_MIGRATE_ROW_FAILED: {e}"))?);
    }

    for (document_id, case_code, original_filename, file_path) in items {
        let source = Path::new(&file_path);
        if storage::is_managed_path(source) || !source.exists() || !source.is_file() {
            continue;
        }
        let managed_path =
            storage::copy_to_originals(source, &case_code, &document_id, &original_filename)?;
        let managed_file_path = managed_path.to_string_lossy().to_string();
        conn.execute(
            "UPDATE documents
             SET file_path = ?1,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE document_id = ?2",
            params![managed_file_path, document_id],
        )
        .map_err(|e| format!("STORAGE_MIGRATE_DOC_UPDATE_FAILED: {e}"))?;
        conn.execute(
            "UPDATE pages
             SET image_path = ?1
             WHERE document_id = ?2 AND image_path = ?3",
            params![managed_path.to_string_lossy().to_string(), document_id, file_path],
        )
        .map_err(|e| format!("STORAGE_MIGRATE_PAGE_UPDATE_FAILED: {e}"))?;
    }

    Ok(())
}

fn main() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("vks-ecms.log".into()),
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::module_cmd::get_module_configs,
            commands::module_cmd::set_module_config,
            commands::module_cmd::get_app_settings,
            commands::module_cmd::set_app_setting,
            commands::catalog_cmd::get_catalog_entries,
            commands::catalog_cmd::get_catalog_stats,
            commands::catalog_cmd::scan_folder_catalog,
            commands::catalog_cmd::update_catalog_entry_status,
            commands::catalog_cmd::delete_catalog_entry,
            commands::case_cmd::list_cases,
            commands::case_cmd::get_case,
            commands::case_cmd::create_case,
            commands::case_cmd::purge_imported_dossier,
            commands::doc_cmd::list_documents,
            commands::doc_cmd::get_document,
            commands::doc_cmd::update_document_status,
            commands::doc_cmd::get_page_ocr,
            commands::doc_cmd::update_page_ocr,
            commands::doc_cmd::run_ocr_for_document,
            commands::doc_cmd::rescan_document_ocr,
            commands::doc_cmd::list_page_layout_blocks,
            commands::doc_cmd::list_document_extracted_fields,
            commands::doc_cmd::analyze_document_with_ai_agent,
            commands::doc_cmd::suggest_document_filename,
            commands::doc_cmd::rename_document_file,
            commands::doc_cmd::create_note_from_selection,
            commands::doc_cmd::move_document_to_case,
            commands::doc_cmd::import_document,
            commands::doc_cmd::enrich_document_metadata,
            commands::doc_cmd::bulk_enrich_documents,
            commands::doc_cmd::get_document_groups,
            commands::doc_cmd::rebuild_text_index,
            commands::import_cmd::import_folder,
            commands::import_cmd::import_multiple_files,
            commands::export_cmd::export_pdf_bundle,
            commands::scan_cmd::get_scan_settings,
            commands::scan_cmd::save_scan_settings,
            commands::scan_cmd::watch_scan_folder,
            commands::scan_cmd::stop_watch_scan_folder,
            commands::scan_cmd::list_scan_inbox_files,
            commands::scan_cmd::import_scanned_file,
            commands::scan_cmd::import_scanned_batch,
            commands::scan_cmd::check_scanner_driver_status,
            commands::scan_cmd::start_pipeline_job,
            commands::scan_cmd::get_pipeline_job,
            commands::scan_cmd::list_pipeline_jobs,
            commands::scan_cmd::pause_pipeline_job,
            commands::scan_cmd::resume_pipeline_job,
            commands::scan_cmd::cancel_pipeline_job,
            commands::scan_cmd::get_pipeline_footer_status,
            commands::scan_cmd::get_pipeline_progress_status,
            commands::scan_cmd::get_pipeline_integration_check,
            commands::scan_cmd::get_pipeline_kpi_summary,
            commands::scan_cmd::get_pipeline_safety_report,
            commands::scan_cmd::get_pipeline_process_mode_settings,
            commands::scan_cmd::save_pipeline_process_mode_settings,
            commands::scan_cmd::pipeline_execution_tick,
            commands::scan_cmd::bootstrap_pipeline_background,
            commands::review_cmd::list_review_queue,
            commands::review_cmd::review_action,
            commands::search_cmd::fts_search,
            commands::ai_cmd::ai_check_status,
            commands::ai_cmd::ai_summarize_case,
            commands::ai_cmd::ai_ask_case,
            commands::system_cmd::run_startup_self_check,
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            storage::ensure_managed_dirs().map_err(std::io::Error::other)?;

            let db_path = app_data_dir.join("vks-ecms.db");
            db::init(&db_path).map_err(std::io::Error::other)?;

            // Open persistent connection for Tauri commands
            let conn = Connection::open(&db_path)
                .map_err(|e| std::io::Error::other(format!("DB open: {e}")))?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")
                .map_err(|e| std::io::Error::other(format!("PRAGMA: {e}")))?;
            migrate_document_paths_to_managed_storage(&conn).map_err(std::io::Error::other)?;
            app.manage(DbState(Mutex::new(conn)));
            app.manage(Arc::new(ScanWatchState::default()));

            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
