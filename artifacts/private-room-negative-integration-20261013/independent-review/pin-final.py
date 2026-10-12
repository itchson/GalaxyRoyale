from pathlib import Path
import hashlib,json,subprocess

own=Path.cwd(); work=own/'work/private-room-negative-independent-review'
author=Path('C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale')
aw=author/'work/private-room-negative-implementation/v2-continuity'
commit='7c188d4198901ead15e4bffa2b0063b9c95f7674'
original='1b14bff9e6da0a1a249f1bdc42ab49e59e74e52c'
sha=lambda data:hashlib.sha256(data).hexdigest()
pins=json.loads((aw/'source-pins.json').read_text(encoding='utf-8-sig'))
assert pins['head']==commit and pins['base']==original
assert subprocess.check_output(['git','rev-parse',commit+'^'],cwd=author,text=True).strip()==original
changed=subprocess.check_output(['git','diff','--name-only',original,commit],cwd=author,text=True).splitlines()
assert changed==['scripts/qa/co-op-transport/wasm-review.js','scripts/test/coop-private-room-negative-qa.test.mjs'],changed
source=[]
for path,pin in pins['sources'].items():
    data=subprocess.check_output(['git','show',commit+':'+path],cwd=author)
    assert len(data)==pin['canonicalGitBytes'] and sha(data)==pin['canonicalGitSha256'],path
    raw=(author/path).read_bytes()
    assert len(raw)==pin['workingBytes'] and sha(raw)==pin['workingSha256'],path
    out=work/'v2-source'/path; out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
    source.append({'path':path,'git_blob':subprocess.check_output(['git','rev-parse',commit+':'+path],cwd=author,text=True).strip(),'bytes':len(data),'sha256':sha(data),'snapshot':out.relative_to(work).as_posix(),'working_bytes':len(raw),'working_sha256':sha(raw)})
for name,pin in pins['evidence'].items():
    data=(aw/name).read_bytes()
    assert len(data)==pin['bytes'] and sha(data)==pin['sha256'],name
    out=work/'v2-evidence'/name;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
(work/'v2-evidence/source-pins.json').write_bytes((aw/'source-pins.json').read_bytes())
for name,count in [('focused-tests.log',17),('existing-wasm-functions.log',16),('existing-fifth-functions.log',10)]:
    log=(aw/name).read_text(encoding='utf-8-sig')
    assert f'pass {count}' in log and 'fail 0' in log,name
for name in pins['originalFailures']:
    log=(aw/name).read_text(encoding='utf-8-sig')
    assert 'fail 0' not in log and ('fail 1' in log or 'fail 2' in log or 'fail 3' in log),name
unchanged=[]
original_ver=json.loads((work/'original-verification.json').read_text(encoding='utf-8'))
for row in original_ver['original_source_pins']+original_ver['unchanged_inputs']:
    path=row['path']; data=subprocess.check_output(['git','show',commit+':'+path],cwd=author)
    if path in changed: continue
    assert len(data)==row['bytes'] and sha(data)==row['sha256'],path
    unchanged.append(path)
result={'scope':'Independent exact frozen source/evidence review; original two source-function failures preserved; no independent rerun, browser, native Game, RTC, build, stage or listener execution for v2','original_candidate':original,'final_candidate':commit,'parent_verified':True,'changed_files':changed,'final_source_pins':source,'final_raw_author_evidence_verified':len(pins['evidence']),'unchanged_original_source_and_consumer_inputs':unchanged,'author_original_function_tests':37,'author_final_function_tests':43,'original_independent_source_function_failures':2,'author_preserved_regression_logs':pins['originalFailures'],'resolved_findings':['post-policy complete-commit floor','visibility loss through awaited commands/observations','available native-loop failure/init/lifetime qualification'],'verdict':'No remaining source blocker for scoped integration and fresh live QA; actual browser/native/RTC acceptance remains pending','open_limits':['pagehide stopped edge is not directly observable from current child protocol','source-function tests use declared boundaries, not actual browser/RTC/native operations','normalized Profile and persisted digests do not prove full boxed-world preservation or zero migration writes','two loopback clients do not prove eight participants, Internet or TURN','policy sequencing does not prove every overlapping request/race']}
(work/'final-verification.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:v for k,v in result.items() if k not in ['final_source_pins']},indent=2))
