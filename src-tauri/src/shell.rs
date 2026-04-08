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
