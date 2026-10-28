//! Double-buffered RGBA frame state shared with libVLC callbacks.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

/// Absolute safety ceiling (8K). Normal 4K/1080p keep native resolution.
pub const MAX_SOFT_WIDTH: u32 = 7680;
/// Absolute safety ceiling height.
pub const MAX_SOFT_HEIGHT: u32 = 4320;

/// Owned pixel buffer for one video frame (RGBA8, row-major, pitch-aligned).
#[derive(Clone, Debug)]
pub struct FrameBuffer {
    /// Pixel bytes (length >= pitch * lines).
    pub pixels: Vec<u8>,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Bytes per row (multiple of 32, as libVLC prefers).
    pub pitch: u32,
    /// Allocated scanlines (multiple of 32).
    pub lines: u32,
}

impl FrameBuffer {
    /// Allocate a black frame with 32-byte pitch/lines alignment.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let pitch = align32(width.saturating_mul(4));
        let lines = align32(height);
        let size = (pitch as usize).saturating_mul(lines as usize);
        Self {
            pixels: vec![0; size],
            width,
            height,
            pitch,
            lines,
        }
    }

    /// Grow in place when possible so existing plane pointers stay valid.
    ///
    /// Returns `false` if a reallocation would be required (caller must defer).
    fn try_ensure_dims(&mut self, width: u32, height: u32) -> bool {
        let width = width.max(1);
        let height = height.max(1);
        let pitch = align32(width.saturating_mul(4));
        let lines = align32(height);
        let need = (pitch as usize).saturating_mul(lines as usize);
        if self.pixels.capacity() < need {
            return false;
        }
        if self.pixels.len() < need {
            self.pixels.resize(need, 0);
        } else if self.width != width || self.height != height {
            // Clear only the active region when dimensions change.
            self.pixels[..need].fill(0);
        }
        self.width = width;
        self.height = height;
        self.pitch = pitch;
        self.lines = lines;
        true
    }

    fn force_ensure_dims(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        let pitch = align32(width.saturating_mul(4));
        let lines = align32(height);
        let need = (pitch as usize).saturating_mul(lines as usize);
        if self.pixels.capacity() < need {
            self.pixels = vec![0; need];
        } else {
            self.pixels.resize(need, 0);
            self.pixels.fill(0);
        }
        self.width = width;
        self.height = height;
        self.pitch = pitch;
        self.lines = lines;
    }
}

fn align32(v: u32) -> u32 {
    v.saturating_add(31) & !31
}

/// Scale `w×h` down to fit inside `max_w×max_h`, preserving aspect ratio.
#[must_use]
pub fn cap_dimensions(w: u32, h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    let w = w.max(1);
    let h = h.max(1);
    if w <= max_w && h <= max_h {
        return (w, h);
    }
    let scale = (max_w as f64 / w as f64).min(max_h as f64 / h as f64);
    let nw = ((w as f64) * scale).round().max(1.0) as u32;
    let nh = ((h as f64) * scale).round().max(1.0) as u32;
    (nw, nh)
}

/// Soft-blit target: keep source resolution (only clamp absurd >8K).
#[must_use]
pub fn soft_blit_cap(src_w: u32, src_h: u32) -> (u32, u32) {
    cap_dimensions(src_w.max(1), src_h.max(1), MAX_SOFT_WIDTH, MAX_SOFT_HEIGHT)
}

/// Double-buffered video frames for software rendering into Slint.
pub struct VideoFrameState {
    width: AtomicU32,
    height: AtomicU32,
    pitch: AtomicU32,
    lines: AtomicU32,
    buffer0: Mutex<FrameBuffer>,
    buffer1: Mutex<FrameBuffer>,
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
    generation: AtomicU64,
    /// True between libVLC `lock` and `unlock` — must not free write-plane bytes.
    planes_locked: AtomicBool,
    /// Deferred resize when `format_cb` races with an active lock.
    pending_w: AtomicU32,
    pending_h: AtomicU32,
    has_pending: AtomicBool,
}

impl VideoFrameState {
    /// Create double buffers at the given resolution.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let fb = FrameBuffer::new(width, height);
        Self {
            width: AtomicU32::new(fb.width),
            height: AtomicU32::new(fb.height),
            pitch: AtomicU32::new(fb.pitch),
            lines: AtomicU32::new(fb.lines),
            buffer0: Mutex::new(FrameBuffer::new(width, height)),
            buffer1: Mutex::new(FrameBuffer::new(width, height)),
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(1),
            generation: AtomicU64::new(0),
            planes_locked: AtomicBool::new(false),
            pending_w: AtomicU32::new(0),
            pending_h: AtomicU32::new(0),
            has_pending: AtomicBool::new(false),
        }
    }

    /// Configured pixel dimensions.
    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        (
            self.width.load(Ordering::Acquire),
            self.height.load(Ordering::Acquire),
        )
    }

    /// Pitch (bytes) and lines negotiated with libVLC.
    #[must_use]
    pub fn pitch_lines(&self) -> (u32, u32) {
        (
            self.pitch.load(Ordering::Acquire),
            self.lines.load(Ordering::Acquire),
        )
    }

    /// Ensure both planes match a new negotiated size without UAF on active locks.
    ///
    /// Prefers in-place grow (pointer-stable). If a reallocation is required while
    /// planes are locked, the resize is deferred until [`Self::unlock_write_plane`].
    pub fn resize(&self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        let (cw, ch) = self.dimensions();
        if cw == width && ch == height {
            let need = frame_byte_size(width, height);
            let ok0 = self
                .buffer0
                .lock()
                .map(|b| b.pixels.len() >= need)
                .unwrap_or(false);
            let ok1 = self
                .buffer1
                .lock()
                .map(|b| b.pixels.len() >= need)
                .unwrap_or(false);
            if ok0 && ok1 {
                return;
            }
        }

        if self.planes_locked.load(Ordering::Acquire) {
            // Never free the plane VLC is writing into.
            if self.try_grow_both(width, height) {
                self.publish_dims(width, height);
                self.has_pending.store(false, Ordering::Release);
            } else {
                self.pending_w.store(width, Ordering::Release);
                self.pending_h.store(height, Ordering::Release);
                self.has_pending.store(true, Ordering::Release);
            }
            return;
        }

        self.apply_resize(width, height);
        self.has_pending.store(false, Ordering::Release);
    }

    /// Called from libVLC `cleanup` — flush any deferred resize while quiescent.
    pub fn on_format_cleanup(&self) {
        self.planes_locked.store(false, Ordering::Release);
        self.flush_pending_resize();
    }

    fn flush_pending_resize(&self) {
        if !self.has_pending.swap(false, Ordering::AcqRel) {
            return;
        }
        let w = self.pending_w.load(Ordering::Acquire).max(1);
        let h = self.pending_h.load(Ordering::Acquire).max(1);
        self.apply_resize(w, h);
    }

    fn try_grow_both(&self, width: u32, height: u32) -> bool {
        let mut b0 = self
            .buffer0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut b1 = self
            .buffer1
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        b0.try_ensure_dims(width, height) && b1.try_ensure_dims(width, height)
    }

    fn apply_resize(&self, width: u32, height: u32) {
        let mut b0 = self
            .buffer0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut b1 = self
            .buffer1
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !b0.try_ensure_dims(width, height) {
            b0.force_ensure_dims(width, height);
        }
        if !b1.try_ensure_dims(width, height) {
            b1.force_ensure_dims(width, height);
        }
        self.publish_dims(b0.width, b0.height);
    }

    fn publish_dims(&self, width: u32, height: u32) {
        let pitch = align32(width.saturating_mul(4));
        let lines = align32(height);
        self.width.store(width, Ordering::Release);
        self.height.store(height, Ordering::Release);
        self.pitch.store(pitch, Ordering::Release);
        self.lines.store(lines, Ordering::Release);
    }

    /// Monotonic frame counter.
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    fn buffer(&self, idx: usize) -> &Mutex<FrameBuffer> {
        if idx & 1 == 0 {
            &self.buffer0
        } else {
            &self.buffer1
        }
    }

    /// Pointer to the writable plane for the lock callback.
    pub fn lock_write_plane(&self) -> *mut u8 {
        self.planes_locked.store(true, Ordering::Release);
        let idx = self.write_idx.load(Ordering::Acquire);
        let mut buf = self
            .buffer(idx)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        buf.pixels.as_mut_ptr()
    }

    /// Mark the write plane unlocked and apply any deferred resize.
    pub fn unlock_write_plane(&self) {
        self.planes_locked.store(false, Ordering::Release);
        self.flush_pending_resize();
    }

    /// Swap write/read indices after a frame is displayed.
    pub fn swap_buffers(&self) {
        let write = self.write_idx.load(Ordering::Acquire);
        let read = self.read_idx.load(Ordering::Acquire);
        self.write_idx.store(read, Ordering::Release);
        self.read_idx.store(write, Ordering::Release);
        self.generation.fetch_add(1, Ordering::Release);
    }

    /// Copy front buffer into tightly packed RGBA `dst` if generation advanced.
    ///
    /// Frames are negotiated as RGBA from libVLC, so this is a pitch→tight memcpy
    /// (no channel swap). Uses `try_lock` so a busy UI drops the frame.
    pub fn copy_front_rgba_if_newer(
        &self,
        last_gen: u64,
        dst: &mut Vec<u8>,
    ) -> Option<(u64, u32, u32)> {
        let gen = self.generation();
        if gen == 0 || gen == last_gen {
            return None;
        }
        // Always present the newest frame (skip intermediates if UI fell behind).
        let idx = self.read_idx.load(Ordering::Acquire);
        let buf = self.buffer(idx).try_lock().ok()?;
        let w = buf.width;
        let h = buf.height;
        let pitch = buf.pitch as usize;
        let row_bytes = (w as usize).saturating_mul(4);
        let need = row_bytes.saturating_mul(h as usize);
        dst.resize(need, 0);
        if pitch == row_bytes {
            dst.copy_from_slice(&buf.pixels[..need]);
        } else {
            for y in 0..h as usize {
                let src = &buf.pixels[y * pitch..y * pitch + row_bytes];
                dst[y * row_bytes..(y + 1) * row_bytes].copy_from_slice(src);
            }
        }
        Some((gen, w, h))
    }

    /// Snapshot front pixels if newer (legacy clone path).
    #[must_use]
    pub fn take_front_if_newer(&self, last_gen: u64) -> Option<(u64, u32, u32, Vec<u8>)> {
        let mut rgba = Vec::new();
        let meta = self.copy_front_rgba_if_newer(last_gen, &mut rgba)?;
        Some((meta.0, meta.1, meta.2, rgba))
    }

    /// Read a clone of the front buffer.
    #[must_use]
    pub fn front_frame(&self) -> FrameBuffer {
        let idx = self.read_idx.load(Ordering::Acquire);
        self.buffer(idx)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Access the front buffer under lock.
    pub fn with_front<R>(&self, f: impl FnOnce(&FrameBuffer) -> R) -> R {
        let idx = self.read_idx.load(Ordering::Acquire);
        let buf = self
            .buffer(idx)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&buf)
    }
}

fn frame_byte_size(width: u32, height: u32) -> usize {
    let pitch = align32(width.max(1).saturating_mul(4)) as usize;
    let lines = align32(height.max(1)) as usize;
    pitch.saturating_mul(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_buffer_should_align_pitch() {
        let frame = FrameBuffer::new(100, 50);
        assert_eq!(frame.pitch % 32, 0);
        assert_eq!(frame.lines % 32, 0);
        assert!(frame.pixels.len() >= (frame.pitch * frame.lines) as usize);
    }

    #[test]
    fn soft_blit_cap_should_keep_4k_native() {
        assert_eq!(soft_blit_cap(3840, 2160), (3840, 2160));
        assert_eq!(soft_blit_cap(1920, 1080), (1920, 1080));
    }

    #[test]
    fn soft_blit_cap_should_clamp_above_8k() {
        let (w, h) = soft_blit_cap(15360, 8640);
        assert!(w <= MAX_SOFT_WIDTH && h <= MAX_SOFT_HEIGHT);
    }

    #[test]
    fn swap_buffers_should_exchange_indices() {
        let state = VideoFrameState::new(4, 4);
        let before_write = state.write_idx.load(Ordering::SeqCst);
        let before_read = state.read_idx.load(Ordering::SeqCst);
        state.swap_buffers();
        assert_eq!(state.write_idx.load(Ordering::SeqCst), before_read);
        assert_eq!(state.read_idx.load(Ordering::SeqCst), before_write);
    }

    #[test]
    fn resize_while_locked_should_not_free_plane() {
        let state = VideoFrameState::new(64, 64);
        let ptr = state.lock_write_plane();
        assert!(!ptr.is_null());
        // Grow within existing capacity — pointer must stay valid.
        state.resize(32, 32);
        let ptr2 = {
            let idx = state.write_idx.load(Ordering::SeqCst);
            let buf = state
                .buffer(idx)
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            buf.pixels.as_ptr()
        };
        assert_eq!(ptr as usize, ptr2 as usize);
        state.unlock_write_plane();
    }

    #[test]
    fn try_ensure_dims_grows_in_place_when_capacity_allows() {
        let mut fb = FrameBuffer::new(256, 256);
        let ptr = fb.pixels.as_ptr();
        assert!(fb.try_ensure_dims(128, 128));
        assert_eq!(fb.pixels.as_ptr(), ptr);
        assert_eq!((fb.width, fb.height), (128, 128));
    }
}
