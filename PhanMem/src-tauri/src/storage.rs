use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const STORAGE_DIR_NAME: &str = "VKS_ECMS_Data";

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub fn managed_root() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("STORAGE_EXE_PATH_FAILED: {e}"))?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| "STORAGE_EXE_DIR_NOT_FOUND".to_string())?;
    Ok(exe_dir.join(STORAGE_DIR_NAME))
}

pub fn originals_dir() -> Result<PathBuf, String> {
    Ok(managed_root()?.join("originals"))
}

pub fn processed_dir() -> Result<PathBuf, String> {
    Ok(managed_root()?.join("processed"))
}

pub fn exports_dir() -> Result<PathBuf, String> {
    Ok(managed_root()?.join("exports"))
}

pub fn processing_dir() -> Result<PathBuf, String> {
    Ok(managed_root()?.join("processing"))
}

pub fn reviewed_dir() -> Result<PathBuf, String> {
    Ok(managed_root()?.join("reviewed"))
}

pub fn managed_dir() -> Result<PathBuf, String> {
    Ok(managed_root()?.join("managed"))
}

pub fn ensure_managed_dirs() -> Result<(), String> {
    let dirs = [
        originals_dir()?,
        processed_dir()?,
        exports_dir()?,
        processing_dir()?,
        reviewed_dir()?,
        managed_dir()?,
    ];
    for dir in &dirs {
        fs::create_dir_all(dir)
            .map_err(|e| format!("STORAGE_CREATE_DIR_FAILED:{}:{e}", dir.display()))?;
    }
    Ok(())
}

pub fn safe_component(value: &str, fallback: &str) -> String {
    let raw = if value.trim().is_empty() {
        fallback
    } else {
        value.trim()
    };
    let mut out = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' || c == '.' {
                '_'
            } else {
                '_'
            }
        })
        .collect::<String>();
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        fallback.to_string()
    } else {
        out
    }
}

pub fn copy_to_originals(
    source: &Path,
    case_code: &str,
    document_id: &str,
    original_filename: &str,
) -> Result<PathBuf, String> {
    ensure_managed_dirs()?;
    let case_dir = originals_dir()?.join(safe_component(case_code, "case"));
    fs::create_dir_all(&case_dir)
        .map_err(|e| format!("STORAGE_CREATE_CASE_DIR_FAILED:{}:{e}", case_dir.display()))?;

    let filename = safe_component(original_filename, "document");
    let target = case_dir.join(format!(
        "{}_{}",
        safe_component(document_id, "doc"),
        filename
    ));
    fs::copy(source, &target).map_err(|e| {
        format!(
            "STORAGE_COPY_ORIGINAL_FAILED:{}->{}:{e}",
            source.display(),
            target.display()
        )
    })?;
    Ok(target)
}

pub fn ocr_pages_dir(document_id: &str) -> Result<PathBuf, String> {
    ensure_managed_dirs()?;
    let dir = processed_dir()?
        .join("ocr_pages")
        .join(safe_component(document_id, "doc"));
    fs::create_dir_all(&dir)
        .map_err(|e| format!("STORAGE_CREATE_OCR_DIR_FAILED:{}:{e}", dir.display()))?;
    Ok(dir)
}

pub fn export_path_for_requested(requested_path: &str) -> Result<PathBuf, String> {
    ensure_managed_dirs()?;
    let requested = Path::new(requested_path);
    let filename = requested
        .file_name()
        .and_then(|v| v.to_str())
        .map(|v| safe_component(v, "export.pdf"))
        .unwrap_or_else(|| format!("export_{}.pdf", now_millis()));
    let filename = if filename.to_lowercase().ends_with(".pdf") {
        filename
    } else {
        format!("{filename}.pdf")
    };
    Ok(exports_dir()?.join(filename))
}

pub fn is_managed_path(path: &Path) -> bool {
    let Ok(root) = managed_root() else {
        return false;
    };
    let Ok(root_abs) = root.canonicalize() else {
        return false;
    };
    let Ok(path_abs) = path.canonicalize() else {
        return false;
    };
    path_abs.starts_with(root_abs)
}
