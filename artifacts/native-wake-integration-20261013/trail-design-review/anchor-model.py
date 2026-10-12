from pathlib import Path
import json, struct
f32=lambda x: struct.unpack('<f', struct.pack('<f',x))[0]
origin=f32(1_000_000)
forward_x=f32(-1)
anchor=f32(origin-f32(forward_x*f32(14)))
result={'scope':'Independent IEEE-754/source-equation model only; not actual native tick or proposed exporter execution','source_equation':'origin - forward(heading) * 14','source_pilot_maximum':origin,'inward_forward_x':forward_x,'anchor_x':anchor,'old_proposed_point_bound':1_000_000,'old_bound_accepts':abs(anchor)<=1_000_000,'new_narrow_point_bound':1_000_014,'narrow_bound_accepts':abs(anchor)<=1_000_014}
Path('work/coop-wake-presentation-review/anchor-model.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result,indent=2))
