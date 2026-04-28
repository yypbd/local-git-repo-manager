use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::settings::AppSettings;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedProject {
    pub id: String,
    pub name: String,
    pub root_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedState {
    pub schema_version: u32,
    pub projects: Vec<PersistedProject>,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            projects: Vec::new(),
        }
    }
}

fn app_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".local-git-repo-manager")
}

pub fn default_state_path() -> PathBuf {
    app_data_dir().join("state.json")
}

/// UI 설정(`AppSettings`) — 프로젝트 목록 `state.json`과 별도 파일
pub fn app_settings_path() -> PathBuf {
    app_data_dir().join("app-settings.json")
}

pub fn load_app_settings() -> Result<AppSettings, String> {
    let path = app_settings_path();
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("failed to read app settings: {e}"))?;
    serde_json::from_str::<AppSettings>(&raw).map_err(|e| format!("failed to parse app settings: {e}"))
}

pub fn save_app_settings(settings: &AppSettings) -> Result<(), String> {
    let path = app_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create app data dir: {e}"))?;
    }
    let tmp_path = path.with_extension("json.tmp");
    let serialized = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("failed to serialize app settings: {e}"))?;
    fs::write(&tmp_path, serialized).map_err(|e| format!("failed to write temp app settings: {e}"))?;
    fs::rename(&tmp_path, &path).map_err(|e| format!("failed to replace app settings: {e}"))?;
    Ok(())
}

pub fn load_state(path: &Path) -> Result<PersistedState, String> {
    if !path.exists() {
        return Ok(PersistedState::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read state.json: {e}"))?;
    let parsed: PersistedState =
        serde_json::from_str(&raw).map_err(|e| format!("failed to parse state.json: {e}"))?;
    migrate_state(parsed)
}

pub fn save_state_atomic(path: &Path, state: &PersistedState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create state dir: {e}"))?;
    }

    let tmp_path = path.with_extension("json.tmp");
    let serialized =
        serde_json::to_string_pretty(state).map_err(|e| format!("failed to serialize state: {e}"))?;
    fs::write(&tmp_path, serialized).map_err(|e| format!("failed to write temp state: {e}"))?;
    fs::rename(&tmp_path, path).map_err(|e| format!("failed to replace state atomically: {e}"))?;
    Ok(())
}

pub fn backup_state_if_stale(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let Some(parent) = path.parent() else {
        return Ok(());
    };

    let backup_dir = parent.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| format!("failed to create backup dir: {e}"))?;
    let backup_path = backup_dir.join("state.backup.json");

    let should_backup = if !backup_path.exists() {
        true
    } else {
        let modified = fs::metadata(&backup_path)
            .and_then(|meta| meta.modified())
            .map_err(|e| format!("failed to inspect backup file: {e}"))?;
        let elapsed = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::from_secs(0));
        elapsed >= Duration::from_secs(60 * 60 * 24 * 7)
    };

    if should_backup {
        let tmp_path = backup_path.with_extension("json.tmp");
        fs::copy(path, &tmp_path).map_err(|e| format!("failed to copy state backup: {e}"))?;
        fs::rename(&tmp_path, &backup_path)
            .map_err(|e| format!("failed to replace state backup: {e}"))?;
    }

    Ok(())
}

pub fn migrate_state(state: PersistedState) -> Result<PersistedState, String> {
    if state.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(format!(
            "unsupported schemaVersion {} (current {})",
            state.schema_version, CURRENT_SCHEMA_VERSION
        ));
    }

    if state.schema_version == CURRENT_SCHEMA_VERSION {
        return Ok(state);
    }

    // Future migration hook chain (example):
    // if state.schema_version == 0 {
    //   return migrate_v0_to_v1(state);
    // }
    Err(format!(
        "missing migration path for schemaVersion {}",
        state.schema_version
    ))
}
