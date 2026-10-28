//! Folder browsing helpers over scanned media paths.

use crate::models::{MediaItem, MediaType};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

/// A folder shown in the Videos / Music sidebar (Figma folder rows).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderEntry {
    /// Absolute folder path.
    pub path: String,
    /// Display name (last path segment).
    pub name: String,
}

/// Normalize for prefix checks (Windows-friendly).
fn norm(p: &Path) -> PathBuf {
    // Keep as-is; comparisons use starts_with on Path.
    p.to_path_buf()
}

fn matches_type(item: &MediaItem, filter: Option<MediaType>) -> bool {
    match filter {
        None => true,
        Some(MediaType::Video) => item.media_type == MediaType::Video,
        Some(MediaType::Audio) => item.media_type == MediaType::Audio,
        Some(other) => item.media_type == other,
    }
}

/// Scan-root folders that contain at least one matching media file.
#[must_use]
pub fn root_folders(
    roots: &[String],
    media: &[MediaItem],
    filter: Option<MediaType>,
) -> Vec<FolderEntry> {
    let mut out = Vec::new();
    for root in roots {
        let root_path = PathBuf::from(root);
        // Only roots that actually contain matching media for this section
        // (Videos vs Music stay separate — empty / wrong-type roots are hidden).
        let has = media
            .iter()
            .any(|m| matches_type(m, filter) && Path::new(&m.path).starts_with(&root_path));
        if has {
            let name = root_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(root)
                .to_owned();
            out.push(FolderEntry {
                path: root.clone(),
                name,
            });
        }
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

/// Immediate child folders of `parent` that contain matching media somewhere below.
#[must_use]
pub fn child_folders(
    parent: &Path,
    media: &[MediaItem],
    filter: Option<MediaType>,
) -> Vec<FolderEntry> {
    let parent = norm(parent);
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    for item in media {
        if !matches_type(item, filter) {
            continue;
        }
        let path = Path::new(&item.path);
        if !path.starts_with(&parent) {
            continue;
        }
        let Ok(rel) = path.strip_prefix(&parent) else {
            continue;
        };
        let mut comps = rel.components();
        // Skip the file itself — need at least dir + file
        let Some(Component::Normal(first)) = comps.next() else {
            continue;
        };
        // If only one component left after first, first is a file name in parent — skip
        if comps.next().is_none() {
            continue;
        }
        let child = parent.join(first);
        let key = child.to_string_lossy().to_string();
        let name = first.to_string_lossy().to_string();
        map.entry(key).or_insert(name);
    }

    // Disk read_dir skipped here — scanning every child against all media is too
    // slow for large libraries; DB-derived children above are enough for browse.

    let mut out: Vec<FolderEntry> = map
        .into_iter()
        .map(|(path, name)| FolderEntry { path, name })
        .collect();
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

/// Media files located directly inside `parent` (not in subfolders).
#[must_use]
pub fn direct_media(
    parent: &Path,
    media: &[MediaItem],
    filter: Option<MediaType>,
) -> Vec<MediaItem> {
    let parent = norm(parent);
    let mut out: Vec<MediaItem> = media
        .iter()
        .filter(|m| matches_type(m, filter))
        .filter(|m| {
            Path::new(&m.path)
                .parent()
                .is_some_and(|p| p == parent.as_path())
        })
        .cloned()
        .collect();
    out.sort_by_key(|a| a.title.to_lowercase());
    out
}

/// One clickable breadcrumb segment (Figma Frame 323).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreadcrumbPart {
    /// Display label for this segment.
    pub label: String,
    /// Absolute folder path to open; empty string = section root (clear browse).
    pub path: String,
}

/// Build clickable breadcrumb segments with paths for each level.
#[must_use]
pub fn breadcrumb_parts(section: &str, browse: &Path, roots: &[String]) -> Vec<BreadcrumbPart> {
    let mut parts: Vec<BreadcrumbPart> = vec![BreadcrumbPart {
        label: section.to_string(),
        path: String::new(),
    }];
    if browse.as_os_str().is_empty() {
        return parts;
    }
    let root = roots
        .iter()
        .map(PathBuf::from)
        .find(|r| browse.starts_with(r));
    if let Some(root) = root {
        let root_name = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Folder");
        parts.push(BreadcrumbPart {
            label: root_name.to_string(),
            path: root.to_string_lossy().to_string(),
        });
        let mut acc = root.clone();
        if let Ok(rel) = browse.strip_prefix(&root) {
            for c in rel.components() {
                if let Component::Normal(s) = c {
                    acc.push(s);
                    parts.push(BreadcrumbPart {
                        label: s.to_string_lossy().to_string(),
                        path: acc.to_string_lossy().to_string(),
                    });
                }
            }
        }
    } else if let Some(name) = browse.file_name().and_then(|s| s.to_str()) {
        parts.push(BreadcrumbPart {
            label: name.to_string(),
            path: browse.to_string_lossy().to_string(),
        });
    }
    // Figma 4778:25968 — deep paths collapse to `… ▶ Shows ▶ The Wand` (no section prefix).
    if parts.len() > 3 {
        let last2 = parts[parts.len() - 2..].to_vec();
        let mut out = vec![BreadcrumbPart {
            label: "…".into(),
            path: String::new(),
        }];
        out.extend(last2);
        return out;
    }
    parts
}

/// Figma-style breadcrumb title string (legacy / status).
#[must_use]
pub fn breadcrumb(section: &str, browse: &Path, roots: &[String]) -> String {
    breadcrumb_parts(section, browse, roots)
        .into_iter()
        .map(|p| p.label)
        .collect::<Vec<_>>()
        .join(" › ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MediaType;

    fn item(path: &str, ty: MediaType) -> MediaItem {
        MediaItem {
            id: 1,
            path: path.into(),
            title: "t".into(),
            duration_ms: None,
            media_type: ty,
            mtime: None,
            artwork_path: None,
        }
    }

    #[test]
    fn child_folders_should_take_next_segment() {
        // Build paths with `PathBuf` so separators work on Windows and Unix CI.
        let shows = PathBuf::from("Lib").join("Shows");
        let path = |parts: &[&str]| -> String {
            parts
                .iter()
                .collect::<PathBuf>()
                .to_string_lossy()
                .into_owned()
        };
        let media = vec![
            item(
                &path(&["Lib", "Shows", "Wand", "S01", "e1.mkv"]),
                MediaType::Video,
            ),
            item(
                &path(&["Lib", "Shows", "Wand", "S02", "e1.mkv"]),
                MediaType::Video,
            ),
            item(
                &path(&["Lib", "Shows", "Other", "e1.mkv"]),
                MediaType::Video,
            ),
        ];
        let kids = child_folders(&shows, &media, Some(MediaType::Video));
        let names: Vec<_> = kids.iter().map(|k| k.name.as_str()).collect();
        assert!(names.contains(&"Wand"));
        assert!(names.contains(&"Other"));
    }
}
