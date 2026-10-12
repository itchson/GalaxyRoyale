import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
const root=process.cwd(),dir='scripts/test/fixtures/co-op-wake-loadout';
await mkdir(dir,{recursive:true});
const log=await readFile('work/native-wake-integration-20261013/native-wire.log','utf8');
if(!log.includes('1 passed; 0 failed'))throw Error('No actual native wire pass');
const names=[['legacy','actual_legacy_export'],...Array.from({length:5},(_,i)=>['ordinary-evolution-'+(i+1),'actual_wake_export_evolution_'+(i+1)])];
const files=[];
for(const [name,label] of names){
 const lines=log.split(/\r?\n/).filter(line=>line.startsWith(label+'='));
 if(lines.length!==1)throw Error('Missing/ambiguous actual export '+label);
 const raw=Buffer.from(lines[0].slice(label.length+1)+'\n');
 JSON.parse(raw);
 const author=await readFile('C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale/work/coop-ion-wake/wire-fixtures/'+name+'.json');
 if(!raw.equals(author))throw Error('Root and author native payload bytes disagree');
 await writeFile(dir+'/'+name+'.json',raw);
 files.push({file:name+'.json',bytes:raw.length,sha256:createHash('sha256').update(raw).digest('hex')});
}
await writeFile(dir+'/.gitattributes','* -text\n');
await writeFile(dir+'/provenance.json',JSON.stringify({schemaVersion:1,nativeSourceCommit:'87de1f7afad1a53183fcca91e146c9a59ee012cd',rootExecutionCommit:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),nativeFunction:'Game::co_op_owned_loadout',nativeTest:'co_op::wake_tests::wake_wire_exports_effect_claim_only_and_keeps_legacy_omission_bytes',command:'cargo test --locked --offline -p game_core co_op::wake_tests::wake_wire_exports_effect_claim_only_and_keeps_legacy_omission_bytes -- --exact --nocapture',scope:'Actual Root native serializer lines, byte-agreed with separate author execution. Envelope supplied only by Node boundary test; no WASM/RTC or earned progression claim.',fixtureSetup:{legacy:'Profile::default()',ordinary:{storedRank:5,shipXp:0,nativeEffectiveRank:1,naturallyEarned:false,evolutionOneExportOmitsOptional:true}},files},null,2)+'\n');
console.log(JSON.stringify({fixtures:files.length}));
