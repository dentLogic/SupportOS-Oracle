//! The SQLite foundation: open with WAL, run deterministic migrations.
//!
//! Persistence lives in core (docs/SPEC.md section 4.1) behind typed
//! operations; UI code never sees SQL. The database is opened through
//! [`open`], which enables the write-ahead log (verified, not assumed), turns
//! foreign keys on and applies every pending migration exactly once. Migrations
//! are recorded in `schema_migrations` together with the exact SQL that ran,
//! so re-running a modified migration or opening a newer database with older
//! code fails loudly instead of silently diverging (A16: no silent failure).

use std::path::Path;

use rusqlite::{params, Connection};

use crate::error::{Error, Result};

/// A schema migration, applied exactly once in ascending version order.
pub struct Migration {
    /// The migration version; unique and strictly ascending in the list.
    pub version: i64,
    /// A short human-readable name for diagnostics.
    pub name: &'static str,
    /// The SQL the migration runs inside one transaction.
    pub sql: &'static str,
}

/// The ordered migrations applied by [`open`].
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "settings_key_value",
    sql: "CREATE TABLE settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    )",
}];

/// Opens the application database at `path` with WAL, foreign keys and all
/// migrations applied.
pub fn open(path: impl AsRef<Path>) -> Result<Connection> {
    let path = path.as_ref();
    let mut conn = Connection::open(path)
        .map_err(|e| Error::Database(format!("cannot open {}: {e}", path.display())))?;

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
        .map_err(|e| {
            Error::Database(format!("cannot enable WAL on {}: {e}", path.display()))
        })?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(Error::Database(format!(
            "{} refused the WAL journal mode (reported {journal_mode})",
            path.display()
        )));
    }

    conn.execute_batch("PRAGMA foreign_keys=ON")
        .map_err(|e| {
            Error::Database(format!("cannot enable foreign keys on {}: {e}", path.display()))
        })?;

    run_migrations(&mut conn, MIGRATIONS)?;
    Ok(conn)
}

/// Applies `migrations` to `conn` exactly once each, in order.
///
/// Deterministic behavior: a migration whose recorded content differs from
/// the list, a database holding versions this list cannot provide, or a list
/// that is not strictly ascending all fail with a database error.
pub fn run_migrations(conn: &mut Connection, migrations: &[Migration]) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            sql TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        )",
    )
    .map_err(|e| Error::Database(format!("cannot create the migration records: {e}")))?;

    for pair in migrations.windows(2) {
        if pair[0].version >= pair[1].version {
            return Err(Error::Database(format!(
                "the migration list is not strictly ascending: {} before {}",
                pair[0].version, pair[1].version
            )));
        }
    }

    let applied = applied_migrations(conn)?;
    for record in &applied {
        if !migrations.iter().any(|m| m.version == record.version) {
            return Err(Error::Database(format!(
                "the database holds migration {} ({}), which this build cannot provide",
                record.version, record.name
            )));
        }
    }

    for migration in migrations {
        if let Some(record) = applied.iter().find(|r| r.version == migration.version) {
            if record.name != migration.name || record.sql != migration.sql {
                return Err(Error::Database(format!(
                    "migration {} ({}) was applied with different content",
                    migration.version, migration.name
                )));
            }
        } else {
            apply(conn, migration)?;
        }
    }
    Ok(())
}

/// One applied migration as recorded in `schema_migrations`.
struct AppliedMigration {
    version: i64,
    name: String,
    sql: String,
}

fn applied_migrations(conn: &Connection) -> Result<Vec<AppliedMigration>> {
    let mut stmt = conn
        .prepare("SELECT version, name, sql FROM schema_migrations ORDER BY version")
        .map_err(|e| Error::Database(format!("cannot read the migration records: {e}")))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AppliedMigration {
                version: row.get(0)?,
                name: row.get(1)?,
                sql: row.get(2)?,
            })
        })
        .map_err(|e| Error::Database(format!("cannot read the migration records: {e}")))?;

    let mut applied = Vec::new();
    for row in rows {
        let record = row
            .map_err(|e| Error::Database(format!("cannot read a migration record: {e}")))?;
        applied.push(record);
    }
    Ok(applied)
}

fn apply(conn: &mut Connection, migration: &Migration) -> Result<()> {
    let tx = conn
        .transaction()
        .map_err(|e| Error::Database(format!("cannot start a migration transaction: {e}")))?;
    tx.execute_batch(migration.sql).map_err(|e| {
        Error::Database(format!(
            "migration {} ({}) failed: {e}",
            migration.version, migration.name
        ))
    })?;
    tx.execute(
        "INSERT INTO schema_migrations (version, name, sql, applied_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            migration.version,
            migration.name,
            migration.sql,
            unix_epoch_seconds()?
        ],
    )
    .map_err(|e| {
        Error::Database(format!(
            "cannot record migration {} ({}): {e}",
            migration.version, migration.name
        ))
    })?;
    tx.commit().map_err(|e| {
        Error::Database(format!(
            "cannot commit migration {} ({}): {e}",
            migration.version, migration.name
        ))
    })?;
    Ok(())
}

fn unix_epoch_seconds() -> Result<i64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .map_err(|e| Error::Io(format!("the system clock is before 1970: {e}")))
}

#[cfg(test)]
mod tests {
    use super::{open, run_migrations, Migration, MIGRATIONS};
    use rusqlite::Connection;
    use std::path::PathBuf;

    /// A unique temporary database path for one test, with stale leftovers
    /// removed so reruns stay deterministic.
    fn temp_path(tag: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "supportos-oracle-test-{tag}-{}.db",
            std::process::id()
        ));
        let file_name = path
            .file_name()
            .expect("the path has a file name")
            .to_string_lossy();
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(path.with_file_name(format!("{file_name}{suffix}")));
        }
        path
    }

    #[test]
    fn opens_with_wal_foreign_keys_and_migrations() {
        let path = temp_path("open");
        let conn = open(&path).expect("the database should open");

        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("the journal mode should be readable");
        assert_eq!(journal_mode.to_ascii_lowercase(), "wal");

        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("the foreign key mode should be readable");
        assert_eq!(foreign_keys, 1);

        let recorded: i64 = conn
            .query_row("SELECT count(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("the migration records should be countable");
        assert_eq!(recorded, MIGRATIONS.len() as i64);

        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('theme', 'dark')",
            [],
        )
        .expect("the v1 settings table should accept rows");
    }

    #[test]
    fn reopening_applies_nothing_and_keeps_data() {
        let path = temp_path("reopen");
        {
            let conn = open(&path).expect("the first open should succeed");
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('theme', 'dark')",
                [],
            )
            .expect("the first open should accept a settings row");
        }

        let conn = open(&path).expect("the second open should succeed");
        let recorded: i64 = conn
            .query_row("SELECT count(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("the migration records should be countable");
        assert_eq!(recorded, MIGRATIONS.len() as i64);

        let stored: String = conn
            .query_row("SELECT value FROM settings WHERE key = 'theme'", [], |row| {
                row.get(0)
            })
            .expect("the settings row should survive the reopen");
        assert_eq!(stored, "dark");
    }

    #[test]
    fn fts5_is_available_for_lexical_search() {
        let conn = Connection::open_in_memory().expect("an in-memory database should open");
        conn.execute_batch("CREATE VIRTUAL TABLE probe USING fts5(content)")
            .expect("FTS5 should be compiled into the bundled SQLite");
        conn.execute(
            "INSERT INTO probe (rowid, content) VALUES (1, 'persistent local first search')",
            [],
        )
        .expect("the FTS5 probe table should accept rows");
        let hits: i64 = conn
            .query_row("SELECT count(*) FROM probe WHERE probe MATCH 'search'", [], |row| {
                row.get(0)
            })
            .expect("the FTS5 match query should run");
        assert_eq!(hits, 1);
    }

    #[test]
    fn a_modified_applied_migration_is_rejected() {
        let mut conn = Connection::open_in_memory().expect("an in-memory database should open");
        let original = [Migration {
            version: 1,
            name: "one",
            sql: "CREATE TABLE probe_original (x INTEGER)",
        }];
        run_migrations(&mut conn, &original)
            .expect("the original migration should apply");

        let tampered = [Migration {
            version: 1,
            name: "one",
            sql: "CREATE TABLE probe_tampered (x INTEGER)",
        }];
        let error = run_migrations(&mut conn, &tampered)
            .expect_err("a modified applied migration must be rejected");
        assert!(
            error.to_string().contains("different content"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn a_database_newer_than_the_code_is_rejected() {
        let mut conn = Connection::open_in_memory().expect("an in-memory database should open");
        let future = [Migration {
            version: 9,
            name: "future",
            sql: "CREATE TABLE probe_future (x INTEGER)",
        }];
        run_migrations(&mut conn, &future)
            .expect("the future migration should apply");

        let error = run_migrations(&mut conn, MIGRATIONS)
            .expect_err("a newer database must be rejected");
        assert!(
            error.to_string().contains("cannot provide"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn a_non_ascending_migration_list_is_rejected() {
        let mut conn = Connection::open_in_memory().expect("an in-memory database should open");
        let unordered = [
            Migration {
                version: 2,
                name: "two",
                sql: "CREATE TABLE probe_two (x INTEGER)",
            },
            Migration {
                version: 1,
                name: "one",
                sql: "CREATE TABLE probe_one (x INTEGER)",
            },
        ];
        let error = run_migrations(&mut conn, &unordered)
            .expect_err("a non-ascending list must be rejected");
        assert!(
            error.to_string().contains("strictly ascending"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn a_failing_migration_is_rolled_back_completely() {
        let mut conn = Connection::open_in_memory().expect("an in-memory database should open");
        let broken = [Migration {
            version: 1,
            name: "broken",
            sql: "CREATE TABLE probe_broken (x INTEGER); CREATE TABLE probe_broken (y INTEGER)",
        }];
        let error = run_migrations(&mut conn, &broken)
            .expect_err("the broken migration must fail");
        assert!(
            error.to_string().contains("failed"),
            "unexpected error: {error}"
        );

        let recorded: i64 = conn
            .query_row("SELECT count(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("the migration records should stay readable");
        assert_eq!(recorded, 0);
    }
}
