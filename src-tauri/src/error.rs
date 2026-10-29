//! IPC-facing command errors (Serialize for the Tauri boundary).

use serde::Serialize;

/// Error returned from `#[tauri::command]` handlers.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// Human-readable failure message.
    #[error("{0}")]
    Message(String),
    /// Mutex poisoned.
    #[error("{0} lock poisoned")]
    Lock(&'static str),
    /// Async task join failure.
    #[error("background task failed: {0}")]
    Join(String),
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl CommandError {
    pub fn msg(s: impl Into<String>) -> Self {
        Self::Message(s.into())
    }
}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for CommandError {
    fn from(value: &str) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<library::LibraryError> for CommandError {
    fn from(value: library::LibraryError) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<settings::SettingsError> for CommandError {
    fn from(value: settings::SettingsError) -> Self {
        Self::Message(value.to_string())
    }
}

#[cfg(feature = "vlc")]
impl From<libvlc::LibVlcError> for CommandError {
    fn from(value: libvlc::LibVlcError) -> Self {
        Self::Message(value.to_string())
    }
}
