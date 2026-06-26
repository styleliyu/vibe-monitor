use tempfile::tempdir;
use vibe_monitor_lib::{attention, db, git, workspace};

#[test]
fn git_status_parser_reads_branch_ahead_behind_and_files() {
    let status = git::parse_status_output(
        "## feature/git-status-diff...origin/feature/git-status-diff [ahead 1, behind 2]\n M src/main.rs\nA  src/new.rs\n?? notes.md\nUU src/conflict.rs\n",
    )
    .expect("status output should parse");

    assert!(status.available);
    assert_eq!(status.branch.as_deref(), Some("feature/git-status-diff"));
    assert_eq!(status.ahead, 1);
    assert_eq!(status.behind, 2);
    assert_eq!(status.files.len(), 4);
    assert_eq!(status.files[0].path, "src/main.rs");
    assert_eq!(status.files[0].index_status, " ");
    assert_eq!(status.files[0].worktree_status, "M");
    assert_eq!(status.files[2].path, "notes.md");
    assert_eq!(status.files[2].index_status, "?");
    assert_eq!(status.files[2].worktree_status, "?");
    assert!(status.has_unmerged_files());
}

#[test]
fn git_status_returns_unavailable_for_workspace_without_git_root() {
    let app_data = tempdir().expect("app data tempdir");
    let workspace_dir = tempdir().expect("workspace tempdir");
    db::init_db(app_data.path()).expect("database initialization");
    let workspace = workspace::add_workspace(
        app_data.path(),
        workspace_dir.path().to_string_lossy().to_string(),
        None,
    )
    .expect("workspace should be added");

    let status = git::status(app_data.path(), workspace.id).expect("git status should not error");

    assert!(!status.available);
    assert_eq!(status.files, Vec::new());
    assert_eq!(
        status.error.as_deref(),
        Some("workspace is not a Git repository")
    );
}

#[test]
fn unmerged_status_creates_blocked_attention_item() {
    let app_data = tempdir().expect("app data tempdir");
    db::init_db(app_data.path()).expect("database initialization");
    let status = git::parse_status_output("## main\nUU src/conflict.rs\n")
        .expect("status output should parse");

    git::create_conflict_attention_item(app_data.path(), "workspace-1", &status)
        .expect("conflict attention should be created");

    let items = attention::list_attention_items(app_data.path(), Some("workspace-1".to_string()))
        .expect("attention list should load");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind, "blocked");
    assert_eq!(items[0].priority, 3);
    assert_eq!(items[0].title, "Git conflict requires attention");
}
