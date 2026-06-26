use std::{
    path::{Path, PathBuf},
    process::Command,
};

use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db, error::AppError, state::AppState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: String,
    pub git_root: Option<String>,
    pub default_ai_engine: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn list_workspaces(app_data_dir: &Path) -> Result<Vec<Workspace>, AppError> {
    let connection = db::connection(app_data_dir)?;
    let mut statement = connection.prepare(
        r#"
        SELECT id, name, path, git_root, default_ai_engine, created_at, updated_at
        FROM workspaces
        ORDER BY updated_at DESC, name ASC
        "#,
    )?;
    let rows = statement.query_map([], workspace_from_row)?;

    let mut workspaces = Vec::new();
    for row in rows {
        workspaces.push(row?);
    }
    Ok(workspaces)
}

pub fn add_workspace(
    app_data_dir: &Path,
    path: String,
    name: Option<String>,
) -> Result<Workspace, AppError> {
    let workspace_path = validate_workspace_path(&path)?;
    let canonical_path = workspace_path.canonicalize()?;
    let display_path = path_to_display_string(&canonical_path);
    let workspace_name = match name.map(|value| value.trim().to_string()) {
        Some(value) if !value.is_empty() => value,
        _ => directory_name(&canonical_path)?,
    };
    let now = Utc::now().to_rfc3339();
    let workspace = Workspace {
        id: Uuid::new_v4().to_string(),
        name: workspace_name,
        path: display_path,
        git_root: detect_git_root(&canonical_path),
        default_ai_engine: "codex".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    let connection = db::connection(app_data_dir)?;
    connection.execute(
        r#"
        INSERT INTO workspaces (
            id, name, path, git_root, default_ai_engine, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(path) DO UPDATE SET
            name = excluded.name,
            git_root = excluded.git_root,
            default_ai_engine = excluded.default_ai_engine,
            updated_at = excluded.updated_at
        "#,
        params![
            workspace.id,
            workspace.name,
            workspace.path,
            workspace.git_root,
            workspace.default_ai_engine,
            workspace.created_at,
            workspace.updated_at
        ],
    )?;

    find_workspace_by_path(app_data_dir, &workspace.path)?
        .ok_or_else(|| AppError::Database("workspace was not saved".to_string()))
}

pub fn remove_workspace(app_data_dir: &Path, id: String) -> Result<(), AppError> {
    let connection = db::connection(app_data_dir)?;
    connection.execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
    Ok(())
}

#[tauri::command]
pub async fn workspace_list(state: tauri::State<'_, AppState>) -> Result<Vec<Workspace>, AppError> {
    list_workspaces(&state.app_data_dir)
}

#[tauri::command]
pub async fn workspace_add(
    state: tauri::State<'_, AppState>,
    path: String,
    name: Option<String>,
) -> Result<Workspace, AppError> {
    add_workspace(&state.app_data_dir, path, name)
}

#[tauri::command]
pub async fn workspace_remove(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    remove_workspace(&state.app_data_dir, id)
}

fn workspace_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        git_root: row.get(3)?,
        default_ai_engine: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn find_workspace_by_path(app_data_dir: &Path, path: &str) -> Result<Option<Workspace>, AppError> {
    let connection = db::connection(app_data_dir)?;
    let workspace = connection
        .query_row(
            r#"
            SELECT id, name, path, git_root, default_ai_engine, created_at, updated_at
            FROM workspaces
            WHERE path = ?1
            "#,
            params![path],
            workspace_from_row,
        )
        .optional()?;

    Ok(workspace)
}

pub fn find_workspace_by_id(app_data_dir: &Path, id: &str) -> Result<Option<Workspace>, AppError> {
    let connection = db::connection(app_data_dir)?;
    let workspace = connection
        .query_row(
            r#"
            SELECT id, name, path, git_root, default_ai_engine, created_at, updated_at
            FROM workspaces
            WHERE id = ?1
            "#,
            params![id],
            workspace_from_row,
        )
        .optional()?;

    Ok(workspace)
}

fn validate_workspace_path(path: &str) -> Result<PathBuf, AppError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(
            "workspace path cannot be empty".to_string(),
        ));
    }

    let path = PathBuf::from(trimmed);
    if !path.is_dir() {
        return Err(AppError::InvalidInput(format!(
            "workspace path is not a directory: {trimmed}"
        )));
    }

    Ok(path)
}

fn directory_name(path: &Path) -> Result<String, AppError> {
    path.file_name()
        .map(|value| value.to_string_lossy().to_string())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::InvalidInput("workspace directory has no name".to_string()))
}

fn detect_git_root(path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args([
            "-C",
            &path.to_string_lossy(),
            "rev-parse",
            "--show-toplevel",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return None;
    }

    let git_root = PathBuf::from(stdout);
    let normalized_git_root = match git_root.canonicalize() {
        Ok(path) => path,
        Err(_) => git_root,
    };
    Some(path_to_display_string(&normalized_git_root))
}

fn path_to_display_string(path: &Path) -> String {
    const EXTENDED_PATH_PREFIX: &str = r"\\?\";
    let path = path.to_string_lossy().to_string();
    path.strip_prefix(EXTENDED_PATH_PREFIX)
        .unwrap_or(&path)
        .to_string()
}
