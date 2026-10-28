//! Settings persistence commands.

use crate::state::AppState;
use settings::{AppSettings, ThemePreference};
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let store = state
        .settings
        .lock()
        .map_err(|_| "settings lock".to_string())?;
    store.load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    let store = state
        .settings
        .lock()
        .map_err(|_| "settings lock".to_string())?;
    store.save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_theme(state: State<'_, AppState>, theme: String) -> Result<(), String> {
    let store = state
        .settings
        .lock()
        .map_err(|_| "settings lock".to_string())?;
    let mut s = store.load().map_err(|e| e.to_string())?;
    s.theme = match theme.to_ascii_lowercase().as_str() {
        "light" => ThemePreference::Light,
        "dark" => ThemePreference::Dark,
        _ => ThemePreference::System,
    };
    store.save(&s).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_language(state: State<'_, AppState>, language: String) -> Result<(), String> {
    let store = state
        .settings
        .lock()
        .map_err(|_| "settings lock".to_string())?;
    let mut s = store.load().map_err(|e| e.to_string())?;
    s.language = language;
    store.save(&s).map_err(|e| e.to_string())
}
