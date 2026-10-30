# Packaging

## Windows

Prereq: populate `vendor/vlc` (DLLs + `plugins/`) via `.\scripts\vendor-libvlc.ps1` or `.\scripts\ci-prepare-vlc.ps1`.

1. Build the app (MSI / NSIS **and** release exe):

```powershell
npm run tauri:build
```

libVLC is packaged into installers via `bundle.resources` in `src-tauri/tauri.conf.json` (`libvlc.dll`, `libvlccore.dll`, `plugins/` next to `Orpheon.exe` — same layout as portable).

2. Optional portable zip (same layout, no installer):

```powershell
$exe = Get-ChildItem -Recurse src-tauri\target\release\Orpheon.exe | Select-Object -First 1 -Expand FullName
.\scripts\bundle-libvlc.ps1 -OutDir dist\win -ExeSource $exe
```

## CI / GitHub Releases

- **CI** (`.github/workflows/ci.yml`): on `push`/`pull_request` to `main`/`master` — typecheck, Vitest (`npm run test:run`), `cargo test`, Tauri Windows build, portable artifact upload.
- **Release** (`.github/workflows/release.yml`): on tag `v*` (e.g. `git tag v0.1.0 && git push origin v0.1.0`) — builds Windows portable zip (+ MSI/NSIS if produced) and publishes a GitHub Release.

CI prepares `vendor/vlc` via `scripts/ci-prepare-vlc.ps1` (system VLC or VideoLAN win64 zip).

## Notes

- Redistribute under **GPL-2.0-or-later**; include VLC/libVLC license texts from `vendor/vlc`.
- Other platforms will use the software RGBA path (no HWND overlay).
- Windows-only packaging for now.
