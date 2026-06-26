pub mod attention;
pub mod db;
pub mod error;
pub mod state;
pub mod workspace;

use tauri::Manager;

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
            app.manage(state::AppState { app_data_dir });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            attention::attention_list,
            attention::attention_create,
            attention::attention_resolve,
            workspace::workspace_list,
            workspace::workspace_add,
            workspace::workspace_remove
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
