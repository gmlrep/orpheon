//! Media resources.

use crate::{Instance, LibVlcError, Result};
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::ptr;

/// Strip Windows verbatim prefixes (`\\?\C:\…`, `\\?\UNC\…`) that libVLC cannot open.
///
/// `Path::canonicalize()` on Windows returns these forms; pass the result through
/// here before `libvlc_media_new_path` / plugin PATH env vars.
#[must_use]
pub fn path_for_libvlc(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{rest}"));
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path.to_path_buf()
}

/// A playable media item (file or network MRL).
pub struct Media {
    pub(crate) raw: *mut libvlc_sys::libvlc_media_t,
}

unsafe impl Send for Media {}

impl Media {
    /// Create media from a filesystem path.
    ///
    /// # Errors
    /// Returns [`LibVlcError::MediaCreate`] or [`LibVlcError::Nul`].
    pub fn from_path(instance: &Instance, path: &Path) -> Result<Self> {
        let path = path_for_libvlc(path);
        let path_str = path.to_string_lossy();
        let c_path = CString::new(path_str.as_ref())?;
        // SAFETY: instance is valid; c_path is NUL-terminated.
        let raw = unsafe { libvlc_sys::libvlc_media_new_path(instance.raw, c_path.as_ptr()) };
        if raw.is_null() {
            return Err(LibVlcError::MediaCreate(path_str.into_owned()));
        }
        Ok(Self { raw })
    }

    /// Create media from an MRL (e.g. `http://…`, `file:///…`).
    ///
    /// # Errors
    /// Returns [`LibVlcError::MediaCreate`] or [`LibVlcError::Nul`].
    pub fn from_mrl(instance: &Instance, mrl: &str) -> Result<Self> {
        let c_mrl = CString::new(mrl)?;
        // SAFETY: instance is valid; c_mrl is NUL-terminated.
        let raw = unsafe { libvlc_sys::libvlc_media_new_location(instance.raw, c_mrl.as_ptr()) };
        if raw.is_null() {
            return Err(LibVlcError::MediaCreate(mrl.to_owned()));
        }
        Ok(Self { raw })
    }

    /// Add a libVLC option string (for example `:sout=#file{dst=…}`).
    ///
    /// # Errors
    /// Returns [`LibVlcError::Nul`] if the option contains interior NULs.
    pub fn add_option(&self, option: &str) -> Result<()> {
        let c_opt = CString::new(option)?;
        // SAFETY: media pointer is owned; option is NUL-terminated.
        unsafe { libvlc_sys::libvlc_media_add_option(self.raw, c_opt.as_ptr()) };
        Ok(())
    }
}

impl Drop for Media {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // SAFETY: we own the media pointer.
            unsafe { libvlc_sys::libvlc_media_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}
