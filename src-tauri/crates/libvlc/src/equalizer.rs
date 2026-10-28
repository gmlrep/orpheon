//! Graphic equalizer (10-band) wired to libVLC.

use crate::{LibVlcError, Result};
use std::ptr;

/// UI sliders are `0.0..=1.0` with `0.5` = flat (0 dB). libVLC uses −20…+20 dB.
#[inline]
fn slider_to_db(slider: f32) -> f32 {
    ((slider.clamp(0.0, 1.0) - 0.5) * 40.0).clamp(-20.0, 20.0)
}

/// Band center frequencies used by the UI (VLC 10-band graphic EQ).
pub const BAND_LABELS: [&str; 10] = [
    "60 Hz", "170 Hz", "310 Hz", "600 Hz", "1 KHz", "3 KHz", "6 KHz", "12 KHz", "14 KHz", "16 KHz",
];

/// Snapshot of equalizer gains from the UI (`0.0..=1.0`, `0.5` = flat).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EqGains {
    /// Whether the equalizer is applied.
    pub enabled: bool,
    /// Preamp slider.
    pub preamp: f32,
    /// Ten band sliders.
    pub bands: [f32; 10],
}

impl EqGains {
    /// Flat / bypassed.
    #[must_use]
    pub const fn flat() -> Self {
        Self {
            enabled: false,
            preamp: 0.5,
            bands: [0.5; 10],
        }
    }
}

/// Owned libVLC equalizer handle.
pub struct Equalizer {
    raw: *mut libvlc_sys::libvlc_equalizer_t,
}

impl Equalizer {
    /// Build from UI gains. Returns `Ok(None)` when disabled (caller clears player EQ).
    ///
    /// # Errors
    /// Returns [`LibVlcError::PlayerCreate`] if libVLC cannot allocate an equalizer.
    pub fn from_gains(gains: &EqGains) -> Result<Option<Self>> {
        if !gains.enabled {
            return Ok(None);
        }
        let raw = unsafe { libvlc_sys::libvlc_audio_equalizer_new() };
        if raw.is_null() {
            return Err(LibVlcError::PlayerCreate);
        }
        let eq = Self { raw };
        unsafe {
            let _ = libvlc_sys::libvlc_audio_equalizer_set_preamp(raw, slider_to_db(gains.preamp));
            for (i, &band) in gains.bands.iter().enumerate() {
                let _ = libvlc_sys::libvlc_audio_equalizer_set_amp_at_index(
                    raw,
                    slider_to_db(band),
                    i as u32,
                );
            }
        }
        Ok(Some(eq))
    }

    pub(crate) fn as_raw(&self) -> *mut libvlc_sys::libvlc_equalizer_t {
        self.raw
    }
}

impl Drop for Equalizer {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { libvlc_sys::libvlc_audio_equalizer_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}
