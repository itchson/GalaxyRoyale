"""Author compact Bastion40 pod rims/recesses on its exact frozen938-cell mask.

No resizing, quantization, new cells, palette changes or new animated sites.
Only --apply writes the guarded Bastion indices span in the legacy atlas.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
from PIL import Image, __version__ as pillow_version

ROOT = Path(__file__).resolve().parents[1]
DEFAULT = ROOT / "assets/pixels/bastion-detail"
SIGNALS = frozenset((4, 6, 33))
PATTERN = re.compile(rb'("bastion":\{"width":40,"height":40,"indices":\[)([0-9,]+)(\]\})')
spec = importlib.util.spec_from_file_location("bastion_asset_pack", ROOT / "scripts/asset-pack.py")
AP = importlib.util.module_from_spec(spec)
spec.loader.exec_module(AP)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((json.dumps(value, indent=2, allow_nan=False) + "\n").encode("utf-8"))


def authored_indices(baseline):
    old = baseline["sprites"]["bastion"]["indices"]
    AP.require(len(old) == 1600 and sum(bool(i) for i in old) == 938,
               "Frozen Bastion grid/mask changed")
    # Short directed bevels around each lower housing. The strongest highlight
    # is only two top-left cells; the opposite/down-facing rim stays in shade.
    rim = {
        (-1, -3): 7, (0, -3): 5, (1, -3): 11,
        (-2, -2): 7, (2, -2): 16,
        (-3, -1): 11, (3, -1): 25,
        (-3, 0): 16, (3, 0): 31,
        (-2, 1): 16, (2, 1): 31,
        (-1, 2): 25, (0, 2): 13, (1, 2): 31,
    }
    result = old.copy()
    for i, original in enumerate(old):
        if not original or original in SIGNALS:
            continue
        x, y = i % 40, i // 40
        color = original
        # Preserve the original graded orange plate fields. Inside the two
        # lower pods, rim/recess/short vents organize existing occupied cells.
        for center in (10, 29):
            dx, dy = x - center, y - 27
            if (dx, dy) in rim:
                color = rim[(dx, dy)]
            elif abs(dx) <= 1 and -2 <= dy <= 1:
                color = 49
                if dy in (-1, 1):
                    color = (16, 11, 25)[dx + 1]
            elif dy == 3 and abs(dx) <= 1:
                color = (31, 13, 31)[dx + 1]
        # A narrow shadow and a few alloy corners frame the retained warm core.
        # These do not convert its fixed amber animated cells into a canopy.
        if x == 17 and 11 <= y <= 17:
            color = 31
        elif x == 22 and 12 <= y <= 16:
            color = 25
        if (x, y) in ((17, 10), (22, 11)):
            color = 11
        # Two small paired attachment/vent accents, not continuous bright rails.
        if x in (7, 32) and y in (31, 32):
            color = 16 if y == 31 else 31
        AP.require(color not in SIGNALS, "New material must not add a signal site")
        result[i] = color
    AP.require([bool(i) for i in result] == [bool(i) for i in old], "Bastion occupancy changed")
    AP.require(all(a == b for a, b in zip(old, result) if a in SIGNALS or b in SIGNALS),
               "Bastion signal sites changed")
    return result


def surgical_atlas(raw, baseline, candidate):
    matches = list(PATTERN.finditer(raw))
    AP.require(len(matches) == 1, "Exact40 Bastion span required")
    found = matches[0]
    old = baseline["sprites"]["bastion"]["indices"]
    current = [int(i) for i in found.group(2).split(b",")]
    AP.require(current in (old, candidate), "Unexpected Bastion art; refusing overwrite")

    def encoded(values):
        return json.dumps(values, separators=(",", ":")).encode()[1:-1]

    restored = raw[:found.start(2)] + encoded(old) + raw[found.end(2):]
    AP.require(digest(restored.replace(b"\r\n", b"\n")) == baseline["source_atlas_sha256_lf"],
               "Non-Bastion atlas bytes changed; review new baseline first")
    changed = raw[:found.start(2)] + encoded(candidate) + raw[found.end(2):]
    before, after = json.loads(raw), json.loads(changed)
    AP.require(after["palette"] == baseline["palette"] == before["palette"], "Palette changed")
    AP.require({k: v for k, v in after["sprites"].items() if k != "bastion"} ==
               {k: v for k, v in before["sprites"].items() if k != "bastion"}, "Other sprites changed")
    return changed


def author(output=DEFAULT):
    baseline = AP.read_json(DEFAULT / "baseline.json")
    AP.validate({key: baseline[key] for key in ("version", "palette", "sprites")})
    indices = authored_indices(baseline)
    palette = baseline["palette"]
    output.mkdir(parents=True, exist_ok=True)
    image = Image.new("P", (40, 40))
    image.putpalette([channel for color in palette for channel in AP.rgba(color)[:3]])
    image.info["transparency"] = bytes(AP.rgba(color)[3] for color in palette)
    image.putdata(indices)
    image.save(output / "source.png", optimize=False, bits=8)
    groups = [{"name": name, "parts": [{"name": "hull", "sprite": "bastion", "transform": {
        "basis_x": [scale, 0], "basis_y": [0, scale]}}]}
        for name, scale in (("bastion-native", 1), ("bastion-game", .7))]
    manifest = {
        "schema": 1, "palette": {"mode": "provided", "max_colors": 64, "colors": palette},
        "sprites": [{"name": "bastion", "source": "source.png", "encoding": "indices", "topology": "holes-allowed"}],
        "groups": groups, "animations": [{"name": "bastion-review", "loop": False,
            "frames": [{"kind": "group", "name": group["name"], "duration_ms": 500} for group in groups]}],
        "budgets": {"sprite_cells": 938, "group_cells": 938, "group_transforms": 1, "extent": 64},
        "provenance": {
            "source_type": "original-authored40-grid-pod-rims-and-mechanical-recess-clusters-on-frozen-mask",
            "reference_commit": baseline["reference_commit"],
            "baseline_sha256_lf": digest((DEFAULT / "baseline.json").read_bytes().replace(b"\r\n", b"\n")),
            "animation_policy": "Preserve exact baseline sites/indices4,6,33; all new material indices are non-pulsing",
        },
    }
    write_json(output / "manifest.json", manifest)
    packed = AP.pack(output / "manifest.json")
    write_json(output / "pack.json", packed)
    AP.require(packed["sprites"]["bastion"]["indices"] == indices and packed["palette"] == palette,
               "Exact indexed conversion changed candidate")
    old = baseline["sprites"]["bastion"]["indices"]
    provenance = {
        "source_grid": [40, 40], "occupied": 938, "palette_entries": 64,
        "changed_color_cells": sum(a != b for a, b in zip(old, indices)),
        "preserved_signal_cells": sum(a in SIGNALS for a in old),
        "mask_sha256": digest(bytes(bool(i) for i in old)),
        "baseline_indices_sha256": digest(bytes(old)), "candidate_indices_sha256": digest(bytes(indices)),
        "generation_model": None,
        "authorship": "Compact shaded lower-pod rims, inset vents and framed warm core; original graded orange armor retained; no generated concept",
        "text_fingerprint_policy": "UTF-8 text hashes canonicalize CRLF to LF; PNG/indices/mask hashes are exact binary",
        "pillow": pillow_version, "source_sha256": digest((output / "source.png").read_bytes()),
        "manifest_sha256_lf": digest((output / "manifest.json").read_bytes().replace(b"\r\n", b"\n")),
        "pack_sha256_lf": digest((output / "pack.json").read_bytes().replace(b"\r\n", b"\n")),
        "author_script_sha256_lf": digest(Path(__file__).read_bytes().replace(b"\r\n", b"\n")),
        "converter_sha256_lf": digest((ROOT / "scripts/asset-pack.py").read_bytes().replace(b"\r\n", b"\n")),
    }
    write_json(output / "provenance.json", provenance)
    return packed, provenance


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    packed, provenance = author(args.output_dir)
    if args.apply:
        baseline = AP.read_json(DEFAULT / "baseline.json")
        path = ROOT / "assets/pixels/atlas.json"
        path.write_bytes(surgical_atlas(path.read_bytes(), baseline, packed["sprites"]["bastion"]["indices"]))
    print(json.dumps({"output": str(args.output_dir), "applied": args.apply, **provenance}, indent=2))


if __name__ == "__main__":
    main()
