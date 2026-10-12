import {readFile,writeFile,readdir,mkdir} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {buildIdentity,treeHash} from '../../scripts/preview-build.mjs';
const root=process.cwd(),out=path.join(root,'artifacts/native-wake-integration-20261013'),pins={};
await mkdir(out,{recursive:true});
const sha=b=>createHash('sha256').update(b).digest('hex');
async function copy(source,name){const b=await readFile(source);await mkdir(path.dirname(path.join(out,name)),{recursive:true});await writeFile(path.join(out,name),b);pins[name]={bytes:b.length,sha256:sha(b)};}
async function tree(source,prefix){for(const entry of await readdir(source,{withFileTypes:true})){if(entry.isSymbolicLink())throw Error('Linked evidence');const name=prefix+'/'+entry.name,src=path.join(source,entry.name);if(entry.isDirectory())await tree(src,name);else if(entry.isFile())await copy(src,name);}}
await tree(path.join(root,'work/native-wake-integration-20261013'),'root-checks');
const art='C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale/work';
const fx='C:/Users/mitch/.codex/worktrees/galaxy-hud-navigation/galaxy-royale/work';
for(const [src,prefix] of [[art+'/coop-ion-wake','native-author'],[fx+'/ion-wake-review','native-independent-review'],[fx+'/hub-shadow-next','shadow-author'],[fx+'/boxed-world-review','boxed-author'],[art+'/boxed-world-independent-review','boxed-independent-review'],[fx+'/ion-wake-browser-boundary-review','browser-boundary-review'],[art+'/coop-wake-presentation-design','trail-design'],[fx+'/coop-wake-presentation-review','trail-design-review']])await tree(src,prefix);
for(const file of ['apps/game/src/coop/wire.ts','crates/game_core/src/co_op_wake.rs','crates/game_core/src/co_op_wake_tests.rs','crates/game_core/src/co_op_tests.rs','crates/game_web/src/dock_art_tests.rs','scripts/test/coop-wake-loadout.test.mjs'])await copy(path.join(root,file),'integrated-source/'+file);
await tree(path.join(root,'scripts/test/fixtures/co-op-wake-loadout'),'native-fixtures');
const native=await readFile('work/native-wake-integration-20261013/native-tests-final.log','utf8');
const results=[...native.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored/g)];
const sum=n=>results.reduce((total,m)=>total+Number(m[n]),0);
if(sum(1)!==914||sum(2)!==0||sum(3)!==17)throw Error('Native result disagreement');
const shell=await readFile('work/native-wake-integration-20261013/shell-final.log','utf8');
if(!shell.includes('pass 657')||!shell.includes('fail 0')||!shell.includes('skipped 1'))throw Error('Shell result disagreement');
const receipt=JSON.parse(await readFile('work/preview-build.json','utf8')),identity=await buildIdentity(root),packageHash=await treeHash(root,['dist']);
if(JSON.stringify(identity)!==JSON.stringify({head:receipt.head,branch:receipt.branch,sourceHash:receipt.sourceHash})||receipt.packageHash!==packageHash)throw Error('Build provenance drift');
await writeFile(path.join(out,'.gitattributes'),'* -text\n');
await writeFile(path.join(out,'verification.json'),JSON.stringify({scope:'Actual Root integrated native and browser-boundary/build checks; not full co-op trail presentation or RTC/GPU shadow acceptance',identity,receipt,native:{pass:sum(1),fail:sum(2),ignored:sum(3)},shell:{pass:657,fail:0,skip:1,scope:'Before documentation-only Wiki update;20 Wiki cases rechecked afterward'},wiki:{pass:20,fail:0},boundary:{pass:21,scope:'Actual Node validators/ports with native-derived values; fixture co_op_batch envelope'},headlessWasm:{cases:50,indexedDb:'fake-indexeddb',qaProbeExcludedFromRelease:true,scope:'Isolated serializer/SaveStore builds; no browser/GPU'},clippy:['native workspace/all-targets','WebGPU WASM','WebGL2 WASM'],typecheck:0,fmt:0,pins},null,2)+'\n');
console.log(JSON.stringify({out,files:Object.keys(pins).length,identity}));
