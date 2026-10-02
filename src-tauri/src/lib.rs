// Thin Tauri shell: each command is a one-line wrapper over a core function
// that has a Rust test (docs/SPEC-AMENDMENTS.md, A5).

use std::fs;

use supportos_oracle_core::db::{self, SharedDb};
use supportos_oracle_core::settings;
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

/// The file name of the application database inside the app data directory.
const DATABASE_FILE: &str = "supportos-oracle.db";

#[tauri::command]
fn app_version() -> String {
    supportos_oracle_core::app::version()
}

#[tauri::command]
fn settings_get(db: tauri::State<SharedDb>, key: String) -> Result<Option<String>, String> {
    db.with(|conn| settings::get(conn, &key)).map_err(|e| e.to_string())
}

#[tauri::command]
fn settings_set(db: tauri::State<SharedDb>, key: String, value: String) -> Result<(), String> {
    db.with(|conn| settings::set(conn, &key, &value)).map_err(|e| e.to_string())
}

/// The persisted theme name, or "system" when no preference is stored.
#[tauri::command]
fn settings_theme(db: tauri::State<SharedDb>) -> Result<String, String> {
    db.with(|conn| settings::theme(conn))
        .map(|stored| stored.unwrap_or_else(|| "system".to_string()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn settings_set_theme(db: tauri::State<SharedDb>, theme: String) -> Result<(), String> {
    db.with(|conn| settings::set_theme(conn, &theme)).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(supportos_oracle_core::logging::default_level_filter())
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            app_version,
            settings_get,
            settings_set,
            settings_theme,
            settings_set_theme
        ])
        // Runs after the plugins are initialized, so the startup record
        // reaches the tauri-plugin-log sinks installed above. A database
        // that cannot be opened fails the launch: without its local state
        // the app has nothing to show (A16: no silent failure).
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&data_dir)?;
            let conn = db::open(data_dir.join(DATABASE_FILE))?;
            app.manage(SharedDb::new(conn));
            supportos_oracle_core::logging::log_app_start(&supportos_oracle_core::app::version());
            Ok(())
        })
        .run(tauri::generate_context!())
}
