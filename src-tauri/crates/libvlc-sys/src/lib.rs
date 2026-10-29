//! Raw FFI declarations for libVLC 3.x.
//!
//! All functions are `unsafe`. Prefer the safe [`libvlc`](../libvlc) crate.
//!
//! When the `link` feature is enabled, the crate links against `libvlc`.
//! Provide `VLC_LIB_DIR` / `VLC_INCLUDE_DIR` or a system install via pkg-config.

#![allow(non_camel_case_types, non_snake_case, dead_code)]
#![allow(clippy::missing_safety_doc, clippy::wildcard_imports)]
#![deny(missing_docs)]

use libc::{c_char, c_float, c_int, c_uint, c_void};

/// Opaque libVLC instance.
#[repr(C)]
pub struct libvlc_instance_t {
    _private: [u8; 0],
}

/// Opaque media object.
#[repr(C)]
pub struct libvlc_media_t {
    _private: [u8; 0],
}

/// Opaque media player.
#[repr(C)]
pub struct libvlc_media_player_t {
    _private: [u8; 0],
}

/// Opaque event manager.
#[repr(C)]
pub struct libvlc_event_manager_t {
    _private: [u8; 0],
}

/// Opaque audio equalizer.
#[repr(C)]
pub struct libvlc_equalizer_t {
    _private: [u8; 0],
}

/// Opaque event.
#[repr(C)]
pub struct libvlc_event_t {
    /// Event type.
    pub type_: c_int,
    _rest: [u8; 0],
}

/// Video lock callback: allocate/return a plane pointer.
pub type libvlc_video_lock_cb =
    Option<unsafe extern "C" fn(opaque: *mut c_void, planes: *mut *mut c_void) -> *mut c_void>;
/// Video unlock callback.
pub type libvlc_video_unlock_cb = Option<
    unsafe extern "C" fn(opaque: *mut c_void, picture: *mut c_void, planes: *const *mut c_void),
>;
/// Video display callback.
pub type libvlc_video_display_cb =
    Option<unsafe extern "C" fn(opaque: *mut c_void, picture: *mut c_void)>;
/// Video format callback.
pub type libvlc_video_format_cb = Option<
    unsafe extern "C" fn(
        opaque: *mut *mut c_void,
        chroma: *mut c_char,
        width: *mut c_uint,
        height: *mut c_uint,
        pitches: *mut c_uint,
        lines: *mut c_uint,
    ) -> c_int,
>;

/// Linked list node for audio/video/SPU track descriptions.
#[repr(C)]
pub struct libvlc_track_description_t {
    /// Track id.
    pub i_id: c_int,
    /// Human-readable name (owned by libVLC).
    pub psz_name: *mut c_char,
    /// Next node.
    pub p_next: *mut libvlc_track_description_t,
}

/// Chapter description (VLC 3.x).
#[repr(C)]
pub struct libvlc_chapter_description_t {
    /// Start time offset in ms.
    pub i_time_offset: i64,
    /// Duration in ms.
    pub i_duration: i64,
    /// Chapter name (owned by libVLC until release).
    pub psz_name: *mut c_char,
}

/// Audio output module entry.
#[repr(C)]
pub struct libvlc_audio_output_t {
    /// Short name of the output module.
    pub psz_name: *mut c_char,
    /// Human-readable description.
    pub psz_description: *mut c_char,
    /// Next entry.
    pub p_next: *mut libvlc_audio_output_t,
}

/// Audio output device entry.
#[repr(C)]
pub struct libvlc_audio_output_device_t {
    /// Device identifier (pass to `device_set`).
    pub psz_device: *mut c_char,
    /// Human-readable device name.
    pub psz_description: *mut c_char,
    /// Next entry.
    pub p_next: *mut libvlc_audio_output_device_t,
}

/// Audio play callback: PCM samples ready.
pub type libvlc_audio_play_cb = Option<
    unsafe extern "C" fn(data: *mut c_void, samples: *const c_void, count: c_uint, pts: i64),
>;
/// Audio pause callback.
pub type libvlc_audio_pause_cb = Option<unsafe extern "C" fn(data: *mut c_void, pts: i64)>;
/// Audio resume callback.
pub type libvlc_audio_resume_cb = Option<unsafe extern "C" fn(data: *mut c_void, pts: i64)>;
/// Audio flush callback.
pub type libvlc_audio_flush_cb = Option<unsafe extern "C" fn(data: *mut c_void, pts: i64)>;
/// Audio drain callback.
pub type libvlc_audio_drain_cb = Option<unsafe extern "C" fn(data: *mut c_void)>;
/// Audio setup callback (negotiate format).
pub type libvlc_audio_setup_cb = Option<
    unsafe extern "C" fn(
        data: *mut *mut c_void,
        format: *mut c_char,
        rate: *mut c_uint,
        channels: *mut c_uint,
    ) -> c_int,
>;
/// Audio cleanup callback.
pub type libvlc_audio_cleanup_cb = Option<unsafe extern "C" fn(data: *mut c_void)>;

#[cfg(feature = "link")]
#[link(name = "vlc")]
extern "C" {
    /// Create a new libVLC instance.
    pub fn libvlc_new(argc: c_int, argv: *const *const c_char) -> *mut libvlc_instance_t;
    /// Release a libVLC instance.
    pub fn libvlc_release(instance: *mut libvlc_instance_t);
    /// Create media from a filesystem path (UTF-8 / local path).
    pub fn libvlc_media_new_path(
        instance: *mut libvlc_instance_t,
        path: *const c_char,
    ) -> *mut libvlc_media_t;
    /// Create media from an MRL/URI.
    pub fn libvlc_media_new_location(
        instance: *mut libvlc_instance_t,
        mrl: *const c_char,
    ) -> *mut libvlc_media_t;
    /// Release media.
    pub fn libvlc_media_release(media: *mut libvlc_media_t);
    /// Add an option to a media (e.g. `:sout=…`).
    pub fn libvlc_media_add_option(media: *mut libvlc_media_t, psz_options: *const c_char);
    /// Create a media player.
    pub fn libvlc_media_player_new(instance: *mut libvlc_instance_t) -> *mut libvlc_media_player_t;
    /// Create a media player from media.
    pub fn libvlc_media_player_new_from_media(
        media: *mut libvlc_media_t,
    ) -> *mut libvlc_media_player_t;
    /// Release a media player.
    pub fn libvlc_media_player_release(player: *mut libvlc_media_player_t);
    /// Set media on a player.
    pub fn libvlc_media_player_set_media(
        player: *mut libvlc_media_player_t,
        media: *mut libvlc_media_t,
    );
    /// Play.
    pub fn libvlc_media_player_play(player: *mut libvlc_media_player_t) -> c_int;
    /// Pause.
    pub fn libvlc_media_player_pause(player: *mut libvlc_media_player_t);
    /// Stop (deprecated in 4, still present in 3).
    pub fn libvlc_media_player_stop(player: *mut libvlc_media_player_t);
    /// Whether playing.
    pub fn libvlc_media_player_is_playing(player: *mut libvlc_media_player_t) -> c_int;
    /// Set position 0.0..=1.0.
    pub fn libvlc_media_player_set_position(player: *mut libvlc_media_player_t, pos: c_float);
    /// Get position 0.0..=1.0.
    pub fn libvlc_media_player_get_position(player: *mut libvlc_media_player_t) -> c_float;
    /// Get length in ms.
    pub fn libvlc_media_player_get_length(player: *mut libvlc_media_player_t) -> i64;
    /// Get time in ms.
    pub fn libvlc_media_player_get_time(player: *mut libvlc_media_player_t) -> i64;
    /// Set time in ms.
    pub fn libvlc_media_player_set_time(player: *mut libvlc_media_player_t, time: i64) -> c_int;
    /// Audio volume 0..=200 (`100` = 0 dB).
    pub fn libvlc_audio_set_volume(player: *mut libvlc_media_player_t, volume: c_int) -> c_int;
    /// Get audio volume.
    pub fn libvlc_audio_get_volume(player: *mut libvlc_media_player_t) -> c_int;
    /// Mute.
    pub fn libvlc_audio_set_mute(player: *mut libvlc_media_player_t, status: c_int) -> c_int;
    /// Number of subtitle (SPU) tracks (`-1` on error).
    pub fn libvlc_video_get_spu_count(player: *mut libvlc_media_player_t) -> c_int;
    /// Current SPU track id (`-1` = disabled).
    pub fn libvlc_video_get_spu(player: *mut libvlc_media_player_t) -> c_int;
    /// Select SPU track by id (`-1` disables).
    pub fn libvlc_video_set_spu(player: *mut libvlc_media_player_t, spu_id: c_int) -> c_int;
    /// Load an external subtitle file.
    pub fn libvlc_video_set_subtitle_file(
        player: *mut libvlc_media_player_t,
        path: *const c_char,
    ) -> c_int;
    /// Linked list of SPU track descriptions (release with track_description_list_release).
    pub fn libvlc_video_get_spu_description(
        player: *mut libvlc_media_player_t,
    ) -> *mut libvlc_track_description_t;
    /// Free a track description list.
    pub fn libvlc_track_description_list_release(desc: *mut libvlc_track_description_t);
    /// Number of video elementary streams.
    pub fn libvlc_video_get_track_count(player: *mut libvlc_media_player_t) -> c_int;
    /// Current video track id (`-1` = disabled).
    pub fn libvlc_video_get_track(player: *mut libvlc_media_player_t) -> c_int;
    /// Select video track by id.
    pub fn libvlc_video_set_track(player: *mut libvlc_media_player_t, track_id: c_int) -> c_int;
    /// Linked list of video track descriptions.
    pub fn libvlc_video_get_track_description(
        player: *mut libvlc_media_player_t,
    ) -> *mut libvlc_track_description_t;
    /// Number of audio tracks.
    pub fn libvlc_audio_get_track_count(player: *mut libvlc_media_player_t) -> c_int;
    /// Current audio track id (`-1` = disabled).
    pub fn libvlc_audio_get_track(player: *mut libvlc_media_player_t) -> c_int;
    /// Select audio track by id.
    pub fn libvlc_audio_set_track(player: *mut libvlc_media_player_t, track_id: c_int) -> c_int;
    /// Linked list of audio track descriptions.
    pub fn libvlc_audio_get_track_description(
        player: *mut libvlc_media_player_t,
    ) -> *mut libvlc_track_description_t;
    /// Chapter count for the current title (`-1` on error).
    pub fn libvlc_media_player_get_chapter_count(player: *mut libvlc_media_player_t) -> c_int;
    /// Current chapter index.
    pub fn libvlc_media_player_get_chapter(player: *mut libvlc_media_player_t) -> c_int;
    /// Jump to chapter index.
    pub fn libvlc_media_player_set_chapter(player: *mut libvlc_media_player_t, chapter: c_int);
    /// Full chapter descriptions for a title (`-1` = current). Returns count.
    pub fn libvlc_media_player_get_full_chapter_descriptions(
        player: *mut libvlc_media_player_t,
        title: c_int,
        chapters: *mut *mut *mut libvlc_chapter_description_t,
    ) -> c_int;
    /// Free chapter description array.
    pub fn libvlc_chapter_descriptions_release(
        chapters: *mut *mut libvlc_chapter_description_t,
        count: c_uint,
    );
    /// List audio output modules.
    pub fn libvlc_audio_output_list_get(
        instance: *mut libvlc_instance_t,
    ) -> *mut libvlc_audio_output_t;
    /// Free audio output module list.
    pub fn libvlc_audio_output_list_release(list: *mut libvlc_audio_output_t);
    /// Select audio output module by short name (e.g. `mmdevice`, `directsound`).
    pub fn libvlc_audio_output_set(
        player: *mut libvlc_media_player_t,
        name: *const c_char,
    ) -> c_int;
    /// List devices for an audio output (`aout` may be null for current).
    pub fn libvlc_audio_output_device_list_get(
        instance: *mut libvlc_instance_t,
        aout: *const c_char,
    ) -> *mut libvlc_audio_output_device_t;
    /// Free audio output device list.
    pub fn libvlc_audio_output_device_list_release(list: *mut libvlc_audio_output_device_t);
    /// Select audio output device.
    pub fn libvlc_audio_output_device_set(
        player: *mut libvlc_media_player_t,
        module: *const c_char,
        device_id: *const c_char,
    ) -> c_int;
    /// Current audio output device id (must be freed with libc free / libvlc frees).
    pub fn libvlc_audio_output_device_get(player: *mut libvlc_media_player_t) -> *mut c_char;
    /// Set raw audio output callbacks (replaces system aout — for metering / analysis).
    pub fn libvlc_audio_set_callbacks(
        player: *mut libvlc_media_player_t,
        play: libvlc_audio_play_cb,
        pause: libvlc_audio_pause_cb,
        resume: libvlc_audio_resume_cb,
        flush: libvlc_audio_flush_cb,
        drain: libvlc_audio_drain_cb,
        opaque: *mut c_void,
    );
    /// Set audio format negotiation callbacks.
    pub fn libvlc_audio_set_format_callbacks(
        player: *mut libvlc_media_player_t,
        setup: libvlc_audio_setup_cb,
        cleanup: libvlc_audio_cleanup_cb,
    );
    /// Set video callbacks for software rendering.
    pub fn libvlc_video_set_callbacks(
        player: *mut libvlc_media_player_t,
        lock: libvlc_video_lock_cb,
        unlock: libvlc_video_unlock_cb,
        display: libvlc_video_display_cb,
        opaque: *mut c_void,
    );
    /// Set requested video format (chroma string e.g. "RV32").
    pub fn libvlc_video_set_format(
        player: *mut libvlc_media_player_t,
        chroma: *const c_char,
        width: c_uint,
        height: c_uint,
        pitch: c_uint,
    );
    /// Set video format callbacks.
    pub fn libvlc_video_set_format_callbacks(
        player: *mut libvlc_media_player_t,
        setup: libvlc_video_format_cb,
        cleanup: Option<unsafe extern "C" fn(opaque: *mut c_void)>,
    );
    /// Take a video snapshot to `path` (JPEG/PNG by extension). `num` is the
    /// video output index (usually 0). Width/height 0 = original size.
    pub fn libvlc_video_take_snapshot(
        player: *mut libvlc_media_player_t,
        num: c_uint,
        path: *const c_char,
        width: c_uint,
        height: c_uint,
    ) -> c_int;
    /// Embed video into a Win32 window (`HWND` as `void*`). Pass null to detach.
    pub fn libvlc_media_player_set_hwnd(player: *mut libvlc_media_player_t, hwnd: *mut c_void);
    /// Enable/disable key events on the video window (0 = let parent handle keys).
    pub fn libvlc_video_set_key_input(player: *mut libvlc_media_player_t, on: c_uint);
    /// Enable/disable mouse events on the video window.
    pub fn libvlc_video_set_mouse_input(player: *mut libvlc_media_player_t, on: c_uint);
    /// Playback rate (1.0 = normal).
    pub fn libvlc_media_player_set_rate(player: *mut libvlc_media_player_t, rate: c_float)
        -> c_int;
    /// Current playback rate.
    pub fn libvlc_media_player_get_rate(player: *mut libvlc_media_player_t) -> c_float;
    /// Audio delay in microseconds.
    pub fn libvlc_audio_set_delay(player: *mut libvlc_media_player_t, delay: i64) -> c_int;
    /// Current audio delay (μs).
    pub fn libvlc_audio_get_delay(player: *mut libvlc_media_player_t) -> i64;
    /// Subtitle delay in microseconds.
    pub fn libvlc_video_set_spu_delay(player: *mut libvlc_media_player_t, delay: i64) -> c_int;
    /// Current subtitle delay (μs).
    pub fn libvlc_video_get_spu_delay(player: *mut libvlc_media_player_t) -> i64;
    /// Enable / set int video adjust option (`libvlc_adjust_*`).
    pub fn libvlc_video_set_adjust_int(
        player: *mut libvlc_media_player_t,
        option: c_uint,
        value: c_int,
    );
    /// Set float video adjust option.
    pub fn libvlc_video_set_adjust_float(
        player: *mut libvlc_media_player_t,
        option: c_uint,
        value: c_float,
    );
    /// Get float video adjust option.
    pub fn libvlc_video_get_adjust_float(
        player: *mut libvlc_media_player_t,
        option: c_uint,
    ) -> c_float;
    /// Aspect ratio string (`""` / null = default).
    pub fn libvlc_video_set_aspect_ratio(player: *mut libvlc_media_player_t, aspect: *const c_char);
    /// Crop geometry (`""` / null = none).
    pub fn libvlc_video_set_crop_geometry(
        player: *mut libvlc_media_player_t,
        geometry: *const c_char,
    );
    /// Create a new equalizer (flat).
    pub fn libvlc_audio_equalizer_new() -> *mut libvlc_equalizer_t;
    /// Create equalizer from a built-in preset index.
    pub fn libvlc_audio_equalizer_new_from_preset(u_index: c_uint) -> *mut libvlc_equalizer_t;
    /// Release an equalizer.
    pub fn libvlc_audio_equalizer_release(p_equalizer: *mut libvlc_equalizer_t);
    /// Set preamp amplification in dB (−20…+20).
    pub fn libvlc_audio_equalizer_set_preamp(
        p_equalizer: *mut libvlc_equalizer_t,
        f_preamp: c_float,
    ) -> c_int;
    /// Amplification for band `u_band` in dB (−20…+20).
    pub fn libvlc_audio_equalizer_set_amp_at_index(
        p_equalizer: *mut libvlc_equalizer_t,
        f_amp: c_float,
        u_band: c_uint,
    ) -> c_int;
    /// Number of equalizer frequency bands.
    pub fn libvlc_audio_equalizer_get_band_count() -> c_uint;
    /// Center frequency (Hz) for band `u_index`.
    pub fn libvlc_audio_equalizer_get_band_frequency(u_index: c_uint) -> c_float;
    /// Apply equalizer to player (`null` disables).
    pub fn libvlc_media_player_set_equalizer(
        player: *mut libvlc_media_player_t,
        p_equalizer: *mut libvlc_equalizer_t,
    ) -> c_int;
    /// Last error message.
    pub fn libvlc_errmsg() -> *const c_char;
    /// Clear last error.
    pub fn libvlc_clearerr();
    /// Free a string allocated by libVLC.
    pub fn libvlc_free(ptr: *mut c_void);
    /// Media currently attached to the player (caller must `libvlc_media_release`).
    pub fn libvlc_media_player_get_media(player: *mut libvlc_media_player_t)
        -> *mut libvlc_media_t;
    /// Read a metadata field (caller must `libvlc_free` the returned string).
    pub fn libvlc_media_get_meta(media: *mut libvlc_media_t, e_meta: libvlc_meta_t) -> *mut c_char;
}

/// libVLC media metadata keys (subset used by Orpheon).
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum libvlc_meta_t {
    /// Title.
    Title = 0,
    /// Artist.
    Artist = 1,
    /// Genre.
    Genre = 2,
    /// Copyright.
    Copyright = 3,
    /// Album.
    Album = 4,
    /// Track number.
    TrackNumber = 5,
    /// Description.
    Description = 6,
    /// Rating.
    Rating = 7,
    /// Date.
    Date = 8,
    /// Setting.
    Setting = 9,
    /// URL.
    URL = 10,
    /// Language.
    Language = 11,
    /// Now playing (ICY / radio).
    NowPlaying = 12,
    /// Publisher.
    Publisher = 13,
    /// Encoded by.
    EncodedBy = 14,
    /// Artwork URL.
    ArtworkURL = 15,
}

#[cfg(not(feature = "link"))]
mod stubs {
    use super::*;

    /// Stub: returns null.
    pub unsafe extern "C" fn libvlc_new(
        _argc: c_int,
        _argv: *const *const c_char,
    ) -> *mut libvlc_instance_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_release(_instance: *mut libvlc_instance_t) {}
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_new_path(
        _instance: *mut libvlc_instance_t,
        _path: *const c_char,
    ) -> *mut libvlc_media_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_new_location(
        _instance: *mut libvlc_instance_t,
        _mrl: *const c_char,
    ) -> *mut libvlc_media_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_release(_media: *mut libvlc_media_t) {}
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_add_option(
        _media: *mut libvlc_media_t,
        _psz_options: *const c_char,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_new(
        _instance: *mut libvlc_instance_t,
    ) -> *mut libvlc_media_player_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_new_from_media(
        _media: *mut libvlc_media_t,
    ) -> *mut libvlc_media_player_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_release(_player: *mut libvlc_media_player_t) {}
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_set_media(
        _player: *mut libvlc_media_player_t,
        _media: *mut libvlc_media_t,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_play(
        _player: *mut libvlc_media_player_t,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_pause(_player: *mut libvlc_media_player_t) {}
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_stop(_player: *mut libvlc_media_player_t) {}
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_is_playing(
        _player: *mut libvlc_media_player_t,
    ) -> c_int {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_set_position(
        _player: *mut libvlc_media_player_t,
        _pos: c_float,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_get_position(
        _player: *mut libvlc_media_player_t,
    ) -> c_float {
        0.0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_get_length(
        _player: *mut libvlc_media_player_t,
    ) -> i64 {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_get_time(
        _player: *mut libvlc_media_player_t,
    ) -> i64 {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_set_time(
        _player: *mut libvlc_media_player_t,
        _time: i64,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_set_volume(
        _player: *mut libvlc_media_player_t,
        _volume: c_int,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_get_volume(_player: *mut libvlc_media_player_t) -> c_int {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_set_mute(
        _player: *mut libvlc_media_player_t,
        _status: c_int,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_get_spu_count(
        _player: *mut libvlc_media_player_t,
    ) -> c_int {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_get_spu(_player: *mut libvlc_media_player_t) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_spu(
        _player: *mut libvlc_media_player_t,
        _spu_id: c_int,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_subtitle_file(
        _player: *mut libvlc_media_player_t,
        _path: *const c_char,
    ) -> c_int {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_get_spu_description(
        _player: *mut libvlc_media_player_t,
    ) -> *mut libvlc_track_description_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_track_description_list_release(
        _desc: *mut libvlc_track_description_t,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_get_track_count(
        _player: *mut libvlc_media_player_t,
    ) -> c_int {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_get_track(_player: *mut libvlc_media_player_t) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_track(
        _player: *mut libvlc_media_player_t,
        _track_id: c_int,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_get_track_description(
        _player: *mut libvlc_media_player_t,
    ) -> *mut libvlc_track_description_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_get_track_count(
        _player: *mut libvlc_media_player_t,
    ) -> c_int {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_get_track(_player: *mut libvlc_media_player_t) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_set_track(
        _player: *mut libvlc_media_player_t,
        _track_id: c_int,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_get_track_description(
        _player: *mut libvlc_media_player_t,
    ) -> *mut libvlc_track_description_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_get_chapter_count(
        _player: *mut libvlc_media_player_t,
    ) -> c_int {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_get_chapter(
        _player: *mut libvlc_media_player_t,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_set_chapter(
        _player: *mut libvlc_media_player_t,
        _chapter: c_int,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_get_full_chapter_descriptions(
        _player: *mut libvlc_media_player_t,
        _title: c_int,
        _chapters: *mut *mut *mut libvlc_chapter_description_t,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_chapter_descriptions_release(
        _chapters: *mut *mut libvlc_chapter_description_t,
        _count: c_uint,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_output_list_get(
        _instance: *mut libvlc_instance_t,
    ) -> *mut libvlc_audio_output_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_output_list_release(_list: *mut libvlc_audio_output_t) {}
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_output_set(
        _player: *mut libvlc_media_player_t,
        _name: *const c_char,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_output_device_list_get(
        _instance: *mut libvlc_instance_t,
        _aout: *const c_char,
    ) -> *mut libvlc_audio_output_device_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_output_device_list_release(
        _list: *mut libvlc_audio_output_device_t,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_output_device_set(
        _player: *mut libvlc_media_player_t,
        _module: *const c_char,
        _device_id: *const c_char,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_output_device_get(
        _player: *mut libvlc_media_player_t,
    ) -> *mut c_char {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_set_callbacks(
        _player: *mut libvlc_media_player_t,
        _play: libvlc_audio_play_cb,
        _pause: libvlc_audio_pause_cb,
        _resume: libvlc_audio_resume_cb,
        _flush: libvlc_audio_flush_cb,
        _drain: libvlc_audio_drain_cb,
        _opaque: *mut c_void,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_set_format_callbacks(
        _player: *mut libvlc_media_player_t,
        _setup: libvlc_audio_setup_cb,
        _cleanup: libvlc_audio_cleanup_cb,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_callbacks(
        _player: *mut libvlc_media_player_t,
        _lock: libvlc_video_lock_cb,
        _unlock: libvlc_video_unlock_cb,
        _display: libvlc_video_display_cb,
        _opaque: *mut c_void,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_format(
        _player: *mut libvlc_media_player_t,
        _chroma: *const c_char,
        _width: c_uint,
        _height: c_uint,
        _pitch: c_uint,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_format_callbacks(
        _player: *mut libvlc_media_player_t,
        _setup: libvlc_video_format_cb,
        _cleanup: Option<unsafe extern "C" fn(opaque: *mut c_void)>,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_take_snapshot(
        _player: *mut libvlc_media_player_t,
        _num: c_uint,
        _path: *const c_char,
        _width: c_uint,
        _height: c_uint,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_set_hwnd(
        _player: *mut libvlc_media_player_t,
        _hwnd: *mut c_void,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_key_input(
        _player: *mut libvlc_media_player_t,
        _on: c_uint,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_mouse_input(
        _player: *mut libvlc_media_player_t,
        _on: c_uint,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_set_rate(
        _player: *mut libvlc_media_player_t,
        _rate: c_float,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_get_rate(
        _player: *mut libvlc_media_player_t,
    ) -> c_float {
        1.0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_set_delay(
        _player: *mut libvlc_media_player_t,
        _delay: i64,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_get_delay(_player: *mut libvlc_media_player_t) -> i64 {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_spu_delay(
        _player: *mut libvlc_media_player_t,
        _delay: i64,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_get_spu_delay(
        _player: *mut libvlc_media_player_t,
    ) -> i64 {
        0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_adjust_int(
        _player: *mut libvlc_media_player_t,
        _option: c_uint,
        _value: c_int,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_adjust_float(
        _player: *mut libvlc_media_player_t,
        _option: c_uint,
        _value: c_float,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_get_adjust_float(
        _player: *mut libvlc_media_player_t,
        _option: c_uint,
    ) -> c_float {
        1.0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_aspect_ratio(
        _player: *mut libvlc_media_player_t,
        _aspect: *const c_char,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_video_set_crop_geometry(
        _player: *mut libvlc_media_player_t,
        _geometry: *const c_char,
    ) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_equalizer_new() -> *mut libvlc_equalizer_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_equalizer_new_from_preset(
        _u_index: c_uint,
    ) -> *mut libvlc_equalizer_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_equalizer_release(_p_equalizer: *mut libvlc_equalizer_t) {
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_equalizer_set_preamp(
        _p_equalizer: *mut libvlc_equalizer_t,
        _f_preamp: c_float,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_equalizer_set_amp_at_index(
        _p_equalizer: *mut libvlc_equalizer_t,
        _f_amp: c_float,
        _u_band: c_uint,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_equalizer_get_band_count() -> c_uint {
        10
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_audio_equalizer_get_band_frequency(
        _u_index: c_uint,
    ) -> c_float {
        0.0
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_set_equalizer(
        _player: *mut libvlc_media_player_t,
        _p_equalizer: *mut libvlc_equalizer_t,
    ) -> c_int {
        -1
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_errmsg() -> *const c_char {
        std::ptr::null()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_clearerr() {}
    /// Stub.
    pub unsafe extern "C" fn libvlc_free(_ptr: *mut c_void) {}
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_player_get_media(
        _player: *mut libvlc_media_player_t,
    ) -> *mut libvlc_media_t {
        std::ptr::null_mut()
    }
    /// Stub.
    pub unsafe extern "C" fn libvlc_media_get_meta(
        _media: *mut libvlc_media_t,
        _e_meta: libvlc_meta_t,
    ) -> *mut c_char {
        std::ptr::null_mut()
    }
}

#[cfg(not(feature = "link"))]
pub use stubs::*;

#[cfg(feature = "link")]
#[allow(clippy::missing_const_for_fn)]
fn _link_hint() {
    // On Windows, set PATH to the folder containing libvlc.dll,
    // or pass VLC_LIB_DIR to the build script of dependents.
}
