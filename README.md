# Orpheon

Standalone local media player — **Vue 3 + Vite + Tauri 2 + libVLC** (v0.1.0).  
New TypeScript UI for the anemone Slint-era player, designed from `.taste-skill/app`.

**Not affiliated with VideoLAN.** GPL-2.0-or-later.

## Stack

| Layer | Choice |
|-------|--------|
| UI | Vue 3, Pinia, vue-i18n, Inter |
| Shell | Tauri 2 (Windows first) |
| Engine | libVLC 3.x (`src-tauri/crates/libvlc`) |
| Library / settings | SQLite (`library`, `settings` crates) |

## Quick start (Windows)

Requirements: Node 22+, Rust stable, WebView2, VLC 3.x **or** vendored `vendor/vlc`.

```powershell
npm install
# if vendor/vlc is empty:
.\scripts\vendor-libvlc.ps1
npm run tauri:dev
```

UI-only (no libVLC link):

```powershell
cd src-tauri
cargo run --no-default-features
```

## Layout

```
src/                 Vue UI (shell + views)
src-tauri/
  crates/libvlc*     FFI + safe wrappers
  crates/library     media scan / playlists
  crates/settings    prefs + resume
  src/               Tauri commands, HWND video embed, poller
vendor/vlc/          bundled libVLC (Windows)
.taste-skill/        design mockup (reference)
.icons/              icon source set
```

## Design

True-black flat shell from `.taste-skill`: amber `#d37918` only on progress/seek; sidebar modes open / rail / hidden; NowBar in the titlebar.

## Packaging

```powershell
npm run tauri:build
.\scripts\bundle-libvlc.ps1 -OutDir dist\win -ExeSource (Get-ChildItem -Recurse src-tauri\target\release\Orpheon.exe | Select-Object -First 1 -Expand FullName)
```
