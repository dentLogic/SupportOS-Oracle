//! The application error type.
//!
//! Core operations return `Result<T, Error>`; the Tauri shell carries the
//! message across the command boundary so the UI shows a human-readable
//! error instead of internals (docs/SPEC.md section 20, A16: no silent
//! failure, no success before the operation succeeded).

use thiserror::Error as ThisError;

/// Errors surfaced by SupportOS Oracle core operations.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Opening, migrating or querying the local database failed.
    #[error("database error: {0}")]
    Database(String),
    /// Reading, parsing or writing persisted settings failed.
    #[error("settings error: {0}")]
    Settings(String),
    /// Queuing or executing a background job failed.
    #[error("job queue error: {0}")]
    JobQueue(String),
    /// An operation received input it cannot handle.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// A local filesystem operation failed.
    #[error("I/O error: {0}")]
    Io(String),
}

/// Result alias used across core operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::Error;
    use std::error::Error as StdError;

    fn all_variants() -> Vec<Error> {
        vec![
            Error::Database("cannot open the database file".into()),
            Error::Settings("cannot parse the settings store".into()),
            Error::JobQueue("the job could not be queued".into()),
            Error::InvalidInput("the identifier is empty".into()),
            Error::Io("the file is not readable".into()),
        ]
    }

    #[test]
    fn every_variant_renders_a_non_empty_message() {
        for error in all_variants() {
            let message = error.to_string();
            assert!(!message.is_empty(), "variant {error:?} rendered empty");
        }
    }

    #[test]
    fn messages_explain_which_subsystem_failed() {
        let expected = [
            "database error:",
            "settings error:",
            "job queue error:",
            "invalid input:",
            "I/O error:",
        ];
        for (error, prefix) in all_variants().into_iter().zip(expected) {
            assert!(
                error.to_string().starts_with(prefix),
                "{error:?} does not start with {prefix}"
            );
        }
    }

    #[test]
    fn implements_the_standard_error_trait() {
        let error = Error::Database("trait object".into());
        let erased: &dyn StdError = &error;
        assert_eq!(erased.to_string(), "database error: trait object");
    }
}
