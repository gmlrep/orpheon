//! Orpheon Tauri backend — Vue UI ↔ libVLC / library / settings.

mod artwork;
mod commands;
mod constants;
mod player_handle;
mod poller;
mod state;
mod pref_units;

#[cfg(feature = "vlc")]
mod player_service;

#[cfg(feature = "vlc")]
mod prefs_apply;

#[cfg(all(feature = "vlc", windows))]
mod video_embed;

use player_handle::PlayerHandle;
use poller::start_playback_poller;
use state::AppState;
use std::time::Instant;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let t_setup = Instant::now();
            let data_dir = directories::ProjectDirs::from("com", "orpheon", "Orpheon")
                .map(|d| d.data_dir().to_path_buf())
                .unwrap_or_else(|| std::env::temp_dir().join("orpheon"));

            // Plugin path for libVLC (DLLs themselves must sit next to the exe —
            // see build.rs). Set before spawning engine init.
            #[cfg(all(feature = "vlc", windows))]
            configure_vlc_runtime();

            // Defer libvlc_new off the UI thread — synchronous init freezes the
            // window ("Not Responding") for the duration of plugin discovery.
            #[cfg(feature = "vlc")]
            let player = PlayerHandle::new(None);
            #[cfg(not(feature = "vlc"))]
            let player = PlayerHandle::empty();

            let t_db = Instant::now();
            let state = AppState::new(data_dir, player.clone())?;
            log::info!("AppState (sqlite) ready in {:?}", t_db.elapsed());
            app.manage(state);

            start_playback_poller(app.handle().clone());

            #[cfg(feature = "vlc")]
            {
                let handle = app.handle().clone();
                let player_bg = player.clone();
                let settings_snapshot = {
                    let state = app.state::<AppState>();
                    state.load_settings()
                };
                std::thread::Builder::new()
                    .name("vlc-init".into())
                    .spawn(move || {
                        let t = Instant::now();
                        let hw = prefs_apply::hw_accel_from_settings(&settings_snapshot);
                        let extra = prefs_apply::extra_vlc_args(&settings_snapshot);
                        match player_service::PlayerService::with_options(hw, &extra) {
                            Ok(svc) => {
                                if player_bg.install(svc).is_err() {
                                    log::error!("libVLC ready but player lock poisoned");
                                } else {
                                    log::info!(
                                        "libVLC engine ready in {:?} (hw={hw:?})",
                                        t.elapsed()
                                    );
                                }
                                let _ = handle.emit("engine://ready", true);
                            }
                            Err(e) => {
                                log::error!(
                                    "libVLC init failed after {:?}: {e}",
                                    t.elapsed()
                                );
                                let _ = handle.emit("engine://ready", false);
                            }
                        }
                    })
                    .map_err(|e| {
                        anyhow::anyhow!("failed to spawn vlc-init thread: {e}")
                    })?;
            }

            log::info!("tauri setup returned in {:?}", t_setup.elapsed());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::engine_status,
            commands::play_path,
            commands::play_mrl,
            commands::play_pause,
            commands::stop_playback,
            commands::seek,
            commands::seek_relative,
            commands::set_volume,
            commands::track_snapshot,
            commands::set_track,
            commands::apply_equalizer,
            commands::list_media,
            commands::list_recent_media,
            commands::list_folders,
            commands::add_library_folder,
            commands::rescan_library,
            commands::list_playlists,
            commands::create_playlist,
            commands::delete_playlist,
            commands::rename_playlist,
            commands::playlist_items,
            commands::add_to_playlist,
            commands::continue_watching,
            commands::touch_recent,
            commands::get_settings,
            commands::save_settings,
            commands::set_theme,
            commands::set_language,
            commands::clear_history,
            commands::update_stage_rect,
            commands::set_overlay_holes,
            commands::ensure_video_surface,
            commands::hide_video_surface,
            commands::cursor_in_stage_bottom,
            commands::resolve_artwork,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(all(feature = "vlc", windows))]
fn configure_vlc_runtime() {
    // DLLs are copied next to the exe by build.rs. Here we only set VLC_PLUGIN_PATH
    // (and PATH as a fallback for system / vendor installs).
    let candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_default(),
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vendor/vlc"),
        std::path::PathBuf::from(crate::constants::VLC_SYSTEM_DIR),
    ];
    for dir in candidates {
        let dir = dir.canonicalize().unwrap_or(dir);
        let plugins = dir.join("plugins");
        if plugins.is_dir() {
            std::env::set_var("VLC_PLUGIN_PATH", &plugins);
            if let Ok(old) = std::env::var("PATH") {
                let joined = format!("{};{}", dir.display(), old);
                std::env::set_var("PATH", joined);
            }
            log::info!("libVLC runtime dir: {}", dir.display());
            break;
        }
    }
}
