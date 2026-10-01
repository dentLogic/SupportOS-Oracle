//! Application identity helpers.

/// The application version, taken from this crate's `CARGO_PKG_VERSION`.
///
/// The version is kept in lockstep across the workspace crates and
/// `src-tauri/tauri.conf.json`; the version-consistency CI check lands in
/// Slice 3 (see docs/DECISIONS.md).
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn reports_a_semver_version() {
        let reported = version();
        let parts: Vec<_> = reported.split('.').collect();
        assert_eq!(parts.len(), 3, "version is not `major.minor.patch`");
        for part in parts {
            assert!(
                !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()),
                "version part {part:?} is not numeric"
            );
        }
    }
}
