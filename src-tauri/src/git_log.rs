use serde::Serialize;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitLogEntry {
    pub id: String,
    pub command: String,
    pub cwd: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub ts: u128,
}

#[derive(Default)]
pub struct GitLogBuffer {
    inner: Mutex<Vec<GitLogEntry>>,
}

impl GitLogBuffer {
    pub fn push(
        &self,
        command: String,
        cwd: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
    ) -> Result<(), String> {
        let mut logs = self
            .inner
            .lock()
            .map_err(|_| "failed to lock git logs".to_string())?;
        let next_id = logs.len() + 1;
        logs.push(GitLogEntry {
            id: format!("log-{next_id}"),
            command,
            cwd,
            exit_code,
            stdout,
            stderr,
            ts: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "system time error".to_string())?
                .as_millis(),
        });
        if logs.len() > 300 {
            let overflow = logs.len() - 300;
            logs.drain(0..overflow);
        }
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<GitLogEntry>, String> {
        let logs = self
            .inner
            .lock()
            .map_err(|_| "failed to lock git logs".to_string())?;
        Ok(logs.clone())
    }
}
