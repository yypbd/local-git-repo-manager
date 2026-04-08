use crate::git_ops;
use crate::paths;
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::process::Command;

fn first_line_stdout_ok(out: std::process::Output) -> Option<String> {
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let line = s.lines().next()?.trim();
    if line.is_empty() {
        None
    } else {
        Some(line.to_string())
    }
}

/// `git remote get-url` / `--push` — `remote -v` 파싱 없이 URL만 가져옵니다.
fn remote_get_url_explicit(git: &str, cwd: &str, name: &str, push: bool) -> Option<String> {
    let args: &[&str] = if push {
        &["remote", "get-url", "--push", name]
    } else {
        &["remote", "get-url", name]
    };
    Command::new(git)
        .current_dir(cwd)
        .args(args)
        .output()
        .ok()
        .and_then(first_line_stdout_ok)
}

/// `git remote` 출력과 config 키를 합친 원격 이름 집합.
fn collect_remote_names(
    git: &str,
    cwd: &str,
    config_urls: &HashMap<String, (Option<String>, Option<String>)>,
) -> Result<BTreeSet<String>, String> {
    let names_out = Command::new(git)
        .current_dir(cwd)
        .arg("remote")
        .output()
        .map_err(|e| format!("failed to run git remote: {e}"))?;
    let mut names = BTreeSet::new();
    if names_out.status.success() {
        for line in String::from_utf8_lossy(&names_out.stdout).lines() {
            let n = line.trim();
            if !n.is_empty() {
                names.insert(n.to_string());
            }
        }
    }
    for n in config_urls.keys() {
        names.insert(n.clone());
    }
    Ok(names)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatusPayload {
    pub branch: String,
    pub clean: bool,
    /// `git status --porcelain`에서 `??`가 아닌 줄(추적 파일의 변경·스테이징 등).
    pub tracked_changes: usize,
    /// 미추적(`??`) 항목 수 (`-uall` 기준, 목록 API와 동일하게 집계).
    pub untracked_files: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatusFilesPayload {
    pub branch: String,
    /// `git rev-parse --abbrev-ref @{u}` — 예: `origin/main`. 추적 브랜치 없으면 `None`.
    pub upstream: Option<String>,
    pub clean: bool,
    pub changed_or_added: Vec<String>,
    pub untracked: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSummaryPayload {
    pub origin: Option<String>,
    pub remote_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitRemoteItem {
    pub name: String,
    pub fetch_url: Option<String>,
    pub push_url: Option<String>,
}

/// 폴더 목록 한 줄용 페이로드 (`folder_root_row` 커맨드).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderRootRowPayload {
    pub has_dot_git: bool,
    pub remote: Option<String>,
    pub remote_count: usize,
    pub branch: String,
    pub clean: bool,
    pub tracked_changes: usize,
    pub untracked_files: usize,
    pub git_error: bool,
}

fn pick_display_remote_from_items(items: &[GitRemoteItem]) -> Option<String> {
    fn url_of(r: &GitRemoteItem) -> Option<String> {
        r.fetch_url
            .as_ref()
            .or(r.push_url.as_ref())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
    for name in ["origin", "upstream"] {
        if let Some(r) = items.iter().find(|r| r.name == *name) {
            if let Some(u) = url_of(r) {
                return Some(u);
            }
        }
    }
    let mut sorted: Vec<&GitRemoteItem> = items.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    sorted.iter().find_map(|r| url_of(r))
}

pub fn folder_root_row(repo_path: &str, git_executable: Option<&str>) -> FolderRootRowPayload {
    let has_dot_git = git_ops::path_dot_git_exists(repo_path).unwrap_or(false);

    let (remote, remote_count) = match git_remote_list(repo_path, git_executable) {
        Ok(items) => {
            let n = items.len();
            (pick_display_remote_from_items(&items), n)
        }
        Err(_) => (None, 0),
    };

    match git_status(repo_path, git_executable) {
        Ok(s) => FolderRootRowPayload {
            has_dot_git,
            remote,
            remote_count,
            branch: s.branch,
            clean: s.clean,
            tracked_changes: s.tracked_changes,
            untracked_files: s.untracked_files,
            git_error: false,
        },
        Err(_) => FolderRootRowPayload {
            has_dot_git,
            remote,
            remote_count,
            branch: "—".to_string(),
            clean: true,
            tracked_changes: 0,
            untracked_files: 0,
            git_error: !has_dot_git,
        },
    }
}

/// `git remote get-url`이 실패해도 `.git/config`에 URL이 있으면 읽습니다.
fn remote_fetch_url(git: &str, cwd: &str, name: &str) -> Option<String> {
    let run = |args: &[&str]| -> Option<String> {
        Command::new(git)
            .current_dir(cwd)
            .args(args)
            .output()
            .ok()
            .and_then(first_line_stdout_ok)
    };
    run(&["remote", "get-url", name])
        .or_else(|| run(&["remote", "get-url", "--push", name]))
        .or_else(|| {
            let k = format!("remote.{name}.url");
            run(&["config", "--get", &k])
        })
        .or_else(|| {
            let k = format!("remote.{name}.pushurl");
            run(&["config", "--get", &k])
        })
}

/// `git config --local -l`에서 `remote.<이름>.url` / `pushurl`을 읽습니다. `get-url`보다 먼저 씁니다.
fn remote_urls_from_local_config(git: &str, cwd: &str) -> HashMap<String, (Option<String>, Option<String>)> {
    let Ok(out) = Command::new(git)
        .current_dir(cwd)
        .args(["config", "--local", "-l"])
        .output()
    else {
        return HashMap::new();
    };
    if !out.status.success() {
        return HashMap::new();
    }
    let mut map: HashMap<String, (Option<String>, Option<String>)> = HashMap::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(eq) = line.find('=') else {
            continue;
        };
        let key = line[..eq].trim();
        let value = line[eq + 1..].trim();
        if value.is_empty() {
            continue;
        }
        let Some(rest) = key.strip_prefix("remote.") else {
            continue;
        };
        if let Some(name) = rest.strip_suffix(".url") {
            if !name.is_empty() {
                map.entry(name.to_string()).or_insert((None, None)).0 = Some(value.to_string());
            }
        } else if let Some(name) = rest.strip_suffix(".pushurl") {
            if !name.is_empty() {
                map.entry(name.to_string()).or_insert((None, None)).1 = Some(value.to_string());
            }
        }
    }
    map
}

pub fn git_version(git_executable: Option<&str>) -> Result<String, String> {
    let git = git_ops::git_executable_or_default(git_executable);
    let output = Command::new(git)
        .arg("--version")
        .output()
        .map_err(|e| format!("failed to run git --version: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// `git status --porcelain -uall` 본문을 파싱해 추적 변경 경로와 미추적 경로를 나눕니다 (정렬·중복 제거 후).
fn parse_porcelain_status_body(body: &str) -> (Vec<String>, Vec<String>) {
    let mut changed_or_added: Vec<String> = vec![];
    let mut untracked: Vec<String> = vec![];

    for line in body.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.len() < 3 {
            continue;
        }

        let status = &trimmed[0..2];
        let path_part = trimmed.get(3..).unwrap_or("").trim();

        if status == "??" {
            if !path_part.is_empty() {
                untracked.push(path_part.to_string());
            }
        } else if !path_part.is_empty() {
            changed_or_added.push(path_part.to_string());
        }
    }

    changed_or_added.sort();
    changed_or_added.dedup();
    untracked.sort();
    untracked.dedup();
    (changed_or_added, untracked)
}

pub fn git_status(repo_path: &str, git_executable: Option<&str>) -> Result<GitStatusPayload, String> {
    let cwd = paths::resolve_repo_workdir(repo_path)?;
    let git = git_ops::git_executable_or_default(git_executable);
    // 먼저 git repo 여부를 확인합니다.
    // - `git init` 직후처럼 HEAD/브랜치가 없을 수 있으므로,
    //   브랜치 조회 실패를 "repo 아님"으로 취급하지 않습니다.
    let git_dir_out = Command::new(git)
        .current_dir(&cwd)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| format!("failed to run git rev-parse --git-dir: {e}"))?;
    if !git_dir_out.status.success() {
        return Err(String::from_utf8_lossy(&git_dir_out.stderr).to_string());
    }

    // HEAD/브랜치가 없을 수 있어 실패해도 기본값을 사용합니다.
    let branch_out = Command::new(git)
        .current_dir(&cwd)
        .args(["symbolic-ref", "--short", "-q", "HEAD"])
        .output();
    let branch = match branch_out {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "—".to_string(),
    };

    let status_out = Command::new(git)
        .current_dir(&cwd)
        .args(["status", "--porcelain", "-uall"])
        .output()
        .map_err(|e| format!("failed to run git status: {e}"))?;
    if !status_out.status.success() {
        return Err(String::from_utf8_lossy(&status_out.stderr).to_string());
    }
    let body = String::from_utf8_lossy(&status_out.stdout).to_string();
    let (tracked_paths, untracked_paths) = parse_porcelain_status_body(&body);
    let tracked_changes = tracked_paths.len();
    let untracked_files = untracked_paths.len();
    Ok(GitStatusPayload {
        branch,
        clean: tracked_changes == 0 && untracked_files == 0,
        tracked_changes,
        untracked_files,
    })
}

/// `git status --porcelain`을 파싱해 "변경/추가"와 "untracked" 파일 목록을 제공합니다.
pub fn git_status_files(
    repo_path: &str,
    git_executable: Option<&str>,
) -> Result<GitStatusFilesPayload, String> {
    let cwd = paths::resolve_repo_workdir(repo_path)?;
    let git = git_ops::git_executable_or_default(git_executable);

    // repo 여부 확인(HEAD/브랜치가 없어도 init 직후인 경우가 있으니 실패 시 에러로 처리)
    let git_dir_out = Command::new(git)
        .current_dir(&cwd)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| format!("failed to run git rev-parse --git-dir: {e}"))?;
    if !git_dir_out.status.success() {
        return Err(String::from_utf8_lossy(&git_dir_out.stderr).to_string());
    }

    let branch_out = Command::new(git)
        .current_dir(&cwd)
        .args(["symbolic-ref", "--short", "-q", "HEAD"])
        .output();

    let branch = match branch_out {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "—".to_string(),
    };

    let upstream_out = Command::new(git)
        .current_dir(&cwd)
        .args(["rev-parse", "--abbrev-ref", "@{u}"])
        .output();
    let upstream = match upstream_out {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
        _ => None,
    };

    let status_out = Command::new(git)
        .current_dir(&cwd)
        .args(["status", "--porcelain", "-uall"])
        .output()
        .map_err(|e| format!("failed to run git status: {e}"))?;
    if !status_out.status.success() {
        return Err(String::from_utf8_lossy(&status_out.stderr).to_string());
    }

    let body = String::from_utf8_lossy(&status_out.stdout).to_string();
    let (changed_or_added, untracked) = parse_porcelain_status_body(&body);

    let clean = changed_or_added.is_empty() && untracked.is_empty();

    Ok(GitStatusFilesPayload {
        branch,
        upstream,
        clean,
        changed_or_added,
        untracked,
    })
}

/// `origin` 원격 URL. Git 저장소가 아니거나 origin이 없으면 `None`.
pub fn git_remote_origin_url(repo_path: &str, git_executable: Option<&str>) -> Result<Option<String>, String> {
    let cwd = paths::resolve_repo_workdir(repo_path)?;
    let git = git_ops::git_executable_or_default(git_executable);
    Ok(remote_fetch_url(git, &cwd, "origin"))
}

/// 원격 요약: 로컬 config의 `remote.*.url` 우선, 없으면 `get-url`. 이름 집합은 `git remote` ∪ config 키.
pub fn git_remote_summary(
    repo_path: &str,
    git_executable: Option<&str>,
) -> Result<RemoteSummaryPayload, String> {
    let cwd = paths::resolve_repo_workdir(repo_path)?;
    let git = git_ops::git_executable_or_default(git_executable);
    let config_urls = remote_urls_from_local_config(git, &cwd);
    let names = collect_remote_names(git, &cwd, &config_urls)?;
    let remote_count = names.len();

    let pick_url = |remote_name: &str| -> Option<String> {
        if let Some((fetch, push)) = config_urls.get(remote_name) {
            let u = fetch
                .as_ref()
                .or(push.as_ref())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            if u.is_some() {
                return u;
            }
        }
        remote_fetch_url(git, &cwd, remote_name)
    };

    let mut origin = pick_url("origin");
    if origin.is_none() {
        let mut rest: Vec<&String> = names.iter().filter(|n| n.as_str() != "origin").collect();
        rest.sort_by(|a, b| {
            let rank = |s: &str| match s {
                "upstream" => 0,
                _ => 1,
            };
            rank(a.as_str())
                .cmp(&rank(b.as_str()))
                .then_with(|| a.cmp(b))
        });
        for name in rest {
            if let Some(u) = pick_url(name.as_str()) {
                origin = Some(u);
                break;
            }
        }
    }

    Ok(RemoteSummaryPayload {
        origin,
        remote_count,
    })
}

/// 정규화된 작업 디렉터리를 반환합니다. `git_remote_list` 등과 `git_status`가 같은 cwd를 씁니다.
fn ensure_repo(git: &str, repo_path: &str) -> Result<String, String> {
    let cwd = paths::resolve_repo_workdir(repo_path)?;
    let out = Command::new(git)
        .current_dir(&cwd)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| format!("failed to run git rev-parse --git-dir: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(cwd)
}

/// 원격 목록 — 로컬 config와 `git remote get-url`만 사용 (`remote -v` 파싱 없음).
pub fn git_remote_list(
    repo_path: &str,
    git_executable: Option<&str>,
) -> Result<Vec<GitRemoteItem>, String> {
    let git = git_ops::git_executable_or_default(git_executable);
    let cwd = ensure_repo(git, repo_path)?;
    let config_urls = remote_urls_from_local_config(git, &cwd);
    let names = collect_remote_names(git, &cwd, &config_urls)?;

    let mut items: Vec<GitRemoteItem> = Vec::new();
    for name in names {
        let (cf, cp) = config_urls
            .get(&name)
            .map(|(a, b)| (a.clone(), b.clone()))
            .unwrap_or((None, None));
        let fetch_url = cf
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| remote_get_url_explicit(git, &cwd, &name, false));
        let push_url = cp
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| remote_get_url_explicit(git, &cwd, &name, true));
        items.push(GitRemoteItem {
            name,
            fetch_url,
            push_url,
        });
    }
    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

pub fn git_remote_add(
    repo_path: &str,
    name: &str,
    url: &str,
    git_executable: Option<&str>,
) -> Result<(), String> {
    let git = git_ops::git_executable_or_default(git_executable);
    let cwd = ensure_repo(git, repo_path)?;
    let n = name.trim();
    let u = url.trim();
    if n.is_empty() || u.is_empty() {
        return Err("remote name/url cannot be empty".to_string());
    }
    let out = Command::new(git)
        .current_dir(&cwd)
        .args(["remote", "add", n, u])
        .output()
        .map_err(|e| format!("failed to run git remote add: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}

pub fn git_remote_remove(
    repo_path: &str,
    name: &str,
    git_executable: Option<&str>,
) -> Result<(), String> {
    let git = git_ops::git_executable_or_default(git_executable);
    let cwd = ensure_repo(git, repo_path)?;
    let n = name.trim();
    if n.is_empty() {
        return Err("remote name cannot be empty".to_string());
    }
    let out = Command::new(git)
        .current_dir(&cwd)
        .args(["remote", "remove", n])
        .output()
        .map_err(|e| format!("failed to run git remote remove: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}

pub fn git_remote_rename(
    repo_path: &str,
    old_name: &str,
    new_name: &str,
    git_executable: Option<&str>,
) -> Result<(), String> {
    let git = git_ops::git_executable_or_default(git_executable);
    let cwd = ensure_repo(git, repo_path)?;
    let old = old_name.trim();
    let newv = new_name.trim();
    if old.is_empty() || newv.is_empty() {
        return Err("remote name cannot be empty".to_string());
    }
    let out = Command::new(git)
        .current_dir(&cwd)
        .args(["remote", "rename", old, newv])
        .output()
        .map_err(|e| format!("failed to run git remote rename: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}

pub fn git_remote_set_url(
    repo_path: &str,
    name: &str,
    url: &str,
    push_only: bool,
    git_executable: Option<&str>,
) -> Result<(), String> {
    let git = git_ops::git_executable_or_default(git_executable);
    let cwd = ensure_repo(git, repo_path)?;
    let n = name.trim();
    let u = url.trim();
    if n.is_empty() || u.is_empty() {
        return Err("remote name/url cannot be empty".to_string());
    }
    let mut cmd = Command::new(git);
    cmd.current_dir(&cwd).arg("remote").arg("set-url");
    if push_only {
        cmd.arg("--push");
    }
    let out = cmd
        .arg(n)
        .arg(u)
        .output()
        .map_err(|e| format!("failed to run git remote set-url: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}
