//! Parse preference display strings (`"10 s"`, `"125%"`).

/// Parse `"10 s"` / `"10s"` / `"10 seconds"` / `"3"` into seconds.
#[must_use]
pub fn parse_seconds(raw: &str, fallback: f32) -> f32 {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return fallback;
    }
    let lower = trimmed.to_ascii_lowercase();
    let body = lower
        .strip_suffix("seconds")
        .or_else(|| lower.strip_suffix("second"))
        .or_else(|| lower.strip_suffix("secs"))
        .or_else(|| lower.strip_suffix("sec"))
        .or_else(|| lower.strip_suffix('s'))
        .unwrap_or(lower.as_str())
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
