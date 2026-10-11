"""Original96-cell overhead RAICHO surfaces, explicit roles and exact PNG packs.

The existing48 collision masks are reference geometry, not resampled color art.
Fine canopy/plate/vent/rail/reactor clusters are authored at96. No runtime hook.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SIZE = 96
KINDS = ("pan_hikari", "renko_hibana", "hugo_mamori")
ROLES = ("metal", "recess", "glass", "main_accent", "reactor", "exhaust", "fastener")
COLORS = (
    (0, 0, 0, 0), (11, 21, 29, 255), (25, 38, 49, 255),
    (48, 62, 73, 255), (72, 93, 106, 255), (109, 135, 146, 255),
    (155, 179, 184, 255), (208, 223, 213, 255), (244, 243, 218, 255),
    (14, 48, 67, 255), (28, 92, 115, 255), (48, 139, 157, 255),
    (93, 191, 198, 255), (183, 230, 218, 255), (133, 92, 50, 255),
    (224, 181, 99, 255), (247, 228, 155, 255),
    (29, 75, 58, 255), (45, 123, 82, 255), (89, 193, 111, 255), (152, 235, 150, 255),
    (86, 33, 43, 255), (149, 53, 60, 255), (222, 84, 80, 255), (249, 158, 127, 255),
    (27, 60, 88, 255), (44, 102, 145, 255), (82, 164, 220, 255), (159, 216, 244, 255),
    (101, 165, 176, 255), (165, 215, 219, 255), (230, 243, 221, 255),
)
INK, RECESS, SHADOW, STEEL, ALLOY, LIGHT, IVORY, EDGE = range(1, 9)
GLASS_DARK, GLASS, GLASS_MID, GLASS_LIGHT, GLASS_EDGE = range(9, 14)
BRASS, GOLD, REACTOR = range(14, 17)
EXHAUST_DARK, EXHAUST, EXHAUST_EDGE = range(29, 32)


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


PACK = module("raicho_asset_pack", ROOT / "scripts/asset-pack.py")
PNG = module("raicho_png", ROOT / "scripts/asset-flight-icons.py")


def write_json(path, value, compact=False):
    path.parent.mkdir(parents=True, exist_ok=True)
    options = {"separators": (",", ":")} if compact else {"indent": 2}
    path.write_bytes((json.dumps(value, ensure_ascii=False, allow_nan=False, **options) + "\n").encode("utf-8"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def code_digest(path):
    """Source-code fingerprints use UTF-8/LF, independent of Git autocrlf."""
    data = path.read_bytes().replace(b"\r\n", b"\n")
    return hashlib.sha256(data).hexdigest()


def reference_mask(variant):
    """Integer rational form of accepted identity.rs48-cell silhouette fields."""
    result = []
    for y in range(48):
        for x in range(48):
            d = abs(2 * x - 47)
            if variant == 0:
                main = 3 <= y <= 37 and (2 * d <= y + 5 if y < 18 else 5 * d <= 95 - 3 * abs(y - 25))
                on = main or (24 <= y <= 40 and 18 <= d <= 28) or (29 <= y <= 32 and d <= 30) or (38 <= y <= 44 and 6 <= d <= 12)
            elif variant == 1:
                main = 2 <= y <= 43 and (50 * d <= 150 + 17 * y if y < 25 else 5 * d <= 65 - 2 * (y - 25))
                on = main or (18 <= y <= 37 and d > 14 and 10 * d <= 320 - 13 * abs(y - 27)) or (12 <= y <= 32 and 22 <= d <= 26) or (28 <= y <= 30 and d <= 32) or (36 <= y <= 43 and 6 <= d <= 14)
            else:
                main = 5 <= y <= 39 and d <= (y + 2 if y < 16 else 25)
                on = main or (16 <= y <= 36 and 28 <= d <= 38) or (22 <= y <= 27 and d <= 41) or (38 <= y <= 43 and 10 <= d <= 18)
            result.append(bool(on))
    return result


def fine_mask(variant):
    coarse = reference_mask(variant)
    mask = [coarse[(y // 2) * 48 + x // 2] for y in range(SIZE) for x in range(SIZE)]
    if variant == 2:
        # A declared one-fine-cell perimeter inset keeps the broad shield frame
        # below4k. Preserve four outer extrema, connected to the remaining roof.
        prior = mask[:]
        at = lambda x, y: 0 <= x < SIZE and 0 <= y < SIZE and prior[y * SIZE + x]
        mask = [on and all(at(x + dx, y + dy) for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)))
                for y in range(SIZE) for x in range(SIZE) for on in [prior[y * SIZE + x]]]
        points = [(i % SIZE, i // SIZE) for i, on in enumerate(prior) if on]
        for axis, fn in ((0, min), (0, max), (1, min), (1, max)):
            boundary = fn(p[axis] for p in points)
            inward = 1 if fn is min else -1
            candidates = []
            for p in points:
                q = list(p)
                q[axis] += inward
                if p[axis] == boundary and mask[q[1] * SIZE + q[0]]:
                    candidates.append(p)
            point = min(candidates, key=lambda p: abs(p[1-axis] * 2 - 95))
            mask[point[1] * SIZE + point[0]] = True
    return mask


class Hull:
    def __init__(self, variant):
        self.variant = variant
        self.mask = fine_mask(variant)
        self.indices = [0] * (SIZE * SIZE)
        self.roles = [None] * (SIZE * SIZE)
        self.fine: set[int] = set()
        self.signals: set[int] = set()
        self.corp = 17 + variant * 4
        for y in range(SIZE):
            for x in range(SIZE):
                i = y * SIZE + x
                if not self.mask[i]:
                    continue
                at = lambda xx, yy: 0 <= xx < SIZE and 0 <= yy < SIZE and self.mask[yy * SIZE + xx]
                edge = not at(x - 1, y) or not at(x, y - 1)
                shadow = not at(x + 1, y) or not at(x, y + 1)
                color = IVORY if edge else SHADOW if shadow else ALLOY if x < 48 else STEEL
                if not edge and not shadow and 37 <= x <= 58:
                    color = LIGHT if x < 48 else ALLOY
                self.indices[i], self.roles[i] = color, "metal"

    def paint(self, shape, color, role="metal", *, fine=True, signal=False):
        layer = Image.new("1", (SIZE, SIZE))
        shape(ImageDraw.Draw(layer))
        for i, on in enumerate(layer.get_flattened_data()):
            if on and self.mask[i]:
                self.indices[i], self.roles[i] = color, role
                self.signals.discard(i)
                if fine:
                    self.fine.add(i)
                if signal:
                    self.signals.add(i)

    def rect(self, rect, color, role="metal", **options):
        self.paint(lambda d: d.rectangle(rect, fill=1), color, role, **options)

    def line(self, points, color, role="metal", width=1, **options):
        self.paint(lambda d: d.line(points, fill=1, width=width), color, role, **options)

    def polygon(self, points, color, role="metal", **options):
        self.paint(lambda d: d.polygon(points, fill=1), color, role, **options)

    def bolt(self, x, y):
        self.rect((x - 1, y - 1, x + 1, y + 1), RECESS, "recess")
        self.rect((x - 1, y - 1, x, y), BRASS, "fastener")
        self.rect((x, y, x, y), EDGE, "fastener")

    def sprite(self, overlay=False):
        cells = [c if i in self.fine else 0 for i, c in enumerate(self.indices)] if overlay else self.indices
        occupied = {i for i, c in enumerate(cells) if c}
        return {"width": SIZE, "height": SIZE, "indices": cells,
                "material_cells": {role: [i for i in sorted(occupied) if self.roles[i] == role] for role in ROLES},
                "signal_cells": sorted(self.signals & occupied),
                "fine_cells": sorted(self.fine & occupied)}

    def image(self, overlay=False):
        image = Image.new("RGBA", (SIZE, SIZE))
        image.putdata([COLORS[i] for i in self.sprite(overlay)["indices"]])
        return image


def author_hulls():
    hulls = {}
    for variant, kind in enumerate(KINDS):
        a = Hull(variant)
        c = a.corp
        # Longitudinal top plate, recessed hinge and flush aft reactor.
        a.line([(46, 43), (46, 73)], IVORY)
        a.line([(49, 43), (49, 73)], RECESS, "recess")
        a.rect((43, 48, 52, 64), SHADOW, "recess")
        a.rect((44, 49, 51, 63), BRASS, "reactor")
        a.rect((45, 50, 50, 60), GOLD, "reactor")
        for y in (51, 54, 57):
            a.rect((46, y, 48, y), REACTOR, "reactor", signal=True)
        a.rect((45, 62, 50, 62), RECESS, "recess")
        a.line([(43, 45), (40, 47), (40, 68)], SHADOW, "recess")
        a.line([(52, 45), (55, 47), (55, 68)], SHADOW, "recess")
        # Corporate roof trims are explicit roles, never a hue-based recolor.
        if variant == 0:
            for x in (22, 70):
                a.rect((x, 51, x + 3, 75), c, "main_accent")
                a.line([(x + 1, 53), (x + 1, 70)], c + 2, "main_accent")
                a.rect((x + 1, 54, x + 2, 55), c + 3, "main_accent", signal=True)
            for x in (28, 61):
                a.line([(x, 57), (x + 5, 63), (x + 5, 67)], IVORY)
            rim = [(43, 21), (52, 21), (57, 35), (53, 43), (42, 43), (38, 35)]
            glass = [(44, 24), (51, 24), (54, 35), (51, 40), (44, 40), (41, 35)]
        elif variant == 1:
            for x in (22, 70):
                a.rect((x, 25, x + 3, 55), RECESS, "recess")
                a.rect((x + 1, 26, x + 2, 50), LIGHT)
                a.line([(x + 1, 28), (x + 1, 41)], EDGE)
                a.rect((x + 1, 43, x + 2, 46), BRASS, "fastener")
            for x in (34, 59):
                a.line([(x, 43), (x, 55), (x - 2 if x < 48 else x + 2, 60)], c, "main_accent", width=3)
                a.line([(x, 45), (x, 53)], c + 2, "main_accent")
                a.rect((x, 46, x, 47), c + 3, "main_accent", signal=True)
            rim = [(44, 22), (51, 22), (54, 32), (52, 42), (43, 42), (41, 32)]
            glass = [(45, 25), (50, 25), (51, 32), (50, 39), (45, 39), (44, 32)]
        else:
            for x in (12, 80):
                a.rect((x, 37, x + 3, 67), c, "main_accent")
                a.line([(x + 1, 39), (x + 1, 64)], c + 2, "main_accent")
                a.rect((x + 1, 41, x + 2, 42), c + 3, "main_accent", signal=True)
            for x in (22, 68):
                a.rect((x, 43, x + 5, 46), SHADOW, "recess")
                a.rect((x + 1, 44, x + 4, 44), LIGHT)
            rim = [(41, 22), (54, 22), (58, 31), (55, 42), (40, 42), (37, 31)]
            glass = [(42, 25), (53, 25), (55, 31), (53, 39), (42, 39), (40, 31)]
        # Distinct recessed alloy assemblies subdivide broad planes without
        # random grain: PAN cooling pods, RENKO rail saddles, HUGO shield decks.
        for mirrored in (False, True):
            p = lambda points: [(95-x if mirrored else x, y) for x, y in points]
            if variant == 0:
                a.line(p([(19, 50), (19, 74), (21, 77)]), LIGHT)
                a.line(p([(27, 50), (27, 74), (25, 77)]), SHADOW, "recess")
                for y in (59, 63, 67, 71):
                    a.line(p([(20, y), (21, y)]), IVORY)
                    a.line(p([(26, y + 1), (27, y + 1)]), RECESS, "recess")
                a.bolt(95 - 20 if mirrored else 20, 53)
            elif variant == 1:
                a.polygon(p([(30, 44), (37, 47), (37, 62), (32, 69), (30, 63)]), SHADOW, "recess")
                a.polygon(p([(31, 46), (35, 49), (35, 61), (32, 66), (31, 62)]), ALLOY)
                a.line(p([(31, 48), (31, 61)]), IVORY)
                for y in (52, 55, 58):
                    a.line(p([(32, y), (34, y)]), RECESS, "recess")
                a.bolt(95 - 32 if mirrored else 32, 48)
            else:
                a.polygon(p([(24, 36), (31, 39), (31, 68), (28, 74), (24, 74)]), SHADOW, "recess")
                a.polygon(p([(25, 39), (29, 42), (29, 67), (27, 72), (25, 72)]), ALLOY)
                a.line(p([(25, 40), (25, 65)]), LIGHT)
                for y in (48, 52, 56, 60):
                    a.line(p([(26, y), (28, y)]), RECESS, "recess")
                a.bolt(95 - 27 if mirrored else 27, 44)
        # Flush canopy: alloy surround, black seal, deep blue glass and fine glints.
        a.polygon(rim, RECESS, "recess")
        a.line(rim + [rim[0]], IVORY)
        a.polygon(glass, GLASS_DARK, "glass")
        a.line([(45, 27), (44, 34), (46, 38)], GLASS, "glass", width=2)
        a.line([(48, 27), (49, 36)], GLASS_MID, "glass", width=2)
        a.line([(45, 27), (45, 31)], GLASS_LIGHT, "glass")
        a.rect((45, 27, 46, 27), GLASS_EDGE, "glass")
        a.line([(44, 40), (51, 40)], SHADOW, "recess")
        # Cooling cassettes with one-cell ribs and recessed alternating channels.
        for x in (33, 57):
            a.rect((x, 54, x + 5, 67), SHADOW, "recess")
            a.rect((x + 1, 55, x + 4, 66), RECESS, "recess")
            for y in range(56, 67, 2):
                a.rect((x + 1, y, x + 4, y), LIGHT)
                a.rect((x + 1, y, x + 1, y), IVORY)
        # Aft nozzle lips and real opaque emitter cells (no spawned plume in pack).
        for x in ((38, 54) if variant < 2 else (31, 60)):
            a.rect((x, 78, x + 3, 85), RECESS, "recess")
            a.rect((x + 1, 80, x + 2, 84), EXHAUST_DARK, "exhaust")
            a.rect((x + 1, 81, x + 2, 83), EXHAUST, "exhaust", signal=True)
            a.rect((x + 1, 81, x + 2, 81), EXHAUST_EDGE, "exhaust", signal=True)
        # Small instrument hinge, stabilizer seams and actual three-cell bolts.
        for x, y in ((42, 46), (53, 46), (30, 61), (65, 61), (41, 71), (54, 71)):
            a.bolt(x, y)
        a.line([(44, 70), (51, 70)], SHADOW, "recess")
        a.rect((46, 72, 49, 73), c + 1, "main_accent")
        a.rect((47, 72, 48, 72), c + 3, "main_accent", signal=True)
        if variant == 1:
            # Separate scarlet rail collars remain legible beside the alloy
            # saddles; never tint the neutral canopy, brass or cooling ribs.
            for x in (22, 71):
                a.rect((x, 47, x + 1, 62), c, "main_accent")
                a.line([(x, 48), (x, 58)], c + 2, "main_accent")
                a.rect((x, 49, x, 50), c + 3, "main_accent", signal=True)
            for x in (31, 63):
                a.line([(x, 63), (x, 66)], c + 1, "main_accent")
        hulls[kind] = a
    return hulls


def validate(atlas):
    PACK.validate(atlas)
    if type(atlas.get("version")) is not int or atlas["version"] != 1 or set(atlas.get("sprites", {})) != {k + suffix for k in KINDS for suffix in ("", "_fine")}:
        raise ValueError("Expected version1 exact three-body/three-overlay catalog")
    palette = atlas.get("palette", [])
    if palette != [sum(c << (8 * i) for i, c in enumerate(rgba)) for rgba in COLORS]:
        raise ValueError("Exact shared32-role palette required")
    counts = {}
    for variant, kind in enumerate(KINDS):
        coarse = reference_mask(variant)
        for suffix in ("", "_fine"):
            name, sprite = kind + suffix, atlas["sprites"][kind + suffix]
            cells = sprite.get("indices", [])
            if sprite.get("width") != SIZE or sprite.get("height") != SIZE or len(cells) != SIZE * SIZE or any(type(i) is not int or not 0 <= i < len(palette) for i in cells):
                raise ValueError("Invalid96-cell dimensions/palette indices")
            occupied = {i for i, c in enumerate(cells) if c}
            if not occupied or len(occupied) > (1800 if suffix else 4000):
                raise ValueError("Body/overlay occupied budget exceeded")
            if any(not coarse[(i // SIZE // 2) * 48 + i % SIZE // 2] for i in occupied):
                raise ValueError("Fine surface extends outside original48 occupied envelope")
            roles = sprite.get("material_cells", {})
            if set(roles) != set(ROLES):
                raise ValueError("Expected complete explicit material roles")
            accounted = set()
            allowed_colors = {"metal": set(range(SHADOW, EDGE + 1)), "recess": {RECESS, SHADOW},
                              "glass": set(range(GLASS_DARK, GLASS_EDGE + 1)),
                              "main_accent": set(range(17 + variant * 4, 21 + variant * 4)),
                              "reactor": {BRASS, GOLD, REACTOR},
                              "exhaust": {EXHAUST_DARK, EXHAUST, EXHAUST_EDGE},
                              "fastener": {BRASS, EDGE}}
            for role in ROLES:
                entries = roles[role]
                if not isinstance(entries, list) or any(type(i) is not int for i in entries) or entries != sorted(set(entries)) or not set(entries) <= occupied or accounted & set(entries):
                    raise ValueError("Material masks must partition occupied cells uniquely")
                accounted.update(entries)
                if any(cells[i] not in allowed_colors[role] for i in entries):
                    raise ValueError("Material mask contains a color from another role")
            if accounted != occupied:
                raise ValueError("Unclassified occupied material")
            for role in ("signal_cells", "fine_cells"):
                entries = sprite.get(role)
                if not isinstance(entries, list) or any(type(i) is not int for i in entries) or entries != sorted(set(entries)) or not set(entries) <= occupied:
                    raise ValueError("Invalid bounded occupied signal/fine mask")
            signals = set(sprite["signal_cells"])
            if not signals or not signals <= set(roles["main_accent"]) | set(roles["reactor"]) | set(roles["exhaust"]):
                raise ValueError("Signals may not animate metal/glass/fasteners")
            if any(cells[i] not in range(17 + variant * 4, 21 + variant * 4) for i in roles["main_accent"]):
                raise ValueError("Corporate material cannot inherit another manufacturer")
            if suffix:
                original = atlas["sprites"][kind]
                if occupied != set(original["fine_cells"]) or any(cells[i] != original["indices"][i] for i in occupied):
                    raise ValueError("Overlay must preserve exact full-body fine cells")
                if signals != set(original["signal_cells"]) & occupied or any(
                        set(roles[role]) != set(original["material_cells"][role]) & occupied for role in ROLES):
                    raise ValueError("Overlay material/signal roles must match the full surface")
            else:
                topology = PACK.topology(cells, SIZE, SIZE)
                # RENKO's accepted rail apertures are intentional voids, not
                # damaged material. Other intact bodies have no interior holes.
                expected_holes = 96 if variant == 1 else 0
                if topology["components"] != 1 or topology["hole_cells"] != expected_holes:
                    raise ValueError("Intact connected hull/declared rail apertures changed")
            counts[name] = {"occupied": len(occupied), "signals": len(signals), "fine": len(sprite["fine_cells"]),
                            "materials": {role: len(roles[role]) for role in ROLES}}
    return counts


def render_review(hulls, output):
    # Exact1:1 top row, explicit2x source review below. This is software authoring.
    image = Image.new("RGBA", (3 * 232, 360), (8, 15, 22, 255))
    draw = ImageDraw.Draw(image)
    for n, (kind, hull) in enumerate(hulls.items()):
        x = n * 232
        draw.text((x + 12, 8), kind.upper(), fill=COLORS[IVORY])
        image.alpha_composite(hull.image(), (x + 68, 30))
        image.alpha_composite(hull.image().resize((192, 192), Image.Resampling.NEAREST), (x + 20, 151))
        draw.text((x + 12, 132), "96 source / " + str(sum(hull.mask)) + " cells", fill=COLORS[LIGHT])
    PNG.write_png(output / "authoring-overview.png", image)
    compact = Image.new("RGBA", (3 * 270, 414), (8, 15, 22, 255))
    draw = ImageDraw.Draw(compact)
    for n, (kind, hull) in enumerate(hulls.items()):
        x = n * 270
        draw.text((x + 10, 8), kind.upper(), fill=COLORS[IVORY])
        for row, size in enumerate((29, 48, 77)):
            sampled = hull.image().resize((size, size), Image.Resampling.NEAREST)
            sampled = sampled.resize((116, 116), Image.Resampling.NEAREST)
            compact.alpha_composite(sampled, (x + 12, 28 + row * 126))
            overlay = hull.image(True).resize((size, size), Image.Resampling.NEAREST).resize((116, 116), Image.Resampling.NEAREST)
            compact.alpha_composite(overlay, (x + 142, 28 + row * 126))
            draw.text((x + 10, 143 + row * 126), f"{size}px full | fine overlay", fill=COLORS[LIGHT])
    PNG.write_png(output / "compact-review.png", compact)


def author(output):
    hulls = author_hulls()
    for kind, hull in hulls.items():
        for suffix in ("", "_fine"):
            PNG.write_png(output / "source" / (kind + suffix + ".png"), hull.image(bool(suffix)))
    palette = [sum(c << (8 * i) for i, c in enumerate(rgba)) for rgba in COLORS]
    names = [kind + suffix for kind in KINDS for suffix in ("", "_fine")]
    manifest = {"schema": 1, "palette": {"mode": "provided", "max_colors": 32, "colors": palette},
                "budgets": {"sprite_cells": 4000, "group_cells": 4000, "group_transforms": 4},
                "defaults": {"alpha": {"mode": "exact"}, "components": {"mode": "all"}, "resize": {"mode": "none"}},
                "sprites": [{"name": name, "source": "source/" + name + ".png", "topology": "holes-allowed" if name.endswith("_fine") or name == "renko_hibana" else "solid"} for name in names],
                "groups": [{"name": kind + suffix + "_view", "parts": [{"name": "surface", "sprite": kind + suffix,
                             "transform": {"position": [0, 0], "basis_x": [0.5, 0], "basis_y": [0, 0.5]}}]} for kind in KINDS for suffix in ("", "_fine")],
                "provenance": {"source_type": "original96-cell-python-authoring", "loss": "none; exact authored PNG import"}}
    write_json(output / "manifest.json", manifest)
    atlas = PACK.pack(output / "manifest.json")
    for kind, hull in hulls.items():
        for suffix in ("", "_fine"):
            atlas["sprites"][kind + suffix].update(hull.sprite(bool(suffix)))
    atlas["raicho_detail"] = {
        "status": "proposed source art; no runtime/collision integration",
        "source_size": [96, 96], "relative_cell_scale": 0.5, "origin": "centered PixelBody",
        "coarse_cell_mapping": "[floor(source_x/2),floor(source_y/2)] into unchanged48 collision grid",
        "full_body_policy": "Standalone presentation proposal; not automatically the collision body",
        "overlay_policy": "Sparse fine cells after retained48 body; host must mask all fine cells whose coarse cell is destroyed",
        "animation_policy": "Host may animate explicit signal colors on real state; fixed occupancy; Reduced Motion freezes cosmetic phase",
        "hugo_surface_policy": "One-fine-cell perimeter inset with four connected extrema retained; original world envelope is not enlarged",
        "intentional_apertures": "RENKO preserves accepted24 coarse rail-aperture void cells as96 fine cells; other full bodies have no enclosed voids",
    }
    atlas["asset_metadata"]["opaque_extension_fields"] = ["raicho_detail"]
    core = {k: atlas[k] for k in ("version", "palette", "sprites")}
    atlas["asset_metadata"]["cell_content_sha256"] = hashlib.sha256(
        json.dumps(core, sort_keys=True, separators=(",", ":")).encode("utf-8")).hexdigest()
    counts = validate(atlas)
    write_json(output / "atlas.json", atlas, compact=True)
    render_review(hulls, output)
    write_json(output / "provenance.json", {
        "schema": 1, "authorship": "Original96-cell overhead material planes, canopy clusters, vents, rails, reactor windows and fasteners authored by Codex in Python.",
        "generation_model": None, "source_geometry": "Integer rational reconstruction of accepted48 starter silhouette fields; no source RGBA resizing.",
        "reference_geometry_commit": "f69d6cbe8d58d182da6e45460b42b963655c5fbc",
        "reference_geometry_path": "crates/game_core/src/identity.rs:STARTERS",
        "new_detail": "Fine cluster geometry/colors authored directly at96; not doubled48 color blocks.",
        "conversion_loss": "none between authored native-grid PNG and packed cells; proposed Hugo surface has declared fine perimeter inset",
        "source_grid": [96, 96], "reference48_occupied": [sum(reference_mask(v)) for v in range(3)],
        "palette_count": len(palette), "alpha_policy": "exact binary", "components": "all retained",
        "source_png_encoding": "fixed filter0 / stored DEFLATE, independent of linked compressor",
        "atlas_sha256": digest(output / "atlas.json"),
        "source_sha256": {name: digest(output / "source" / (name + ".png")) for name in names},
        "code_fingerprint_policy": "UTF-8 source bytes with CRLF normalized to LF; PNG, manifest and atlas hashes retain exact raw bytes",
        "authoring_inputs": {name: code_digest(ROOT / "scripts" / name) for name in ("asset-raicho-fine.py", "asset-pack.py", "asset-flight-icons.py")},
        "budgets": counts,
        "review_scope": "Source-native PNG cells and declared nearest29/48/77px software samples only; no native extraction, GPU, animation, damage integration, human or device acceptance.",
    })
    return atlas


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "assets/pixels/raicho-fine-detail")
    args = parser.parse_args()
    atlas = author(args.output.resolve())
    print(json.dumps(validate(atlas), indent=2))


if __name__ == "__main__":
    main()
