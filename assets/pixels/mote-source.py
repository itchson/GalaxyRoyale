"""Original Isotope Mote occupied-cell concept; no generated image/model.

The JSON holds runtime cell data. The PNG is an inspection preview only.
"""
import json
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent
SIZE = 40
COLORS = [
    (0, 0, 0, 0), (27, 24, 42, 255), (46, 35, 68, 255),
    (75, 48, 105, 255), (106, 70, 143, 255), (142, 104, 170, 255),
    (189, 149, 199, 255), (78, 100, 69, 255), (128, 156, 83, 255),
    (193, 217, 102, 255), (235, 244, 153, 255), (249, 251, 210, 255),
    (93, 141, 144, 255), (122, 204, 199, 255), (184, 244, 217, 255),
    (57, 68, 83, 255),
]
cells = [[0] * SIZE for _ in range(SIZE)]
roles = [[0] * SIZE for _ in range(SIZE)]

def disc(cx, cy, radius, color, role=0):
    for y in range(SIZE):
        for x in range(SIZE):
            if (x - cx) ** 2 + (y - cy) ** 2 <= radius ** 2:
                cells[y][x] = color
                roles[y][x] = role

def rect(x0, y0, x1, y1, color, role=0):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            cells[y][x] = color
            roles[y][x] = role

# Three stepped radial housings connected to a hollow annular reactor.
rect(16, 8, 23, 21, 1)
rect(17, 10, 22, 20, 3)
for row in range(7):
    rect(6 + row, 26 - row, 12 + row, 28 - row, 1)
    rect(27 - row, 26 - row, 33 - row, 28 - row, 1)
    rect(8 + row, 26 - row, 10 + row, 26 - row, 4)
    rect(29 - row, 26 - row, 31 - row, 26 - row, 4)
for cx, cy, role in [(19.5, 6.5, 1), (7.5, 29.5, 2), (31.5, 29.5, 3)]:
    disc(cx, cy, 7.0, 1, role)
    disc(cx, cy, 5.8, 3, role)
    disc(cx, cy, 4.6, 5, role)
    disc(cx, cy, 3.5, 7, role)
    disc(cx, cy, 2.5, 9, role)
    disc(cx - 0.5, cy - 0.5, 1.4, 11, role)

disc(19.5, 21.5, 9.8, 1)
disc(19.5, 21.5, 8.4, 3)
disc(19.5, 21.5, 7.0, 5)
disc(19.5, 21.5, 5.7, 2)
disc(19.5, 21.5, 4.2, 8)
disc(19.5, 21.5, 2.7, 0)  # real holes, never a black transparent substitute
for x, y in [(16, 15), (23, 15), (12, 24), (27, 24), (18, 30), (21, 30)]:
    rect(x, y, x, y, 6)
rect(18, 11, 21, 12, 10)
rect(12, 26, 13, 27, 10)
rect(26, 26, 27, 27, 10)
for x in [18, 21]:
    rect(x, 33, x, 34, 13)

indices = [color for row in cells for color in row]
masks = [[[x, y] for y in range(SIZE) for x in range(SIZE)
          if roles[y][x] == role and cells[y][x]] for role in [1, 2, 3]]
palette = [int.from_bytes(bytes(c), "little") for c in COLORS]
palette += [palette[1]] * (64 - len(palette))
atlas = {"version": 1, "width": SIZE, "height": SIZE, "palette": palette,
         "indices": indices, "emitters": masks}
(ROOT / "mote-atlas.json").write_text(json.dumps(atlas, separators=(",", ":")) + "\n", encoding="utf-8")
image = Image.new("RGBA", (SIZE, SIZE))
image.putdata([COLORS[i] for i in indices])
preview = Image.new("RGB", (420, 368), (9, 13, 32))
large = image.resize((280, 280), Image.Resampling.NEAREST)
preview.paste(large, (70, 48), large)
draw = ImageDraw.Draw(preview)
draw.text((14, 12), "ISOTOPE MOTE / ORIGINAL OCCUPIED CELLS", fill=COLORS[10])
draw.text((14, 342), "CONCEPT PROTOTYPE / NOT IN PRODUCTION", fill=COLORS[6])
preview.save(ROOT / "mote-preview.png")
provenance = {"method": "original authored geometry and palette; no AI image/model",
              "runtime": "indexed cells only; preview is not loaded", "size": [SIZE, SIZE],
              "occupied": sum(i != 0 for i in indices), "opaque_colors": len(COLORS) - 1,
              "emitter_cells": [len(mask) for mask in masks]}
(ROOT / "mote-provenance.json").write_text(json.dumps(provenance, indent=2) + "\n", encoding="utf-8")
print(json.dumps(provenance))
