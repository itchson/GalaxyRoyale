"""Semantic PAN surface sidecar and CPU art study; no renderer/ABI integration.

Painted albedo is never used as a height source. Exact roles/signals from the
reviewed96 atlas author a local height field; coarse48/fine96 ownership remains
the existing retained-body/overlay layout. Core owns actual Surface packing.
"""
from __future__ import annotations

import argparse
from collections import deque
from hashlib import sha256
import json
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
ATLAS = ROOT / "assets/pixels/raicho-fine-detail/atlas.json"
ROLES = ("metal", "recess", "glass", "main_accent", "reactor", "exhaust", "fastener")
SIZE = 96


def write_json(path: Path, value) -> None:
    path.write_bytes((json.dumps(value, allow_nan=False, separators=(",", ":")) + "\n").encode("utf-8"))


def unit(v):
    length = math.sqrt(sum(x*x for x in v))
    if not math.isfinite(length) or length <= 0:
        raise ValueError("finite nonzero normal required")
    return tuple(x/length for x in v)


def roles_for(sprite):
    if (sprite["width"], sprite["height"], len(sprite["indices"])) != (SIZE, SIZE, SIZE*SIZE):
        raise ValueError("exact96 source grid required")
    roles = [None] * (SIZE*SIZE)
    if set(sprite["material_cells"]) != set(ROLES):
        raise ValueError("exact seven roles required")
    for role, indices in sprite["material_cells"].items():
        if indices != sorted(set(indices)):
            raise ValueError("ordered unique role positions required")
        for i in indices:
            if not 0 <= i < SIZE*SIZE or not sprite["indices"][i] or roles[i] is not None:
                raise ValueError("roles must partition actual occupied cells")
            roles[i] = role
    if any(bool(p) != (r is not None) for p, r in zip(sprite["indices"], roles, strict=True)):
        raise ValueError("complete actual occupancy partition required")
    return roles


def distance_inside(mask, width):
    """Four-neighbor bevel distance, computed once during offline authoring."""
    height = len(mask)//width
    distances = [0]*len(mask)
    queue = deque()
    def neighbors(i):
        x, y = i%width, i//width
        return [ny*width+nx for nx, ny in ((x-1,y),(x+1,y),(x,y-1),(x,y+1)) if 0 <= nx < width and 0 <= ny < height]
    for i, occupied in enumerate(mask):
        if occupied and (len(neighbors(i)) < 4 or any(not mask[j] for j in neighbors(i))):
            distances[i] = 1
            queue.append(i)
    while queue:
        i = queue.popleft()
        for j in neighbors(i):
            if mask[j] and not distances[j]:
                distances[j] = distances[i]+1
                queue.append(j)
    return distances


def author_height(roles):
    distances = {r: distance_inside([v == r for v in roles], SIZE) for r in ROLES}
    glass = [i for i, r in enumerate(roles) if r == "glass"]
    gx = (min(i%SIZE for i in glass)+max(i%SIZE for i in glass))*.5
    gy = (min(i//SIZE for i in glass)+max(i//SIZE for i in glass))*.5
    rx = (max(i%SIZE for i in glass)-min(i%SIZE for i in glass)+2)*.5
    ry = (max(i//SIZE for i in glass)-min(i//SIZE for i in glass)+2)*.5
    heights = []
    for i, role in enumerate(roles):
        if role is None:
            heights.append(None)
            continue
        bevel = min(distances[role][i], 3)/3
        if role == "glass":
            dome = max(0.0, 1-((i%SIZE-gx)/rx)**2-((i//SIZE-gy)/ry)**2)
            height = 1.15+.65*math.sqrt(dome)
        else:
            base, rise = {"metal":(.58,.42), "recess":(.18,.06), "main_accent":(.63,.42),
                          "reactor":(.42,.12), "exhaust":(.26,.09), "fastener":(1.05,.18)}[role]
            height = base+rise*bevel
        heights.append(height)
    return heights


def normals(heights, width, spacing):
    height = len(heights)//width
    result = []
    for i, value in enumerate(heights):
        if value is None:
            result.append(None)
            continue
        x, y = i%width, i//width
        def sample(nx, ny):
            if not (0 <= nx < width and 0 <= ny < height):
                return value
            other = heights[ny*width+nx]
            return value if other is None else other
        dx = (sample(x+1,y)-sample(x-1,y))/(2*spacing)
        dy = (sample(x,y+1)-sample(x,y-1))/(2*spacing)
        result.append(unit((-dx,-dy,1.0)))
    return result


def author(atlas):
    full = atlas["sprites"]["pan_hikari"]
    fine = atlas["sprites"]["pan_hikari_fine"]
    roles = roles_for(full)
    fine_roles = roles_for(fine)
    occupied = [i for i, p in enumerate(full["indices"]) if p]
    fine_positions = [i for i, p in enumerate(fine["indices"]) if p]
    if fine_positions != full["fine_cells"] or full["signal_cells"] != fine["signal_cells"]:
        raise ValueError("retained exact fine/signal masks required")
    if any(full["indices"][i] != fine["indices"][i] or roles[i] != fine_roles[i] for i in fine_positions):
        raise ValueError("fine pixels must retain actual full colors/roles")
    heights = author_height(roles)
    normal = normals(heights, SIZE, .5)
    signal = set(full["signal_cells"])
    if not signal <= set(fine_positions):
        raise ValueError("emission must belong to actual fine signal pixels")
    # Linear multiplier on the renderer's bounded object emission, not an
    # absolute light intensity. Only existing explicit signals retain it.
    emission = [255 if i in signal else 0 for i in range(SIZE*SIZE)]
    cells = [None if n is None else [*(round(v,9) for v in n), emission[i]] for i,n in enumerate(normal)]
    coarse_height = []
    for cy in range(48):
        for cx in range(48):
            samples = [heights[(cy*2+dy)*SIZE+cx*2+dx] for dy in (0,1) for dx in (0,1)]
            if any(v is not None for v in samples) and any(v is None for v in samples):
                raise ValueError("PAN source must exactly cover every retained coarse block")
            coarse_height.append(None if samples[0] is None else sum(samples)/4)
    coarse_normal = normals(coarse_height, 48, 1.0)
    coarse_cells = [None if n is None else [*(round(v,9) for v in n),0] for n in coarse_normal]
    fine_set = set(fine_positions)
    sidecar = {"version":1,"asset":"pan_hikari","source_atlas_sha256":sha256(ATLAS.read_bytes()).hexdigest(),
        "status":"authored semantic proposal; no native Surface packing or renderer integration",
        "axes":"local +X right, +Y down, +Z viewer; height in coarse actor-local units",
        "height_source":"explicit disjoint material roles; no albedo luminance input",
        "coarse48":{"width":48,"height":48,"cells":coarse_cells},
        "fine96":{"width":96,"height":96,"cells":[c if i in fine_set else None for i,c in enumerate(cells)]},
        "full96_authoring_only":{"width":96,"height":96,"cells":cells},
        "mask_policy":"None -> missing Surface legacy0; actual PixelBody occupancy alone admits cells; map fine[x,y] to live coarse[x//2,y//2]",
        "emission_policy":"Linear object-emission multiplier:0 blocks hull self-illumination;255 retains object emission on31existing fine signals; no pixel point lights; coarse0 avoids broad duplicate glow; dynamic throttle/actor state belongs to runtime",
        "packing":"engine_core::Surface::new owns normalization/oct8/packed slot bits; this authorer emits no manual payload"}
    return sidecar, heights, roles


def validate(sidecar, atlas):
    full = atlas["sprites"]["pan_hikari"]
    fine = atlas["sprites"]["pan_hikari_fine"]
    for name, width, expected in (("full96_authoring_only",96,2648),("fine96",96,1205),("coarse48",48,662)):
        grid = sidecar[name]
        if (grid["width"],grid["height"],len(grid["cells"])) != (width,width,width*width):
            raise ValueError("exact body-index array dimensions required")
        if sum(c is not None for c in grid["cells"]) != expected:
            raise ValueError("unchanged mask occupancy required")
        for c in grid["cells"]:
            if c is not None and (len(c)!=4 or not all(math.isfinite(v) for v in c[:3]) or abs(sum(v*v for v in c[:3])-1)>1e-7 or c[2]<=0 or not isinstance(c[3],int) or not 0<=c[3]<=255):
                raise ValueError("finite unit normal and exact emissive byte required")
    if any(bool(i) != (c is not None) for i,c in zip(full["indices"],sidecar["full96_authoring_only"]["cells"],strict=True)):
        raise ValueError("full material mask changed source occupancy")
    if any(bool(i) != (c is not None) for i,c in zip(fine["indices"],sidecar["fine96"]["cells"],strict=True)):
        raise ValueError("fine material mask changed source occupancy")
    emissive = {i for i,c in enumerate(sidecar["fine96"]["cells"]) if c is not None and c[3]}
    if emissive != set(full["signal_cells"]) or any(c is not None and c[3] for c in sidecar["coarse48"]["cells"]):
        raise ValueError("sparse fine signal emission only")


def art_images(output, atlas, sidecar, heights):
    cells = sidecar["full96_authoring_only"]["cells"]
    full = atlas["sprites"]["pan_hikari"]
    palette = [tuple(c.to_bytes(4,"little")) for c in atlas["palette"]]
    images = {name:Image.new("RGBA",(96,96),(0,0,0,0)) for name in ("source","height","normal","emissive","flat-left","normal-left","normal-right","normal-emission","damage-fixture")}
    source = [palette[p] for p in full["indices"]]
    images["source"].putdata(source)
    removed_coarse = (23,16)  # An authoring fixture, not a claimed native projectile hit.
    for i,c in enumerate(cells):
        if c is None:
            continue
        x,y = i%96,i//96
        images["height"].putpixel((x,y),(int(round(heights[i]/2*255)),)*3+(255,))
        images["normal"].putpixel((x,y),tuple(int(round((n*.5+.5)*255)) for n in c[:3])+(255,))
        images["emissive"].putpixel((x,y),(c[3],)*3+(255,))
        for mode in ("flat-left","normal-left","normal-right","normal-emission","damage-fixture"):
            if mode=="damage-fixture" and (x//2,y//2)==removed_coarse:
                continue
            right = mode=="normal-right"
            position = ((x+.5-48)*.5,(y+.5-48)*.5)
            delta = ((18 if right else -18)-position[0],-10-position[1],24)
            direction = unit(delta)
            normal = (0,0,1) if mode=="flat-left" else c[:3]
            diffuse = max(0,sum(n*l for n,l in zip(normal,direction,strict=True)))
            attenuation = max(0,1-(delta[0]**2+delta[1]**2)/72**2)**2
            color = (1,.34,.19) if right else (.26,.72,1)
            base = [(v/255)**2.2 for v in source[i][:3]]
            # A declared .2 object-emission study, not ACTOR_MATERIAL's current0.
            e = .2*c[3]/255 if mode in ("normal-emission","damage-fixture") else 0
            lit = [min(1,b*(1-.12*.75+.3*e)+l*(b*.8+.012)*attenuation*.75*diffuse) for b,l in zip(base,color,strict=True)]
            rgb = tuple(round(v**(1/2.2)*255) for v in lit)
            images[mode].putpixel((x,y),rgb+(255,))
    for name,image in images.items():
        image.save(output/f"pan-{name}.png")
    contact=Image.new("RGB",(864,966),(3,5,9))
    draw=ImageDraw.Draw(contact)
    font=ImageFont.truetype("C:/Windows/Fonts/consola.ttf",14)
    draw.text((10,7),"PAN SEMANTIC MATERIAL STUDY / AUTHORING CPU / NOT GAME GPU",font=font,fill=(206,222,229))
    for number,(name,image) in enumerate(images.items()):
        x,y=(number%3)*288,30+(number//3)*312
        draw.text((x+8,y),name,font=font,fill=(162,195,210))
        review=image.resize((288,288),Image.Resampling.NEAREST)
        contact.paste(review,(x,y+23),review)
    contact.save(output/"authoring-contact.png")


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args()
    if not args.output.is_absolute():
        parser.error("explicit absolute owned proof output required")
    args.output.mkdir(parents=True,exist_ok=True)
    atlas=json.loads(ATLAS.read_text(encoding="utf-8"))
    sidecar,heights,roles=author(atlas)
    validate(sidecar,atlas)
    write_json(args.output/"pan-material-proposal.json",sidecar)
    art_images(args.output,atlas,sidecar,heights)
    metrics={"scope":"semantic role authoring and CPU light study; no native renderer/packing execution",
        "coarse_cells":662,"fine_cells":1205,"full_authoring_cells":2648,"fine_emissive_cells":31,
        "distinct_full_normals":len({tuple(c[:3]) for c in sidecar["full96_authoring_only"]["cells"] if c is not None}),
        "normal_xy_max":max(math.hypot(c[0],c[1]) for c in sidecar["full96_authoring_only"]["cells"] if c is not None),
        "proposed_coarse_fine_Surface4_payload_bytes":(48*48+96*96)*4,
        "proposed_runtime_extra_quads":0,"packing_owned_by":"engine_core::Surface::new; not run here",
        "source_atlas_sha256":sha256(ATLAS.read_bytes()).hexdigest(),"script_lf_sha256":sha256(Path(__file__).read_bytes().replace(b"\r\n",b"\n")).hexdigest()}
    write_json(args.output/"metrics.json",metrics)
    print(json.dumps(metrics))


if __name__=="__main__":
    main()
