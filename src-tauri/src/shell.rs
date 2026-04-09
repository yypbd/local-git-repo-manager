use std::path::Path;
use std::process::Command;

/// `args_template`를 공백으로 나눈 뒤 `$PATH`·`{path}` 토큰을 폴더 경로 한 인자로 치환합니다.
pub fn expand_external_args(template: &str, path: &str) -> Vec<String> {
    template
        .split_whitespace()
        .map(|token| {
            if token == "$PATH" || token == "{path}" {
                path.to_string()
            } else {
                token.replace("$PATH", path).replace("{path}", path)
            }
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// 외부 도구 실행. macOS에서 `.app` 번들 경로는 `exec`할 수 없으므로 `open -a`로 실행합니다.
pub fn run_external_tool(command: &str, args: &[String]) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let p = Path::new(command);
        let looks_like_app_bundle =
            command.to_lowercase().ends_with(".app") && p.is_dir();
        if looks_like_app_bundle {
            let mut c = Command::new("open");
            c.arg("-a").arg(command);
            for a in args {
                c.arg(a);
            }
            let status = c
                .status()
                .map_err(|e| format!("failed to run open -a: {e}"))?;
            if !status.success() {
                return Err(format!(
                    "open -a exited with status {:?}",
                    status.code()
                ));
            }
            return Ok(());
        }
    }

    let mut c = Command::new(command);
    for a in args {
        c.arg(a);
    }
    let status = c
        .status()
        .map_err(|e| format!("failed to run external command: {e}"))?;
    if !status.success() {
        return Err(format!(
            "external command exited with status {:?}",
            status.code()
        ));
    }
    Ok(())
}

pub fn open_path(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(path);
        c
    };
    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(path);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", path]);
        c
    };
    cmd.status()
        .map_err(|e| format!("failed to open path in shell: {e}"))?;
    Ok(())
}

/// Git 원격 URL을 시스템 기본 브라우저로 열 수 있는 HTTP(S) 주소로 바꿉니다.
/// — `https://` / `http://` 그대로  
/// — `git@github.com:org/repo.git` 등 SSH 형식은 호스트별로 웹 URL로 변환
pub fn remote_to_browser_url(remote: &str) -> Option<String> {
    let u = remote.trim();
    if u.is_empty() {
        return None;
    }
    if u.starts_with("https://") || u.starts_with("http://") {
        return Some(u.to_string());
    }
    if let Some(rest) = u.strip_prefix("git@") {
        if let Some((host, path)) = rest.split_once(':') {
            let path = path.trim_end_matches(".git");
            return match host {
                "github.com" => Some(format!("https://github.com/{path}")),
                "gitlab.com" => Some(format!("https://gitlab.com/{path}")),
                "bitbucket.org" => Some(format!("https://bitbucket.org/{path}")),
                _ => None,
            };
        }
    }
    if let Some(rest) = u.strip_prefix("ssh://git@github.com/") {
        let path = rest.trim_end_matches(".git");
        return Some(format!("https://github.com/{path}"));
    }
    if let Some(rest) = u.strip_prefix("ssh://git@gitlab.com/") {
        let path = rest.trim_end_matches(".git");
        return Some(format!("https://gitlab.com/{path}"));
    }
    if let Some(rest) = u.strip_prefix("ssh://git@bitbucket.org/") {
        let path = rest.trim_end_matches(".git");
        return Some(format!("https://bitbucket.org/{path}"));
    }
    None
}

pub fn open_remote_in_browser(remote: &str) -> Result<(), String> {
    let url = remote_to_browser_url(remote).ok_or_else(|| {
        "cannot open this remote in a browser (use https:// or git@github.com / git@gitlab.com / git@bitbucket.org)"
            .to_string()
    })?;
    open_path(&url)
}
