use std::{
    collections::HashMap,
    io::{Read, Write},
    path::Path,
    thread,
};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::{error::AppError, state::AppState, workspace};

pub const OUTPUT_EVENT: &str = "terminal://output";
const DEFAULT_COLS: u16 = 80;
const DEFAULT_ROWS: u16 = 24;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSession {
    pub id: String,
    pub workspace_id: String,
    pub cwd: String,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLaunchSpec {
    pub session_id: String,
    pub workspace_id: String,
    pub cwd: String,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutputEvent {
    pub session_id: String,
    pub data: String,
}

pub trait TerminalBackend: Send {
    fn write(&mut self, data: &str) -> Result<(), AppError>;
    fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AppError>;
    fn close(&mut self) -> Result<(), AppError>;
}

pub trait TerminalLauncher: Send {
    fn spawn(&self, spec: TerminalLaunchSpec) -> Result<Box<dyn TerminalBackend>, AppError>;
}

pub struct TerminalManager {
    sessions: HashMap<String, TerminalEntry>,
    launcher: Box<dyn TerminalLauncher>,
}

struct TerminalEntry {
    session: TerminalSession,
    backend: Box<dyn TerminalBackend>,
}

impl TerminalManager {
    pub fn new(launcher: Box<dyn TerminalLauncher>) -> Self {
        Self {
            sessions: HashMap::new(),
            launcher,
        }
    }

    pub fn open_session(
        &mut self,
        app_data_dir: &Path,
        workspace_id: String,
        cols: u16,
        rows: u16,
    ) -> Result<TerminalSession, AppError> {
        validate_dimensions(cols, rows)?;
        let workspace_id = workspace_id.trim();
        if workspace_id.is_empty() {
            return Err(AppError::InvalidInput(
                "workspace id cannot be empty".to_string(),
            ));
        }

        let workspace = workspace::find_workspace_by_id(app_data_dir, workspace_id)?
            .ok_or_else(|| AppError::InvalidInput("workspace was not found".to_string()))?;
        let session_id = Uuid::new_v4().to_string();
        let shell = default_shell();
        let spec = TerminalLaunchSpec {
            session_id: session_id.clone(),
            workspace_id: workspace.id.clone(),
            cwd: workspace.path.clone(),
            shell: shell.clone(),
            cols,
            rows,
        };
        let backend = self.launcher.spawn(spec)?;
        let session = TerminalSession {
            id: session_id.clone(),
            workspace_id: workspace.id,
            cwd: workspace.path,
            shell,
            cols,
            rows,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.sessions.insert(
            session_id,
            TerminalEntry {
                session: session.clone(),
                backend,
            },
        );
        Ok(session)
    }

    pub fn write_session(&mut self, session_id: &str, data: &str) -> Result<(), AppError> {
        let entry = self.entry_mut(session_id)?;
        entry.backend.write(data)
    }

    pub fn resize_session(
        &mut self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), AppError> {
        validate_dimensions(cols, rows)?;
        let entry = self.entry_mut(session_id)?;
        entry.backend.resize(cols, rows)?;
        entry.session.cols = cols;
        entry.session.rows = rows;
        Ok(())
    }

    pub fn close_session(&mut self, session_id: &str) -> Result<(), AppError> {
        let mut entry = self
            .sessions
            .remove(session_id.trim())
            .ok_or_else(|| AppError::InvalidInput("terminal session was not found".to_string()))?;
        entry.backend.close()
    }

    pub fn session(&self, session_id: &str) -> Option<&TerminalSession> {
        self.sessions
            .get(session_id.trim())
            .map(|entry| &entry.session)
    }

    fn entry_mut(&mut self, session_id: &str) -> Result<&mut TerminalEntry, AppError> {
        self.sessions
            .get_mut(session_id.trim())
            .ok_or_else(|| AppError::InvalidInput("terminal session was not found".to_string()))
    }
}

pub struct PtyTerminalLauncher {
    app: AppHandle,
}

impl PtyTerminalLauncher {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl TerminalLauncher for PtyTerminalLauncher {
    fn spawn(&self, spec: TerminalLaunchSpec) -> Result<Box<dyn TerminalBackend>, AppError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: spec.rows,
                cols: spec.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| {
                AppError::CommandFailed(format!("failed to open terminal: {error}"))
            })?;

        let mut command = CommandBuilder::new(&spec.shell);
        command.cwd(&spec.cwd);
        let child = pair.slave.spawn_command(command).map_err(|error| {
            AppError::CommandFailed(format!("failed to start terminal: {error}"))
        })?;
        let mut reader = pair.master.try_clone_reader().map_err(|error| {
            AppError::CommandFailed(format!("failed to read terminal output: {error}"))
        })?;
        let writer = pair.master.take_writer().map_err(|error| {
            AppError::CommandFailed(format!("failed to open terminal input: {error}"))
        })?;

        let app = self.app.clone();
        let session_id = spec.session_id;
        thread::spawn(move || stream_terminal_output(app, session_id, &mut reader));

        Ok(Box::new(PtyTerminalBackend {
            master: pair.master,
            child,
            writer,
        }))
    }
}

struct PtyTerminalBackend {
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
}

impl TerminalBackend for PtyTerminalBackend {
    fn write(&mut self, data: &str) -> Result<(), AppError> {
        self.writer.write_all(data.as_bytes()).map_err(|error| {
            AppError::CommandFailed(format!("failed to write terminal input: {error}"))
        })?;
        self.writer.flush().map_err(|error| {
            AppError::CommandFailed(format!("failed to flush terminal input: {error}"))
        })
    }

    fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AppError> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| AppError::CommandFailed(format!("failed to resize terminal: {error}")))
    }

    fn close(&mut self) -> Result<(), AppError> {
        self.child.kill().map_err(|error| {
            AppError::CommandFailed(format!("failed to close terminal process: {error}"))
        })
    }
}

impl Drop for PtyTerminalBackend {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

#[cfg(windows)]
pub fn default_shell() -> String {
    "powershell.exe".to_string()
}

#[cfg(not(windows))]
pub fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

#[tauri::command]
pub async fn terminal_open(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
) -> Result<TerminalSession, AppError> {
    let mut terminal = state.terminal.lock().await;
    terminal.open_session(
        &state.app_data_dir,
        workspace_id,
        DEFAULT_COLS,
        DEFAULT_ROWS,
    )
}

#[tauri::command]
pub async fn terminal_write(
    state: tauri::State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), AppError> {
    let mut terminal = state.terminal.lock().await;
    terminal.write_session(&session_id, &data)
}

#[tauri::command]
pub async fn terminal_resize(
    state: tauri::State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), AppError> {
    let mut terminal = state.terminal.lock().await;
    terminal.resize_session(&session_id, cols, rows)
}

#[tauri::command]
pub async fn terminal_close(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<(), AppError> {
    let mut terminal = state.terminal.lock().await;
    terminal.close_session(&session_id)
}

fn stream_terminal_output(app: AppHandle, session_id: String, reader: &mut Box<dyn Read + Send>) {
    let mut buffer = [0_u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                let data = String::from_utf8_lossy(&buffer[..count]).to_string();
                let _ = app.emit(
                    OUTPUT_EVENT,
                    TerminalOutputEvent {
                        session_id: session_id.clone(),
                        data,
                    },
                );
            }
        }
    }
}

fn validate_dimensions(cols: u16, rows: u16) -> Result<(), AppError> {
    if cols == 0 || rows == 0 {
        return Err(AppError::InvalidInput(
            "terminal dimensions must be greater than zero".to_string(),
        ));
    }
    Ok(())
}
