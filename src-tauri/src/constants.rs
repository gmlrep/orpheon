//! Shared magic numbers / paths for the Tauri backend.

use std::time::Duration;

/// Playback status poll interval (~30 Hz).
pub const PLAYBACK_POLL_INTERVAL: Duration = Duration::from_millis(33);

/// Persist resume positions at most this often while playing.
pub const RESUME_PERSIST_INTERVAL: Duration = Duration::from_secs(5);

/// Media shorter than this is ignored for end-of-track detection.
pub const END_MIN_LENGTH_MS: i64 = 1000;

/// Treat as ended when position is within this fraction of the end.
pub const END_POSITION_THRESHOLD: f32 = 0.98;

/// Or when remaining time is within this many milliseconds.
pub const END_TIME_SLACK_MS: i64 = 250;

/// Default softvol (100 = 0 dB).
pub const DEFAULT_VOLUME: i32 = 80;

/// Engine softvol ceiling.
pub const SOFTVOL_ENGINE_MAX: i32 = 200;

/// ffmpeg seek offset (seconds) when grabbing a video frame thumbnail.
pub const ARTWORK_FFMPEG_SEEK_SECS: &str = "5";

/// Minimum non-empty artwork file size (bytes).
pub const ARTWORK_MIN_BYTES: u64 = 32;

/// Files larger than this use a later VLC scene sample point.
pub const ARTWORK_LARGE_FILE_BYTES: u64 = 80_000_000;

pub const ARTWORK_VLC_START_LARGE: &str = "--start-time=90";
pub const ARTWORK_VLC_START_SMALL: &str = "--start-time=1";

/// Fallback Windows VLC install path.
pub const VLC_SYSTEM_DIR: &str = r"C:\Program Files\VideoLAN\VLC";
