use std::{process::Command, path::Path};

use tempfile::tempdir;
use vibe_monitor_lib::{db, workspace};

#[test]
fn workspace_add_rejects_non_directory_paths() {
    let app_data = tempdir().expect("app data tempdir");
    db::init_db(app_data.path()).expect("database initialization");

    let result = workspace::add_workspace(
        app_data.path(),
        app_data.path().join("missing").to_string_lossy().to_string(),
        None,
    );

    assert!(result.is_err());
}

#[test]
fn workspace_add_list_and_remove_persists_workspace_state() {
    let app_data = tempdir().expect("app data tempdir");
    let workspace_dir = tempdir().expect("workspace tempdir");
    db::init_db(app_data.path()).expect("database initialization");

    let added = workspace::add_workspace(
        app_data.path(),
        workspace_dir.path().to_string_lossy().to_string(),
        None,
    )
    .expect("workspace should be added");

    assert_eq!(added.name, directory_name(workspace_dir.path()));
    assert_eq!(added.default_ai_engine, "codex");
    assert!(added.git_root.is_none());

    let listed = workspace::list_workspaces(app_data.path()).expect("workspace list");
    assert_eq!(listed, vec![added.clone()]);

    workspace::remove_workspace(app_data.path(), added.id).expect("workspace removal");
    let listed_after_remove = workspace::list_workspaces(app_data.path()).expect("workspace list");
    assert!(listed_after_remove.is_empty());
}

#[test]
fn workspace_add_records_git_root_when_directory_is_inside_a_git_repo() {
    let app_data = tempdir().expect("app data tempdir");
    let repo_dir = tempdir().expect("repo tempdir");
    db::init_db(app_data.path()).expect("database initialization");

    let git_available = Command::new("git")
        .args(["init"])
        .current_dir(repo_dir.path())
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    if !git_available {
        return;
    }

    let nested = repo_dir.path().join("nested");
    std::fs::create_dir(&nested).expect("nested workspace directory");

    let added = workspace::add_workspace(app_data.path(), nested.to_string_lossy().to_string(), None)
        .expect("workspace should be added");

    assert_eq!(
        added.git_root.as_deref(),
        Some(repo_dir.path().to_string_lossy().as_ref())
    );
}

fn directory_name(path: &Path) -> String {
    path.file_name()
        .expect("directory should have a name")
        .to_string_lossy()
        .to_string()
}
