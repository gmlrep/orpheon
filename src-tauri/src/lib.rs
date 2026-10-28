//! Orpheon Tauri backend — Vue UI ↔ libVLC / library / settings.

mod artwork;
mod commands;
mod player_handle;
mod poller;
mod state;

#[cfg(feature = "vlc")]
mod player_service;

#[cfg(all(feature = "vlc", windows))]
mod video_embed;

use player_handle::PlayerHandle;
use poller::start_playback_poller;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let data_dir = directories::ProjectDirs::from("com", "orpheon", "Orpheon")
                .map(|d| d.data_dir().to_path_buf())
                .unwrap_or_else(|| std::env::temp_dir().join("orpheon"));

            // Plugin path for libVLC (DLLs themselves must sit next to the exe —
            // see build.rs). Set this before creating the player.
            #[cfg(all(feature = "vlc", windows))]
            configure_vlc_runtime();

            let player = init_player();
            let state = AppState::new(data_dir, player)?;
            app.manage(state);

            start_playback_poller(app.handle().clone());
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

fn init_player() -> PlayerHandle {
    #[cfg(feature = "vlc")]
    {
        match player_service::PlayerService::new() {
            Ok(svc) => {
                log::info!("libVLC engine ready");
                PlayerHandle::new(Some(svc))
            }
            Err(e) => {
                log::error!("libVLC init failed: {e}");
                PlayerHandle::new(None)
            }
        }
    }
    #[cfg(not(feature = "vlc"))]
    {
        PlayerHandle::empty()
    }
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
        std::path::PathBuf::from(r"C:\Program Files\VideoLAN\VLC"),
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
