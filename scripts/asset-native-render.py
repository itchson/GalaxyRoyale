"""Flat software review image from ACTUAL native extracted quad corners.

No smoothing, GPU shading, live simulation or device-performance claim.
"""
import argparse, importlib.util, math, sys
from pathlib import Path
from PIL import Image, ImageDraw
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('asset_pack', Path(__file__).with_name('asset-pack.py'))
ap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ap)
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--scale', type=int, default=6)
args = parser.parse_args()
ap.require(args.input.resolve() != args.output.resolve(), 'Input/output must differ')
ap.integer(args.scale, 1, 16, 'scale')
proof = ap.read_json(args.input)
frames = proof['frames']
ap.require(1 <= len(frames) <= 256, 'Frame count bound')
sizes = []
for f in frames:
    bounds = ap.vector(f['bounds'], 4, 'native bounds')
    ap.require(bounds[2] > bounds[0] and bounds[3] > bounds[1], 'Empty native bounds')
    sizes.append((math.ceil((bounds[2] - bounds[0]) * args.scale) + 32, math.ceil((bounds[3] - bounds[1]) * args.scale) + 68))
width = sum((s[0] for s in sizes))
height = max((s[1] for s in sizes))
ap.require(width * height <= ap.MAX_SOURCE_PIXELS, 'Native preview exceeds pixel bound')
image = Image.new('RGB', (width, height), (5, 10, 17))
draw = ImageDraw.Draw(image)
left = 0
for f, size in zip(frames, sizes):
    draw.text((left + 12, 10), f"{f['group']} / {f['duration_ms']}ms", fill=(176, 213, 218))
    draw.text((left + 12, 26), f"{f['occupied_cells']} cells / {f['transforms']} transforms", fill=(108, 151, 162))
    for cell in f['cells']:
        corners = [ap.vector(p, 2, 'native corner') for p in cell['corners']]
        ap.require(len(corners) == 4, 'Native quad needs four corners')
        c = ap.rgba(ap.integer(cell['color'], 0, 4294967295, 'native color'))
        if not c[3]:
            continue
        rgb = tuple((round(v * c[3] / 255 + bg * (1 - c[3] / 255)) for v, bg in zip(c[:3], (5, 10, 17))))
        coords = [(left + 16 + (p[0] - f['bounds'][0]) * args.scale, 52 + (p[1] - f['bounds'][1]) * args.scale) for p in corners]
        draw.polygon(coords, fill=rgb)
    left += size[0]
args.output.parent.mkdir(parents=True, exist_ok=True)
image.save(args.output)
print(args.output)
