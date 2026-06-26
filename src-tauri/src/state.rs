use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub app_data_dir: PathBuf,
}
