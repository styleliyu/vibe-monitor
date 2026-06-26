use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::AppError;

pub fn db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("vibe-monitor.db")
}

pub fn connection(app_data_dir: &Path) -> Result<Connection, AppError> {
    Ok(Connection::open(db_path(app_data_dir))?)
}

pub fn init_db(app_data_dir: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(app_data_dir)?;
    let connection = connection(app_data_dir)?;
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS workspaces (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          path TEXT NOT NULL UNIQUE,
          git_root TEXT,
          default_ai_engine TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        "#,
    )?;

    Ok(())
}
