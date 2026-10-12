"""Original 64-cell flight controls; exact PNG -> packed-cell authoring.

No raster upscaling, generated-image approximation, filtering or color loss.
The small source PNG encoder uses explicit stored DEFLATE blocks so committed
source bytes are reproducible across Pillow/zlib platforms.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import struct
import zlib
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SIZE = 64
KINDS = ("status", "map", "settings", "save", "close", "resume", "restart",
         "title", "quit", "fullscreen")
COLORS = (
    (0, 0, 0, 0), (12, 13, 16, 255), (27, 30, 35, 255),
    (53, 60, 68, 255), (100, 110, 120, 255), (153, 165, 177, 255),
    (207, 217, 226, 255), (234, 239, 242, 255), (251, 252, 253, 255),
    (42, 51, 61, 255), (84, 100, 115, 255), (157, 180, 195, 255),
    (21, 85, 100, 255), (26, 143, 163, 255), (66, 213, 222, 255),
    (169, 245, 237, 255), (104, 81, 51, 255), (170, 133, 77, 255),
    (224, 189, 123, 255), (75, 40, 47, 255), (139, 69, 72, 255),
)
INK, RECESS, SHADOW, BEVEL, METAL, LIGHT, IVORY, EDGE = range(1, 9)
STEEL, STEEL_MID, STEEL_LIGHT, CYAN_DARK, CYAN, CYAN_LIGHT, CYAN_EDGE = range(9, 16)
BRASS_DARK, BRASS, BRASS_LIGHT, RED_DARK, RED = range(16, 21)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value, compact=False):
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(value, ensure_ascii=False, allow_nan=False,
                      **({"separators": (",", ":")} if compact else {"indent": 2}))
    path.write_bytes((text + "\n").encode("utf-8"))


def write_png(path: Path, image: Image.Image):
    """Lossless RGBA PNG, fixed filter0/stored blocks; no linked encoder drift."""
    image = image.convert("RGBA")
    width, height = image.size
    pixels = image.tobytes()
    raw = b"".join(b"\0" + pixels[y * width * 4:(y + 1) * width * 4]
                   for y in range(height))
    encoded = bytearray(b"\x78\x01")
    for start in range(0, len(raw), 65535):
        block = raw[start:start + 65535]
        encoded.append(int(start + len(block) == len(raw)))
        encoded.extend(struct.pack("<HH", len(block), len(block) ^ 0xffff))
        encoded.extend(block)
    encoded.extend(struct.pack(">I", zlib.adler32(raw) & 0xffffffff))

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", bytes(encoded)) + chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


class Icon:
    """A silhouette, immutable metal cells and explicit host animation masks."""
    def __init__(self, kind):
        self.kind = kind
        self.mask = Image.new("1", (SIZE, SIZE))
        self.shape = ImageDraw.Draw(self.mask)
        self.indices = [0] * (SIZE * SIZE)
        self.accents: set[int] = set()
        self.emitters: set[int] = set()

    def shade(self):
        for y in range(SIZE):
            for x in range(SIZE):
                if not self.mask.getpixel((x, y)):
                    continue
                distance = 4
                for step in range(1, 4):
                    if any(not (0 <= nx < SIZE and 0 <= ny < SIZE) or not self.mask.getpixel((nx, ny))
                           for nx, ny in ((x-step, y), (x+step, y), (x, y-step), (x, y+step))):
                        distance = step
                        break
                shade = METAL
                if distance == 1:
                    shade = EDGE if x + y < 64 else SHADOW
                elif distance == 2:
                    shade = LIGHT if x + y < 64 else BEVEL
                elif distance == 3:
                    shade = IVORY if y < 27 else LIGHT
                self.indices[y * SIZE + x] = shade

    def paint(self, shape, color, *, accent=False, emitter=False):
        layer = Image.new("1", (SIZE, SIZE))
        draw = ImageDraw.Draw(layer)
        shape(draw)
        for i, on in enumerate(layer.get_flattened_data()):
            if on and self.indices[i]:
                self.indices[i] = color
                self.accents.discard(i)
                self.emitters.discard(i)
                if accent or emitter:
                    self.accents.add(i)
                if emitter:
                    self.emitters.add(i)

    def rect(self, bounds, color, **roles):
        self.paint(lambda d: d.rectangle(bounds, fill=1), color, **roles)

    def line(self, points, color, width=1, **roles):
        self.paint(lambda d: d.line(points, fill=1, width=width), color, **roles)

    def polygon(self, points, color, **roles):
        self.paint(lambda d: d.polygon(points, fill=1), color, **roles)

    def ellipse(self, bounds, color, **roles):
        self.paint(lambda d: d.ellipse(bounds, fill=1), color, **roles)

    def rivet(self, x, y):
        self.rect((x-1, y-1, x+1, y+1), SHADOW)
        self.rect((x-1, y-1, x, y), IVORY)
        self.rect((x, y, x+1, y+1), RECESS)

    def image(self):
        image = Image.new("RGBA", (SIZE, SIZE))
        image.putdata([COLORS[i] for i in self.indices])
        return image

    def sprite(self):
        return {"width": SIZE, "height": SIZE, "indices": self.indices,
                "accent_cells": sorted(self.accents), "emitter_cells": sorted(self.emitters)}


def sports_metal(a):
    """Per-object alloy planes, never generic repeated diagonal texture.

    These cuts only shade the already authored mask, before semantic indicators
    are painted. They cannot fill apertures, change bounds, or tint accent roles.
    Fixed gold locator fittings are quiet hardware, not ownership/game state.
    """
    if a.kind == "status":
        a.polygon([(9, 42), (24, 28), (23, 39), (15, 44)], IVORY)
        a.polygon([(40, 29), (53, 42), (50, 46), (41, 41)], BEVEL)
        a.polygon([(27, 44), (37, 44), (38, 49), (26, 49)], STEEL)
        a.line([(28, 47), (36, 47)], SHADOW)
        a.rect((11, 42, 13, 43), BRASS_LIGHT)
        a.rect((26, 49, 28, 50), BRASS)
        a.polygon([(31, 5), (34, 5), (35, 10), (30, 10)], IVORY)
    elif a.kind == "map":
        # Three distinct folded metal planes, with strong recess/face contrast.
        a.polygon([(7, 17), (17, 12), (17, 46), (7, 51)], LIGHT)
        a.polygon([(7, 40), (17, 35), (17, 47), (7, 52)], STEEL)
        a.polygon([(26, 10), (36, 14), (36, 48), (26, 45)], METAL)
        a.polygon([(46, 15), (55, 11), (55, 42), (46, 46)], BEVEL)
        a.line([(8, 18), (16, 15)], IVORY)
        a.line([(47, 17), (54, 14)], LIGHT)
        a.rect((8, 14, 10, 15), BRASS_LIGHT)
        a.rect((49, 46, 52, 47), BRASS)
    elif a.kind == "settings":
        a.polygon([(9, 25), (15, 22), (22, 13), (25, 11), (26, 17), (15, 28)], IVORY)
        a.polygon([(42, 46), (49, 39), (54, 35), (55, 42), (49, 51), (43, 54)], BEVEL)
        a.polygon([(22, 54), (20, 46), (25, 48), (33, 51), (30, 57)], STEEL)
        a.rect((34, 6, 37, 8), BRASS_LIGHT)
        a.rect((8, 42, 10, 43), BRASS)
    elif a.kind == "save":
        a.polygon([(43, 9), (51, 17), (49, 23), (43, 16)], IVORY)
        a.polygon([(48, 35), (53, 30), (53, 53), (48, 49)], BEVEL)
        a.line([(12, 25), (12, 47)], LIGHT)
        a.line([(17, 54), (25, 54)], SHADOW, 2)
        a.rect((12, 9, 14, 11), BRASS_LIGHT)
        a.rect((41, 53, 46, 54), BRASS)
    elif a.kind == "close":
        a.polygon([(8, 10), (11, 9), (25, 22), (24, 25)], IVORY)
        a.polygon([(39, 23), (51, 9), (55, 12), (41, 27)], LIGHT)
        a.polygon([(38, 42), (52, 56), (54, 52), (41, 38)], BEVEL)
        a.polygon([(9, 51), (22, 38), (25, 42), (12, 56)], STEEL)
        a.rect((10, 7, 12, 9), BRASS_LIGHT)
        a.rect((50, 54, 52, 55), BRASS)
    elif a.kind == "resume":
        a.polygon([(16, 9), (19, 9), (48, 28), (48, 31), (16, 11)], IVORY)
        a.polygon([(20, 52), (47, 36), (49, 38), (20, 56)], STEEL)
        a.polygon([(14, 12), (16, 14), (16, 51), (14, 54)], LIGHT)
        a.rect((16, 9, 18, 11), BRASS_LIGHT)
        a.rect((18, 54, 22, 55), BRASS)
    elif a.kind == "restart":
        a.polygon([(32, 7), (45, 10), (55, 21), (48, 19), (43, 14), (33, 12)], IVORY)
        a.polygon([(49, 40), (54, 39), (50, 50), (40, 55), (32, 54), (34, 50)], STEEL)
        a.polygon([(7, 21), (22, 8), (21, 12), (11, 22)], LIGHT)
        a.rect((53, 40, 55, 42), BRASS_LIGHT)
        a.rect((20, 48, 22, 49), BRASS)
    elif a.kind == "title":
        a.polygon([(10, 28), (31, 10), (31, 15), (16, 29)], IVORY)
        a.polygon([(35, 15), (54, 29), (52, 31), (36, 20)], BEVEL)
        a.rect((16, 37, 18, 49), LIGHT)
        a.rect((45, 37, 48, 50), STEEL)
        a.rect((30, 8, 33, 10), BRASS_LIGHT)
        a.rect((41, 53, 45, 54), BRASS)
    elif a.kind == "quit":
        a.polygon([(38, 10), (48, 15), (54, 24), (50, 24), (43, 16), (37, 14)], IVORY)
        a.polygon([(8, 38), (14, 43), (22, 51), (24, 55), (14, 51), (8, 44)], STEEL)
        a.line([(29, 6), (29, 27)], IVORY)
        a.rect((50, 43, 52, 45), BRASS_LIGHT)
        a.rect((14, 20, 15, 23), BRASS)
    else:
        for fx, fy in ((False, False), (True, False), (False, True), (True, True)):
            t = lambda x, y: (63-x if fx else x, 63-y if fy else y)
            a.polygon([t(x, y) for x, y in ((6, 6), (26, 6), (26, 8), (9, 8), (9, 13), (6, 13))], IVORY)
            a.polygon([t(x, y) for x, y in ((11, 15), (14, 15), (14, 25), (11, 25))], STEEL)
            a.line([t(18, 7), t(21, 7)], BRASS_LIGHT)


def author_icons():
    result = {}
    for kind in KINDS:
        a = Icon(kind)
        d = a.shape
        if kind == "status":
            # Overhead RAICHO: long core, separate engine pods and swept wings.
            d.polygon([(31, 4), (35, 4), (40, 23), (56, 42), (54, 48),
                       (42, 44), (39, 57), (25, 57), (22, 44), (9, 48),
                       (7, 42), (24, 23)], fill=1)
            d.rectangle((25, 48, 29, 61), fill=1)
            d.rectangle((35, 48, 39, 61), fill=1)
        elif kind == "map":
            # Three physically separate folded plates; large folds survive fit.
            for panel in ([(6, 13), (20, 7), (20, 49), (6, 55)],
                          [(23, 7), (40, 13), (40, 55), (23, 49)],
                          [(43, 13), (57, 7), (57, 49), (43, 55)]):
                d.polygon(panel, fill=1)
        elif kind == "settings":
            # Explicit integer authoring avoids platform sin/round boundary drift.
            d.polygon([(54, 32), (59, 34), (59, 37), (58, 40), (57, 42), (52, 42),
                       (51, 44), (49, 46), (48, 48), (49, 53), (47, 55), (45, 56),
                       (42, 57), (38, 54), (36, 54), (34, 54), (32, 54), (29, 59),
                       (26, 59), (23, 58), (21, 57), (21, 52), (19, 51), (17, 49),
                       (15, 48), (10, 49), (8, 47), (7, 45), (6, 42), (9, 38),
                       (9, 36), (9, 34), (8, 32), (4, 29), (4, 26), (5, 23),
                       (6, 21), (11, 21), (12, 19), (14, 17), (15, 15), (14, 10),
                       (16, 8), (18, 7), (21, 6), (25, 9), (27, 9), (29, 9),
                       (31, 8), (34, 4), (37, 4), (40, 5), (42, 6), (42, 11),
                       (44, 12), (46, 14), (48, 15), (53, 14), (55, 16), (56, 18),
                       (57, 21), (54, 25), (54, 27), (54, 29)], fill=1)
            d.ellipse((22, 22, 41, 41), fill=0)
        elif kind == "save":
            d.polygon([(10, 7), (45, 7), (54, 16), (54, 56), (10, 56)], fill=1)
            d.rectangle((19, 8, 40, 24), fill=0)
            d.rectangle((16, 37, 47, 50), fill=0)
            d.rectangle((20, 42, 43, 46), fill=1)
        elif kind == "close":
            d.polygon([(11, 6), (32, 24), (52, 6), (58, 12), (40, 32),
                       (58, 52), (52, 58), (32, 40), (11, 58), (5, 52),
                       (24, 32), (5, 12)], fill=1)
        elif kind == "resume":
            d.polygon([(13, 7), (18, 7), (57, 31), (57, 34), (18, 58), (13, 58)], fill=1)
        elif kind == "restart":
            d.ellipse((7, 6, 58, 57), fill=1)
            d.ellipse((17, 16, 48, 47), fill=0)
            d.rectangle((4, 3, 33, 20), fill=0)
            d.polygon([(5, 21), (25, 4), (26, 34)], fill=1)
        elif kind == "title":
            # Haven landing hub, with roof/antenna silhouette and docking throat.
            d.polygon([(4, 28), (30, 7), (34, 7), (60, 28), (54, 34),
                       (49, 30), (49, 56), (38, 56), (38, 43), (26, 43),
                       (26, 56), (15, 56), (15, 30), (10, 34)], fill=1)
        elif kind == "quit":
            d.ellipse((6, 7, 57, 58), fill=1)
            d.ellipse((16, 17, 47, 48), fill=0)
            d.rectangle((21, 3, 42, 28), fill=0)
            d.rectangle((28, 5, 35, 34), fill=1)
        else:
            for flip_x, flip_y in ((False, False), (True, False), (False, True), (True, True)):
                arm = [(6, 6), (26, 6), (26, 13), (14, 13), (14, 26), (6, 26)]
                d.polygon([(63-x if flip_x else x, 63-y if flip_y else y) for x, y in arm], fill=1)
        a.shade()
        sports_metal(a)
        if kind == "status":
            a.polygon([(31, 11), (34, 11), (38, 27), (34, 36), (30, 36), (26, 27)], RECESS)
            a.polygon([(31, 14), (33, 14), (35, 26), (32, 30), (29, 26)], CYAN_DARK)
            a.line([(30, 16), (32, 14), (34, 23)], CYAN_LIGHT, 2, accent=True)
            for bounds in ((25, 51, 29, 58), (35, 51, 39, 58)):
                a.rect(bounds, CYAN, accent=True)
            a.rect((26, 57, 28, 59), CYAN_EDGE, emitter=True)
            a.rect((36, 57, 38, 59), CYAN_EDGE, emitter=True)
            for x in (16, 44):
                a.line([(x, 39), (x+3, 42), (x+7, 43)], STEEL, 2)
                a.line([(x+1, 39), (x+4, 40)], IVORY)
            a.line([(27, 41), (37, 41)], SHADOW, 2)
            for x in (28, 35):
                a.rivet(x, 45)
            a.rect((30, 38, 33, 39), BRASS_LIGHT)
        elif kind == "map":
            a.line([(18, 12), (18, 47)], STEEL, 2)
            a.line([(25, 12), (25, 45)], IVORY)
            a.line([(38, 18), (38, 51)], STEEL, 2)
            a.line([(45, 18), (45, 48)], IVORY)
            # Wide route intersects panels but cannot bridge the real folds.
            a.line([(10, 39), (15, 34), (31, 36), (36, 25), (51, 24)], CYAN, 3, accent=True)
            for x, y in ((12, 37), (31, 36), (50, 24)):
                a.ellipse((x-3, y-3, x+3, y+3), CYAN_DARK, accent=True)
                a.ellipse((x-1, y-1, x+1, y+1), CYAN_EDGE, emitter=True)
            for bounds in ((9, 20, 14, 21), (28, 17, 33, 18), (48, 40, 53, 41)):
                a.rect(bounds, SHADOW)
            a.rivet(12, 48)
            a.rivet(51, 15)
        elif kind == "settings":
            a.ellipse((17, 17, 46, 46), STEEL)
            a.ellipse((20, 20, 43, 43), SHADOW)
            # Bore stays transparent; explicit mask preserves its open center.
            for x, y in ((17, 31), (31, 17), (47, 31), (31, 47)):
                a.rect((x-2, y-2, x+2, y+2), CYAN_DARK, accent=True)
                a.rect((x-1, y-1, x+1, y+1), CYAN_LIGHT, emitter=True)
            for x, y in ((13, 13), (49, 13), (49, 49), (13, 49)):
                a.rivet(x, y)
            a.line([(25, 10), (37, 10)], IVORY)
            a.line([(52, 26), (52, 37)], SHADOW, 2)
        elif kind == "save":
            a.rect((13, 29, 51, 33), RECESS)
            a.rect((15, 29, 37, 30), STEEL_LIGHT)
            a.rect((44, 28, 49, 33), CYAN_DARK, accent=True)
            a.rect((45, 29, 48, 31), CYAN_EDGE, emitter=True)
            a.rect((20, 43, 42, 44), STEEL_LIGHT)
            a.line([(14, 13), (14, 22)], STEEL, 2)
            for x, y in ((14, 51), (50, 51), (45, 19)):
                a.rivet(x, y)
            a.rect((28, 53, 36, 54), BRASS)
        elif kind == "close":
            a.line([(13, 13), (50, 50)], RECESS, 4)
            a.line([(50, 13), (13, 50)], RECESS, 4)
            a.line([(14, 12), (27, 25)], CYAN, 2, accent=True)
            a.line([(49, 12), (36, 25)], CYAN, 2, accent=True)
            a.ellipse((28, 28, 35, 35), STEEL)
            a.rect((30, 30, 33, 33), CYAN_EDGE, emitter=True)
            for x, y in ((12, 52), (52, 52)):
                a.rivet(x, y)
        elif kind == "resume":
            a.line([(19, 16), (45, 32), (19, 48), (19, 16)], RECESS, 3)
            a.line([(23, 21), (40, 32), (23, 43)], CYAN, 3, accent=True)
            a.line([(24, 24), (34, 31)], CYAN_EDGE, 2, emitter=True)
            a.rect((16, 23, 17, 40), STEEL)
            for x, y in ((19, 14), (19, 51), (50, 32)):
                a.rivet(x, y)
        elif kind == "restart":
            a.line([(31, 11), (44, 14), (51, 25), (51, 38), (44, 48), (32, 53), (19, 48)], STEEL, 2)
            a.line([(43, 14), (50, 24), (52, 31)], CYAN, 3, accent=True)
            a.rect((50, 28, 52, 33), CYAN_EDGE, emitter=True)
            a.polygon([(12, 21), (22, 12), (22, 28)], RECESS)
            a.line([(13, 21), (21, 15)], CYAN_LIGHT, 2, accent=True)
            a.rivet(31, 52)
            a.rivet(11, 35)
        elif kind == "title":
            a.line([(14, 27), (32, 13), (49, 27)], STEEL, 3)
            a.line([(18, 27), (31, 17)], IVORY, 2)
            a.rect((21, 30, 43, 36), RECESS)
            a.rect((24, 31, 40, 33), CYAN_DARK, accent=True)
            a.rect((29, 31, 34, 33), CYAN_EDGE, emitter=True)
            for x in (19, 44):
                a.rect((x, 38, x+1, 49), STEEL)
                a.rivet(x, 52)
            a.rect((28, 37, 35, 38), BRASS)
        elif kind == "quit":
            a.line([(16, 18), (11, 29), (12, 40), (21, 49), (32, 53), (43, 49), (52, 38)], STEEL, 2)
            a.rect((30, 8, 32, 30), CYAN, accent=True)
            a.rect((30, 9, 32, 17), CYAN_EDGE, emitter=True)
            a.line([(49, 20), (52, 28), (52, 35)], CYAN, 2, accent=True)
            a.rivet(32, 54)
            a.rivet(10, 32)
        else:
            for fx, fy in ((False, False), (True, False), (False, True), (True, True)):
                t = lambda x, y: (63-x if fx else x, 63-y if fy else y)
                a.line([t(9, 23), t(9, 9), t(23, 9)], CYAN_DARK, 3, accent=True)
                a.line([t(9, 15), t(9, 9), t(15, 9)], CYAN_LIGHT, 2, emitter=True)
                a.line([t(16, 11), t(23, 11)], STEEL_LIGHT)
                a.rivet(*t(10, 10))
        result["icon_" + kind] = a
    return result


def validate(atlas):
    if type(atlas.get("version")) is not int or atlas["version"] != 1 or set(atlas.get("sprites", {})) != {"icon_" + k for k in KINDS}:
        raise ValueError("Expected exact version1 ten-icon catalog")
    palette = atlas.get("palette", [])
    if not 1 < len(palette) <= 64 or palette[0] != 0 or any(type(c) is not int or not 0 <= c <= 0xffffffff for c in palette):
        raise ValueError("Invalid bounded RGBA palette")
    if len(set(palette)) != len(palette) or any(c >> 24 != 255 for c in palette[1:]):
        raise ValueError("Palette is unique with opaque nonzero entries")
    counts = {}
    for name, sprite in atlas["sprites"].items():
        indices = sprite.get("indices", [])
        if sprite.get("width") != SIZE or sprite.get("height") != SIZE or len(indices) != SIZE * SIZE:
            raise ValueError("Icons must be actual64×64 authored cells")
        if any(type(i) is not int or not 0 <= i < len(palette) for i in indices):
            raise ValueError("Invalid cell palette index")
        occupied = {i for i, p in enumerate(indices) if p}
        if not 700 <= len(occupied) <= 2100:
            raise ValueError("Icon occupancy outside authored budget700..2100")
        for role in ("accent_cells", "emitter_cells"):
            values = sprite.get(role)
            if not isinstance(values, list) or any(type(i) is not int for i in values) or values != sorted(set(values)) or not set(values) <= occupied:
                raise ValueError("Role cells must be unique sorted occupied linear indices")
        if not sprite["emitter_cells"] or not set(sprite["emitter_cells"]) < set(sprite["accent_cells"]):
            raise ValueError("Nonempty emitters must be a strict accent subset")
        if len(sprite["accent_cells"]) > len(occupied) // 3:
            raise ValueError("Animation may not recolor the whole object")
        if any(indices[i] not in (CYAN_DARK, CYAN, CYAN_LIGHT, CYAN_EDGE) for i in sprite["accent_cells"]):
            raise ValueError("Accent masks may not capture immutable metal")
        if any(indices[y * SIZE + x] for y in range(SIZE) for x in range(SIZE) if x < 3 or y < 3 or x >= 61 or y >= 62):
            raise ValueError("Authored padding/containment violated")
        if any((palette[p] >> 24) != (255 if p else 0) for p in indices):
            raise ValueError("No hidden partially-transparent source cells")
        counts[name] = {"occupied": len(occupied), "accent": len(sprite["accent_cells"]), "emitter": len(sprite["emitter_cells"])}
    return counts


def render_review(icons, output):
    sheet = Image.new("RGBA", (5 * 160, 2 * 196), (8, 9, 12, 255))
    draw = ImageDraw.Draw(sheet)
    for n, (name, icon) in enumerate(icons.items()):
        x, y = n % 5 * 160, n // 5 * 196
        sheet.alpha_composite(icon.image().resize((128, 128), Image.Resampling.NEAREST), (x+16, y+12))
        draw.text((x+12, y+151), name.removeprefix("icon_"), fill=COLORS[IVORY])
        draw.text((x+12, y+169), str(sum(bool(i) for i in icon.indices)) + " cells", fill=COLORS[STEEL_LIGHT])
    write_png(output / "authoring-overview.png", sheet)
    small = Image.new("RGBA", (800, 244), (8, 9, 12, 255))
    draw = ImageDraw.Draw(small)
    for n, (name, icon) in enumerate(icons.items()):
        x = n * 80
        # A declared nearest software sample at14 logical /1x,2x,3x.
        # Enlarging each sample for review exposes lost subcell details honestly.
        for row, pixels in enumerate((14, 28, 42)):
            sampled = icon.image().resize((pixels, pixels), Image.Resampling.NEAREST)
            sampled = sampled.resize((56, 56), Image.Resampling.NEAREST)
            small.alpha_composite(sampled, (x+12, row*68+12))
        draw.text((x+4, 218), name.removeprefix("icon_"), fill=COLORS[IVORY])
    write_png(output / "compact-readability.png", small)


def author(output):
    icons = author_icons()
    palette = [sum(v << (8 * i) for i, v in enumerate(c)) for c in COLORS]
    for name, icon in icons.items():
        write_png(output / "source" / (name + ".png"), icon.image())
    manifest = {"schema": 1, "palette": {"mode": "provided", "max_colors": 64, "colors": palette},
                "defaults": {"alpha": {"mode": "exact"}, "components": {"mode": "all"}, "resize": {"mode": "none"}},
                "sprites": [{"name": name, "source": "source/" + name + ".png"} for name in icons],
                "provenance": {"source_type": "original64-cell-python-authoring", "tool": "asset-flight-icons.py", "loss": "none; exact PNG import", "style_revision": "sports-league-graphite-platinum-facets"}}
    write_json(output / "manifest.json", manifest)
    spec = importlib.util.spec_from_file_location("asset_pack", ROOT / "scripts/asset-pack.py")
    pack = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(pack)
    atlas = pack.pack(output / "manifest.json")
    for name, icon in icons.items():
        atlas["sprites"][name].update({"accent_cells": sorted(icon.accents), "emitter_cells": sorted(icon.emitters)})
    counts = validate(atlas)
    write_json(output / "atlas.json", atlas, compact=True)
    render_review(icons, output)
    write_json(output / "provenance.json", {
        "schema": 1, "authorship": "Original pixel cells authored directly by Codex with Python/Pillow shape masks and explicit pixel clusters.",
        "generation_model": None, "source_grid": [SIZE, SIZE], "palette_count": len(palette),
        "alpha_policy": "binary exact", "components": "all retained, intentional open silhouette holes", "conversion_loss": "none",
        "source_png_encoding": "filter0, explicitly stored DEFLATE blocks; decoded by Pillow; reproducible encoded bytes",
        "style_revision": "Athletic league material pass: authored alloy planes, recessed chip/vent details and fixed gold locators; neutral graphite/platinum metal replaces blue-gray wash.",
        "preserved_baseline": "e86980bbb99f3c1423900283f78e7bba2aac4b90d02b4a0855291c2e5ff8ee73: all occupancy/bounds and both animation masks unchanged",
        "atlas_sha256": digest(output / "atlas.json"), "source_sha256": {name: digest(output / "source" / (name + ".png")) for name in icons},
        "budgets": counts, "all_ten_occupied": sum(c["occupied"] for c in counts.values()),
        "animation_contract": "Host may vary only explicit accent/emitter color or emission, with fixed geometry. Metadata does not install an animation loop.",
        "review_scope": "Authoring PNGs and nearest software14/28/42px samples only; actual native extraction, GPU presentation, human/phone readability remain unmeasured."})
    return atlas


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "assets/ui/flight-icons")
    args = parser.parse_args()
    atlas = author(args.output.resolve())
    print(json.dumps(validate(atlas), indent=2))


if __name__ == "__main__":
    main()
