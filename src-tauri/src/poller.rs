//! Periodic playback status emitter (~30 Hz).

use crate::constants::{
    END_MIN_LENGTH_MS, END_POSITION_THRESHOLD, END_TIME_SLACK_MS, PLAYBACK_POLL_INTERVAL,
    RESUME_PERSIST_INTERVAL,
};
use crate::emit_util::emit_logged;
use crate::state::{AppState, PlaybackTick, SessionInfo};
use tauri::{AppHandle, Manager};

#[cfg(feature = "vlc")]
fn apply_stream_meta(session: &mut SessionInfo, meta: &libvlc::MediaMetaSnapshot) -> bool {
    let looks_network = session.path.contains("://");
    if !looks_network {
        return false;
    }

    let mut changed = false;
    let display = meta
        .now_playing
        .clone()
        .or_else(|| meta.title.clone())
        .filter(|t| !t.is_empty());
    if let Some(title) = display {
        // Prefer live now-playing; otherwise upgrade placeholder URL titles.
        if (meta.now_playing.is_some()
            || session.title == session.path
            || session.title.is_empty()
            || session.title == "Network stream")
            && session.title != title
        {
            session.title = title;
            changed = true;
        }
    }
    if let Some(artist) = meta.artist.clone().filter(|a| !a.is_empty()) {
        if session.subtitle != artist {
            session.subtitle = artist;
            changed = true;
        }
    }
    if let Some(art) = crate::mrl::sanitize_artwork_url(meta.artwork_url.as_deref()) {
        if session.artwork_url.as_deref() != Some(art.as_str()) {
            session.artwork_url = Some(art);
            changed = true;
        }
    }
    changed
}

/// Spawn a background thread that emits `playback://tick` events.
pub fn start_playback_poller(app: AppHandle) {
    if let Err(e) = std::thread::Builder::new()
        .name("playback-poller".into())
        .spawn(move || poll_loop(app))
    {
        log::error!("failed to spawn playback-poller thread: {e}");
    }
}

fn poll_loop(app: AppHandle) {
    let mut last_persist = std::time::Instant::now();
    let mut was_playing = false;
    let mut duration_saved_for: Option<i64> = None;
    loop {
        std::thread::sleep(PLAYBACK_POLL_INTERVAL);

        let Some(state) = app.try_state::<AppState>() else {
            continue;
        };

        #[cfg(feature = "vlc")]
        {
            #[cfg(windows)]
            {
                handle_video_hotkeys(&app, &state);
            }

            let snapshot = state
                .player
                .with(|p| (p.is_playing(), p.position(), p.time_ms(), p.length_ms()));

            if let Some((playing, position, time_ms, length_ms)) = snapshot {
                // Apply deferred resume once length is known.
                if length_ms > 0 {
                    if let Ok(mut pending) = state.lock_pending_seek() {
                        if let Some(pos) = pending.take() {
                            let _ = state.player.with(|p| p.seek(pos));
                        }
                    }
                }

                // Persist probed duration into the library DB once per media.
                if length_ms > 0 {
                    let media_id = state.lock_session().ok().and_then(|s| s.media_id);
                    if let Some(mid) = media_id {
                        if duration_saved_for != Some(mid) {
                            if let Ok(db) = state.lock_library() {
                                let _ = db.set_duration_ms(mid, length_ms);
                            }
                            duration_saved_for = Some(mid);
                        }
                    }
                }

                let ended = was_playing
                    && !playing
                    && length_ms > END_MIN_LENGTH_MS
                    && (position >= END_POSITION_THRESHOLD
                        || time_ms + END_TIME_SLACK_MS >= length_ms);

                // Pull ICY / stream metadata when libVLC exposes it.
                let meta = state.player.with(|p| p.media_meta());

                // Update session under lock, then emit/persist after releasing it.
                let mut tick_payload: Option<PlaybackTick> = None;
                let mut enrich: Option<(String, String, Option<String>)> = None;
                if let Ok(mut s) = state.lock_session() {
                    if s.active {
                        s.paused = !playing;
                        s.position = position;
                        // libVLC returns -1 while length is unknown — keep prior/seeded value.
                        if time_ms >= 0 {
                            s.time_ms = time_ms;
                        }
                        if length_ms > 0 {
                            s.length_ms = length_ms;
                        }
                        if let Some(ref meta) = meta {
                            if apply_stream_meta(&mut s, meta) {
                                enrich =
                                    Some((s.path.clone(), s.title.clone(), s.artwork_url.clone()));
                            }
                        }
                        tick_payload = Some(PlaybackTick {
                            session: s.clone(),
                            playing,
                        });
                    }
                }

                if let Some((mrl, title, art)) = enrich {
                    crate::commands::streams::enrich_saved_stream_meta(
                        &state,
                        &mrl,
                        &title,
                        art.as_deref(),
                    );
                }
                if let Some(tick) = tick_payload {
                    emit_logged(&app, "playback://tick", tick);
                }

                if ended {
                    emit_logged(&app, "playback://ended", ());
                }

                was_playing = playing;

                if last_persist.elapsed() > RESUME_PERSIST_INTERVAL {
                    crate::commands::persist_resume(&state);
                    last_persist = std::time::Instant::now();
                }
            }
        }

        #[cfg(not(feature = "vlc"))]
        {
            let _ = (
                &state,
                &last_persist,
                &mut was_playing,
                &mut duration_saved_for,
            );
        }
    }
}

#[cfg(all(feature = "vlc", windows))]
fn handle_video_hotkeys(app: &AppHandle, state: &AppState) {
    use crate::commands::persist_resume;
    use crate::video_embed::{VideoEmbed, VideoHotkey};

    let media_only = state.load_settings().prefs.hotkey_profile == "Media keys";

    if VideoEmbed::take_space_pause() {
        let _ = state.player.with_mut(|p| p.play_pause());
    }
    match VideoEmbed::take_hotkey() {
        VideoHotkey::PlayPause => {
            let _ = state.player.with_mut(|p| p.play_pause());
        }
        VideoHotkey::Stop if !media_only => {
            persist_resume(state);
            let _ = state.player.with(|p| p.stop());
            if let Ok(mut s) = state.lock_session() {
                *s = Default::default();
            }
            emit_logged(app, "playback://stopped", ());
        }
        VideoHotkey::ToggleFullscreen => {
            if let Some(win) = app.get_webview_window("main") {
                let fs = win.is_fullscreen().unwrap_or(false);
                let _ = win.set_fullscreen(!fs);
            }
        }
        VideoHotkey::Escape => {
            if let Some(win) = app.get_webview_window("main") {
                if win.is_fullscreen().unwrap_or(false) {
                    let _ = win.set_fullscreen(false);
                }
            }
        }
        VideoHotkey::Next => {
            emit_logged(app, "playback://hotkey", "next");
        }
        VideoHotkey::Previous => {
            emit_logged(app, "playback://hotkey", "previous");
        }
        VideoHotkey::Miniplayer if !media_only => {
            emit_logged(app, "playback://hotkey", "minimize");
        }
        VideoHotkey::ToggleUiHidden if !media_only => {
            emit_logged(app, "playback://hotkey", "toggleUi");
        }
        _ => {}
    }
    if VideoEmbed::take_double_click() {
        if let Some(win) = app.get_webview_window("main") {
            let fs = win.is_fullscreen().unwrap_or(false);
            let _ = win.set_fullscreen(!fs);
        }
    }
}
