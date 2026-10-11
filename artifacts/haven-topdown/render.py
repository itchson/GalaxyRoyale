"""Flat software raster of actual native Game quads; no GPU/browser claim."""
import json
import sys
from pathlib import Path
from PIL import Image, ImageDraw

folder=Path(sys.argv[1]) if len(sys.argv)>1 else Path("work/haven-topdown-repro")
data=json.loads((folder/'native-extraction.json').read_text(encoding='utf-8'))
background=(5,11,18)
image=Image.new('RGB',(1460,650),background)
draw=ImageDraw.Draw(image)
for row,frame in enumerate(data['frames']):
    for column,group in enumerate(frame['groups']):
        x=column*292+12
        y=row*325+48
        zoom=4 if group['name']=='dock' else 6
        half=32 if group['name']=='dock' else 22
        draw.text((x,row*325+10),f"{group['name'].upper()} / {frame['stage']}",fill=(181,210,218))
        draw.text((x,row*325+26),f"{group['occupied_cells']} cells / {group['transforms']} transforms",fill=(103,145,158))
        for cell in group['cells']:
            rgba=[(cell['color']>>s)&255 for s in (0,8,16,24)]
            color=tuple(round(c*rgba[3]/255+b*(1-rgba[3]/255)) for c,b in zip(rgba[:3],background))
            corners=[(x+((p[0]-group['center'][0])/group['scale']+half)*zoom,
                      y+((p[1]-group['center'][1])/group['scale']+half)*zoom) for p in cell['corners']]
            draw.polygon(corners,fill=color)
image.save(folder/'contact-sheet.png')

overview=Image.new('RGB',(620,510),background)
world=ImageDraw.Draw(overview)
world.text((16,12),'ACTUAL NATIVE HAVEN / common 2px per world unit / flat software cells',fill=(181,210,218))
for group in data['frames'][0]['groups']:
    for cell in group['cells']:
        color=tuple((cell['color']>>s)&255 for s in (0,8,16))
        world.polygon([(310+p[0]*2,245+p[1]*2) for p in cell['corners']],fill=color)
    world.text((310+group['center'][0]*2-24,245+group['center'][1]*2+32),group['name'].upper(),fill=(140,176,185))
overview.save(folder/'world-overview.png')
print(json.dumps({'scope':'Actual native Game packets, flat software colors, normalized magnification on contact sheet',
 'objects':[{'name':g['name'],'cells':g['occupied_cells'],'transforms':g['transforms']} for g in data['frames'][0]['groups']],
 'total_haven_cells':sum(g['occupied_cells'] for g in data['frames'][0]['groups']),
 'images':['contact-sheet.png','world-overview.png']},indent=2))
