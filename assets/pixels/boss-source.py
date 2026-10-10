"""Original Royale boss hulls and rigid pixel-role layouts; stdlib authoring.

Every visible cell belongs to one destructible hull and exactly one subobject.
Generation silhouettes are individually authored, not rescaled base artwork.
"""
import json
import math
from pathlib import Path

ROOT = Path(__file__).parent
FAMILIES = ["dart", "bastion", "coil"]
NAMES = [
    ["Matriarch", "Harrier", "Reaver", "Tempest", "Imperatrix"],
    ["Prime", "Sentinel", "Gatekeeper", "Citadel", "Oblivion"],
    ["Sovereign", "Sidewinder", "Ouroboros", "Leviathan", "Worldeater"],
]
PALETTE = [0] + [r | g << 8 | b << 16 | 255 << 24 for r, g, b in [
    (7, 12, 25), (20, 32, 49), (39, 56, 76), (75, 100, 126),
    (123, 157, 181), (185, 210, 220), (242, 248, 238),
    (73, 38, 71), (136, 61, 122), (210, 88, 178), (255, 163, 218),
    (87, 58, 33), (145, 94, 40), (220, 159, 65), (255, 220, 139),
    (28, 65, 62), (42, 117, 101), (85, 188, 145), (175, 245, 188),
    (28, 111, 141), (79, 217, 232), (188, 247, 250),
    (198, 96, 61), (252, 168, 79), (255, 238, 189),
]]
PALETTE += [PALETTE[1]] * (64 - len(PALETTE))
INK, DEEP, DARK, METAL, LIGHT, SILVER, WHITE = range(1, 8)

# Coordinates describe a 64-unit design canvas. Each generation has its own
# outline, internal cutouts and hardware plan before its native cell sampling.
HULLS = [
    [
        [(32,2),(40,18),(58,12),(56,36),(44,47),(40,59),(24,59),(20,47),(8,36),(6,12),(24,18)],
        [(32,1),(37,24),(55,5),(60,10),(50,34),(57,52),(51,59),(37,48),(32,62),(27,48),(13,59),(7,52),(14,34),(4,10),(9,5),(27,24)],
        [(14,3),(23,13),(30,8),(32,1),(34,8),(41,13),(50,3),(59,10),(50,29),(59,45),(53,53),(42,47),(37,60),(27,60),(22,47),(11,53),(5,45),(14,29),(5,10)],
        [(32,1),(38,17),(54,4),(62,12),(47,30),(62,44),(59,56),(40,47),(32,62),(24,47),(5,56),(2,44),(17,30),(2,12),(10,4),(26,17)],
        [(3,3),(18,16),(22,4),(29,16),(32,0),(35,16),(42,4),(46,16),(61,3),(59,37),(52,45),(58,56),(47,62),(36,54),(32,64),(28,54),(17,62),(6,56),(12,45),(5,37)],
    ],
    [
        [(16,4),(48,4),(60,16),(59,47),(51,55),(40,52),(40,61),(24,61),(24,52),(13,55),(5,47),(4,16)],
        [(25,2),(39,2),(42,13),(58,13),(62,19),(62,42),(51,47),(46,60),(18,60),(13,47),(2,42),(2,19),(6,13),(22,13)],
        [(8,4),(56,4),(62,10),(62,51),(53,60),(43,55),(40,62),(24,62),(21,55),(11,60),(2,51),(2,10)],
        [(9,2),(24,2),(24,10),(40,10),(40,2),(55,2),(62,9),(62,24),(54,24),(54,40),(62,40),(62,55),(55,62),(40,62),(40,54),(24,54),(24,62),(9,62),(2,55),(2,40),(10,40),(10,24),(2,24),(2,9)],
        [(8,1),(23,1),(28,10),(36,10),(41,1),(56,1),(62,10),(59,20),(64,24),(64,42),(58,46),(59,57),(50,64),(40,60),(36,64),(28,64),(24,60),(14,64),(5,57),(6,46),(0,42),(0,24),(5,20),(2,10)],
    ],
    [
        [(11,3),(23,15),(32,10),(41,15),(53,3),(60,9),(53,28),(55,39),(45,50),(39,60),(25,60),(19,50),(9,39),(11,28),(4,9)],
        [(10,1),(23,15),(30,8),(44,11),(55,5),(62,14),(51,31),(56,44),(42,56),(39,62),(24,59),(17,49),(7,42),(12,26),(2,13)],
        [],
        [(3,8),(19,15),(25,4),(32,10),(39,4),(45,15),(61,8),(62,22),(55,28),(59,39),(50,47),(43,44),(39,59),(32,64),(25,59),(21,44),(14,47),(5,39),(9,28),(2,22)],
        [(5,1),(18,15),(22,1),(29,15),(32,4),(35,15),(42,1),(46,15),(59,1),(62,14),(56,26),(64,36),(57,51),(49,47),(42,61),(32,56),(22,61),(15,47),(7,51),(0,36),(8,26),(2,14)],
    ],
]

def inside(x, y, vertices):
    result = False
    for a, b in zip(vertices, vertices[1:] + vertices[:1]):
        if (a[1] > y) != (b[1] > y) and x < (b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0]: result = not result
    return result

class Hull:
    def __init__(self, family, generation, size):
        self.family, self.generation, self.size = family, generation, size
        self.factor = size / 64
        self.accent = 10 + family * 4
        self.shadow, self.pale = self.accent - 2, self.accent + 1
        self.cells, self.roles = [0] * (size*size), [0] * (size*size)
        for y in range(size):
            for x in range(size):
                nx, ny = (x+.5)/self.factor, (y+.5)/self.factor
                if family == 2 and generation == 3:
                    radius = math.hypot((nx-32)*.94, ny-31)
                    occupied = 17 <= radius <= 29 or inside(nx,ny,[(17,3),(22,13),(12,15)]) or inside(nx,ny,[(47,3),(52,15),(42,13)])
                else: occupied = inside(nx,ny,HULLS[family][generation-1])
                if occupied: self.cells[y*size+x] = METAL
        if family == 1 and generation in [3,4,5]:
            for x0,x1 in [(14,23),(41,50)] if generation == 3 else [(15,21),(43,49)] if generation == 4 else [(12,24),(40,52)]:
                y0,y1 = (19,47) if generation == 5 else (24,40)
                self.cut([(x0,y0),(x1,y0),(x1,y1),(x0,y1)])
        if family == 0 and generation >= 4:
            for x0,x1 in [(19,24),(40,45)]: self.cut([(x0,28),(x1,28),(x1,35),(x0,35)])
        if family == 2 and generation >= 4:
            for x0,x1 in [(12,16),(48,52)]: self.cut([(x0,27),(x1,25),(x1,34),(x0,36)])
        self.occupied = {(i%size,i//size) for i,c in enumerate(self.cells) if c}
        neighbors = lambda x,y: [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
        self.edge = {p for p in self.occupied if any(q not in self.occupied for q in neighbors(*p))}
        for x,y in self.occupied:
            rim = any(q in self.edge for q in neighbors(x,y))
            color = INK if (x,y) in self.edge else self.accent if rim else LIGHT if x+y < size*.75 else METAL if x < size*.5 else DARK
            if (x,y-1) not in self.occupied: color = SILVER
            self.cells[y*size+x] = color
            self.roles[y*size+x] = 1 if x < size*.36 else 2 if x >= size*.64 else 0

    def pixel(self,x,y,color,role=None):
        if (x,y) in self.occupied and (x,y) not in self.edge:
            self.cells[y*self.size+x] = color
            if role is not None: self.roles[y*self.size+x] = role

    def cut(self, vertices):
        for y in range(self.size):
            for x in range(self.size):
                if inside((x+.5)/self.factor,(y+.5)/self.factor,vertices): self.cells[y*self.size+x] = 0

    def line(self,points,color,role=None):
        points = [(round(x*self.factor),round(y*self.factor)) for x,y in points]
        for (x,y),(xx,yy) in zip(points,points[1:]):
            dx,dy = abs(xx-x),-abs(yy-y); sx,sy = 1 if x<xx else -1,1 if y<yy else -1; error=dx+dy
            while True:
                self.pixel(x,y,color,role)
                if (x,y)==(xx,yy): break
                twice=error*2
                if twice>=dy: error+=dy; x+=sx
                if twice<=dx: error+=dx; y+=sy

    def panel(self,vertices,color):
        for y in range(self.size):
            for x in range(self.size):
                if inside((x+.5)/self.factor,(y+.5)/self.factor,vertices): self.pixel(x,y,color)
        self.line(vertices+vertices[:1],DEEP)
        self.line(vertices[:2],SILVER)

    def vents(self,x,y,width,rows):
        for row in range(rows):
            self.line([(x,y+row*3),(x+width,y+row*3)],DEEP)
            self.line([(x,y+row*3+1),(x+width-1,y+row*3+1)],LIGHT)

    def hardware(self):
        f,g = self.family,self.generation
        if f == 0:
            self.panel([(10,17),(21,22),(24,34),(18,41),(11,31)],METAL)
            self.panel([(43,22),(54,17),(53,31),(46,41),(40,34)],DARK)
            self.panel([(27,19),(32,8+g),(37,19),(39,43),(25,43)],LIGHT)
            self.line([(15,24),(19,32),(17,36)],self.pale)
            self.line([(49,24),(45,32),(47,36)],self.accent)
            self.vents(21,45,6,3); self.vents(38,45,6,3)
            self.line([(27,22),(29,20),(35,20),(37,22)],DEEP)
            self.line([(29,24),(35,24)],SILVER)
            for x,sign in [(13,1),(51,-1)]:
                self.line([(x,18),(x+sign*4,24),(x+sign*5,29)],DEEP)
                self.line([(x+sign,18),(x+sign*5,24)],SILVER)
                self.line([(x+sign*2,33),(x+sign*7,37)],self.shadow)
                if g>=3: self.vents(x-sign*2,41,5,2)
            if g >= 3:
                self.line([(9,12),(15,17),(18,25)],self.accent)
                self.line([(55,12),(49,17),(46,25)],self.pale)
            if g == 5:
                for x in [9,21,43,55]: self.line([(x,9),(x,19)],self.pale)
                self.line([(9,48),(16,53),(22,52)],DEEP)
                self.line([(55,48),(48,53),(42,52)],SILVER)
        elif f == 1:
            self.panel([(9,15),(21,15),(21,45),(15,51),(8,42)],METAL)
            self.panel([(43,15),(55,15),(56,42),(49,51),(43,45)],DARK)
            self.panel([(24,13),(40,13),(43,22),(40,48),(24,48),(21,22)],LIGHT)
            self.panel([(18,7),(46,7),(50,12),(14,12)],SILVER)
            self.line([(21,9),(43,9)],self.accent)
            self.vents(10,21,8,7); self.vents(46,21,8,7)
            self.line([(25,17),(25,24),(27,26)],DEEP)
            self.line([(39,17),(39,24),(37,26)],SILVER)
            self.line([(28,18),(36,18)],self.shadow)
            self.line([(28,20),(32,20)],self.pale)
            self.vents(27,46,10,2)
            self.line([(7,18),(7,39),(10,43)],SILVER)
            self.line([(57,18),(57,39),(54,43)],DEEP)
            for x,y in [(10,16),(51,16),(12,48),(48,48),(25,53),(37,53)]:
                self.pixel(round(x*self.factor),round(y*self.factor),WHITE)
            if g >= 4:
                for x,y in [(13,9),(49,9),(13,52),(49,52)]:
                    self.panel([(x-4,y-3),(x+4,y-3),(x+4,y+3),(x-4,y+3)],self.shadow)
        else:
            self.panel([(10,13),(18,17),(22,29),(17,39),(11,29)],METAL)
            self.panel([(46,17),(54,13),(53,29),(47,39),(42,29)],DARK)
            self.line([(18,20),(22,27),(23,39),(28,49)],self.pale)
            self.line([(46,20),(42,27),(41,39),(36,49)],self.accent)
            for y in [29,34,39]:
                self.line([(12,y),(19,y+2)],self.shadow)
                self.line([(52,y),(45,y+2)],self.shadow)
            if g >= 4:
                for x in [17,23,41,47]: self.line([(x,13),(x,19)],self.pale)
            if g==3:
                for angle in [0,.7,1.4,2.1,2.8,3.5,4.2,4.9,5.6]:
                    points=[(32+radius*math.cos(angle),31+radius*math.sin(angle)) for radius in [19,23,27]]
                    self.line(points,DEEP)
                    self.line([(32+radius*math.cos(angle+.07),31+radius*math.sin(angle+.07)) for radius in [22,26]],self.pale)
                self.line([(21,8),(24,11)],self.accent)
                self.line([(43,8),(40,11)],self.accent)
            else:
                self.line([(27,18),(32,15),(37,18),(37,22)],DEEP)
                self.line([(29,20),(35,20)],self.pale)
                for x,sign in [(12,1),(52,-1)]:
                    for y in [19,23,27]:
                        self.line([(x,y),(x+sign*4,y+3)],self.shadow)
                        self.line([(x+sign,y),(x+sign*4,y+2)],SILVER)
                self.line([(25,44),(28,50),(32,53),(36,50),(39,44)],DEEP)
                self.line([(28,47),(32,50),(36,47)],self.pale)
        # Reactor has a beveled mechanical collar, dark gasket, chromatic lens.
        cx,cy = self.size//2, round((53 if f==2 and g==3 else 33)*self.factor)
        radius = max(4,round(5*self.factor))
        for y in range(cy-radius-4,cy+radius+5):
            for x in range(cx-radius-4,cx+radius+5):
                distance=math.hypot(x-cx,y-cy)
                if distance <= radius+3:
                    role,color = 3, SILVER if x+y<cx+cy else METAL
                    if distance <= radius+2: color=INK
                    if distance <= radius:
                        role=4
                        color = WHITE if distance<radius*.35 else self.pale if x<cx else self.accent
                    self.pixel(x,y,color,role)
        # Emitter role cells are small actual occupied firing hardware.
        emitters = [(17,43),(47,43)] if f==0 else [(8,45),(56,45)] if f==1 else [(8,31),(56,31)] if g==3 else [(19,23),(45,23)]
        for part,(x,y) in enumerate(emitters,5):
            for yy in range(round((y-3)*self.factor),round((y+3)*self.factor)):
                for xx in range(round((x-2)*self.factor),round((x+2)*self.factor)):
                    self.pixel(xx,yy,self.pale if xx%3==0 else self.accent,part)
            self.line([(x-2,y-4),(x+2,y-4)],DEEP)
        core_position=[cx+.5-self.size/2,cy+.5-self.size/2]
        wing=round(self.size*.22)
        parts=[
            {"name":"hull","parent":None,"position":[0,0],"role":"armor"},
            {"name":"port_armor","parent":0,"position":[-wing,0],"role":"armor"},
            {"name":"starboard_armor","parent":0,"position":[wing,0],"role":"armor"},
            {"name":"reactor_collar","parent":0,"position":core_position,"role":"trim"},
            {"name":"reactor_lens","parent":3,"position":[0,0],"role":"core"},
        ]
        for index,(x,y) in enumerate(emitters):
            absolute=[round(x*self.factor)+.5-self.size/2,round(y*self.factor)+.5-self.size/2]
            parts.append({"name":["port_emitter","starboard_emitter"][index],"parent":index+1,"position":[absolute[0]-[-wing,wing][index],absolute[1]],"role":"emitter"})
        self.metadata={"name":NAMES[f][g-1],"family":f,"generation":g,"parts":parts,"roles":self.roles,"core_position":core_position,"core_radius":radius+.5}
        assert self.cells[cy*self.size+cx],"core must be an occupied collision cell"
        assert 4 in self.roles,"core must have visible role cells"

def segment(generation):
    size=14+generation*2; center=size/2
    cells=[];roles=[]
    for y in range(size):
        for x in range(size):
            dx,dy=x+.5-center,y+.5-center
            radius=math.hypot(dx,dy)
            occupied=(abs(dx)+abs(dy)<center-1) if generation in [1,3] else (abs(dx)<center-2 and abs(dy)<center-3)
            if generation==3: occupied|=abs(dy)<1 and abs(dx)<center
            if generation>=4: occupied|=abs(dx)<2 and abs(dy)<center
            if generation==5: occupied|=abs(dy)<2 and abs(dx)<center
            color=0
            if occupied:
                color=LIGHT if dx+dy<0 else DARK
                if radius<center*.45: color=18 if dx<0 else 17
                elif (x+y)%4==0: color=16
            cells.append(color);roles.append(1 if color and radius<center*.45 else 0)
    occupied={(i%size,i//size) for i,c in enumerate(cells) if c}
    for x,y in occupied:
        index=y*size+x
        if any(p not in occupied for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]):
            cells[index]=6 if (x,y-1) not in occupied else INK
        elif abs(x+.5-center)>center*.5 and y%4==1: cells[index]=16
        elif abs(x+.5-center)>center*.5 and y%4==2: cells[index]=6
    return {"width":size,"height":size,"indices":cells},{"name":f"Coil segment {generation}","family":2,"generation":generation,"parts":[{"name":"segment_armor","parent":None,"position":[0,0],"role":"armor"},{"name":"segment_reactor","parent":0,"position":[0,0],"role":"core"}],"roles":roles,"core_position":[0,0],"core_radius":size*.2}

sprites,rigs,budgets={},{},[]
for family,name in enumerate(FAMILIES):
    for generation in range(1,6):
        size=56+generation*8
        hull=Hull(family,generation,size);hull.hardware()
        key=f"boss_{name}_{generation}"
        sprites[key]={"width":size,"height":size,"indices":hull.cells}
        rigs[key]=hull.metadata
        occupied=sum(bool(c) for c in hull.cells)
        assert occupied<=6500,(key,occupied)
        budgets.append({"sprite":key,"size":size,"cells":occupied,"parts":7,"core_cells":sum(c!=0 and r==4 for c,r in zip(hull.cells,hull.roles))})
for generation in range(1,6):
    key=f"boss_coil_{generation}_segment";sprites[key],rigs[key]=segment(generation)
    budgets.append({"sprite":key,"size":sprites[key]["width"],"cells":sum(bool(c) for c in sprites[key]["indices"]),"parts":2})

atlas={"version":1,"palette":PALETTE,"sprites":sprites,"rigs":rigs}
(ROOT/"boss-atlas.json").write_text(json.dumps(atlas,separators=(",",":"))+"\n",encoding="utf-8")
(ROOT/"boss-budgets.json").write_text(json.dumps({"provenance":"Original authored cells from boss-source.py; no generated-image provenance implied.","source_dimensions":"64-unit individually authored design canvases, sampled at each native generation size","sprites":budgets},indent=2)+"\n",encoding="utf-8")
print(f"Baked {len(sprites)} boss bodies; maximum head occupancy {max(b['cells'] for b in budgets[:15])}, palette64")

# Optional contact sheet for actual cell review; runtime never loads this bitmap.
try:
    from PIL import Image, ImageDraw
    preview=Image.new("RGB",(1120,820),(7,12,25));draw=ImageDraw.Draw(preview)
    for family,name in enumerate(FAMILIES):
        for generation in range(1,6):
            key=f"boss_{name}_{generation}";sprite=sprites[key];size=sprite["width"]
            body=Image.new("RGBA",(size,size));body.putdata([tuple(PALETTE[i].to_bytes(4,"little")) for i in sprite["indices"]])
            body=body.resize((size*2,size*2),Image.Resampling.NEAREST)
            x=(generation-1)*224+(224-body.width)//2;y=family*244+29
            preview.paste(body,(x,y),body)
            draw.text(((generation-1)*224+12,family*244+8),f"{name.upper()} {generation} / {NAMES[family][generation-1]}",fill=(185,210,220))
            draw.text(((generation-1)*224+12,family*244+224),f"{size}px / {sum(bool(i) for i in sprite['indices'])} cells",fill=(123,157,181))
    for generation in range(1,6):
        sprite=sprites[f"boss_coil_{generation}_segment"];size=sprite["width"]
        body=Image.new("RGBA",(size,size));body.putdata([tuple(PALETTE[i].to_bytes(4,"little")) for i in sprite["indices"]])
        body=body.resize((size*2,size*2),Image.Resampling.NEAREST)
        preview.paste(body,((generation-1)*224+30,756),body)
        draw.text(((generation-1)*224+92,762),f"SEGMENT {generation}\n{sum(bool(i) for i in sprite['indices'])} cells",fill=(175,245,188))
    preview.save(ROOT/"boss-preview.png")
except ImportError:
    print("Preview skipped: install Pillow to review the optional contact sheet.")
