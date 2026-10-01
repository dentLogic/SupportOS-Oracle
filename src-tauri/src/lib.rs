// Thin Tauri shell: each command is a one-line wrapper over a core function
// that has a Rust test (docs/SPEC-AMENDMENTS.md, A5).

use tauri_plugin_log::{Target, TargetKind};

#[tauri::command]
fn app_version() -> String {
    supportos_oracle_core::app::version()
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
        .invoke_handler(tauri::generate_handler![app_version])
        // Runs after the plugins are initialized, so the startup record
        // reaches the tauri-plugin-log sinks installed above.
        .setup(|_app| {
            supportos_oracle_core::logging::log_app_start(&supportos_oracle_core::app::version());
            Ok(())
        })
        .run(tauri::generate_context!())
}
