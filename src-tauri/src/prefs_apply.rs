//! Map persisted prefs onto a live [`PlayerService`] / media open options.

use crate::pref_units::parse_seconds;
use crate::player_service::PlayerService;
use settings::{AppSettings, ContinueMode, Prefs};

/// Build media-level options applied at open (deinterlace, subtitles, …).
#[must_use]
pub fn media_options_from_prefs(prefs: &Prefs) -> Vec<String> {
    let mut opts = Vec::new();

    match prefs.deinterlace.as_str() {
        "Off" => opts.push(":deinterlace=0".into()),
        "On" => opts.push(":deinterlace=1".into()),
        _ => {}
    }
    let mode = prefs.deinterlace_mode.trim();
    if !mode.is_empty() && mode != "—" {
        opts.push(format!(":deinterlace-mode={}", mode.to_ascii_lowercase()));
    }

    if !prefs.spu_autoload {
        opts.push(":no-sub-autodetect-file".into());
    }

    if !prefs.enable_audio {
        opts.push(":no-audio".into());
    }
    if !prefs.enable_video {
        opts.push(":no-video".into());
    }

    opts
}

/// Whether resume-from-position should run for this open.
#[must_use]
pub fn should_resume(settings: &AppSettings) -> bool {
    if !settings.resume_playback {
        return false;
    }
    match settings.prefs.continue_playback_mode() {
        ContinueMode::Never => false,
        ContinueMode::Always | ContinueMode::Ask => true,
    }
}

/// Apply session-level prefs that can change without reopening media.
pub fn apply_session_prefs(player: &PlayerService, prefs: &Prefs) {
    let rate = prefs.default_rate.trim().parse::<f32>().unwrap_or(1.0);
    if rate.is_finite() && rate > 0.0 {
        player.set_rate(rate.clamp(0.25, 4.0));
    }

    apply_picture(player, prefs);
    player.set_aspect_ratio(&prefs.aspect_ratio);
    player.set_crop_geometry(&prefs.crop);

    let audio_delay = prefs.audio_delay.trim().parse::<f32>().unwrap_or(0.0);
    if audio_delay.is_finite() {
        player.set_audio_delay_secs(audio_delay);
    }
    let spu_delay = parse_seconds(&prefs.spu_delay, 0.0);
    if spu_delay.is_finite() {
        player.set_spu_delay_secs(spu_delay);
    }

    let device = prefs.audio_device.trim();
    if !device.is_empty()
        && !device.eq_ignore_ascii_case("System default")
        && !device.eq_ignore_ascii_case("—")
    {
        player.set_audio_output_device(device);
    }

    if !prefs.enable_audio {
        player.set_mute(true);
    }
}

fn apply_picture(player: &PlayerService, prefs: &Prefs) {
    let hue = prefs.hue.trim().parse::<f32>().unwrap_or(0.0);
    let brightness = prefs.brightness.trim().parse::<f32>().unwrap_or(1.0);
    let contrast = prefs.contrast.trim().parse::<f32>().unwrap_or(1.0);
    let saturation = prefs.saturation.trim().parse::<f32>().unwrap_or(1.0);
    let gamma = prefs.gamma.trim().parse::<f32>().unwrap_or(1.0);

    // Prefs store degrees / multipliers; adjust API uses 0..=1 sliders (0.5 = neutral).
    player.set_video_adjust_slider("hue", ((hue / 360.0) + 0.5).clamp(0.0, 1.0));
    player.set_video_adjust_slider("brightness", (brightness / 2.0).clamp(0.0, 1.0));
    player.set_video_adjust_slider("contrast", (contrast / 2.0).clamp(0.0, 1.0));
    player.set_video_adjust_slider("saturation", (saturation / 2.0).clamp(0.0, 1.0));
    player.set_video_adjust_slider("gamma", (gamma / 2.0).clamp(0.0, 1.0));
}

/// Split `extra_cmdline` / `vlc_options` into libVLC argv tokens.
#[must_use]
pub fn extra_vlc_args(settings: &AppSettings) -> Vec<String> {
    let mut out = Vec::new();
    for token in settings.prefs.extra_cmdline.split_whitespace() {
        if !token.is_empty() {
            out.push(token.to_string());
        }
    }
    for opt in &settings.vlc_options {
        let t = opt.trim();
        if !t.is_empty() {
            out.push(t.to_string());
        }
    }

    match settings.prefs.skip_loop_filter.as_str() {
        "Non-ref" => out.push("--avcodec-skiploopfilter=1".into()),
        "All" => out.push("--avcodec-skiploopfilter=4".into()),
        _ => {}
    }

    let threads = settings
        .prefs
        .decode_threads
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or("0");
    if let Ok(n) = threads.parse::<u32>() {
        if n > 0 {
            out.push(format!("--avcodec-threads={n}"));
        }
    }

    match settings.prefs.postproc.as_str() {
        "Low" => out.push("--video-filter=postproc".into()),
        "Medium" => {
            out.push("--video-filter=postproc".into());
            out.push("--postproc-q=4".into());
        }
        "High" => {
            out.push("--video-filter=postproc".into());
            out.push("--postproc-q=6".into());
        }
        _ => {}
    }

    out
}

/// Map settings hw prefs → [`libvlc::HwAccelMode`].
#[must_use]
pub fn hw_accel_from_settings(settings: &AppSettings) -> libvlc::HwAccelMode {
    use libvlc::HwAccelMode;
    if !settings.hw_accel {
        return HwAccelMode::Disabled;
    }
    match settings.prefs.hw_accel_mode.as_str() {
        "Disabled" => HwAccelMode::Disabled,
        "D3D11" => HwAccelMode::D3D11,
        "DXVA2" => HwAccelMode::Dxva2,
        _ => HwAccelMode::Automatic,
    }
}
