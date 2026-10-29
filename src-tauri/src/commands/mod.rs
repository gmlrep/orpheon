//! Tauri command surface (replaces Slint `Logic` wire).

mod artwork;
mod library;
pub mod playback;
mod settings;
pub(crate) mod streams;
mod window;

pub use artwork::*;
pub use library::*;
pub use playback::*;
pub use settings::*;
pub use streams::*;
pub use window::*;
