//! Time cold `libvlc_new` — proxy for UI-thread startup freeze.
//! `cargo run -p libvlc --example time_init --features link`

use std::time::Instant;

fn main() {
    let candidates = [
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/vlc"),
        std::path::PathBuf::from(r"C:\Program Files\VideoLAN\VLC"),
    ];
    for dir in candidates {
        let plugins = dir.join("plugins");
        if plugins.is_dir() {
            std::env::set_var("VLC_PLUGIN_PATH", &plugins);
            if let Ok(old) = std::env::var("PATH") {
                std::env::set_var("PATH", format!("{};{}", dir.display(), old));
            }
            eprintln!("VLC_PLUGIN_PATH={}", plugins.display());
            break;
        }
    }

    let t0 = Instant::now();
    match libvlc::Instance::for_playback(libvlc::HwAccelMode::Automatic, &[]) {
        Ok(_inst) => {
            let elapsed = t0.elapsed();
            println!("libvlc_new_ok_ms={}", elapsed.as_millis());
            eprintln!("Instance::for_playback OK in {elapsed:?}");
        }
        Err(e) => {
            let elapsed = t0.elapsed();
            println!("libvlc_new_err_ms={}", elapsed.as_millis());
            eprintln!("Instance::for_playback FAILED in {elapsed:?}: {e}");
            std::process::exit(1);
        }
    }
}
