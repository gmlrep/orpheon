//! Parse preference display strings (`"10 s"`, `"125%"`).

/// Parse `"10 s"` / `"3"` into seconds.
#[must_use]
#[allow(dead_code)] // paired with FE `parseSeconds`; ready for skip prefs on the backend
pub fn parse_seconds(raw: &str, fallback: f32) -> f32 {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return fallback;
    }
    let body = trimmed
        .strip_suffix("seconds")
        .or_else(|| trimmed.strip_suffix("second"))
        .or_else(|| trimmed.strip_suffix("secs"))
        .or_else(|| trimmed.strip_suffix("sec"))
        .or_else(|| trimmed.strip_suffix('s'))
        .unwrap_or(trimmed)
        .trim();
    body.parse::<f32>().unwrap_or(fallback)
}

/// Parse `"125%"` / `"80"` into a percent number.
#[must_use]
pub fn parse_percent(raw: &str, fallback: i32) -> i32 {
    let trimmed = raw.trim().trim_end_matches('%').trim();
    if trimmed.is_empty() {
        return fallback;
    }
    trimmed
        .parse::<f32>()
        .map(|n| n.round() as i32)
        .unwrap_or(fallback)
}
