"""Flat alpha-composited native packet review; not GPU soft filtering."""
import json
import math
import sys
from pathlib import Path
from PIL import Image, ImageDraw

folder=Path(sys.argv[1]).resolve()
data=json.loads((folder/'native-packets.json').read_text(encoding='utf-8'))
background=(5,9,16,255)
sample=4
canvas=Image.new('RGB',(1066,4*336+320),background[:3])
labels=ImageDraw.Draw(canvas)


def render(scene,size,translate=(0,0),scale=1,head_only=False):
    image=Image.new('RGBA',(size[0]*sample,size[1]*sample),background)
    for cell in scene['cells']:
        if head_only and cell['emission']<0:
            continue
        r,g,b,a=((cell['color']>>s)&255 for s in (0,8,16,24))
        if not a:continue
        # Bounded shader emission brightness approximated only for the flat proof.
        emission=max(0,-cell['emission']-1) if cell['emission']<0 else cell['emission']
        factor=(1+0.3*emission)**(1/2.2)
        color=(min(255,round(r*factor)),min(255,round(g*factor)),min(255,round(b*factor)),a)
        polygon=[((cell['corners'][i][0]-translate[0])*scale*sample,
                  (cell['corners'][i][1]-translate[1])*scale*sample) for i in (0,1,3,2)]
        x0=max(0,math.floor(min(p[0] for p in polygon)))
        y0=max(0,math.floor(min(p[1] for p in polygon)))
        x1=min(image.width,math.ceil(max(p[0] for p in polygon))+1)
        y1=min(image.height,math.ceil(max(p[1] for p in polygon))+1)
        if x1<=x0 or y1<=y0:continue
        overlay=Image.new('RGBA',(x1-x0,y1-y0))
        d=ImageDraw.Draw(overlay)
        d.polygon([(x-x0,y-y0) for x,y in polygon],fill=color)
        image.alpha_composite(overlay,(x0,y0))
    return image.resize(size,Image.Resampling.BOX).convert('RGB')


for i,scene in enumerate(data['scenes']):
    x=i%2*533;y=i//2*336
    labels.text((x+10,y+6),f"{scene['name']} / {scene['quads']} cells / zoom{scene['zoom']}",fill=(156,195,214))
    canvas.paste(render(scene,(533,300)),(x,y+24))
labels.text((16,4*336+6),'Actual native head packets, magnified24x; no replacement art or GPU coverage claim',fill=(156,195,214))
canvas.paste(render(data['scenes'][0],(480,280),translate=(256.5,144.5),scale=24,head_only=True),(16,4*336+26))
canvas.save(folder/'native-contact-sheet.png')
print('Eight CameraView-projected packet scenes and a magnified exact-cell inset rasterized')
