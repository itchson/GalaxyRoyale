from pathlib import Path
import json, hashlib, subprocess, struct
own = Path.cwd()
work = own/'work/coop-wake-presentation-review'
work.mkdir(parents=True, exist_ok=True)
author = Path('C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale')
aw = author/'work/coop-wake-presentation-design'
pins = json.loads((aw/'source-pins.json').read_text(encoding='utf-8-sig'))
verified = []
for item in pins['source']:
    raw = (author/item['path']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == item['raw_file_sha256'], item['path']
    canonical = subprocess.check_output(['git','show',pins['frozen_base']+':'+item['path']], cwd=author)
    blob = subprocess.check_output(['git','rev-parse',pins['frozen_base']+':'+item['path']],cwd=author,text=True).strip()
    assert blob == item['head_git_blob'] == item['working_git_blob'], item['path']
    snapshot = work/'source'/item['path']
    snapshot.parent.mkdir(parents=True, exist_ok=True)
    snapshot.write_bytes(canonical)
    verified.append({'path':item['path'],'git_blob':blob,'git_bytes':len(canonical),'git_sha256':hashlib.sha256(canonical).hexdigest(),'raw_sha256':item['raw_file_sha256'],'snapshot':snapshot.relative_to(work).as_posix()})
for item in pins['primary_local_library_source']:
    data=Path(item['path']).read_bytes()
    assert hashlib.sha256(data).hexdigest()==item['sha256'],item['path']
    out=work/'serde-primary'/Path(item['path']).name
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_bytes(data)
for item in pins['design_evidence']:
    data=(aw/item['path']).read_bytes()
    assert len(data)==item['bytes'] and hashlib.sha256(data).hexdigest()==item['sha256'],item['path']
    out=work/'author-design'/item['path']
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_bytes(data)
(work/'author-design/source-pins.json').write_bytes((aw/'source-pins.json').read_bytes())
maximum=struct.unpack('<f',struct.pack('<f',3.4))[0]
probe=[]
for value in [1e-46,1.401298464324817e-45,3.4,maximum,3.4000001,3.4000003]:
    f32=struct.unpack('<f',struct.pack('<f',value))[0]
    probe.append({'raw_f64':value,'f32':f32,'raw_bound_pass':value>0 and value<=maximum,'f32_bound_pass':f32>0 and f32<=maximum})
(work/'float-model.json').write_text(json.dumps({'scope':'Independent Python IEEE-754 f32 conversion model, not execution of a future native/browser wake validator','maximum_f32':maximum,'values':probe},indent=2)+'\n',encoding='utf-8')
(work/'verified-source.json').write_text(json.dumps({'scope':'Independent raw byte and canonical Git pin verification only; no test/build/native/browser/RTC execution','frozen_base':pins['frozen_base'],'sources':verified,'library_source_count':len(pins['primary_local_library_source']),'design_evidence_count':len(pins['design_evidence'])},indent=2)+'\n',encoding='utf-8')
print(json.dumps({'verified_repo_sources':len(verified),'verified_primary_sources':len(pins['primary_local_library_source']),'verified_design_evidence':len(pins['design_evidence']),'f32_model':probe},indent=2))
