//! Periodic playback status emitter (~30 Hz).

use crate::state::{AppState, PlaybackTick};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Spawn a background thread that emits `playback://tick` events.
pub fn start_playback_poller(app: AppHandle) {
    std::thread::Builder::new()
        .name("playback-poller".into())
        .spawn(move || poll_loop(app))
        .ok();
}

fn poll_loop(app: AppHandle) {
    let mut last_persist = std::time::Instant::now();
    let mut was_playing = false;
    let mut duration_saved_for: Option<i64> = None;
    loop {
        std::thread::sleep(Duration::from_millis(33));

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
                    if let Ok(mut pending) = state.pending_seek.lock() {
                        if let Some(pos) = pending.take() {
                            let _ = state.player.with(|p| p.seek(pos));
                        }
                    }
                }

                // Persist probed duration into the library DB once per media.
                if length_ms > 0 {
                    if let Ok(s) = state.session.lock() {
                        if let Some(mid) = s.media_id {
                            if duration_saved_for != Some(mid) {
                                if let Ok(db) = state.library.lock() {
                                    let _ = db.set_duration_ms(mid, length_ms);
                                }
                                duration_saved_for = Some(mid);
                            }
                        }
                    }
                }

                let ended = was_playing
                    && !playing
                    && length_ms > 1000
                    && (position >= 0.98 || time_ms + 250 >= length_ms);

                if let Ok(mut s) = state.session.lock() {
                    if s.active {
                        s.paused = !playing;
                        s.position = position;
                        s.time_ms = time_ms;
                        s.length_ms = length_ms;
                        let tick = PlaybackTick {
                            session: s.clone(),
                            playing,
                        };
                        let _ = app.emit("playback://tick", tick);
                    }
                }

                if ended {
                    let _ = app.emit("playback://ended", ());
                }

                was_playing = playing;

                if last_persist.elapsed() > Duration::from_secs(5) {
                    crate::commands::persist_resume(&state);
                    last_persist = std::time::Instant::now();
                }
            }
        }

        #[cfg(not(feature = "vlc"))]
        {
            let _ = (&state, &last_persist, &mut was_playing, &mut duration_saved_for);
        }
    }
}

#[cfg(all(feature = "vlc", windows))]
fn handle_video_hotkeys(app: &AppHandle, state: &AppState) {
    use crate::commands::persist_resume;
    use crate::video_embed::{VideoEmbed, VideoHotkey};

    if VideoEmbed::take_space_pause() {
        let _ = state.player.with_mut(|p| p.play_pause());
    }
    match VideoEmbed::take_hotkey() {
        VideoHotkey::PlayPause => {
            let _ = state.player.with_mut(|p| p.play_pause());
        }
        VideoHotkey::Stop => {
            persist_resume(state);
            let _ = state.player.with(|p| p.stop());
            if let Ok(mut s) = state.session.lock() {
                *s = Default::default();
            }
            let _ = app.emit("playback://stopped", ());
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
            let _ = app.emit("playback://hotkey", "next");
        }
        VideoHotkey::Previous => {
            let _ = app.emit("playback://hotkey", "previous");
        }
        VideoHotkey::Miniplayer => {
            let _ = app.emit("playback://hotkey", "minimize");
        }
        VideoHotkey::ToggleUiHidden => {
            let _ = app.emit("playback://hotkey", "toggleUi");
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
