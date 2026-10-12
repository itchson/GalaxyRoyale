"""Original overhead ice nucleus and coma cells; exact asset-pack import."""
import argparse
import importlib.util
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("asset_pack", ROOT / "scripts/asset-pack.py")
asset_pack = importlib.util.module_from_spec(spec)
spec.loader.exec_module(asset_pack)

ICE = [(19, 32, 44, 255), (29, 49, 64, 255), (46, 76, 94, 255),
       (67, 108, 123, 255), (99, 144, 157, 255), (142, 184, 190, 255),
       (181, 210, 208, 255), (217, 230, 218, 255)]
HOT = [(93, 104, 113, 255), (151, 147, 128, 255), (223, 194, 151, 255),
       (244, 219, 181, 255)]
COMA = [(41, 66, 82, 255), (68, 113, 134, 255), (115, 173, 188, 255)]


def author(output):
    output.mkdir(parents=True, exist_ok=True)
    source = output / "source"
    source.mkdir(exist_ok=True)
    head = Image.new("RGBA", (48, 32))
    shape = Image.new("1", head.size)
    d = ImageDraw.Draw(shape)
    # Irregular planar slab with a heated leading edge, not an upright cone.
    d.polygon([(7, 14), (10, 8), (17, 5), (22, 6), (29, 4), (37, 8),
               (42, 12), (43, 18), (38, 24), (31, 27), (24, 25),
               (18, 27), (11, 23), (8, 19)], fill=1)
    for y in range(32):
        for x in range(48):
            if not shape.getpixel((x, y)):
                continue
            # Broad facets and localized grooves, rather than high-frequency fill.
            face = (x * 3 + y * 2) // 12
            tone = max(0, min(7, 6 - y // 7 + (face % 3 == 0)))
            if x < 14 or y > 23:
                tone = max(0, tone - 2)
            if abs(y - (x * .23 + 8)) < 1.1:
                tone = max(0, tone - 3)
            if abs(y - (-x * .38 + 27)) < .7 and x < 30:
                tone = max(0, tone - 2)
            crater = ((x - 25) / 4.2) ** 2 + ((y - 18) / 3.1) ** 2
            if crater < .72:
                tone = max(0, tone - 2)
            elif .72 <= crater < 1.4:
                tone = min(7, tone + (x + y < 43))
            leading = not shape.getpixel((min(47, x + 1), y)) and x > 30
            color = HOT[min(3, (x - 30) // 4)] if leading else ICE[tone]
            head.putpixel((x, y), color)
    detail = ImageDraw.Draw(head)
    detail.line([(17, 8), (22, 9), (28, 7), (33, 9)], fill=ICE[7])
    detail.line([(12, 13), (15, 10)], fill=ICE[5])
    detail.line([(29, 19), (34, 21), (37, 17)], fill=ICE[2])
    detail.point((25, 13), fill=ICE[7])
    coma = Image.new("RGBA", (64, 44))
    for y in range(44):
        for x in range(64):
            dx, dy = x + .5 - 32, y + .5 - 22
            outer = ((dx - 3) / 23) ** 2 + (dy / 15) ** 2
            inner = ((dx - 1) / 18) ** 2 + (dy / 10.5) ** 2
            if .85 < outer < 1.15 and inner > 1.02 and (x + y * 3) % 7 < 3:
                coma.putpixel((x, y), COMA[(x + y) % 3])
            elif dx > 15 and inner > .85 and outer < 1 and (x + y) % 5 == 0:
                coma.putpixel((x, y), COMA[2])
    images = {"nucleus": head, "coma": coma}
    for name, image in images.items():
        image.save(source / f"{name}.png")
    palette = [0] + sorted({asset_pack.color(c) for image in images.values()
                            for c in image.get_flattened_data() if c[3]})
    provenance = {"authoring": "Original explicit overhead nucleus facets, crater/grooves, heated rim and sparse coma mask",
                  "ai_generated": False, "conversion": "Exact binary-alpha authored PNG; no resize, threshold, dithering or component removal",
                  "runtime": "Cached native PixelBody plus bounded analytic local tail; no Game/save/RNG authority"}
    manifest = {"schema": 1, "palette": {"mode": "provided", "max_colors": 64, "colors": palette},
                "sprites": [{"name": name, "source": f"source/{name}.png", "topology": "holes-allowed"} for name in images],
                "provenance": provenance}
    asset_pack.write_json(output / "manifest.json", manifest)
    atlas = asset_pack.pack(output / "manifest.json")
    asset_pack.write_json(output / "atlas.json", atlas)
    asset_pack.write_json(output / "provenance.json", provenance)
    asset_pack.preview(atlas, output / "authored-preview.png", 8)
    return atlas


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=ROOT / "assets/pixels/shooting-star")
    args = parser.parse_args()
    atlas = author(args.output_dir)
    print({name: sum(i != 0 for i in sprite["indices"]) for name, sprite in atlas["sprites"].items()})
