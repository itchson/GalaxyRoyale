"""192x256 KIKAI authoring from pinned full-resolution references.

The alpha/crop/nearest-fit/55-color conversion is explicitly lossy. The
subsequent PNG-to-packed-cell import is exact, with no hidden resizing or
component filtering. This offline tool does not drive gameplay or render UI.
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
WIDTH, HEIGHT = 192, 256
TRANSPARENT = (0, 0, 0, 0)
INK = (9, 16, 20, 255)
GOLD = (235, 179, 39, 255)
GOLD_LIGHT = (255, 239, 151, 255)
GOLD_SHADOW = (147, 91, 17, 255)
GOLD_WARM = (254, 216, 59, 255)
WHITE = (245, 235, 224, 255)
MOUTH = (90, 38, 43, 255)
SKIN_SHADE = (180, 130, 108, 255)
EYE_STATES = ("neutral", "focused", "concerned", "pleased", "blink")
MOUTH_STATES = ("rest", "small", "open", "smile")
AUTHORING = {
    "dora": {"eyes": [(52, 103, 82, 126), (96, 99, 129, 121)],
             "iris": [(66, 116), (111, 112)], "mouth": (80, 137, 103, 155),
             "lip": (91, 147), "hair_top": 96, "hair_sides": (60, 130, 226),
             "fringe": (72, 101, 115)},
    "doro": {"eyes": [(59, 94, 91, 117), (99, 92, 132, 114)],
             "iris": [(74, 103), (112, 101)], "mouth": (80, 125, 108, 143),
             "lip": (94, 132), "hair_top": 92, "hair_sides": (61, 133, 123),
             "fringe": (73, 108, 119)},
    "talo": {"eyes": [(57, 104, 86, 124), (100, 101, 133, 121)],
             "iris": [(73, 113), (115, 110)], "mouth": (79, 134, 107, 152),
             "lip": (94, 141), "hair_top": 90, "hair_sides": (59, 132, 124),
             "fringe": (76, 98, 91)},
}
# Spatially reviewed regions in the committed192-grid anatomy. Hue alone
# cannot distinguish warm forehead skin from red KESSHO crystal facets.
CORPORATE_REGIONS = {
    "dora": {"gems": [((42, 0, 65, 78), (45, 20)), ((101, 0, 123, 78), (119, 1))],
             "eyebrows": [(52, 103, 82, 126), (96, 99, 129, 121)],
             "neck": (60, 158, 130, 256)},
    "doro": {"gems": [((47, 2, 68, 73), (49, 20)), ((109, 2, 129, 75), (127, 4))],
             "eyebrows": [(59, 94, 91, 117), (99, 92, 132, 114)],
             "neck": (60, 145, 134, 256)},
    "talo": {"gems": [((47, 1, 70, 77), (49, 20)), ((110, 1, 131, 78), (128, 5))],
             "eyebrows": [(57, 104, 86, 124), (100, 101, 133, 121)],
             "neck": (59, 155, 134, 256)},
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path, value):
    # asset-pack fingerprints raw manifest bytes. Canonical LF is necessary
    # before import as well as at Git checkout, for Windows/Linux parity.
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n",
                    encoding="utf-8", newline="\n")


def source_records():
    metadata = json.loads((ROOT / "assets/concepts/kikai/detail/generation.json").read_text(encoding="utf-8"))
    records = {"dora": metadata["dora"]} | metadata["portraits"]
    for kind, record in records.items():
        if sha(ROOT / record["path"]) != record["sha256"]:
            raise ValueError(f"{kind}: source differs from recorded reference")
        if "input" in record and sha(ROOT / record["input"]) != record["input_sha256"]:
            raise ValueError(f"{kind}: original edit input differs from provenance")
    return metadata, records


def prepare_reference(source, kind):
    image = Image.open(source).convert("RGBA")
    expected = (1881, 836) if kind == "dora" else (1086, 1448)
    if image.size != expected:
        raise ValueError(f"{kind}: expected reviewed {expected}; recrop explicitly for another source")
    crop_rect = (0, 0, 627, 836) if kind == "dora" else (0, 0, image.width, image.height)
    cropped = image.crop(crop_rect)
    alpha = cropped.getchannel("A")
    hard = alpha.point(lambda value: 255 if value >= 128 else 0)
    bbox = hard.getbbox()
    if bbox is None:
        raise ValueError("Reference contains no occupied pixels")
    cropped.putalpha(hard)
    trimmed = cropped.crop(bbox)
    scale = min(WIDTH / trimmed.width, HEIGHT / trimmed.height)
    fitted_size = (max(1, round(trimmed.width * scale)), max(1, round(trimmed.height * scale)))
    offset = ((WIDTH - fitted_size[0]) // 2, (HEIGHT - fitted_size[1]) // 2)
    fitted = Image.new("RGBA", (WIDTH, HEIGHT))
    fitted.paste(trimmed.resize(fitted_size, Image.Resampling.NEAREST), offset)
    pixels = list(fitted.get_flattened_data())
    occupied = [pixel[:3] for pixel in pixels if pixel[3]]
    # Eye hues occupy few cells relative to hair/armor. Explicit eye-region
    # weighting retains gold rings instead of letting skin/red dominate them.
    eye_samples = [p[:3] for rect in AUTHORING[kind]["eyes"]
                   for p in fitted.crop(rect).get_flattened_data() if p[3]]
    training = occupied + eye_samples * 20
    samples = Image.new("RGB", (len(training), 1))
    samples.putdata(training)
    palette_image = samples.quantize(colors=55, method=Image.Quantize.MEDIANCUT,
                                     dither=Image.Dither.NONE)
    cell_samples = Image.new("RGB", (len(occupied), 1))
    cell_samples.putdata(occupied)
    reduced = cell_samples.quantize(palette=palette_image, dither=Image.Dither.NONE).convert("RGB")
    colors = iter(reduced.get_flattened_data())
    result = Image.new("RGBA", fitted.size)
    result.putdata([(*next(colors), 255) if pixel[3] else TRANSPARENT for pixel in pixels])
    gold_cells = 0
    # Preserve the actual source iris mask in four deliberate golden tones;
    # global color reduction otherwise desaturates these small defining areas.
    for rect in AUTHORING[kind]["eyes"]:
        for y in range(rect[1], rect[3]):
            for x in range(rect[0], rect[2]):
                r, g, b, a = fitted.getpixel((x, y))
                if a and r > 100 and r > b * 2.5 and g > b * 1.4 and r * .4 < g < r * 1.1:
                    color = GOLD_SHADOW if g < 115 else GOLD if g < 185 else GOLD_WARM if g < 225 else GOLD_LIGHT
                    result.putpixel((x, y), color)
                    gold_cells += 1
    conversion = {
        "source_dimensions": list(image.size), "crop": list(crop_rect),
        "alpha_threshold": 128, "source_nonbinary_alpha_cells": sum(a not in (0, 255) for a in alpha.get_flattened_data()),
        "source_hard_occupied_cells": sum(a != 0 for a in hard.get_flattened_data()),
        "tight_bbox": list(bbox), "output_dimensions": [WIDTH, HEIGHT],
        "fitted_dimensions": list(fitted_size), "center_offset": list(offset),
        "nearest_resize": True, "aspect_policy": "uniform fit; integer output rounding only",
        "occupied_after_resize": len(occupied), "quantize_colors": 55, "dither": False,
        "palette_eye_region_extra_weight": 20,
        "source_gold_mask_cells": gold_cells, "authored_gold_tones": 4,
        "changed_occupied_color_cells": sum(a != b for a, b in zip(pixels, result.get_flattened_data()) if a[3]),
        "component_policy": "all; no connected-component removal",
        "source_is_old_96cell_pack": False,
    }
    return result, conversion


def portrait_layers(base, kind):
    """Fine one-cell features preserve the reference's neutral iris/lip detail."""
    layout = AUTHORING[kind]
    eye_rects, mouth_rect = layout["eyes"], layout["mouth"]
    for rect in eye_rects + [mouth_rect]:
        if not all(p[3] == 255 for p in base.crop(rect).get_flattened_data()):
            raise ValueError(f"{kind}: facial patch intersects transparency")
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
                # Replace the eyeball with local skin shading; eyelid is a thin
                # authored curve, not a doubled96-grid rectangle/black bar.
                for y in range(cy - 7, cy + 7):
                    for x in range(rect[0] + 1, rect[2] - 1):
                        if abs(x - cx) / 15 + abs(y - cy) / 7 <= 1.65:
                            eyes.putpixel((x, y), base.getpixel((x, rect[3])))
                draw.line([(rect[0] + 3, cy - 1), (cx - 4, cy + 1),
                           (cx + 4, cy + 1), (rect[2] - 3, cy - 2)], fill=INK, width=1)
                draw.line([(cx - 7, cy + 3), (cx + 6, cy + 3)], fill=SKIN_SHADE, width=1)
            else:
                # Keep the gold iris rings already present in the source. Add
                # just one-cell catchlights/pupil accents, not chunky repaint.
                draw.point((cx - 2, cy - 3), fill=GOLD_LIGHT)
                draw.point((cx + 1, cy + 1), fill=GOLD)
                draw.point((cx + 1, cy), fill=INK)
                if state == "focused":
                    draw.line([(rect[0] + 3, cy - 7), (cx - 2, cy - 5),
                               (rect[2] - 3, cy - 6)], fill=INK, width=1)
                elif state == "concerned":
                    draw.line([(rect[0] + 3, cy - 6), (cx, cy - 9),
                               (rect[2] - 3, cy - 5)], fill=SKIN_SHADE, width=1)
                elif state == "pleased":
                    draw.line([(cx - 9, cy + 6), (cx - 2, cy + 7),
                               (cx + 8, cy + 5)], fill=SKIN_SHADE, width=1)
        layers[f"eyes_{state}"] = eyes
    cx, cy = layout["lip"]
    for state in MOUTH_STATES:
        mouth = Image.new("RGBA", base.size)
        mouth.paste(base.crop(mouth_rect), mouth_rect[:2])
        draw = ImageDraw.Draw(mouth)
        if state != "rest":
            # Preserve rest reference exactly. Talking patches use narrow
            # outlines and one-cell highlights at the192 grid's own scale.
            for y in range(cy - 4, cy + 5):
                for x in range(mouth_rect[0] + 1, mouth_rect[2] - 1):
                    mouth.putpixel((x, y), base.getpixel((mouth_rect[0] if x < cx else mouth_rect[2] - 1, y)))
            if state == "small":
                draw.polygon([(cx - 3, cy - 1), (cx + 3, cy - 1),
                              (cx + 4, cy + 2), (cx + 1, cy + 4),
                              (cx - 2, cy + 3), (cx - 4, cy + 1)], fill=MOUTH)
                draw.line([(cx - 2, cy), (cx + 2, cy)], fill=WHITE, width=1)
            elif state == "open":
                draw.polygon([(cx - 5, cy - 2), (cx + 4, cy - 2),
                              (cx + 6, cy + 1), (cx + 4, cy + 5),
                              (cx, cy + 6), (cx - 4, cy + 4),
                              (cx - 6, cy + 1)], fill=MOUTH)
                draw.line([(cx - 4, cy - 1), (cx + 3, cy - 1)], fill=WHITE, width=1)
                draw.line([(cx - 2, cy + 4), (cx + 2, cy + 4)], fill=SKIN_SHADE, width=1)
            else:
                draw.line([(cx - 8, cy - 1), (cx - 4, cy + 2),
                           (cx + 2, cy + 2), (cx + 7, cy - 1)], fill=MOUTH, width=1)
                draw.line([(cx - 3, cy + 1), (cx + 2, cy + 1)], fill=WHITE, width=1)
        layers[f"mouth_{state}"] = mouth
    return layers


def portrait_roles(body, kind):
    layout = AUTHORING[kind]
    regions = CORPORATE_REGIONS[kind]
    hair, horns, signals = [], [], []
    left, right, side_bottom = layout["hair_sides"]
    fringe_left, fringe_right, fringe_bottom = layout["fringe"]

    def inside(x, y, rect):
        return rect[0] <= x < rect[2] and rect[1] <= y < rect[3]

    def protected(x, y, color):
        r, g, b, _ = color
        # Warm skin is tested only within the facial anatomy, not across
        # ivory armor or neutral bright crystal highlights elsewhere.
        facial_skin = inside(x, y, (52, 70, 136, 175)) and r > 150 and g > 100 and b > 80 and r >= g >= b
        return (facial_skin or any(inside(x, y, rect) for rect in regions["eyebrows"])
                or inside(x, y, regions["neck"]))

    # Keep only saturated facets connected to each declared crystal seed,
    # within its explicit spatial region. Forehead clusters cannot join the
    # role just because their pale warm RGB satisfies a loose red threshold.
    gem_cells = set()
    for rect, seed in regions["gems"]:
        candidates = set()
        for y in range(rect[1], rect[3]):
            for x in range(rect[0], rect[2]):
                color = body.getpixel((x, y))
                r, g, b, a = color
                if a and not protected(x, y, color) and r > g + 20 and r > b + 10 and r > g * 1.5:
                    candidates.add(y * WIDTH + x)
        start = seed[1] * WIDTH + seed[0]
        if start not in candidates:
            raise ValueError(f"{kind}: reviewed crystal seed is not a saturated occupied facet")
        pending, selected = [start], {start}
        while pending:
            index = pending.pop()
            x, y = index % WIDTH, index // WIDTH
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    neighbor = (y + dy) * WIDTH + x + dx
                    if 0 <= x + dx < WIDTH and 0 <= y + dy < HEIGHT and neighbor in candidates and neighbor not in selected:
                        selected.add(neighbor)
                        pending.append(neighbor)
        gem_cells.update(selected)

    for i, (r, g, b, a) in enumerate(body.get_flattened_data()):
        if not a:
            continue
        x, y = i % WIDTH, i // WIDTH
        head_region = (y < layout["hair_top"] or (y < side_bottom and (x < left or x >= right))
                       or (fringe_left <= x < fringe_right and y < fringe_bottom))
        if head_region and not protected(x, y, (r, g, b, a)) and g > r + 3 and g > b + 2:
            hair.append(i)
        if i in gem_cells:
            horns.append(i)
            if r > 170 and r > g * 2 and r > b * 1.5:
                signals.append(i)
    return {"source_size": [WIDTH, HEIGHT], "default_main": "PAN", "default_sub": "RENKO",
            "hair_cells": hair, "horn_cells": horns, "signal_cells": signals}


def compose(layers, eyes="neutral", mouth="rest"):
    result = layers["body"].copy()
    result.alpha_composite(layers[f"eyes_{eyes}"])
    result.alpha_composite(layers[f"mouth_{mouth}"])
    return result


def corporate_preview(atlas, main, sub):
    """Offline role review with the native static color formulas; no GPU claim."""
    sprites, palette, roles = atlas["sprites"], atlas["palette"], atlas["portrait_roles"]
    hair, horns = set(roles["hair_cells"]), set(roles["horn_cells"])
    result = Image.new("RGBA", (WIDTH, HEIGHT))
    colors = []
    for index, (a, b, c) in enumerate(zip(sprites["body"]["indices"], sprites["eyes_neutral"]["indices"], sprites["mouth_rest"]["indices"])):
        r, g, b, alpha = pack.rgba(palette[a or b or c])
        if index in hair:
            if main == "RENKO":
                r, g, b = g, (r + b) * 3 // 10, b * 4 // 5
            elif main == "HUGO":
                r, g, b = b * 7 // 25, (r * 13 + g * 6) // 20, g
        if index in horns:
            if sub == "PAN":
                r, g, b = g * 4 // 5, r, b * 4 // 5
            elif sub == "HUGO":
                r, g, b = b * 4 // 5, g * 4 // 5, r
        colors.append((r, g, b, alpha))
    result.putdata(colors)
    return result


def validate_profile(atlas):
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
            raise ValueError("Every layer must be192x256")
        if any(type(i) is not int or not 0 <= i < len(palette) for i in sprite["indices"]):
            raise ValueError("Cell palette index invalid")
    def mask(name):
        return {i for i, c in enumerate(sprites[name]["indices"]) if c}
    body, eyes, mouth = mask("body"), mask("eyes_neutral"), mask("mouth_rest")
    if not body or not eyes or not mouth or body & eyes or body & mouth or eyes & mouth:
        raise ValueError("Portrait layers must be nonempty and mutually disjoint")
    if any(mask(f"eyes_{s}") != eyes for s in EYE_STATES) or any(mask(f"mouth_{s}") != mouth for s in MOUTH_STATES):
        raise ValueError("Pose states must retain occupied positions")
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


def author(output):
    generation, records = source_records()
    output.mkdir(parents=True, exist_ok=True)
    profiles, summaries, previews = {}, {}, {}
    for kind in AUTHORING:
        record = records[kind]
        base, conversion = prepare_reference(ROOT / record["path"], kind)
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
                      "source_path": record["path"], "source_sha256": record["sha256"],
                      "pillow": pillow_version, "generation": record, "conversion": conversion,
                      "guide": "APR-1337 / DORA-PAN-RENKO-1337" if kind == "dora" else None,
                      "manual_authoring": {"facial_patches": AUTHORING[kind],
                                           "fine_feature_line_width": 1, "neutral_reference_iris_retained": True,
                                           "expression_states": list(EYE_STATES), "mouth_states": list(MOUTH_STATES),
                                           "role_selection": "Explicit crystal regions and seeded saturated-facet masks; facial skin, eyebrows and neck protected",
                                           "corporate_regions": CORPORATE_REGIONS[kind]},
                      "exact_import": "asset-pack imports authored binary-alpha PNGs with provided palette; no resize or removed components",
                      "runtime_boundary": "Host selects cached layers and corporate colors; metadata alone does not animate"}
        manifest = {"schema": 1, "palette": {"mode": "provided", "max_colors": 64, "colors": palette},
                    "sprites": [{"name": name, "source": f"source/{name}.png", "topology": "holes-allowed"} for name in layers],
                    "provenance": provenance, "extensions": {"portrait_roles": roles}}
        write_json(folder / "manifest.json", manifest)
        atlas = pack.pack(folder / "manifest.json")
        count = validate_profile(atlas)
        profiles[kind] = atlas
        write_json(folder / "provenance.json", provenance)
        previews[kind] = [compose(layers, e) for e in EYE_STATES] + [compose(layers, "neutral", m) for m in MOUTH_STATES[1:]]
        summaries[kind] = {"palette_entries": len(palette), "composed_occupied_cells": count,
                           "body_cells": sum(p[3] != 0 for p in layers["body"].get_flattened_data()),
                           "eye_cells": sum(p[3] != 0 for p in layers["eyes_neutral"].get_flattened_data()),
                           "mouth_cells": sum(p[3] != 0 for p in layers["mouth_rest"].get_flattened_data()),
                           "role_cells": {key: len(roles[key]) for key in ("hair_cells", "horn_cells", "signal_cells")},
                           "transforms": 3, "max_portrait_submissions": count}
    result = {"version": 1, "profiles": profiles}
    # Runtime include_str payload: whitespace is not cell information. Keep
    # human-readable metadata in the separate manifests/provenance files.
    (output / "atlas.json").write_text(
        json.dumps(result, ensure_ascii=False, separators=(",", ":")) + "\n",
        encoding="utf-8", newline="\n")
    write_json(output / "authoring-evidence.json", {
        "version": 1, "scope": "Authored PNG/packed-cell software previews; not native/GPU/browser acceptance",
        "generation_metadata_sha256": sha(ROOT / "assets/concepts/kikai/detail/generation.json"),
        "profiles": summaries, "shared_runtime_ui_budget": 60000,
        "admission": "The host admits each whole portrait atomically against remaining pixel and work budgets."})
    canvas = Image.new("RGB", (8 * 400, 3 * 544), (5, 11, 18))
    draw = ImageDraw.Draw(canvas)
    labels = list(EYE_STATES) + ["speech-small", "speech-open", "smile"]
    for row, (kind, images) in enumerate(previews.items()):
        for column, image in enumerate(images):
            x, y = column * 400 + 8, row * 544 + 24
            draw.text((x, y - 18), kind.upper() + " / " + labels[column], fill=(179, 217, 229))
            enlarged = image.resize((384, 512), Image.Resampling.NEAREST)
            canvas.paste(enlarged, (x, y), enlarged)
    canvas.save(output / "authoring-contact-sheet.png")
    overview = Image.new("RGB", (3 * 400, 544), (5, 11, 18))
    draw = ImageDraw.Draw(overview)
    for column, (kind, images) in enumerate(previews.items()):
        x = column * 400 + 8
        draw.text((x, 6), kind.upper() + " /192x256 source cells / authoring preview", fill=(179, 217, 229))
        enlarged = images[0].resize((384, 512), Image.Resampling.NEAREST)
        overview.paste(enlarged, (x, 24), enlarged)
    overview.save(output / "authoring-overview.png")
    role_review = Image.new("RGB", (3 * 400, 3 * 544), (5, 11, 18))
    draw = ImageDraw.Draw(role_review)
    for row, (main, sub) in enumerate((("PAN", "RENKO"), ("HUGO", "PAN"), ("RENKO", "HUGO"))):
        for column, (kind, atlas) in enumerate(profiles.items()):
            x, y = column * 400 + 8, row * 544 + 24
            draw.text((x, y - 18), f"{kind.upper()} / {main} hair / {sub} horns / software roles", fill=(179, 217, 229))
            image = corporate_preview(atlas, main, sub).resize((384, 512), Image.Resampling.NEAREST)
            role_review.paste(image, (x, y), image)
    role_review.save(output / "corporate-role-review.png")
    return result, summaries


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=ROOT / "assets/ui/kikai/detail")
    parser.add_argument("--prepare-only", action="store_true", help="Inspect explicit192-grid fits before authoring features")
    args = parser.parse_args()
    if args.prepare_only:
        _, records = source_records()
        args.output_dir.mkdir(parents=True, exist_ok=True)
        for kind, record in records.items():
            image, conversion = prepare_reference(ROOT / record["path"], kind)
            image.save(args.output_dir / f"{kind}-fitted.png")
            image.resize((576, 768), Image.Resampling.NEAREST).save(args.output_dir / f"{kind}-fitted-review.png")
            print(json.dumps({"kind": kind, "conversion": conversion}))
        return
    _, summaries = author(args.output_dir)
    print(json.dumps(summaries))


if __name__ == "__main__":
    main()
