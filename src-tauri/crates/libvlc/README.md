# libvlc

Safe Rust wrappers around libVLC 3.x for Orpheon.

Uses `thiserror` (`LibVlcError`). Video frames use a double-buffered `VideoFrameState` shared with decoder threads via `Arc`.
