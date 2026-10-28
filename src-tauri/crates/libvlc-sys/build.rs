fn main() {
    println!("cargo:rerun-if-env-changed=VLC_LIB_DIR");
    println!("cargo:rerun-if-env-changed=VLC_PLUGIN_PATH");

    if std::env::var("CARGO_FEATURE_LINK").is_err() {
        return;
    }

    if let Some(dir) = non_empty_env("VLC_LIB_DIR") {
        println!("cargo:rustc-link-search=native={dir}");
        println!("cargo:rustc-link-lib=dylib=vlc");
        return;
    }

    // Prefer vendored import libs (see scripts/vendor-libvlc.ps1).
    // From src-tauri/crates/libvlc-sys → ../../../vendor/vlc (repo root).
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let manifest = std::path::Path::new(&manifest_dir);
    for rel in ["../../../vendor/vlc", "../../vendor/vlc"] {
        let vendor = manifest.join(rel);
        let vendor = vendor.canonicalize().unwrap_or(vendor);
        if vendor.join("vlc.lib").exists() || vendor.join("libvlc.dll").exists() {
            println!("cargo:rustc-link-search=native={}", vendor.display());
            println!("cargo:rustc-link-lib=dylib=vlc");
            return;
        }
    }

    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Program Files\VideoLAN\VLC",
            r"C:\Program Files (x86)\VideoLAN\VLC",
        ];
        for path in candidates {
            if std::path::Path::new(path).join("libvlc.dll").exists() {
                println!("cargo:rustc-link-search=native={path}");
                println!("cargo:rustc-link-lib=dylib=vlc");
                return;
            }
        }
        println!(
            "cargo:warning=libVLC not found — run scripts/vendor-libvlc.ps1 or set VLC_LIB_DIR"
        );
    }

    #[cfg(target_os = "macos")]
    {
        let vlc_lib = "/Applications/VLC.app/Contents/MacOS/lib";
        if std::path::Path::new(vlc_lib).join("libvlc.dylib").exists()
            || std::path::Path::new(vlc_lib)
                .join("libvlc.5.dylib")
                .exists()
        {
            println!("cargo:rustc-link-search=native={vlc_lib}");
            println!("cargo:rustc-link-lib=dylib=vlc");
            return;
        }
        println!(
            "cargo:warning=libVLC not found under {vlc_lib} — install VLC.app or set VLC_LIB_DIR"
        );
    }

    #[cfg(target_os = "linux")]
    {
        // Rely on the system linker / pkg-config path (libvlc-dev).
        println!("cargo:rustc-link-lib=dylib=vlc");
    }
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key).ok().and_then(|v| {
        let t = v.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}
