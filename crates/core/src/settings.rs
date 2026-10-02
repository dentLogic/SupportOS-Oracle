//! The settings store: typed access to the `settings` table.
//!
//! Settings are the local application state of SPEC 12 (connection,
//! providers, preferences, theme, demo mode). The store keeps every value as
//! text under a dotted key and validates at the typed boundary: generic
//! [`get`]/[`set`] carry raw strings for future subsystems, while named
//! accessors like [`log_level`] and [`theme`] reject values their subsystem
//! cannot use (A16: no silent failure, no invalid state stored).
//!
//! Secrets never pass through this module's logging surface; values are
//! stored as given and only ever returned to the caller that asked for them
//! (SPEC 12: secrets must not be rendered into UI logs or diagnostics).

use rusqlite::{Connection, params};

use crate::error::{Error, Result};
use crate::logging;

/// The key holding the configured log level name.
pub const LOG_LEVEL_KEY: &str = "log.level";

/// The key holding the persisted theme preference.
pub const THEME_KEY: &str = "app.theme";

/// The theme names the shell understands: follow the system color-scheme
/// preference, or pin the light or dark palette.
pub const THEME_NAMES: &[&str] = &["system", "light", "dark"];

/// Reads the value stored under `key`, or `None` when unset.
pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    validate_key(key)?;
    match conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    ) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(Error::Settings(format!(
            "cannot read the setting {key}: {e}"
        ))),
    }
}

/// Stores `value` under `key`, replacing any previous value.
pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    validate_key(key)?;
    let changed = conn
        .execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| Error::Settings(format!("cannot write the setting {key}: {e}")))?;
    if changed != 1 {
        return Err(Error::Settings(format!(
            "writing the setting {key} changed {changed} rows"
        )));
    }
    Ok(())
}

/// Removes the value under `key`; removing an absent key is fine.
pub fn remove(conn: &Connection, key: &str) -> Result<()> {
    validate_key(key)?;
    conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
        .map_err(|e| Error::Settings(format!("cannot remove the setting {key}: {e}")))?;
    Ok(())
}

/// Lists every stored setting as `(key, value)`, ordered by key.
pub fn list(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings ORDER BY key")
        .map_err(|e| Error::Settings(format!("cannot list the settings: {e}")))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| Error::Settings(format!("cannot list the settings: {e}")))?;

    let mut entries = Vec::new();
    for row in rows {
        let entry = row.map_err(|e| Error::Settings(format!("cannot read a setting row: {e}")))?;
        entries.push(entry);
    }
    Ok(entries)
}

/// Reads the configured log level name, validated against the levels the
/// `log` facade knows.
///
/// `None` means the level is unset and the caller should apply the logging
/// module's default explicitly. A stored name that no longer maps to a level
/// (for example after manual database edits) is an error, not a fallback.
pub fn log_level(conn: &Connection) -> Result<Option<String>> {
    match get(conn, LOG_LEVEL_KEY)? {
        Some(name) => {
            if logging::level_filter_from_name(&name).is_none() {
                return Err(Error::Settings(format!(
                    "the stored log level {name} is not a level name"
                )));
            }
            Ok(Some(name))
        }
        None => Ok(None),
    }
}

/// Stores the log level name after validating it maps to a level filter.
pub fn set_log_level(conn: &Connection, name: &str) -> Result<()> {
    if logging::level_filter_from_name(name).is_none() {
        return Err(Error::InvalidInput(format!(
            "{name} is not a log level name"
        )));
    }
    set(conn, LOG_LEVEL_KEY, name)
}

/// Reads the persisted theme name, validated against [`THEME_NAMES`].
///
/// `None` means the theme is unset and the caller should follow the system
/// color-scheme preference. A stored name outside [`THEME_NAMES`] (for
/// example after manual database edits) is an error, not a fallback.
pub fn theme(conn: &Connection) -> Result<Option<String>> {
    match get(conn, THEME_KEY)? {
        Some(name) => {
            if !THEME_NAMES.contains(&name.as_str()) {
                return Err(Error::Settings(format!(
                    "the stored theme {name} is not a theme name"
                )));
            }
            Ok(Some(name))
        }
        None => Ok(None),
    }
}

/// Stores the theme name after validating it is one of [`THEME_NAMES`].
pub fn set_theme(conn: &Connection, name: &str) -> Result<()> {
    if !THEME_NAMES.contains(&name) {
        return Err(Error::InvalidInput(format!("{name} is not a theme name")));
    }
    set(conn, THEME_KEY, name)
}

fn validate_key(key: &str) -> Result<()> {
    if key.trim().is_empty() {
        return Err(Error::InvalidInput("the setting key is empty".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::db;
    use crate::error::Error;

    use super::{
        LOG_LEVEL_KEY, THEME_KEY, get, list, log_level, remove, set, set_log_level, set_theme,
        theme,
    };

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().expect("an in-memory database should open");
        db::run_migrations(&mut conn, db::MIGRATIONS)
            .expect("the migrations should apply to the test database");
        conn
    }

    #[test]
    fn set_then_get_roundtrips_and_replaces() {
        let conn = test_conn();
        set(&conn, "preference.window", "compact").expect("the first set should succeed");
        assert_eq!(
            get(&conn, "preference.window").expect("the first get should succeed"),
            Some("compact".to_string())
        );

        set(&conn, "preference.window", "wide").expect("the second set should succeed");
        assert_eq!(
            get(&conn, "preference.window").expect("the second get should succeed"),
            Some("wide".to_string())
        );
    }

    #[test]
    fn getting_a_missing_key_returns_none() {
        let conn = test_conn();
        assert_eq!(
            get(&conn, "preference.absent").expect("the get should succeed"),
            None
        );
    }

    #[test]
    fn remove_deletes_and_stays_idempotent() {
        let conn = test_conn();
        set(&conn, "preference.window", "compact").expect("the set should succeed");
        remove(&conn, "preference.window").expect("the first remove should succeed");
        assert_eq!(
            get(&conn, "preference.window").expect("the get should succeed"),
            None
        );
        remove(&conn, "preference.window").expect("the second remove should succeed");
    }

    #[test]
    fn list_is_ordered_by_key() {
        let conn = test_conn();
        set(&conn, "preference.theme", "dark").expect("the first set should succeed");
        set(&conn, "preference.demo", "off").expect("the second set should succeed");
        set(&conn, "ai.provider", "local").expect("the third set should succeed");

        let listed = list(&conn).expect("the list should succeed");
        let keys: Vec<&str> = listed.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["ai.provider", "preference.demo", "preference.theme"]);
    }

    #[test]
    fn empty_keys_are_rejected() {
        let conn = test_conn();
        for key in ["", "   "] {
            match get(&conn, key) {
                Err(Error::InvalidInput(message)) => {
                    assert!(message.contains("key is empty"), "unexpected: {message}");
                }
                other => panic!("an empty key must be rejected, got {other:?}"),
            }
            match set(&conn, key, "value") {
                Err(Error::InvalidInput(message)) => {
                    assert!(message.contains("key is empty"), "unexpected: {message}");
                }
                other => panic!("an empty key must be rejected, got {other:?}"),
            }
        }
        match remove(&conn, "") {
            Err(Error::InvalidInput(_)) => {}
            other => panic!("an empty key must be rejected, got {other:?}"),
        }
    }

    #[test]
    fn the_log_level_roundtrips() {
        let conn = test_conn();
        set_log_level(&conn, "debug").expect("a valid level name should be stored");
        assert_eq!(
            log_level(&conn).expect("the stored level should read back"),
            Some("debug".to_string())
        );

        set_log_level(&conn, "TRACE").expect("case-insensitive level names should be stored");
        assert_eq!(
            log_level(&conn).expect("the stored level should read back"),
            Some("TRACE".to_string())
        );
    }

    #[test]
    fn set_theme_roundtrips_and_replaces() {
        let conn = test_conn();
        set_theme(&conn, "dark").expect("a valid theme should store");
        set_theme(&conn, "light").expect("another valid theme should replace it");
        assert_eq!(
            theme(&conn).expect("the stored theme should read back"),
            Some("light".to_string())
        );
        assert_eq!(
            get(&conn, THEME_KEY).expect("the value should sit under the theme key"),
            Some("light".to_string())
        );
    }

    #[test]
    fn an_unknown_theme_name_is_rejected_on_write() {
        let conn = test_conn();
        match set_theme(&conn, "sepia") {
            Err(Error::InvalidInput(message)) => {
                assert!(message.contains("sepia"), "unexpected: {message}");
            }
            other => panic!("an unknown theme name must be rejected, got {other:?}"),
        }
        assert_eq!(
            get(&conn, THEME_KEY).expect("the rejected write must not store anything"),
            None
        );
    }

    #[test]
    fn a_tampered_theme_is_rejected_on_read() {
        let conn = test_conn();
        set_theme(&conn, "dark").expect("a valid theme should store");
        conn.execute(
            "UPDATE settings SET value = 'sepia' WHERE key = ?1",
            [&THEME_KEY],
        )
        .expect("the tampering should run");
        match theme(&conn) {
            Err(Error::Settings(message)) => {
                assert!(message.contains("sepia"), "unexpected: {message}");
            }
            other => panic!("a tampered theme must be rejected, got {other:?}"),
        }
    }

    #[test]
    fn an_unknown_log_level_name_is_rejected_on_write() {
        let conn = test_conn();
        match set_log_level(&conn, "verbose") {
            Err(Error::InvalidInput(message)) => {
                assert!(message.contains("verbose"), "unexpected: {message}");
            }
            other => panic!("an unknown level name must be rejected, got {other:?}"),
        }
        assert_eq!(
            get(&conn, LOG_LEVEL_KEY).expect("the rejected write must not store anything"),
            None
        );
    }

    #[test]
    fn a_tampered_log_level_is_rejected_on_read() {
        let conn = test_conn();
        set_log_level(&conn, "warn").expect("a valid level name should be stored");
        conn.execute(
            "UPDATE settings SET value = 'verbose' WHERE key = ?1",
            [&LOG_LEVEL_KEY],
        )
        .expect("the tampering should run");
        match log_level(&conn) {
            Err(Error::Settings(message)) => {
                assert!(message.contains("verbose"), "unexpected: {message}");
            }
            other => panic!("a tampered level must be rejected, got {other:?}"),
        }
    }
}
