//! High-level playback service wrapping libVLC for the UI thread.

#![allow(dead_code)] // Full API retained for upcoming wire commands / software fallback.

use libvlc::{
    EqGains, HwAccelMode, Instance, LibVlcError, Media, MediaPlayer, Result, VideoFrameState,
};
#[cfg(windows)]
use std::os::raw::c_void;
use std::path::Path;
use std::sync::Arc;

/// Snapshot of elementary streams / chapters for the track panels.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackSnapshot {
    /// Video tracks `(id, label)` — Disable (`-1`) filtered out.
    pub video: Vec<(i32, String)>,
    /// Audio tracks `(id, label)`.
    pub audio: Vec<(i32, String)>,
    /// Subtitle tracks `(id, label)`.
    pub spu: Vec<(i32, String)>,
    /// Chapters `(index, label)`.
    pub chapters: Vec<(i32, String)>,
    /// Audio output devices `(device_id, label)`.
    pub audio_devices: Vec<(String, String)>,
    /// Current video track id.
    pub selected_video: i32,
    /// Current audio track id.
    pub selected_audio: i32,
    /// Current SPU id (`-1` = off).
    pub selected_spu: i32,
    /// Current chapter index.
    pub selected_chapter: i32,
    /// Current audio output device id (empty = default).
    pub selected_audio_device: String,
}

/// How video frames reach the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoOutput {
    /// Windows: waiting for HWND bind after first frame tick.
    #[cfg(windows)]
    PendingHwnd,
    /// Native HWND / Direct3D path (no CPU frame copies).
    #[cfg(windows)]
    Hwnd,
    /// Software RGBA callbacks (fallback when HWND is unavailable).
    Software,
}

/// Owns libVLC instance + player and exposes UI-friendly controls.
pub struct PlayerService {
    instance: Instance,
    player: MediaPlayer,
    frames: Arc<VideoFrameState>,
    /// Reused RGBA upload buffer (software fallback path).
    upload: Vec<u8>,
    /// Active video output path.
    video_output: VideoOutput,
}

impl PlayerService {
    /// Create engine + player with default options (`hw_accel` on).
    ///
    /// # Errors
    /// Returns a libVLC error if the engine cannot be initialized.
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        Self::with_options(HwAccelMode::Automatic, &[])
    }

    /// Create engine + player.
    ///
    /// On Windows we prefer HWND embedding (set later via [`Self::set_hwnd`]) and
    /// skip software callbacks so libVLC can use Direct3D at native resolution.
    ///
    /// # Errors
    /// Returns a libVLC error if the engine cannot be initialized.
    pub fn with_options(hw: HwAccelMode, extra_vlc_options: &[String]) -> Result<Self> {
        let instance = Instance::for_playback(hw, extra_vlc_options)?;
        let mut player = MediaPlayer::new(&instance)?;
        let frames = Arc::new(VideoFrameState::new(64, 64));
        #[cfg(not(windows))]
        {
            player.set_video_callbacks(frames.clone());
        }
        #[cfg(windows)]
        {
            // Callbacks stay off until HWND fails — then enable_software_fallback().
            let _ = &mut player;
        }
        Ok(Self {
            instance,
            player,
            frames,
            upload: Vec::new(),
            #[cfg(windows)]
            video_output: VideoOutput::PendingHwnd,
            #[cfg(not(windows))]
            video_output: VideoOutput::Software,
        })
    }

    /// Bind video output to a native window (Win32 HWND). Enables hardware path.
    #[cfg(windows)]
    pub fn set_hwnd(&mut self, hwnd: *mut c_void) {
        self.player.set_hwnd(hwnd);
        self.video_output = if hwnd.is_null() {
            VideoOutput::Software
        } else {
            VideoOutput::Hwnd
        };
    }

    /// Fall back to software RGBA callbacks if HWND embedding is unavailable.
    #[cfg(windows)]
    pub fn enable_software_fallback(&mut self) {
        self.player.set_hwnd(std::ptr::null_mut());
        self.player.set_video_callbacks(self.frames.clone());
        self.video_output = VideoOutput::Software;
    }

    /// Whether frames are drawn by libVLC into an HWND (no CPU frame upload).
    #[cfg(windows)]
    #[must_use]
    pub fn hwnd_mode(&self) -> bool {
        matches!(self.video_output, VideoOutput::Hwnd)
    }

    /// Open a local file and start playback.
    ///
    /// # Errors
    /// Propagates media/play errors.
    pub fn open_path(&mut self, path: &Path) -> Result<()> {
        self.open_path_with_options(path, &[])
    }

    /// Open a local file with extra libVLC media options (e.g. recording `:sout=`).
    ///
    /// # Errors
    /// Propagates media/play errors.
    pub fn open_path_with_options(&mut self, path: &Path, options: &[&str]) -> Result<()> {
        let media = Media::from_path(&self.instance, path)?;
        for opt in options {
            media.add_option(opt)?;
        }
        self.player.set_media(&media);
        self.player.play()?;
        Ok(())
    }

    /// Open an MRL (network stream, disc, or capture device).
    ///
    /// # Errors
    /// Rejects disallowed schemes / empty MRLs; propagates media/play errors.
    pub fn open_mrl(&mut self, mrl: &str) -> Result<()> {
        self.open_mrl_with_options(mrl, &[])
    }

    /// Open an MRL with extra libVLC media options.
    ///
    /// # Errors
    /// Rejects disallowed schemes / empty MRLs; propagates media/play errors.
    pub fn open_mrl_with_options(&mut self, mrl: &str, options: &[&str]) -> Result<()> {
        let mrl = validate_network_mrl(mrl)?;
        let media = Media::from_mrl(&self.instance, mrl)?;
        for opt in options {
            media.add_option(opt)?;
        }
        self.player.set_media(&media);
        self.player.play()?;
        Ok(())
    }

    /// Start or resume playback.
    ///
    /// # Errors
    /// Returns play errors when starting.
    pub fn play(&mut self) -> Result<()> {
        self.player.play()
    }

    /// Toggle play/pause.
    ///
    /// # Errors
    /// Returns play errors when starting.
    pub fn play_pause(&mut self) -> Result<()> {
        if self.player.is_playing() {
            self.player.pause();
            Ok(())
        } else {
            self.player.play()
        }
    }

    /// Stop playback.
    pub fn stop(&self) {
        self.player.stop();
    }

    /// Whether currently playing.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.player.is_playing()
    }

    /// Metadata for the current media (title / now-playing / artwork).
    #[must_use]
    pub fn media_meta(&self) -> libvlc::MediaMetaSnapshot {
        self.player.media_meta()
    }

    /// Relative position 0..=1.
    #[must_use]
    pub fn position(&self) -> f32 {
        self.player.position()
    }

    /// Seek relative 0..=1.
    pub fn seek(&self, pos: f32) {
        self.player.set_position(pos);
    }

    /// Length in ms.
    #[must_use]
    pub fn length_ms(&self) -> i64 {
        self.player.length_ms()
    }

    /// Current time in ms.
    #[must_use]
    pub fn time_ms(&self) -> i64 {
        self.player.time_ms()
    }

    /// Set volume 0..=200 (`100` = 0 dB).
    pub fn set_volume(&self, volume: i32) {
        self.player.set_volume(volume);
    }

    /// Mute / unmute.
    pub fn set_mute(&self, mute: bool) {
        self.player.set_mute(mute);
    }

    /// Enable/disable burned-in subtitles (first SPU track).
    pub fn set_subtitles_enabled(&self, on: bool) {
        self.player.set_subtitles_enabled(on);
    }

    /// Load an external subtitle file.
    pub fn set_subtitle_file(&self, path: &Path) -> bool {
        self.player.set_subtitle_file(path)
    }

    /// Collect current video / audio / SPU / chapter lists from libVLC.
    #[must_use]
    pub fn track_snapshot(&self) -> TrackSnapshot {
        let filter = |tracks: Vec<(i32, String)>| {
            tracks
                .into_iter()
                .filter(|(id, _)| *id >= 0)
                .collect::<Vec<_>>()
        };
        TrackSnapshot {
            video: filter(self.player.video_tracks()),
            audio: filter(self.player.audio_tracks()),
            spu: filter(self.player.spu_tracks()),
            chapters: self.player.chapters(),
            audio_devices: self.instance.audio_output_devices(),
            selected_video: self.player.video_track(),
            selected_audio: self.player.audio_track(),
            selected_spu: self.player.spu_track(),
            selected_chapter: self.player.chapter(),
            selected_audio_device: self.player.audio_output_device(),
        }
    }

    /// Select an audio output device by id (empty = system default).
    pub fn set_audio_output_device(&self, device_id: &str) {
        self.player.set_audio_output_device(device_id);
    }

    /// Playback rate (`1.0` = normal).
    pub fn set_rate(&self, rate: f32) {
        self.player.set_rate(rate);
    }

    /// Audio delay in seconds.
    pub fn set_audio_delay_secs(&self, secs: f32) {
        self.player.set_audio_delay_secs(secs);
    }

    /// Subtitle delay in seconds.
    pub fn set_spu_delay_secs(&self, secs: f32) {
        self.player.set_spu_delay_secs(secs);
    }

    /// Video adjust from a `0..=1` slider (`hue` / `brightness` / …).
    pub fn set_video_adjust_slider(&self, which: &str, slider: f32) {
        self.player.set_video_adjust_slider(which, slider);
    }

    /// Aspect ratio label (`Automatic` clears).
    pub fn set_aspect_ratio(&self, aspect: &str) {
        self.player.set_aspect_ratio(aspect);
    }

    /// Crop geometry label (`None` clears).
    pub fn set_crop_geometry(&self, crop: &str) {
        self.player.set_crop_geometry(crop);
    }

    /// Apply UI equalizer gains to the engine (`enabled == false` clears).
    pub fn apply_equalizer(&self, gains: &EqGains) {
        if let Err(e) = self.player.apply_equalizer(gains) {
            log::warn!("equalizer apply failed: {e}");
        }
    }

    /// Select video track by libVLC id.
    pub fn set_video_track(&self, id: i32) {
        self.player.set_video_track(id);
    }

    /// Select audio track by libVLC id.
    pub fn set_audio_track(&self, id: i32) {
        self.player.set_audio_track(id);
    }

    /// Select SPU track (`-1` disables).
    pub fn set_spu_track(&self, id: i32) {
        self.player.set_spu_track(id);
    }

    /// Jump to chapter index.
    pub fn set_chapter(&self, index: i32) {
        self.player.set_chapter(index);
    }

    /// Seek by a relative offset in milliseconds.
    pub fn seek_relative_ms(&self, delta_ms: i64) {
        let next = (self.time_ms() + delta_ms).max(0);
        let len = self.length_ms();
        let clamped = if len > 0 { next.min(len) } else { next };
        self.player.set_time_ms(clamped);
    }

    /// Copy a newer front frame as RGBA (software path only).
    pub fn take_new_front_rgba(&mut self, last_gen: u64) -> Option<(u64, u32, u32, &[u8])> {
        #[cfg(windows)]
        if self.hwnd_mode() {
            return None;
        }
        if !self.player.is_playing() {
            return None;
        }
        let (gen, w, h) = self
            .frames
            .copy_front_rgba_if_newer(last_gen, &mut self.upload)?;
        Some((gen, w, h, self.upload.as_slice()))
    }
}

/// Validate a network MRL before handing it to libVLC.
///
/// # Errors
/// Empty input, control characters, missing/unknown scheme, or `file:`.
pub fn validate_network_mrl(mrl: &str) -> Result<&str> {
    crate::mrl::validate_network_mrl(mrl).map_err(LibVlcError::InvalidMrl)
}
