import json, math, sys
from pathlib import Path
from PIL import Image, ImageDraw
folder=Path(sys.argv[1]).resolve()
data=json.loads((folder/'native-packets.json').read_text(encoding='utf-8'))
zoom=2
tile_width=max(scene['width'] for scene in data['portraits'])*zoom+32
tile_height=max(scene['height'] for scene in data['portraits'])*zoom+40
portrait_height=math.ceil(len(data['portraits'])/3)*tile_height
canvas=Image.new('RGB',(max(3*tile_width,5*192),portrait_height+310),(5,11,18));d=ImageDraw.Draw(canvas)
def cells(items,x,y,zoom):
    for cell in items:
        r,g,b,a=((cell['color']>>s)&255 for s in (0,8,16,24))
        if not a:continue
        d.polygon([(x+cell['corners'][i][0]*zoom,y+cell['corners'][i][1]*zoom) for i in (0,1,3,2)],fill=(r,g,b))
for i,scene in enumerate(data['portraits']):
    x=i%3*tile_width+16;y=i//3*tile_height+28
    d.text((x,y-20),scene['name']+f" / {scene['quads']} cells",fill=(179,217,229))
    cells(scene['cells'],x,y,zoom)
for i,scene in enumerate(data['icons']):
    x=i%5*192+48;y=portrait_height+22+i//5*150
    d.text((x,y-15),scene['name'].split('-')[0],fill=(179,217,229))
    cells(scene['cells'],x,y,3)
    d.text((x,y+104),str(scene['quads'])+' cells',fill=(110,152,169))
canvas.save(folder/'native-contact-sheet.png')
print('Flat native packets rasterized; no GPU/browser/material or animation-playback claim')
