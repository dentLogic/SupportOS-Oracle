//! Pure-Rust core of SupportOS Oracle: domain, persistence, services, jobs and
//! business logic. This crate must not depend on Tauri or GTK so its tests run
//! without Linux GUI system libraries (docs/SPEC-AMENDMENTS.md, A5).

pub mod app;
pub mod db;
pub mod error;
pub mod logging;
