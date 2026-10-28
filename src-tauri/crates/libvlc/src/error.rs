//! Typed errors for the safe libVLC layer.

use std::ffi::NulError;

/// Errors produced by the safe libVLC wrappers.
#[derive(Debug, thiserror::Error)]
pub enum LibVlcError {
    /// libVLC failed to create an instance (missing DLL / plugins).
    #[error("failed to create libVLC instance (is libvlc installed and on PATH?)")]
    InstanceCreate,
    /// Media could not be created from the given path or MRL.
    #[error("failed to create media from '{0}'")]
    MediaCreate(String),
    /// Network MRL rejected by the application allowlist.
    #[error("{0}")]
    InvalidMrl(String),
    /// Media player could not be created.
    #[error("failed to create media player")]
    PlayerCreate,
    /// Playback could not be started.
    #[error("failed to start playback")]
    PlayFailed,
    /// Path or MRL contained an interior NUL.
    #[error("invalid C string: {0}")]
    Nul(#[from] NulError),
    /// Feature `link` is disabled — stubs only.
    #[error("libVLC linking is disabled (build with --features vlc / link)")]
    LinkingDisabled,
}
