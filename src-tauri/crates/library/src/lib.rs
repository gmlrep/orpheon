//! Local media library backed by SQLite.

#![deny(missing_docs)]

mod browse;
mod db;
mod error;
mod models;
mod scan;

pub use browse::{
    breadcrumb, breadcrumb_parts, child_folders, direct_media, root_folders, BreadcrumbPart,
    FolderEntry,
};
pub use db::LibraryDb;
pub use error::LibraryError;
pub use models::{MediaItem, MediaType, Playlist};
pub use scan::{is_audio_extension, is_media_extension, is_video_extension, scan_root};

/// Crate result alias.
pub type Result<T> = std::result::Result<T, LibraryError>;
