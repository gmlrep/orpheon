# Packaging

## Windows

1. Build the app:

```powershell
npm run tauri:build
```

2. Bundle libVLC next to the exe (DLLs + `plugins/`):

```powershell
$exe = Get-ChildItem -Recurse src-tauri\target\release\Orpheon.exe | Select-Object -First 1 -Expand FullName
.\scripts\bundle-libvlc.ps1 -OutDir dist\win -ExeSource $exe
```

`vendor/vlc` is populated by `.\scripts\vendor-libvlc.ps1` (copies from an installed VLC 3.x).

## CI / GitHub Releases

- **CI** (`.github/workflows/ci.yml`): on `push`/`pull_request` to `main`/`master` — typecheck, Vitest (`npm run test:run`), `cargo test`, Tauri Windows build, portable artifact upload.
- **Release** (`.github/workflows/release.yml`): on tag `v*` (e.g. `git tag v0.1.0 && git push origin v0.1.0`) — builds Windows portable zip (+ MSI/NSIS if produced) and publishes a GitHub Release.

CI prepares `vendor/vlc` via `scripts/ci-prepare-vlc.ps1` (system VLC or VideoLAN win64 zip).

## Notes

- Redistribute under **GPL-2.0-or-later**; include VLC/libVLC license texts from `vendor/vlc`.
- Other platforms will use the software RGBA path (no HWND overlay).
- Windows-only packaging for now.
