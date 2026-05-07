use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Default)]
struct ResourceContext {
    resource_dir: Option<PathBuf>,
    managed_root: Option<PathBuf>,
    runtime_bundle_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PayloadManifest {
    pub offline_bundle: bool,
    #[serde(default)]
    pub bundle_profile: Option<String>,
    pub payload_name: String,
    pub payload_relative_path: String,
    pub payload_sha256: String,
    #[serde(default)]
    pub payload_size: u64,
    pub extract_dir_name: String,
    #[serde(default)]
    pub contains: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub required_components: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub optional_components: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeBundleState {
    pub payload_manifest_path: PathBuf,
    pub payload_archive_path: PathBuf,
    pub runtime_bundle_dir: PathBuf,
    pub payload_name: String,
    pub extract_dir_name: String,
    pub payload_sha256_expected: String,
    pub payload_sha256_actual: String,
    pub payload_size_expected: u64,
    pub payload_size_actual: u64,
    pub extracted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PayloadMarker {
    payload_name: String,
    payload_sha256: String,
    payload_size: u64,
}

static RESOURCE_CONTEXT: OnceLock<Mutex<ResourceContext>> = OnceLock::new();

fn context_cell() -> &'static Mutex<ResourceContext> {
    RESOURCE_CONTEXT.get_or_init(|| Mutex::new(ResourceContext::default()))
}

fn current_context() -> ResourceContext {
    context_cell()
        .lock()
        .map(|ctx| ctx.clone())
        .unwrap_or_default()
}

pub fn set_resource_context(
    resource_dir: Option<PathBuf>,
    managed_root: Option<PathBuf>,
    runtime_bundle_dir: Option<PathBuf>,
) {
    if let Ok(mut ctx) = context_cell().lock() {
        *ctx = ResourceContext {
            resource_dir,
            managed_root,
            runtime_bundle_dir,
        };
    }
}

fn app_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
}

pub fn executable_path() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

pub fn runtime_mode() -> &'static str {
    if cfg!(debug_assertions) {
        "DEV"
    } else {
        // Check if the release exe is inside the source repo (local test)
        let exe_in_repo = std::env::current_exe()
            .ok()
            .map(|p| {
                let s = p.display().to_string().to_ascii_lowercase();
                s.contains("target\\release")
                    || s.contains("target/release")
                    || s.contains("src-tauri")
            })
            .unwrap_or(false);
        if exe_in_repo {
            "RELEASE_LOCAL_TEST"
        } else {
            "INSTALLED_RELEASE"
        }
    }
}

pub fn is_release_portable_mode() -> bool {
    !cfg!(debug_assertions)
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

#[cfg(debug_assertions)]
fn dev_resources_dir() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("resources");
    path.exists().then_some(path)
}

#[cfg(not(debug_assertions))]
fn dev_resources_dir() -> Option<PathBuf> {
    None
}

#[cfg(debug_assertions)]
fn dev_python_dir() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("python");
    path.exists().then_some(path)
}

#[cfg(not(debug_assertions))]
fn dev_python_dir() -> Option<PathBuf> {
    None
}

pub fn resolve_runtime_bundle_dir() -> PathBuf {
    let ctx = current_context();
    if let Some(path) = ctx.runtime_bundle_dir {
        return path;
    }
    if let Some(root) = ctx.managed_root {
        if let Some(parent) = root.parent() {
            return parent.join("VKS_ECMS").join("runtime_bundle");
        }
        return root.join("runtime_bundle");
    }
    if let Some(exe_dir) = app_dir() {
        return exe_dir.join("VKS_ECMS").join("runtime_bundle");
    }
    PathBuf::from("runtime_bundle")
}

pub fn resource_roots() -> Vec<PathBuf> {
    let mut roots = Vec::<PathBuf>::new();
    let ctx = current_context();

    if cfg!(debug_assertions) {
        push_unique(&mut roots, resolve_runtime_bundle_dir());
        if let Some(dev_resources) = dev_resources_dir() {
            push_unique(&mut roots, dev_resources);
        }
    }

    if let Some(resource_dir) = ctx.resource_dir {
        push_unique(&mut roots, resource_dir);
    }
    if let Some(exe_dir) = app_dir() {
        push_unique(&mut roots, exe_dir.join("resources"));
        push_unique(&mut roots, exe_dir.clone());
        if let Some(parent) = exe_dir.parent() {
            push_unique(&mut roots, parent.join("resources"));
        }
    }

    if !cfg!(debug_assertions) {
        push_unique(&mut roots, resolve_runtime_bundle_dir());
        if let Some(managed_root) = ctx.managed_root {
            push_unique(&mut roots, managed_root);
        }
    }

    roots
}

fn app_resource_roots() -> Vec<PathBuf> {
    let mut roots = Vec::<PathBuf>::new();
    let ctx = current_context();
    if let Some(resource_dir) = ctx.resource_dir {
        push_unique(&mut roots, resource_dir);
    }
    if let Some(exe_dir) = app_dir() {
        push_unique(&mut roots, exe_dir.join("resources"));
        push_unique(&mut roots, exe_dir.clone());
        if let Some(parent) = exe_dir.parent() {
            push_unique(&mut roots, parent.join("resources"));
        }
    }
    roots
}

fn component_relative_candidates(component_name: &str) -> Vec<PathBuf> {
    let normalized = component_name.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "payload_manifest" | "payload_manifest.json" => {
            vec![PathBuf::from("config/payload_manifest.json")]
        }
        "payload" | "payload_archive" | "offline_runtime_payload.zip" => {
            vec![PathBuf::from("payload/offline_runtime_payload.zip")]
        }
        "python_embedded" | "python" => vec![PathBuf::from("runtime/python_embedded")],
        "python_sources" | "python_source" => vec![
            PathBuf::from("runtime/python_embedded/scripts"),
            PathBuf::from("python"),
        ],
        "python_scripts" | "ocr_worker" => vec![PathBuf::from("runtime/python_embedded/scripts")],
        "pdfium" => vec![PathBuf::from("runtime/pdfium")],
        "model_manifest" | "model_manifest.json" => {
            vec![PathBuf::from("config/model_manifest.json")]
        }
        "installer_manifest" | "installer_manifest.json" => {
            vec![PathBuf::from("config/installer_manifest.json")]
        }
        "ocr_models" | "ocr_model_dir" => vec![PathBuf::from("ocr/models")],
        "ollama" | "ollama.exe" => vec![PathBuf::from("ai/ollama/ollama.exe")],
        "qwen" | "qwen_model" | "qwen_models" => vec![PathBuf::from("ai/models")],
        other => vec![PathBuf::from(other.replace('\\', "/"))],
    }
}

pub fn component_candidate_paths(component_name: &str) -> Vec<PathBuf> {
    let rels = component_relative_candidates(component_name);
    let mut paths = Vec::<PathBuf>::new();
    let normalized = component_name.trim().to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "ollama" | "ollama.exe" | "qwen" | "qwen_model" | "qwen_models"
    ) {
        for rel in &rels {
            push_unique(&mut paths, resolve_runtime_bundle_dir().join(rel));
        }
        for root in app_resource_roots() {
            for rel in &rels {
                push_unique(&mut paths, root.join(rel));
            }
        }
        if cfg!(debug_assertions) {
            if let Some(dev_resources) = dev_resources_dir() {
                for rel in &rels {
                    push_unique(&mut paths, dev_resources.join(rel));
                }
            }
        }
        return paths;
    }
    for root in resource_roots() {
        for rel in &rels {
            push_unique(&mut paths, root.join(rel));
        }
    }
    paths
}

pub fn resolve_resource_path(component_name: &str) -> Option<PathBuf> {
    component_candidate_paths(component_name)
        .into_iter()
        .find(|path| path.exists())
}

pub fn describe_component_candidates(component_name: &str) -> String {
    component_candidate_paths(component_name)
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(" | ")
}

pub fn resolve_payload_manifest() -> Option<PathBuf> {
    resolve_resource_path("payload_manifest").filter(|path| path.exists() && path.is_file())
}

pub fn resolve_payload_archive() -> Option<PathBuf> {
    if let Ok((manifest, manifest_path)) = load_payload_manifest() {
        if let Some(root) = resource_root_for_path(&manifest_path) {
            let candidate = root.join(&manifest.payload_relative_path);
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    resolve_resource_path("payload_archive").filter(|path| path.exists() && path.is_file())
}

pub fn resolve_python_embedded() -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        "python.exe"
    } else {
        "bin/python3"
    };
    component_candidate_paths("python_embedded")
        .into_iter()
        .map(|dir| dir.join(exe_name))
        .find(|path| path.exists() && path.is_file())
}

pub fn resolve_python_embedded_dir() -> Option<PathBuf> {
    resolve_python_embedded().and_then(|python| python.parent().map(|parent| parent.to_path_buf()))
}

pub fn resolve_python_scripts_dir() -> Option<PathBuf> {
    resolve_python_embedded_dir()
        .map(|dir| dir.join("scripts"))
        .filter(|path| path.exists() && path.is_dir())
}

pub fn resolve_python_source_dir() -> Option<PathBuf> {
    #[cfg(debug_assertions)]
    {
        if let Some(path) = dev_python_dir().filter(|path| {
            path.join("ocr").join("ocr_pipeline.py").exists()
                || path.join("parsers").join("docx_parser.py").exists()
        }) {
            return Some(path);
        }
    }
    #[cfg(not(debug_assertions))]
    {
        if let Some(path) = resolve_python_scripts_dir().filter(|path| {
            path.join("ocr").join("pdf_to_images.py").exists()
                || path.join("ocr").join("ocr_pipeline.py").exists()
                || path.join("ocr_worker.py").exists()
        }) {
            return Some(path);
        }
    }
    if let Some(path) = resolve_resource_path("python_sources").filter(|path| {
        path.join("ocr").join("ocr_pipeline.py").exists()
            || path.join("parsers").join("docx_parser.py").exists()
    }) {
        return Some(path);
    }
    dev_python_dir()
}

pub fn resolve_pdfium() -> Option<PathBuf> {
    let dll_name = if cfg!(windows) {
        "pdfium.dll"
    } else {
        "libpdfium.so"
    };
    component_candidate_paths("pdfium")
        .into_iter()
        .flat_map(|dir| [dir.join(dll_name), dir])
        .find(|path| path.exists() && path.is_file())
}

pub fn resolve_model_manifest() -> Option<PathBuf> {
    resolve_resource_path("model_manifest").filter(|path| path.exists() && path.is_file())
}

pub fn resolve_installer_manifest() -> Option<PathBuf> {
    resolve_resource_path("installer_manifest").filter(|path| path.exists() && path.is_file())
}

pub fn resolve_ocr_model_dir() -> Option<PathBuf> {
    resolve_resource_path("ocr_models").filter(|path| path.exists() && path.is_dir())
}

pub fn resolve_ollama_path_optional() -> Option<PathBuf> {
    resolve_resource_path("ollama").filter(|path| path.exists() && path.is_file())
}

pub fn resolve_qwen_models_optional() -> Option<PathBuf> {
    resolve_resource_path("qwen_models").filter(|path| path.exists() && path.is_dir())
}

pub fn resolve_ollama_path() -> Option<PathBuf> {
    resolve_ollama_path_optional()
}

pub fn resolve_qwen_model_dir() -> Option<PathBuf> {
    resolve_qwen_models_optional()
}

pub fn bundle_profile() -> String {
    if let Ok(value) = std::env::var("VKS_BUNDLE_PROFILE") {
        let normalized = value.trim().to_ascii_lowercase();
        if normalized == "offline_ai_full" || normalized == "offline_ocr" {
            return normalized;
        }
    }
    if let Ok((manifest, _)) = load_payload_manifest() {
        if let Some(profile) = manifest.bundle_profile {
            let normalized = profile.trim().to_ascii_lowercase();
            if normalized == "offline_ai_full" || normalized == "offline_ocr" {
                return normalized;
            }
        }
    }
    if let Some(installer_path) = resolve_installer_manifest() {
        if let Ok(raw) = fs::read_to_string(installer_path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(profile) = value.get("bundle_profile").and_then(|v| v.as_str()) {
                    let normalized = profile.trim().to_ascii_lowercase();
                    if normalized == "offline_ai_full" || normalized == "offline_ocr" {
                        return normalized;
                    }
                }
            }
        }
    }
    "offline_ocr".to_string()
}

pub fn bundle_profile_label(profile: &str) -> &'static str {
    if profile == "offline_ai_full" {
        "AI Full"
    } else {
        "OCR-only"
    }
}

pub fn resource_root_for_path(path: &Path) -> Option<PathBuf> {
    for root in resource_roots() {
        if path.starts_with(&root) {
            return Some(root);
        }
    }
    let parent = path.parent()?;
    if parent.file_name().and_then(|name| name.to_str()) == Some("config") {
        return parent.parent().map(|root| root.to_path_buf());
    }
    parent.parent().map(|root| root.to_path_buf())
}

pub fn load_payload_manifest() -> Result<(PayloadManifest, PathBuf), String> {
    let manifest_path = resolve_payload_manifest().ok_or_else(|| {
        format!(
            "payload_manifest.json missing; checked: {}",
            describe_component_candidates("payload_manifest")
        )
    })?;
    let raw = fs::read_to_string(&manifest_path).map_err(|e| {
        format!(
            "READ_PAYLOAD_MANIFEST_FAILED:{}:{e}",
            manifest_path.display()
        )
    })?;
    let manifest = serde_json::from_str::<PayloadManifest>(&raw).map_err(|e| {
        format!(
            "PARSE_PAYLOAD_MANIFEST_FAILED:{}:{e}",
            manifest_path.display()
        )
    })?;
    if !manifest.offline_bundle {
        return Err(format!(
            "payload_manifest.json offline_bundle must be true: {}",
            manifest_path.display()
        ));
    }
    if manifest.payload_relative_path.trim().is_empty() {
        return Err(format!(
            "payload_manifest.json missing payload_relative_path: {}",
            manifest_path.display()
        ));
    }
    if manifest.payload_sha256.trim().is_empty() {
        return Err(format!(
            "payload_manifest.json missing payload_sha256: {}",
            manifest_path.display()
        ));
    }
    if manifest.extract_dir_name.trim().is_empty() {
        return Err(format!(
            "payload_manifest.json missing extract_dir_name: {}",
            manifest_path.display()
        ));
    }
    Ok((manifest, manifest_path))
}

#[allow(dead_code)]
fn collect_files_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(path).map_err(|e| format!("READ_DIR_FAILED:{}:{e}", path.display()))?
    {
        let entry = entry.map_err(|e| format!("READ_DIR_ENTRY_FAILED:{}:{e}", path.display()))?;
        let child = entry.path();

        let file_name = child
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if file_name == ".ds_store"
            || file_name == "thumbs.db"
            || file_name == ".payload_state.json"
            || file_name == ".payload_installed.json"
            || file_name == "__pycache__"
        {
            continue;
        }

        if child.is_dir() {
            collect_files_recursive(&child, files)?;
        } else if child.is_file() {
            files.push(child);
        }
    }
    Ok(())
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("OPEN_FAILED:{}:{e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|e| format!("READ_FAILED:{}:{e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[allow(dead_code)]
pub fn sha256_path(path: &Path) -> Result<String, String> {
    if path.is_file() {
        return sha256_file(path);
    }
    if !path.is_dir() {
        return Err(format!("MODEL_PATH_NOT_FILE_OR_DIR:{}", path.display()));
    }
    let mut files = Vec::<PathBuf>::new();
    collect_files_recursive(path, &mut files)?;
    if files.is_empty() {
        return Err(format!("MODEL_DIR_EMPTY:{}", path.display()));
    }

    let mut entries: Vec<(String, PathBuf)> = files
        .into_iter()
        .map(|file| {
            let relative = file
                .canonicalize()
                .unwrap_or_else(|_| file.clone())
                .strip_prefix(&path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
                .unwrap_or(&file)
                .to_string_lossy()
                .replace('\\', "/");
            (relative, file)
        })
        .collect();

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (relative, file) in entries {
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update(sha256_file(&file)?.as_bytes());
        hasher.update([0]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn payload_marker_path(runtime_bundle_dir: &Path) -> PathBuf {
    runtime_bundle_dir.join(".payload_state.json")
}

fn read_payload_marker(runtime_bundle_dir: &Path) -> Option<PayloadMarker> {
    let path = payload_marker_path(runtime_bundle_dir);
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<PayloadMarker>(&raw).ok()
}

fn write_payload_marker(
    runtime_bundle_dir: &Path,
    payload_name: &str,
    payload_sha256: &str,
    payload_size: u64,
) -> Result<(), String> {
    let marker = PayloadMarker {
        payload_name: payload_name.to_string(),
        payload_sha256: payload_sha256.to_string(),
        payload_size,
    };
    let path = payload_marker_path(runtime_bundle_dir);
    let raw = serde_json::to_string_pretty(&marker)
        .map_err(|e| format!("PAYLOAD_MARKER_SERIALIZE_FAILED:{e}"))?;
    fs::write(&path, raw)
        .map_err(|e| format!("WRITE_PAYLOAD_MARKER_FAILED:{}:{e}", path.display()))?;
    Ok(())
}

fn bundle_contains_ready(runtime_bundle_dir: &Path, manifest: &PayloadManifest) -> bool {
    if !runtime_bundle_dir.exists() || !runtime_bundle_dir.is_dir() {
        return false;
    }
    for rel in &manifest.contains {
        let candidate = runtime_bundle_dir.join(rel);
        if !candidate.exists() {
            return false;
        }
    }
    true
}

fn runtime_bundle_matches_payload(
    runtime_bundle_dir: &Path,
    manifest: &PayloadManifest,
    actual_sha256: &str,
    actual_size: u64,
) -> bool {
    let Some(marker) = read_payload_marker(runtime_bundle_dir) else {
        return false;
    };
    marker.payload_sha256.eq_ignore_ascii_case(actual_sha256)
        && marker.payload_size == actual_size
        && marker.payload_name == manifest.payload_name
        && bundle_contains_ready(runtime_bundle_dir, manifest)
}

fn escape_powershell_literal(value: &Path) -> String {
    value.display().to_string().replace('\'', "''")
}

#[cfg(windows)]
fn extract_payload_archive(archive_path: &Path, destination_dir: &Path) -> Result<(), String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    fs::create_dir_all(destination_dir).map_err(|e| {
        format!(
            "CREATE_PAYLOAD_TMP_DIR_FAILED:{}:{e}",
            destination_dir.display()
        )
    })?;

    let archive_for_tools = destination_dir
        .parent()
        .map(|parent| parent.join("payload_extract_source.zip"))
        .unwrap_or_else(|| destination_dir.join("payload_extract_source.zip"));
    fs::copy(archive_path, &archive_for_tools).map_err(|e| {
        format!(
            "COPY_PAYLOAD_FOR_EXTRACT_FAILED:{} -> {}:{e}",
            archive_path.display(),
            archive_for_tools.display()
        )
    })?;
    let destination_for_tools =
        fs::canonicalize(destination_dir).unwrap_or_else(|_| destination_dir.to_path_buf());

    let command = format!(
        "$ErrorActionPreference='Stop'; Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
        escape_powershell_literal(&archive_for_tools),
        escape_powershell_literal(&destination_for_tools),
    );
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &command,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| {
            format!(
                "PAYLOAD_EXTRACT_START_FAILED:{}:{e}",
                archive_path.display()
            )
        })?;
    if output.status.success() {
        let _ = fs::remove_file(&archive_for_tools);
        return Ok(());
    }

    let powershell_error = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let tar_output = Command::new("tar.exe")
        .arg("-xf")
        .arg(&archive_for_tools)
        .arg("-C")
        .arg(&destination_for_tools)
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    match tar_output {
        Ok(output) if output.status.success() => {
            let _ = fs::remove_file(&archive_for_tools);
            return Ok(());
        }
        Ok(output) => {
            let _ = fs::remove_file(&archive_for_tools);
            return Err(format!(
                "PAYLOAD_EXTRACT_FAILED:{} -> {}: powershell={}; tar={}",
                archive_path.display(),
                destination_dir.display(),
                powershell_error,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Err(e) => {
            let _ = fs::remove_file(&archive_for_tools);
            return Err(format!(
                "PAYLOAD_EXTRACT_FAILED:{} -> {}: powershell={}; tar_start={e}",
                archive_path.display(),
                destination_dir.display(),
                powershell_error,
            ));
        }
    }
}

#[cfg(not(windows))]
fn extract_payload_archive(_archive_path: &Path, _destination_dir: &Path) -> Result<(), String> {
    Err("PAYLOAD_EXTRACT_UNSUPPORTED: automatic ZIP extraction is implemented for Windows builds only.".to_string())
}

pub fn ensure_runtime_bundle_ready() -> Result<RuntimeBundleState, String> {
    let (manifest, manifest_path) = load_payload_manifest()?;
    let payload_archive_path = resolve_payload_archive().ok_or_else(|| {
        format!(
            "offline_runtime_payload.zip missing; checked: {}",
            describe_component_candidates("payload_archive")
        )
    })?;
    let payload_size_actual = fs::metadata(&payload_archive_path)
        .map_err(|e| format!("PAYLOAD_STAT_FAILED:{}:{e}", payload_archive_path.display()))?
        .len();
    if manifest.payload_size > 0 && payload_size_actual != manifest.payload_size {
        return Err(format!(
            "payload size mismatch: expected={} actual={} path={}",
            manifest.payload_size,
            payload_size_actual,
            payload_archive_path.display()
        ));
    }
    let payload_sha256_actual = sha256_file(&payload_archive_path)?;
    if !payload_sha256_actual.eq_ignore_ascii_case(manifest.payload_sha256.trim()) {
        return Err(format!(
            "payload checksum mismatch: expected={} actual={} path={}",
            manifest.payload_sha256.trim(),
            payload_sha256_actual,
            payload_archive_path.display()
        ));
    }

    let runtime_bundle_dir = resolve_runtime_bundle_dir();
    let mut extracted = false;
    if !runtime_bundle_matches_payload(
        &runtime_bundle_dir,
        &manifest,
        &payload_sha256_actual,
        payload_size_actual,
    ) {
        let parent = runtime_bundle_dir.parent().ok_or_else(|| {
            format!(
                "runtime bundle parent missing: {}",
                runtime_bundle_dir.display()
            )
        })?;
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "CREATE_RUNTIME_BUNDLE_PARENT_FAILED:{}:{e}",
                parent.display()
            )
        })?;
        let temp_dir = parent.join(format!(
            "{}__extracting__{}",
            manifest.extract_dir_name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|value| value.as_millis())
                .unwrap_or(0)
        ));
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
        }
        extract_payload_archive(&payload_archive_path, &temp_dir)?;
        if !bundle_contains_ready(&temp_dir, &manifest) {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(format!(
                "payload extracted but required contents are missing under {}",
                temp_dir.display()
            ));
        }
        if runtime_bundle_dir.exists() {
            fs::remove_dir_all(&runtime_bundle_dir).map_err(|e| {
                format!(
                    "REMOVE_OLD_RUNTIME_BUNDLE_FAILED:{}:{e}",
                    runtime_bundle_dir.display()
                )
            })?;
        }
        fs::rename(&temp_dir, &runtime_bundle_dir).map_err(|e| {
            format!(
                "ACTIVATE_RUNTIME_BUNDLE_FAILED:{} -> {}:{e}",
                temp_dir.display(),
                runtime_bundle_dir.display()
            )
        })?;
        write_payload_marker(
            &runtime_bundle_dir,
            &manifest.payload_name,
            &payload_sha256_actual,
            payload_size_actual,
        )?;
        extracted = true;
    }

    if !bundle_contains_ready(&runtime_bundle_dir, &manifest) {
        return Err(format!(
            "runtime bundle is incomplete after extraction: {}",
            runtime_bundle_dir.display()
        ));
    }

    Ok(RuntimeBundleState {
        payload_manifest_path: manifest_path,
        payload_archive_path,
        runtime_bundle_dir,
        payload_name: manifest.payload_name,
        extract_dir_name: manifest.extract_dir_name,
        payload_sha256_expected: manifest.payload_sha256,
        payload_sha256_actual,
        payload_size_expected: manifest.payload_size,
        payload_size_actual,
        extracted,
    })
}
