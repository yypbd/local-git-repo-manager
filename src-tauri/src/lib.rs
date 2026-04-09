pub mod app_lock;
mod fs_tree;
mod git;
mod git_log;
mod git_ops;
mod gitignore;
mod gitignore_templates;
mod paths;
mod persistence;
mod settings;
mod shell;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Project {
    id: String,
    name: String,
    root_paths: Vec<String>,
}

#[derive(Default)]
struct AppState {
    bootstrap_path: PathBuf,
    state_path: PathBuf,
    projects: Mutex<Vec<Project>>,
    settings: Mutex<settings::AppSettings>,
    git_logs: git_log::GitLogBuffer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapState {
    confirmed_data_root: Option<String>,
}

impl Default for BootstrapState {
    fn default() -> Self {
        Self {
            confirmed_data_root: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapPayload {
    recommended_data_root: String,
    confirmed_data_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RootPathCheckPayload {
    normalized_path: String,
    is_available: bool,
}

fn normalize_project_name_for_compare(name: &str) -> String {
    name.trim().to_lowercase()
}

#[tauri::command]
fn projects_list(state: tauri::State<AppState>) -> Result<Vec<Project>, String> {
    let projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;
    Ok(projects.clone())
}

#[tauri::command]
fn projects_create(name: String, state: tauri::State<AppState>) -> Result<Project, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("project name cannot be empty".to_string());
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system time error".to_string())?
        .as_nanos();
    let project = Project {
        id: format!("p-{now}"),
        name: trimmed.to_string(),
        root_paths: Vec::new(),
    };

    let mut projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;
    let normalized_new = normalize_project_name_for_compare(trimmed);
    if projects
        .iter()
        .any(|p| normalize_project_name_for_compare(&p.name) == normalized_new)
    {
        return Err("project name already exists".to_string());
    }
    projects.push(project.clone());
    persist_projects(&state.state_path, &projects)?;
    Ok(project)
}

#[tauri::command]
fn projects_update(id: String, name: String, state: tauri::State<AppState>) -> Result<Project, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("project name cannot be empty".to_string());
    }

    let mut projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;
    let normalized_new = normalize_project_name_for_compare(trimmed);
    if projects.iter().any(|p| {
        p.id != id && normalize_project_name_for_compare(&p.name) == normalized_new
    }) {
        return Err("project name already exists".to_string());
    }
    let target = projects
        .iter_mut()
        .find(|project| project.id == id)
        .ok_or_else(|| "project not found".to_string())?;
    target.name = trimmed.to_string();
    let updated = target.clone();
    persist_projects(&state.state_path, &projects)?;
    Ok(updated)
}

#[tauri::command]
fn projects_delete(id: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;
    let previous_len = projects.len();
    projects.retain(|project| project.id != id);
    if projects.len() == previous_len {
        return Err("project not found".to_string());
    }
    persist_projects(&state.state_path, &projects)?;
    Ok(())
}

#[tauri::command]
fn projects_reorder(
    ordered_ids: Vec<String>,
    state: tauri::State<AppState>,
) -> Result<Vec<Project>, String> {
    let mut projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;
    let mut sorted_existing: Vec<String> = projects.iter().map(|p| p.id.clone()).collect();
    let mut sorted_request = ordered_ids.clone();
    sorted_existing.sort();
    sorted_request.sort();
    if sorted_existing != sorted_request {
        return Err("invalid reorder: must be a permutation of project ids".to_string());
    }
    let mut map: HashMap<String, Project> = projects
        .drain(..)
        .map(|p| (p.id.clone(), p))
        .collect();
    let mut new_vec = Vec::with_capacity(ordered_ids.len());
    for id in ordered_ids {
        new_vec.push(
            map.remove(&id)
                .ok_or_else(|| "invalid reorder: internal error".to_string())?,
        );
    }
    if !map.is_empty() {
        return Err("invalid reorder: internal error".to_string());
    }
    *projects = new_vec;
    persist_projects(&state.state_path, &projects)?;
    Ok(projects.clone())
}

/// Returns `Ok(true)` if a new root was appended, `Ok(false)` if this project already had it.
/// `take_from_other`: true면 다른 프로젝트에 있던 동일 경로를 제거한 뒤 이 프로젝트에 연결(폴더 추가 UI).
/// false면 다른 프로젝트에 있으면 에러(폴더 드롭 일괄 가져오기 등).
fn try_push_root(
    projects: &mut Vec<Project>,
    project_id: &str,
    normalized: String,
    take_from_other: bool,
) -> Result<bool, String> {
    let target_idx = projects
        .iter()
        .position(|p| p.id == project_id)
        .ok_or_else(|| "project not found".to_string())?;

    let already_in_this = projects[target_idx].root_paths.iter().any(|root| {
        paths::normalize_path_for_compare(root)
            .map(|r| r == normalized)
            .unwrap_or(false)
    });
    if already_in_this {
        return Ok(false);
    }

    let taken_in_other = projects.iter().enumerate().any(|(i, p)| {
        if i == target_idx {
            return false;
        }
        p.root_paths.iter().any(|root| {
            paths::normalize_path_for_compare(root)
                .map(|r| r == normalized)
                .unwrap_or(false)
        })
    });
    if taken_in_other {
        if take_from_other {
            for (i, p) in projects.iter_mut().enumerate() {
                if i == target_idx {
                    continue;
                }
                p.root_paths.retain(|root| {
                    paths::normalize_path_for_compare(root)
                        .map(|r| r != normalized)
                        .unwrap_or(true)
                });
            }
        } else {
            return Err("root path already exists in another project".to_string());
        }
    }

    projects[target_idx].root_paths.push(normalized);
    Ok(true)
}

#[tauri::command]
fn projects_add_root(
    project_id: String,
    path: String,
    state: tauri::State<AppState>,
) -> Result<Project, String> {
    let normalized = paths::normalize_path_for_compare(&path)?;
    if paths::is_network_path(&normalized) {
        return Err("network path is not allowed".to_string());
    }
    let mut projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;

    let added = try_push_root(&mut projects, &project_id, normalized, true)?;
    let updated = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| "project not found".to_string())?
        .clone();
    if added {
        persist_projects(&state.state_path, &projects)?;
    }
    Ok(updated)
}

#[tauri::command]
fn projects_import_folder_drop(path: String, state: tauri::State<AppState>) -> Result<Project, String> {
    const ERR_OTHER_PROJECT: &str = "root path already exists in another project";

    let dropped_norm = paths::normalize_path_for_compare(&path)?;
    if paths::is_network_path(&dropped_norm) {
        return Err("network path is not allowed".to_string());
    }
    let meta = fs::metadata(&dropped_norm).map_err(|e| format!("{e}"))?;
    if !meta.is_dir() {
        return Err("not a directory".to_string());
    }

    let project_name = Path::new(&dropped_norm)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "invalid folder name".to_string())?
        .to_string();

    // 드롭한 폴더 자체는 루트로 넣지 않고, 직계 자식 디렉터리만 추가한다. 먼저 수집해 비어 있으면 프로젝트를 만들지 않는다.
    let mut to_add: Vec<String> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dropped_norm) {
        for entry in rd.flatten() {
            let ft = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if !ft.is_dir() {
                continue;
            }
            let child_path = entry.path();
            let Some(s) = child_path.to_str() else {
                continue;
            };
            if let Ok(n) = paths::normalize_path_for_compare(s) {
                if n != dropped_norm {
                    to_add.push(n);
                }
            }
        }
    }
    to_add.sort();
    to_add.dedup();

    if to_add.is_empty() {
        return Err("no subfolders to import".to_string());
    }

    let mut projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;

    let project_id = if let Some(existing) = projects.iter().find(|p| p.name == project_name) {
        existing.id.clone()
    } else {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "system time error".to_string())?
            .as_nanos();
        let id = format!("p-{now}");
        projects.push(Project {
            id: id.clone(),
            name: project_name,
            root_paths: vec![],
        });
        id
    };

    for n in to_add {
        match try_push_root(&mut projects, &project_id, n, false) {
            Ok(_) => {}
            Err(e) if e == ERR_OTHER_PROJECT => {}
            Err(e) => return Err(e),
        }
    }

    let updated = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| "project not found".to_string())?
        .clone();
    persist_projects(&state.state_path, &projects)?;
    Ok(updated)
}

#[tauri::command]
fn projects_remove_root(
    project_id: String,
    path: String,
    state: tauri::State<AppState>,
) -> Result<Project, String> {
    let normalized = paths::normalize_path_for_compare(&path)?;
    let mut projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;
    let target = projects
        .iter_mut()
        .find(|p| p.id == project_id)
        .ok_or_else(|| "project not found".to_string())?;
    target.root_paths.retain(|root| {
        paths::normalize_path_for_compare(root)
            .map(|it| it != normalized)
            .unwrap_or(true)
    });
    let updated = target.clone();
    persist_projects(&state.state_path, &projects)?;
    Ok(updated)
}

#[tauri::command]
fn move_root_to_project(
    from_project_id: String,
    to_project_id: String,
    path: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let normalized = paths::normalize_path_for_compare(&path)?;
    let mut projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;
    let from = projects
        .iter_mut()
        .find(|p| p.id == from_project_id)
        .ok_or_else(|| "source project not found".to_string())?;
    from.root_paths.retain(|root| {
        paths::normalize_path_for_compare(root)
            .map(|it| it != normalized)
            .unwrap_or(true)
    });
    let to = projects
        .iter_mut()
        .find(|p| p.id == to_project_id)
        .ok_or_else(|| "target project not found".to_string())?;
    to.root_paths.push(normalized);
    persist_projects(&state.state_path, &projects)?;
    Ok(())
}

#[tauri::command]
fn tree_readdir(path: String) -> Result<Vec<fs_tree::TreeEntry>, String> {
    fs_tree::read_dir(&path)
}

#[tauri::command]
fn get_bootstrap(state: tauri::State<AppState>) -> Result<BootstrapPayload, String> {
    let bootstrap = load_bootstrap(&state.bootstrap_path)?;
    Ok(BootstrapPayload {
        recommended_data_root: paths::recommended_data_root()
            .to_string_lossy()
            .to_string(),
        confirmed_data_root: bootstrap.confirmed_data_root,
    })
}

#[tauri::command]
fn confirm_data_root(path: String, state: tauri::State<AppState>) -> Result<BootstrapPayload, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("data root path cannot be empty".to_string());
    }

    let root = PathBuf::from(trimmed);
    fs::create_dir_all(&root).map_err(|e| format!("failed to create data root: {e}"))?;

    let confirmed = root.to_string_lossy().to_string();

    save_bootstrap(
        &state.bootstrap_path,
        &BootstrapState {
            confirmed_data_root: Some(confirmed.clone()),
        },
    )?;

    {
        let mut settings = state
            .settings
            .lock()
            .map_err(|_| "failed to lock settings".to_string())?;
        settings.data_root_path = Some(confirmed.clone());
        persistence::save_app_settings(&settings.clone()).map_err(|e| e.to_string())?;
    }

    Ok(BootstrapPayload {
        recommended_data_root: paths::recommended_data_root()
            .to_string_lossy()
            .to_string(),
        confirmed_data_root: Some(confirmed),
    })
}

#[tauri::command]
fn check_root_path_available(
    path: String,
    exclude_project_id: Option<String>,
    state: tauri::State<AppState>,
) -> Result<RootPathCheckPayload, String> {
    let normalized = paths::normalize_path_for_compare(&path)?;
    let projects = state
        .projects
        .lock()
        .map_err(|_| "failed to lock project state".to_string())?;

    let is_taken = projects
        .iter()
        .filter(|project| {
            if let Some(exclude_id) = &exclude_project_id {
                &project.id != exclude_id
            } else {
                true
            }
        })
        .flat_map(|project| project.root_paths.iter())
        .filter_map(|root| paths::normalize_path_for_compare(root).ok())
        .any(|normalized_root| normalized_root == normalized);

    Ok(RootPathCheckPayload {
        normalized_path: normalized,
        is_available: !is_taken,
    })
}

#[tauri::command]
fn git_remote_origin(repo_path: String, state: tauri::State<AppState>) -> Result<Option<String>, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git::git_remote_origin_url(&repo_path, settings.git_executable_path.as_deref())
}

#[tauri::command]
fn git_remote_summary(
    repo_path: String,
    state: tauri::State<AppState>,
) -> Result<git::RemoteSummaryPayload, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git::git_remote_summary(&repo_path, settings.git_executable_path.as_deref())
}

#[tauri::command]
fn git_remote_list(
    repo_path: String,
    state: tauri::State<AppState>,
) -> Result<Vec<git::GitRemoteItem>, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git::git_remote_list(&repo_path, settings.git_executable_path.as_deref())
}

#[tauri::command]
fn folder_root_row(
    repo_path: String,
    state: tauri::State<AppState>,
) -> Result<git::FolderRootRowPayload, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    Ok(git::folder_root_row(
        &repo_path,
        settings.git_executable_path.as_deref(),
    ))
}

#[tauri::command]
fn git_remote_add(
    repo_path: String,
    name: String,
    url: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git::git_remote_add(&repo_path, &name, &url, settings.git_executable_path.as_deref())
}

#[tauri::command]
fn git_remote_remove(
    repo_path: String,
    name: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git::git_remote_remove(&repo_path, &name, settings.git_executable_path.as_deref())
}

#[tauri::command]
fn git_remote_rename(
    repo_path: String,
    old_name: String,
    new_name: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git::git_remote_rename(
        &repo_path,
        &old_name,
        &new_name,
        settings.git_executable_path.as_deref(),
    )
}

#[tauri::command]
fn git_remote_set_url(
    repo_path: String,
    name: String,
    url: String,
    push_only: bool,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git::git_remote_set_url(
        &repo_path,
        &name,
        &url,
        push_only,
        settings.git_executable_path.as_deref(),
    )
}

#[tauri::command]
fn git_status(repo_path: String, state: tauri::State<AppState>) -> Result<git::GitStatusPayload, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    let status = git::git_status(&repo_path, settings.git_executable_path.as_deref())?;
    let _ = state.git_logs.push(
        "git status --porcelain".to_string(),
        repo_path,
        0,
        format!("branch={}, clean={}", status.branch, status.clean),
        String::new(),
    );
    Ok(status)
}

#[tauri::command]
fn git_status_files(
    repo_path: String,
    state: tauri::State<AppState>,
) -> Result<git::GitStatusFilesPayload, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git::git_status_files(&repo_path, settings.git_executable_path.as_deref())
}

#[tauri::command]
fn logs_list(state: tauri::State<AppState>) -> Result<Vec<git_log::GitLogEntry>, String> {
    state.git_logs.list()
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Result<settings::AppSettings, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
fn update_settings(
    mut next: settings::AppSettings,
    state: tauri::State<AppState>,
) -> Result<settings::AppSettings, String> {
    if next
        .git_executable_path
        .as_deref()
        .is_some_and(|s| s.trim().is_empty())
    {
        next.git_executable_path = None;
    }
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    *settings = next.clone();
    persistence::save_app_settings(&next).map_err(|e| e.to_string())?;
    if let Some(ref root) = next.data_root_path {
        let trimmed = root.trim();
        if !trimmed.is_empty() {
            let pb = PathBuf::from(trimmed);
            fs::create_dir_all(&pb).map_err(|e| format!("failed to create data root: {e}"))?;
            save_bootstrap(
                &state.bootstrap_path,
                &BootstrapState {
                    confirmed_data_root: Some(trimmed.to_string()),
                },
            )?;
        }
    }
    Ok(next)
}

#[tauri::command]
fn git_probe_version(state: tauri::State<AppState>) -> Result<String, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git::git_version(settings.git_executable_path.as_deref())
}

#[tauri::command]
fn reveal_path(path: String) -> Result<(), String> {
    shell::open_path(&path)
}

#[tauri::command]
fn open_remote_in_browser(remote: String) -> Result<(), String> {
    shell::open_remote_in_browser(&remote)
}

#[tauri::command]
fn open_external(
    path: String,
    command: Option<String>,
    args_template: Option<String>,
) -> Result<(), String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("path is empty".to_string());
    }
    let Some(cmd) = command else {
        return shell::open_path(path);
    };
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return Err("external command is empty".to_string());
    }
    let tpl = args_template.as_deref().unwrap_or("").trim();
    let args: Vec<String> = if tpl.is_empty() {
        vec![path.to_string()]
    } else {
        shell::expand_external_args(tpl, path)
    };
    shell::run_external_tool(cmd, &args)
}

#[tauri::command]
fn git_init(repo_path: String, state: tauri::State<AppState>) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git_ops::git_init(&repo_path, settings.git_executable_path.as_deref())
}

#[tauri::command]
fn git_remove_dot_git(repo_path: String) -> Result<(), String> {
    git_ops::git_remove_dot_git(&repo_path)
}

#[tauri::command]
fn path_dot_git_exists(repo_path: String) -> Result<bool, String> {
    git_ops::path_dot_git_exists(&repo_path)
}

#[tauri::command]
fn path_is_git_repo_root(
    repo_path: String,
    state: tauri::State<AppState>,
) -> Result<bool, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    git_ops::path_is_git_repo_root(&repo_path, settings.git_executable_path.as_deref())
}

#[tauri::command]
fn read_gitignore(repo_path: String) -> Result<gitignore::GitignoreReadResult, String> {
    gitignore::read_gitignore(&repo_path)
}

#[tauri::command]
fn write_gitignore(repo_path: String, content: String) -> Result<(), String> {
    gitignore::write_gitignore(&repo_path, &content)
}

#[tauri::command]
fn template_list() -> Vec<gitignore_templates::TemplateItem> {
    gitignore_templates::list_templates()
}

#[tauri::command]
fn template_sync() -> Result<gitignore_templates::TemplateSyncResult, String> {
    gitignore_templates::sync_from_github()
}

#[tauri::command]
fn git_archive_export(
    repo_path: String,
    output_parent_dir: String,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?;
    let git_exe = settings.git_executable_path.clone();
    drop(settings);
    git_ops::git_archive_export_zip(&repo_path, &output_parent_dir, git_exe.as_deref())
}

fn persist_projects(state_path: &PathBuf, projects: &[Project]) -> Result<(), String> {
    let persisted = persistence::PersistedState {
        schema_version: persistence::CURRENT_SCHEMA_VERSION,
        projects: projects
            .iter()
            .map(|p| persistence::PersistedProject {
                id: p.id.clone(),
                name: p.name.clone(),
                root_paths: p.root_paths.clone(),
            })
            .collect(),
    };
    persistence::save_state_atomic(state_path, &persisted)
}

/// 앱 시작 시 state.json의 root_paths 중 실제로 사라진 경로를 정리한다.
/// 존재하지 않거나 디렉터리가 아닌 경로는 제거하고, 변경이 있으면 즉시 저장한다.
fn sanitize_projects_on_boot(
    state_path: &PathBuf,
    mut projects: Vec<Project>,
) -> Result<Vec<Project>, String> {
    let mut changed = false;
    for project in &mut projects {
        let before_len = project.root_paths.len();
        project.root_paths.retain(|p| {
            let path = Path::new(p);
            fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
        });
        if project.root_paths.len() != before_len {
            changed = true;
        }
    }
    if changed {
        persist_projects(state_path, &projects)?;
    }
    Ok(projects)
}

fn default_bootstrap_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local-git-repo-manager")
        .join("bootstrap.json")
}

fn load_bootstrap(path: &PathBuf) -> Result<BootstrapState, String> {
    if !path.exists() {
        return Ok(BootstrapState::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read bootstrap: {e}"))?;
    serde_json::from_str::<BootstrapState>(&raw)
        .map_err(|e| format!("failed to parse bootstrap: {e}"))
}

fn save_bootstrap(path: &PathBuf, bootstrap: &BootstrapState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create bootstrap dir: {e}"))?;
    }
    let raw = serde_json::to_string_pretty(bootstrap)
        .map_err(|e| format!("failed to serialize bootstrap: {e}"))?;
    fs::write(path, raw).map_err(|e| format!("failed to write bootstrap: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state_path = persistence::default_state_path();
    let bootstrap_path = default_bootstrap_path();
    let persisted_state = persistence::load_state(&state_path).unwrap_or_else(|error| {
        eprintln!("failed to load state: {error}");
        persistence::PersistedState::default()
    });
    let projects = persisted_state
        .projects
        .into_iter()
        .map(|p| Project {
            id: p.id,
            name: p.name,
            root_paths: p.root_paths,
        })
        .collect::<Vec<_>>();
    let projects = sanitize_projects_on_boot(&state_path, projects).unwrap_or_else(|e| {
        eprintln!("failed to sanitize projects on boot: {e}");
        Vec::new()
    });

    let bootstrap = load_bootstrap(&bootstrap_path).unwrap_or_else(|e| {
        eprintln!("failed to load bootstrap: {e}");
        BootstrapState::default()
    });

    let mut app_settings = persistence::load_app_settings().unwrap_or_else(|e| {
        eprintln!("failed to load app settings: {e}");
        settings::AppSettings::default()
    });

    let dr = app_settings
        .data_root_path
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    if dr.is_empty() {
        app_settings.data_root_path = bootstrap.confirmed_data_root.clone();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            app.manage(AppState {
                bootstrap_path: bootstrap_path.clone(),
                state_path: state_path.clone(),
                projects: Mutex::new(projects),
                settings: Mutex::new(app_settings),
                git_logs: git_log::GitLogBuffer::default(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            projects_list,
            projects_create,
            projects_update,
            projects_delete,
            projects_reorder,
            projects_add_root,
            projects_import_folder_drop,
            projects_remove_root,
            move_root_to_project,
            tree_readdir,
            get_bootstrap,
            confirm_data_root,
            check_root_path_available,
            git_remote_origin,
            git_remote_summary,
            git_remote_list,
            folder_root_row,
            git_remote_add,
            git_remote_remove,
            git_remote_rename,
            git_remote_set_url,
            git_status,
            git_status_files,
            logs_list,
            get_settings,
            update_settings,
            git_probe_version,
            reveal_path,
            open_remote_in_browser,
            open_external,
            git_init,
            git_remove_dot_git,
            path_dot_git_exists,
            path_is_git_repo_root,
            read_gitignore,
            write_gitignore,
            template_list,
            template_sync,
            git_archive_export
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
