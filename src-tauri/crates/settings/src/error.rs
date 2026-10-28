//! Settings errors.

/// Errors from the settings store.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// SQLite failure.
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    /// JSON (de)serialization failure.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// I/O when creating parent dirs.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
