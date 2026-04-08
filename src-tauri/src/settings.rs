use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalTool {
    pub id: String,
    pub label: String,
    pub command: String,
    pub args_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub data_root_path: Option<String>,
    pub locale: String,
    pub git_executable_path: Option<String>,
    pub log_mask_sensitive: bool,
    pub external_tools: Vec<ExternalTool>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            data_root_path: None,
            locale: "ko".to_string(),
            git_executable_path: None,
            log_mask_sensitive: true,
            external_tools: Vec::new(),
        }
    }
}
