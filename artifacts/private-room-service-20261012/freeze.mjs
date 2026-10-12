import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
const sourceCommit='eeccc79d630b4d17533ae53f41b8c7dbb835b8fb';
const dir=path.resolve('work/short-room-service-live-codes');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
if(execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim()!==sourceCommit)throw new Error('Do not freeze another revision');
const source=['apps/coop/store.mjs','apps/coop/server.mjs','apps/coop/README.md','scripts/test/coop-signalling.test.mjs','scripts/test/coop-private-rooms.test.mjs'].map(p=>{
 const b=execFileSync('git',['show',`${sourceCommit}:${p}`],{maxBuffer:8_000_000});return{path:p,bytes:b.length,sha256:sha(b)};
});
const priorEvidence=['work/short-room-service/HANDOFF.md','work/short-room-service/manifest.json'].map(p=>{const b=fs.readFileSync(p);return{path:p,bytes:b.length,sha256:sha(b)};});
const files=fs.readdirSync(dir).sort().filter(n=>n!=='manifest.json'&&fs.statSync(path.join(dir,n)).isFile()).map(n=>{const b=fs.readFileSync(path.join(dir,n));return{path:n,bytes:b.length,sha256:sha(b)};});
fs.writeFileSync(path.join(dir,'manifest.json'),JSON.stringify({version:1,sourceCommit,parentCommit:'a7d45d84760d55a66535a6a98d3f5e2ba81fe7b7',sourceBase:'d2acfd2d9faaec2810d67d62f212066e7f1b5b81',scope:'Final live-room code lifetime without independent invite TTL; short-room service/approval/policy/lifecycle actual scoped tests. No RTC/native/TURN/deployment/browser claim.',source,priorEvidence,files},null,2)+'\n');
process.stdout.write(JSON.stringify({sourceCommit,sourcePins:source.length,evidencePins:files.length,priorEvidencePins:priorEvidence.length,handoffSha256:sha(fs.readFileSync(path.join(dir,'HANDOFF.md'))),manifestSha256:sha(fs.readFileSync(path.join(dir,'manifest.json')))}));
