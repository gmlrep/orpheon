//! Settings persistence commands.

use crate::error::CommandError;
use crate::state::AppState;
use settings::{AppSettings, ThemePreference};
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    Ok(state.lock_settings()?.load()?)
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), CommandError> {
    {
        let store = state.lock_settings()?;
        store.save(&settings)?;
    }

    // Live-apply session prefs when media is already open (picture, rate, delays…).
    #[cfg(feature = "vlc")]
    {
        let _ = state.player.with(|p| {
            crate::prefs_apply::apply_session_prefs(p, &settings.prefs);
        });
    }

    Ok(())
}

#[tauri::command]
pub fn set_theme(state: State<'_, AppState>, theme: String) -> Result<(), CommandError> {
    let store = state.lock_settings()?;
    let mut s = store.load()?;
    s.theme = match theme.to_ascii_lowercase().as_str() {
        "light" => ThemePreference::Light,
        "dark" => ThemePreference::Dark,
        _ => ThemePreference::System,
    };
    Ok(store.save(&s)?)
}

#[tauri::command]
pub fn set_language(state: State<'_, AppState>, language: String) -> Result<(), CommandError> {
    let store = state.lock_settings()?;
    let mut s = store.load()?;
    s.language = language;
    Ok(store.save(&s)?)
}

/// Clear recently played + resume positions (Privacy → Clear history).
#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), CommandError> {
    {
        let db = state.lock_library()?;
        db.clear_recent()?;
    }
    {
        let store = state.lock_settings()?;
        store.save_resume_positions(&std::collections::HashMap::new())?;
    }
    Ok(())
}
