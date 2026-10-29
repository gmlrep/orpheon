//! Native Win32 video surface for libVLC (`set_hwnd`) — hardware path, full resolution.
//!
//! Owned `WS_POPUP` so WebView2 cannot cover Direct3D. Parent Tauri window is
//! subclassed to track moves during the Win32 modal drag loop.

#![cfg(windows)]

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Mutex, Once, OnceLock};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    ClientToScreen, CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject, ScreenToClient,
    SetWindowRgn, RGN_DIFF,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, SetFocus, VK_CONTROL, VK_ESCAPE, VK_F11, VK_SPACE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DestroyWindow, GetClassInfoExW, GetCursorPos,
    GetWindowLongPtrW, LoadCursorW, RegisterClassExW, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, GWLP_WNDPROC, HTCLIENT, HWND_TOP, IDC_ARROW,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SW_HIDE, SW_SHOW, WM_DESTROY, WM_EXITSIZEMOVE,
    WM_KEYDOWN, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_MOVE, WM_MOVING, WM_NCHITTEST, WM_SIZE,
    WNDCLASSEXW, WS_CLIPSIBLINGS, WS_POPUP,
};

const CLASS: &[u16] = &[
    b'V' as u16,
    b'l' as u16,
    b'c' as u16,
    b'P' as u16,
    b'l' as u16,
    b'a' as u16,
    b'y' as u16,
    b'V' as u16,
    b'i' as u16,
    b'd' as u16,
    b'e' as u16,
    b'o' as u16,
    0,
];

static REGISTER: Once = Once::new();
static DBL_CLICK: AtomicBool = AtomicBool::new(false);
/// Space pressed on the video HWND (or forwarded) — polled by the UI timer.
static SPACE_PAUSE: AtomicBool = AtomicBool::new(false);
/// Hotkey action id from video HWND (see [`VideoHotkey`]) — polled by the UI timer.
static PENDING_HOTKEY: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Actions captured when the Direct3D video HWND steals keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum VideoHotkey {
    None = 0,
    PlayPause = 1,
    Stop = 2,
    Next = 3,
    Previous = 4,
    ToggleFullscreen = 5,
    ToggleUiHidden = 6,
    ShowTime = 7,
    OpenFile = 8,
    OpenFolder = 9,
    Settings = 10,
    Quit = 11,
    Escape = 12,
    Miniplayer = 13,
    NetworkStream = 14,
}

impl VideoHotkey {
    fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::PlayPause,
            2 => Self::Stop,
            3 => Self::Next,
            4 => Self::Previous,
            5 => Self::ToggleFullscreen,
            6 => Self::ToggleUiHidden,
            7 => Self::ShowTime,
            8 => Self::OpenFile,
            9 => Self::OpenFolder,
            10 => Self::Settings,
            11 => Self::Quit,
            12 => Self::Escape,
            13 => Self::Miniplayer,
            14 => Self::NetworkStream,
            _ => Self::None,
        }
    }
}
/// Prevents re-entrant SetWindowPos → message → subclass deadlocks.
static IN_SYNC: AtomicBool = AtomicBool::new(false);
/// Previous parent WndProc — atomic so we never need the mutex to forward messages.
static OLD_PROC: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());
/// Parent Tauri HWND — so the video surface can return focus / forward keys.
static PARENT_HWND: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());

type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

struct ParentHook {
    parent: HWND,
    video: HWND,
    /// Client-space rect inside parent `(x, y, w, h)`.
    client: (i32, i32, i32, i32),
    visible: bool,
}

// HWND pointers are only touched on the UI thread; the Mutex serializes access.
unsafe impl Send for ParentHook {}

fn hook_slot() -> &'static Mutex<Option<ParentHook>> {
    static SLOT: OnceLock<Mutex<Option<ParentHook>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn apply_screen_pos(parent: HWND, video: HWND, client: (i32, i32, i32, i32)) {
    let (x, y, w, h) = client;
    let w = w.max(1);
    let h = h.max(1);
    let mut pt = POINT { x, y };
    unsafe {
        ClientToScreen(parent, &mut pt);
        SetWindowPos(video, HWND_TOP, pt.x, pt.y, w, h, SWP_NOACTIVATE);
    }
}

fn forward_to_old(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ptr = OLD_PROC.load(Ordering::Acquire);
    let old: WndProc = if ptr.is_null() {
        DefWindowProcW
    } else {
        // SAFETY: stored from GetWindowLongPtrW as the previous WndProc.
        unsafe { std::mem::transmute::<*mut (), WndProc>(ptr) }
    };
    unsafe { CallWindowProcW(Some(old), hwnd, msg, wparam, lparam) }
}

unsafe extern "system" fn parent_subclass(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let should_sync = matches!(msg, WM_MOVE | WM_MOVING | WM_SIZE | WM_EXITSIZEMOVE);
    if should_sync && !IN_SYNC.load(Ordering::Relaxed) {
        // Copy under lock, release before any Win32 call that can re-enter.
        let snapshot = hook_slot().lock().ok().and_then(|guard| {
            guard.as_ref().and_then(|h| {
                if h.parent == hwnd && h.visible {
                    Some((h.parent, h.video, h.client))
                } else {
                    None
                }
            })
        });
        if let Some((parent, video, client)) = snapshot {
            IN_SYNC.store(true, Ordering::Relaxed);
            apply_screen_pos(parent, video, client);
            IN_SYNC.store(false, Ordering::Relaxed);
        }
    }

    forward_to_old(hwnd, msg, wparam, lparam)
}

/// Owned popup HWND that libVLC renders into via Direct3D.
pub struct VideoEmbed {
    hwnd: HWND,
    parent: HWND,
    /// Last applied screen rect `(x, y, w, h)` — for timer-path dedup.
    last_screen: (i32, i32, i32, i32),
    client: (i32, i32, i32, i32),
    visible: bool,
    /// Holes in video-client coords `(x, y, w, h, radius)` so WebView chrome composites above video.
    excludes: Vec<(i32, i32, i32, i32, i32)>,
}

// HWND pointers are only touched under the AppState mutex / UI path.
unsafe impl Send for VideoEmbed {}
unsafe impl Sync for VideoEmbed {}

impl VideoEmbed {
    /// Create an owned popup tied to the Tauri top-level HWND.
    pub fn create(parent: HWND) -> Option<Self> {
        REGISTER.call_once(|| unsafe {
            let mut info = std::mem::zeroed::<WNDCLASSEXW>();
            info.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
            if GetClassInfoExW(std::ptr::null_mut(), CLASS.as_ptr(), &mut info) == 0 {
                let mut wc = std::mem::zeroed::<WNDCLASSEXW>();
                wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
                wc.style = CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS;
                wc.lpfnWndProc = Some(wnd_proc);
                wc.hCursor = LoadCursorW(std::ptr::null_mut(), IDC_ARROW);
                wc.lpszClassName = CLASS.as_ptr();
                RegisterClassExW(&wc);
            }
        });

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                CLASS.as_ptr(),
                std::ptr::null(),
                WS_POPUP | WS_CLIPSIBLINGS,
                0,
                0,
                1,
                1,
                parent,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        if hwnd.is_null() {
            return None;
        }

        // IMPORTANT: publish old WndProc + hook state BEFORE replacing GWLP_WNDPROC,
        // otherwise in-flight messages would call DefWindowProc and freeze the UI.
        let old_ptr = unsafe { GetWindowLongPtrW(parent, GWLP_WNDPROC) };
        if old_ptr == 0 {
            unsafe {
                DestroyWindow(hwnd);
            }
            return None;
        }
        OLD_PROC.store(old_ptr as *mut (), Ordering::Release);
        PARENT_HWND.store(parent as *mut (), Ordering::Release);
        *hook_slot()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(ParentHook {
            parent,
            video: hwnd,
            client: (0, 0, 1, 1),
            visible: false,
        });
        unsafe {
            SetWindowLongPtrW(parent, GWLP_WNDPROC, parent_subclass as *const () as isize);
        }

        Some(Self {
            hwnd,
            parent,
            last_screen: (0, 0, 0, 0),
            client: (0, 0, 1, 1),
            visible: false,
            excludes: Vec::new(),
        })
    }

    /// HWND as `*mut c_void` for `libvlc_media_player_set_hwnd`.
    #[must_use]
    pub fn hwnd_void(&self) -> *mut std::os::raw::c_void {
        self.hwnd as *mut std::os::raw::c_void
    }

    /// Last applied screen rect `(x, y, w, h)` in physical pixels.
    #[must_use]
    pub fn screen_rect(&self) -> (i32, i32, i32, i32) {
        self.last_screen
    }

    /// Punch holes so Slint overlays (controls bar, side panels) composite above the video.
    ///
    /// Each rect is `(x, y, w, h, radius)` in **video client** physical pixels.
    /// Empty slice restores a full rectangular video surface. Clicks in holes fall through.
    pub fn set_overlay_excludes(&mut self, rects: &[(i32, i32, i32, i32, i32)]) {
        if self.excludes == rects {
            return;
        }
        self.excludes.clear();
        self.excludes.extend_from_slice(rects);
        self.apply_window_rgn();
    }

    fn apply_window_rgn(&self) {
        let vw = self.client.2.max(1);
        let vh = self.client.3.max(1);
        unsafe {
            if self.excludes.is_empty() {
                SetWindowRgn(self.hwnd, std::ptr::null_mut(), 1);
                return;
            }
            let result = CreateRectRgn(0, 0, vw, vh);
            let mut any = false;
            for &(x, y, w, h, radius) in &self.excludes {
                if w <= 0 || h <= 0 {
                    continue;
                }
                let x2 = (x + w).clamp(0, vw);
                let y2 = (y + h).clamp(0, vh);
                let x1 = x.clamp(0, vw);
                let y1 = y.clamp(0, vh);
                if x2 <= x1 || y2 <= y1 {
                    continue;
                }
                let dia = (radius * 2).max(0);
                let hole = if dia > 0 {
                    CreateRoundRectRgn(x1, y1, x2, y2, dia, dia)
                } else {
                    CreateRectRgn(x1, y1, x2, y2)
                };
                CombineRgn(result, result, hole, RGN_DIFF);
                DeleteObject(hole);
                any = true;
            }
            if !any {
                DeleteObject(result);
                SetWindowRgn(self.hwnd, std::ptr::null_mut(), 1);
                return;
            }
            // System takes ownership of `result`.
            SetWindowRgn(self.hwnd, result, 1);
        }
    }

    /// Move/resize using parent **client** coordinates (physical pixels).
    pub fn set_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let w = w.max(1);
        let h = h.max(1);
        let size_changed = self.client.2 != w || self.client.3 != h;
        self.client = (x, y, w, h);
        if let Ok(mut guard) = hook_slot().lock() {
            if let Some(h) = guard.as_mut() {
                if h.video == self.hwnd {
                    h.client = self.client;
                }
            }
        }

        let mut pt = POINT { x, y };
        unsafe {
            ClientToScreen(self.parent, &mut pt);
        }
        let next = (pt.x, pt.y, w, h);
        if next == self.last_screen && self.visible {
            if size_changed {
                self.apply_window_rgn();
            }
            return;
        }
        self.last_screen = next;
        if IN_SYNC.load(Ordering::Relaxed) {
            return;
        }
        IN_SYNC.store(true, Ordering::Relaxed);
        unsafe {
            SetWindowPos(self.hwnd, HWND_TOP, pt.x, pt.y, w, h, SWP_NOACTIVATE);
        }
        IN_SYNC.store(false, Ordering::Relaxed);
        self.apply_window_rgn();
    }

    pub fn set_visible(&mut self, show: bool) {
        if self.visible == show {
            return;
        }
        self.visible = show;
        if let Ok(mut guard) = hook_slot().lock() {
            if let Some(h) = guard.as_mut() {
                if h.video == self.hwnd {
                    h.visible = show;
                }
            }
        }
        unsafe {
            ShowWindow(self.hwnd, if show { SW_SHOW } else { SW_HIDE });
            if show {
                SetWindowPos(
                    self.hwnd,
                    HWND_TOP,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                );
                IN_SYNC.store(true, Ordering::Relaxed);
                apply_screen_pos(self.parent, self.hwnd, self.client);
                IN_SYNC.store(false, Ordering::Relaxed);
            }
        }
    }

    /// Consume a double-click that happened on the video surface.
    pub fn take_double_click() -> bool {
        DBL_CLICK.swap(false, Ordering::SeqCst)
    }

    /// Consume a Space key that landed on the video HWND (focus-steal case).
    pub fn take_space_pause() -> bool {
        SPACE_PAUSE.swap(false, Ordering::SeqCst)
    }

    /// Consume a hotkey that landed on the video HWND (focus-steal case).
    pub fn take_hotkey() -> VideoHotkey {
        VideoHotkey::from_u32(PENDING_HOTKEY.swap(0, Ordering::SeqCst))
    }
}

fn ctrl_down() -> bool {
    unsafe { GetKeyState(VK_CONTROL as i32) < 0 }
}

fn map_video_hotkey(vk: u32) -> VideoHotkey {
    let ctrl = ctrl_down();
    match vk {
        x if x == VK_SPACE as u32 => VideoHotkey::PlayPause,
        x if x == VK_ESCAPE as u32 => VideoHotkey::Escape,
        x if x == VK_F11 as u32 => VideoHotkey::ToggleFullscreen,
        // Letter VKs are uppercase ASCII.
        0x46 if !ctrl => VideoHotkey::ToggleFullscreen, // F
        0x49 if !ctrl => VideoHotkey::ToggleUiHidden,   // I
        0x54 if !ctrl => VideoHotkey::ShowTime,         // T
        0x53 if !ctrl => VideoHotkey::Stop,             // S
        0x4E if !ctrl => VideoHotkey::Next,             // N
        0x50 if !ctrl => VideoHotkey::Previous,         // P
        0x4F if ctrl => VideoHotkey::OpenFile,          // Ctrl+O
        0x46 if ctrl => VideoHotkey::OpenFolder,        // Ctrl+F
        0x50 if ctrl => VideoHotkey::Settings,          // Ctrl+P
        0x51 if ctrl => VideoHotkey::Quit,              // Ctrl+Q
        0x4D if ctrl => VideoHotkey::Miniplayer,        // Ctrl+M
        0x4E if ctrl => VideoHotkey::NetworkStream,     // Ctrl+N
        _ => VideoHotkey::None,
    }
}

impl Drop for VideoEmbed {
    fn drop(&mut self) {
        if let Ok(mut guard) = hook_slot().lock() {
            if guard.take().is_some() {
                let old = OLD_PROC.swap(std::ptr::null_mut(), Ordering::AcqRel);
                if !old.is_null() {
                    unsafe {
                        SetWindowLongPtrW(self.parent, GWLP_WNDPROC, old as isize);
                    }
                }
            }
        }
        if !self.hwnd.is_null() {
            unsafe {
                DestroyWindow(self.hwnd);
            }
            self.hwnd = std::ptr::null_mut();
        }
    }
}

/// Extract the Win32 HWND from a Tauri webview window.
pub fn parent_hwnd_from_tauri(window: &tauri::WebviewWindow) -> Option<HWND> {
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get() as HWND),
        _ => None,
    }
}

/// Cursor position in `parent` client coordinates (physical pixels), or `None` if unavailable.
#[must_use]
#[allow(dead_code)]
pub fn cursor_in_parent(parent: HWND) -> Option<(i32, i32)> {
    if parent.is_null() {
        return None;
    }
    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut pt) == 0 {
            return None;
        }
        if ScreenToClient(parent, &mut pt) == 0 {
            return None;
        }
    }
    Some((pt.x, pt.y))
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_LBUTTONDBLCLK => {
            DBL_CLICK.store(true, Ordering::SeqCst);
            0
        }
        WM_LBUTTONDOWN => {
            // Return keyboard focus to the Slint window so Space keeps working.
            let parent = PARENT_HWND.load(Ordering::Acquire) as HWND;
            if !parent.is_null() {
                unsafe { SetFocus(parent) };
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_KEYDOWN => {
            let vk = wparam as u32;
            let action = map_video_hotkey(vk);
            if action == VideoHotkey::PlayPause {
                SPACE_PAUSE.store(true, Ordering::SeqCst);
            } else if action != VideoHotkey::None {
                PENDING_HOTKEY.store(action as u32, Ordering::SeqCst);
            }
            // Return focus so subsequent keys can reach Slint FocusScope.
            let parent = PARENT_HWND.load(Ordering::Acquire) as HWND;
            if !parent.is_null() {
                unsafe { SetFocus(parent) };
            }
            0
        }
        WM_DESTROY => 0,
        WM_NCHITTEST => HTCLIENT as LRESULT,
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
