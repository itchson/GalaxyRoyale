"""Higher-detail candidate: native256 source tiled into <=128-cell bodies.

Uses the frozen128 palette; no cap change, production adapter or shipping swap.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location("dock128", Path(__file__).with_name("convert.py"))
M = importlib.util.module_from_spec(spec)
spec.loader.exec_module(M)
DEFAULT = M.DEFAULT / "native256"
PRIOR_PACK_SHA = "03df174447f9d8ac2562b31e1a4cf492065a78743d205f7f8ba20d0c7e1396d3"
SIZE = 256
TILES = (("nw", 0, 0), ("ne", 128, 0), ("sw", 0, 128), ("se", 128, 128))


def author(output=DEFAULT):
    prior_path = M.DEFAULT / "pack.json"
    M.AP.require(M.sha(prior_path) == PRIOR_PACK_SHA, "Frozen128 palette dependency changed")
    prior = M.AP.read_json(prior_path)
    palette = prior["palette"]
    M.AP.require(len(palette) == 64, "Expected approved64-color candidate palette")
    source_path = M.DEFAULT / "source.png"
    M.AP.require(M.sha(source_path) == M.SOURCE_SHA, "Reference changed")
    with M.Image.open(source_path) as opened:
        source = opened.convert("RGBA")
    sampled = M.sample(source, 256)
    raw = list(sampled.get_flattened_data())
    roles = [M.role(c) for c in raw]
    # Exact frozen strata:47metal,8green,8cyan. Repeated RGB samples are cached
    # during authoring only, no runtime/per-step allocations implied.
    ranges = (range(1, 48), range(1, 48), range(56, 64), range(48, 56))
    cache = {}
    indices = []
    for role, c in zip(roles, raw):
        if not c[3]:
            indices.append(0)
            continue
        key = (role, c[:3])
        if key not in cache:
            cache[key] = min(ranges[role], key=lambda ix: sum((a-b)**2 for a,b in zip(c[:3], M.AP.rgba(palette[ix])[:3])))
        indices.append(cache[key])
    lamps = [i for i, c in enumerate(raw) if c[3] and roles[i] == 3 and
             (i % SIZE - 128)**2 + (i // SIZE - 128)**2 > 36**2]
    frames = [indices.copy() for _ in range(3)]
    green = sorted({indices[i] for i in lamps})
    for state, gain in ((1, .78), (2, 1.35)):
        mapping = {}
        for index in green:
            target = [min(255, round(c*gain)) for c in M.AP.rgba(palette[index])[:3]]
            mapping[index] = min(green, key=lambda ix: sum((a-b)**2 for a,b in zip(target, M.AP.rgba(palette[ix])[:3])))
        for i in lamps:
            frames[state][i] = mapping[indices[i]]
    output.mkdir(parents=True, exist_ok=True)
    sprites, groups, tiles = [], [], []
    for name, x0, y0 in TILES:
        role_slice = [roles[(y0+y)*SIZE+x0+x] for y in range(128) for x in range(128)]
        tile = {"name": name, "width": 128, "height": 128, "position": [x0-64, y0-64], "roles": role_slice,
                "frame_sprites": [f"{name}-body-{n}" for n in range(3)],
                "parts": [{"name": p, "parent": 0 if k else None, "position": [0, 0],
                           "role": ("armor", "trim", "trim", "emitter")[k]} for k,p in enumerate(M.PARTS)]}
        tiles.append(tile)
        for state, frame in enumerate(frames):
            sub = [frame[(y0+y)*SIZE+x0+x] for y in range(128) for x in range(128)]
            im = M.image({"width": 128, "height": 128, "indices": sub}, palette)
            M.PNG.write_png(output / f"{name}-body-{state}.png", im)
            sprites.append({"name": f"{name}-body-{state}", "source": f"{name}-body-{state}.png", "topology": "holes-allowed"})
            for k, part in enumerate(M.PARTS):
                key = f"{name}-{part}-{state}"
                subpart = [v if role_slice[i] == k else 0 for i,v in enumerate(sub)]
                M.PNG.write_png(output / f"{key}.png", M.image({"width":128,"height":128,"indices":subpart}, palette))
                sprites.append({"name": key, "source": f"{key}.png", "topology": "holes-allowed"})
    for state in range(3):
        parts = []
        for name, x0, y0 in TILES:
            for k, part in enumerate(M.PARTS):
                parts.append({"name": f"{name}-{part}", "sprite": f"{name}-{part}-{state}",
                              **({"parent": f"{name}-deck"} if k else {}),
                              "transform": {"position": [0,0] if k else [(x0-64)*.5,(y0-64)*.5],
                                            "basis_x": [1,0] if k else [.5,0], "basis_y": [0,1] if k else [0,.5]}})
        groups.append({"name": f"pan256-{state}", "parts": parts})
    manifest = {"schema":1,"palette":{"mode":"provided","max_colors":64,"colors":palette},
                "sprites":sprites,"groups":groups,"animations":[{"name":"pan256-lamps","loop":True,
                "frames":[{"kind":"group","name":f"pan256-{n}","duration_ms":700} for n in range(3)]}],
                "budgets":{"sprite_cells":8192,"group_cells":32000,"group_transforms":16,"extent":1000},
                "provenance":{"source_sha256":M.SOURCE_SHA,"source_dimensions":[1254,1254],
                              "palette_dependency_sha256":PRIOR_PACK_SHA,"source_type":"explicit-lossy-sampling-same64palette-authoredlampstates",
                              "note":"Source-only tiled candidate; no engine cap change or shipping adapter"},
                "extensions":{"pixel_rig_tiles":{"source_size":[256,256],"reference_footprint":[128,128],
                   "parent":{"position":[0,0],"basis_x":[.5,0],"basis_y":[0,.5]},"tiles":tiles,"lamp_cells_source":lamps,
                   "reduced_motion":"Freeze complete state0 and rig time0; host policy, not automatic Game integration"}}}
    M.write_json(output / "manifest.json", manifest, True)
    pack = M.AP.pack(output / "manifest.json")
    M.write_json(output / "pack.json", pack, True)
    overview = M.Image.new("RGB", (3*544, 576), (5,10,17))
    draw = M.ImageDraw.Draw(overview)
    for n, frame in enumerate(frames):
        im = M.image({"width":SIZE,"height":SIZE,"indices":frame}, palette)
        M.PNG.write_png(output / f"full-state-{n}.png", im)
        enlarged = im.resize((512,512), M.Image.Resampling.NEAREST)
        overview.paste(enlarged,(n*544+16,44),enlarged)
        draw.text((n*544+16,12),f"256 source / state{n} / 28549 cells",fill=(174,216,205))
    M.PNG.write_png(output / "authoring-states.png",overview)
    metrics = M.sampling_metrics(source, sampled, M.image({"width":SIZE,"height":SIZE,"indices":frames[0]},palette))
    metrics.update({"palette_colors":64,"lamp_cells":len(lamps),"changed_lamp_cells":[sum(a!=b for a,b in zip(indices,f)) for f in frames],
                    "source_cell_scale":.5,"reference_footprint":[128,128],"group_cells":28549,"group_transforms":16,
                    "tile_counts":{name:sum(frame[(y0+y)*SIZE+x0+x]!=0 for y in range(128) for x in range(128)) for name,x0,y0 in TILES for frame in [frames[0]]},
                    "part_counts":{p:sum(ix!=0 and roles[i]==k for i,ix in enumerate(indices)) for k,p in enumerate(M.PARTS)}})
    M.write_json(output / "conversion-losses.json",metrics)
    return pack,metrics


if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output",type=Path,default=DEFAULT)
    args=parser.parse_args()
    _, result=author(args.output)
    print(json.dumps({k:result[k] for k in ("group_cells","group_transforms","tile_counts","part_counts","lamp_cells","changed_lamp_cells")},indent=2))
