#!/usr/bin/env python3
"""Rasterize Synth branding (emblem, tray icons, NSIS bitmaps, ICO) with stdlib only."""
from __future__ import annotations

import math
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "branding"

BG = (11, 19, 24, 255)
FRAME = (16, 185, 129, 255)
CORE = (14, 23, 21, 255)
SAT = (56, 189, 248, 255)
BANNER_PLATE = (20, 83, 74, 255)


def clamp(v: float) -> int:
    return 0 if v < 0 else 255 if v > 255 else int(v + 0.5)


def blend(dst: list[int], x: int, y: int, w: int, rgba: tuple[int, int, int, int], cover: float) -> None:
    if cover <= 0 or x < 0 or y < 0 or x >= w:
        return
    a = rgba[3] / 255.0 * min(1.0, cover)
    if a <= 0:
        return
    i = (y * w + x) * 4
    ia = 1.0 - a
    dst[i] = clamp(dst[i] * ia + rgba[0] * a)
    dst[i + 1] = clamp(dst[i + 1] * ia + rgba[1] * a)
    dst[i + 2] = clamp(dst[i + 2] * ia + rgba[2] * a)
    dst[i + 3] = clamp(dst[i + 3] * ia + 255 * a)


def fill_round_rect(px: list[int], w: int, h: int, cx: float, cy: float, rw: float, rh: float, radius: float, color: tuple[int, int, int, int]) -> None:
    x0 = max(0, int(cx - rw - 1))
    x1 = min(w - 1, int(cx + rw + 1))
    y0 = max(0, int(cy - rh - 1))
    y1 = min(h - 1, int(cy + rh + 1))
    r = max(0.5, radius)
    ix0, ix1 = cx - rw + r, cx + rw - r
    iy0, iy1 = cy - rh + r, cy + rh - r
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            px_x, px_y = x + 0.5, y + 0.5
            dx = 0.0 if ix0 <= px_x <= ix1 else (ix0 - px_x if px_x < ix0 else px_x - ix1)
            dy = 0.0 if iy0 <= px_y <= iy1 else (iy0 - px_y if px_y < iy0 else px_y - iy1)
            dist = math.hypot(dx, dy) - r
            cover = 1.0 - dist
            if cover > 0:
                blend(px, x, y, w, color, cover)


def fill_circle(px: list[int], w: int, h: int, cx: float, cy: float, radius: float, color: tuple[int, int, int, int]) -> None:
    r = max(0.5, radius)
    x0 = max(0, int(cx - r - 1))
    x1 = min(w - 1, int(cx + r + 1))
    y0 = max(0, int(cy - r - 1))
    y1 = min(h - 1, int(cy + r + 1))
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            dist = math.hypot(x + 0.5 - cx, y + 0.5 - cy) - r
            cover = 1.0 - dist
            if cover > 0:
                blend(px, x, y, w, color, cover)


def canvas(w: int, h: int, bg: tuple[int, int, int, int]) -> list[int]:
    px = [0] * (w * h * 4)
    for i in range(0, len(px), 4):
        px[i : i + 4] = bg
    return px


def write_png(path: Path, w: int, h: int, rgba: list[int]) -> None:
    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    raw = b"".join(b"\x00" + bytes(rgba[y * w * 4 : (y + 1) * w * 4]) for y in range(h))
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def write_bmp24(path: Path, w: int, h: int, rgba: list[int]) -> None:
    row_stride = (w * 3 + 3) & ~3
    pixel_bytes = bytearray(row_stride * h)
    for y in range(h):
        src_y = h - 1 - y
        off = y * row_stride
        for x in range(w):
            i = (src_y * w + x) * 4
            pixel_bytes[off + x * 3] = rgba[i + 2]
            pixel_bytes[off + x * 3 + 1] = rgba[i + 1]
            pixel_bytes[off + x * 3 + 2] = rgba[i]
    header = struct.pack("<2sIHHI", b"BM", 14 + 40 + len(pixel_bytes), 0, 0, 14 + 40)
    dib = struct.pack("<IiiHHIIiiII", 40, w, h, 1, 24, 0, len(pixel_bytes), 2835, 2835, 0, 0)
    path.write_bytes(header + dib + pixel_bytes)


def write_ico_png(path: Path, png: bytes, w: int, h: int) -> None:
    # Vista+ ICO can embed a PNG. Explorer and NSIS both accept this.
    header = struct.pack("<HHH", 0, 1, 1)
    entry = struct.pack("<BBBBHHII", w if w < 256 else 0, h if h < 256 else 0, 0, 0, 1, 32, len(png), 6 + 16)
    path.write_bytes(header + entry + png)


def paint_mark(px: list[int], w: int, h: int, frame: tuple[int, int, int, int], core: tuple[int, int, int, int], sat: tuple[int, int, int, int]) -> None:
    cx, cy = w / 2.0, h / 2.0
    s = min(w, h)
    fill_round_rect(px, w, h, cx, cy, s * 0.406, s * 0.406, s * 0.109, frame)
    fill_circle(px, w, h, cx, cy, s * 0.141, core)
    fill_circle(px, w, h, cx + s * 0.188, cy - s * 0.156, s * 0.063, sat)


def paint_icon(size: int, bg: tuple[int, int, int, int] = BG) -> list[int]:
    px = canvas(size, size, bg)
    paint_mark(px, size, size, FRAME, CORE, SAT)
    return px


def paint_banner() -> tuple[int, int, list[int]]:
    w, h = 320, 112
    px = canvas(w, h, BG)
    fill_round_rect(px, w, h, 160, 56, 140, 42, 18, BANNER_PLATE)
    fill_circle(px, w, h, 64, 56, 18, FRAME)
    fill_circle(px, w, h, 64, 56, 10, CORE)
    fill_circle(px, w, h, 248, 56, 8, SAT)
    return w, h, px


def emblem_svg() -> str:
    return """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" role="img" aria-label="Webizen AI Synth">
  <rect width="64" height="64" rx="14" fill="#0b1318"/>
  <rect x="6" y="6" width="52" height="52" rx="14" fill="#10b981"/>
  <circle cx="32" cy="32" r="18" fill="#0e1715"/>
  <circle cx="44" cy="22" r="8" fill="#38bdf8"/>
</svg>
"""


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "emblem.svg").write_text(emblem_svg(), encoding="utf-8")
    for size in (16, 32, 48, 64, 128, 256):
        write_png(OUT / f"icon-{size}.png", size, size, paint_icon(size))
    write_png(OUT / "icon-256.png", 256, 256, paint_icon(256))
    write_png(OUT / "icon.png", 256, 256, paint_icon(256))
    bw, bh, banner = paint_banner()
    write_png(OUT / "banner.png", bw, bh, banner)

    png256 = (OUT / "icon-256.png").read_bytes()
    write_ico_png(OUT / "icon.ico", png256, 256, 256)

    sidebar = canvas(164, 314, BG)
    paint_mark(sidebar, 164, 314, FRAME, CORE, SAT)
    write_bmp24(OUT / "installer-sidebar.bmp", 164, 314, sidebar)

    header = canvas(150, 57, BG)
    paint_mark(header, 150, 57, FRAME, CORE, SAT)
    write_bmp24(OUT / "installer-header.bmp", 150, 57, header)

    print(f"wrote branding into {OUT}")


if __name__ == "__main__":
    main()
