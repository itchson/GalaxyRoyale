import json, sys
from pathlib import Path
from PIL import Image, ImageDraw
folder=Path(sys.argv[1]).resolve()
data=json.loads((folder/'native-packets.json').read_text(encoding='utf-8'))
canvas=Image.new('RGB',(3*320,3*416+310),(5,11,18));d=ImageDraw.Draw(canvas)
def cells(items,x,y,zoom):
    for cell in items:
        r,g,b,a=((cell['color']>>s)&255 for s in (0,8,16,24))
        if not a:continue
        d.polygon([(x+cell['corners'][i][0]*zoom,y+cell['corners'][i][1]*zoom) for i in (0,1,3,2)],fill=(r,g,b))
for i,scene in enumerate(data['portraits']):
    x=i%3*320+16;y=i//3*416+28
    d.text((x,y-20),scene['name']+f" / {scene['quads']} cells",fill=(179,217,229))
    cells(scene['cells'],x,y,3)
for i,scene in enumerate(data['icons']):
    x=i%5*192+48;y=3*416+22+i//5*150
    d.text((x,y-15),scene['name'].split('-')[0],fill=(179,217,229))
    cells(scene['cells'],x,y,3)
    d.text((x,y+104),str(scene['quads'])+' cells',fill=(110,152,169))
canvas.save(folder/'native-contact-sheet.png')
print('Flat native packets rasterized; no GPU/browser/material or animation-playback claim')
