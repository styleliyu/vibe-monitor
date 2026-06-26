use std::{
    path::Path,
    process::{Command, Output},
};

use serde::{Deserialize, Serialize};

use crate::{
    attention::{self, CreateAttentionItem},
    error::AppError,
    state::AppState,
    workspace,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitFileStatus {
    pub path: String,
    pub index_status: String,
    pub worktree_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub available: bool,
    pub branch: Option<String>,
    pub ahead: i64,
    pub behind: i64,
    pub files: Vec<GitFileStatus>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitDiff {
    pub path: Option<String>,
    pub text: String,
}

impl GitStatus {
    pub fn unavailable(error: impl Into<String>) -> Self {
        Self {
            available: false,
            branch: None,
            ahead: 0,
            behind: 0,
            files: Vec::new(),
            error: Some(error.into()),
        }
    }

    pub fn has_unmerged_files(&self) -> bool {
        self.files.iter().any(|file| {
            matches!(
                (file.index_status.as_str(), file.worktree_status.as_str()),
                ("D", "D")
                    | ("A", "U")
                    | ("U", "D")
                    | ("U", "A")
                    | ("D", "U")
                    | ("A", "A")
                    | ("U", "U")
            )
        })
    }
}

pub fn status(app_data_dir: &Path, workspace_id: String) -> Result<GitStatus, AppError> {
    let workspace = workspace::find_workspace_by_id(app_data_dir, workspace_id.trim())?
        .ok_or_else(|| AppError::InvalidInput("workspace not found".to_string()))?;
    let Some(git_root) = workspace.git_root else {
        return Ok(GitStatus::unavailable("workspace is not a Git repository"));
    };

    let output = run_git(&git_root, &["status", "--porcelain=v1", "--branch"])?;
    if !output.status.success() {
        return Ok(GitStatus::unavailable(command_output_message(&output)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let status = parse_status_output(&stdout)?;
    create_conflict_attention_item(app_data_dir, &workspace.id, &status)?;
    Ok(status)
}

pub fn diff(
    app_data_dir: &Path,
    workspace_id: String,
    path: Option<String>,
) -> Result<GitDiff, AppError> {
    let git_root = git_root_for_workspace(app_data_dir, &workspace_id)?;
    let clean_path = clean_optional_path(path)?;

    let mut command = Command::new("git");
    command.arg("-C").arg(git_root).arg("diff");
    if let Some(path) = clean_path.as_ref() {
        command.arg("--").arg(path);
    }

    let output = command.output()?;
    if !output.status.success() {
        return Err(AppError::CommandFailed(command_output_message(&output)));
    }

    Ok(GitDiff {
        path: clean_path,
        text: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}

pub fn stage(app_data_dir: &Path, workspace_id: String, path: String) -> Result<(), AppError> {
    let git_root = git_root_for_workspace(app_data_dir, &workspace_id)?;
    let path = validate_path(path)?;
    run_git_checked(&git_root, &["add", "--", &path])
}

pub fn unstage(app_data_dir: &Path, workspace_id: String, path: String) -> Result<(), AppError> {
    let git_root = git_root_for_workspace(app_data_dir, &workspace_id)?;
    let path = validate_path(path)?;
    run_git_checked(&git_root, &["restore", "--staged", "--", &path])
}

pub fn parse_status_output(output: &str) -> Result<GitStatus, AppError> {
    let mut status = GitStatus {
        available: true,
        branch: None,
        ahead: 0,
        behind: 0,
        files: Vec::new(),
        error: None,
    };

    for line in output.lines() {
        if let Some(header) = line.strip_prefix("## ") {
            parse_branch_header(header, &mut status);
            continue;
        }

        if line.len() < 4 {
            continue;
        }

        let index_status = line[0..1].to_string();
        let worktree_status = line[1..2].to_string();
        let mut path = line[3..].trim().to_string();
        if let Some((_, renamed_path)) = path.rsplit_once(" -> ") {
            path = renamed_path.to_string();
        }

        if !path.is_empty() {
            status.files.push(GitFileStatus {
                path,
                index_status,
                worktree_status,
            });
        }
    }

    Ok(status)
}

pub fn create_conflict_attention_item(
    app_data_dir: &Path,
    workspace_id: &str,
    status: &GitStatus,
) -> Result<(), AppError> {
    if !status.has_unmerged_files() {
        return Ok(());
    }

    let existing = attention::list_attention_items(app_data_dir, Some(workspace_id.to_string()))?;
    if existing
        .iter()
        .any(|item| item.kind == "blocked" && item.title == "Git conflict requires attention")
    {
        return Ok(());
    }

    attention::create_attention_item(
        app_data_dir,
        CreateAttentionItem {
            workspace_id: workspace_id.to_string(),
            session_id: None,
            kind: "blocked".to_string(),
            priority: 3,
            title: "Git conflict requires attention".to_string(),
            summary: "Resolve unmerged files before continuing normal development work."
                .to_string(),
            action_label: Some("Open Git".to_string()),
            action_ref: Some("git://conflicts".to_string()),
        },
    )?;
    Ok(())
}

#[tauri::command]
pub async fn git_status(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
) -> Result<GitStatus, AppError> {
    status(&state.app_data_dir, workspace_id)
}

#[tauri::command]
pub async fn git_diff(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    path: Option<String>,
) -> Result<GitDiff, AppError> {
    diff(&state.app_data_dir, workspace_id, path)
}

#[tauri::command]
pub async fn git_stage(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    path: String,
) -> Result<(), AppError> {
    stage(&state.app_data_dir, workspace_id, path)
}

#[tauri::command]
pub async fn git_unstage(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    path: String,
) -> Result<(), AppError> {
    unstage(&state.app_data_dir, workspace_id, path)
}

fn parse_branch_header(header: &str, status: &mut GitStatus) {
    let (branch_part, metadata) = match header.split_once(" [") {
        Some((branch, metadata)) => (branch, Some(metadata.trim_end_matches(']'))),
        None => (header, None),
    };

    status.branch = parse_branch_name(branch_part);
    if let Some(metadata) = metadata {
        for part in metadata.split(',') {
            let part = part.trim();
            if let Some(value) = part.strip_prefix("ahead ") {
                status.ahead = value.parse::<i64>().unwrap_or(0);
            }
            if let Some(value) = part.strip_prefix("behind ") {
                status.behind = value.parse::<i64>().unwrap_or(0);
            }
        }
    }
}

fn parse_branch_name(branch_part: &str) -> Option<String> {
    let branch = branch_part
        .split_once("...")
        .map(|(branch, _)| branch)
        .unwrap_or(branch_part)
        .trim();
    let branch = branch
        .strip_prefix("No commits yet on ")
        .unwrap_or(branch)
        .trim();

    if branch.is_empty() {
        None
    } else {
        Some(branch.to_string())
    }
}

fn git_root_for_workspace(app_data_dir: &Path, workspace_id: &str) -> Result<String, AppError> {
    let workspace = workspace::find_workspace_by_id(app_data_dir, workspace_id.trim())?
        .ok_or_else(|| AppError::InvalidInput("workspace not found".to_string()))?;
    workspace
        .git_root
        .ok_or_else(|| AppError::InvalidInput("workspace is not a Git repository".to_string()))
}

fn run_git(git_root: &str, args: &[&str]) -> Result<Output, AppError> {
    Ok(Command::new("git")
        .arg("-C")
        .arg(git_root)
        .args(args)
        .output()?)
}

fn run_git_checked(git_root: &str, args: &[&str]) -> Result<(), AppError> {
    let output = run_git(git_root, args)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::CommandFailed(command_output_message(&output)))
    }
}

fn command_output_message(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }

    format!("git exited with status {}", output.status)
}

fn clean_optional_path(path: Option<String>) -> Result<Option<String>, AppError> {
    path.map(validate_path).transpose()
}

fn validate_path(path: String) -> Result<String, AppError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(AppError::InvalidInput(
            "git path cannot be empty".to_string(),
        ));
    }
    if path.contains('\0') {
        return Err(AppError::InvalidInput(
            "git path cannot contain NUL bytes".to_string(),
        ));
    }
    Ok(path.to_string())
}
