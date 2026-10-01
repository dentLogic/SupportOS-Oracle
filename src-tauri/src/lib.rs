// The version reported by the `app_version` command. Must stay in sync with
// `version` in tauri.conf.json.
fn package_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn app_version() -> String {
    package_version()
}

#[cfg(test)]
mod tests {
    use super::package_version;

    #[test]
    fn reports_a_semver_version() {
        let parts: Vec<_> = package_version().split('.').collect();
        assert_eq!(parts.len(), 3, "version is not `major.minor.patch`");
        for part in parts {
            assert!(
                !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()),
                "version part {part:?} is not numeric"
            );
        }
    }

    #[test]
    fn reports_the_crate_version() {
        assert_eq!(package_version(), env!("CARGO_PKG_VERSION"));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![app_version])
        .run(tauri::generate_context!())
}
