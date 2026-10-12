from pathlib import Path
import json
from PIL import Image,ImageDraw
folder=Path(__file__).resolve().parent
packet=json.loads((folder/"packets/menu.json").read_text())
factor=4
view=packet["viewport"]
canvas=Image.new("RGB",(view[0]*factor,view[1]*factor),(3,5,9))
draw=ImageDraw.Draw(canvas)
for batch in packet["batches"]:
    for cell in batch["pixels"]:
        color=cell["color"].to_bytes(4,"little")
        if not color[3]: continue
        t=batch["transforms"][cell["transform"]]
        points=[]
        for dx,dy in ((-.5,-.5),(.5,-.5),(.5,.5),(-.5,.5)):
            x,y=cell["local"][0]+dx,cell["local"][1]+dy
            points.append((factor*(t["a"][0]+t["a"][2]*x+t["b"][0]*y+view[0]/2),
                           factor*(t["a"][1]+t["a"][3]*x+t["b"][1]*y+view[1]/2)))
        draw.polygon(points,fill=tuple(color[:3]))
canvas=canvas.resize((view[0]*2,view[1]*2),Image.Resampling.BOX)
canvas.save(folder/"native-software-menu.png")
print(json.dumps({"scope":"Actual native UiScene packet;4x supersample BOX unlit software approximation, not GPU/browser","cells":packet["cells"],"transforms":sum(len(b["transforms"]) for b in packet["batches"])}))

