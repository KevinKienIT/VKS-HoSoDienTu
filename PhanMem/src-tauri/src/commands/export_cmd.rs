use crate::commands::module_cmd::DbState;
use crate::storage;
use log::warn;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Bookmark, Document, Object, ObjectId, Stream};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;
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
    pub merged_documents: i64,
    pub skipped_documents: Vec<String>,
}

fn text_ops(lines: &[String], start_y: i64, font_size: i64) -> Result<Vec<u8>, String> {
    let mut operations = vec![
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec!["F1".into(), font_size.into()]),
        Operation::new("Td", vec![72.into(), start_y.into()]),
    ];
    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            operations.push(Operation::new(
                "Td",
                vec![0.into(), (-(font_size + 8)).into()],
            ));
        }
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(line.as_str())],
        ));
    }
    operations.push(Operation::new("ET", vec![]));
    Content { operations }
        .encode()
        .map_err(|e| format!("EXPORT_TEXT_ENCODE_FAILED: {e}"))
}

fn add_text_page(
    output: &mut Document,
    pages_id: ObjectId,
    font_id: ObjectId,
    lines: &[String],
    title_size: i64,
) -> Result<ObjectId, String> {
    let content = text_ops(lines, 760, title_size)?;
    let content_id = output.add_object(Stream::new(dictionary! {}, content));
    let resources_id = output.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        }
    });
    let page_id = output.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Resources" => resources_id,
        "Contents" => content_id,
    });
    Ok(page_id)
}

fn add_page_number_stream(
    output: &mut Document,
    page_dict: &mut lopdf::Dictionary,
    font_id: ObjectId,
    page_no: i64,
    total: i64,
) -> Result<(), String> {
    let content = Content {
        operations: vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["FPN".into(), 9.into()]),
            Operation::new("Td", vec![500.into(), 28.into()]),
            Operation::new(
                "Tj",
                vec![Object::string_literal(format!("{page_no}/{total}"))],
            ),
            Operation::new("ET", vec![]),
        ],
    }
    .encode()
    .map_err(|e| format!("EXPORT_PAGE_NUMBER_ENCODE_FAILED: {e}"))?;
    let content_id = output.add_object(Stream::new(dictionary! {}, content));

    if let Ok(resources) = page_dict.get_mut(b"Resources") {
        match resources {
            Object::Dictionary(dict) => {
                let mut fonts = match dict.get(b"Font") {
                    Ok(Object::Dictionary(existing)) => existing.clone(),
                    _ => dictionary! {},
                };
                fonts.set("FPN", font_id);
                dict.set("Font", Object::Dictionary(fonts));
            }
            Object::Reference(resource_id) => {
                if let Ok(Object::Dictionary(dict)) = output.get_object_mut(*resource_id) {
                    let mut fonts = match dict.get(b"Font") {
                        Ok(Object::Dictionary(existing)) => existing.clone(),
                        _ => dictionary! {},
                    };
                    fonts.set("FPN", font_id);
                    dict.set("Font", Object::Dictionary(fonts));
                }
            }
            _ => {}
        }
    } else {
        page_dict.set(
            "Resources",
            dictionary! {
                "Font" => dictionary! {
                    "FPN" => font_id,
                }
            },
        );
    }

    if let Ok(existing) = page_dict.get(b"Contents").cloned() {
        match existing {
            Object::Array(mut arr) => {
                arr.push(Object::Reference(content_id));
                page_dict.set("Contents", Object::Array(arr));
            }
            other => {
                page_dict.set(
                    "Contents",
                    Object::Array(vec![other, Object::Reference(content_id)]),
                );
            }
        }
    } else {
        page_dict.set("Contents", content_id);
    }
    Ok(())
}

fn is_lock_error(err: &io::Error) -> bool {
    err.raw_os_error()
        .map(|c| c == 32 || c == 33)
        .unwrap_or(false)
}

fn save_export_with_retry(output: &mut Document, out_path: &str) -> Result<(), String> {
    let waits_ms = [40_u64, 120, 260];
    for (idx, wait_ms) in waits_ms.iter().enumerate() {
        match output.save(out_path) {
            Ok(_) => return Ok(()),
            Err(ioe) if is_lock_error(&ioe) => {
                warn!(
                    "file-lock path={} op=export.save attempt={} wait_ms={} final_error={}",
                    out_path,
                    idx + 1,
                    wait_ms,
                    ioe
                );
                if idx + 1 < waits_ms.len() {
                    thread::sleep(Duration::from_millis(*wait_ms));
                    continue;
                }
                return Err(format!(
                    "EXPORT_SAVE_FILE_LOCKED:path={}:op=export.save:attempt={}:wait_ms={}:final_error={}",
                    out_path,
                    idx + 1,
                    wait_ms,
                    ioe
                ));
            }
            Err(e) => return Err(format!("EXPORT_SAVE_FAILED: {e}")),
        }
    }
    Err("EXPORT_SAVE_FAILED: unknown".to_string())
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

    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut pdfs: Vec<(String, String, Document)> = Vec::new();
    let mut skipped = Vec::<String>::new();

    for doc_id in &input.document_ids {
        let (display_name, file_path): (String, String) = conn
            .query_row(
                "SELECT display_name, file_path FROM documents WHERE document_id = ?1",
                [doc_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| format!("EXPORT_DOC_QUERY_FAILED: {e}"))?;

        if !file_path.to_lowercase().ends_with(".pdf") {
            skipped.push(format!("{display_name}: không phải PDF"));
            continue;
        }
        if !Path::new(&file_path).exists() {
            skipped.push(format!("{display_name}: không tìm thấy file"));
            continue;
        }

        match Document::load(&file_path) {
            Ok(doc) => pdfs.push((display_name, file_path, doc)),
            Err(e) => skipped.push(format!("{display_name}: {e}")),
        }
    }
    drop(conn);

    if pdfs.is_empty() {
        return Err("EXPORT_NO_VALID_PDF".to_string());
    }

    let mut max_id = 1;
    let mut page_number = 1;
    let mut layer_parent: [Option<u32>; 4] = [None; 4];
    let mut documents_pages = BTreeMap::<ObjectId, Object>::new();
    let mut documents_objects = BTreeMap::<ObjectId, Object>::new();
    let mut page_order = Vec::<ObjectId>::new();
    let mut document_titles = Vec::<String>::new();
    let mut output = Document::with_version("1.5");

    if input.table_of_contents {
        layer_parent[0] = Some(output.add_bookmark(
            Bookmark::new("Mục lục hồ sơ".to_string(), [0.0, 0.0, 0.0], 0, (0, 0)),
            None,
        ));
    }

    for (display_name, _file_path, mut doc) in pdfs {
        document_titles.push(display_name.clone());
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        let mut first_object = None;
        let pages = doc.get_pages();
        for object_id in pages.into_values() {
            if first_object.is_none() {
                first_object = Some(object_id);
                if input.table_of_contents {
                    let title = format!("Trang {} - {}", page_number, display_name);
                    let parent = layer_parent[0];
                    output
                        .add_bookmark(Bookmark::new(title, [0.0, 0.0, 0.0], 0, object_id), parent);
                }
            }
            page_number += 1;
            let object = doc
                .get_object(object_id)
                .map_err(|e| format!("EXPORT_PAGE_OBJECT_FAILED: {e}"))?
                .to_owned();
            page_order.push(object_id);
            documents_pages.insert(object_id, object);
        }
        documents_objects.extend(doc.objects);
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects.into_iter() {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                catalog_object = Some((
                    catalog_object.map(|(id, _)| id).unwrap_or(object_id),
                    object,
                ));
            }
            b"Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref existing)) = pages_object {
                        if let Ok(old_dictionary) = existing.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }
                    pages_object = Some((
                        pages_object.map(|(id, _)| id).unwrap_or(object_id),
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                output.objects.insert(object_id, object);
            }
        }
    }

    let (pages_id, pages_object) =
        pages_object.ok_or_else(|| "EXPORT_PAGES_ROOT_NOT_FOUND".to_string())?;
    let font_id = output.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let total_scan_pages = page_order.len() as i64;
    for (idx, object_id) in page_order.iter().enumerate() {
        let object = documents_pages
            .get(object_id)
            .ok_or_else(|| "EXPORT_PAGE_ORDER_MISSING".to_string())?;
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_id);
            if input.page_numbers {
                add_page_number_stream(
                    &mut output,
                    &mut dictionary,
                    font_id,
                    idx as i64 + 1,
                    total_scan_pages,
                )?;
            }
            output
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    let (catalog_id, catalog_object) =
        catalog_object.ok_or_else(|| "EXPORT_CATALOG_NOT_FOUND".to_string())?;
    let mut prefix_pages = Vec::<ObjectId>::new();
    if input.cover_page {
        let mut lines = vec![
            "BO HO SO DIEN TU".to_string(),
            "VKS ECMS Export Bundle".to_string(),
            format!("So tai lieu: {}", document_titles.len()),
            format!("So trang scan goc: {}", total_scan_pages),
            "Ban scan goc duoc giu nguyen trong cac trang tiep theo.".to_string(),
        ];
        if input.include_ocr_text {
            lines.push(
                "OCR text layer: da yeu cau, can engine layer builder de dong goi text an."
                    .to_string(),
            );
        }
        prefix_pages.push(add_text_page(&mut output, pages_id, font_id, &lines, 20)?);
    }
    if input.table_of_contents {
        let mut lines = vec!["MUC LUC TAI LIEU".to_string()];
        for (idx, title) in document_titles.iter().enumerate().take(34) {
            lines.push(format!("{:02}. {}", idx + 1, title));
        }
        if document_titles.len() > 34 {
            lines.push(format!(
                "... va {} tai lieu khac",
                document_titles.len() - 34
            ));
        }
        prefix_pages.push(add_text_page(&mut output, pages_id, font_id, &lines, 14)?);
    }
    if let Ok(dictionary) = pages_object.as_dict() {
        let mut dictionary = dictionary.clone();
        let mut kids = Vec::<Object>::new();
        kids.extend(prefix_pages.iter().copied().map(Object::Reference));
        kids.extend(page_order.iter().copied().map(Object::Reference));
        dictionary.set("Count", kids.len() as u32);
        dictionary.set("Kids", kids);
        output
            .objects
            .insert(pages_id, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_object.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_id);
        if input.table_of_contents {
            dictionary.set("PageMode", "UseOutlines");
        }
        dictionary.remove(b"Outlines");
        output
            .objects
            .insert(catalog_id, Object::Dictionary(dictionary));
    }

    output.trailer.set("Root", catalog_id);
    output.max_id = output.objects.len() as u32;
    output.renumber_objects();
    output.adjust_zero_pages();
    if input.table_of_contents {
        if let Some(outline_id) = output.build_outline() {
            if let Ok(Object::Dictionary(dict)) = output.get_object_mut(catalog_id) {
                dict.set("Outlines", Object::Reference(outline_id));
            }
        }
    }
    let managed_output_path = storage::export_path_for_requested(&input.output_path)?;
    let managed_output = managed_output_path.to_string_lossy().to_string();
    save_export_with_retry(&mut output, &managed_output)?;

    Ok(ExportPdfResult {
        output_path: managed_output,
        merged_documents: page_number as i64 - 1,
        skipped_documents: skipped,
    })
}
