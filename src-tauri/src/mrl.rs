//! Network MRL / artwork URL validation (no libVLC dependency).

const ALLOWED_MRL_SCHEMES: &[&str] = &[
    "http",
    "https",
    "rtsp",
    "rtp",
    "mms",
    "mmsh",
    "udp",
    "tcp",
    "srt",
    "dvd",
    "cdda",
    "vcd",
    "dshow",
    "v4l2",
    "qtsound",
    "avcapture",
];

/// Validate a network MRL before handing it to libVLC or saving a bookmark.
///
/// # Errors
/// Empty input, control characters, missing/unknown scheme, or `file:`.
pub fn validate_network_mrl(mrl: &str) -> Result<&str, String> {
    let mrl = mrl.trim();
    if mrl.is_empty() {
        return Err("empty stream URL".into());
    }
    if mrl.chars().any(|c| c.is_control()) {
        return Err("stream URL contains control characters".into());
    }
    let Some((scheme, _rest)) = mrl.split_once(':') else {
        return Err("stream URL must include a scheme (e.g. https://…)".into());
    };
    let scheme = scheme.to_ascii_lowercase();
    if scheme == "file" {
        return Err("file: URLs are not allowed here — use Open File".into());
    }
    if !ALLOWED_MRL_SCHEMES.iter().any(|s| *s == scheme) {
        return Err(format!(
            "unsupported URL scheme '{scheme}' (allowed: {})",
            ALLOWED_MRL_SCHEMES.join(", ")
        ));
    }
    Ok(mrl)
}

/// Allow only remote `http` / `https` artwork URLs (reject `data:`, `file:`, local paths).
#[must_use]
pub fn sanitize_artwork_url(raw: Option<&str>) -> Option<String> {
    let url = raw?.trim();
    if url.is_empty() {
        return None;
    }
    let (scheme, rest) = url.split_once(':')?;
    let scheme = scheme.to_ascii_lowercase();
    if (scheme == "http" || scheme == "https") && rest.starts_with("//") {
        Some(url.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https_mrl() {
        assert!(validate_network_mrl("https://example.com/a.m3u8").is_ok());
    }

    #[test]
    fn rejects_file_mrl() {
        assert!(validate_network_mrl("file:///tmp/x").is_err());
    }

    #[test]
    fn artwork_https_ok_data_rejected() {
        assert_eq!(
            sanitize_artwork_url(Some("https://cdn/x.png")).as_deref(),
            Some("https://cdn/x.png")
        );
        assert_eq!(sanitize_artwork_url(Some("data:image/png;base64,xx")), None);
        assert_eq!(sanitize_artwork_url(Some("C:\\art.png")), None);
        assert_eq!(sanitize_artwork_url(Some("file:///C:/a.png")), None);
    }
}
