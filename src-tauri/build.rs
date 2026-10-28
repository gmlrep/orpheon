use std::path::{Path, PathBuf};

fn main() {
    tauri_build::build();

    // Copy vendored libVLC next to the binary so Windows finds DLLs at process start
    // (STATUS_DLL_NOT_FOUND happens before Rust `main` can fix PATH).
    if std::env::var_os("CARGO_FEATURE_VLC").is_some() {
        if let Err(e) = copy_vendored_libvlc() {
            println!("cargo:warning=libVLC vendor copy failed: {e}");
        }
    }
}

fn copy_vendored_libvlc() -> Result<(), String> {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").map_err(|e| e.to_string())?);
    // src-tauri → ../vendor/vlc
    let vendor = manifest_dir.join("../vendor/vlc");
    let vendor = vendor
        .canonicalize()
        .map_err(|e| format!("vendor/vlc not found ({e}); run scripts/vendor-libvlc.ps1"))?;

    println!(
        "cargo:rerun-if-changed={}",
        vendor.join("libvlc.dll").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        vendor.join("libvlccore.dll").display()
    );

    let target_dir = resolve_target_dir(&manifest_dir)?;

    for name in ["libvlc.dll", "libvlccore.dll"] {
        let src = vendor.join(name);
        if !src.is_file() {
            return Err(format!("missing {name} in {}", vendor.display()));
        }
        std::fs::copy(&src, target_dir.join(name)).map_err(|e| format!("copy {name}: {e}"))?;
    }

    let plugins_src = vendor.join("plugins");
    if plugins_src.is_dir() {
        let plugins_dst = target_dir.join("plugins");
        copy_dir_recursive(&plugins_src, &plugins_dst)?;
    } else {
        println!("cargo:warning=vendor/vlc/plugins missing — run scripts/vendor-libvlc.ps1");
    }

    Ok(())
}

fn resolve_target_dir(manifest_dir: &Path) -> Result<PathBuf, String> {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    // Prefer CARGO_TARGET_DIR, then src-tauri/target/<profile>.
    if let Ok(td) = std::env::var("CARGO_TARGET_DIR") {
        let p = PathBuf::from(td).join(&profile);
        if p.is_dir() {
            return Ok(p);
        }
    }
    let local = manifest_dir.join("target").join(&profile);
    if local.is_dir() {
        return Ok(local);
    }
    // OUT_DIR = target/<profile>/build/<crate>-<hash>/out
    let out = PathBuf::from(std::env::var("OUT_DIR").map_err(|e| e.to_string())?);
    out.ancestors()
        .nth(3)
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "cannot resolve target dir from OUT_DIR".to_string())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else if ty.is_file() {
            // Skip overwrite if same size (faster rebuilds).
            let src_meta = entry.metadata().ok();
            let dst_meta = std::fs::metadata(&to).ok();
            let skip = matches!((src_meta, dst_meta), (Some(s), Some(d)) if s.len() == d.len());
            if !skip {
                std::fs::copy(entry.path(), &to)
                    .map_err(|e| format!("copy {}: {e}", entry.path().display()))?;
            }
        }
    }
    Ok(())
}
