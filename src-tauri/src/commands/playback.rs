//! Playback commands.

use crate::constants::{DEFAULT_VOLUME, SOFTVOL_ENGINE_MAX};
use crate::error::CommandError;
use crate::path_guard::assert_playable_path;
use crate::pref_units::parse_percent;
use crate::state::{AppState, SessionInfo};
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

fn max_volume_from_settings(state: &AppState) -> i32 {
    let settings = state.load_settings();
    let pct = parse_percent(&settings.prefs.max_volume, 125);
    pct.clamp(1, SOFTVOL_ENGINE_MAX)
}

fn initial_volume(state: &AppState) -> i32 {
    let settings = state.load_settings();
    let max = parse_percent(&settings.prefs.max_volume, 125).clamp(1, SOFTVOL_ENGINE_MAX);
    let mode = settings.prefs.startup_volume_mode.as_str();
    let vol = if mode == "Always starts at" {
        parse_percent(&settings.prefs.startup_volume_pct, DEFAULT_VOLUME)
    } else {
        settings.volume
    };
    vol.clamp(0, max)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub available: bool,
    pub session: SessionInfo,
}

/// IPC DTO for track / chapter lists (mirrors frontend `TrackSnapshot`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackSnapshotDto {
    pub video: Vec<(i32, String)>,
    pub audio: Vec<(i32, String)>,
    pub spu: Vec<(i32, String)>,
    pub chapters: Vec<(i32, String)>,
    pub selected_video: i32,
    pub selected_audio: i32,
    pub selected_spu: i32,
    pub selected_chapter: i32,
}

#[tauri::command]
pub fn engine_status(state: State<'_, AppState>) -> Result<EngineStatus, CommandError> {
    let session = state.lock_session()?.clone();
    Ok(EngineStatus {
        available: state.player.available(),
        session,
    })
}

#[tauri::command]
pub fn play_path(
    state: State<'_, AppState>,
    path: String,
    title: Option<String>,
    media_id: Option<i64>,
    is_video: Option<bool>,
) -> Result<SessionInfo, CommandError> {
    #[cfg(feature = "vlc")]
    {
        let path_buf = assert_playable_path(&state, &path)?;
        let display = title.unwrap_or_else(|| {
            path_buf
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });
        let video = is_video.unwrap_or_else(|| looks_like_video(&path));

        rebind_hwnd_if_needed(&state, video);

        let settings = state.load_settings();
        let media_opts = crate::prefs_apply::media_options_from_prefs(&settings.prefs);
        let opt_refs: Vec<&str> = media_opts.iter().map(String::as_str).collect();

        state
            .player
            .with_mut(|p| p.open_path_with_options(&path_buf, &opt_refs))
            .ok_or_else(|| CommandError::msg("Playback engine unavailable"))?
            .map_err(CommandError::from)?;

        let volume = initial_volume(&state);
        let _ = state.player.with(|p| {
            p.set_volume(volume);
            crate::prefs_apply::apply_session_prefs(p, &settings.prefs);
        });

        // Defer resume until length is known (poller applies pending_seek).
        if let Ok(mut pending) = state.lock_pending_seek() {
            *pending = None;
        }
        let mut resume_offer = None;
        let saved_pos = if crate::prefs_apply::should_auto_resume(&settings)
            || crate::prefs_apply::should_ask_resume(&settings)
        {
            state
                .lock_settings()
                .ok()
                .and_then(|store| store.load_resume_positions().ok())
                .and_then(|map| map.get(&path).map(|(pos, _)| pos.clamp(0.02, 0.98)))
        } else {
            None
        };
        if let Some(pos) = saved_pos {
            if crate::prefs_apply::should_auto_resume(&settings) {
                if let Ok(mut pending) = state.lock_pending_seek() {
                    *pending = Some(pos);
                }
            } else if crate::prefs_apply::should_ask_resume(&settings) {
                resume_offer = Some(pos);
            }
        }

        let session = SessionInfo {
            active: true,
            paused: false,
            title: display,
            subtitle: path_buf
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            path: path.clone(),
            media_id,
            is_video: video,
            position: 0.0,
            time_ms: 0,
            length_ms: 0,
            volume,
            artwork_url: None,
            resume_offer,
        };
        *state.lock_session()? = session.clone();
        Ok(session)
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, path, title, media_id, is_video);
        Err(CommandError::msg(
            "Built without libVLC (vlc feature disabled)",
        ))
    }
}

#[tauri::command]
pub fn play_mrl(
    state: State<'_, AppState>,
    mrl: String,
    title: Option<String>,
) -> Result<SessionInfo, CommandError> {
    #[cfg(feature = "vlc")]
    {
        let mrl = crate::mrl::validate_network_mrl(&mrl)?.to_string();
        let looks_video = !mrl.to_ascii_lowercase().contains("audio")
            && !mrl.to_ascii_lowercase().ends_with(".mp3")
            && !mrl.to_ascii_lowercase().ends_with(".flac")
            && !mrl.to_ascii_lowercase().ends_with(".m4a")
            && !mrl.to_ascii_lowercase().ends_with(".aac")
            && !mrl.to_ascii_lowercase().contains("radio");
        rebind_hwnd_if_needed(&state, looks_video);

        let saved = state
            .lock_settings()
            .ok()
            .and_then(|store| store.load_saved_streams().ok())
            .and_then(|list| list.into_iter().find(|s| s.mrl == mrl));

        let settings = state.load_settings();
        let media_opts = crate::prefs_apply::media_options_from_prefs(&settings.prefs);
        let opt_refs: Vec<&str> = media_opts.iter().map(String::as_str).collect();
        state
            .player
            .with_mut(|p| p.open_mrl_with_options(&mrl, &opt_refs))
            .ok_or_else(|| CommandError::msg("Playback engine unavailable"))?
            .map_err(CommandError::from)?;

        let volume = initial_volume(&state);
        let _ = state.player.with(|p| {
            p.set_volume(volume);
            crate::prefs_apply::apply_session_prefs(p, &settings.prefs);
        });
        if let Ok(mut pending) = state.lock_pending_seek() {
            *pending = None;
        }

        let display = title
            .filter(|t| !t.trim().is_empty())
            .or_else(|| saved.as_ref().map(|s| s.title.clone()))
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| mrl.clone());
        let artwork_url =
            crate::mrl::sanitize_artwork_url(saved.as_ref().and_then(|s| s.artwork_url.as_deref()));

        let session = SessionInfo {
            active: true,
            paused: false,
            title: display,
            subtitle: "Network stream".into(),
            path: mrl,
            media_id: None,
            is_video: looks_video,
            position: 0.0,
            time_ms: 0,
            length_ms: 0,
            volume,
            artwork_url,
            resume_offer: None,
        };
        *state.lock_session()? = session.clone();
        Ok(session)
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, mrl, title);
        Err(CommandError::msg(
            "Built without libVLC (vlc feature disabled)",
        ))
    }
}

#[tauri::command]
pub fn play_pause(state: State<'_, AppState>) -> Result<bool, CommandError> {
    #[cfg(feature = "vlc")]
    {
        state
            .player
            .with_mut(|p| p.play_pause())
            .ok_or_else(|| CommandError::msg("Playback engine unavailable"))?
            .map_err(CommandError::from)?;
        let playing = state.player.with(|p| p.is_playing()).unwrap_or(false);
        let mut s = state.lock_session()?;
        s.paused = !playing;
        s.active = true;
        Ok(playing)
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = state;
        Err(CommandError::msg("Built without libVLC"))
    }
}

#[tauri::command]
pub fn stop_playback(state: State<'_, AppState>) -> Result<(), CommandError> {
    #[cfg(feature = "vlc")]
    {
        persist_resume(&state);
        let _ = state.player.with(|p| p.stop());
        *state.lock_session()? = SessionInfo::default();
        #[cfg(windows)]
        if let Ok(mut v) = state.lock_video() {
            if let Some(embed) = v.as_mut() {
                embed.set_visible(false);
            }
        }
        Ok(())
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = state;
        Ok(())
    }
}

#[tauri::command]
pub fn seek(state: State<'_, AppState>, position: f32) -> Result<(), CommandError> {
    #[cfg(feature = "vlc")]
    {
        state
            .player
            .with(|p| p.seek(position.clamp(0.0, 1.0)))
            .ok_or_else(|| CommandError::msg("Playback engine unavailable"))?;
        state.lock_session()?.position = position.clamp(0.0, 1.0);
        Ok(())
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, position);
        Err(CommandError::msg("Built without libVLC"))
    }
}

#[tauri::command]
pub fn seek_relative(state: State<'_, AppState>, delta_ms: i64) -> Result<(), CommandError> {
    #[cfg(feature = "vlc")]
    {
        state
            .player
            .with(|p| p.seek_relative_ms(delta_ms))
            .ok_or_else(|| CommandError::msg("Playback engine unavailable"))?;
        Ok(())
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, delta_ms);
        Err(CommandError::msg("Built without libVLC"))
    }
}

#[tauri::command]
pub fn set_volume(state: State<'_, AppState>, volume: i32) -> Result<(), CommandError> {
    let max = max_volume_from_settings(&state);
    let volume = volume.clamp(0, max);
    #[cfg(feature = "vlc")]
    {
        let _ = state.player.with(|p| p.set_volume(volume));
    }
    state.lock_session()?.volume = volume;
    let store = state.lock_settings()?;
    if let Ok(mut settings) = store.load() {
        settings.volume = volume;
        store.save(&settings)?;
    }
    Ok(())
}

#[tauri::command]
pub fn track_snapshot(state: State<'_, AppState>) -> Result<TrackSnapshotDto, CommandError> {
    #[cfg(feature = "vlc")]
    {
        let snap = state
            .player
            .with(|p| p.track_snapshot())
            .ok_or_else(|| CommandError::msg("Playback engine unavailable"))?;
        Ok(TrackSnapshotDto {
            video: snap.video,
            audio: snap.audio,
            spu: snap.spu,
            chapters: snap.chapters,
            selected_video: snap.selected_video,
            selected_audio: snap.selected_audio,
            selected_spu: snap.selected_spu,
            selected_chapter: snap.selected_chapter,
        })
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = state;
        Ok(TrackSnapshotDto {
            video: vec![],
            audio: vec![],
            spu: vec![],
            chapters: vec![],
            selected_video: -1,
            selected_audio: -1,
            selected_spu: -1,
            selected_chapter: -1,
        })
    }
}

#[tauri::command]
pub fn set_track(state: State<'_, AppState>, kind: String, id: i32) -> Result<(), CommandError> {
    #[cfg(feature = "vlc")]
    {
        state
            .player
            .with(|p| match kind.as_str() {
                "video" => p.set_video_track(id),
                "audio" => p.set_audio_track(id),
                "spu" | "subs" => p.set_spu_track(id),
                "chapter" => p.set_chapter(id),
                _ => {}
            })
            .ok_or_else(|| CommandError::msg("Playback engine unavailable"))?;
        Ok(())
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, kind, id);
        Err(CommandError::msg("Built without libVLC"))
    }
}

#[tauri::command]
pub fn apply_equalizer(
    state: State<'_, AppState>,
    enabled: bool,
    preamp: f32,
    bands: Vec<f32>,
) -> Result<(), CommandError> {
    #[cfg(feature = "vlc")]
    {
        use libvlc::EqGains;
        let mut gains = EqGains::flat();
        gains.enabled = enabled;
        gains.preamp = preamp;
        for (i, g) in bands.into_iter().take(10).enumerate() {
            gains.bands[i] = g;
        }
        state
            .player
            .with(|p| p.apply_equalizer(&gains))
            .ok_or_else(|| CommandError::msg("Playback engine unavailable"))?;
        Ok(())
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, enabled, preamp, bands);
        Err(CommandError::msg("Built without libVLC"))
    }
}

fn rebind_hwnd_if_needed(state: &AppState, video: bool) {
    #[cfg(all(feature = "vlc", windows))]
    if video {
        if let Ok(slot) = state.lock_video() {
            if let Some(embed) = slot.as_ref() {
                let hwnd = embed.hwnd_void();
                let _ = state.player.with_mut(|p| p.set_hwnd(hwnd));
            }
        }
    }
    #[cfg(not(all(feature = "vlc", windows)))]
    {
        let _ = (state, video);
    }
}

fn looks_like_video(path: &str) -> bool {
    let path_buf = PathBuf::from(path);
    let ext = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if library::is_audio_extension(&ext) {
        return false;
    }
    library::is_video_extension(&ext) || matches!(ext.as_str(), "mpg" | "mpeg" | "")
}

#[cfg(feature = "vlc")]
pub(crate) fn persist_resume(state: &AppState) {
    let settings = state.load_settings();
    if !crate::prefs_apply::should_persist_resume(&settings) {
        return;
    }
    let (path, pos, time_ms) = {
        let Ok(s) = state.lock_session() else {
            return;
        };
        if !s.active || s.path.is_empty() {
            return;
        }
        (s.path.clone(), s.position, s.time_ms)
    };
    if !(0.02..0.95).contains(&pos) {
        return;
    }
    if let Ok(store) = state.lock_settings() {
        if let Ok(mut map) = store.load_resume_positions() {
            map.insert(path, (pos, time_ms));
            let _ = store.save_resume_positions(&map);
        }
    }
}
