use std::path::Path;

use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db, error::AppError, state::AppState};

const ALLOWED_KINDS: [&str; 6] = ["approval", "blocked", "failed", "done", "unread", "info"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttentionItem {
    pub id: String,
    pub workspace_id: String,
    pub session_id: Option<String>,
    pub kind: String,
    pub priority: i64,
    pub title: String,
    pub summary: String,
    pub action_label: Option<String>,
    pub action_ref: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttentionItem {
    pub workspace_id: String,
    pub session_id: Option<String>,
    pub kind: String,
    pub priority: i64,
    pub title: String,
    pub summary: String,
    pub action_label: Option<String>,
    pub action_ref: Option<String>,
}

pub fn list_attention_items(
    app_data_dir: &Path,
    workspace_id: Option<String>,
) -> Result<Vec<AttentionItem>, AppError> {
    let connection = db::connection(app_data_dir)?;

    let mut items = Vec::new();
    if let Some(workspace_id) = workspace_id {
        let mut statement = connection.prepare(
            r#"
            SELECT id, workspace_id, session_id, kind, priority, title, summary,
                   action_label, action_ref, created_at, resolved_at
            FROM attention_items
            WHERE resolved_at IS NULL AND workspace_id = ?1
            ORDER BY priority DESC, created_at ASC
            "#,
        )?;
        let rows = statement.query_map(params![workspace_id], attention_item_from_row)?;
        for row in rows {
            items.push(row?);
        }
    } else {
        let mut statement = connection.prepare(
            r#"
            SELECT id, workspace_id, session_id, kind, priority, title, summary,
                   action_label, action_ref, created_at, resolved_at
            FROM attention_items
            WHERE resolved_at IS NULL
            ORDER BY priority DESC, created_at ASC
            "#,
        )?;
        let rows = statement.query_map([], attention_item_from_row)?;
        for row in rows {
            items.push(row?);
        }
    }

    Ok(items)
}

pub fn create_attention_item(
    app_data_dir: &Path,
    input: CreateAttentionItem,
) -> Result<AttentionItem, AppError> {
    validate_create_input(&input)?;

    let now = Utc::now().to_rfc3339();
    let item = AttentionItem {
        id: Uuid::new_v4().to_string(),
        workspace_id: input.workspace_id.trim().to_string(),
        session_id: clean_optional(input.session_id),
        kind: input.kind,
        priority: input.priority,
        title: input.title.trim().to_string(),
        summary: input.summary.trim().to_string(),
        action_label: clean_optional(input.action_label),
        action_ref: clean_optional(input.action_ref),
        created_at: now,
        resolved_at: None,
    };

    let connection = db::connection(app_data_dir)?;
    connection.execute(
        r#"
        INSERT INTO attention_items (
            id, workspace_id, session_id, kind, priority, title, summary,
            action_label, action_ref, created_at, resolved_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        params![
            item.id,
            item.workspace_id,
            item.session_id,
            item.kind,
            item.priority,
            item.title,
            item.summary,
            item.action_label,
            item.action_ref,
            item.created_at,
            item.resolved_at,
        ],
    )?;

    Ok(item)
}

pub fn resolve_attention_item(app_data_dir: &Path, id: String) -> Result<(), AppError> {
    let trimmed_id = id.trim();
    if trimmed_id.is_empty() {
        return Err(AppError::InvalidInput(
            "attention item id cannot be empty".to_string(),
        ));
    }

    let connection = db::connection(app_data_dir)?;
    connection.execute(
        "UPDATE attention_items SET resolved_at = ?1 WHERE id = ?2 AND resolved_at IS NULL",
        params![Utc::now().to_rfc3339(), trimmed_id],
    )?;
    Ok(())
}

#[tauri::command]
pub async fn attention_list(
    state: tauri::State<'_, AppState>,
    workspace_id: Option<String>,
) -> Result<Vec<AttentionItem>, AppError> {
    list_attention_items(&state.app_data_dir, clean_optional(workspace_id))
}

#[tauri::command]
pub async fn attention_create(
    state: tauri::State<'_, AppState>,
    input: CreateAttentionItem,
) -> Result<AttentionItem, AppError> {
    create_attention_item(&state.app_data_dir, input)
}

#[tauri::command]
pub async fn attention_resolve(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    resolve_attention_item(&state.app_data_dir, id)
}

fn attention_item_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AttentionItem> {
    Ok(AttentionItem {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        session_id: row.get(2)?,
        kind: row.get(3)?,
        priority: row.get(4)?,
        title: row.get(5)?,
        summary: row.get(6)?,
        action_label: row.get(7)?,
        action_ref: row.get(8)?,
        created_at: row.get(9)?,
        resolved_at: row.get(10)?,
    })
}

fn validate_create_input(input: &CreateAttentionItem) -> Result<(), AppError> {
    if input.workspace_id.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "workspace id cannot be empty".to_string(),
        ));
    }
    if !ALLOWED_KINDS.contains(&input.kind.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "unsupported attention kind: {}",
            input.kind
        )));
    }
    if !(0..=3).contains(&input.priority) {
        return Err(AppError::InvalidInput(
            "attention priority must be between 0 and 3".to_string(),
        ));
    }
    if input.title.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "attention title cannot be empty".to_string(),
        ));
    }
    if input.summary.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "attention summary cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
