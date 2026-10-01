// Thin Tauri shell: each command is a one-line wrapper over a core function
// that has a Rust test (docs/SPEC-AMENDMENTS.md, A5).

#[tauri::command]
fn app_version() -> String {
    supportos_oracle_core::app::version()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![app_version])
        .run(tauri::generate_context!())
}
