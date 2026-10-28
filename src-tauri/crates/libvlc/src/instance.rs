//! libVLC instance ownership.

use crate::{LibVlcError, Result};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

/// Hardware video decode preference mapped to `--avcodec-hw=…`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwAccelMode {
    /// Let libVLC pick (`any`).
    Automatic,
    /// Force software decode.
    Disabled,
    /// Direct3D 11 VA (Windows).
    D3D11,
    /// DXVA2 (Windows).
    Dxva2,
}

impl HwAccelMode {
    /// Parse UI / prefs labels (`Automatic`, `Disabled`, `D3D11`, `DXVA2`).
    ///
    /// Unknown values fall back to [`Self::Automatic`] when `legacy_enabled`, else
    /// [`Self::Disabled`].
    #[must_use]
    pub fn parse(mode: &str, legacy_enabled: bool) -> Self {
        match mode.trim() {
            "Disabled" => Self::Disabled,
            "D3D11" => Self::D3D11,
            "DXVA2" => Self::Dxva2,
            "Automatic" => Self::Automatic,
            "" => {
                if legacy_enabled {
                    Self::Automatic
                } else {
                    Self::Disabled
                }
            }
            _ => {
                if legacy_enabled {
                    Self::Automatic
                } else {
                    Self::Disabled
                }
            }
        }
    }

    /// libVLC CLI argument for this mode.
    #[must_use]
    pub const fn avcodec_arg(self) -> &'static str {
        match self {
            Self::Automatic => "--avcodec-hw=any",
            Self::Disabled => "--avcodec-hw=none",
            Self::D3D11 => "--avcodec-hw=d3d11va",
            Self::Dxva2 => "--avcodec-hw=dxva2",
        }
    }
}

/// A libVLC engine instance.
pub struct Instance {
    pub(crate) raw: *mut libvlc_sys::libvlc_instance_t,
}

// SAFETY: libVLC instance is documented as usable from multiple threads when
// each media player is used according to libVLC rules. We treat the raw pointer
// as Send+Sync for Arc sharing of the Instance handle.
unsafe impl Send for Instance {}
unsafe impl Sync for Instance {}

impl Instance {
    /// Create a libVLC instance tuned for embedded HWND / software rendering.
    ///
    /// # Errors
    /// Returns [`LibVlcError::InstanceCreate`] if libVLC cannot be initialized.
    pub fn new() -> Result<Self> {
        Self::for_playback(HwAccelMode::Automatic, &[])
    }

    /// Create a playback instance with hardware-decode mode and extra args.
    ///
    /// On Windows, prefers Direct3D11 video output for HWND embedding. Other
    /// platforms omit `--vout` so libVLC picks a native module (or software
    /// callbacks set by the player).
    ///
    /// # Errors
    /// Returns [`LibVlcError::InstanceCreate`] if libVLC cannot be initialized.
    pub fn for_playback(hw: HwAccelMode, extra: &[String]) -> Result<Self> {
        let mut args: Vec<&str> = vec![
            "--intf=dummy",
            "--no-video-title-show",
            "--no-snapshot-preview",
            "--quiet",
            hw.avcodec_arg(),
        ];
        #[cfg(windows)]
        {
            // HWND Direct3D path — omit on other platforms (software/callback vout).
            args.push("--vout=direct3d11");
        }
        args.extend_from_slice(&[
            "--drop-late-frames",
            "--skip-frames",
            "--no-keyboard-events",
            "--no-mouse-events",
        ]);
        for opt in extra {
            args.push(opt.as_str());
        }
        Self::with_args(&args)
    }

    /// Create a libVLC instance with custom arguments.
    ///
    /// # Errors
    /// Returns [`LibVlcError::InstanceCreate`] if libVLC cannot be initialized.
    pub fn with_args(args: &[&str]) -> Result<Self> {
        let c_args: Vec<CString> = args
            .iter()
            .map(|s| CString::new(*s).map_err(|_| LibVlcError::InstanceCreate))
            .collect::<std::result::Result<_, _>>()?;
        let mut ptrs: Vec<*const c_char> = c_args.iter().map(|s| s.as_ptr()).collect();
        // SAFETY: ptrs point into c_args which live until end of this call.
        let raw = unsafe {
            libvlc_sys::libvlc_new(ptrs.len() as i32, ptrs.as_mut_ptr() as *const *const c_char)
        };
        if raw.is_null() {
            return Err(LibVlcError::InstanceCreate);
        }
        Ok(Self { raw })
    }

    /// Enumerate audio output devices for the UI.
    ///
    /// Returns `(id, label)` where `id` is `""` for system default, or
    /// `"module::device"` for a real WASAPI endpoint.
    ///
    /// Policy:
    /// - At most one real endpoint → only `System default`.
    /// - Multiple connected endpoints → `System default` plus each device.
    /// - Dummy aout modules (`waveout`, `directsound`, …) are never listed alone.
    ///
    /// Note: vendor libVLC 3.0.24 returns corrupt `p_next` in
    /// `libvlc_audio_output_device_list_get` (points into string data → AV).
    /// We only accept device nodes whose `p_next` looks like a real heap node.
    #[must_use]
    pub fn audio_output_devices(&self) -> Vec<(String, String)> {
        let mut real: Vec<(String, String)> = Vec::new();

        // Prefer WASAPI (`mmdevice`); fall back to probing without aout filter.
        self.push_devices_for_aout(Some("mmdevice"), &mut real);
        if real.is_empty() {
            self.push_devices_for_aout(None, &mut real);
        }

        // Drop synthetic "Default" / empty-id entries — we add our own.
        real.retain(|(id, label)| {
            let id_l = id.to_ascii_lowercase();
            let lab_l = label.to_ascii_lowercase();
            !id.is_empty()
                && !id.ends_with("::")
                && !id_l.contains("default")
                && lab_l != "default"
                && lab_l != "system default"
                && looks_like_text(label)
        });

        let mut seen = std::collections::HashSet::new();
        real.retain(|(id, _)| seen.insert(id.clone()));

        if real.len() <= 1 {
            return vec![("".into(), "System default".into())];
        }

        let mut out = vec![("".into(), "System default".into())];
        out.extend(real);
        out
    }

    fn push_devices_for_aout(&self, aout: Option<&str>, out: &mut Vec<(String, String)>) {
        let aout_c = aout.and_then(|s| CString::new(s).ok());
        let head = unsafe {
            libvlc_sys::libvlc_audio_output_device_list_get(
                self.raw,
                aout_c.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            )
        };
        if head.is_null() {
            return;
        }

        let mut cur = head;
        let mut guard = 0_u32;
        while !cur.is_null() && guard < 32 {
            guard += 1;
            let id = unsafe { cstr_owned((*cur).psz_device).unwrap_or_default() };
            let label = unsafe {
                cstr_owned((*cur).psz_description).unwrap_or_else(|| {
                    if id.is_empty() {
                        "Audio device".into()
                    } else {
                        id.clone()
                    }
                })
            };
            // Skip nodes whose strings look like heap corruption / non-text.
            if !id.is_empty() && !looks_like_text(&id) && !looks_like_text(&label) {
                break;
            }
            let composed_id = match aout {
                Some(m) => format!("{m}::{id}"),
                None => id.clone(),
            };
            let composed_label = if looks_like_text(&label) {
                label
            } else {
                "Audio device".into()
            };
            if !composed_id.is_empty() {
                out.push((composed_id, composed_label));
            }

            let next = unsafe { (*cur).p_next };
            if next.is_null() {
                break;
            }
            // Vendor bug: p_next sometimes aliases psz_description / string data.
            let desc_ptr = unsafe { (*cur).psz_description as usize };
            let dev_ptr = unsafe { (*cur).psz_device as usize };
            let next_addr = next as usize;
            if next_addr == desc_ptr || next_addr == dev_ptr || !looks_like_heap_node(next) {
                break;
            }
            cur = next;
        }
        unsafe { libvlc_sys::libvlc_audio_output_device_list_release(head) };
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { libvlc_sys::libvlc_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

fn looks_like_text(s: &str) -> bool {
    !s.is_empty()
        && s.chars().count() < 256
        && s.chars()
            .all(|c| c.is_ascii_graphic() || c.is_whitespace() || (c as u32) >= 0xa0)
}

/// True if `node` looks like another `libvlc_audio_output_device_t` (not string bytes).
fn looks_like_heap_node(node: *mut libvlc_sys::libvlc_audio_output_device_t) -> bool {
    if node.is_null() {
        return false;
    }
    let addr = node as usize;
    // Reject obvious non-pointers (low, kernel-ish, tagged).
    if addr < 0x10000 || (addr >> 48) != 0 {
        return false;
    }
    // Peek purported string pointers; both null is ok, both bogus is not.
    let device = unsafe { (*node).psz_device as usize };
    let desc = unsafe { (*node).psz_description as usize };
    let ok_ptr = |p: usize| p == 0 || ((p >> 48) == 0 && p >= 0x10000);
    ok_ptr(device) && ok_ptr(desc)
}

fn cstr_owned(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // Guard against non-string pointers (AV when walking corrupt device lists).
    let addr = ptr as usize;
    if addr < 0x10000 || (addr >> 48) != 0 {
        return None;
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    let bytes = cstr.to_bytes();
    if bytes.len() >= 512 {
        return None;
    }
    Some(cstr.to_string_lossy().into_owned())
}
