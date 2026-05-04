// PhanMem/src-tauri/src/commands/system_cmd.rs
use crate::commands::module_cmd::DbState;
use crate::storage;
use serde::Serialize;
use sysinfo::{System, Disks};
use tauri::State;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub status: String, // "pass", "warning", "fail"
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct StartupCheckPayload {
    pub status: String, // "pass", "warning", "fail"
    pub checks: Vec<CheckResult>,
    pub fatal_errors: Vec<String>,
}

#[tauri::command]
pub async fn run_startup_self_check(state: State<'_, DbState>) -> Result<StartupCheckPayload, String> {
    let mut checks = Vec::new();
    let mut fatal_errors = Vec::new();
    let mut overall_status = "pass".to_string();

    // 1. Check RAM
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_ram_gb = sys.total_memory() / 1024 / 1024 / 1024;
    if total_ram_gb < 4 {
        checks.push(CheckResult {
            name: "RAM".to_string(),
            status: "fail".to_string(),
            message: format!("RAM ({total_ram_gb}GB) thieu (yeu cau >= 4GB)"),
        });
        overall_status = "fail".to_string();
        fatal_errors.push("He thong yeu cau toi thieu 4GB RAM de chay OCR.".to_string());
    } else if total_ram_gb < 8 {
        checks.push(CheckResult {
            name: "RAM".to_string(),
            status: "warning".to_string(),
            message: format!("RAM ({total_ram_gb}GB) hoi thap (khuyen nghi >= 8GB)"),
        });
        if overall_status == "pass" { overall_status = "warning".to_string(); }
    } else {
        checks.push(CheckResult {
            name: "RAM".to_string(),
            status: "pass".to_string(),
            message: format!("RAM ({total_ram_gb}GB) OK"),
        });
    }

    // 2. Check Disk Space
    let disks = Disks::new_with_refreshed_list();
    let root_dir = storage::managed_root().map_err(|e| e.to_string())?;
    let mut disk_ok = false;
    for disk in &disks {
        if root_dir.starts_with(disk.mount_point()) {
            let free_gb = disk.available_space() / 1024 / 1024 / 1024;
            if free_gb < 2 {
                checks.push(CheckResult {
                    name: "Disk".to_string(),
                    status: "fail".to_string(),
                    message: format!("O dia con trong {free_gb}GB (yeu cau >= 2GB)"),
                });
                overall_status = "fail".to_string();
                fatal_errors.push("Khong du dung luong o dia de luu tru ho so.".to_string());
            } else if free_gb < 10 {
                checks.push(CheckResult {
                    name: "Disk".to_string(),
                    status: "warning".to_string(),
                    message: format!("O dia con trong {free_gb}GB (khuyen nghi >= 10GB)"),
                });
                if overall_status == "pass" { overall_status = "warning".to_string(); }
            } else {
                checks.push(CheckResult {
                    name: "Disk".to_string(),
                    status: "pass".to_string(),
                    message: format!("O dia OK ({free_gb}GB free)"),
                });
            }
            disk_ok = true;
            break;
        }
    }
    if !disk_ok {
        checks.push(CheckResult {
            name: "Disk".to_string(),
            status: "warning".to_string(),
            message: "Khong the xac dinh dung luong o dia".to_string(),
        });
    }

    // 3. Check SQLite
    {
        let conn = state.0.lock().map_err(|e| format!("DB_LOCK_FAILED: {e}"))?;
        match conn.query_row("SELECT 1", [], |r| r.get::<_, i32>(0)) {
            Ok(_) => {
                checks.push(CheckResult {
                    name: "Database".to_string(),
                    status: "pass".to_string(),
                    message: "SQLite OK".to_string(),
                });
            },
            Err(e) => {
                checks.push(CheckResult {
                    name: "Database".to_string(),
                    status: "fail".to_string(),
                    message: format!("Database error: {e}"),
                });
                overall_status = "fail".to_string();
                fatal_errors.push("Khong the ket noi database.".to_string());
            }
        }
    }

    // 4. Check Python / OCR Pipeline
    let python_candidates = if cfg!(windows) { vec!["python", "py"] } else { vec!["python3", "python"] };
    let mut python_found = false;
    for candidate in python_candidates {
        if Command::new(candidate).arg("--version").output().is_ok() {
            python_found = true;
            break;
        }
    }
    if python_found {
        checks.push(CheckResult {
            name: "Python".to_string(),
            status: "pass".to_string(),
            message: "Python runtime found".to_string(),
        });
    } else {
        checks.push(CheckResult {
            name: "Python".to_string(),
            status: "warning".to_string(),
            message: "Khong tim thay Python (OCR se khong hoat dong)".to_string(),
        });
        if overall_status == "pass" { overall_status = "warning".to_string(); }
    }

    Ok(StartupCheckPayload {
        status: overall_status,
        checks,
        fatal_errors,
    })
}
