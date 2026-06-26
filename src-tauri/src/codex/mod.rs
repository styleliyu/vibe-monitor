pub mod jsonrpc;
pub mod process;

use std::{path::Path, process::Command};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    attention::{create_attention_item, AttentionItem, CreateAttentionItem},
    error::AppError,
    state::AppState,
    workspace,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAvailability {
    pub available: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexThreadSummary {
    pub id: String,
    pub workspace_id: String,
    pub title: String,
    pub status: String,
    pub preview: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexApprovalRequest {
    pub thread_id: String,
    pub approval_id: String,
    pub title: String,
    pub summary: String,
}

pub fn detect_with_runner<F>(runner: F) -> CodexAvailability
where
    F: FnOnce() -> Result<String, String>,
{
    match runner() {
        Ok(version) => CodexAvailability {
            available: true,
            version: Some(version.trim().to_string()),
            error: None,
        },
        Err(error) => CodexAvailability {
            available: false,
            version: None,
            error: Some(error),
        },
    }
}

pub fn detect_codex_cli() -> CodexAvailability {
    detect_with_runner(|| {
        let output = Command::new("codex")
            .arg("--version")
            .output()
            .map_err(|error| error.to_string())?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout.is_empty() {
                Ok("codex version unknown".to_string())
            } else {
                Ok(stdout)
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("codex --version exited with {}", output.status)
            } else {
                stderr
            })
        }
    })
}

pub fn notification_to_attention_item(
    app_data_dir: &Path,
    workspace_id: &str,
    request: CodexApprovalRequest,
) -> Result<Option<AttentionItem>, AppError> {
    if workspace_id.trim().is_empty()
        || request.thread_id.trim().is_empty()
        || request.approval_id.trim().is_empty()
    {
        return Ok(None);
    }

    let item = create_attention_item(
        app_data_dir,
        CreateAttentionItem {
            workspace_id: workspace_id.trim().to_string(),
            session_id: Some(request.thread_id.trim().to_string()),
            kind: "approval".to_string(),
            priority: 3,
            title: "Codex approval required".to_string(),
            summary: request.summary.trim().to_string(),
            action_label: Some(request.title.trim().to_string()),
            action_ref: Some(format!(
                "codex://thread/{}/approval/{}",
                request.thread_id.trim(),
                request.approval_id.trim()
            )),
        },
    )?;

    Ok(Some(item))
}

pub fn notification_value_to_attention_item(
    app_data_dir: &Path,
    value: &Value,
) -> Result<Option<AttentionItem>, AppError> {
    let method = value
        .get("method")
        .and_then(Value::as_str)
        .or_else(|| value.get("type").and_then(Value::as_str))
        .unwrap_or_default();
    if !method.contains("approval") {
        return Ok(None);
    }

    let params = value.get("params").unwrap_or(value);
    let workspace_id = params
        .get("workspaceId")
        .or_else(|| params.get("workspace_id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let thread_id = params
        .get("threadId")
        .or_else(|| params.get("thread_id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let approval_id = params
        .get("approvalId")
        .or_else(|| params.get("approval_id"))
        .or_else(|| params.get("id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let title = params
        .get("title")
        .or_else(|| params.get("command"))
        .or_else(|| params.get("toolName"))
        .and_then(Value::as_str)
        .unwrap_or("Review request");
    let summary = params
        .get("summary")
        .or_else(|| params.get("message"))
        .or_else(|| params.get("command"))
        .and_then(Value::as_str)
        .unwrap_or("Codex is waiting for approval.");

    notification_to_attention_item(
        app_data_dir,
        workspace_id,
        CodexApprovalRequest {
            thread_id: thread_id.to_string(),
            approval_id: approval_id.to_string(),
            title: title.to_string(),
            summary: summary.to_string(),
        },
    )
}

#[tauri::command]
pub async fn codex_detect() -> Result<CodexAvailability, AppError> {
    Ok(detect_codex_cli())
}

#[tauri::command]
pub async fn codex_thread_list(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<CodexThreadSummary>, AppError> {
    let workspace = workspace::find_workspace_by_id(&state.app_data_dir, &workspace_id)?
        .ok_or_else(|| AppError::InvalidInput("workspace not found".to_string()))?;
    let mut codex = state.codex.lock().await;
    codex
        .ensure_started(app, state.app_data_dir.clone())
        .await?;
    codex.thread_list(workspace.path).await
}

#[tauri::command]
pub async fn codex_thread_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    prompt: String,
) -> Result<CodexThreadSummary, AppError> {
    let workspace = workspace::find_workspace_by_id(&state.app_data_dir, &workspace_id)?
        .ok_or_else(|| AppError::InvalidInput("workspace not found".to_string()))?;
    let mut codex = state.codex.lock().await;
    codex
        .ensure_started(app, state.app_data_dir.clone())
        .await?;
    codex
        .thread_start(workspace_id, workspace.path, prompt)
        .await
}

#[tauri::command]
pub async fn codex_turn_send(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    thread_id: String,
    prompt: String,
) -> Result<(), AppError> {
    let mut codex = state.codex.lock().await;
    codex
        .ensure_started(app, state.app_data_dir.clone())
        .await?;
    codex.turn_send(thread_id, prompt).await
}

#[tauri::command]
pub async fn codex_turn_interrupt(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    thread_id: String,
) -> Result<(), AppError> {
    let mut codex = state.codex.lock().await;
    codex
        .ensure_started(app, state.app_data_dir.clone())
        .await?;
    codex.turn_interrupt(thread_id).await
}
