use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitignoreReadResult {
    pub exists: bool,
    pub content: String,
}

pub fn read_gitignore(repo_path: &str) -> Result<GitignoreReadResult, String> {
    let path = std::path::Path::new(repo_path).join(".gitignore");
    if !path.exists() {
        return Ok(GitignoreReadResult {
            exists: false,
            content: String::new(),
        });
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("failed to read .gitignore: {e}"))?;
    Ok(GitignoreReadResult { exists: true, content })
}

pub fn write_gitignore(repo_path: &str, content: &str) -> Result<(), String> {
    let path = std::path::Path::new(repo_path).join(".gitignore");
    fs::write(path, content).map_err(|e| format!("failed to write .gitignore: {e}"))
}
