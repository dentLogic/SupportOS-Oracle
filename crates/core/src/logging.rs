//! The logging facade.
//!
//! Core code emits records through the `log` facade and never installs a
//! sink: the Tauri shell installs tauri-plugin-log (stdout plus a log file in
//! the platform log directory), so the same facade serves core tests, the
//! shell and later settings-driven level configuration without core gaining
//! a dependency on any sink (docs/SPEC-AMENDMENTS.md, A5: core stays pure).
//!
//! Secrets must never be written to logs (docs/SPEC.md section 19); this
//! module therefore only carries level plumbing and the version-bearing
//! startup record, never business payloads.

use log::LevelFilter;

/// The fallback filter used when no valid level name is configured.
///
/// `Info` keeps the log useful (launches, migrations, job outcomes) without
/// the noise of debug traces; settings will be able to raise or lower it
/// once the settings store exists.
pub fn default_level_filter() -> LevelFilter {
    LevelFilter::Info
}

/// Maps a configured level name onto a [`LevelFilter`].
///
/// Matching is case-insensitive and exact: whitespace is not trimmed, so
/// `Settings` must hand over a clean value. Unknown names return `None` so
/// the caller can log a warning and fall back to [`default_level_filter`]
/// explicitly instead of silently guessing a level (docs/SPEC-AMENDMENTS.md,
/// A16: no silent failure).
pub fn level_filter_from_name(name: &str) -> Option<LevelFilter> {
    match name.to_ascii_lowercase().as_str() {
        "off" => Some(LevelFilter::Off),
        "error" => Some(LevelFilter::Error),
        "warn" => Some(LevelFilter::Warn),
        "info" => Some(LevelFilter::Info),
        "debug" => Some(LevelFilter::Debug),
        "trace" => Some(LevelFilter::Trace),
        _ => None,
    }
}

/// Emits the application startup record through the facade.
///
/// The shell calls this from its setup hook, after the sink plugin is
/// installed, so every launch is visible in the log file with the exact
/// version that produced it.
pub fn log_app_start(version: &str) {
    log::info!("SupportOS Oracle starting (version {version})");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Records every core record the capture logger receives, as
    /// `(level, rendered message)` pairs.
    static CAPTURED: Mutex<Vec<(log::Level, String)>> = Mutex::new(Vec::new());

    struct Capture;

    impl log::Log for Capture {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            if record.target().starts_with("supportos_oracle_core") {
                CAPTURED
                    .lock()
                    .expect("the capture logger lock was poisoned")
                    .push((record.level(), record.args().to_string()));
            }
        }

        fn flush(&self) {}
    }

    #[test]
    fn known_level_names_map_to_filters() {
        let expected = [
            ("off", LevelFilter::Off),
            ("error", LevelFilter::Error),
            ("warn", LevelFilter::Warn),
            ("info", LevelFilter::Info),
            ("debug", LevelFilter::Debug),
            ("trace", LevelFilter::Trace),
            ("INFO", LevelFilter::Info),
            ("Warn", LevelFilter::Warn),
        ];
        for (name, filter) in expected {
            assert_eq!(
                level_filter_from_name(name),
                Some(filter),
                "{name} did not map to {filter:?}"
            );
        }
    }

    #[test]
    fn unknown_level_names_are_rejected() {
        for name in ["verbose", "", " warn ", "fatal"] {
            assert_eq!(
                level_filter_from_name(name),
                None,
                "{name} should not map to a filter"
            );
        }
    }

    #[test]
    fn the_default_filter_is_info() {
        assert_eq!(default_level_filter(), LevelFilter::Info);
    }

    #[test]
    fn the_startup_record_reaches_the_installed_logger() {
        // The global logger can only be claimed once per test process: the
        // first call wins and later calls fail, which still leaves the
        // capture logger in place for this test.
        let _ = log::set_boxed_logger(Box::new(Capture));
        log::set_max_level(log::LevelFilter::Trace);

        CAPTURED
            .lock()
            .expect("the capture logger lock was poisoned")
            .clear();
        log_app_start("0.1.0-test");

        let captured = CAPTURED
            .lock()
            .expect("the capture logger lock was poisoned");
        assert!(
            captured.iter().any(|(level, message)| {
                *level == log::Level::Info
                    && message.contains("SupportOS Oracle starting")
                    && message.contains("0.1.0-test")
            }),
            "the startup record did not reach the capture logger: {captured:?}"
        );
    }
}
