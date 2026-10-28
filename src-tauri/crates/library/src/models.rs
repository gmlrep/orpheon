//! Domain models for the media library.

use serde::{Deserialize, Serialize};

/// High-level media kind for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// Video file.
    Video,
    /// Audio file.
    Audio,
    /// Unknown / other.
    Unknown,
}

impl MediaType {
    /// Database / wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Unknown => "unknown",
        }
    }

    /// Parse from a string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s {
            "video" => Self::Video,
            "audio" => Self::Audio,
            _ => Self::Unknown,
        }
    }
}

/// A scanned media file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    /// Database id.
    pub id: i64,
    /// Absolute path or URI.
    pub path: String,
    /// Display title.
    pub title: String,
    /// Duration in milliseconds, if known.
    pub duration_ms: Option<i64>,
    /// Media kind.
    pub media_type: MediaType,
    /// Filesystem mtime (unix seconds).
    pub mtime: Option<i64>,
    /// Optional artwork path.
    pub artwork_path: Option<String>,
}

/// A named playlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    /// Database id.
    pub id: i64,
    /// Display name.
    pub name: String,
}
