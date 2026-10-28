//! Application settings store (SQLite key/value + typed sections).

#![deny(missing_docs)]

mod error;
mod prefs;
mod store;
mod typed;

pub use error::SettingsError;
pub use prefs::{cycle_pref_str, pref_options, Prefs};
pub use store::{AppSettings, SettingsStore, ThemePreference};
pub use typed::{ContinueMode, DeinterlaceMode, HwAccelModePref, ResizeToVideo};

/// Crate result alias.
pub type Result<T> = std::result::Result<T, SettingsError>;
