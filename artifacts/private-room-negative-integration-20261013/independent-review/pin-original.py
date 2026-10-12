from pathlib import Path
import hashlib,json,subprocess
own=Path.cwd();work=own/'work/private-room-negative-independent-review'
author=Path('C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale')
aw=author/'work/private-room-negative-implementation'
pins=json.loads((aw/'source-pins.json').read_text(encoding='utf-8-sig'))
commit='1b14bff9e6da0a1a249f1bdc42ab49e59e74e52c'
source=[]
for path,pin in pins['sources'].items():
    data=subprocess.check_output(['git','show',commit+':'+path],cwd=author)
    assert len(data)==pin['canonicalGitBytes'] and hashlib.sha256(data).hexdigest()==pin['canonicalGitSha256'],path
    out=work/'original-source'/path;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
    source.append({'path':path,'git_blob':subprocess.check_output(['git','rev-parse',commit+':'+path],cwd=author,text=True).strip(),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'snapshot':out.relative_to(work).as_posix()})
for name,pin in pins['evidence'].items():
    data=(aw/name).read_bytes();assert len(data)==pin['bytes'] and hashlib.sha256(data).hexdigest()==pin['sha256'],name
    out=work/'original-evidence'/name;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
(work/'original-evidence/source-pins.json').write_bytes((aw/'source-pins.json').read_bytes())
inputs=['apps/game/src/coop/controller.ts','apps/game/src/coop/transport.ts','apps/game/src/coop/signaling.ts','apps/game/src/coop/wasm-ports.ts','scripts/qa/co-op-transport/prepare.mjs','scripts/test/coop-transport-wasm-qa.test.mjs','scripts/test/coop-transport-fifth-qa.test.mjs']
unchanged=[]
for path in inputs:
    data=subprocess.check_output(['git','show',commit+':'+path],cwd=author)
    base=subprocess.check_output(['git','show',pins['base']+':'+path],cwd=author)
    assert data==base,path
    out=work/'baseline-inputs'/path;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
    unchanged.append({'path':path,'git_blob':subprocess.check_output(['git','rev-parse',commit+':'+path],cwd=author,text=True).strip(),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'unchanged_from_base':True,'snapshot':out.relative_to(work).as_posix()})
for filename,count in [('negative-tests-final.log',11),('existing-wasm-functions.log',16),('existing-fifth-functions.log',10)]:
    text=(aw/filename).read_text(encoding='utf-8-sig')
    assert f'pass {count}' in text and 'fail 0' in text,filename
result={'scope':'Independent frozen Git/evidence/source consumer review, plus two declared-observation source-function counterexamples; no actual native/browser/RTC/listener execution','original_candidate':commit,'original_source_pins':source,'unchanged_inputs':unchanged,'original_raw_evidence_verified':len(pins['evidence']),'author_recorded_function_tests':37,'original_independent_reproduction_pass':0,'original_independent_reproduction_fail':2,'remaining_findings':['post-policy commit floor','visibility through awaited observations'],'native_loop_health_qualification_pending':True}
(work/'original-verification.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:v for k,v in result.items() if k not in ['original_source_pins','unchanged_inputs']},indent=2))
