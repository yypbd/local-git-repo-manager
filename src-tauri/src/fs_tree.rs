use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

pub fn read_dir(path: &str) -> Result<Vec<TreeEntry>, String> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| format!("failed to read dir: {e}"))? {
        let item = entry.map_err(|e| format!("failed to read entry: {e}"))?;
        let metadata = item
            .metadata()
            .map_err(|e| format!("failed to read metadata: {e}"))?;
        entries.push(TreeEntry {
            name: item.file_name().to_string_lossy().to_string(),
            path: item.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
        });
    }
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}
