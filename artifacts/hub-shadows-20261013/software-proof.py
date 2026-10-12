"""Actual native packets, independent CPU ray oracle. No GPU/screenshot claim."""
from pathlib import Path
import json, math
from PIL import Image,ImageDraw
base=Path(__file__).resolve().parent
cases=json.loads((base/'native-deck-packets.json').read_text())['cases']
image=Image.new('RGB',(1536,1080),(4,7,11)); draw=ImageDraw.Draw(image)
draw.text((16,8),'CPU RAY PROOF / native256 occupied deck + actual Game RAICHO masks / NOT GPU',(166,206,218))
def blocked(c,p,source=(0.,0.)):
 n=c['native']; z=n['height']; t=z/42.; q=[p[i]*(1-t)+source[i]*t-n['position'][i] for i in range(2)]
 x=n['basis_x']; y=n['basis_y']; det=x[0]*y[1]-x[1]*y[0]
 local=[(y[1]*q[0]-y[0]*q[1])/det,(-x[1]*q[0]+x[0]*q[1])/det]
 cx=math.floor(local[0]+n['width']/2); cy=math.floor(local[1]+n['height_cells']/2)
 if cx<0 or cy<0 or cx>=n['width'] or cy>=n['height_cells']:return False
 bit=cy*128+cx; return bool(n['mask'][bit//32]&(1<<(bit%32)))
assertions=0
for case_index,c in enumerate(cases):
 col=case_index//2; row=case_index%2; ox=col*512; oy=30+row*520; center=(ox+256,oy+268); scale=2.4
 def screen(p):return (center[0]+p[0]*scale,center[1]+p[1]*scale)
 n=c['native']; label=f"{n['position']} / {'DAMAGED 618' if n['damaged'] else 'HEALTHY 662'} / deck28549"
 draw.text((ox+12,oy+8),label,(162,199,206))
 native_blocks=0
 for p in c['pixels']:
  center_block=p['receiver'] and blocked(c,p['center']); assert center_block==p['blocked_center']; assertions+=1
  native_blocks+=center_block
  rgb=[((p['rgba']>>(8*i))&255)/255. for i in range(3)]; response,emission=p['material']
  lin=[v**2.2 for v in rgb]; contact=0.
  if p['receiver']:
   contact=sum(blocked(c,p['center'],off) for off in [(-1.5,-1.5),(1.5,-1.5),(-1.5,1.5),(1.5,1.5)])/4.
  attenuation=max(0.,1.-sum(v*v for v in p['center'])/(120*120))**2
  energy=attenuation*n['light']['intensity']*response*(1-contact)
  reflected=[lin[i]*(1-.12*response)*(1-.18*contact)+lin[i]*.3*emission+n['light']['color'][i]*(lin[i]*.8+.012)*energy for i in range(3)]
  color=tuple(round(max(0.,min(1.,v))**(1/2.2)*255) for v in reflected)
  x=p['basis_x']; y=p['basis_y']; q=p['center']
  corners=[screen([q[0]+x[0]*a+y[0]*b,q[1]+x[1]*a+y[1]*b]) for a,b in [(-.5,-.5),(.5,-.5),(.5,.5),(-.5,.5)]]
  draw.polygon(corners,fill=color)
 assert native_blocks==c['center_blocked']
 # Actual coarse physics silhouette only, deliberately separate from the native
 # fine96 overlay; the proof concerns occlusion occupancy, not RAICHO art.
 for index,rgba in n['cells']:
  local=[index%n['width']+.5-n['width']/2,index//n['width']+.5-n['height_cells']/2]
  x=n['basis_x'];y=n['basis_y'];q=n['position']
  corners=[screen([q[0]+x[0]*(local[0]+a)+y[0]*(local[1]+b),q[1]+x[1]*(local[0]+a)+y[1]*(local[1]+b)]) for a,b in [(-.5,-.5),(.5,-.5),(.5,.5),(-.5,.5)]]
  draw.polygon(corners,fill=tuple((rgba>>(8*i))&255 for i in range(3)))
 draw.text((ox+12,oy+490),f"receiver25919 / center rays blocked{native_blocks} / z6 / source z42",(137,178,187))
image.save(base/'native-software-ray-contact.png')
print(f'{assertions} independent center-ray comparisons match actual native booleans; 6 cases; output native-software-ray-contact.png')
