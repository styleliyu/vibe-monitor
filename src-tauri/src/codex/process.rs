use std::process::Stdio;

use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
};

use crate::error::AppError;

use super::{
    jsonrpc::{encode_request, JsonRpcRequest},
    notification_value_to_attention_item, CodexThreadSummary,
};

pub const THREAD_UPDATED_EVENT: &str = "codex://thread-updated";
pub const ITEM_EVENT: &str = "codex://item";
pub const APPROVAL_REQUESTED_EVENT: &str = "codex://approval-requested";
pub const TURN_FINISHED_EVENT: &str = "codex://turn-finished";

#[derive(Default)]
pub struct CodexProcessManager {
    server: Option<CodexAppServer>,
    next_id: u64,
}

struct CodexAppServer {
    child: Child,
    stdin: ChildStdin,
}

impl Drop for CodexAppServer {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

impl CodexProcessManager {
    pub async fn ensure_started(
        &mut self,
        app: AppHandle,
        app_data_dir: std::path::PathBuf,
    ) -> Result<(), AppError> {
        if self.server.is_some() {
            return Ok(());
        }

        let mut child = Command::new("codex")
            .arg("app-server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                AppError::CommandFailed(format!("failed to start codex app-server: {error}"))
            })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            AppError::CommandFailed("codex app-server stdin unavailable".to_string())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            AppError::CommandFailed("codex app-server stdout unavailable".to_string())
        })?;

        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                emit_stdout_line(&app, &app_data_dir, &line);
            }
        });

        self.server = Some(CodexAppServer { child, stdin });
        Ok(())
    }

    pub async fn thread_list(&mut self, cwd: String) -> Result<Vec<CodexThreadSummary>, AppError> {
        self.send_request("thread/list", json!({ "cwd": cwd }))
            .await?;
        Ok(Vec::new())
    }

    pub async fn thread_start(
        &mut self,
        workspace_id: String,
        cwd: String,
        prompt: String,
    ) -> Result<CodexThreadSummary, AppError> {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return Err(AppError::InvalidInput(
                "codex prompt cannot be empty".to_string(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        self.send_request(
            "thread/start",
            json!({
                "cwd": cwd,
                "prompt": prompt,
                "sandbox": "workspace-write",
                "approvalPolicy": "on-request"
            }),
        )
        .await?;

        Ok(CodexThreadSummary {
            id,
            workspace_id,
            title: first_line_title(prompt),
            status: "running".to_string(),
            preview: Some(prompt.to_string()),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        })
    }

    pub async fn turn_send(&mut self, thread_id: String, prompt: String) -> Result<(), AppError> {
        let prompt = prompt.trim();
        if thread_id.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "codex thread id cannot be empty".to_string(),
            ));
        }
        if prompt.is_empty() {
            return Err(AppError::InvalidInput(
                "codex prompt cannot be empty".to_string(),
            ));
        }

        self.send_request(
            "turn/start",
            json!({ "threadId": thread_id.trim(), "prompt": prompt }),
        )
        .await
    }

    pub async fn turn_interrupt(&mut self, thread_id: String) -> Result<(), AppError> {
        if thread_id.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "codex thread id cannot be empty".to_string(),
            ));
        }

        self.send_request("turn/interrupt", json!({ "threadId": thread_id.trim() }))
            .await
    }

    async fn send_request<T: Serialize>(
        &mut self,
        method: &str,
        params: T,
    ) -> Result<(), AppError> {
        let server = self.server.as_mut().ok_or_else(|| {
            AppError::CommandFailed("codex app-server is not running".to_string())
        })?;
        self.next_id += 1;
        let line = encode_request(&JsonRpcRequest {
            id: self.next_id,
            method: method.to_string(),
            params,
        })?;

        server
            .stdin
            .write_all(format!("{line}\n").as_bytes())
            .await
            .map_err(|error| {
                AppError::CommandFailed(format!("failed to write to codex app-server: {error}"))
            })?;
        server.stdin.flush().await.map_err(|error| {
            AppError::CommandFailed(format!("failed to flush codex app-server stdin: {error}"))
        })
    }
}

fn emit_stdout_line(app: &AppHandle, app_data_dir: &std::path::Path, line: &str) {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return;
    };
    let method = value
        .get("method")
        .and_then(Value::as_str)
        .or_else(|| value.get("type").and_then(Value::as_str))
        .unwrap_or_default();

    let event = match method {
        value if value.contains("approval") => APPROVAL_REQUESTED_EVENT,
        value if value.contains("thread") => THREAD_UPDATED_EVENT,
        value if value.contains("turn") && value.contains("finished") => TURN_FINISHED_EVENT,
        value if value.contains("item") => ITEM_EVENT,
        _ => return,
    };

    if event == APPROVAL_REQUESTED_EVENT {
        let _ = notification_value_to_attention_item(app_data_dir, &value);
    }
    let _ = app.emit(event, &value);
}

fn first_line_title(prompt: &str) -> String {
    let title = prompt.lines().next().unwrap_or("Codex Thread").trim();
    if title.chars().count() > 80 {
        let truncated: String = title.chars().take(77).collect();
        format!("{truncated}...")
    } else if title.is_empty() {
        "Codex Thread".to_string()
    } else {
        title.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::first_line_title;

    #[test]
    fn first_line_title_truncates_utf8_without_panicking() {
        let prompt = "请".repeat(100);

        let title = first_line_title(&prompt);

        assert_eq!(title.chars().count(), 80);
        assert!(title.ends_with("..."));
    }
}
