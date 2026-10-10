#!/usr/bin/env python3
"""Convert the generated Galaxy Royale atlas into engine pixel-cell data.

Requires Python 3 and Pillow 12.3 or newer (verified with Pillow 12.3.0).
For a fresh Python environment: python -m pip install Pillow==12.3.0
Run this single command from the repository root (PowerShell or Unix shell):
  python scripts/convert-atlas.py --input artifacts/concepts/galaxy-royale-sprite-atlas.png --output assets/pixels/atlas.json --preview artifacts/pixel-atlas-preview.png --provenance artifacts/asset-provenance.json --session-date 2026-10-11

On this Windows workstation, replace python with the bundled runtime:
  & 'C:/Users/mitch/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'

The source is four columns and two rows. The row boundary is explicitly 410;
equal-height rows clip the dock. Sprite indices are row-major, with transparent
index zero and 63 shared opaque colors. No blur, interpolation or dithering is
used. Output cells are source-derived pixels, ready for individual engine quads.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from datetime import date
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, __version__ as pillow_version


SPRITES = (
    ("player", 48),
    ("dart", 32),
    ("bastion", 40),
    ("coil_head", 32),
    ("coil_segment", 16),
    ("asteroid", 40),
    ("dock", 64),
    ("dart_boss", 64),
)
ALPHA_THRESHOLD = 128
OPAQUE_COLORS = 63
ROW_BOUNDARY = 410


def largest_component(frame: Image.Image) -> tuple[Image.Image, dict]:
    """Hard-mask alpha and retain one deterministic 8-neighbor component."""
    width, height = frame.size
    pixels = list(frame.get_flattened_data())
    remaining = bytearray(pixel[3] >= ALPHA_THRESHOLD for pixel in pixels)
    source_occupied = sum(remaining)
    largest: list[int] = []
    component_count = 0
    for first in range(len(remaining)):
        if not remaining[first]:
            continue
        component_count += 1
        remaining[first] = 0
        pending = [first]
        component = []
        while pending:
            current = pending.pop()
            component.append(current)
            x, y = current % width, current // width
            for adjacent_y in range(max(0, y - 1), min(height, y + 2)):
                for adjacent_x in range(max(0, x - 1), min(width, x + 2)):
                    adjacent = adjacent_y * width + adjacent_x
                    if remaining[adjacent]:
                        remaining[adjacent] = 0
                        pending.append(adjacent)
        # Equal-sized components prefer the earliest source pixel.
        if len(component) > len(largest):
            largest = component
    if not largest:
        raise ValueError("Source frame has no occupied pixels above alpha 128.")
    clean = Image.new("RGBA", frame.size, (0, 0, 0, 0))
    clean_pixels = clean.load()
    for index in largest:
        red, green, blue, _ = pixels[index]
        clean_pixels[index % width, index // width] = (red, green, blue, 255)
    box = clean.getchannel("A").getbbox()
    assert box is not None
    return clean.crop(box), {
        "alpha_mask_occupied": source_occupied,
        "components": component_count,
        "retained_source_pixels": len(largest),
        "removed_component_pixels": source_occupied - len(largest),
        "tight_bbox_in_crop": list(box),
    }


def fit_cells(sprite: Image.Image, size: int) -> Image.Image:
    scale = min(size / sprite.width, size / sprite.height)
    resized_size = (
        max(1, min(size, round(sprite.width * scale))),
        max(1, min(size, round(sprite.height * scale))),
    )
    resized = sprite.resize(resized_size, Image.Resampling.NEAREST)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    canvas.paste(resized, ((size - resized.width) // 2, (size - resized.height) // 2))
    return canvas


def quantize_shared(images: dict[str, Image.Image]) -> dict:
    occupied_colors = []
    for sprite in images.values():
        occupied_colors.extend(pixel[:3] for pixel in sprite.get_flattened_data() if pixel[3])
    samples = Image.new("RGB", (len(occupied_colors), 1))
    samples.putdata(occupied_colors)
    quantized = samples.quantize(
        colors=OPAQUE_COLORS,
        method=Image.Quantize.MEDIANCUT,
        dither=Image.Dither.NONE,
    )
    rgb_palette = quantized.getpalette()
    assert rgb_palette is not None
    palette = [0]
    for offset in range(0, OPAQUE_COLORS * 3, 3):
        red, green, blue = rgb_palette[offset : offset + 3]
        palette.append(red | (green << 8) | (blue << 16) | (255 << 24))
    occupied_indices = iter(quantized.get_flattened_data())
    sprites = {}
    for name, sprite in images.items():
        indices = [next(occupied_indices) + 1 if pixel[3] else 0 for pixel in sprite.get_flattened_data()]
        sprites[name] = {"width": sprite.width, "height": sprite.height, "indices": indices}
    return {"version": 1, "palette": palette, "sprites": sprites}


def unpack(color: int) -> tuple[int, int, int, int]:
    return tuple((color >> shift) & 255 for shift in (0, 8, 16, 24))


def preview_atlas(atlas: dict, output: Path) -> None:
    zoom = 5
    grid_size = 64 * zoom
    cell_width, cell_height = grid_size + 36, grid_size + 66
    header_height = 66
    preview = Image.new("RGB", (cell_width * 4, cell_height * 2 + header_height), "#081222")
    draw = ImageDraw.Draw(preview)
    font = ImageFont.load_default(size=14)
    title_font = ImageFont.load_default(size=22)
    draw.text((20, 16), "GALAXY ROYALE / ENGINE PIXEL ATLAS", fill="#b7e6dc", font=title_font)
    draw.text((20, 44), "64 SHARED COLORS / BINARY ALPHA / NEAREST-NEIGHBOR CELLS / 5x PREVIEW", fill="#688daa", font=font)
    for index, (name, sprite) in enumerate(atlas["sprites"].items()):
        column, row = index % 4, index // 4
        x, y = column * cell_width + 18, header_height + row * cell_height
        draw.rectangle((x, y, x + grid_size - 1, y + grid_size - 1), fill="#0c1b2a", outline="#294254")
        for grid in range(8 * zoom, grid_size, 8 * zoom):
            draw.line((x + grid, y + 1, x + grid, y + grid_size - 2), fill="#112333")
            draw.line((x + 1, y + grid, x + grid_size - 2, y + grid), fill="#112333")
        pixels = Image.new("RGBA", (sprite["width"], sprite["height"]))
        pixels.putdata([unpack(atlas["palette"][color]) for color in sprite["indices"]])
        pixels = pixels.resize((pixels.width * zoom, pixels.height * zoom), Image.Resampling.NEAREST)
        preview.paste(pixels, (x + (grid_size - pixels.width) // 2, y + (grid_size - pixels.height) // 2), pixels)
        occupied = sum(color != 0 for color in sprite["indices"])
        draw.text((x, y + grid_size + 12), name.upper().replace("_", " "), fill="#c0d6e6", font=font)
        draw.text((x, y + grid_size + 33), f"{sprite['width']} x {sprite['height']} / {occupied} occupied cells", fill="#668ba3", font=font)
    output.parent.mkdir(parents=True, exist_ok=True)
    preview.save(output, format="PNG", optimize=False)


def file_name(path: Path) -> str:
    try:
        return path.resolve().relative_to(Path.cwd().resolve()).as_posix()
    except ValueError:
        return path.resolve().as_posix()


def write_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, separators=(",", ":")) + "\n", encoding="utf-8")


def convert(source: Path, output: Path, preview: Path, provenance: Path, session_date: str) -> dict:
    date.fromisoformat(session_date)
    with Image.open(source) as original:
        if original.mode != "RGBA":
            raise ValueError("Atlas source must have an actual RGBA alpha channel.")
        image = original.copy()
    if image.width < 4 or image.height <= ROW_BOUNDARY:
        raise ValueError("Atlas source is too small for the explicit 4-column, y=410 layout.")
    images = {}
    frame_records = {}
    for index, (name, size) in enumerate(SPRITES):
        column, row = index % 4, index // 4
        box = (
            column * image.width // 4,
            0 if row == 0 else ROW_BOUNDARY,
            (column + 1) * image.width // 4,
            ROW_BOUNDARY if row == 0 else image.height,
        )
        cropped, record = largest_component(image.crop(box))
        images[name] = fit_cells(cropped, size)
        frame_records[name] = {"source_crop": list(box), **record, "output_width": size, "output_height": size}
    atlas = quantize_shared(images)
    write_json(output, atlas)
    preview_atlas(atlas, preview)
    for name, sprite in atlas["sprites"].items():
        frame_records[name]["occupied_output_cells"] = sum(index != 0 for index in sprite["indices"])
    record = {
        "version": 1,
        "source": {"path": file_name(source), "sha256": hashlib.sha256(source.read_bytes()).hexdigest(), "width": image.width, "height": image.height, "mode": image.mode},
        "generation": {"tool": "image_gen.imagegen", "model": None, "model_note": "The specific image model identifier was not exposed in the provided session artifact metadata.", "session_date": session_date},
        "conversion": {"script": "scripts/convert-atlas.py", "pillow_version": pillow_version, "layout": "4 columns, 2 rows; row boundary y=410", "alpha_threshold": ALPHA_THRESHOLD, "component_connectivity": 8, "component_policy": "Largest component per crop; equal sizes prefer earliest source pixel", "fit": "Tight alpha bounding box, aspect-preserving rounded size, nearest-neighbor sampling, centered in output frame", "palette": "One median-cut palette: index 0 transparent, 63 opaque RGB colors; no dithering", "rgba_encoding": "u32 little-endian RGBA: red in lowest byte; opaque alpha=255", "frames": frame_records},
        "outputs": {"atlas": file_name(output), "atlas_sha256": hashlib.sha256(output.read_bytes()).hexdigest(), "preview": file_name(preview), "preview_sha256": hashlib.sha256(preview.read_bytes()).hexdigest()},
    }
    provenance.parent.mkdir(parents=True, exist_ok=True)
    provenance.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    return record


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--input", type=Path, required=True, help="Generated RGBA source atlas")
    parser.add_argument("--output", type=Path, required=True, help="Engine version-1 indexed JSON atlas")
    parser.add_argument("--preview", type=Path, required=True, help="Magnified PNG review preview")
    parser.add_argument("--provenance", type=Path, required=True, help="Source and conversion provenance JSON")
    parser.add_argument("--session-date", required=True, help="Actual generation session date, YYYY-MM-DD")
    args = parser.parse_args()
    paths = [args.input.resolve(), args.output.resolve(), args.preview.resolve(), args.provenance.resolve()]
    if len(set(paths)) != 4:
        parser.error("Input, output, preview and provenance paths must be distinct.")
    try:
        record = convert(args.input, args.output, args.preview, args.provenance, args.session_date)
    except (OSError, ValueError) as error:
        parser.exit(1, f"Atlas conversion failed: {error}\n")
    print(json.dumps({"atlas": record["outputs"]["atlas"], "sha256": record["outputs"]["atlas_sha256"], "sprites": {name:frame["occupied_output_cells"] for name, frame in record["conversion"]["frames"].items()}}, indent=2))


if __name__ == "__main__":
    main()
