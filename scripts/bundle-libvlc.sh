#!/usr/bin/env bash
# Bundle system libVLC for Linux/macOS packaging.
# Usage: ./scripts/bundle-libvlc.sh [out-dir]
set -euo pipefail
OUT="${1:-./dist/native}"
mkdir -p "$OUT"

if [[ "$(uname)" == "Darwin" ]]; then
  VLC_LIB="/Applications/VLC.app/Contents/MacOS/lib"
  VLC_PLUGINS="/Applications/VLC.app/Contents/MacOS/plugins"
  cp -R "$VLC_LIB"/* "$OUT/" || true
  cp -R "$VLC_PLUGINS" "$OUT/plugins" || true
else
  # Prefer distro packages; copy .so if found via ldconfig
  echo "On Linux, prefer packaging against distro libvlc (libvlc-dev)."
  echo "Place libvlc.so.* and plugins under $OUT if bundling."
fi

echo "Bundle output: $OUT"
