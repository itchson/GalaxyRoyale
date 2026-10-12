"""Reproducible DORA reference conversion and original 32-cell control icons.

Loss is explicit: first expression crop, alpha128, nearest96x128, 47-color
median-cut. Eye/mouth layers are authored after conversion. asset-pack then
imports every authored PNG exactly; no component filtering or hidden resize.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path

from PIL import Image, ImageDraw, __version__ as pillow_version

SCRIPTS = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("asset_pack", SCRIPTS / "asset-pack.py")
asset_pack = importlib.util.module_from_spec(spec)
spec.loader.exec_module(asset_pack)
WIDTH, HEIGHT = 96, 128
TRANSPARENT = (0, 0, 0, 0)
SKIN = (231, 202, 175, 255)
SKIN_SHADE = (180, 130, 108, 255)
INK = (9, 16, 20, 255)
GOLD = (235, 179, 39, 255)
GOLD_LIGHT = (255, 239, 151, 255)
WHITE = (245, 235, 224, 255)
MOUTH = (90, 38, 43, 255)
ICON_COLORS = [(217, 234, 246, 255), (255, 255, 255, 255),
               (95, 117, 126, 255), (42, 56, 65, 255),
               (99, 133, 164, 255), (88, 222, 239, 255)]
EYES = [(26, 52, 40, 62), (48, 50, 64, 60)]
MOUTH_RECT = (40, 69, 51, 77)


def fingerprint(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def original_icons():
    """Canonical silhouettes are authored at32, not enlarged old ASCII icons."""
    icons = {}
    for kind in ["status", "map", "settings", "save", "close", "resume", "restart", "title", "quit", "fullscreen"]:
        mask = Image.new("1", (32, 32))
        d = ImageDraw.Draw(mask)
        if kind == "status":
            d.polygon([(16, 2), (22, 13), (28, 23), (23, 26), (19, 22),
                       (19, 29), (12, 29), (12, 22), (8, 26), (3, 23), (10, 13)], fill=1)
        elif kind == "map":
            d.polygon([(2, 7), (11, 3), (21, 7), (29, 3), (29, 25),
                       (21, 29), (11, 25), (2, 29)], fill=1)
        elif kind == "settings":
            for y in range(32):
                for x in range(32):
                    px, py = x - 15.5, y - 15.5
                    radius = math.hypot(px, py)
                    angle = math.atan2(py, px)
                    tooth = abs(math.sin(angle * 4)) < .62
                    if 5.6 <= radius <= (15 if tooth else 12.6):
                        mask.putpixel((x, y), 1)
        elif kind == "save":
            d.polygon([(4, 3), (24, 3), (28, 7), (28, 28), (4, 28)], fill=1)
        elif kind == "close":
            d.polygon([(5, 3), (16, 12), (26, 3), (29, 6), (20, 16),
                       (29, 26), (26, 29), (16, 20), (5, 29), (2, 26),
                       (12, 16), (2, 6)], fill=1)
        elif kind == "resume":
            d.polygon([(6, 3), (29, 16), (6, 29)], fill=1)
        elif kind == "restart":
            d.ellipse((3, 3, 29, 29), fill=1)
            d.ellipse((8, 8, 24, 24), fill=0)
            d.rectangle((1, 1, 15, 10), fill=0)
            d.polygon([(2, 12), (13, 2), (13, 17)], fill=1)
        elif kind == "title":
            d.polygon([(2, 14), (16, 2), (30, 14), (26, 17), (26, 29),
                       (18, 29), (18, 21), (13, 21), (13, 29), (5, 29), (5, 17)], fill=1)
        elif kind == "fullscreen":
            arm = [(3, 3), (13, 3), (13, 7), (7, 7), (7, 13), (3, 13)]
            for flip_x, flip_y in [(False, False), (True, False), (False, True), (True, True)]:
                d.polygon([(31-x if flip_x else x, 31-y if flip_y else y) for x, y in arm], fill=1)
        else:
            d.ellipse((3, 3, 29, 29), fill=1)
            d.ellipse((8, 8, 24, 24), fill=0)
            d.rectangle((11, 1, 21, 12), fill=0)
            d.rectangle((14, 2, 18, 16), fill=1)
        im = Image.new("RGBA", (32, 32))
        for y in range(32):
            for x in range(32):
                if not mask.getpixel((x, y)):
                    continue
                edge = any(not (0 <= nx < 32 and 0 <= ny < 32) or not mask.getpixel((nx, ny))
                           for nx, ny in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)])
                role = 1 if edge and x+y < 31 else 2 if edge else 0
                if kind == "status":
                    if 13 <= x <= 18 and 11 <= y <= 18:
                        role = 5 if x+y < 31 else 3
                    elif y >= 24 and 14 <= x <= 17:
                        role = 4
                    elif y in (19, 22) and x in range(7, 25):
                        role = 3
                elif kind == "map":
                    if x in (10, 11, 20, 21):
                        role = 3
                    elif (x, y) in [(6, 13), (16, 18), (25, 11), (25, 23)]:
                        role = 5
                    elif y == 18 and 5 <= x <= 25 or x == 25 and 11 <= y <= 23:
                        role = 4
                elif kind == "settings":
                    if 7 <= math.hypot(x-15.5, y-15.5) <= 8.7:
                        role = 3
                    elif (x, y) in [(7, 15), (24, 15), (15, 7), (15, 24)]:
                        role = 5
                    elif (x+y) % 9 == 0 and not edge:
                        role = 4
                elif kind == "save":
                    if 8 <= x <= 22 and 4 <= y <= 12:
                        role = 3 if x < 18 else 5
                    elif 8 <= x <= 24 and 18 <= y <= 26:
                        role = 4 if y in (21, 24) else 3
                    elif x in (6, 26) and y in (15, 26):
                        role = 1
                elif kind == "fullscreen":
                    if min(x, 31-x) == 5 or min(y, 31-y) == 5:
                        role = 5
                    elif not edge and (x+y) % 3 == 0:
                        role = 4
                else:
                    if not edge and (x-y) % 11 == 0:
                        role = 4
                    if not edge and x+y < 26:
                        role = 5
                im.putpixel((x, y), ICON_COLORS[role])
        icons[f"icon_{kind}"] = im
    return icons


def prepare_reference(source):
    im = Image.open(source).convert("RGBA")
    if im.size != (1881, 836):
        raise ValueError("Expected reviewed1881x836 expression sheet; recrop explicitly for another source")
    crop_rect = (0, 0, 627, 836)
    crop = im.crop(crop_rect)
    hard = crop.getchannel("A").point(lambda value: 255 if value >= 128 else 0)
    bbox = hard.getbbox()
    if bbox is None:
        raise ValueError("Reference crop contains no occupied pixels")
    changed_alpha = sum(a not in (0, 255) for a in crop.getchannel("A").get_flattened_data())
    crop.putalpha(hard)
    fitted = crop.crop(bbox).resize((WIDTH, HEIGHT), Image.Resampling.NEAREST)
    occupied = [p[:3] for p in fitted.get_flattened_data() if p[3]]
    samples = Image.new("RGB", (len(occupied), 1))
    samples.putdata(occupied)
    quantized = samples.quantize(colors=47, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).convert("RGB")
    q = iter(quantized.get_flattened_data())
    result = Image.new("RGBA", fitted.size)
    result.putdata([(*next(q), 255) if p[3] else TRANSPARENT for p in fitted.get_flattened_data()])
    color_changes = sum(p != q for p, q in zip(fitted.get_flattened_data(), result.get_flattened_data()) if p[3])
    return result, {"source_dimensions": list(im.size), "crop": list(crop_rect), "tight_bbox": list(bbox),
                    "alpha_threshold": 128, "source_nonbinary_alpha_cells": changed_alpha,
                    "source_hard_occupied_cells": sum(a != 0 for a in hard.get_flattened_data()),
                    "output_dimensions": [WIDTH, HEIGHT], "nearest_resize": True,
                    "occupied_after_resize": len(occupied), "quantize_colors": 47,
                    "dither": False, "changed_occupied_color_cells": color_changes,
                    "component_policy": "all; no component removal"}


def portrait_layers(base):
    layers = {"dora_body": base.copy()}
    body_draw = ImageDraw.Draw(layers["dora_body"])
    for rect in EYES + [MOUTH_RECT]:
        body_draw.rectangle((rect[0], rect[1], rect[2]-1, rect[3]-1), fill=TRANSPARENT)
    for expression in ["neutral", "focused", "concerned", "pleased", "blink"]:
        eye = Image.new("RGBA", base.size)
        d = ImageDraw.Draw(eye)
        for rect in EYES:
            eye.paste(base.crop(rect), rect[:2])
        if expression == "blink":
            for mid_x, mid_y, below in [(33, 57, 62), (55, 55, 60)]:
                for y in range(mid_y-3, mid_y+3):
                    for x in range(mid_x-6, mid_x+6):
                        if abs(x-mid_x)/6 + abs(y-mid_y)/3 < 1.5:
                            d.point((x, y), fill=base.getpixel((x, below)))
                d.line([(mid_x-5, mid_y), (mid_x+4, mid_y+1)], fill=INK, width=1)
        else:
            # Small deliberate golden clusters/pupils retain the defining eyes.
            for x, y in [(33, 57), (55, 55)]:
                d.rectangle((x-2, y-1, x+2, y+1), fill=GOLD)
                d.point((x-1, y-1), fill=GOLD_LIGHT)
                d.point((x+1, y), fill=INK)
                d.point((x-3, y), fill=WHITE)
            if expression == "focused":
                d.line([(27, 55), (37, 56)], fill=INK, width=1)
                d.line([(49, 53), (61, 52)], fill=INK, width=1)
            elif expression == "concerned":
                d.line([(27, 54), (37, 52)], fill=INK, width=1)
                d.line([(49, 50), (61, 52)], fill=INK, width=1)
            elif expression == "pleased":
                d.line([(29, 60), (37, 60)], fill=SKIN_SHADE, width=1)
                d.line([(50, 58), (59, 58)], fill=SKIN_SHADE, width=1)
        layers[f"dora_eyes_{expression}"] = eye
    for pose in ["rest", "small", "open", "smile"]:
        mouth = Image.new("RGBA", base.size)
        mouth.paste(base.crop(MOUTH_RECT), MOUTH_RECT[:2])
        d = ImageDraw.Draw(mouth)
        # Preserve local face shading; erase only the original lip cluster.
        for y in range(71, 74):
            for x in range(42, 49):
                d.point((x, y), fill=base.getpixel((41 if x < 46 else 50, y)))
        if pose == "rest":
            d.line([(42, 72), (48, 72)], fill=SKIN_SHADE, width=1)
            d.point((45, 73), fill=MOUTH)
        elif pose == "small":
            d.ellipse((43, 71, 48, 74), fill=MOUTH)
            d.line([(44, 71), (47, 71)], fill=WHITE, width=1)
        elif pose == "open":
            d.ellipse((42, 70, 49, 75), fill=MOUTH)
            d.line([(43, 71), (48, 71)], fill=WHITE, width=1)
            d.line([(44, 74), (47, 74)], fill=SKIN_SHADE, width=1)
        else:
            d.line([(41, 71), (44, 73), (47, 73), (49, 71)], fill=MOUTH, width=1)
            d.line([(43, 72), (48, 72)], fill=WHITE, width=1)
        layers[f"dora_mouth_{pose}"] = mouth
    return layers


def author(source, output):
    output.mkdir(parents=True, exist_ok=True)
    base, conversion = prepare_reference(source)
    layers = portrait_layers(base)
    layers.update(original_icons())
    source_folder = output / "source"
    source_folder.mkdir(exist_ok=True)
    for name, im in layers.items():
        im.save(source_folder / f"{name}.png")
    palette = [0] + sorted({asset_pack.color(p) for im in layers.values() for p in im.get_flattened_data() if p[3]})
    if len(palette) > 64:
        raise ValueError(f"Authored palette exceeds64: {len(palette)}")
    hair = []
    for i, (r, g, b, a) in enumerate(layers["dora_body"].get_flattened_data()):
        x, y = i % WIDTH, i // WIDTH
        if a and g > r + 7 and g > b + 5 and (y < 104 or 20 <= x <= 78):
            hair.append(i)
    signals = [i for i, (r, g, b, a) in enumerate(layers["dora_body"].get_flattened_data())
               if a and r > 170 and r > g * 2 and r > b * 1.5 and i // WIDTH < 38]
    horns = [i for i, (r, g, b, a) in enumerate(layers["dora_body"].get_flattened_data())
             if a and r > g + 20 and r > b + 10 and i // WIDTH < 38]
    provenance = {"character": "DORA-PAN-RENKO-1337", "short_name": "DPR#1337",
                  "main_corporation": "PAN", "sub_corporation": "RENKO", "generation_tool": "image_gen.imagegen",
                  "model": None, "model_note": "Model identifier not supplied with session artifact",
                  "reference_path": "assets/concepts/kikai/dora-reference-expression-sheet.png",
                  "reference_sha256": fingerprint(source), "user_original_reference_sha256":
                  "8a47a42eae2e4bf96926d659d917d66bfd58a8b872b694632f166e93fdc8de00",
                  "pillow": pillow_version, "conversion": conversion,
                  "manual_authoring": "Named eye/mouth patch states and original32x32 six-role control masks; no SVG",
                  "runtime": "Cached PixelBody layers; exactPNG import through asset-pack after explicit authoring loss"}
    manifest = {"schema": 1, "palette": {"mode": "provided", "max_colors": 64, "colors": palette},
                "sprites": [{"name": name, "source": f"source/{name}.png", "topology": "holes-allowed"}
                            for name in layers], "provenance": provenance,
                "extensions": {"portrait_roles": {"hair_cells": hair, "horn_cells": horns, "signal_cells": signals,
                               "source_size": [WIDTH, HEIGHT], "default_main": "PAN", "default_sub": "RENKO"}}}
    asset_pack.write_json(output / "manifest.json", manifest)
    atlas = asset_pack.pack(output / "manifest.json")
    asset_pack.write_json(output / "atlas.json", atlas)
    asset_pack.write_json(output / "provenance.json", provenance)
    asset_pack.preview(atlas, output / "layers-preview.png", 3)
    return atlas


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--source", type=Path, default=SCRIPTS.parent / "assets/concepts/kikai/dora-reference-expression-sheet.png")
    p.add_argument("--output-dir", type=Path, default=SCRIPTS.parent / "assets/ui/kikai")
    args = p.parse_args()
    result = author(args.source, args.output_dir)
    print(json.dumps({"sprites": len(result["sprites"]), "palette": len(result["palette"]),
                      "source": str(args.source), "output": str(args.output_dir)}))
