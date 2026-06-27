use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};

use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
    sync::{oneshot, Mutex},
    time::timeout,
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

pub struct CodexProcessManager {
    server: Option<CodexAppServer>,
    next_id: u64,
    active_turns: Arc<Mutex<HashMap<String, String>>>,
    thread_cwds: HashMap<String, String>,
}

struct CodexAppServer {
    child: Child,
    stdin: ChildStdin,
    pending: PendingRequests,
}

type PendingResult = Result<Value, String>;
type PendingRequests = Arc<Mutex<HashMap<u64, oneshot::Sender<PendingResult>>>>;

impl Default for CodexProcessManager {
    fn default() -> Self {
        Self {
            server: None,
            next_id: 0,
            active_turns: Arc::new(Mutex::new(HashMap::new())),
            thread_cwds: HashMap::new(),
        }
    }
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
        if let Some(server) = self.server.as_mut() {
            match server.child.try_wait() {
                Ok(None) => return Ok(()),
                Ok(Some(_)) => self.server = None,
                Err(error) => {
                    return Err(AppError::CommandFailed(format!(
                        "failed to inspect codex app-server: {error}"
                    )))
                }
            }
        }

        let mut command = Command::new("codex");
        command
            .arg("app-server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        command.creation_flags(0x08000000);

        let mut child = command.spawn().map_err(|error| {
            AppError::CommandFailed(format!("failed to start codex app-server: {error}"))
        })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            AppError::CommandFailed("codex app-server stdin unavailable".to_string())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            AppError::CommandFailed("codex app-server stdout unavailable".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            AppError::CommandFailed("codex app-server stderr unavailable".to_string())
        })?;

        let pending = Arc::new(Mutex::new(HashMap::new()));
        let pending_for_reader = pending.clone();
        let active_turns = self.active_turns.clone();

        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                handle_stdout_line(
                    &app,
                    &app_data_dir,
                    &pending_for_reader,
                    &active_turns,
                    &line,
                )
                .await;
            }

            let mut pending = pending_for_reader.lock().await;
            for (_, sender) in pending.drain() {
                let _ = sender.send(Err("codex app-server exited".to_string()));
            }
        });

        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("codex app-server: {line}");
            }
        });

        self.server = Some(CodexAppServer {
            child,
            stdin,
            pending,
        });

        if let Err(error) = self.send_request("initialize", initialize_params()).await {
            self.server = None;
            return Err(error);
        }
        self.send_notification("initialized", json!({})).await?;

        Ok(())
    }

    pub async fn thread_list(&mut self, cwd: String) -> Result<Vec<CodexThreadSummary>, AppError> {
        self.send_request("thread/list", json!({ "cwd": cwd, "limit": 50 }))
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

        let response = self
            .send_request("thread/start", thread_start_params(&cwd))
            .await?;
        let summary = thread_summary_from_response(&workspace_id, prompt, &response)?;
        self.send_request("turn/start", turn_start_params(&summary.id, prompt, &cwd))
            .await?;
        self.thread_cwds.insert(summary.id.clone(), cwd);

        Ok(summary)
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

        let thread_id = thread_id.trim();
        let cwd = self.thread_cwds.get(thread_id).ok_or_else(|| {
            AppError::CommandFailed("codex thread workspace is unavailable".to_string())
        })?;
        self.send_request("turn/start", turn_start_params(thread_id, prompt, cwd))
            .await
            .map(|_| ())
    }

    pub async fn turn_interrupt(&mut self, thread_id: String) -> Result<(), AppError> {
        if thread_id.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "codex thread id cannot be empty".to_string(),
            ));
        }

        let thread_id = thread_id.trim();
        let turn_id = self
            .active_turns
            .lock()
            .await
            .get(thread_id)
            .cloned()
            .ok_or_else(|| AppError::CommandFailed("codex turn is not running".to_string()))?;

        self.send_request(
            "turn/interrupt",
            json!({ "threadId": thread_id, "turnId": turn_id }),
        )
        .await
        .map(|_| ())
    }

    async fn send_request<T: Serialize>(
        &mut self,
        method: &str,
        params: T,
    ) -> Result<Value, AppError> {
        let server = self.server.as_mut().ok_or_else(|| {
            AppError::CommandFailed("codex app-server is not running".to_string())
        })?;
        self.next_id += 1;
        let request_id = self.next_id;
        let line = encode_request(&JsonRpcRequest {
            id: request_id,
            method: method.to_string(),
            params,
        })?;
        let (sender, receiver) = oneshot::channel();
        server.pending.lock().await.insert(request_id, sender);

        if let Err(error) = server.stdin.write_all(format!("{line}\n").as_bytes()).await {
            server.pending.lock().await.remove(&request_id);
            return Err(AppError::CommandFailed(format!(
                "failed to write to codex app-server: {error}"
            )));
        }
        if let Err(error) = server.stdin.flush().await {
            server.pending.lock().await.remove(&request_id);
            return Err(AppError::CommandFailed(format!(
                "failed to flush codex app-server stdin: {error}"
            )));
        }

        match timeout(Duration::from_secs(20), receiver).await {
            Ok(Ok(Ok(value))) => Ok(value),
            Ok(Ok(Err(error))) => Err(AppError::CommandFailed(error)),
            Ok(Err(_)) => Err(AppError::CommandFailed(
                "codex app-server response channel closed".to_string(),
            )),
            Err(_) => {
                server.pending.lock().await.remove(&request_id);
                Err(AppError::CommandFailed(format!(
                    "codex app-server request timed out: {method}"
                )))
            }
        }
    }

    async fn send_notification<T: Serialize>(
        &mut self,
        method: &str,
        params: T,
    ) -> Result<(), AppError> {
        let server = self.server.as_mut().ok_or_else(|| {
            AppError::CommandFailed("codex app-server is not running".to_string())
        })?;
        let line = serde_json::to_string(&json!({ "method": method, "params": params }))
            .map_err(|error| AppError::CommandFailed(error.to_string()))?;
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

fn initialize_params() -> Value {
    json!({
        "clientInfo": {
            "name": "vibe_monitor",
            "title": "Vibe Monitor",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn thread_start_params(cwd: &str) -> Value {
    json!({
        "cwd": cwd,
        "approvalPolicy": "on-request",
        "sandbox": "workspace-write"
    })
}

fn turn_start_params(thread_id: &str, prompt: &str, cwd: &str) -> Value {
    json!({
        "threadId": thread_id,
        "input": [{ "type": "text", "text": prompt }],
        "cwd": cwd,
        "approvalPolicy": "on-request",
        "sandboxPolicy": {
            "type": "workspaceWrite",
            "writableRoots": [cwd],
            "networkAccess": true
        }
    })
}

fn thread_summary_from_response(
    workspace_id: &str,
    prompt: &str,
    response: &Value,
) -> Result<CodexThreadSummary, AppError> {
    let id = response
        .pointer("/thread/id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AppError::CommandFailed("codex app-server returned no thread id".to_string())
        })?;
    let created_at = response
        .pointer("/thread/createdAt")
        .and_then(Value::as_i64)
        .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))
        .map(|timestamp| timestamp.to_rfc3339())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    Ok(CodexThreadSummary {
        id: id.to_string(),
        workspace_id: workspace_id.to_string(),
        title: first_line_title(prompt),
        status: "running".to_string(),
        preview: Some(prompt.to_string()),
        created_at: Some(created_at.clone()),
        updated_at: Some(created_at),
    })
}

#[derive(Debug, PartialEq, Eq)]
enum TurnLifecycle {
    Started { thread_id: String, turn_id: String },
    Completed { thread_id: String },
}

fn turn_lifecycle(value: &Value) -> Option<TurnLifecycle> {
    let method = value.get("method").and_then(Value::as_str)?;
    let thread_id = value
        .pointer("/params/threadId")
        .and_then(Value::as_str)?
        .to_string();

    match method {
        "turn/started" => Some(TurnLifecycle::Started {
            thread_id,
            turn_id: value
                .pointer("/params/turn/id")
                .and_then(Value::as_str)?
                .to_string(),
        }),
        "turn/completed" => Some(TurnLifecycle::Completed { thread_id }),
        _ => None,
    }
}

async fn handle_stdout_line(
    app: &AppHandle,
    app_data_dir: &std::path::Path,
    pending: &PendingRequests,
    active_turns: &Arc<Mutex<HashMap<String, String>>>,
    line: &str,
) {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return;
    };

    if value.get("method").is_none() {
        let Some(id) = value.get("id").and_then(Value::as_u64) else {
            return;
        };
        let Some(sender) = pending.lock().await.remove(&id) else {
            return;
        };
        let result = if let Some(error) = value.get("error") {
            let code = error
                .get("code")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown app-server error");
            Err(format!("codex app-server error {code}: {message}"))
        } else {
            Ok(value.get("result").cloned().unwrap_or(Value::Null))
        };
        let _ = sender.send(result);
        return;
    }

    match turn_lifecycle(&value) {
        Some(TurnLifecycle::Started { thread_id, turn_id }) => {
            active_turns.lock().await.insert(thread_id, turn_id);
        }
        Some(TurnLifecycle::Completed { thread_id }) => {
            active_turns.lock().await.remove(&thread_id);
        }
        None => {}
    }

    emit_stdout_value(app, app_data_dir, &value);
}

fn emit_stdout_value(app: &AppHandle, app_data_dir: &std::path::Path, value: &Value) {
    let method = value
        .get("method")
        .and_then(Value::as_str)
        .or_else(|| value.get("type").and_then(Value::as_str))
        .unwrap_or_default();
    let normalized_method = method.to_ascii_lowercase();

    let event = if normalized_method.contains("approval") {
        APPROVAL_REQUESTED_EVENT
    } else if normalized_method == "turn/completed" {
        TURN_FINISHED_EVENT
    } else if normalized_method.contains("thread") {
        THREAD_UPDATED_EVENT
    } else if normalized_method.contains("item") || normalized_method.contains("turn") {
        ITEM_EVENT
    } else {
        return;
    };

    if event == APPROVAL_REQUESTED_EVENT {
        let _ = notification_value_to_attention_item(app_data_dir, value);
    }
    let _ = app.emit(event, value);
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
    use serde_json::json;

    use super::{
        first_line_title, initialize_params, thread_start_params, thread_summary_from_response,
        turn_lifecycle, turn_start_params, TurnLifecycle,
    };

    #[test]
    fn first_line_title_truncates_utf8_without_panicking() {
        let prompt = "请".repeat(100);

        let title = first_line_title(&prompt);

        assert_eq!(title.chars().count(), 80);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn protocol_messages_match_current_app_server_contract() {
        assert_eq!(
            initialize_params(),
            json!({
                "clientInfo": {
                    "name": "vibe_monitor",
                    "title": "Vibe Monitor",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })
        );

        let thread = thread_start_params("D:\\workspace");
        assert_eq!(thread["cwd"], "D:\\workspace");
        assert_eq!(thread["approvalPolicy"], "on-request");
        assert_eq!(thread["sandbox"], "workspace-write");
        assert!(thread.get("prompt").is_none());

        assert_eq!(
            turn_start_params("thr_123", "Run tests", "D:\\workspace"),
            json!({
                "threadId": "thr_123",
                "input": [{ "type": "text", "text": "Run tests" }],
                "cwd": "D:\\workspace",
                "approvalPolicy": "on-request",
                "sandboxPolicy": {
                    "type": "workspaceWrite",
                    "writableRoots": ["D:\\workspace"],
                    "networkAccess": true
                }
            })
        );
    }

    #[test]
    fn thread_summary_uses_the_server_thread_id() {
        let summary = thread_summary_from_response(
            "workspace-1",
            "Run tests",
            &json!({
                "thread": {
                    "id": "thr_server_123",
                    "preview": "",
                    "createdAt": 1_750_000_000
                }
            }),
        )
        .expect("valid thread response");

        assert_eq!(summary.id, "thr_server_123");
        assert_eq!(summary.workspace_id, "workspace-1");
        assert_eq!(summary.title, "Run tests");
        assert_eq!(summary.status, "running");
    }

    #[test]
    fn turn_lifecycle_is_driven_by_ordered_server_notifications() {
        assert_eq!(
            turn_lifecycle(&json!({
                "method": "turn/started",
                "params": {
                    "threadId": "thr_123",
                    "turn": { "id": "turn_456" }
                }
            })),
            Some(TurnLifecycle::Started {
                thread_id: "thr_123".to_string(),
                turn_id: "turn_456".to_string()
            })
        );
        assert_eq!(
            turn_lifecycle(&json!({
                "method": "turn/completed",
                "params": { "threadId": "thr_123" }
            })),
            Some(TurnLifecycle::Completed {
                thread_id: "thr_123".to_string()
            })
        );
    }
}
