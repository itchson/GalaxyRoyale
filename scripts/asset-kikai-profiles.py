"""Explicit raster conversion and authored DORO/TALO licensing portrait layers.

The reference conversion is lossy and recorded. The later asset-pack import is
exact: all authored RGBA cells, palette entries and independent layers survive.
No runtime source, guide art or generated reference is changed by this script.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

from PIL import Image, ImageDraw, __version__ as pillow_version

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("asset_pack", ROOT / "scripts/asset-pack.py")
pack = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pack)
WIDTH, HEIGHT = 96, 128
TRANSPARENT = (0, 0, 0, 0)
INK = (9, 16, 20, 255)
GOLD = (235, 179, 39, 255)
GOLD_LIGHT = (255, 239, 151, 255)
WHITE = (245, 235, 224, 255)
MOUTH = (90, 38, 43, 255)
SKIN_SHADE = (180, 130, 108, 255)
EYE_STATES = ("neutral", "focused", "concerned", "pleased", "blink")
MOUTH_STATES = ("rest", "small", "open", "smile")
AUTHORING = {
    "doro": {"eyes": [(31, 49, 43, 58), (49, 47, 65, 56)],
             "iris": [(37, 53), (57, 51)], "mouth": (40, 66, 52, 74),
             "lip": (45, 69), "hair_top": 46, "hair_sides": (31, 65, 61),
             "fringe": (36, 48, 57)},
    "talo": {"eyes": [(27, 49, 44, 60), (49, 46, 68, 58)],
             "iris": [(35, 55), (56, 53)], "mouth": (39, 68, 52, 76),
             "lip": (45, 71), "hair_top": 40, "hair_sides": (29, 67, 59),
             "fringe": (38, 44, 43)},
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prepare_reference(source):
    """Full-source alpha128, all components, tight-bbox nearest fit,47 colors."""
    image = Image.open(source).convert("RGBA")
    if image.size != (1086, 1448):
        raise ValueError("Expected reviewed1086x1448 source; recrop/refit explicitly for another source")
    alpha = image.getchannel("A")
    hard = alpha.point(lambda value: 255 if value >= 128 else 0)
    bbox = hard.getbbox()
    if bbox is None:
        raise ValueError("Reference has no occupied pixels")
    hardened = image.copy()
    hardened.putalpha(hard)
    trimmed = hardened.crop(bbox)
    scale = min(WIDTH / trimmed.width, HEIGHT / trimmed.height)
    fitted_size = (max(1, round(trimmed.width * scale)), max(1, round(trimmed.height * scale)))
    offset = ((WIDTH - fitted_size[0]) // 2, (HEIGHT - fitted_size[1]) // 2)
    fitted = Image.new("RGBA", (WIDTH, HEIGHT))
    fitted.paste(trimmed.resize(fitted_size, Image.Resampling.NEAREST), offset)
    samples_data = [p[:3] for p in fitted.get_flattened_data() if p[3]]
    samples = Image.new("RGB", (len(samples_data), 1))
    samples.putdata(samples_data)
    quantized = samples.quantize(colors=47, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).convert("RGB")
    colors = iter(quantized.get_flattened_data())
    result = Image.new("RGBA", fitted.size)
    result.putdata([(*next(colors), 255) if p[3] else TRANSPARENT for p in fitted.get_flattened_data()])
    record = {
        "source_dimensions": list(image.size), "crop": [0, 0, image.width, image.height],
        "alpha_threshold": 128, "source_nonbinary_alpha_cells": sum(a not in (0, 255) for a in alpha.get_flattened_data()),
        "source_hard_occupied_cells": sum(a != 0 for a in hard.get_flattened_data()),
        "tight_bbox": list(bbox), "output_dimensions": [WIDTH, HEIGHT],
        "fitted_dimensions": list(fitted_size), "center_offset": list(offset),
        "nearest_resize": True, "aspect_policy": "uniform fit; integer output rounding only",
        "occupied_after_resize": len(samples_data), "quantize_colors": 47, "dither": False,
        "changed_occupied_color_cells": sum(a != b for a, b in zip(fitted.get_flattened_data(), result.get_flattened_data()) if a[3]),
        "component_policy": "all; no connected-component removal",
        "output_components_8_neighbor": len(pack.components([p[3] != 0 for p in result.get_flattened_data()], WIDTH, HEIGHT, True)),
    }
    return result, record


def portrait_layers(base, kind):
    """Authored local features; every state retains the reviewed layer mask."""
    layout = AUTHORING[kind]
    eye_rects, mouth_rect = layout["eyes"], layout["mouth"]
    # Features must sit on the opaque face; no accidental authored silhouette fill.
    for rect in eye_rects + [mouth_rect]:
        if not all(p[3] == 255 for p in base.crop(rect).get_flattened_data()):
            raise ValueError(f"{kind}: reviewed facial patch intersects transparency")
    body = base.copy()
    draw = ImageDraw.Draw(body)
    for x0, y0, x1, y1 in eye_rects + [mouth_rect]:
        draw.rectangle((x0, y0, x1 - 1, y1 - 1), fill=TRANSPARENT)
    layers = {"body": body}
    for state in EYE_STATES:
        eyes = Image.new("RGBA", base.size)
        for rect in eye_rects:
            eyes.paste(base.crop(rect), rect[:2])
        draw = ImageDraw.Draw(eyes)
        for eye_index, (cx, cy) in enumerate(layout["iris"]):
            rect = eye_rects[eye_index]
            if state == "blink":
                # Local skin samples preserve each face's asymmetric shading.
                for y in range(cy - 2, cy + 3):
                    for x in range(rect[0] + 1, rect[2] - 1):
                        if x < rect[0] + 3 and abs(y - cy) > 1:
                            continue
                        eyes.putpixel((x, y), base.getpixel((x, rect[3])))
                draw.line([(rect[0] + 2, cy), (cx, cy + 1), (rect[2] - 2, cy)], fill=INK)
            else:
                # The reference's reddish iris clusters become authored gold eyes.
                for y in range(cy - 1, cy + 3):
                    for x in range(cx - 2, cx + 2):
                        eyes.putpixel((x, y), GOLD if y < cy + 2 else SKIN_SHADE)
                draw.point((cx + 1, cy), fill=INK)
                draw.point((cx - 1, cy - 1), fill=GOLD_LIGHT)
                draw.point((cx - 2, cy), fill=WHITE)
                if state == "focused":
                    draw.line([(rect[0] + 1, cy - 3), (rect[2] - 2, cy - 2)], fill=INK)
                elif state == "concerned":
                    draw.line([(rect[0] + 1, cy - 2), (rect[2] - 2, cy - 4)], fill=INK)
                elif state == "pleased":
                    draw.line([(cx - 4, cy + 3), (cx + 3, cy + 3)], fill=SKIN_SHADE)
        layers[f"eyes_{state}"] = eyes
    cx, cy = layout["lip"]
    for state in MOUTH_STATES:
        mouth = Image.new("RGBA", base.size)
        mouth.paste(base.crop(mouth_rect), mouth_rect[:2])
        draw = ImageDraw.Draw(mouth)
        for y in range(cy - 2, cy + 3):
            for x in range(cx - 4, cx + 5):
                mouth.putpixel((x, y), base.getpixel((mouth_rect[0] if x < cx else mouth_rect[2] - 1, y)))
        if state == "rest":
            draw.line([(cx - 3, cy), (cx + 3, cy)], fill=SKIN_SHADE)
            draw.point((cx, cy + 1), fill=MOUTH)
        elif state == "small":
            draw.ellipse((cx - 2, cy - 1, cx + 2, cy + 2), fill=MOUTH)
            draw.line([(cx - 1, cy - 1), (cx + 1, cy - 1)], fill=WHITE)
        elif state == "open":
            draw.ellipse((cx - 3, cy - 2, cx + 3, cy + 3), fill=MOUTH)
            draw.line([(cx - 2, cy - 1), (cx + 2, cy - 1)], fill=WHITE)
            draw.line([(cx - 1, cy + 2), (cx + 1, cy + 2)], fill=SKIN_SHADE)
        else:
            draw.line([(cx - 4, cy - 1), (cx - 2, cy + 1), (cx + 2, cy + 1), (cx + 4, cy - 1)], fill=MOUTH)
            draw.line([(cx - 2, cy), (cx + 2, cy)], fill=WHITE)
        layers[f"mouth_{state}"] = mouth
    return layers


def portrait_roles(body, kind):
    layout = AUTHORING[kind]
    hair, horns, signals = [], [], []
    left, right, side_bottom = layout["hair_sides"]
    fringe_left, fringe_right, fringe_bottom = layout["fringe"]
    for i, (r, g, b, a) in enumerate(body.get_flattened_data()):
        if not a:
            continue
        x, y = i % WIDTH, i // WIDTH
        head_region = (y < layout["hair_top"] or (y < side_bottom and (x < left or x >= right))
                       or (fringe_left <= x < fringe_right and y < fringe_bottom))
        if head_region and g > r + 3 and g > b + 2:
            hair.append(i)
        if y < 40 and r > g + 20 and r > b + 10:
            horns.append(i)
            if r > 170 and r > g * 2 and r > b * 1.5:
                signals.append(i)
    return {"source_size": [WIDTH, HEIGHT], "default_main": "PAN", "default_sub": "RENKO",
            "hair_cells": hair, "horn_cells": horns, "signal_cells": signals}


def compose(layers, eyes="neutral", mouth="rest"):
    image = layers["body"].copy()
    for name in (f"eyes_{eyes}", f"mouth_{mouth}"):
        image.alpha_composite(layers[name])
    return image


def validate_profile(atlas):
    """Validate the shared importer/host contract, including all20 pose masks."""
    if atlas.get("version") != 1:
        raise ValueError("Profile pack version must be1")
    palette, sprites, roles = atlas["palette"], atlas["sprites"], atlas["portrait_roles"]
    if not 2 <= len(palette) <= 64 or palette[0] != 0 or len(set(palette)) != len(palette):
        raise ValueError("Palette must have2..64 distinct entries and transparent index0")
    if any(type(c) is not int or c < 0 or c > 0xffffffff or c >> 24 != 255 for c in palette[1:]):
        raise ValueError("Occupied palette entries must be opaque RGBAu32")
    names = {"body"} | {f"eyes_{s}" for s in EYE_STATES} | {f"mouth_{s}" for s in MOUTH_STATES}
    if set(sprites) != names:
        raise ValueError("Profile must have exactly body,5eye and4mouth layers")
    for sprite in sprites.values():
        if (sprite["width"], sprite["height"]) != (WIDTH, HEIGHT) or len(sprite["indices"]) != WIDTH * HEIGHT:
            raise ValueError("Every layer must be96x128")
        if any(type(i) is not int or not 0 <= i < len(palette) for i in sprite["indices"]):
            raise ValueError("Cell palette index invalid")
    def mask(name):
        return {i for i, c in enumerate(sprites[name]["indices"]) if c}
    body, eyes, mouth = mask("body"), mask("eyes_neutral"), mask("mouth_rest")
    if not body or not eyes or not mouth or body & eyes or body & mouth or eyes & mouth:
        raise ValueError("Portrait layers must be nonempty and mutually disjoint")
    if any(mask(f"eyes_{s}") != eyes for s in EYE_STATES) or any(mask(f"mouth_{s}") != mouth for s in MOUTH_STATES):
        raise ValueError("Pose states must retain occupied positions")
    if len(body | eyes | mouth) > WIDTH * HEIGHT:
        raise ValueError("Portrait exceeds12288-cell bound")
    if roles["source_size"] != [WIDTH, HEIGHT] or roles["default_main"] != "PAN" or roles["default_sub"] != "RENKO":
        raise ValueError("Corporate baseline/source dimensions differ")
    masks = {}
    for name in ("hair_cells", "horn_cells", "signal_cells"):
        cells = roles[name]
        if not cells or any(type(i) is not int for i in cells) or cells != sorted(set(cells)) or not set(cells) <= body:
            raise ValueError("Role cells must be sorted unique occupied body positions")
        masks[name] = set(cells)
    if masks["hair_cells"] & masks["horn_cells"] or not masks["signal_cells"] <= masks["horn_cells"]:
        raise ValueError("Hair/crystals must be disjoint; signal subset of crystals")
    return len(body | eyes | mouth)


def author(source_dir, output):
    output.mkdir(parents=True, exist_ok=True)
    generation = json.loads((source_dir / "generation.json").read_text(encoding="utf-8"))
    profiles, summaries, previews = {}, {}, {}
    for kind in AUTHORING:
        source = source_dir / f"{kind}-reference.png"
        if sha(source) != generation["portraits"][kind]["sha256"]:
            raise ValueError(f"{kind}: source differs from recorded generated artifact")
        base, conversion = prepare_reference(source)
        layers = portrait_layers(base, kind)
        folder = output / kind
        (folder / "source").mkdir(parents=True, exist_ok=True)
        for name, image in layers.items():
            image.save(folder / "source" / f"{name}.png")
        palette = [0] + sorted({pack.color(p) for image in layers.values() for p in image.get_flattened_data() if p[3]})
        if len(palette) > 64:
            raise ValueError(f"{kind}: authored palette exceeds64: {len(palette)}")
        roles = portrait_roles(layers["body"], kind)
        provenance = {"type": kind.upper(), "main_corporation": "PAN", "sub_corporation": "RENKO",
                      "source_path": f"assets/concepts/kikai/profiles/{kind}-reference.png",
                      "source_sha256": sha(source), "pillow": pillow_version,
                      "generation": generation["portraits"][kind], "conversion": conversion,
                      "manual_authoring": {"facial_patches": AUTHORING[kind],
                                           "gold_iris_clusters": True, "expression_states": list(EYE_STATES),
                                           "mouth_states": list(MOUTH_STATES),
                                           "role_selection": "Reviewed head-only green hair/red crystals; no neck or eyebrow recolor"},
                      "exact_import": "asset-pack imports authored binary-alpha PNG layers with provided palette; no resize or removed components",
                      "runtime_boundary": "Host selects layers and corporate colors; JSON metadata alone does not animate"}
        manifest = {"schema": 1, "palette": {"mode": "provided", "max_colors": 64, "colors": palette},
                    "sprites": [{"name": name, "source": f"source/{name}.png", "topology": "holes-allowed"} for name in layers],
                    "provenance": provenance, "extensions": {"portrait_roles": roles}}
        pack.write_json(folder / "manifest.json", manifest)
        atlas = pack.pack(folder / "manifest.json")
        validate_profile(atlas)
        profiles[kind] = atlas
        pack.write_json(folder / "provenance.json", provenance)
        previews[kind] = [compose(layers, e) for e in EYE_STATES] + [compose(layers, "neutral", m) for m in MOUTH_STATES[1:]]
        summaries[kind] = {"palette_entries": len(palette), "composed_occupied_cells": conversion["occupied_after_resize"],
                           "body_cells": sum(p[3] != 0 for p in layers["body"].get_flattened_data()),
                           "eye_cells": sum(p[3] != 0 for p in layers["eyes_neutral"].get_flattened_data()),
                           "mouth_cells": sum(p[3] != 0 for p in layers["mouth_rest"].get_flattened_data()),
                           "role_cells": {key: len(roles[key]) for key in ("hair_cells", "horn_cells", "signal_cells")},
                           "max_portrait_submissions": conversion["occupied_after_resize"], "transforms": 3}
    result = {"version": 1, "profiles": profiles}
    pack.write_json(output / "atlas.json", result)
    pack.write_json(output / "authoring-evidence.json", {"version": 1, "scope": "Authored PNG/packed-cell software preview; no GPU or browser acceptance", "profiles": summaries})
    canvas = Image.new("RGB", (8 * 304, 2 * 416), (5, 11, 18))
    draw = ImageDraw.Draw(canvas)
    for row, (kind, images) in enumerate(previews.items()):
        for column, image in enumerate(images):
            x, y = column * 304 + 8, row * 416 + 24
            label = list(EYE_STATES) + ["speech-small", "speech-open", "smile"]
            draw.text((x, y - 18), kind.upper() + " / " + label[column], fill=(179, 217, 229))
            canvas.paste(image.resize((288, 384), Image.Resampling.NEAREST), (x, y), image.resize((288, 384), Image.Resampling.NEAREST))
    canvas.save(output / "authoring-contact-sheet.png")
    return result, summaries


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, default=ROOT / "assets/concepts/kikai/profiles")
    parser.add_argument("--output-dir", type=Path, default=ROOT / "assets/ui/kikai/profiles")
    parser.add_argument("--prepare-only", action="store_true", help="Review lossy base fit before authoring layer coordinates")
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    if args.prepare_only:
        for kind in ("doro", "talo"):
            image, conversion = prepare_reference(args.source_dir / f"{kind}-reference.png")
            image.save(args.output_dir / f"{kind}-fitted.png")
            image.resize((576, 768), Image.Resampling.NEAREST).save(args.output_dir / f"{kind}-fitted-review.png")
            print(json.dumps({"kind": kind, "conversion": conversion}))
        return
    _, summaries = author(args.source_dir, args.output_dir)
    print(json.dumps(summaries))


if __name__ == "__main__":
    main()
