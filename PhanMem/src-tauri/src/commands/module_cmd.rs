// VKS ECMS — Module config Tauri commands

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleConfig {
    pub module_id: String,
    pub enabled: bool,
    pub selected_version: String,
    pub settings: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
}

pub struct DbState(pub Mutex<Connection>);

#[tauri::command]
pub fn get_module_configs(db: State<'_, DbState>) -> Result<Vec<ModuleConfig>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut stmt = conn
        .prepare("SELECT module_id, enabled, selected_version, settings, updated_at FROM module_configs")
        .map_err(|e| format!("Prepare failed: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ModuleConfig {
                module_id: row.get(0)?,
                enabled: row.get::<_, i32>(1)? != 0,
                selected_version: row.get(2)?,
                settings: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut configs = Vec::new();
    for row in rows {
        configs.push(row.map_err(|e| format!("Row read failed: {e}"))?);
    }
    Ok(configs)
}

#[tauri::command]
pub fn set_module_config(
    db: State<'_, DbState>,
    module_id: String,
    enabled: bool,
    selected_version: String,
    settings: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    conn.execute(
        "INSERT INTO module_configs (module_id, enabled, selected_version, settings, updated_at)
         VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         ON CONFLICT(module_id) DO UPDATE SET
             enabled = excluded.enabled,
             selected_version = excluded.selected_version,
             settings = excluded.settings,
             updated_at = excluded.updated_at",
        rusqlite::params![module_id, enabled as i32, selected_version, settings],
    )
    .map_err(|e| format!("Upsert failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn get_app_settings(db: State<'_, DbState>) -> Result<Vec<AppSetting>, String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| format!("Prepare failed: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut settings = Vec::new();
    for row in rows {
        settings.push(row.map_err(|e| format!("Row read failed: {e}"))?);
    }
    Ok(settings)
}

#[tauri::command]
pub fn set_app_setting(
    db: State<'_, DbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = excluded.updated_at",
        rusqlite::params![key, value],
    )
    .map_err(|e| format!("Upsert failed: {e}"))?;
    Ok(())
}
