"""Original nine freeform arena/league emblems; no AI/imported imagery.
24 local cells fit the current 24-logical-pixel campaign icon at integer scale.
"""
from pathlib import Path
import hashlib
import json
import math
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parent
SIZE=24
ROSTER=[
    ("Lyra Voss","Dawnneedle","Stutter dash / needle burst",(255,195,88),"dash-needle"),
    ("Korr-9","Bulwark","Rotating broadside",(255,104,102),"broadside-shield"),
    ("Sable Myrr","Nightrail","Serpent spiral / guided missiles",(188,123,255),"forked-serpent"),
    ("Xil of Aurora","Prism","Offset fork volleys",(104,230,255),"split-crystal"),
    ("Maera Quill","Windrunner","Charge / damaging ion wake",(113,244,153),"swept-wing-wake"),
    ("Orun Vale","Ironwake","Telegraphed ram / impact nova",(255,157,88),"ram-anvil"),
    ("Vexa Null","Voidcoil","Circling helix / piercing seekers",(239,116,228),"void-helix"),
    ("Atlas-0","Sunspear","Sniper burst / piercing spear",(255,233,158),"solar-spear"),
    ("Galaxy Royale","Royale Sovereign","League victory / Sovereign reward",(247,205,100),"crown-star-laurel"),
]
COMMON=[(0,0,0,0),(9,17,32,255),(39,50,72,255),(100,124,148,255),(209,227,235,255),(250,247,224,255)]
palette=COMMON[:]
def rgba_index(rgb):
    color=(*rgb,255)
    if color not in palette:palette.append(color)
    return palette.index(color)
def inside(x,y,points):
    hit=False
    for a,b in zip(points,points[1:]+points[:1]):
        if (a[1]>y)!=(b[1]>y) and x<(b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0]:hit=not hit
    return hit
def polygon(mask,points):
    for y in range(24):
        for x in range(24):
            if inside(x+.5,y+.5,points):mask.add((x,y))
def stroke(mask,points,width=1):
    for a,b in zip(points,points[1:]):
        dx,dy=b[0]-a[0],b[1]-a[1]
        for y in range(24):
            for x in range(24):
                t=max(0,min(1,((x-a[0])*dx+(y-a[1])*dy)/(dx*dx+dy*dy)))
                if math.hypot(x-a[0]-t*dx,y-a[1]-t*dy)<=width*.5:mask.add((x,y))
def disc(mask,cx,cy,radius):
    for y in range(24):
        for x in range(24):
            if math.hypot(x-cx,y-cy)<=radius:mask.add((x,y))
def shapes(style):
    mask=set();details={}
    if style==0:
        polygon(mask,[(12,1),(15,12),(19,17),(14,16),(12,22),(10,16),(7,17),(10,12)])
        for a,b in [((3,9),(7,9)),((2,13),(6,13)),((3,17),(5,17))]:stroke(mask,[a,b],1)
        for y in range(5,15):details[(11,y)]=5
    elif style==1:
        polygon(mask,[(7,3),(17,3),(19,8),(17,17),(12,21),(6,17),(4,8)])
        for y in [7,11,15]:
            polygon(mask,[(2,y-1),(6,y-1),(6,y+1),(2,y+1)])
            polygon(mask,[(18,y-1),(22,y-1),(22,y+1),(18,y+1)])
        for x,y in [(9,7),(10,7),(14,7),(15,7),(9,8),(14,8),(12,12),(12,13)]:details[(x,y)]=2
        stroke(mask,[(12,8),(12,16)],2)
    elif style==2:
        stroke(mask,[(18,3),(13,3),(7,5),(5,8),(8,11),(16,12),(19,15),(17,19),(11,21),(5,19)],3)
        polygon(mask,[(17,2),(22,2),(20,5),(18,7),(15,6)])
        for x,y in [(19,3),(20,3),(12,11),(13,11)]:details[(x,y)]=5
        for x,y in [(4,17),(3,18),(2,19)]:mask.add((x,y))
        details[(18,4)]=1
    elif style==3:
        polygon(mask,[(7,2),(11,7),(8,14),(3,10)])
        polygon(mask,[(17,3),(21,9),(17,14),(13,9)])
        polygon(mask,[(12,9),(17,16),(12,22),(8,16)])
        for x,y in [(7,5),(7,6),(7,7),(17,6),(17,7),(12,13),(12,14),(12,15),(12,16)]:details[(x,y)]=5
    elif style==4:
        polygon(mask,[(2,8),(9,11),(13,5),(21,2),(17,10),(12,16),(6,15)])
        polygon(mask,[(5,7),(10,7),(10,10),(7,10)])
        for a,b in [((3,19),(9,17)),((9,21),(13,18)),((17,17),(20,14))]:stroke(mask,[a,b],1.3)
        for x,y in [(10,12),(11,11),(12,10),(13,9),(14,8),(15,7),(16,6)]:details[(x,y)]=5
    elif style==5:
        polygon(mask,[(4,3),(20,3),(22,8),(18,12),(15,10),(14,16),(19,18),(19,20),(5,20),(5,18),(10,16),(9,10),(6,12),(2,8)])
        for x,y in [(7,6),(8,6),(9,6),(14,6),(15,6),(16,6),(12,12),(12,13),(12,14)]:details[(x,y)]=4
        for a,b in [((3,15),(1,16)),((21,15),(23,16)),((11,22),(13,22))]:stroke(mask,[a,b],1)
    elif style==6:
        stroke(mask,[(6,3),(12,2),(18,5),(18,8),(12,12),(6,16),(6,19),(12,22),(18,21)],2.5)
        stroke(mask,[(18,3),(12,5),(6,8),(6,11),(12,12),(18,14),(18,17),(12,19),(6,21)],2.5)
        for x,y in [(6,3),(18,3),(12,12),(12,13),(6,21),(18,21)]:details[(x,y)]=5
        details[(11,12)]=1;details[(13,12)]=1
    elif style==7:
        disc(mask,11.5,8,4.4)
        for a,b in [((12,1),(12,4)),((5,3),(7,5)),((3,8),(7,8)),((5,13),(8,11)),
            ((19,3),(17,5)),((21,8),(17,8)),((19,13),(16,11))]:stroke(mask,[a,b],1.2)
        polygon(mask,[(10,10),(13,10),(13,18),(12,22),(10,18)])
        for y in range(5,19):details[(11,y)]=5
        for x,y in [(9,7),(10,6),(13,7),(13,8)]:details[(x,y)]=4
    else:
        polygon(mask,[(6,3),(10,7),(12,2),(14,7),(18,3),(17,11),(7,11)])
        polygon(mask,[(12,10),(14,14),(19,14),(15,17),(16,22),(12,19),(8,22),(9,17),(5,14),(10,14)])
        stroke(mask,[(4,6),(2,10),(3,16),(6,20)],1.3)
        stroke(mask,[(20,6),(22,10),(21,16),(18,20)],1.3)
        for x,y in [(3,8),(2,12),(4,17),(21,8),(22,12),(20,17)]:mask.add((x,y))
        details[(9,8)]=rgba_index(ROSTER[3][3]);details[(12,7)]=5;details[(15,8)]=rgba_index(ROSTER[6][3])
        for x,y in [(11,15),(12,15),(12,16),(13,16)]:details[(x,y)]=5
    return {(x,y) for x,y in mask if 0<=x<24 and 0<=y<24},details

sprites={};provenance=[]
for style,(champion,ship,ability,theme,silhouette) in enumerate(ROSTER):
    primary=rgba_index(theme)
    shadow=rgba_index(tuple(round(v*.42) for v in theme))
    highlight=rgba_index(tuple(round(v+(255-v)*.52) for v in theme))
    mask,details=shapes(style)
    border={(x+dx,y+dy) for x,y in mask for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]}
    border={p for p in border if p not in mask and 0<=p[0]<24 and 0<=p[1]<24}
    indices=[0]*576;shimmer=[]
    for x,y in border:indices[y*24+x]=1
    for x,y in mask:
        edge=any((x+dx,y+dy) not in mask for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)])
        color=highlight if (x,y-1) not in mask and y<16 else shadow if edge and x+y>24 else primary
        indices[y*24+x]=color
    for (x,y),color in details.items():
        if (x,y) in mask:indices[y*24+x]=color
    # Small selected beveled/engraved cells only; no geometry or external glints.
    candidates=sorted(((x,y) for x,y in mask if indices[y*24+x] in [highlight,4,5]),key=lambda p:(p[0]+p[1],p[1],p[0]))
    shimmer=[y*24+x for x,y in candidates[::2]][:24]
    name=f"campaign_patch_{style}"
    sprites[name]={"width":24,"height":24,"indices":indices,"shimmer_cells":shimmer}
    provenance.append({"index":style,"champion":champion,"ship":ship,"ability":ability,
        "silhouette":silhouette,"occupied":sum(v!=0 for v in indices),"shimmer_cells":len(shimmer)})
assert len(palette)<=64
defined=len(palette)-1
palette += [palette[1]]*(64-len(palette))
atlas={"version":1,"palette":[int.from_bytes(bytes(c),"little") for c in palette],"sprites":sprites}
path=ROOT/"champion-patches-atlas.json"
path.write_text(json.dumps(atlas,separators=(",",":"))+"\n",encoding="utf-8")
preview=Image.new("RGB",(840,636),(9,13,32));draw=ImageDraw.Draw(preview)
draw.text((16,12),"ORIGINAL ARENA PATCH EMBLEMS / 24 LOCAL CELLS / WORK ONLY",fill=(250,247,224))
for index,entry in enumerate(provenance):
    left=16+index%3*276;top=40+index//3*196
    sprite=sprites[f"campaign_patch_{index}"]
    image=Image.new("RGBA",(24,24));image.putdata([palette[i] for i in sprite["indices"]])
    enlarged=image.resize((144,144),Image.Resampling.NEAREST)
    preview.paste(enlarged,(left,top),enlarged)
    preview.paste(image,(left+169,top+55),image)
    draw.text((left,top+150),f"{entry['index']}: {entry['ship']}",fill=ROSTER[index][3])
    draw.text((left,top+166),entry["silhouette"],fill=(100,124,148))
draw.text((16,620),"NO REPEATED OUTER FRAME / PACKED QUADS / ORIGINAL GEOMETRY + PALETTE / NO AI ASSETS",fill=(100,124,148))
preview.save(ROOT/"authored-contact-sheet.png")
used={i for sprite in sprites.values() for i in sprite["indices"] if i}
result={"method":"original authored polygon/stroke/symbol cell masks and themes; no AI/imported imagery",
    "runtime":"cached local occupied cells; inspection PNG never loaded", "size":[24,24],
    "defined_opaque_colors":defined,"used_opaque_colors":len(used),"emblems":provenance,
    "atlas_sha256":hashlib.sha256(path.read_bytes()).hexdigest()}
(ROOT/"champion-patches-provenance.json").write_text(json.dumps(result,indent=2)+"\n",encoding="utf-8")
print(json.dumps(result,indent=2))
