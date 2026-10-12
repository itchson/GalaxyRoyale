"""Independent read-only frozen-source/evidence verification. No native reruns."""
from pathlib import Path
import hashlib,json,re,subprocess
repo=Path('C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale')
review=Path(__file__).resolve().parent
commit='87de1f7afad1a53183fcca91e146c9a59ee012cd'
parent='5e4b629e91f3334582ffa30d5e65af98675f8c39'
base=repo/'work/coop-ion-wake'
def digest(b):return hashlib.sha256(b).hexdigest()
def blob(path,rev=commit):return subprocess.check_output(['git','cat-file','blob',f'{rev}:{path}'],cwd=repo)
def rawpin(path):
 data=path.read_bytes();return {'path':str(path).replace('\\','/'),'bytes':len(data),'sha256':digest(data)}
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==commit
assert subprocess.check_output(['git','rev-parse','HEAD^'],cwd=repo,text=True).strip()==parent
assert subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True)==''
manifest=json.loads((base/'candidate-pins.json').read_text())
source=[]
for row in manifest['source']:
 raw=(repo/row['path']).read_bytes();assert digest(raw)==row['raw_file_sha256'],row['path']
 gitid=subprocess.check_output(['git','rev-parse',f"{commit}:{row['path']}"],cwd=repo,text=True).strip()
 assert gitid==row['staged_git_blob'],row['path']
 canonical=blob(row['path']);target=review/'source'/row['path'];target.parent.mkdir(parents=True,exist_ok=True)
 if target.exists():assert target.read_bytes()==canonical
 else:target.write_bytes(canonical)
 source.append({'path':row['path'],'git_blob':gitid,'canonical_bytes':len(canonical),'canonical_sha256':digest(canonical),
 'raw_checkout_bytes':len(raw),'raw_checkout_sha256':digest(raw),'snapshot':str(target.relative_to(review)).replace('\\','/')})
inputs=[]
for name in ['crates/game_core/src/lib.rs','crates/game_core/src/ship_progression.rs','crates/game_core/src/bosses.rs','crates/game_core/src/regions.rs',
 'crates/game_core/src/co_op_fx.rs','crates/game_core/src/co_op_control.rs','crates/game_core/src/campaign.rs','crates/game_core/src/campaign_runtime.rs',
 'crates/game_core/src/co_op_publish.rs','crates/game_core/src/co_op_replica.rs','crates/game_web/src/co_op_bridge.rs','apps/game/src/coop/wire.ts']:
 data=blob(name);assert data==blob(name,parent),name
 target=review/'source'/name;target.parent.mkdir(parents=True,exist_ok=True)
 if target.exists():assert target.read_bytes()==data
 else:target.write_bytes(data)
 inputs.append({'path':name,'git':f'{commit}:{name}','bytes':len(data),'sha256':digest(data),'unchanged_from_parent':True,'snapshot':str(target.relative_to(review)).replace('\\','/')})
log=(base/'wake-tests-attempt-3.log').read_text(encoding='utf-8-sig')
lines=log.splitlines();fixtures=[]
for row in manifest['native_wire_fixtures']:
 data=(repo/row['path']).read_bytes();assert len(data)==row['bytes'] and digest(data)==row['sha256']
 name=Path(row['path']).stem
 prefix='actual_legacy_export=' if name=='legacy' else f'actual_wake_export_evolution_{name.rsplit("-",1)[1]}='
 payload=next(line[len(prefix):] for line in lines if line.startswith(prefix))
 assert data.rstrip(b'\r\n')==payload.encode(),name
 value=json.loads(data)
 if name=='legacy':assert 'wake_evolution' not in value and value['slots']==[None,None,None]
 else:
  evolution=int(name.rsplit('-',1)[1]);assert value['slots'][2]=={'index':5,'rank':5}
  assert value['ship_xp']==0
  assert value.get('wake_evolution',1)==evolution
  assert ('wake_evolution' in value)==(evolution>1)
 fixtures.append({**row,'matches_actual_native_output':True,'parsed_slot':value['slots'][2],
                  'ordinary_evolution':value.get('wake_evolution',1)})
assert 'test result: ok. 13 passed; 0 failed; 0 ignored' in log
co_op=(base/'co-op-regression.log').read_text(encoding='utf-8-sig')
assert 'test result: ok. 115 passed; 0 failed; 3 ignored' in co_op
baseline=(base/'baseline-reproduction.log').read_text(encoding='utf-8-sig')
assert 'actual_coop_enemy_health=100; ordinary_ion_wake_expected=99.9' in baseline and '0 passed; 1 failed' in baseline
code=blob('crates/game_core/src/co_op_wake_tests.rs').decode()
test_names=re.findall(r'#\[test\]\s*fn\s+(\w+)',code)
assert len(test_names)==13 and all(f'test co_op::wake_tests::{name} ... ok' in log for name in test_names)
paths=[path for path in sorted(base.rglob('*')) if path.is_file()]
raw=[rawpin(path) for path in paths]
result={'version':1,'candidate':commit,'parent':parent,'source':source,'unchanged_inputs':inputs,'raw_author_evidence':raw,
 'serializer_fixtures':fixtures,'test_names':test_names,'verified':{'raw_sources':len(source),'canonical_sources':len(source),
 'unchanged_inputs':len(inputs),'raw_author_files':len(raw),'wire_payloads':len(fixtures),'recorded_native_wake_passes':13,
 'recorded_co_op_passes':115,'recorded_co_op_ignored':3,'baseline_actual_failure_preserved':True},
 'scope':'Independent read-only source/byte/output checks; author native runs were not rerun; no browser/RTC/GPU/ownership/visual acceptance.'}
(review/'verification.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'candidate':commit,'verified':result['verified'],'source_pin_semantics':'canonical Git bytes and raw checkout bytes explicit; evidence pins exact raw filesystem bytes',
 'verification':rawpin(review/'verification.json')},indent=2))
