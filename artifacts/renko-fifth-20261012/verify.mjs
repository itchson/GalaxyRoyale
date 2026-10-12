import {readFile,readdir,writeFile,mkdir,copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const stage='C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/work/co-op-wasm-qa/renko-fifth/stage-650a546';
const dst='artifacts/renko-fifth-20261012';
const sha=b=>createHash('sha256').update(b).digest('hex');
await mkdir(dst+'/captures',{recursive:true});
await writeFile(dst+'/.gitattributes','* -text\n');
const captures=[];
for(const name of await readdir(stage+'/captures')){
 const bytes=await readFile(stage+'/captures/'+name),v=JSON.parse(bytes);
 if(!name.endsWith('-'+sha(bytes)+'.json')||v.source_commit!=='650a546b8f8c07150d4df27f8de7cb1d4e13d4e8'||v.backend!=='webgl2'||v.profile_mode!=='owned'||v.scenario!=='renko-fifth'||v.checks.some(c=>!c.passed))throw Error('Provenance/check failure '+name);
 await copyFile(stage+'/captures/'+name,dst+'/captures/'+name);
 captures.push({name,bytes:bytes.length,sha256:sha(bytes),v});
}
captures.sort((a,b)=>a.v.captured_at.localeCompare(b.v.captured_at));
if(captures.length!==8)throw Error('Eight actual actions required');
const final=captures.at(-1).v;
for(const c of final.clients){
 if(c.phase!=='disposed'||c.mode!=='solo'||c.created!==2||c.closed!==2||c.requests||c.timers||!c.profile_equal||!c.persisted_equal||c.profile_sha!==c.baseline_sha||c.persisted_sha!==c.baseline_sha)throw Error('Terminal/save mismatch');
 if(c.fifth_shot.state!=='complete'||c.fifth_shot.ordinals.length!==10)throw Error('Incomplete cadence');
 for(const row of c.fifth_shot.ordinals){const bonus=row.ordinal===5||row.ordinal===10;if(!row.owner_matched||!row.epoch_matched||row.projectiles!==(bonus?2:1)||bonus&&(Math.abs(row.damage_ratio-1.5)>1e-6||Math.abs(row.speed_delta-70)>1e-6))throw Error('Cadence mismatch');}
}
if(final.clients[0].fifth_shot.primaries!==10||final.clients[1].fifth_shot.primaries!==0||!final.session_distinct||!final.checks.some(c=>c.name==='native-fifth-shot')||!final.checks.some(c=>c.name==='dispose-clean'))throw Error('Required coverage missing');
await copyFile(stage+'/manifest.json',dst+'/stage-manifest.json');
await writeFile(dst+'/verified-summary.json',JSON.stringify({schemaVersion:1,scope:'Actual two native clients and real loopback RTC WebGL2; RENKO host/HUGO silent guest. Cadence from first session; second session only lifecycle. No internet/eight-player/real-time or other abilities claim.',source:final.source_commit,build:final.build_stamp,operations:captures.map(c=>({file:'captures/'+c.name,bytes:c.bytes,sha256:c.sha256,capturedAt:c.v.captured_at})),final},null,2)+'\n');
console.log(JSON.stringify({captures:captures.length,cumulativeChecks:final.checks.length,uniqueChecks:new Set(final.checks.map(c=>c.name)).size,primaries:10,bonusOrdinals:[5,10],disposed:true,savesUnchanged:true}));
