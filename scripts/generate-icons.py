"""Generate Orpheon app icons from .icons/main_00007.png into src-tauri/icons and public/."""

from __future__ import annotations

import io
import struct
from collections import deque
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / ".icons" / "main_00007.png"
OUT = ROOT / "src-tauri" / "icons"
PUBLIC = ROOT / "public"
ASSETS = ROOT / "src" / "assets"


def punch_white_corners(im: Image.Image, threshold: int = 235) -> Image.Image:
    """Make near-white background outside the rounded tile transparent via flood fill."""
    im = im.convert("RGBA")
    w, h = im.size
    px = im.load()
    assert px is not None

    def is_bg(x: int, y: int) -> bool:
        r, g, b, a = px[x, y]
        return a > 0 and r >= threshold and g >= threshold and b >= threshold

    seen = [[False] * w for _ in range(h)]
    q: deque[tuple[int, int]] = deque()
    for x, y in ((0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)):
        if is_bg(x, y):
            q.append((x, y))
            seen[y][x] = True

    while q:
        x, y = q.popleft()
        px[x, y] = (0, 0, 0, 0)
        for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
            if 0 <= nx < w and 0 <= ny < h and not seen[ny][nx] and is_bg(nx, ny):
                seen[ny][nx] = True
                q.append((nx, ny))
    return im


def main() -> None:
    if not SRC.is_file():
        raise SystemExit(f"missing source icon: {SRC}")

    src = punch_white_corners(Image.open(SRC))
    OUT.mkdir(parents=True, exist_ok=True)
    PUBLIC.mkdir(parents=True, exist_ok=True)

    targets = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }

    for name, size in targets.items():
        img = src.resize((size, size), Image.Resampling.LANCZOS)
        img.save(OUT / name, format="PNG", optimize=True)
        print(f"wrote {OUT / name}")

    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    ico_imgs = [src.resize((s, s), Image.Resampling.LANCZOS) for s in ico_sizes]
    ico_path = OUT / "icon.ico"
    ico_imgs[-1].save(
        ico_path,
        format="ICO",
        sizes=[(im.width, im.height) for im in ico_imgs],
        append_images=ico_imgs[:-1],
    )
    print(f"wrote {ico_path}")

    fav = src.resize((192, 192), Image.Resampling.LANCZOS)
    fav.save(PUBLIC / "icon.png", format="PNG", optimize=True)
    print(f"wrote {PUBLIC / 'icon.png'}")

    # Windows favicon for browsers / static hosting
    favico = OUT / "icon.ico"
    if favico.is_file():
        (PUBLIC / "favicon.ico").write_bytes(favico.read_bytes())
        print(f"wrote {PUBLIC / 'favicon.ico'}")

    mark_png = src.resize((256, 256), Image.Resampling.LANCZOS)
    mark_png.save(ASSETS / "mark.png", format="PNG", optimize=True)
    print(f"wrote {ASSETS / 'mark.png'}")

    # Minimal ICNS with one 512x512 PNG chunk (ic09) — enough for Tauri mac packaging.
    png512 = src.resize((512, 512), Image.Resampling.LANCZOS)
    buf = io.BytesIO()
    png512.save(buf, format="PNG")
    png_bytes = buf.getvalue()
    chunk = b"ic09" + struct.pack(">I", 8 + len(png_bytes)) + png_bytes
    file_size = 8 + len(chunk)
    icns_path = OUT / "icon.icns"
    icns_path.write_bytes(b"icns" + struct.pack(">I", file_size) + chunk)
    print(f"wrote {icns_path}")


if __name__ == "__main__":
    main()
