//! [github/gitignore](https://github.com/github/gitignore) 저장소에서 템플릿을 받아 로컬 캐시합니다.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::persistence;

const GITHUB_TREE_URL: &str =
    "https://api.github.com/repos/github/gitignore/git/trees/main?recursive=1";
const RAW_BASE: &str = "https://raw.githubusercontent.com/github/gitignore/main/";
const MAX_TEMPLATE_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateItem {
    pub name: String,
    pub content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSyncResult {
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheFile {
    #[serde(default)]
    source: String,
    items: Vec<TemplateItem>,
    #[serde(default)]
    updated_at_ms: u64,
}

#[derive(Deserialize)]
struct TreeResponse {
    tree: Vec<TreeEntry>,
}

#[derive(Deserialize)]
struct TreeEntry {
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
}

fn cache_path() -> PathBuf {
    persistence::default_state_path()
        .parent()
        .map(|p| p.join("gitignore-templates-cache.json"))
        .unwrap_or_else(|| PathBuf::from("gitignore-templates-cache.json"))
}

fn template_name_from_repo_path(repo_path: &str) -> String {
    repo_path
        .strip_suffix(".gitignore")
        .unwrap_or(repo_path)
        .to_string()
}

fn builtin_templates() -> Vec<TemplateItem> {
    vec![
        TemplateItem {
            name: "Node (builtin)".to_string(),
            content: "node_modules/\ndist/\n.env\n".to_string(),
        },
        TemplateItem {
            name: "Rust (builtin)".to_string(),
            content: "/target\nCargo.lock\n".to_string(),
        },
    ]
}

pub fn list_templates() -> Vec<TemplateItem> {
    let path = cache_path();
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(cache) = serde_json::from_str::<CacheFile>(&raw) {
            if !cache.items.is_empty() {
                return cache.items;
            }
        }
    }
    builtin_templates()
}

fn github_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("local-git-repo-manager/0.1 (+https://github.com/github/gitignore sync)")
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("HTTP 클라이언트 생성 실패: {e}"))
}

fn fetch_tree(client: &reqwest::blocking::Client) -> Result<Vec<String>, String> {
    let tree: TreeResponse = client
        .get(GITHUB_TREE_URL)
        .send()
        .map_err(|e| format!("GitHub API 연결 실패: {e}"))?
        .error_for_status()
        .map_err(|e| format!("GitHub API 오류: {e}"))?
        .json()
        .map_err(|e| format!("GitHub 트리 응답 파싱 실패: {e}"))?;

    Ok(tree
        .tree
        .into_iter()
        .filter(|e| e.entry_type == "blob" && e.path.ends_with(".gitignore"))
        .map(|e| e.path)
        .collect())
}

fn fetch_raw(client: &reqwest::blocking::Client, repo_path: &str) -> Result<String, String> {
    let url = format!("{RAW_BASE}{repo_path}");
    let text = client
        .get(&url)
        .send()
        .map_err(|e| format!("{repo_path}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("{repo_path}: {e}"))?
        .text()
        .map_err(|e| format!("{repo_path}: {e}"))?;
    if text.len() > MAX_TEMPLATE_BYTES {
        return Err(format!("{repo_path}: 파일이 너무 큼"));
    }
    Ok(text)
}

pub fn sync_from_github() -> Result<TemplateSyncResult, String> {
    let client = github_client()?;
    let paths = fetch_tree(&client)?;
    if paths.is_empty() {
        return Err("GitHub에서 .gitignore 목록을 찾지 못했습니다.".to_string());
    }

    let items: Vec<TemplateItem> = paths
        .par_iter()
        .filter_map(|repo_path| match fetch_raw(&client, repo_path) {
            Ok(content) => Some(TemplateItem {
                name: template_name_from_repo_path(repo_path),
                content,
            }),
            Err(e) => {
                eprintln!("gitignore template skip: {e}");
                None
            }
        })
        .collect();

    if items.is_empty() {
        return Err(
            "템플릿을 하나도 받지 못했습니다. 네트워크와 GitHub 접근을 확인하세요.".to_string(),
        );
    }

    let mut sorted = items;
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let updated_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let cache = CacheFile {
        source: "github/gitignore".to_string(),
        items: sorted.clone(),
        updated_at_ms,
    };
    let json =
        serde_json::to_string_pretty(&cache).map_err(|e| format!("캐시 직렬화 실패: {e}"))?;
    let path = cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("캐시 디렉터리 생성 실패: {e}"))?;
    }
    fs::write(&path, json).map_err(|e| format!("캐시 저장 실패: {e}"))?;

    Ok(TemplateSyncResult {
        count: sorted.len(),
    })
}
