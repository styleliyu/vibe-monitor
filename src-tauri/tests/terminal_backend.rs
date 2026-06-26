use std::sync::{Arc, Mutex};

use tempfile::tempdir;
use vibe_monitor_lib::{
    db,
    error::AppError,
    terminal::{self, TerminalBackend, TerminalLaunchSpec, TerminalLauncher, TerminalManager},
    workspace,
};

#[derive(Clone, Default)]
struct FakeLauncher {
    specs: Arc<Mutex<Vec<TerminalLaunchSpec>>>,
    writes: Arc<Mutex<Vec<String>>>,
}

impl TerminalLauncher for FakeLauncher {
    fn spawn(&self, spec: TerminalLaunchSpec) -> Result<Box<dyn TerminalBackend>, AppError> {
        self.specs.lock().expect("specs lock").push(spec);
        Ok(Box::new(FakeTerminalBackend {
            writes: self.writes.clone(),
        }))
    }
}

struct FakeTerminalBackend {
    writes: Arc<Mutex<Vec<String>>>,
}

impl TerminalBackend for FakeTerminalBackend {
    fn write(&mut self, data: &str) -> Result<(), AppError> {
        self.writes
            .lock()
            .expect("writes lock")
            .push(data.to_string());
        Ok(())
    }

    fn resize(&mut self, _cols: u16, _rows: u16) -> Result<(), AppError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), AppError> {
        Ok(())
    }
}

#[test]
fn terminal_open_rejects_unknown_workspace_without_spawning() {
    let app_data = tempdir().expect("app data tempdir");
    db::init_db(app_data.path()).expect("database initialization");
    let launcher = FakeLauncher::default();
    let mut manager = TerminalManager::new(Box::new(launcher.clone()));

    let result = manager.open_session(app_data.path(), "missing-workspace".to_string(), 80, 24);

    assert!(result.is_err());
    assert!(launcher.specs.lock().expect("specs lock").is_empty());
}

#[test]
fn terminal_open_stores_session_and_launches_default_shell_in_workspace() {
    let app_data = tempdir().expect("app data tempdir");
    let workspace_dir = tempdir().expect("workspace tempdir");
    db::init_db(app_data.path()).expect("database initialization");
    let workspace = workspace::add_workspace(
        app_data.path(),
        workspace_dir.path().to_string_lossy().to_string(),
        None,
    )
    .expect("workspace should be added");
    let launcher = FakeLauncher::default();
    let mut manager = TerminalManager::new(Box::new(launcher.clone()));

    let session = manager
        .open_session(app_data.path(), workspace.id.clone(), 120, 30)
        .expect("terminal should open");

    assert_eq!(session.workspace_id, workspace.id);
    assert_eq!(session.cwd, workspace.path);
    assert_eq!(session.cols, 120);
    assert_eq!(session.rows, 30);

    let specs = launcher.specs.lock().expect("specs lock");
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].cwd, workspace.path);
    assert_eq!(specs[0].shell, terminal::default_shell());
    assert_eq!(specs[0].cols, 120);
    assert_eq!(specs[0].rows, 30);
}

#[test]
fn terminal_resize_rejects_zero_dimensions_and_updates_session() {
    let app_data = tempdir().expect("app data tempdir");
    let workspace_dir = tempdir().expect("workspace tempdir");
    db::init_db(app_data.path()).expect("database initialization");
    let workspace = workspace::add_workspace(
        app_data.path(),
        workspace_dir.path().to_string_lossy().to_string(),
        None,
    )
    .expect("workspace should be added");
    let mut manager = TerminalManager::new(Box::new(FakeLauncher::default()));
    let session = manager
        .open_session(app_data.path(), workspace.id, 80, 24)
        .expect("terminal should open");

    assert!(manager.resize_session(&session.id, 0, 24).is_err());

    manager
        .resize_session(&session.id, 100, 40)
        .expect("terminal should resize");
    let resized = manager.session(&session.id).expect("session should exist");
    assert_eq!(resized.cols, 100);
    assert_eq!(resized.rows, 40);
}

#[test]
fn terminal_write_and_close_reject_unknown_session_ids() {
    let app_data = tempdir().expect("app data tempdir");
    let workspace_dir = tempdir().expect("workspace tempdir");
    db::init_db(app_data.path()).expect("database initialization");
    let workspace = workspace::add_workspace(
        app_data.path(),
        workspace_dir.path().to_string_lossy().to_string(),
        None,
    )
    .expect("workspace should be added");
    let launcher = FakeLauncher::default();
    let mut manager = TerminalManager::new(Box::new(launcher.clone()));
    let session = manager
        .open_session(app_data.path(), workspace.id, 80, 24)
        .expect("terminal should open");

    assert!(manager.write_session("missing", "git status\r\n").is_err());
    assert!(manager.close_session("missing").is_err());

    manager
        .write_session(&session.id, "git status\r\n")
        .expect("terminal should accept input");
    assert_eq!(
        launcher.writes.lock().expect("writes lock").as_slice(),
        &["git status\r\n".to_string()]
    );

    manager
        .close_session(&session.id)
        .expect("terminal should close");
    assert!(manager
        .write_session(&session.id, "node --version\r\n")
        .is_err());
}
