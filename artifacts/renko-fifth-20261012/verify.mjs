import {readFile,readdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const base=new URL('./',import.meta.url);
const summary=JSON.parse(await readFile(new URL('verified-summary.json',base),'utf8'));
const sha=b=>createHash('sha256').update(b).digest('hex');
const captures=[];
for(const name of await readdir(new URL('captures/',base))){
 const bytes=await readFile(new URL('captures/'+name,base)),v=JSON.parse(bytes);
 const pin=summary.operations.find(p=>p.file==='captures/'+name);
 if(!pin||pin.bytes!==bytes.length||pin.sha256!==sha(bytes))throw Error('Byte provenance mismatch '+name);
 if(!name.endsWith('-'+sha(bytes)+'.json')||v.source_commit!=='650a546b8f8c07150d4df27f8de7cb1d4e13d4e8'||v.backend!=='webgl2'||v.profile_mode!=='owned'||v.scenario!=='renko-fifth'||v.checks.some(c=>!c.passed))throw Error('Provenance/check failure '+name);
 captures.push({name,bytes:bytes.length,sha256:sha(bytes),v});
}
captures.sort((a,b)=>a.v.captured_at.localeCompare(b.v.captured_at));
if(captures.length!==8)throw Error('Eight actual actions required');
const final=captures.at(-1).v;
if(JSON.stringify(final)!==JSON.stringify(summary.final))throw Error('Changed terminal summary');
for(const c of final.clients){
 if(c.phase!=='disposed'||c.mode!=='solo'||c.created!==2||c.closed!==2||c.requests||c.timers||!c.profile_equal||!c.persisted_equal||c.profile_sha!==c.baseline_sha||c.persisted_sha!==c.baseline_sha)throw Error('Terminal/save mismatch');
 if(c.fifth_shot.state!=='complete'||c.fifth_shot.ordinals.length!==10)throw Error('Incomplete cadence');
 for(const row of c.fifth_shot.ordinals){const bonus=row.ordinal===5||row.ordinal===10;if(!row.owner_matched||!row.epoch_matched||row.projectiles!==(bonus?2:1)||bonus&&(Math.abs(row.damage_ratio-1.5)>1e-6||Math.abs(row.speed_delta-70)>1e-6))throw Error('Cadence mismatch');}
}
if(final.clients[0].fifth_shot.primaries!==10||final.clients[1].fifth_shot.primaries!==0||!final.session_distinct||!final.checks.some(c=>c.name==='native-fifth-shot')||!final.checks.some(c=>c.name==='dispose-clean'))throw Error('Required coverage missing');
console.log(JSON.stringify({scope:'Read-only immutable evidence check; no browser/native/RTC rerun.',captures:captures.length,cumulativeChecks:final.checks.length,uniqueChecks:new Set(final.checks.map(c=>c.name)).size,primaries:10,bonusOrdinals:[5,10],disposed:true,savesUnchanged:true}));
