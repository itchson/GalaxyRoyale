from pathlib import Path
import hashlib, json, subprocess

root = Path.cwd()
work = root / 'work/boxed-world-review'
candidate = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
base = subprocess.check_output(['git', 'rev-parse', 'HEAD^'], text=True).strip()
reference = '5e4b629e91f3334582ffa30d5e65af98675f8c39'
paths = [
 'crates/game_core/src/co_op_tests.rs',
 'crates/game_core/src/co_op.rs',
 'crates/game_core/src/co_op_publish.rs',
 'crates/game_core/src/co_op_control.rs',
 'crates/game_core/src/co_op_replica.rs',
 'crates/game_core/src/lib.rs',
 'crates/game_core/src/campaign.rs',
 'crates/engine_core/src/pixels.rs',
 'crates/engine_core/src/frame.rs',
 'crates/game_web/src/co_op_replica_bridge.rs',
 'crates/game_web/src/co_op_bridge.rs',
 'crates/game_web/src/browser.rs',
 'apps/game/src/coop/controller.ts',
 'apps/game/src/coop/wasm-ports.ts',
 'apps/game/src/main.ts',
]
source = []
for path in paths:
    blob = subprocess.check_output(['git', 'show', candidate + ':' + path])
    old = subprocess.check_output(['git', 'show', reference + ':' + path])
    out = work / 'source' / 'candidate' / path
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(blob)
    source.append({'path': path, 'git_blob': subprocess.check_output(['git','rev-parse',candidate+':'+path],text=True).strip(), 'bytes':len(blob), 'sha256':hashlib.sha256(blob).hexdigest(), 'reference_root_git_blob':subprocess.check_output(['git','rev-parse',reference+':'+path],text=True).strip(), 'identical_to_reference_root':blob==old, 'snapshot':out.relative_to(work).as_posix()})
    if blob != old:
        out = work / 'source' / 'root-5e4' / path
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_bytes(old)
        source[-1]['reference_root_snapshot'] = out.relative_to(work).as_posix()
        source[-1]['reference_root_sha256'] = hashlib.sha256(old).hexdigest()
        source[-1]['reference_root_bytes'] = len(old)

print(json.dumps({'candidate':candidate,'base':base,'reference_root':reference,'unchanged_boundary_files':[item['path'] for item in source if item['identical_to_reference_root']], 'different_files':[item['path'] for item in source if not item['identical_to_reference_root']]}, indent=2))
(work/'source-pins.json').write_text(json.dumps({'version':1,'semantics':'bytes and SHA256 are exact canonical git show blob bytes; snapshots preserve them. Working checkout newline bytes are not substituted.','candidate':candidate,'base':base,'reference_root':reference,'source':source},indent=2)+'\n',encoding='utf-8')
