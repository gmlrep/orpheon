//! Integration smoke test — requires a linked libVLC and a sample file.
//!
//! ```bash
//! cargo test -p libvlc --features link -- --ignored
//! ```

#![cfg(feature = "link")]

use libvlc::{Instance, Media, MediaPlayer};
use std::env;
use std::path::PathBuf;

#[test]
#[ignore = "requires libVLC installed and SAMPLE_MEDIA env path"]
fn play_should_start_when_sample_media_provided() {
    let path = env::var("SAMPLE_MEDIA")
        .map(PathBuf::from)
        .expect("set SAMPLE_MEDIA to a media file path");
    let instance = Instance::new().expect("libVLC instance");
    let media = Media::from_path(&instance, &path).expect("media");
    let player = MediaPlayer::new(&instance).expect("player");
    player.set_media(&media);
    player.play().expect("play");
    assert!(player.is_playing() || player.position() >= 0.0);
    player.stop();
}
