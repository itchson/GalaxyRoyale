from pathlib import Path
import hashlib, json, re, subprocess
own=Path.cwd()
work=own/'work/ion-wake-browser-boundary-review'
root=Path('C:/Users/mitch/Documents/ChatGPT/galaxy-royale')
author=Path('C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale')
commit='332ddd5ec0370e13ad346ea4ea988f35b61ca3e7'
paths=['apps/game/src/coop/wire.ts','apps/game/src/coop/wasm-ports.ts','apps/game/src/coop/controller.ts','crates/game_core/src/co_op_loadout.rs','crates/game_core/src/co_op_wake.rs','crates/game_core/src/co_op_wake_tests.rs','crates/game_core/src/co_op.rs','crates/game_web/src/co_op_bridge.rs','scripts/test/coop-wake-loadout.test.mjs','scripts/test/fixtures/co-op-publication/capture.json','scripts/test/fixtures/co-op-wake-loadout/.gitattributes','scripts/test/fixtures/co-op-wake-loadout/provenance.json']+[f'scripts/test/fixtures/co-op-wake-loadout/{name}.json' for name in ['legacy']+[f'ordinary-evolution-{i}' for i in range(1,6)]]
sources=[]
for path in paths:
    blob=subprocess.check_output(['git','show',commit+':'+path],cwd=root)
    raw=(root/path).read_bytes()
    git_blob=subprocess.check_output(['git','rev-parse',commit+':'+path],cwd=root,text=True).strip()
    working_blob=subprocess.check_output(['git','hash-object','--path='+path,path],cwd=root,text=True).strip()
    assert working_blob==git_blob,path
    out=work/'source'/path
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_bytes(blob)
    sources.append({'path':path,'git_blob':git_blob,'git_bytes':len(blob),'git_sha256':hashlib.sha256(blob).hexdigest(),'working_raw_bytes':len(raw),'working_raw_sha256':hashlib.sha256(raw).hexdigest(),'snapshot':out.relative_to(work).as_posix()})
logs=['native-wire.log','browser-boundary.log','clippy-native.log','clippy-native-final.log','wake-final.log','native-tests.log','native-tests-final.log','fmt-final.log','typecheck-final.log','export-fixtures.mjs']
for name in logs:
    out=work/'root-evidence'/name
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_bytes((root/'work/native-wake-integration-20261013'/name).read_bytes())
prov=json.loads((root/'scripts/test/fixtures/co-op-wake-loadout/provenance.json').read_text(encoding='utf-8-sig'))
log=(work/'root-evidence/native-wire.log').read_text(encoding='utf-8-sig')
log_final=(work/'root-evidence/wake-final.log').read_text(encoding='utf-8-sig')
fixtures=[]
for pin in prov['files']:
    name=pin['file']
    label='actual_legacy_export' if name=='legacy.json' else 'actual_wake_export_evolution_'+re.search(r'(\d)\.json$',name)[1]
    lines=[line for line in log.splitlines() if line.startswith(label+'=')]
    last=[line for line in log_final.splitlines() if line.startswith(label+'=')]
    assert len(lines)==len(last)==1,name
    actual=(lines[0].split('=',1)[1]+'\n').encode('utf-8')
    assert actual==(last[0].split('=',1)[1]+'\n').encode('utf-8'),name
    raw=(root/'scripts/test/fixtures/co-op-wake-loadout'/name).read_bytes()
    auth=(author/'work/coop-ion-wake/wire-fixtures'/name).read_bytes()
    assert raw==actual==auth,name
    assert len(raw)==pin['bytes'] and hashlib.sha256(raw).hexdigest()==pin['sha256'],name
    out=work/'author-fixtures'/name
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_bytes(auth)
    fixtures.append({'file':name,'actual_root_serializer_line':label,'bytes':len(raw),'sha256':pin['sha256'],'root_final_wake_run_and_author_byte_equal':True})
assert '1 passed; 0 failed' in log
assert '13 passed; 0 failed' in log_final
node=(work/'root-evidence/browser-boundary.log').read_text(encoding='utf-8-sig')
assert re.search(r'pass 21\b',node) and re.search(r'fail 0\b',node)
results={}
for name in ['native-tests.log','native-tests-final.log']:
    text=(work/'root-evidence'/name).read_text(encoding='utf-8-sig')
    rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
    results[name]={'passed':sum(int(p) for p,_,_ in rows),'failed':sum(int(f) for _,f,_ in rows),'ignored':sum(int(i) for _,_,i in rows),'result_records':len(rows)}
    assert results[name]['failed']==0 and results[name]['passed']==914 and results[name]['ignored']==17,name
result={'scope':'Independent source/recorded-evidence verification only; this reviewer did not rerun Node/native/build/WASM/browser/RTC','root_frozen_commit':commit,'source_count':len(sources),'sources':sources,'actual_fixture_agreement':fixtures,'recorded_node_tests':21,'recorded_native_serializer_tests':1,'recorded_final_wake_tests':13,'recorded_full_native_suites':results,'retained_clippy_failure':True}
(work/'verification.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:v for k,v in result.items() if k not in ['sources','actual_fixture_agreement']},indent=2))
