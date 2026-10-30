//! Probe: does libVLC open a Windows canonicalize() path (`\\?\…`)?
//!
//! Usage:
//!   cargo run -p libvlc --example open_path_probe --features link -- "D:\path\to\file.mp3"

use libvlc::{HwAccelMode, Instance, Media, MediaPlayer};
use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn try_open(label: &str, instance: &Instance, path: &std::path::Path) -> bool {
    println!("--- {label}");
    println!("path={path:?}");
    match Media::from_path(instance, path) {
        Ok(media) => {
            let player = match MediaPlayer::new(instance) {
                Ok(p) => p,
                Err(e) => {
                    println!("FAIL MediaPlayer::new: {e}");
                    return false;
                }
            };
            player.set_media(&media);
            match player.play() {
                Ok(()) => {
                    thread::sleep(Duration::from_millis(400));
                    let playing = player.is_playing();
                    let len = player.length_ms();
                    println!("play() ok playing={playing} length_ms={len}");
                    player.stop();
                    playing || len > 0
                }
                Err(e) => {
                    println!("FAIL play: {e}");
                    false
                }
            }
        }
        Err(e) => {
            println!("FAIL Media::from_path: {e}");
            false
        }
    }
}

fn main() {
    let arg = env::args()
        .nth(1)
        .expect("usage: open_path_probe <media-file>");
    let raw = PathBuf::from(&arg);
    assert!(raw.is_file(), "not a file: {arg}");

    let instance = Instance::for_playback(HwAccelMode::Disabled, &[]).expect("libvlc init");

    let ok_raw = try_open("raw path", &instance, &raw);
    let canon = raw.canonicalize().expect("canonicalize");
    let ok_canon_raw = try_open("canonicalize() path (verbatim)", &instance, &canon);
    let stripped = libvlc::path_for_libvlc(&canon);
    let ok_stripped = try_open("canonicalize() + path_for_libvlc", &instance, &stripped);

    println!("==== RESULT raw={ok_raw} canon_verbatim={ok_canon_raw} stripped={ok_stripped}");
    if ok_raw && !ok_stripped {
        eprintln!("BUG: stripped canonicalize path still fails libVLC open");
        std::process::exit(2);
    }
    if !ok_raw {
        eprintln!("raw path also failed — check VLC plugins / file");
        std::process::exit(1);
    }
    // Verbatim may fail on Windows; that is expected without stripping.
    if ok_raw && !ok_canon_raw && ok_stripped {
        println!("OK: path_for_libvlc repairs Windows verbatim canonicalize paths");
    }
}
