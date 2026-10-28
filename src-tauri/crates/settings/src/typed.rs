//! Typed views over stringy preference fields (Primitive Obsession antidote).

use serde::{Deserialize, Serialize};

/// `resize_to_video` preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResizeToVideo {
    /// Interface follows video size.
    #[default]
    #[serde(rename = "Always")]
    Always,
    /// Keep current window size.
    #[serde(rename = "Never")]
    Never,
    /// Only when larger than the UI.
    #[serde(rename = "When larger")]
    WhenLarger,
}

impl ResizeToVideo {
    /// Parse a prefs string (unknown → [`Self::Always`]).
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s {
            "Never" => Self::Never,
            "When larger" => Self::WhenLarger,
            _ => Self::Always,
        }
    }

    /// Canonical prefs string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "Always",
            Self::Never => "Never",
            Self::WhenLarger => "When larger",
        }
    }
}

/// `continue_mode` preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ContinueMode {
    /// Ask / continue when sensible.
    #[default]
    #[serde(rename = "Ask")]
    Ask,
    /// Always resume.
    #[serde(rename = "Always")]
    Always,
    /// Never resume.
    #[serde(rename = "Never")]
    Never,
}

impl ContinueMode {
    /// Parse a prefs string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s {
            "Always" => Self::Always,
            "Never" => Self::Never,
            _ => Self::Ask,
        }
    }

    /// Canonical prefs string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ask => "Ask",
            Self::Always => "Always",
            Self::Never => "Never",
        }
    }
}

/// `deinterlace` preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeinterlaceMode {
    /// Automatic.
    #[default]
    #[serde(rename = "Automatic")]
    Automatic,
    /// Off.
    #[serde(rename = "Off")]
    Off,
    /// On.
    #[serde(rename = "On")]
    On,
}

impl DeinterlaceMode {
    /// Parse a prefs string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s {
            "Off" => Self::Off,
            "On" => Self::On,
            _ => Self::Automatic,
        }
    }

    /// Canonical prefs string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "Automatic",
            Self::Off => "Off",
            Self::On => "On",
        }
    }
}

/// Hardware acceleration mode labels used in prefs UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HwAccelModePref {
    /// Automatic / any.
    #[default]
    #[serde(rename = "Automatic")]
    Automatic,
    /// Disabled.
    #[serde(rename = "Disabled")]
    Disabled,
    /// D3D11.
    #[serde(rename = "D3D11")]
    D3D11,
    /// DXVA2.
    #[serde(rename = "DXVA2")]
    Dxva2,
}

impl HwAccelModePref {
    /// Parse prefs UI string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "Disabled" => Self::Disabled,
            "D3D11" => Self::D3D11,
            "DXVA2" => Self::Dxva2,
            _ => Self::Automatic,
        }
    }

    /// Canonical prefs string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "Automatic",
            Self::Disabled => "Disabled",
            Self::D3D11 => "D3D11",
            Self::Dxva2 => "DXVA2",
        }
    }
}
