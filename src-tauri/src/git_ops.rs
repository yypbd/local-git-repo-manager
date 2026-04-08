use std::fs;
use std::path::Path;
use std::process::Command;

use crate::paths;

/// 비어 있거나 공백만 있으면 `git`(PATH)을 사용합니다.
pub fn git_executable_or_default<'a>(git_executable: Option<&'a str>) -> &'a str {
    match git_executable {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => "git",
    }
}

pub fn git_init(path: &str, git_executable: Option<&str>) -> Result<(), String> {
    let cwd = paths::resolve_repo_workdir(path)?;
    let git = git_executable_or_default(git_executable);
    let out = Command::new(git)
        .current_dir(&cwd)
        .arg("init")
        .output()
        .map_err(|e| format!("failed to run git init: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(())
}

/// 루트 디렉터리 아래 `.git` 디렉터리만 제거합니다(작업 트리 파일은 유지).
pub fn git_remove_dot_git(repo_path: &str) -> Result<(), String> {
    let trimmed = repo_path.trim();
    if paths::is_network_path(trimmed) {
        return Err("network path is not allowed".to_string());
    }
    let normalized = paths::normalize_path_for_compare(repo_path)?;
    let root = Path::new(&normalized);
    if !root.is_dir() {
        return Err("path is not a directory".to_string());
    }
    let dot_git = root.join(".git");
    if !dot_git.exists() {
        return Err(".git not found".to_string());
    }
    if dot_git.is_file() {
        return fs::remove_file(&dot_git).map_err(|e| format!("failed to remove .git file: {e}"));
    }
    if !dot_git.is_dir() {
        return Err(".git is not a supported type".to_string());
    }
    fs::remove_dir_all(&dot_git).map_err(|e| format!("failed to remove .git: {e}"))
}

/// 해당 경로에 Git 메타데이터(`.git` 파일 또는 디렉터리)가 있는지. nested worktree의 `.git` 파일도 true.
pub fn path_dot_git_exists(repo_path: &str) -> Result<bool, String> {
    let trimmed = repo_path.trim();
    if paths::is_network_path(trimmed) {
        return Err("network path is not allowed".to_string());
    }
    let normalized = paths::normalize_path_for_compare(repo_path)?;
    Ok(Path::new(&normalized).join(".git").exists())
}

/// 해당 경로에 `.git`이 있고, 해당 경로를 기준으로 `git`이 실제 repo로 인식하는지 확인합니다.
///
/// - `.git`만 존재하면 true로 처리하기엔 부족한 케이스가 있어(깨진 메타 등),
///   `.git` 존재 + `git rev-parse --git-dir` 성공을 함께 체크합니다.
pub fn path_is_git_repo_root(
    repo_path: &str,
    git_executable: Option<&str>,
) -> Result<bool, String> {
    let trimmed = repo_path.trim();
    if paths::is_network_path(trimmed) {
        return Err("network path is not allowed".to_string());
    }

    let normalized = paths::normalize_path_for_compare(repo_path)?;
    let root = Path::new(&normalized);
    if !root.is_dir() {
        return Err("path is not a directory".to_string());
    }

    let dot_git = root.join(".git");
    if !dot_git.exists() {
        return Ok(false);
    }

    let git = git_executable_or_default(git_executable);
    let out = Command::new(git)
        .current_dir(root)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| format!("failed to run git rev-parse: {e}"))?;

    Ok(out.status.success())
}

fn git_stdout_trimmed(repo_path: &str, git: &str, args: &[&str]) -> Result<String, String> {
    let cwd = paths::resolve_repo_workdir(repo_path)?;
    let out = Command::new(git)
        .current_dir(&cwd)
        .args(args)
        .output()
        .map_err(|e| format!("git 실행 실패: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(err.trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn sanitize_filename_component(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => out.push('_'),
            c if c.is_control() => out.push('_'),
            _ => out.push(c),
        }
    }
    let t = out.trim().trim_end_matches('.');
    if t.is_empty() {
        "repo".to_string()
    } else {
        t.chars().take(80).collect()
    }
}

fn unique_zip_path(initial: std::path::PathBuf) -> std::path::PathBuf {
    if !initial.exists() {
        return initial;
    }
    let parent = initial
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let stem = initial
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("archive");
    for i in 1u32..1000 {
        let p = parent.join(format!("{stem}_{i}.zip"));
        if !p.exists() {
            return p;
        }
    }
    initial
}

/// `output_parent_dir` 아래에 `저장소명_브랜치_날짜_짧은커밋.zip` 형태로 `git archive` (HEAD) 저장.
pub fn git_archive_export_zip(
    repo_path: &str,
    output_parent_dir: &str,
    git_executable: Option<&str>,
) -> Result<String, String> {
    let trimmed_repo = repo_path.trim();
    let trimmed_parent = output_parent_dir.trim();
    if trimmed_repo.is_empty() || trimmed_parent.is_empty() {
        return Err("경로가 비어 있습니다.".to_string());
    }
    if paths::is_network_path(trimmed_repo) || paths::is_network_path(trimmed_parent) {
        return Err("네트워크 경로는 지원하지 않습니다.".to_string());
    }

    let git = git_executable_or_default(git_executable);
    if !path_is_git_repo_root(trimmed_repo, git_executable)? {
        return Err("Git 저장소 루트가 아닙니다.".to_string());
    }

    let cwd = paths::resolve_repo_workdir(trimmed_repo)?;
    let root = Path::new(&cwd);
    let parent_out = Path::new(trimmed_parent);
    if !parent_out.is_dir() {
        return Err("저장할 폴더가 없거나 디렉터리가 아닙니다.".to_string());
    }

    let branch_raw = git_stdout_trimmed(trimmed_repo, git, &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|_| "unknown".to_string());
    let short_sha = git_stdout_trimmed(trimmed_repo, git, &["rev-parse", "--short=7", "HEAD"])
        .unwrap_or_else(|_| "unknown".to_string());

    let repo_base = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "repo".to_string());

    let repo_safe = sanitize_filename_component(&repo_base);
    let branch_safe = sanitize_filename_component(&branch_raw);
    let sha_safe = sanitize_filename_component(&short_sha);
    let date_part = chrono::Local::now().format("%Y-%m-%d_%H%M%S");

    let base_name = format!("{repo_safe}_{branch_safe}_{date_part}_{sha_safe}");
    let zip_name = format!("{}.zip", sanitize_filename_component(&base_name));
    let initial_out = parent_out.join(&zip_name);
    let out_path = unique_zip_path(initial_out);

    let out_display = out_path.to_string_lossy().to_string();
    let out = Command::new(git)
        .current_dir(root)
        .arg("archive")
        .arg("--format=zip")
        .arg("-o")
        .arg(&out_path)
        .arg("HEAD")
        .output()
        .map_err(|e| format!("git archive 실행 실패: {e}"))?;

    if !out.status.success() {
        let _ = fs::remove_file(&out_path);
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(err.trim().to_string());
    }

    Ok(out_display)
}
