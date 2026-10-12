"""Evidence-only verification; no source writes, builds, or runtime claims."""
from pathlib import Path
import hashlib,json,subprocess
base=Path(__file__).resolve().parent
repo=base.parent.parent
original_path=base/'MANIFEST.json'; handoff_path=base/'HANDOFF.md'
original=json.loads(original_path.read_text())
def digest(data): return hashlib.sha256(data).hexdigest()
def raw_pin(path):
 data=path.read_bytes()
 return {'path':str(path.relative_to(repo)).replace('\\','/'),'bytes':len(data),'sha256':digest(data)}
assert digest(original_path.read_bytes())=='626cdf5ea5e9c443495b464632ab9771827c6a8ce795e7ebfaeef83ce94353d8'
assert digest(handoff_path.read_bytes())=='8b065e405cf1c24d6b23f0f00136c728e373153386426163eaa0413326fbacfc'
commit=original['commit']
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==commit
status=subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True)
assert status=='', 'tracked worktree must remain clean'
source=[]; comparisons=[]
for item in original['source']:
 path=repo/item['path']; working=path.read_bytes()
 assert len(working)==item['bytes'] and digest(working)==item['sha256'],item['path']
 object_name=f"{commit}:{item['path']}"
 blob=subprocess.check_output(['git','rev-parse',object_name],cwd=repo,text=True).strip()
 assert blob==item['git_blob'],item['path']
 canonical=subprocess.check_output(['git','cat-file','blob',object_name],cwd=repo)
 assert working.replace(b'\r\n',b'\n')==canonical,item['path']
 source.append({'path':item['path'],'git_blob':blob,'bytes':len(canonical),'sha256':digest(canonical),
  'working_checkout':{'bytes':len(working),'sha256':digest(working)}})
 comparisons.append({'path':item['path'],'git_blob_matches_original':True,
  'working_matches_original_raw_pin':True,'canonical_bytes':len(canonical),
  'canonical_sha256':digest(canonical),'working_bytes':len(working),
  'working_sha256':digest(working),'exact_byte_equal':working==canonical,
  'difference_is_only_CRLF_vs_LF':working!=canonical and working.replace(b'\r\n',b'\n')==canonical,
  'working_CRLF_count':working.count(b'\r\n'),'canonical_CRLF_count':canonical.count(b'\r\n')})
for item in original['inputs']+original['evidence']:
 current=raw_pin(repo/item['path'])
 assert current==item,item['path']
report={'version':1,'commit':commit,'operations':['git rev-parse <commit>:<path>','git cat-file blob <commit>:<path>','sha256 raw bytes'],
 'tracked_status':status,'preserved_original_manifest':raw_pin(original_path),'preserved_original_handoff':raw_pin(handoff_path),
 'source_count':len(source),'input_count':len(original['inputs']),'original_evidence_count':len(original['evidence']),
 'all_original_raw_pins_verified':True,'source_comparisons':comparisons,
 'canonical_source_pins':source,'scope':'Evidence byte verification only; no tests/builds/shaders/GPU/browser/runtime acceptance.'}
(base/'CANONICAL-GIT-VERIFY.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'commit':commit,'source_verified':len(source),'inputs_verified':len(original['inputs']),
 'raw_evidence_verified':len(original['evidence']),'newline_only_differences':[r for r in comparisons if not r['exact_byte_equal']],
 'verification':raw_pin(base/'CANONICAL-GIT-VERIFY.json')},indent=2))
