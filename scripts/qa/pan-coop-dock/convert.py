"""Explicit lossy raster sampling, followed by exact asset-pack cell import.

This produces a review candidate, not a replacement for the shipping Haven.
The generated reference is not assumed to have a regular native pixel grid.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[3]
DEFAULT = ROOT / "assets/pipeline/pan-coop-dock"
SOURCE_SHA = "eab5fa7c3b604451fdb5fadff75c2138d614aa8d8bbcedd31402527c59380918"
SIZE = 128
PARTS = ("deck", "alloy", "screens", "signals")


def module(name, relative):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


AP = module("dock_asset_pack", "scripts/asset-pack.py")
PNG = module("dock_png", "scripts/asset-flight-icons.py")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path, value, compact=False):
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(value, ensure_ascii=False, allow_nan=False,
                      **({"separators": (",", ":")} if compact else {"indent": 2}))
    path.write_bytes((text + "\n").encode("utf-8"))


def image(sprite, palette):
    result = Image.new("RGBA", (sprite["width"], sprite["height"]))
    result.putdata([AP.rgba(palette[i]) for i in sprite["indices"]])
    return result


def is_green(c):
    r, g, b, a = c
    return a > 0 and g > r + 12 and g > b + 12


def role(c):
    r, g, b, a = c
    if not a:
        return 0  # ignored by live occupancy
    if is_green(c):
        return 3
    if g > r + 12 and b > r + 12:
        return 2
    if min(r, g, b) >= 145:
        return 1
    return 0


def sample(source, side):
    """Match asset-pack's declared threshold then nearest-fit policies exactly."""
    threshold = Image.new("RGBA", source.size)
    threshold.putdata([(r, g, b, 255) if a >= 128 else (0, 0, 0, 0)
                       for r, g, b, a in source.get_flattened_data()])
    return threshold.resize((side, side), Image.Resampling.NEAREST)


def sampling_metrics(source, sampled, quantized):
    before = [a >= 128 for *_, a in source.get_flattened_data()]
    after = [a != 0 for *_, a in sampled.get_flattened_data()]
    projected = sampled.resize(source.size, Image.Resampling.NEAREST)
    expanded = [a != 0 for *_, a in projected.get_flattened_data()]
    fp = sum(b and not a for a, b in zip(before, expanded))
    fn = sum(a and not b for a, b in zip(before, expanded))
    union = sum(a or b for a, b in zip(before, expanded))
    pairs = [(a[:3], b[:3]) for a, b in
             zip(sampled.get_flattened_data(), quantized.get_flattened_data()) if a[3]]
    differences = [abs(x - y) for a, b in pairs for x, y in zip(a, b)]
    return {
        "source_dimensions": list(source.size), "native_dimensions": list(sampled.size),
        "threshold": 128, "sampling": "Pillow nearest center sample; no crop or component filter",
        "source_topology_threshold128": AP.topology(before, *source.size),
        "native_topology": AP.topology(after, *sampled.size),
        "alpha_values_changed_before_sampling": sum(a not in (0, 255) for *_, a in source.get_flattened_data()),
        "source_mask_vs_native_nearest_reprojection": {
            "false_opaque_source_pixels": fp, "false_transparent_source_pixels": fn,
            "intersection_over_union": (union - fp - fn) / union},
        "quantized_occupied_color_cells": sum(a != b for a, b in pairs),
        "mean_absolute_rgb_channel_error": sum(differences) / len(differences),
        "maximum_rgb_channel_error": max(differences),
        "rgb_channel_rmse": math.sqrt(sum(d * d for d in differences) / len(differences)),
    }


def author(output=DEFAULT, source_path=DEFAULT / "source.png"):
    AP.require(sha(source_path) == SOURCE_SHA, "Reference hash changed: explicit new review required")
    source = Image.open(source_path).convert("RGBA")
    AP.require(source.size == (1254, 1254), "Reference dimensions changed")
    output.mkdir(parents=True, exist_ok=True)
    if output.resolve() != source_path.parent.resolve():
        (output / "source.png").write_bytes(source_path.read_bytes())
    generation = source_path.parent / "generation-provenance.json"
    prompt = source_path.parent / "generation-prompt.txt"
    record = json.loads(generation.read_text(encoding="utf-8"))
    AP.require(record["sourceSha256"] == SOURCE_SHA and record["sourceDimensions"] == [1254, 1254],
               "Generation provenance does not match reference")
    if output.resolve() != source_path.parent.resolve():
        for path in (generation, prompt):
            (output / path.name).write_bytes(path.read_bytes())
    raw = list(sample(source, SIZE).get_flattened_data())
    roles = [role(c) for c in raw]
    # All three quantizations use the existing importer, median cut and no
    # dithering. Separate source-color strata keep sparse green/cyan clusters
    # from being swallowed by the much larger metal population.
    palette, indices = [0], [0] * (SIZE * SIZE)
    sampling_records = {}
    for label, zone, colors in (("metal", (0, 1), 48), ("green", (3,), 9), ("cyan", (2,), 9)):
        im = Image.new("RGBA", (SIZE, SIZE))
        im.putdata([c if roles[i] in zone else (0, 0, 0, 0) for i, c in enumerate(raw)])
        PNG.write_png(output / f"sample-{label}.png", im)
        manifest = {"schema": 1, "palette": {"mode": "quantize", "max_colors": colors},
                    "sprites": [{"name": label, "source": f"sample-{label}.png",
                                 "topology": "holes-allowed", "components": {"mode": "all"}}]}
        path = output / f"quantize-{label}-manifest.json"
        write_json(path, manifest)
        zone_pack = AP.pack(path)
        mapping = {0: 0}
        for n, value in enumerate(zone_pack["palette"][1:], 1):
            if value not in palette:
                palette.append(value)
            mapping[n] = palette.index(value)
        for i, index in enumerate(zone_pack["sprites"][label]["indices"]):
            if index:
                indices[i] = mapping[index]
        sampling_records[label] = zone_pack["asset_metadata"]
    sprite = {"width": SIZE, "height": SIZE, "indices": indices}
    write_json(output / "sampling-records.json", sampling_records)
    # Only exterior green luminous clusters animate. The central crystal stays
    # at its authored baseline, avoiding a whole-dock/whole-palette flash.
    lamps = [i for i, (index, c) in enumerate(zip(sprite["indices"], raw))
             if index and is_green(c) and (i % SIZE - 64) ** 2 + (i // SIZE - 64) ** 2 > 18 ** 2]
    lamp_colors = sorted({sprite["indices"][i] for i in lamps})
    variants = {}
    for state, factor in ((1, 0.78), (2, 1.35)):
        for index in lamp_colors:
            r, g, b, a = AP.rgba(palette[index])
            target = (min(255, round(r * factor)), min(255, round(g * factor)), min(255, round(b * factor)))
            variants[state, index] = min(lamp_colors, key=lambda ix: sum((x - y) ** 2 for x, y in zip(AP.rgba(palette[ix])[:3], target)))
    AP.require(len(palette) <= 64, "Animation palette exceeds64; no silent color merging")
    frames = [sprite["indices"].copy() for _ in range(3)]
    for state in (1, 2):
        for i in lamps:
            frames[state][i] = variants[state, frames[0][i]]
    groups, entries, full_images = [], [], []
    for state, indices in enumerate(frames):
        full = {"width": SIZE, "height": SIZE, "indices": indices}
        im = image(full, palette)
        full_images.append(im)
        PNG.write_png(output / f"dock-{state}.png", im)
        entries.append({"name": f"dock-{state}", "source": f"dock-{state}.png", "topology": "holes-allowed"})
        parts = []
        for k, part in enumerate(PARTS):
            name = f"{part}-{state}"
            mask_indices = [index if roles[i] == k else 0 for i, index in enumerate(indices)]
            PNG.write_png(output / f"{name}.png", image({**full, "indices": mask_indices}, palette))
            entries.append({"name": name, "source": f"{name}.png", "topology": "holes-allowed"})
            parts.append({"name": part, "sprite": name,
                          **({"parent": "deck"} if k else {}),
                          "transform": {"position": [0, 0], "basis_x": [1, 0], "basis_y": [0, 1]}})
        groups.append({"name": f"pan-dock-{state}", "parts": parts})
    manifest = {"schema": 1, "palette": {"mode": "provided", "max_colors": 64, "colors": palette},
                "sprites": entries, "groups": groups,
                "animations": [{"name": "pan-lamps", "loop": True,
                                "frames": [{"kind": "group", "name": f"pan-dock-{n}", "duration_ms": 700}
                                           for n in range(3)]}],
                "budgets": {"sprite_cells": 8000, "group_cells": 8000, "group_transforms": 4, "extent": 1000},
                "provenance": {"source_type": "generated-reference-explicit-lossy-conversion-and-authored-lamp-states",
                               "source_sha256": SOURCE_SHA, "source_dimensions": [1254, 1254],
                               "generation_prompt_sha256": sha(prompt), "generation_record_sha256": sha(generation),
                               "generation_tool": "built-in image_gen", "model_version": "not exposed",
                               "transparent_requested": True, "reference_role": "accepted APR portrait used for style only",
                               "animation_author": "explicit color-state authoring; not generated animation"},
                "extensions": {"pixel_rig": {"width": SIZE, "height": SIZE, "roles": roles,
                   "lamp_cells": lamps, "frame_sprites": [f"dock-{n}" for n in range(3)],
                   "parts": [{"name": p, "parent": 0 if k else None, "position": [0, 0],
                              "role": ("armor", "trim", "trim", "emitter")[k]} for k, p in enumerate(PARTS)],
                   "runtime_policy": "explicit host swaps complete body colors; fixed geometry/roles; freeze state0 in Reduced Motion"}}}
    write_json(output / "manifest.json", manifest)
    pack = AP.pack(output / "manifest.json")
    write_json(output / "pack.json", pack, True)
    # Exact candidate art and role masks, not extracted native/GPU images.
    sheet = Image.new("RGB", (3 * 416, 448), (5, 10, 17))
    draw = ImageDraw.Draw(sheet)
    for state, im in enumerate(full_images):
        draw.text((state * 416 + 16, 12), f"AUTHORED lamp state {state} / 7100 cells", fill=(174, 216, 205))
        sheet.paste(im.resize((384, 384), Image.Resampling.NEAREST), (state * 416 + 16, 44),
                    im.resize((384, 384), Image.Resampling.NEAREST))
    PNG.write_png(output / "lamp-states.png", sheet)
    mask_image = Image.new("RGBA", (SIZE, SIZE))
    role_colors = ((82, 99, 115, 255), (228, 233, 214, 255), (70, 217, 232, 255), (77, 237, 121, 255))
    mask_image.putdata([role_colors[roles[i]] if index else (0, 0, 0, 0)
                        for i, index in enumerate(frames[0])])
    PNG.write_png(output / "material-mask.png", mask_image)
    lamp_image = Image.new("RGBA", (SIZE, SIZE))
    lamp_set = set(lamps)
    lamp_image.putdata([(255, 255, 255, 255) if i in lamp_set else (0, 0, 0, 0) for i in range(SIZE * SIZE)])
    PNG.write_png(output / "lamp-mask.png", lamp_image)
    metrics = {"source_sha256": SOURCE_SHA, "grid_comparison": {}, "palette_colors": len(palette),
               "lamp_cells": len(lamps), "part_counts": {p: sum(index != 0 and roles[i] == k for i, index in enumerate(frames[0])) for k, p in enumerate(PARTS)},
               "quantization": "3 explicit asset-pack median-cut passes: metal47/green8/cyan8 maximum opaque colors, no dither",
               "animations": "3 authored same-occupancy lamp-only color states, nearest existing green palette shade to gain1/.78/1.35; no generated motion",
               "lamp_changed_cells": [sum(a != b for a, b in zip(frames[0], frame)) for frame in frames],
               "pending": ["Shipping adapter and co-op facility semantics", "GPU presentation", "Human readability", "Physical performance"]}
    comparisons = Image.new("RGB", (4 * 416, 448), (5, 10, 17))
    d = ImageDraw.Draw(comparisons)
    thumb = source.resize((384, 384), Image.Resampling.NEAREST)
    comparisons.paste(thumb, (16, 44), thumb)
    d.text((16, 12), "RAW1254 / review thumbnail only", fill=(174, 216, 205))
    for col, side in enumerate((128, 192, 256), 1):
        im = sample(source, side)
        if side == 128:
            quantized = full_images[0]
        else:
            # Comparisons use the SAME candidate palette, no higher-detail ABI claim.
            rgb = Image.new("P", (1, 1))
            rgb.putpalette([v for c in palette for v in AP.rgba(c)[:3]] + [0] * (768 - len(palette) * 3))
            q = im.convert("RGB").quantize(palette=rgb, dither=Image.Dither.NONE).convert("RGBA")
            q.putalpha(im.getchannel("A"))
            quantized = q
        PNG.write_png(output / f"comparison-{side}.png", quantized)
        metrics["grid_comparison"][str(side)] = sampling_metrics(source, im, quantized)
        view = quantized.resize((384, 384), Image.Resampling.NEAREST)
        comparisons.paste(view, (col * 416 + 16, 44), view)
        d.text((col * 416 + 16, 12), f"{side} grid / {sum(a>0 for *_,a in im.get_flattened_data())} cells", fill=(174, 216, 205))
    PNG.write_png(output / "grid-comparison.png", comparisons)
    write_json(output / "conversion-losses.json", metrics)
    return pack, metrics


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=DEFAULT)
    parser.add_argument("--source", type=Path, default=DEFAULT / "source.png")
    parser.add_argument("--compact-native", action="store_true", help="Lossless JSON whitespace-only compaction of retained native packets")
    args = parser.parse_args()
    if args.compact_native:
        for name in ("native-extraction.json", "rig-extraction.json"):
            path = args.output / name
            write_json(path, AP.read_json(path), True)
        audit = args.output / "native-audit.json"
        if audit.exists():
            write_json(audit, AP.read_json(audit))
        return
    _, metrics = author(args.output, args.source)
    print(json.dumps({k: v for k, v in metrics.items() if k != "grid_comparison"}, indent=2))


if __name__ == "__main__":
    main()
