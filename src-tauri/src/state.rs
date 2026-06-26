use std::{path::PathBuf, sync::Arc};

use tokio::sync::Mutex;

use crate::codex::process::CodexProcessManager;

#[derive(Clone)]
pub struct AppState {
    pub app_data_dir: PathBuf,
    pub codex: Arc<Mutex<CodexProcessManager>>,
}
