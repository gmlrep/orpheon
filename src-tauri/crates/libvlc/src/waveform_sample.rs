//! Seek-sample audio peaks via libVLC (for formats Symphonia cannot decode).

use crate::{Instance, Media, MediaPlayer, Result};
use std::os::raw::{c_char, c_uint, c_void};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Ensures [`MediaPlayer::stop`] runs on every exit path (success or error).
struct StopOnDrop<'a>(&'a MediaPlayer);

impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.stop();
    }
}

/// Sample `bars` peak levels (`0..=1`) by seeking through `path` with a silent libVLC player.
///
/// # Errors
/// Returns an error when libVLC cannot open the file or no audio peaks are heard.
pub fn sample_peaks(path: &Path, bars: usize) -> Result<Vec<f32>> {
    if bars == 0 {
        return Ok(vec![]);
    }

    let instance = Instance::with_args(&[
        "--intf=dummy",
        "--quiet",
        "--no-video",
        "--no-video-title-show",
        "--no-sub-autodetect-file",
    ])?;
    let media = Media::from_path(&instance, path)?;
    let player = MediaPlayer::new(&instance)?;
    let _stop_guard = StopOnDrop(&player);

    let peak = Arc::new(AtomicU32::new(0));
    let peak_ptr = Arc::as_ptr(&peak) as *mut c_void;
    let _keep = peak.clone();

    unsafe {
        libvlc_sys::libvlc_audio_set_callbacks(
            player.as_raw(),
            Some(wave_play_cb),
            None,
            None,
            None,
            None,
            peak_ptr,
        );
        libvlc_sys::libvlc_audio_set_format_callbacks(player.as_raw(), Some(wave_setup_cb), None);
    }

    player.set_media(&media);
    player.play()?;

    let mut length = 0_i64;
    for _ in 0..40 {
        std::thread::sleep(Duration::from_millis(50));
        length = player.length_ms();
        if length > 0 {
            break;
        }
    }
    if length <= 0 {
        return Err(crate::LibVlcError::PlayFailed);
    }

    let samples = bars.min(80);
    let mut sparse = vec![0.0_f32; samples];
    for (i, slot) in sparse.iter_mut().enumerate() {
        let pos = (i as f64 + 0.5) / samples as f64;
        player.set_position(pos as f32);
        peak.store(0, Ordering::Release);
        if !player.is_playing() {
            let _ = player.play();
        }
        std::thread::sleep(Duration::from_millis(90));
        *slot = f32::from_bits(peak.load(Ordering::Acquire));
    }

    Ok(downsample(&sparse, bars))
}

fn downsample(src: &[f32], bars: usize) -> Vec<f32> {
    const MIN: f32 = 0.02;
    let mut out = vec![0.0_f32; bars];
    let n = src.len().max(1);
    for (i, slot) in out.iter_mut().enumerate() {
        let start = i * n / bars;
        let end = ((i + 1) * n / bars).max(start + 1).min(n);
        let mut peak = 0.0_f32;
        for &v in &src[start..end] {
            if v > peak {
                peak = v;
            }
        }
        *slot = peak;
    }
    let mut sorted = out.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p95_idx = ((sorted.len() as f32) * 0.95).floor() as usize;
    let ref_level = sorted
        .get(p95_idx.min(sorted.len().saturating_sub(1)))
        .copied()
        .unwrap_or(1.0)
        .max(1e-6);
    out.into_iter()
        .map(|v| {
            let n = (v / ref_level).clamp(0.0, 1.0);
            n.powf(1.35).clamp(MIN, 1.0)
        })
        .collect()
}

unsafe extern "C" fn wave_setup_cb(
    _data: *mut *mut c_void,
    format: *mut c_char,
    rate: *mut c_uint,
    channels: *mut c_uint,
) -> i32 {
    unsafe {
        if !format.is_null() {
            let s16 = *b"S16N";
            std::ptr::copy_nonoverlapping(s16.as_ptr(), format.cast::<u8>(), 4);
        }
        if !rate.is_null() && *rate == 0 {
            *rate = 44100;
        }
        if !channels.is_null() && *channels == 0 {
            *channels = 2;
        }
    }
    0
}

unsafe extern "C" fn wave_play_cb(
    data: *mut c_void,
    samples: *const c_void,
    count: c_uint,
    _pts: i64,
) {
    if data.is_null() || samples.is_null() || count == 0 {
        return;
    }
    let peak_cell = unsafe { &*(data as *const AtomicU32) };
    let slice = unsafe { std::slice::from_raw_parts(samples as *const i16, count as usize) };
    let mut local = 0.0_f32;
    for &s in slice {
        let a = (s as f32).abs() / 32768.0;
        if a > local {
            local = a;
        }
    }
    loop {
        let prev = peak_cell.load(Ordering::Acquire);
        if local <= f32::from_bits(prev) {
            break;
        }
        if peak_cell
            .compare_exchange(prev, local.to_bits(), Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            break;
        }
    }
}
