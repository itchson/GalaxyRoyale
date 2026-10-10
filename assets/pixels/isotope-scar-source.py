"""Original authored Isotope Scar cells. PNGs are inspection artifacts only.

Hand-authored crater/fault/polygon placement plus deterministic geometric shading;
no image/model generation, imported textures, samples or network assets.
"""
from pathlib import Path
import hashlib
import json
import math
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent
COLORS = [
    (0, 0, 0, 0),
    (12, 15, 33, 255), (23, 20, 45, 255), (34, 27, 57, 255),
    (47, 33, 71, 255), (63, 41, 86, 255), (80, 53, 100, 255),
    (99, 68, 115, 255), (119, 84, 133, 255), (142, 108, 148, 255),
    (158, 126, 160, 255), (46, 60, 47, 255), (68, 85, 48, 255),
    (103, 124, 57, 255), (146, 163, 72, 255), (189, 201, 96, 255),
    (216, 224, 128, 255), (49, 48, 76, 255), (63, 74, 88, 255),
    (79, 103, 111, 255), (105, 132, 135, 255),
]
SIZE = 96
CRATERS = [(-21, -15, 7.4, .82), (-8, -25, 4.8, .65), (9, -19, 6.1, .82),
    (-28, 1, 4.8, .75), (-12, 3, 9.0, .78), (9, 7, 5.5, .8),
    (-11, 26, 5.2, .67), (20, 22, 7.3, .71), (27, -2, 4.2, .9),
    (-24, 18, 2.8, .8), (2, 31, 2.6, .6), (5, -33, 2.3, .7),
    (-32, -10, 2.2, .8), (16, -6, 2.6, .85), (-1, -8, 3.2, .65)]
FAULTS = [[(-31,-32),(-21,-25),(-19,-17),(-9,-10),(-6,-1),(3,3),(5,13),(15,18),(18,30),(31,34)],
    [(-19,-17),(-11,-22),(-3,-23),(5,-29)],
    [(3,3),(11,-1),(17,-8),(26,-11),(35,-21)],
    [(-6,-1),(-16,7),(-20,13),(-33,18)],
    [(15,18),(7,23),(4,35)]]
def segment_distance(x,y,a,b):
    dx,dy=b[0]-a[0],b[1]-a[1]
    t=max(0,min(1,((x-a[0])*dx+(y-a[1])*dy)/(dx*dx+dy*dy)))
    return math.hypot(x-a[0]-t*dx,y-a[1]-t*dy)
def fault_distance(x,y):
    return min(segment_distance(x,y,a,b) for line in FAULTS for a,b in zip(line,line[1:]))

surface=[]
for y in range(SIZE):
    for x in range(SIZE):
        px,py=x+.5-48,y+.5-48
        radius=math.hypot(px,py)
        # A bitten, fractured limb and a pronounced dark terminator distinguish
        # the scar from the existing whole-disc planets and simple rings.
        edge=37.6 + .65*math.sin(math.atan2(py,px)*13) + .35*math.sin(px*.7)
        bite=((px-32)/10)**2+((py+19)/11)**2 < 1
        if radius>edge or bite:
            surface.append(0); continue
        z=math.sqrt(max(0,37.6**2-radius**2))
        illumination=max(0,min(1,(-.66*px-.4*py+.59*z)/37.6))
        shade=1+min(9,int(illumination*9.5))
        plates=math.sin(px*.19+py*.12)+.5*math.cos(py*.26-px*.11)
        if plates<-.9: shade=max(1,shade-1)
        for cx,cy,r,flatten in CRATERS:
            dx,dy=px-cx,(py-cy)/flatten
            d=math.hypot(dx,dy)
            if d<r-1.1:
                shade=max(1,shade-(3 if dx+dy<0 else 1))
            elif abs(d-r)<.8:
                shade=min(10,shade+2) if dx+dy<0 else max(1,shade-2)
        seam=fault_distance(px,py)
        if seam<2.1: shade=max(1,shade-3)
        if seam<.7 and illumination>.11:
            shade=12+min(4,int(illumination*4.5))
        elif 1.4<seam<2.05 and illumination>.2 and py+px*.4<25:
            shade=11+min(3,int(illumination*3.4))
        # Sparse mineral strata have coherent 3–8 cell shapes rather than noise.
        if ((px+24)/6)**2+((py-11)/2.2)**2<1 and illumination>.3:
            shade=12 if py>11 else 14
        if ((px-3)/3.1)**2+((py+13)/1.5)**2<1 and illumination>.4:
            shade=13 if px>3 else 15
        surface.append(shade)

def wisp(width,height,front):
    indices=[]
    for y in range(height):
        for x in range(width):
            px,py=x+.5-width/2,y+.5-height/2
            center=math.sin(px*.077+.8)*6 + px*.065
            band=abs(py-center)
            taper=max(0,1-abs(px)/(width*.45))
            value=0
            if band<1.0+taper*.65 and abs(px)<width*.45:
                # Gaps keep the plume airy even at its smallest projected scale.
                if (x//6+y//3)%5!=0:
                    value=(18 if front else 5) if band>.8 else (19 if front else 7)
            elif band<3 and taper>.2 and (x+2*y)%11==0:
                value=17 if front else 4
            indices.append(value)
    return indices

def low_surface():
    # A separately authored-resolution representation of the same lithosphere.
    # Favor mineral seam and lit-rim cells when reducing 2×2 geometry.
    indices=[]
    for y in range(48):
        for x in range(48):
            block=[surface[(y*2+dy)*96+x*2+dx] for dy in range(2) for dx in range(2)]
            occupied=[v for v in block if v]
            if len(occupied)<2: indices.append(0); continue
            minerals=[v for v in occupied if 11<=v<=16]
            indices.append(max(minerals) if minerals else max(occupied))
    return indices

fragment=[]
for y in range(24):
    for x in range(24):
        px,py=x+.5-12,y+.5-12
        inside=abs(px)*.72+abs(py)*1.1<7.7 and py>-.7*px-6.3
        color=0
        if inside:
            color=6 if px+py<0 else 3
            if abs(px*.7+py)<.7: color=13
            if py<-.8*px-3.5: color=9
        fragment.append(color)
sprites={
    "scar_surface":{"width":96,"height":96,"indices":surface},
    "scar_surface_low":{"width":48,"height":48,"indices":low_surface()},
    "scar_ion_back":{"width":112,"height":48,"indices":wisp(112,48,False)},
    "scar_ion_front":{"width":96,"height":40,"indices":wisp(96,40,True)},
    "scar_fragment":{"width":24,"height":24,"indices":fragment},
}
palette=[int.from_bytes(bytes(c),"little") for c in COLORS]
palette += [palette[1]]*(64-len(palette))
atlas={"version":1,"palette":palette,"sprites":sprites}
atlas_path=ROOT/"isotope-scar-atlas.json"
atlas_path.write_text(json.dumps(atlas,separators=(",",":"))+"\n",encoding="utf-8")

preview=Image.new("RGB",(904,498),(9,13,32))
draw=ImageDraw.Draw(preview)
draw.text((18,14),"ISOTOPE SCAR / ORIGINAL AUTHORED CELL FIELDS / WORK ONLY",fill=COLORS[15])
locations={"scar_surface":(18,48,4),"scar_surface_low":(418,48,4),
    "scar_ion_back":(418,288,3),"scar_ion_front":(418,354,3),"scar_fragment":(786,74,3)}
for name,(left,top,scale) in locations.items():
    sprite=sprites[name]
    image=Image.new("RGBA",(sprite["width"],sprite["height"]))
    image.putdata([COLORS[i] for i in sprite["indices"]])
    image=image.resize((image.width*scale,image.height*scale),Image.Resampling.NEAREST)
    preview.paste(image,(left,top),image)
    draw.text((left,top+image.height+8),name,fill=COLORS[9])
draw.text((18,472),"NO BITMAP RUNTIME / PACKED RGBA + OCCUPIED CELLS / NO AI OR NETWORK ASSETS",fill=COLORS[9])
preview.save(ROOT/"authored-cells-preview.png")
counts={name:sum(v!=0 for v in sprite["indices"]) for name,sprite in sprites.items()}
used=sorted({v for sprite in sprites.values() for v in sprite["indices"] if v})
provenance={"method":"original authored crater/fault/fragment/plume geometry and palette; no AI/model/imported textures",
    "runtime":"packed palette indices expanded once into occupied PixelBodies; preview PNG never loaded",
    "defined_opaque_colors":len(COLORS)-1,"used_opaque_colors":len(used),"used_indices":used,
    "occupied_cells":counts,"atlas_sha256":hashlib.sha256(atlas_path.read_bytes()).hexdigest(),
    "animation":"cosmetic only: slow ion drift and restrained mineral-color shimmer; reduced motion freezes it"}
(ROOT/"isotope-scar-provenance.json").write_text(json.dumps(provenance,indent=2)+"\n",encoding="utf-8")
print(json.dumps(provenance,indent=2))
