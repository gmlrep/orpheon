//! Window / video surface layout commands.

use crate::error::CommandError;
use crate::state::{AppState, StageRect};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn update_stage_rect(state: State<'_, AppState>, rect: StageRect) -> Result<(), CommandError> {
    *state.lock_stage()? = rect;

    #[cfg(all(feature = "vlc", windows))]
    {
        let scale = if rect.scale_factor <= 0.0 {
            1.0
        } else {
            rect.scale_factor
        };
        let x = (rect.x as f64 * scale).round() as i32;
        let y = (rect.y as f64 * scale).round() as i32;
        let w = (rect.width as f64 * scale).round() as i32;
        let h = (rect.height as f64 * scale).round() as i32;

        if let Ok(mut slot) = state.lock_video() {
            if let Some(embed) = slot.as_mut() {
                if rect.visible && w > 2 && h > 2 {
                    embed.set_rect(x, y, w, h);
                    embed.set_visible(true);
                } else {
                    embed.set_visible(false);
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn set_overlay_holes(
    state: State<'_, AppState>,
    holes: Vec<[i32; 5]>,
) -> Result<(), CommandError> {
    #[cfg(all(feature = "vlc", windows))]
    {
        let rects: Vec<(i32, i32, i32, i32, i32)> = holes
            .into_iter()
            .map(|h| (h[0], h[1], h[2], h[3], h[4]))
            .collect();
        if let Ok(mut slot) = state.lock_video() {
            if let Some(embed) = slot.as_mut() {
                embed.set_overlay_excludes(&rects);
            }
        }
    }
    #[cfg(not(all(feature = "vlc", windows)))]
    {
        let _ = (state, holes);
    }
    Ok(())
}

#[tauri::command]
pub fn ensure_video_surface(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<bool, CommandError> {
    #[cfg(all(feature = "vlc", windows))]
    {
        use crate::video_embed::{parent_hwnd_from_tauri, VideoEmbed};

        let window = app
            .get_webview_window("main")
            .ok_or_else(|| CommandError::msg("main window missing"))?;
        let parent = parent_hwnd_from_tauri(&window)
            .ok_or_else(|| CommandError::msg("could not resolve parent HWND"))?;

        let mut slot = state.lock_video()?;
        if slot.is_none() {
            let embed = VideoEmbed::create(parent)
                .ok_or_else(|| CommandError::msg("failed to create video HWND"))?;
            // Bind libVLC to the surface.
            let hwnd = embed.hwnd_void();
            let _ = state.player.with_mut(|p| p.set_hwnd(hwnd));
            *slot = Some(embed);
        } else if let Some(embed) = slot.as_ref() {
            let hwnd = embed.hwnd_void();
            let _ = state.player.with_mut(|p| p.set_hwnd(hwnd));
        }
        Ok(true)
    }
    #[cfg(not(all(feature = "vlc", windows)))]
    {
        let _ = (app, state);
        Ok(false)
    }
}

#[tauri::command]
pub fn hide_video_surface(state: State<'_, AppState>) -> Result<(), CommandError> {
    #[cfg(all(feature = "vlc", windows))]
    {
        if let Ok(mut slot) = state.lock_video() {
            if let Some(embed) = slot.as_mut() {
                embed.set_visible(false);
                embed.set_overlay_excludes(&[]);
            }
        }
    }
    let _ = state;
    Ok(())
}

/// Whether the OS cursor is inside the bottom `strip_css_px` of the video stage.
/// Used for fullscreen chrome reveal without punching HWND region holes (which
/// would clip video paint and shift letterboxing).
#[tauri::command]
pub fn cursor_in_stage_bottom(
    state: State<'_, AppState>,
    strip_css_px: i32,
) -> Result<bool, CommandError> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::POINT;
        use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

        let stage = state.lock_stage()?;
        if !stage.visible || stage.height < 2 || stage.width < 2 {
            return Ok(false);
        }
        let scale = if stage.scale_factor <= 0.0 {
            1.0
        } else {
            stage.scale_factor
        };
        let mut pt = POINT { x: 0, y: 0 };
        unsafe {
            if GetCursorPos(&mut pt) == 0 {
                return Ok(false);
            }
        }
        // Stage is stored in CSS pixels; convert cursor (physical) → CSS.
        let cx = pt.x as f64 / scale;
        let cy = pt.y as f64 / scale;
        // Stage x/y are window-client CSS coords; cursor is screen. Approximate
        // using the window client origin via WebView parent is heavy — instead
        // compare against last applied screen rect from the embed when available.
        #[cfg(all(feature = "vlc", windows))]
        {
            if let Ok(slot) = state.lock_video() {
                if let Some(embed) = slot.as_ref() {
                    let (sx, sy, sw, sh) = embed.screen_rect();
                    if sw < 2 || sh < 2 {
                        return Ok(false);
                    }
                    let strip = ((strip_css_px as f64) * scale).round() as i32;
                    let in_x = pt.x >= sx && pt.x < sx + sw;
                    let in_y = pt.y >= sy + sh - strip.max(1) && pt.y < sy + sh;
                    return Ok(in_x && in_y);
                }
            }
        }
        let _ = (cx, cy, strip_css_px);
        Ok(false)
    }
    #[cfg(not(windows))]
    {
        let _ = (state, strip_css_px);
        Ok(false)
    }
}
