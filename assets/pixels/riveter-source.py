"""Original authored Riveter cell hull; emits engine data and a nearest preview.

No generated image/model is used. Coordinates, surfaces and machinery are authored
here. PNG is a review preview; the runtime consumes the indexed occupied cells.
"""
import json
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent
SIZE = 40
COLORS = [
    (0, 0, 0, 0), (27, 23, 32, 255), (47, 40, 51, 255),
    (72, 54, 61, 255), (101, 65, 58, 255), (151, 89, 53, 255),
    (204, 132, 62, 255), (239, 183, 83, 255), (255, 223, 136, 255),
    (249, 244, 207, 255), (72, 130, 141, 255), (117, 211, 204, 255),
    (173, 247, 223, 255), (142, 152, 173, 255), (202, 211, 223, 255),
    (73, 82, 104, 255),
]
cells = [[0] * SIZE for _ in range(SIZE)]
roles = [[0] * SIZE for _ in range(SIZE)]

def rect(x0, y0, x1, y1, color, role=0):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            cells[y][x] = color
            roles[y][x] = role

# Split barrels, broad stepped shoulders and a recessed reactor. Nose faces up.
for x0, role in [(6, 1), (27, 2)]:
    rect(x0, 2, x0 + 6, 20, 1, role)
    rect(x0 + 1, 3, x0 + 5, 18, 13, role)
    rect(x0 + 2, 4, x0 + 4, 16, 2, role)
    rect(x0 + 2, 2, x0 + 4, 4, 8, role)
    rect(x0 + 2, 3, x0 + 4, 3, 9, role)
    rect(x0 + 2, 6, x0 + 4, 7, 6, role)
    rect(x0 + 2, 10, x0 + 4, 11, 6, role)
    rect(x0 + 2, 14, x0 + 4, 15, 6, role)
    for y in [8, 12, 16]:
        rect(x0 + 1, y, x0 + 1, y, 14, role)

for x0 in [2, 29]:
    rect(x0, 16, x0 + 8, 30, 1)
    rect(x0 + 1, 17, x0 + 7, 28, 4)
    rect(x0 + 2, 18, x0 + 6, 26, 6)
    rect(x0 + 2, 18, x0 + 5, 19, 8)
    rect(x0 + 6, 20, x0 + 6, 27, 5)
    for y in [21, 24, 27]:
        rect(x0 + 2, y, x0 + 4, y, 2)
        rect(x0 + 2, y - 1, x0 + 2, y - 1, 7)

rect(8, 17, 31, 33, 1)
rect(10, 18, 29, 32, 3)
rect(12, 19, 27, 30, 5)
rect(13, 19, 26, 20, 8)
rect(14, 21, 25, 28, 2)
rect(16, 22, 23, 27, 10)
rect(17, 22, 22, 25, 11)
rect(18, 22, 20, 23, 12)
rect(16, 28, 23, 28, 13)
rect(10, 30, 29, 32, 4)
for x in [11, 14, 25, 28]:
    rect(x, 31, x, 31, 9)
rect(11, 33, 28, 36, 1)
rect(12, 33, 27, 34, 13)
rect(14, 35, 17, 37, 10)
rect(22, 35, 25, 37, 10)
rect(15, 36, 16, 37, 11)
rect(23, 36, 24, 37, 11)

# Whole living barrel masks matter to the weapon; no invisible pristine muzzle.
indices = [value for row in cells for value in row]
muzzles = [[[x, y] for y in range(SIZE) for x in range(SIZE)
            if roles[y][x] == role and cells[y][x]] for role in [1, 2]]
palette = [int.from_bytes(bytes(color), "little") for color in COLORS]
palette += [palette[1]] * (64 - len(palette))
atlas = {"version": 1, "palette": palette,
         "sprites": {"riveter": {"width": SIZE, "height": SIZE, "indices": indices}},
         "muzzles": muzzles}
(ROOT / "riveter-atlas.json").write_text(json.dumps(atlas, separators=(",", ":")) + "\n", encoding="utf-8")

image = Image.new("RGBA", (SIZE, SIZE))
image.putdata([COLORS[index] for index in indices])
preview = Image.new("RGB", (380, 350), (9, 13, 32))
preview.paste(image.resize((280, 280), Image.Resampling.NEAREST), (50, 42),
              image.resize((280, 280), Image.Resampling.NEAREST))
draw = ImageDraw.Draw(preview)
draw.text((16, 12), "IRON BELT RIVETER / ORIGINAL AUTHORED CELLS", fill=(255, 223, 136))
preview.save(ROOT / "riveter-preview.png")
provenance = {"source": "riveter-source.py", "method": "original authored cell geometry and surface palette; no AI-generated image",
              "runtime": "riveter-atlas.json indexed cells; preview PNG is not loaded by the game",
              "width": SIZE, "height": SIZE, "occupied_cells": sum(index != 0 for index in indices),
              "opaque_palette_colors": len(COLORS) - 1, "muzzle_cells": [len(mask) for mask in muzzles]}
(ROOT / "riveter-provenance.json").write_text(json.dumps(provenance, indent=2) + "\n", encoding="utf-8")
print(json.dumps(provenance))
