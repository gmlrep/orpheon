//! Shared time helpers.

use std::time::{SystemTime, UNIX_EPOCH};

/// Current Unix epoch seconds (0 if the clock is before the epoch).
#[must_use]
pub fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
