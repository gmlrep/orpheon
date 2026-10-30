//! Orpheon Tauri backend — Vue UI ↔ libVLC / library / settings.

mod artwork;
mod commands;
mod constants;
mod emit_util;
mod error;
mod mrl;
mod path_guard;
mod player_handle;
mod poller;
mod pref_units;
mod process_util;
mod state;
mod time_util;

#[cfg(feature = "vlc")]
mod player_service;

#[cfg(feature = "vlc")]
mod prefs_apply;

#[cfg(all(feature = "vlc", windows))]
mod video_embed;

use emit_util::emit_logged;
use player_handle::PlayerHandle;
use poller::start_playback_poller;
use state::AppState;
use std::path::PathBuf;
use std::time::Instant;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let t_setup = Instant::now();
            let data_dir = resolve_data_dir(app.handle())?;
            log::info!("data dir: {}", data_dir.display());

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
                                emit_logged(&handle, "engine://ready", true);
                            }
                            Err(e) => {
                                log::error!("libVLC init failed after {:?}: {e}", t.elapsed());
                                emit_logged(&handle, "engine://ready", false);
                            }
                        }
                    })
                    .map_err(|e| anyhow::anyhow!("failed to spawn vlc-init thread: {e}"))?;
            }

            log::info!("tauri setup returned in {:?}", t_setup.elapsed());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::engine_status,
            commands::play_path,
            commands::play_mrl,
            commands::list_saved_streams,
            commands::save_stream,
            commands::remove_saved_stream,
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
            commands::default_media_folders,
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

/// Resolve app data dir via Tauri path API; migrate legacy ProjectDirs layout once.
fn resolve_data_dir(app: &tauri::AppHandle) -> anyhow::Result<PathBuf> {
    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    // Legacy: directories::ProjectDirs::from("com", "orpheon", "Orpheon")
    // → %APPDATA%\orpheon\Orpheon on Windows.
    let legacy = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|p| p.join("orpheon").join("Orpheon"))
        .filter(|p| p.join("library.sqlite").is_file());

    if let Some(legacy) = legacy {
        let new_db = data_dir.join("library.sqlite");
        if !new_db.is_file() {
            log::info!(
                "migrating data from {} → {}",
                legacy.display(),
                data_dir.display()
            );
            for name in [
                "library.sqlite",
                "library.sqlite-wal",
                "library.sqlite-shm",
                "settings.sqlite",
                "settings.sqlite-wal",
                "settings.sqlite-shm",
            ] {
                let src = legacy.join(name);
                if src.is_file() {
                    let _ = std::fs::copy(&src, data_dir.join(name));
                }
            }
            let legacy_art = legacy.join("artwork");
            let new_art = data_dir.join("artwork");
            if legacy_art.is_dir() {
                let _ = copy_dir_best_effort(&legacy_art, &new_art);
            }
        }
    }

    Ok(data_dir)
}

fn copy_dir_best_effort(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_best_effort(&entry.path(), &to)?;
        } else if entry.file_type()?.is_file() && !to.exists() {
            let _ = std::fs::copy(entry.path(), &to);
        }
    }
    Ok(())
}

#[cfg(all(feature = "vlc", windows))]
fn configure_vlc_runtime() {
    // DLLs are copied next to the exe by build.rs. Here we only set VLC_PLUGIN_PATH
    // (and PATH as a fallback for system / vendor installs).
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.to_path_buf());
        }
    }
    // Dev-only vendor tree next to the crate (not baked into release path search).
    #[cfg(debug_assertions)]
    {
        candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vendor/vlc"));
    }
    candidates.push(PathBuf::from(crate::constants::VLC_SYSTEM_DIR));

    for dir in candidates {
        let dir = dir.canonicalize().unwrap_or(dir);
        // Avoid `\\?\…` in env vars — libVLC plugin loader rejects them.
        let dir = strip_verbatim_prefix(dir);
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

fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{rest}"));
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path
}
