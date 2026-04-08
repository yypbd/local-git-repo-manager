use std::path::{Path, PathBuf};

pub fn recommended_data_root() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("local-git-repo-manager");
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        return PathBuf::from(app_data).join("local-git-repo-manager");
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join("local-git-repo-manager");
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("local-git-repo-manager");
    }

    #[allow(unreachable_code)]
    PathBuf::from(".").join("local-git-repo-manager")
}

pub fn normalize_path_for_compare(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("path cannot be empty".to_string());
    }

    let candidate = Path::new(trimmed);
    let absolute = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("failed to resolve current dir: {e}"))?
            .join(candidate)
    };

    let normalized = if absolute.exists() {
        absolute
            .canonicalize()
            .map_err(|e| format!("failed to canonicalize path: {e}"))?
    } else {
        absolute
    };

    let as_string = normalized.to_string_lossy().to_string();
    #[cfg(target_os = "windows")]
    {
        return Ok(as_string.to_lowercase());
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(as_string)
    }
}

/// Git 등에서 `current_dir`로 쓸 경로를 `normalize_path_for_compare`와 동일하게 맞춥니다.
pub fn resolve_repo_workdir(raw: &str) -> Result<String, String> {
    let normalized = normalize_path_for_compare(raw.trim())?;
    if !Path::new(&normalized).is_dir() {
        return Err("repo path is not a directory".to_string());
    }
    Ok(normalized)
}

pub fn is_network_path(path: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        path.starts_with("\\\\")
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.starts_with("smb://")
            || path.starts_with("nfs://")
            || path.starts_with("/net/")
            || path.starts_with("/mnt/")
    }
}
