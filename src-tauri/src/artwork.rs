//! Cover art / video frame previews for library thumbnails.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use log::debug;

/// Resolve or generate a thumbnail for `media_path` under `cache_dir`.
pub fn ensure_artwork(media_path: &Path, cache_dir: &Path, media_id: i64) -> Option<PathBuf> {
    if let Some(sidecar) = find_sidecar_cover(media_path) {
        return Some(sidecar);
    }

    let _ = fs::create_dir_all(cache_dir);
    let cached_jpg = cache_dir.join(format!("{media_id}.jpg"));
    let cached_png = cache_dir.join(format!("{media_id}.png"));
    for candidate in [&cached_jpg, &cached_png] {
        if candidate.is_file() {
            if let Ok(meta) = candidate.metadata() {
                if meta.len() > 32 {
                    return Some(candidate.clone());
                }
            }
        }
    }

    if is_audio(media_path) {
        if try_ffmpeg_embedded_cover(media_path, &cached_jpg) {
            return Some(cached_jpg);
        }
        if try_ffmpeg_embedded_cover(media_path, &cached_png) {
            return Some(cached_png);
        }
        return None;
    }

    if try_ffmpeg_frame(media_path, &cached_jpg) {
        return Some(cached_jpg);
    }
    if try_vlc_cli_frame(media_path, cache_dir, media_id) {
        for candidate in [&cached_jpg, &cached_png] {
            if candidate.is_file() {
                if let Ok(meta) = candidate.metadata() {
                    if meta.len() > 32 {
                        return Some(candidate.clone());
                    }
                }
            }
        }
    }
    None
}

fn is_audio(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref(),
        Some("mp3" | "flac" | "wav" | "aac" | "ogg" | "m4a" | "wma" | "opus" | "aiff")
    )
}

fn find_sidecar_cover(media_path: &Path) -> Option<PathBuf> {
    let dir = media_path.parent()?;
    // Prefer same-stem art first so a shared cover.jpg is not forced onto every file.
    if let Some(stem) = media_path.file_stem().and_then(|s| s.to_str()) {
        for ext in ["jpg", "jpeg", "png", "webp"] {
            let p = dir.join(format!("{stem}.{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
    }
    const NAMES: &[&str] = &[
        "cover.jpg",
        "cover.jpeg",
        "cover.png",
        "folder.jpg",
        "folder.png",
        "AlbumArt.jpg",
        "AlbumArt.png",
        "front.jpg",
        "Front.jpg",
    ];
    for name in NAMES {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn find_ffmpeg() -> Option<PathBuf> {
    if let Ok(output) = Command::new("where").arg("ffmpeg").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let first = text.lines().next().unwrap_or("").trim();
            let p = PathBuf::from(first);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn find_vlc_exe() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    // Vendored / next-to-exe copies (build.rs puts DLLs here; some installs also ship vlc.exe).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("vlc.exe"));
        }
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../vendor/vlc/vlc.exe"),
    );
    if let Ok(pf) = std::env::var("ProgramFiles") {
        candidates.push(PathBuf::from(pf).join("VideoLAN").join("VLC").join("vlc.exe"));
    }
    if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
        candidates.push(PathBuf::from(pf86).join("VideoLAN").join("VLC").join("vlc.exe"));
    }
    candidates.into_iter().find(|p| p.is_file())
}

fn try_ffmpeg_frame(media_path: &Path, out_jpg: &Path) -> bool {
    let Some(ffmpeg) = find_ffmpeg() else {
        return false;
    };
    let status = Command::new(&ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-ss", "5", "-i"])
        .arg(media_path)
        .args(["-frames:v", "1", "-q:v", "4", "-y"])
        .arg(out_jpg)
        .status();
    matches!(status, Ok(s) if s.success() && out_jpg.is_file() && file_nonempty(out_jpg))
}

/// Extract embedded album art (attached pic / cover stream) from audio files.
fn try_ffmpeg_embedded_cover(media_path: &Path, out: &Path) -> bool {
    let Some(ffmpeg) = find_ffmpeg() else {
        return false;
    };
    let status = Command::new(&ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(media_path)
        .args(["-an", "-vcodec", "copy", "-y"])
        .arg(out)
        .status();
    matches!(status, Ok(s) if s.success() && out.is_file() && file_nonempty(out))
}

fn file_nonempty(p: &Path) -> bool {
    fs::metadata(p).map(|m| m.len() > 32).unwrap_or(false)
}

fn try_vlc_cli_frame(media_path: &Path, cache_dir: &Path, media_id: i64) -> bool {
    let Some(vlc) = find_vlc_exe() else {
        debug!("vlc.exe not found for thumbnails");
        return false;
    };
    let scene_dir = cache_dir.join(format!("vlc-scene-{media_id}"));
    let _ = fs::remove_dir_all(&scene_dir);
    if fs::create_dir_all(&scene_dir).is_err() {
        return false;
    }
    // Note: `--vout=dummy` prevents the scene filter from writing frames.
    // Prefer `--run-time` over `--stop-time` (more reliable with -I dummy).
    // Longer files often start on black/title cards — sample further in.
    let start = if media_path
        .metadata()
        .map(|m| m.len() > 80_000_000)
        .unwrap_or(false)
    {
        "--start-time=90"
    } else {
        "--start-time=1"
    };
    let status = Command::new(&vlc)
        .args([
            "-I",
            "dummy",
            "--quiet",
            "--no-video-title-show",
            "--no-audio",
            "--no-osd",
            "--play-and-exit",
            start,
            "--run-time=1",
            "--video-filter=scene",
            "--scene-format=jpg",
            "--scene-ratio=1",
            "--scene-prefix",
            "shot",
            "--scene-replace",
        ])
        .arg(format!("--scene-path={}", scene_dir.display()))
        .arg(media_path)
        .arg("vlc://quit")
        .status();
    let _ = status;
    // Prefer the last (usually sharpest mid-run) scene frame.
    let mut found: Option<PathBuf> = None;
    if let Ok(rd) = fs::read_dir(&scene_dir) {
        let mut jpgs: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jpg"))
            .collect();
        jpgs.sort();
        found = jpgs.pop();
    }
    let Some(src) = found else {
        let _ = fs::remove_dir_all(&scene_dir);
        return false;
    };
    let dest = cache_dir.join(format!("{media_id}.jpg"));
    let copied = fs::copy(&src, &dest).is_ok();
    let _ = fs::remove_dir_all(&scene_dir);
    copied && dest.is_file() && file_nonempty(&dest)
}
