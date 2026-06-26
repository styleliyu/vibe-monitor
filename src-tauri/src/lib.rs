pub mod attention;
pub mod codex;
pub mod db;
pub mod error;
pub mod state;
pub mod terminal;
pub mod workspace;

use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            db::init_db(&app_data_dir)?;
            app.manage(state::AppState {
                app_data_dir,
                codex: Arc::new(Mutex::new(codex::process::CodexProcessManager::default())),
                terminal: Arc::new(Mutex::new(terminal::TerminalManager::new(Box::new(
                    terminal::PtyTerminalLauncher::new(app.handle().clone()),
                )))),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            attention::attention_list,
            attention::attention_create,
            attention::attention_resolve,
            codex::codex_detect,
            codex::codex_thread_list,
            codex::codex_thread_start,
            codex::codex_turn_send,
            codex::codex_turn_interrupt,
            terminal::terminal_open,
            terminal::terminal_write,
            terminal::terminal_resize,
            terminal::terminal_close,
            workspace::workspace_list,
            workspace::workspace_add,
            workspace::workspace_remove
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
