#!/usr/bin/env python3
"""Convert generated interface artwork to shared-palette engine pixel cells.

Requires Python 3 and Pillow 12.3.0. Reuses shared quantization and RGBA encoding
from scripts/convert-atlas.py. No component is removed: logo letters, its star
and separated ornaments remain intact. Alpha is thresholded at 128; tight
bounding boxes are aspect-fitted with nearest sampling and no dithering.

Run from the repository root:
  python scripts/convert-ui-atlas.py --input artifacts/concepts/galaxy-royale-ui-atlas.png --output assets/ui/atlas.json --preview assets/ui/atlas-preview.png --provenance assets/ui/provenance.json --session-date 2026-10-11

The four exact source regions are deliberately not a regular grid. The wide
button crosses the source midpoint. Every preview pixel is magnified 8 times.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import sys
from datetime import date
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, __version__ as pillow_version

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("pixel_atlas_conversion", Path(__file__).with_name("convert-atlas.py"))
assert spec is not None and spec.loader is not None
pixel_conversion = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pixel_conversion)

FRAMES = (
    ("logo", (32, 80, 924, 472), (256, 104)),
    ("panel", (928, 80, 1760, 472), (96, 48)),
    ("button", (32, 590, 1030, 796), (96, 20)),
    ("seal", (1152, 480, 1555, 860), (40, 40)),
)


def hard_crop(source: Image.Image, box: tuple[int, int, int, int]) -> tuple[Image.Image, dict]:
    frame = source.crop(box)
    mask = frame.getchannel("A").point([0] * 128 + [255] * 128)
    occupied = sum(alpha != 0 for alpha in mask.get_flattened_data())
    bounds = mask.getbbox()
    if bounds is None:
        raise ValueError(f"Source region {box} contains no opaque artwork.")
    frame.putalpha(mask)
    return frame.crop(bounds), {"source_crop": list(box), "tight_bbox_in_crop": list(bounds), "source_occupied_cells": occupied, "component_policy": "Keep all components; no connected-component filtering"}


def fit(sprite: Image.Image, size: tuple[int, int]) -> Image.Image:
    factor = min(size[0] / sprite.width, size[1] / sprite.height)
    resized_size = (max(1, min(size[0], round(sprite.width * factor))), max(1, min(size[1], round(sprite.height * factor))))
    resized = sprite.resize(resized_size, Image.Resampling.NEAREST)
    frame = Image.new("RGBA", size, (0, 0, 0, 0))
    frame.paste(resized, ((size[0] - resized.width) // 2, (size[1] - resized.height) // 2))
    return frame


def validate(atlas: dict) -> None:
    if atlas["version"] != 1 or len(atlas["palette"]) != 64 or atlas["palette"][0] != 0:
        raise ValueError("Atlas version or shared 64-color palette is invalid.")
    if not all(color >> 24 == 255 for color in atlas["palette"][1:]):
        raise ValueError("Occupied palette colors must have alpha 255.")
    for name, _box, (width, height) in FRAMES:
        sprite = atlas["sprites"][name]
        if (sprite["width"], sprite["height"]) != (width, height) or len(sprite["indices"]) != width * height:
            raise ValueError(f"Invalid row-major frame for {name}.")
        if not all(type(index) is int and 0 <= index < 64 for index in sprite["indices"]) or not any(sprite["indices"]):
            raise ValueError(f"Invalid palette indices for {name}.")


def preview_atlas(atlas: dict, path: Path) -> None:
    zoom = 8
    canvas = Image.new("RGB", (2104, 1464), "#081222")
    draw = ImageDraw.Draw(canvas)
    heading = ImageFont.load_default(size=24)
    label = ImageFont.load_default(size=18)
    draw.text((28, 16), "GALAXY ROYALE / INTERFACE PIXEL CELLS", fill="#b7e6dc", font=heading)
    draw.text((28, 51), "8x NEAREST PREVIEW / 64 SHARED COLORS / HARD ALPHA / ALL COMPONENTS RETAINED", fill="#719ab4", font=label)
    positions = {"logo": (28, 110), "panel": (28, 1040), "button": (836, 1040), "seal": (1716, 1040)}
    for name, sprite in atlas["sprites"].items():
        x, y = positions[name]
        cells = Image.new("RGBA", (sprite["width"], sprite["height"]))
        cells.putdata([pixel_conversion.unpack(atlas["palette"][index]) for index in sprite["indices"]])
        cells = cells.resize((cells.width * zoom, cells.height * zoom), Image.Resampling.NEAREST)
        draw.rectangle((x, y, x + cells.width - 1, y + cells.height - 1), fill="#0d1b2a", outline="#243f54")
        canvas.paste(cells, (x, y), cells)
        occupied = sum(index != 0 for index in sprite["indices"])
        draw.text((x, y - 26), f"{name.upper()} / {sprite['width']} x {sprite['height']} / {occupied} CELLS", fill="#aac4d7", font=label)
    path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(path, format="PNG", optimize=False)


def convert(source: Path, output: Path, preview: Path, provenance: Path, session_date: str) -> dict:
    date.fromisoformat(session_date)
    with Image.open(source) as original:
        if original.mode != "RGBA" or original.size != (1774, 887):
            raise ValueError("Expected the inspected RGBA 1774x887 UI source; revise explicit crops for a different source layout.")
        image = original.copy()
    sprites = {}
    frames = {}
    for name, box, size in FRAMES:
        cropped, record = hard_crop(image, box)
        sprites[name] = fit(cropped, size)
        frames[name] = {**record, "output_width": size[0], "output_height": size[1]}
    atlas = pixel_conversion.quantize_shared(sprites)
    validate(atlas)
    pixel_conversion.write_json(output, atlas)
    preview_atlas(atlas, preview)
    for name, sprite in atlas["sprites"].items():
        frames[name]["occupied_output_cells"] = sum(index != 0 for index in sprite["indices"])
    result = {
        "version": 1,
        "source": {"path": pixel_conversion.file_name(source), "sha256": hashlib.sha256(source.read_bytes()).hexdigest(), "width": image.width, "height": image.height, "mode": image.mode},
        "generation": {"tool": "image_gen.imagegen", "model": None, "model_note": "Specific image model metadata was not exposed with the provided session artifact.", "session_date": session_date},
        "conversion": {"script": "scripts/convert-ui-atlas.py", "shared_palette_script": "scripts/convert-atlas.py", "pillow_version": pillow_version, "alpha_threshold": 128, "fit": "Tight hard-alpha bbox, aspect-preserving nearest resample, centered in target frame", "palette": "Index0 transparent; 63 opaque median-cut RGB colors; no dithering", "encoding": "u32 little-endian RGBA; row-major palette indices", "frames": frames},
        "outputs": {"atlas": pixel_conversion.file_name(output), "atlas_sha256": hashlib.sha256(output.read_bytes()).hexdigest(), "preview": pixel_conversion.file_name(preview), "preview_sha256": hashlib.sha256(preview.read_bytes()).hexdigest()},
    }
    font_folder = Path(__file__).resolve().parent.parent / "apps/game/public/fonts"
    font_file, license_file = font_folder / "Tiny5.ttf", font_folder / "OFL.txt"
    if font_file.is_file() and license_file.is_file():
        result["supporting_assets"] = [{"name":"Tiny5", "path":pixel_conversion.file_name(font_file), "sha256":hashlib.sha256(font_file.read_bytes()).hexdigest(), "supplied_license_path":pixel_conversion.file_name(license_file), "supplied_license_sha256":hashlib.sha256(license_file.read_bytes()).hexdigest(), "license_note":"The font and its supplied SIL Open Font License 1.1 text are retained locally."}]
    provenance.parent.mkdir(parents=True, exist_ok=True)
    provenance.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--preview", required=True, type=Path)
    parser.add_argument("--provenance", required=True, type=Path)
    parser.add_argument("--session-date", required=True)
    args = parser.parse_args()
    if len({path.resolve() for path in (args.input, args.output, args.preview, args.provenance)}) != 4:
        parser.error("Input, atlas, preview and provenance paths must be distinct.")
    try:
        result = convert(args.input, args.output, args.preview, args.provenance, args.session_date)
    except (ValueError, OSError) as error:
        parser.exit(1, f"UI atlas conversion failed: {error}\n")
    print(json.dumps({"atlas": result["outputs"]["atlas"], "sha256": result["outputs"]["atlas_sha256"], "occupied_cells": {name:frame["occupied_output_cells"] for name,frame in result["conversion"]["frames"].items()}}, indent=2))


if __name__ == "__main__":
    main()
