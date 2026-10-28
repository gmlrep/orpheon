//! Media player controls and video callback wiring.

use crate::equalizer::{EqGains, Equalizer};
use crate::video_frame::{soft_blit_cap, VideoFrameState};
use crate::{Instance, LibVlcError, Media, Result};
use std::os::raw::{c_char, c_uint, c_void};
use std::ptr;
use std::sync::Arc;

/// A media player bound to an [`Instance`].
pub struct MediaPlayer {
    raw: *mut libvlc_sys::libvlc_media_player_t,
    /// Keeps the frame state alive while callbacks are registered.
    _frame_state: Option<Arc<VideoFrameState>>,
}

unsafe impl Send for MediaPlayer {}

impl MediaPlayer {
    /// Create an empty media player.
    ///
    /// # Errors
    /// Returns [`LibVlcError::PlayerCreate`] if allocation fails.
    pub fn new(instance: &Instance) -> Result<Self> {
        // SAFETY: instance is valid for the duration of the call.
        let raw = unsafe { libvlc_sys::libvlc_media_player_new(instance.raw) };
        if raw.is_null() {
            return Err(LibVlcError::PlayerCreate);
        }
        Ok(Self {
            raw,
            _frame_state: None,
        })
    }

    /// Attach media to this player (replaces any previous media).
    pub fn set_media(&self, media: &Media) {
        // SAFETY: both pointers are owned and valid.
        unsafe { libvlc_sys::libvlc_media_player_set_media(self.raw, media.raw) };
    }

    /// Configure software video output into a double-buffered [`VideoFrameState`].
    ///
    /// Negotiates RGBA via format callbacks so libVLC can renegotiate size
    /// (and downscale 4K/HDR) instead of a fixed `set_format` that breaks converters.
    pub fn set_video_callbacks(&mut self, frame_state: Arc<VideoFrameState>) {
        let opaque = Arc::as_ptr(&frame_state) as *mut VideoFrameState as *mut c_void;
        // SAFETY: callbacks only dereference `opaque` as `VideoFrameState` while
        // `frame_state` is held in `_frame_state` (and thus alive).
        unsafe {
            libvlc_sys::libvlc_video_set_callbacks(
                self.raw,
                Some(lock_cb),
                Some(unlock_cb),
                Some(display_cb),
                opaque,
            );
            libvlc_sys::libvlc_video_set_format_callbacks(
                self.raw,
                Some(format_cb),
                Some(cleanup_cb),
            );
        }
        self._frame_state = Some(frame_state);
    }

    /// Start playback.
    ///
    /// # Errors
    /// Returns [`LibVlcError::PlayFailed`] on failure.
    pub fn play(&self) -> Result<()> {
        // SAFETY: player is valid.
        let rc = unsafe { libvlc_sys::libvlc_media_player_play(self.raw) };
        if rc != 0 {
            return Err(LibVlcError::PlayFailed);
        }
        Ok(())
    }

    /// Pause / resume toggle semantics per libVLC (pause when playing).
    pub fn pause(&self) {
        // SAFETY: player is valid.
        unsafe { libvlc_sys::libvlc_media_player_pause(self.raw) };
    }

    /// Stop playback.
    pub fn stop(&self) {
        // SAFETY: player is valid.
        unsafe { libvlc_sys::libvlc_media_player_stop(self.raw) };
    }

    /// Whether the player is currently playing.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        // SAFETY: player is valid.
        unsafe { libvlc_sys::libvlc_media_player_is_playing(self.raw) != 0 }
    }

    /// Playback position in `0.0..=1.0`.
    #[must_use]
    pub fn position(&self) -> f32 {
        // SAFETY: player is valid.
        unsafe { libvlc_sys::libvlc_media_player_get_position(self.raw) }
    }

    /// Seek to a relative position `0.0..=1.0`.
    pub fn set_position(&self, pos: f32) {
        // SAFETY: player is valid.
        unsafe { libvlc_sys::libvlc_media_player_set_position(self.raw, pos.clamp(0.0, 1.0)) };
    }

    /// Media length in milliseconds (`-1` if unknown).
    #[must_use]
    pub fn length_ms(&self) -> i64 {
        // SAFETY: player is valid.
        unsafe { libvlc_sys::libvlc_media_player_get_length(self.raw) }
    }

    /// Current time in milliseconds.
    #[must_use]
    pub fn time_ms(&self) -> i64 {
        // SAFETY: player is valid.
        unsafe { libvlc_sys::libvlc_media_player_get_time(self.raw) }
    }

    /// Seek to an absolute time in milliseconds.
    pub fn set_time_ms(&self, time: i64) {
        // SAFETY: player is valid.
        let _ = unsafe { libvlc_sys::libvlc_media_player_set_time(self.raw, time.max(0)) };
    }

    /// Audio volume in `0..=200` (`100` = 0 dB, above amplifies).
    #[must_use]
    pub fn volume(&self) -> i32 {
        // SAFETY: player is valid.
        unsafe { libvlc_sys::libvlc_audio_get_volume(self.raw) }
    }

    /// Set audio volume in `0..=200` (`100` = 0 dB, above amplifies).
    pub fn set_volume(&self, volume: i32) {
        // SAFETY: player is valid.
        let _ = unsafe { libvlc_sys::libvlc_audio_set_volume(self.raw, volume.clamp(0, 200)) };
    }

    /// Mute or unmute.
    pub fn set_mute(&self, mute: bool) {
        // SAFETY: player is valid.
        let _ = unsafe { libvlc_sys::libvlc_audio_set_mute(self.raw, i32::from(mute)) };
    }

    /// Number of subtitle tracks (0 if none / unknown yet).
    #[must_use]
    pub fn spu_count(&self) -> i32 {
        unsafe { libvlc_sys::libvlc_video_get_spu_count(self.raw) }.max(0)
    }

    /// Walk a libVLC track-description linked list into `(id, name)` pairs.
    fn collect_track_descriptions(
        head: *mut libvlc_sys::libvlc_track_description_t,
    ) -> Vec<(i32, String)> {
        let mut out = Vec::new();
        if head.is_null() {
            return out;
        }
        let mut cur = head;
        while !cur.is_null() {
            let id = unsafe { (*cur).i_id };
            let name = unsafe {
                if (*cur).psz_name.is_null() {
                    format!("Track {id}")
                } else {
                    std::ffi::CStr::from_ptr((*cur).psz_name)
                        .to_string_lossy()
                        .into_owned()
                }
            };
            out.push((id, name));
            cur = unsafe { (*cur).p_next };
        }
        unsafe { libvlc_sys::libvlc_track_description_list_release(head) };
        out
    }

    /// Video elementary streams (may include a Disable sentinel with id `-1`).
    #[must_use]
    pub fn video_tracks(&self) -> Vec<(i32, String)> {
        let head = unsafe { libvlc_sys::libvlc_video_get_track_description(self.raw) };
        Self::collect_track_descriptions(head)
    }

    /// Current video track id (`-1` if none).
    #[must_use]
    pub fn video_track(&self) -> i32 {
        unsafe { libvlc_sys::libvlc_video_get_track(self.raw) }
    }

    /// Select a video track by id.
    pub fn set_video_track(&self, id: i32) {
        let _ = unsafe { libvlc_sys::libvlc_video_set_track(self.raw, id) };
    }

    /// Audio tracks (may include a Disable sentinel with id `-1`).
    #[must_use]
    pub fn audio_tracks(&self) -> Vec<(i32, String)> {
        let head = unsafe { libvlc_sys::libvlc_audio_get_track_description(self.raw) };
        Self::collect_track_descriptions(head)
    }

    /// Current audio track id (`-1` if none).
    #[must_use]
    pub fn audio_track(&self) -> i32 {
        unsafe { libvlc_sys::libvlc_audio_get_track(self.raw) }
    }

    /// Select an audio track by id.
    pub fn set_audio_track(&self, id: i32) {
        let _ = unsafe { libvlc_sys::libvlc_audio_set_track(self.raw, id) };
    }

    /// Subtitle / SPU tracks (may include a Disable sentinel with id `-1`).
    #[must_use]
    pub fn spu_tracks(&self) -> Vec<(i32, String)> {
        let head = unsafe { libvlc_sys::libvlc_video_get_spu_description(self.raw) };
        Self::collect_track_descriptions(head)
    }

    /// Current SPU track id (`-1` = disabled).
    #[must_use]
    pub fn spu_track(&self) -> i32 {
        unsafe { libvlc_sys::libvlc_video_get_spu(self.raw) }
    }

    /// Select SPU track by id (`-1` disables).
    pub fn set_spu_track(&self, id: i32) {
        let _ = unsafe { libvlc_sys::libvlc_video_set_spu(self.raw, id) };
    }

    /// Enable the first available subtitle track, or disable when `on` is false.
    pub fn set_subtitles_enabled(&self, on: bool) {
        if !on {
            self.set_spu_track(-1);
            return;
        }
        let chosen = self
            .spu_tracks()
            .into_iter()
            .find(|(id, _)| *id >= 0)
            .map(|(id, _)| id)
            .unwrap_or(-1);
        if chosen >= 0 {
            self.set_spu_track(chosen);
        }
    }

    /// Load an external subtitle file (`.srt` / `.ass` / …).
    pub fn set_subtitle_file(&self, path: &std::path::Path) -> bool {
        let Some(s) = path.to_str() else {
            return false;
        };
        let Ok(c) = std::ffi::CString::new(s) else {
            return false;
        };
        unsafe { libvlc_sys::libvlc_video_set_subtitle_file(self.raw, c.as_ptr()) != 0 }
    }

    /// Chapters for the current title: `(index, label)`.
    #[must_use]
    pub fn chapters(&self) -> Vec<(i32, String)> {
        let mut ptr: *mut *mut libvlc_sys::libvlc_chapter_description_t = std::ptr::null_mut();
        let count = unsafe {
            libvlc_sys::libvlc_media_player_get_full_chapter_descriptions(self.raw, -1, &mut ptr)
        };
        if count <= 0 || ptr.is_null() {
            // Fallback: numbered chapters without names.
            let n = unsafe { libvlc_sys::libvlc_media_player_get_chapter_count(self.raw) };
            if n <= 0 {
                return Vec::new();
            }
            return (0..n).map(|i| (i, format!("Chapter {}", i + 1))).collect();
        }
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count {
            let desc = unsafe { *ptr.add(i as usize) };
            if desc.is_null() {
                out.push((i, format!("Chapter {}", i + 1)));
                continue;
            }
            let name = unsafe {
                if (*desc).psz_name.is_null() {
                    format!("Chapter {}", i + 1)
                } else {
                    let s = std::ffi::CStr::from_ptr((*desc).psz_name)
                        .to_string_lossy()
                        .into_owned();
                    if s.is_empty() {
                        format!("Chapter {}", i + 1)
                    } else {
                        s
                    }
                }
            };
            out.push((i, name));
        }
        unsafe {
            libvlc_sys::libvlc_chapter_descriptions_release(ptr, count as u32);
        }
        out
    }

    /// Current chapter index (`-1` if none).
    #[must_use]
    pub fn chapter(&self) -> i32 {
        unsafe { libvlc_sys::libvlc_media_player_get_chapter(self.raw) }
    }

    /// Jump to chapter index.
    pub fn set_chapter(&self, index: i32) {
        unsafe { libvlc_sys::libvlc_media_player_set_chapter(self.raw, index) };
    }

    /// Current audio output device id (empty = system default).
    #[must_use]
    pub fn audio_output_device(&self) -> String {
        let ptr = unsafe { libvlc_sys::libvlc_audio_output_device_get(self.raw) };
        if ptr.is_null() {
            return String::new();
        }
        let s = unsafe { std::ffi::CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        unsafe { libvlc_sys::libvlc_free(ptr.cast()) };
        s
    }

    /// Select an audio output device.
    ///
    /// `id` is `""` for system default, or `"module::device"` as produced by
    /// [`crate::Instance::audio_output_devices`].
    pub fn set_audio_output_device(&self, id: &str) {
        if id.is_empty() {
            let _ = unsafe {
                libvlc_sys::libvlc_audio_output_device_set(
                    self.raw,
                    std::ptr::null(),
                    std::ptr::null(),
                )
            };
            return;
        }
        let (module, device) = id.split_once("::").unwrap_or(("", id));
        let Ok(mod_c) = std::ffi::CString::new(module) else {
            return;
        };
        if !module.is_empty() {
            let _ = unsafe { libvlc_sys::libvlc_audio_output_set(self.raw, mod_c.as_ptr()) };
        }
        let Ok(dev_c) = std::ffi::CString::new(device) else {
            return;
        };
        let _ = unsafe {
            libvlc_sys::libvlc_audio_output_device_set(
                self.raw,
                if module.is_empty() {
                    std::ptr::null()
                } else {
                    mod_c.as_ptr()
                },
                if device.is_empty() {
                    std::ptr::null()
                } else {
                    dev_c.as_ptr()
                },
            )
        };
    }

    /// Raw libVLC player pointer (for advanced module wiring).
    #[must_use]
    pub fn as_raw(&self) -> *mut libvlc_sys::libvlc_media_player_t {
        self.raw
    }

    /// Embed / detach video into a native window handle (Win32 `HWND` as `*mut c_void`).
    ///
    /// When set, libVLC uses its hardware Direct3D/OpenGL output instead of
    /// software callbacks — required for smooth native-resolution playback.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn set_hwnd(&self, hwnd: *mut std::os::raw::c_void) {
        unsafe {
            libvlc_sys::libvlc_media_player_set_hwnd(self.raw, hwnd);
            // Let the host UI receive keys; we handle mouse in our child WndProc.
            libvlc_sys::libvlc_video_set_key_input(self.raw, 0);
            libvlc_sys::libvlc_video_set_mouse_input(self.raw, 0);
        }
    }

    /// Capture the current video frame to `path` (JPEG/PNG by extension).
    ///
    /// `width`/`height` of 0 keep the original size. Returns `true` on success.
    pub fn take_snapshot(&self, path: &std::path::Path, width: u32, height: u32) -> bool {
        let Some(path_s) = path.to_str() else {
            return false;
        };
        let Ok(c_path) = std::ffi::CString::new(path_s) else {
            return false;
        };
        let rc = unsafe {
            libvlc_sys::libvlc_video_take_snapshot(self.raw, 0, c_path.as_ptr(), width, height)
        };
        rc == 0
    }

    /// Playback rate (`1.0` = normal).
    pub fn set_rate(&self, rate: f32) {
        let _ = unsafe { libvlc_sys::libvlc_media_player_set_rate(self.raw, rate) };
    }

    /// Current playback rate.
    #[must_use]
    pub fn rate(&self) -> f32 {
        unsafe { libvlc_sys::libvlc_media_player_get_rate(self.raw) }
    }

    /// Audio delay in seconds (positive = audio later).
    pub fn set_audio_delay_secs(&self, secs: f32) {
        let us = (secs as f64 * 1_000_000.0).round() as i64;
        let _ = unsafe { libvlc_sys::libvlc_audio_set_delay(self.raw, us) };
    }

    /// Subtitle delay in seconds (positive = subtitles later).
    pub fn set_spu_delay_secs(&self, secs: f32) {
        let us = (secs as f64 * 1_000_000.0).round() as i64;
        let _ = unsafe { libvlc_sys::libvlc_video_set_spu_delay(self.raw, us) };
    }

    /// Enable the video adjust filter and set hue / brightness / contrast / saturation / gamma.
    ///
    /// Sliders are `0..=1` with midpoint `0.5` = neutral:
    /// - hue → `-180..=180`
    /// - brightness / contrast / saturation / gamma → `0..=2` (1.0 neutral)
    pub fn set_video_adjust_slider(&self, which: &str, slider: f32) {
        let v = slider.clamp(0.0, 1.0);
        let (option, value) = match which {
            "hue" => (3u32, (v - 0.5) * 360.0),
            "brightness" => (2u32, v * 2.0),
            "contrast" => (1u32, v * 2.0),
            "saturation" => (4u32, v * 2.0),
            "gamma" => (5u32, v * 2.0),
            _ => return,
        };
        unsafe {
            // libvlc_adjust_Enable = 0
            libvlc_sys::libvlc_video_set_adjust_int(self.raw, 0, 1);
            libvlc_sys::libvlc_video_set_adjust_float(self.raw, option, value);
        }
    }

    /// Aspect ratio (`""` = automatic).
    pub fn set_aspect_ratio(&self, aspect: &str) {
        if aspect.is_empty() || aspect.eq_ignore_ascii_case("Automatic") {
            unsafe { libvlc_sys::libvlc_video_set_aspect_ratio(self.raw, std::ptr::null()) };
            return;
        }
        let Ok(c) = std::ffi::CString::new(aspect) else {
            return;
        };
        unsafe { libvlc_sys::libvlc_video_set_aspect_ratio(self.raw, c.as_ptr()) };
    }

    /// Crop geometry (`""` / `"None"` = none).
    pub fn set_crop_geometry(&self, crop: &str) {
        if crop.is_empty() || crop.eq_ignore_ascii_case("None") {
            unsafe { libvlc_sys::libvlc_video_set_crop_geometry(self.raw, std::ptr::null()) };
            return;
        }
        let Ok(c) = std::ffi::CString::new(crop) else {
            return;
        };
        unsafe { libvlc_sys::libvlc_video_set_crop_geometry(self.raw, c.as_ptr()) };
    }

    /// Apply or clear the graphic equalizer from UI gains.
    ///
    /// # Errors
    /// Returns an error if libVLC rejects the equalizer.
    pub fn apply_equalizer(&self, gains: &EqGains) -> Result<()> {
        match Equalizer::from_gains(gains)? {
            None => {
                let rc = unsafe {
                    libvlc_sys::libvlc_media_player_set_equalizer(self.raw, ptr::null_mut())
                };
                if rc != 0 {
                    return Err(LibVlcError::PlayFailed);
                }
            }
            Some(eq) => {
                let rc =
                    unsafe { libvlc_sys::libvlc_media_player_set_equalizer(self.raw, eq.as_raw()) };
                // libVLC copies band values; we can drop `eq` after set.
                drop(eq);
                if rc != 0 {
                    return Err(LibVlcError::PlayFailed);
                }
            }
        }
        Ok(())
    }
}

impl Drop for MediaPlayer {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }
        // Stop and clear callbacks before release so worker threads cannot
        // invoke into freed `VideoFrameState` / audio userdata.
        // SAFETY: we own the player for the duration of drop.
        unsafe {
            libvlc_sys::libvlc_media_player_stop(self.raw);
            if self._frame_state.is_some() {
                libvlc_sys::libvlc_video_set_callbacks(self.raw, None, None, None, ptr::null_mut());
                libvlc_sys::libvlc_video_set_format_callbacks(self.raw, None, None);
            }
            libvlc_sys::libvlc_audio_set_callbacks(
                self.raw,
                None,
                None,
                None,
                None,
                None,
                ptr::null_mut(),
            );
            libvlc_sys::libvlc_media_player_release(self.raw);
        }
        self.raw = ptr::null_mut();
        self._frame_state = None;
    }
}

unsafe extern "C" fn lock_cb(opaque: *mut c_void, planes: *mut *mut c_void) -> *mut c_void {
    // SAFETY: opaque is Arc<VideoFrameState> kept alive by MediaPlayer.
    unsafe {
        let state = &*(opaque as *const VideoFrameState);
        let ptr = state.lock_write_plane();
        if !planes.is_null() {
            *planes = ptr.cast();
        }
        ptr.cast()
    }
}

unsafe extern "C" fn unlock_cb(
    opaque: *mut c_void,
    _picture: *mut c_void,
    _planes: *const *mut c_void,
) {
    // SAFETY: opaque is Arc<VideoFrameState> kept alive by MediaPlayer.
    unsafe {
        let state = &*(opaque as *const VideoFrameState);
        state.unlock_write_plane();
    }
}

unsafe extern "C" fn display_cb(opaque: *mut c_void, _picture: *mut c_void) {
    // SAFETY: opaque is Arc<VideoFrameState> kept alive by MediaPlayer.
    unsafe {
        let state = &*(opaque as *const VideoFrameState);
        state.swap_buffers();
    }
}

/// Negotiate RGBA + capped size so 4K/HDR content converts cleanly into our buffer.
unsafe extern "C" fn format_cb(
    opaque: *mut *mut c_void,
    chroma: *mut c_char,
    width: *mut c_uint,
    height: *mut c_uint,
    pitches: *mut c_uint,
    lines: *mut c_uint,
) -> i32 {
    // SAFETY: VLC passes pointer to the opaque from set_callbacks.
    unsafe {
        if opaque.is_null()
            || (*opaque).is_null()
            || chroma.is_null()
            || width.is_null()
            || height.is_null()
        {
            return 0;
        }
        let state = &*((*opaque) as *const VideoFrameState);

        // RGBA matches Slint — avoids a BGRA↔RGBA pass on the UI thread.
        let rgba = *b"RGBA";
        ptr::copy_nonoverlapping(rgba.as_ptr(), chroma.cast::<u8>(), 4);

        // Keep native stream resolution (Slint `image-fit: contain` scales to the widget).
        let src_w = (*width).max(1);
        let src_h = (*height).max(1);
        let (w, h) = soft_blit_cap(src_w, src_h);
        *width = w;
        *height = h;

        state.resize(w, h);
        let (pitch, nlines) = state.pitch_lines();

        // pitches/lines are arrays (one entry per plane); RGBA uses plane 0.
        if !pitches.is_null() {
            *pitches = pitch;
        }
        if !lines.is_null() {
            *lines = nlines;
        }
        // Single pool slot: our lock always returns the current write plane.
        // (Returning >1 without a real picture pool corrupts frames under load.)
        1
    }
}

unsafe extern "C" fn cleanup_cb(opaque: *mut c_void) {
    // SAFETY: opaque is Arc<VideoFrameState> kept alive by MediaPlayer.
    unsafe {
        let state = &*(opaque as *const VideoFrameState);
        state.on_format_cleanup();
    }
}
