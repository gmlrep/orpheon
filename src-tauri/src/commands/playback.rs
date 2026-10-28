//! Playback commands.

use crate::state::{AppState, SessionInfo};
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub available: bool,
    pub session: SessionInfo,
}

#[tauri::command]
pub fn engine_status(state: State<'_, AppState>) -> EngineStatus {
    let session = state.session.lock().map(|s| s.clone()).unwrap_or_default();
    EngineStatus {
        available: state.player.available(),
        session,
    }
}

#[tauri::command]
pub fn play_path(
    state: State<'_, AppState>,
    path: String,
    title: Option<String>,
    media_id: Option<i64>,
    is_video: Option<bool>,
) -> Result<SessionInfo, String> {
    #[cfg(feature = "vlc")]
    {
        let path_buf = PathBuf::from(&path);
        let display = title.unwrap_or_else(|| {
            path_buf
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });
        let video = is_video.unwrap_or_else(|| looks_like_video(&path));

        rebind_hwnd_if_needed(&state, video);

        state
            .player
            .with_mut(|p| p.open_path(&path_buf))
            .ok_or_else(|| "Playback engine unavailable".to_string())?
            .map_err(|e| e.to_string())?;

        let volume = state.load_settings().volume;
        let _ = state.player.with(|p| p.set_volume(volume));

        // Defer resume until length is known (poller applies pending_seek).
        if let Ok(mut pending) = state.pending_seek.lock() {
            *pending = None;
        }
        if state.load_settings().resume_playback {
            if let Ok(store) = state.settings.lock() {
                if let Ok(map) = store.load_resume_positions() {
                    if let Some((pos, _)) = map.get(&path) {
                        if let Ok(mut pending) = state.pending_seek.lock() {
                            *pending = Some(pos.clamp(0.02, 0.98));
                        }
                    }
                }
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
        };
        if let Ok(mut s) = state.session.lock() {
            *s = session.clone();
        }
        Ok(session)
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, path, title, media_id, is_video);
        Err("Built without libVLC (vlc feature disabled)".into())
    }
}

#[tauri::command]
pub fn play_mrl(state: State<'_, AppState>, mrl: String) -> Result<SessionInfo, String> {
    #[cfg(feature = "vlc")]
    {
        let looks_video = !mrl.to_ascii_lowercase().contains("audio")
            && !mrl.to_ascii_lowercase().ends_with(".mp3")
            && !mrl.to_ascii_lowercase().ends_with(".flac")
            && !mrl.to_ascii_lowercase().ends_with(".m4a");
        rebind_hwnd_if_needed(&state, looks_video);

        state
            .player
            .with_mut(|p| p.open_mrl(&mrl))
            .ok_or_else(|| "Playback engine unavailable".to_string())?
            .map_err(|e| e.to_string())?;

        let volume = state.load_settings().volume;
        let _ = state.player.with(|p| p.set_volume(volume));
        if let Ok(mut pending) = state.pending_seek.lock() {
            *pending = None;
        }

        let session = SessionInfo {
            active: true,
            paused: false,
            title: mrl.clone(),
            subtitle: "Network stream".into(),
            path: mrl,
            media_id: None,
            is_video: looks_video,
            position: 0.0,
            time_ms: 0,
            length_ms: 0,
            volume,
        };
        if let Ok(mut s) = state.session.lock() {
            *s = session.clone();
        }
        Ok(session)
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, mrl);
        Err("Built without libVLC (vlc feature disabled)".into())
    }
}

#[tauri::command]
pub fn play_pause(state: State<'_, AppState>) -> Result<bool, String> {
    #[cfg(feature = "vlc")]
    {
        state
            .player
            .with_mut(|p| p.play_pause())
            .ok_or_else(|| "Playback engine unavailable".to_string())?
            .map_err(|e| e.to_string())?;
        let playing = state.player.with(|p| p.is_playing()).unwrap_or(false);
        if let Ok(mut s) = state.session.lock() {
            s.paused = !playing;
            s.active = true;
        }
        Ok(playing)
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = state;
        Err("Built without libVLC".into())
    }
}

#[tauri::command]
pub fn stop_playback(state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(feature = "vlc")]
    {
        persist_resume(&state);
        let _ = state.player.with(|p| p.stop());
        if let Ok(mut s) = state.session.lock() {
            *s = SessionInfo::default();
        }
        #[cfg(windows)]
        if let Ok(mut v) = state.video.lock() {
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
pub fn seek(state: State<'_, AppState>, position: f32) -> Result<(), String> {
    #[cfg(feature = "vlc")]
    {
        let _ = state
            .player
            .with(|p| p.seek(position.clamp(0.0, 1.0)))
            .ok_or_else(|| "Playback engine unavailable".to_string())?;
        if let Ok(mut s) = state.session.lock() {
            s.position = position.clamp(0.0, 1.0);
        }
        Ok(())
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, position);
        Err("Built without libVLC".into())
    }
}

#[tauri::command]
pub fn seek_relative(state: State<'_, AppState>, delta_ms: i64) -> Result<(), String> {
    #[cfg(feature = "vlc")]
    {
        let _ = state
            .player
            .with(|p| p.seek_relative_ms(delta_ms))
            .ok_or_else(|| "Playback engine unavailable".to_string())?;
        Ok(())
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, delta_ms);
        Err("Built without libVLC".into())
    }
}

#[tauri::command]
pub fn set_volume(state: State<'_, AppState>, volume: i32) -> Result<(), String> {
    let volume = volume.clamp(0, 200);
    #[cfg(feature = "vlc")]
    {
        let _ = state.player.with(|p| p.set_volume(volume));
    }
    if let Ok(mut s) = state.session.lock() {
        s.volume = volume;
    }
    if let Ok(store) = state.settings.lock() {
        if let Ok(mut settings) = store.load() {
            settings.volume = volume;
            let _ = store.save(&settings);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn track_snapshot(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    #[cfg(feature = "vlc")]
    {
        let snap = state
            .player
            .with(|p| p.track_snapshot())
            .ok_or_else(|| "Playback engine unavailable".to_string())?;
        Ok(serde_json::json!({
            "video": snap.video,
            "audio": snap.audio,
            "spu": snap.spu,
            "chapters": snap.chapters,
            "selectedVideo": snap.selected_video,
            "selectedAudio": snap.selected_audio,
            "selectedSpu": snap.selected_spu,
            "selectedChapter": snap.selected_chapter,
        }))
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = state;
        Ok(serde_json::json!({}))
    }
}

#[tauri::command]
pub fn set_track(
    state: State<'_, AppState>,
    kind: String,
    id: i32,
) -> Result<(), String> {
    #[cfg(feature = "vlc")]
    {
        let _ = state
            .player
            .with(|p| match kind.as_str() {
                "video" => p.set_video_track(id),
                "audio" => p.set_audio_track(id),
                "spu" | "subs" => p.set_spu_track(id),
                "chapter" => p.set_chapter(id),
                _ => {}
            })
            .ok_or_else(|| "Playback engine unavailable".to_string())?;
        Ok(())
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, kind, id);
        Err("Built without libVLC".into())
    }
}

#[tauri::command]
pub fn apply_equalizer(
    state: State<'_, AppState>,
    enabled: bool,
    preamp: f32,
    bands: Vec<f32>,
) -> Result<(), String> {
    #[cfg(feature = "vlc")]
    {
        use libvlc::EqGains;
        let mut gains = EqGains::flat();
        gains.enabled = enabled;
        gains.preamp = preamp;
        for (i, g) in bands.into_iter().take(10).enumerate() {
            gains.bands[i] = g;
        }
        let _ = state
            .player
            .with(|p| p.apply_equalizer(&gains))
            .ok_or_else(|| "Playback engine unavailable".to_string())?;
        Ok(())
    }
    #[cfg(not(feature = "vlc"))]
    {
        let _ = (state, enabled, preamp, bands);
        Err("Built without libVLC".into())
    }
}

fn rebind_hwnd_if_needed(state: &AppState, video: bool) {
    #[cfg(all(feature = "vlc", windows))]
    if video {
        if let Ok(slot) = state.video.lock() {
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
    if !state.load_settings().resume_playback {
        return;
    }
    let (path, pos, time_ms) = {
        let Ok(s) = state.session.lock() else {
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
    if let Ok(store) = state.settings.lock() {
        if let Ok(mut map) = store.load_resume_positions() {
            map.insert(path, (pos, time_ms));
            let _ = store.save_resume_positions(&map);
        }
    }
}
