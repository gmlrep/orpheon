//! Cloneable handle around the optional playback engine.

#[cfg(feature = "vlc")]
use crate::player_service::PlayerService;
use std::sync::{Arc, Mutex, MutexGuard};

/// Shared optional [`PlayerService`] (`None` = UI-only / engine unavailable).
#[derive(Clone, Default)]
pub struct PlayerHandle {
    #[cfg(feature = "vlc")]
    inner: Arc<Mutex<Option<PlayerService>>>,
}

impl PlayerHandle {
    /// Wrap an optional engine.
    #[cfg(feature = "vlc")]
    #[must_use]
    pub fn new(inner: Option<PlayerService>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// UI-only handle with no engine.
    #[cfg(not(feature = "vlc"))]
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Lock the inner mutex.
    #[cfg(feature = "vlc")]
    #[allow(dead_code)]
    pub fn lock(&self) -> Result<MutexGuard<'_, Option<PlayerService>>, ()> {
        self.inner.lock().map_err(|_| ())
    }

    /// Run `f` when the engine is available.
    #[cfg(feature = "vlc")]
    pub fn with<R>(&self, f: impl FnOnce(&PlayerService) -> R) -> Option<R> {
        let Ok(guard) = self.inner.lock() else {
            return None;
        };
        guard.as_ref().map(f)
    }

    /// Run `f` mutably when the engine is available.
    #[cfg(feature = "vlc")]
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut PlayerService) -> R) -> Option<R> {
        let Ok(mut guard) = self.inner.lock() else {
            return None;
        };
        guard.as_mut().map(f)
    }

    /// Whether a live engine is present.
    #[must_use]
    pub fn available(&self) -> bool {
        #[cfg(feature = "vlc")]
        {
            self.with(|_| ()).is_some()
        }
        #[cfg(not(feature = "vlc"))]
        {
            false
        }
    }
}
