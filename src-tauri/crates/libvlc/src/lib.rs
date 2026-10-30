//! Safe wrappers around libVLC 3.x for Orpheon.
//!
//! # Errors
//! Fallible operations return [`LibVlcError`]. There is no `unwrap` in this crate's
//! public API paths intended for production use.
//!
//! # Threading
//! Video callbacks run on libVLC decoder threads. Share frame state with
//! [`std::sync::Arc`] + [`std::sync::Mutex`] and marshal UI updates via the
//! Slint event loop.

#![deny(missing_docs)]

mod equalizer;
mod error;
mod instance;
mod media;
mod player;
mod video_frame;
mod waveform_sample;

pub use equalizer::{EqGains, Equalizer, BAND_LABELS};
pub use error::LibVlcError;
pub use instance::{HwAccelMode, Instance};
pub use media::{path_for_libvlc, Media};
pub use player::{MediaMetaSnapshot, MediaPlayer};
pub use video_frame::{
    cap_dimensions, soft_blit_cap, FrameBuffer, VideoFrameState, MAX_SOFT_HEIGHT, MAX_SOFT_WIDTH,
};
pub use waveform_sample::sample_peaks;

/// Crate-level result alias.
pub type Result<T> = std::result::Result<T, LibVlcError>;
