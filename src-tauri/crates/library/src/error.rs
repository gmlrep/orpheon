//! Library errors.

/// Errors from library persistence and scanning.
#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    /// SQLite failure.
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    /// I/O during scan.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Requested row was not found.
    #[error("not found: {0}")]
    NotFound(String),
}
