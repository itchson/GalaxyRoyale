"""Explicit reference conversion and exact native pack for GALAXY ROYALE."""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "assets/ui/title-mark"
SOURCE_SIZE = (512, 192)
CROWN = (232, 0, 280, 64)
CUTOUT_FLOOR = 40

def digest(data):
    return hashlib.sha256(data).hexdigest()

def module():
    spec = importlib.util.spec_from_file_location("asset_pack", ROOT / "scripts/asset-pack.py")
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

def canonical_png(path, image):
    """Keep encoded canonical bytes only after an independent full RGBA match."""
    canonical = path if path.exists() else FOLDER / path.name
    if canonical.exists():
        with Image.open(canonical) as old:
            assert old.size == image.size and old.convert("RGBA").tobytes() == image.convert("RGBA").tobytes(), f"Authored cells changed: {path}; review before replacing canonical PNG"
        if path != canonical:
            path.write_bytes(canonical.read_bytes())
    else:
        image.save(path, optimize=False)

def build(folder=FOLDER):
    folder.mkdir(parents=True, exist_ok=True)
    source = Image.open(FOLDER / "reference.png").convert("RGBA")
    assert source.size == (2048, 768), "Reference identity/grid changed"
    sampled = source.resize(SOURCE_SIZE, Image.Resampling.NEAREST)
    original = list(sampled.get_flattened_data())
    # The generated cutout includes opaque near-black haze/recess fills. Remove
    # only the explicitly declared dark threshold, never a largest component.
    pixels = [(*c[:3], 255) if c[3] >= 128 and max(c[:3]) >= CUTOUT_FLOOR else (0, 0, 0, 0) for c in original]
    occupied = [c[:3] for c in pixels if c[3]]
    colors = Image.new("RGB", (len(occupied), 1))
    colors.putdata(occupied)
    q = colors.quantize(colors=63, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    p = q.getpalette()
    palette = [0] + [int.from_bytes(bytes((*p[i*3:i*3+3], 255)), "little") for i in range(63)]
    indices = iter(q.get_flattened_data())
    values = [next(indices)+1 if c[3] else 0 for c in pixels]
    native = Image.new("RGBA", SOURCE_SIZE)
    native.putdata([tuple(c.to_bytes(4, "little")) for c in (palette[i] for i in values)])
    canonical_png(folder / "source-cells.png", native)
    pieces = []
    for row in range(2):
        for col in range(4):
            pieces.append((f"tile-{row}-{col}", (col*128, row*96, (col+1)*128, (row+1)*96), False))
    pieces.append(("crest", CROWN, True))
    sprites, roles, parts = [], {}, []
    for name, (x0, y0, x1, y1), crown in pieces:
        image = Image.new("RGBA", (x1-x0, y1-y0))
        data, energy, silver, award = [], [], [], []
        for y in range(y0, y1):
            for x in range(x0, x1):
                i = y*512+x
                color = tuple(palette[values[i]].to_bytes(4, "little"))
                # Only the gold award face moves inside the static stadium.
                # Silver rails/counters and all letter geometry stay in tiles.
                in_crown = (CROWN[0] <= x < CROWN[2] and CROWN[1] <= y < CROWN[3]
                            and color[0] >= color[1] >= color[2] and color[0]-color[2] >= 40 and color[0] >= 90)
                c = tuple(palette[values[i]].to_bytes(4, "little")) if crown == in_crown else (0, 0, 0, 0)
                local_index = len(data)
                data.append(c)
                if c[3] and crown:
                    award.append(local_index)
                elif c[3] and c[2] > c[0]+25 and c[1] > c[0]+15:
                    energy.append(local_index)
                elif c[3] and min(c[:3]) >= 135 and max(c[:3])-min(c[:3]) <= 35:
                    silver.append(local_index)
        image.putdata(data)
        canonical_png(folder / f"{name}.png", image)
        sprites.append({"name": name, "source": f"{name}.png", "topology": "holes-allowed"})
        roles[name] = {"energy_cells": energy, "scan_cells": silver, "award_cells": award}
        parts.append({"name": name, "sprite": name, "transform": {"position": [(x0+x1)/2-256, (y0+y1)/2-96]}})
    manifest = {
        "schema": 1, "palette": {"mode": "provided", "max_colors": 64, "colors": palette},
        "sprites": sprites, "groups": [{"name": "title-mark", "parts": parts}],
        "budgets": {"group_cells": 45000, "sprite_cells": 12288, "group_transforms": 9},
        "provenance": {"source": "reference.png", "source_sha256": digest((FOLDER/"reference.png").read_bytes()), "tool": "built-in image_gen", "model_version": "not exposed", "conversion": "Explicit 2048x768 to512x192 nearest sampling, alpha>=128 and maxRGB>=40 dark-cutout threshold, median-cut63 opaque colors; no component-size filter. Gold award face separated without duplicating cells; static stadium/letters retain all remaining cells."},
        "extensions": {"title_roles": {"source_size": list(SOURCE_SIZE), "parts": roles}},
    }
    (folder/"manifest.json").write_text(json.dumps(manifest, indent=2)+"\n", encoding="utf-8", newline="\n")
    atlas = module().pack(folder/"manifest.json")
    (folder/"atlas.json").write_text(json.dumps(atlas, separators=(",", ":"))+"\n", encoding="utf-8", newline="\n")
    error = [abs(a-b) for c,i in zip(pixels,values) if c[3] for a,b in zip(c[:3],palette[i].to_bytes(4,"little")[:3])]
    report = {
        "source_dimensions": list(source.size), "grid": list(SOURCE_SIZE), "source_sha256": digest((FOLDER/"reference.png").read_bytes()),
        "source_alpha_threshold_changes": sum(c[3] not in (0,255) for c in source.get_flattened_data()),
        "sampled_alpha_threshold_changes": sum(a[3]!=b[3] for a,b in zip(original,pixels)),
        "occupied_cells": len(occupied), "palette_entries": len(palette),
        "rgb_channel_mean_absolute_error": sum(error)/len(error), "rgb_channel_max_absolute_error": max(error),
        "decoded_rgba_sha256": digest(native.tobytes()), "pack_sha256": digest((folder/"atlas.json").read_bytes()),
        "component_size_filter_used": False, "lossless_reference_claim": False,
        "opaque_sampled_before_dark_cutout": sum(c[3]>=128 for c in original),
        "dark_cutout_cells_removed": sum(c[3]>=128 and max(c[:3])<CUTOUT_FLOOR for c in original),
        "dark_cutout_floor": CUTOUT_FLOOR,
        "runtime_animation": "Explicit gold award-face relative movement/glint, arena light chase and narrow metal sweep; no added glitter or model-generated animation.",
    }
    (folder/"conversion.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8", newline="\n")
    return atlas, report

if __name__ == "__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=FOLDER)
    args=parser.parse_args()
    _, report=build(args.output)
    print(json.dumps(report, indent=2))
