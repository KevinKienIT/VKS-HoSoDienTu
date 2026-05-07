use crate::commands::governed_event::persist_governed_event;
use crate::commands::module_cmd::DbState;
use crate::resource_resolver;
use crate::storage;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportPdfInput {
    pub document_ids: Vec<String>,
    pub output_path: String,
    pub cover_page: bool,
    pub table_of_contents: bool,
    pub page_numbers: bool,
    pub include_ocr_text: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportPdfResult {
    pub output_path: String,
    pub manifest_path: String,
    pub merged_documents: i64,
    pub exported_pages: i64,
    pub removed_pages: i64,
    pub skipped_documents: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExportPngPagePlan {
    export_page_number: Option<i64>,
    document_id: String,
    document_title: String,
    page_id: String,
    page_index: i64,
    current_order: i64,
    image_path: String,
    rotation: i64,
    removed: bool,
    included: bool,
    file_exists: bool,
    file_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportPngManifest {
    schema_version: String,
    exported_at: String,
    output_pdf: String,
    manifest_path: String,
    source: String,
    document_ids: Vec<String>,
    pages_total: i64,
    pages_included: i64,
    pages_removed: i64,
    pages: Vec<ExportPngPagePlan>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DossierPackageInput {
    pub case_id: String,
    pub output_dir: String,
    pub include_originals: bool,
    pub include_ocr_text: bool,
    pub include_audit: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DossierPackageResult {
    pub output_dir: String,
    pub file_count: usize,
    pub warnings: Vec<String>,
}

fn validate_before_export(
    conn: &rusqlite::Connection,
    case_id: &str,
) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();

    let managed_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE case_id = ?1 AND COALESCE(file_status, 'imported') IN ('managed_ready', 'reviewed', 'exported')",
            [case_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("EXPORT_VALIDATION_MANAGED_COUNT_FAILED: {e}"))?;
    if managed_count == 0 {
        warnings
            .push("Không có tài liệu nào ở trạng thái managed_ready/reviewed/exported".to_string());
    }

    let field_count: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM document_extracted_fields ef
             JOIN documents d ON d.document_id = ef.document_id
             WHERE d.case_id = ?1",
            [case_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if field_count == 0 {
        warnings.push("Chưa có trường trích xuất OCR (bút lục, ngày, số văn bản)".to_string());
    }

    let section_count: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM document_sections ds
             JOIN documents d ON d.document_id = ds.document_id
             WHERE d.case_id = ?1",
            [case_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if section_count == 0 {
        warnings.push("Chưa có mục lục/section trung gian từ parser DOCX/HTML".to_string());
    }

    let timeline_count: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM document_sections ds
             JOIN documents d ON d.document_id = ds.document_id
             WHERE d.case_id = ?1 AND ds.section_type = 'timeline_event'",
            [case_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if timeline_count == 0 {
        warnings.push("Chưa có timeline_event từ parser HTML/sơ đồ vụ án".to_string());
    }

    Ok(warnings)
}

fn write_text_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content)
        .map_err(|e| format!("EXPORT_PACKAGE_WRITE_FAILED:{}:{e}", path.display()))
}

fn manifest_path_for_pdf(output_path: &str) -> PathBuf {
    let path = PathBuf::from(output_path);
    let file_name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("export.pdf");
    path.with_file_name(format!("{file_name}.manifest.json"))
}

fn normalize_rotation(rotation: i64) -> i64 {
    match rotation.rem_euclid(360) {
        0 => 0,
        90 => 90,
        180 => 180,
        270 => 270,
        _ => 0,
    }
}

fn export_temp_path(output_path: &str, suffix: &str) -> PathBuf {
    let path = PathBuf::from(output_path);
    let file_name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("export.pdf");
    path.with_file_name(format!("{file_name}.{suffix}"))
}

fn python_source_dir_for_export() -> PathBuf {
    if let Some(src) = resource_resolver::resolve_python_source_dir() {
        return src;
    }
    #[cfg(debug_assertions)]
    {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("python");
    }
    #[cfg(not(debug_assertions))]
    resource_resolver::resolve_python_scripts_dir().unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources_missing_python")
    })
}

fn run_png_pdf_exporter(plan_path: &Path, output_path: &Path) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let bundle_state = resource_resolver::ensure_runtime_bundle_ready()
        .map_err(|e| format!("EXPORT_RUNTIME_BUNDLE_FAILED: {e}"))?;
    let embedded_python = resource_resolver::resolve_python_embedded()
        .ok_or_else(|| "EXPORT_PYTHON_EMBEDDED_MISSING".to_string())?;
    let python_dir = python_source_dir_for_export();
    let result_path = export_temp_path(&output_path.to_string_lossy(), "result.json");

    let mut command = Command::new(&embedded_python);
    command.env("PYTHONUTF8", "1");
    command.env("PYTHONIOENCODING", "utf-8");
    command.env("PYTHONPATH", &python_dir);
    command.env("VKS_RUNTIME_BUNDLE_DIR", &bundle_state.runtime_bundle_dir);
    command.env("VKS_OFFLINE_MODE", "1");
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let script = format!(
        "import sys; sys.path.insert(0, r'{}'); import runpy; sys.argv = ['export.png_pages_to_pdf', '--plan', r'{}', '--output', r'{}', '--result', r'{}']; runpy.run_module('export.png_pages_to_pdf', run_name='__main__')",
        python_dir.display(),
        plan_path.display(),
        output_path.display(),
        result_path.display()
    );
    let output = command
        .arg("-c")
        .arg(script)
        .current_dir(&python_dir)
        .output()
        .map_err(|e| {
            format!(
                "EXPORT_PNG_PYTHON_START_FAILED:{}:{e}",
                embedded_python.display()
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "EXPORT_PNG_PYTHON_FAILED: exit_status={}; stderr={}; stdout={}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    let stdout_text = String::from_utf8_lossy(&output.stdout).to_string();
    serde_json::from_str::<serde_json::Value>(&stdout_text).or_else(|_| {
        let result_text = fs::read_to_string(&result_path).map_err(|e| {
            format!(
                "EXPORT_PNG_RESULT_READ_FAILED:{}:{e}",
                result_path.display()
            )
        })?;
        serde_json::from_str::<serde_json::Value>(&result_text).map_err(|e| {
            format!(
                "EXPORT_PNG_RESULT_PARSE_FAILED:{}:{e}",
                result_path.display()
            )
        })
    })
}

fn copy_with_retry(source: &Path, target: &Path) -> Result<(), String> {
    let waits_ms = [40_u64, 120, 260];
    for (idx, wait_ms) in waits_ms.iter().enumerate() {
        match fs::copy(source, target) {
            Ok(_) => return Ok(()),
            Err(e) if is_lock_error(&e) && idx + 1 < waits_ms.len() => {
                thread::sleep(Duration::from_millis(*wait_ms));
            }
            Err(e) => {
                return Err(format!(
                    "EXPORT_PACKAGE_COPY_FAILED:{}->{}:{e}",
                    source.display(),
                    target.display()
                ))
            }
        }
    }
    Err("EXPORT_PACKAGE_COPY_FAILED: unknown".to_string())
}

fn is_lock_error(err: &io::Error) -> bool {
    err.raw_os_error()
        .map(|c| c == 32 || c == 33)
        .unwrap_or(false)
}

#[tauri::command]
pub fn export_pdf_bundle(
    db: State<'_, DbState>,
    input: ExportPdfInput,
) -> Result<ExportPdfResult, String> {
    if input.document_ids.is_empty() {
        return Err("EXPORT_NO_DOCUMENTS".to_string());
    }
    if input.output_path.trim().is_empty() {
        return Err("EXPORT_OUTPUT_EMPTY".to_string());
    }

    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut skipped = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let mut pages = Vec::<ExportPngPagePlan>::new();
    let mut document_titles = Vec::<String>::new();

    for doc_id in &input.document_ids {
        let (display_name, status, file_status): (String, String, String) = conn
            .query_row(
                "SELECT display_name, status, COALESCE(file_status, 'imported') FROM documents WHERE document_id = ?1",
                [doc_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| format!("EXPORT_DOC_QUERY_FAILED:{doc_id}:{e}"))?;

        if status != "reviewed"
            && file_status != "reviewed"
            && file_status != "managed_ready"
            && file_status != "ocr_done"
        {
            skipped.push(format!(
                "{display_name}: chưa được duyệt (status={status}, file_status={file_status})"
            ));
            continue;
        }
        document_titles.push(display_name.clone());

        let mut stmt = conn
            .prepare(
                "SELECT
                    page_id,
                    page_index,
                    COALESCE(current_order, page_index) AS export_order,
                    COALESCE(rotation, 0) AS rotation,
                    COALESCE(is_removed, 0) AS is_removed,
                    COALESCE(image_path, '') AS image_path,
                    COALESCE(extract_status, '') AS extract_status
                 FROM pages
                 WHERE document_id = ?1
                 ORDER BY COALESCE(current_order, page_index) ASC, page_index ASC",
            )
            .map_err(|e| format!("EXPORT_PAGES_PREPARE_FAILED:{doc_id}:{e}"))?;
        let rows = stmt
            .query_map([doc_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|e| format!("EXPORT_PAGES_QUERY_FAILED:{doc_id}:{e}"))?;

        let mut doc_page_count = 0_i64;
        for row in rows {
            let (
                page_id,
                page_index,
                current_order,
                rotation,
                is_removed,
                image_path,
                extract_status,
            ) = row.map_err(|e| format!("EXPORT_PAGE_ROW_FAILED:{doc_id}:{e}"))?;
            doc_page_count += 1;
            let path = Path::new(&image_path);
            let file_exists = !image_path.trim().is_empty() && path.exists() && path.is_file();
            let file_size = if file_exists {
                fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0)
            } else {
                0
            };
            let removed = is_removed != 0;
            let extracted = extract_status == "extracted";
            let included = !removed && extracted && file_exists;
            if removed {
                warnings.push(format!(
                    "{display_name} trang {page_index}: removed, không đưa vào PDF"
                ));
            } else if !extracted {
                warnings.push(format!(
                    "{display_name} trang {page_index}: extract_status={extract_status}, bỏ qua"
                ));
            } else if !file_exists {
                warnings.push(format!(
                    "{display_name} trang {page_index}: thiếu PNG {image_path}"
                ));
            }
            pages.push(ExportPngPagePlan {
                export_page_number: None,
                document_id: doc_id.clone(),
                document_title: display_name.clone(),
                page_id,
                page_index,
                current_order,
                image_path,
                rotation: normalize_rotation(rotation),
                removed,
                included,
                file_exists,
                file_size,
            });
        }
        if doc_page_count == 0 {
            skipped.push(format!("{display_name}: không có page rows để export"));
        }
    }
    drop(conn);

    let mut export_page_number = 1_i64;
    for page in pages.iter_mut().filter(|page| page.included) {
        page.export_page_number = Some(export_page_number);
        export_page_number += 1;
    }
    let included_count = export_page_number - 1;
    let removed_count = pages.iter().filter(|page| page.removed).count() as i64;
    if included_count == 0 {
        return Err(format!(
            "EXPORT_NO_STORED_PAGE_IMAGES: skipped={}; warnings={}",
            skipped.join(" | "),
            warnings.join(" | ")
        ));
    }

    let managed_output_path = storage::export_path_for_requested(&input.output_path)?;
    let managed_output = managed_output_path.to_string_lossy().to_string();
    let manifest_path = manifest_path_for_pdf(&managed_output);
    let plan_path = export_temp_path(&managed_output, "plan.json");
    let manifest_path_string = manifest_path.to_string_lossy().to_string();
    let manifest = ExportPngManifest {
        schema_version: "export_png_order_v1".to_string(),
        exported_at: chrono::Local::now().to_rfc3339(),
        output_pdf: managed_output.clone(),
        manifest_path: manifest_path_string.clone(),
        source: "stored_page_images".to_string(),
        document_ids: input.document_ids.clone(),
        pages_total: pages.len() as i64,
        pages_included: included_count,
        pages_removed: removed_count,
        pages: pages.clone(),
        warnings: warnings.clone(),
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("EXPORT_MANIFEST_SERIALIZE_FAILED: {e}"))?;
    write_text_file(&plan_path, &manifest_json)?;
    run_png_pdf_exporter(&plan_path, &managed_output_path)?;
    write_text_file(&manifest_path, &manifest_json)?;

    // Persist governed event for audit trail
    {
        let conn2 = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;
        let _ = persist_governed_event(
            &conn2,
            "export_pdf_bundle",
            "export_cmd.export_pdf_bundle",
            "P3-export",
            &serde_json::json!({
                "output_path": managed_output,
                "manifest_path": manifest_path_string,
                "source": "stored_page_images",
                "merged_documents": document_titles.len() as i64,
                "exported_pages": included_count,
                "removed_pages": removed_count,
                "skipped_count": skipped.len(),
                "warning_count": warnings.len(),
            }),
        );
    }

    Ok(ExportPdfResult {
        output_path: managed_output,
        manifest_path: manifest_path_string,
        merged_documents: document_titles.len() as i64,
        exported_pages: included_count,
        removed_pages: removed_count,
        skipped_documents: skipped,
        warnings,
    })
}

#[tauri::command]
pub fn export_dossier_package(
    db: State<'_, DbState>,
    input: DossierPackageInput,
) -> Result<DossierPackageResult, String> {
    if input.case_id.trim().is_empty() {
        return Err("EXPORT_PACKAGE_CASE_EMPTY".to_string());
    }

    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let (case_code, case_name): (String, String) = conn
        .query_row(
            "SELECT case_code, case_display_name FROM cases WHERE case_id = ?1",
            [&input.case_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("EXPORT_PACKAGE_CASE_QUERY_FAILED: {e}"))?;

    let package_root = if input.output_dir.trim().is_empty() {
        let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        storage::case_exports_dir(&case_code)?.join(format!("package_{ts}"))
    } else {
        PathBuf::from(input.output_dir.trim()).join(format!(
            "{}_package",
            storage::safe_component(&case_code, "case")
        ))
    };
    fs::create_dir_all(&package_root).map_err(|e| {
        format!(
            "EXPORT_PACKAGE_CREATE_ROOT_FAILED:{}:{e}",
            package_root.display()
        )
    })?;
    let managed_docs_dir = package_root.join("03_tai_lieu_quan_ly");
    fs::create_dir_all(&managed_docs_dir).map_err(|e| {
        format!(
            "EXPORT_PACKAGE_CREATE_DOCS_FAILED:{}:{e}",
            managed_docs_dir.display()
        )
    })?;

    let mut warnings = validate_before_export(&conn, &input.case_id)?;
    let mut index_lines = vec![
        "BANG CHI MUC TRICH DAN TAI LIEU".to_string(),
        format!("Ho so: {case_code} - {case_name}"),
        String::new(),
    ];
    let mut citation_items = Vec::<serde_json::Value>::new();
    let mut file_count = 0usize;

    let mut stmt = conn
        .prepare(
            "SELECT document_id, display_name, COALESCE(managed_path, file_path), COALESCE(original_path, file_path), COALESCE(file_status, 'imported')
             FROM documents
             WHERE case_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(|e| format!("EXPORT_PACKAGE_DOC_PREPARE_FAILED: {e}"))?;
    let rows = stmt
        .query_map([&input.case_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| format!("EXPORT_PACKAGE_DOC_QUERY_FAILED: {e}"))?;

    for (idx, row) in rows.enumerate() {
        let (document_id, display_name, managed_path, original_path, file_status) =
            row.map_err(|e| format!("EXPORT_PACKAGE_DOC_ROW_FAILED: {e}"))?;
        if file_status != "managed_ready" && file_status != "reviewed" && file_status != "exported"
        {
            warnings.push(format!(
                "{display_name}: file_status={file_status}, chua managed_ready"
            ));
        }
        let source = if input.include_originals {
            &original_path
        } else {
            &managed_path
        };
        let source_path = Path::new(source);
        let ext = source_path
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or("dat");
        let safe_name = format!(
            "{:03}_{}.{}",
            idx + 1,
            storage::safe_component(&display_name, "document"),
            ext
        );
        let target = managed_docs_dir.join(safe_name);
        if source_path.exists() {
            copy_with_retry(source_path, &target)?;
            file_count += 1;
        } else {
            warnings.push(format!("{display_name}: missing source file {source}"));
        }
        index_lines.push(format!(
            "{:02}. {} [{}]",
            idx + 1,
            display_name,
            file_status
        ));

        citation_items.push(serde_json::json!({
            "document_id": document_id,
            "display_name": display_name,
            "file_status": file_status,
            "source_path": source,
            "exported_path": target.to_string_lossy(),
        }));
    }

    // NOTE: These are plain-text files. Extension is .txt until real OOXML generation is implemented.
    write_text_file(
        &package_root.join("00_index_trich_dan.txt"),
        &index_lines.join("\n"),
    )?;
    file_count += 1;
    write_text_file(
        &package_root.join("01_bao_cao_tong_hop.txt"),
        &format!("Bao cao tong hop vu an\nHo so: {case_code} - {case_name}\n"),
    )?;
    file_count += 1;
    write_text_file(&package_root.join("02_so_do_vu_an.html"), &format!("<!doctype html><meta charset=\"utf-8\"><title>{case_code}</title><h1>{case_name}</h1><p>So do vu an se duoc bo sung o phase export master.</p>"))?;
    file_count += 1;
    write_text_file(
        &package_root.join("04_phu_luc_citation.json"),
        &serde_json::to_string_pretty(&citation_items)
            .map_err(|e| format!("EXPORT_PACKAGE_CITATION_JSON_FAILED: {e}"))?,
    )?;
    file_count += 1;

    let case_ai_payload = conn
        .query_row(
            "SELECT payload_json
             FROM case_ai_summaries
             WHERE case_id = ?1 AND TRIM(COALESCE(payload_json, '')) <> ''
             ORDER BY created_at DESC
             LIMIT 1",
            [&input.case_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| {
            warnings.push("Chưa có AI summary toàn hồ sơ (06_tom_tat_ai_ho_so.json sẽ chứa dữ liệu mặc định).".to_string());
            serde_json::json!({
                "case_id": &input.case_id,
                "summary": "Chưa có AI summary toàn hồ sơ.",
                "document_count": citation_items.len(),
                "documents_analyzed": 0,
                "review_required_count": 0,
            })
            .to_string()
        });
    write_text_file(
        &package_root.join("06_tom_tat_ai_ho_so.json"),
        &serde_json::to_string_pretty(
            &serde_json::from_str::<serde_json::Value>(&case_ai_payload)
                .unwrap_or_else(|_| serde_json::json!({ "raw": case_ai_payload })),
        )
        .map_err(|e| format!("EXPORT_PACKAGE_AI_CASE_JSON_FAILED: {e}"))?,
    )?;
    file_count += 1;

    let mut people_profiles = Vec::<serde_json::Value>::new();
    let mut people_stmt = conn
        .prepare(
            "SELECT payload_json
             FROM case_people_profiles
             WHERE case_id = ?1
             ORDER BY role ASC, full_name ASC",
        )
        .map_err(|e| format!("EXPORT_PACKAGE_AI_PEOPLE_PREPARE_FAILED: {e}"))?;
    let people_rows = people_stmt
        .query_map([&input.case_id], |row| row.get::<_, String>(0))
        .map_err(|e| format!("EXPORT_PACKAGE_AI_PEOPLE_QUERY_FAILED: {e}"))?;
    for row in people_rows {
        let payload = row.map_err(|e| format!("EXPORT_PACKAGE_AI_PEOPLE_ROW_FAILED: {e}"))?;
        people_profiles.push(
            serde_json::from_str::<serde_json::Value>(&payload)
                .unwrap_or_else(|_| serde_json::json!({ "raw": payload })),
        );
    }
    if people_profiles.is_empty() {
        warnings.push(
            "Chưa có danh sách người liên quan từ AI (07_danh_sach_nguoi_lien_quan.json sẽ rỗng)."
                .to_string(),
        );
        people_profiles.push(serde_json::json!({
            "audit_warning": "EMPTY_AI_PEOPLE_PROFILES",
            "message": "Chưa có danh sách người liên quan từ AI; dữ liệu rỗng hợp lệ sau OCR/AI fallback.",
            "case_id": &input.case_id,
            "review_required": true,
        }));
    }
    write_text_file(
        &package_root.join("07_danh_sach_nguoi_lien_quan.json"),
        &serde_json::to_string_pretty(&people_profiles)
            .map_err(|e| format!("EXPORT_PACKAGE_AI_PEOPLE_JSON_FAILED: {e}"))?,
    )?;
    file_count += 1;

    let mut but_luc_items = Vec::<serde_json::Value>::new();
    let mut bl_stmt = conn
        .prepare(
            "SELECT document_id, page_number, but_luc_value, confidence, bbox_json,
                    source_text, source, review_required, payload_json
             FROM document_but_luc
             WHERE case_id = ?1
             ORDER BY page_number ASC, document_id ASC",
        )
        .map_err(|e| format!("EXPORT_PACKAGE_AI_BUT_LUC_PREPARE_FAILED: {e}"))?;
    let bl_rows = bl_stmt
        .query_map([&input.case_id], |row| {
            Ok(serde_json::json!({
                "document_id": row.get::<_, String>(0)?,
                "page_number": row.get::<_, i64>(1)?,
                "but_luc_value": row.get::<_, String>(2)?,
                "confidence": row.get::<_, f64>(3)?,
                "bbox": row.get::<_, Option<String>>(4)?,
                "source_text": row.get::<_, String>(5)?,
                "source": row.get::<_, String>(6)?,
                "review_required": row.get::<_, i64>(7)? != 0,
                "payload": row.get::<_, Option<String>>(8)?,
            }))
        })
        .map_err(|e| format!("EXPORT_PACKAGE_AI_BUT_LUC_QUERY_FAILED: {e}"))?;
    for row in bl_rows {
        but_luc_items.push(row.map_err(|e| format!("EXPORT_PACKAGE_AI_BUT_LUC_ROW_FAILED: {e}"))?);
    }
    if but_luc_items.is_empty() {
        warnings.push(
            "Chưa có bút lục tài liệu từ OCR/AI (08_but_luc_tai_lieu.json sẽ rỗng).".to_string(),
        );
        but_luc_items.push(serde_json::json!({
            "audit_warning": "EMPTY_BUT_LUC_ITEMS",
            "message": "Chưa có bút lục tài liệu từ OCR/AI; dữ liệu rỗng hợp lệ và cần rà soát.",
            "case_id": &input.case_id,
            "review_required": true,
        }));
    }
    write_text_file(
        &package_root.join("08_but_luc_tai_lieu.json"),
        &serde_json::to_string_pretty(&but_luc_items)
            .map_err(|e| format!("EXPORT_PACKAGE_AI_BUT_LUC_JSON_FAILED: {e}"))?,
    )?;
    file_count += 1;

    if input.include_audit {
        let audit = serde_json::json!({
            "case_id": input.case_id,
            "case_code": case_code,
            "case_name": case_name,
            "exported_at": chrono::Local::now().to_rfc3339(),
            "include_originals": input.include_originals,
            "include_ocr_text": input.include_ocr_text,
            "warnings": warnings,
        });
        write_text_file(
            &package_root.join("05_audit_export.json"),
            &serde_json::to_string_pretty(&audit)
                .map_err(|e| format!("EXPORT_PACKAGE_AUDIT_JSON_FAILED: {e}"))?,
        )?;
        file_count += 1;
    }

    // Persist governed event for audit trail
    let _ = persist_governed_event(
        &conn,
        "export_dossier_package",
        "export_cmd.export_dossier_package",
        "P3-export",
        &serde_json::json!({
            "case_id": input.case_id,
            "case_code": case_code,
            "output_dir": package_root.to_string_lossy(),
            "file_count": file_count,
            "warning_count": warnings.len(),
        }),
    );

    Ok(DossierPackageResult {
        output_dir: package_root.to_string_lossy().to_string(),
        file_count,
        warnings,
    })
}
