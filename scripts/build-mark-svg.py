"""Build src/assets/mark.svg from .icons/main_00007.png (exact, non-distorted)."""

from __future__ import annotations

import base64
import io
from collections import deque
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / ".icons" / "main_00007.png"
OUTS = [
    ROOT / "src" / "assets" / "mark.svg",
    ROOT / "src" / "assets" / "vue.svg",  # replace scaffold logo
    ROOT / "public" / "orpheon.svg",
    ROOT / "public" / "tauri.svg",  # replace scaffold logo
    ROOT / "public" / "vite.svg",  # replace scaffold logo
    ROOT / "src-tauri" / "icons" / "icon.svg",
]


def punch_white_corners(im: Image.Image, threshold: int = 235) -> Image.Image:
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
    src = punch_white_corners(Image.open(SRC))
    img = src.resize((256, 256), Image.Resampling.LANCZOS)
    buf = io.BytesIO()
    img.save(buf, format="PNG", optimize=True)
    b64 = base64.b64encode(buf.getvalue()).decode("ascii")
    svg = (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" '
        'width="256" height="256" role="img" aria-label="Orpheon">\n'
        f'  <image width="256" height="256" href="data:image/png;base64,{b64}"/>\n'
        "</svg>\n"
    )
    for out in OUTS:
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(svg, encoding="utf-8")
        print(f"wrote {out} ({out.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
