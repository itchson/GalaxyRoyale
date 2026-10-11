import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {treeHash} from '../preview-build.mjs';
import {isProjectGameProcess,isRelativeGameProcess} from '../preview-game.mjs';
test('owned game listener detection excludes other projects, Studio and QA tools',()=>{
 const root='C:/games/royale',vite='"C:\\games\\royale\\node_modules\\vite\\bin\\vite.js"';
 assert.equal(isProjectGameProcess('node '+vite+' preview --config apps/game/vite.config.ts --port 4210',root),true);
 assert.equal(isProjectGameProcess('node '+vite+' --config apps/studio/vite.config.ts',root),false);
 assert.equal(isProjectGameProcess('node "C:/games/other/node_modules/vite/bin/vite.js" --config apps/game/vite.config.ts',root),false);
 assert.equal(isProjectGameProcess('node C:/games/royale/work/audio-review/server.mjs',root),false);
 assert.equal(isProjectGameProcess('node node_modules/vite/bin/vite.js preview --config apps/game/vite.config.ts --outDir C:/games/royale/work/baseline-dist --port 4190',root),true);
 assert.equal(isRelativeGameProcess('node node_modules/vite/bin/vite.js --config apps/game/vite.config.ts'),true);
 assert.equal(isRelativeGameProcess('node C:/games/other/node_modules/vite/bin/vite.js --config apps/game/vite.config.ts'),false);
});
test('build fingerprint catches cell source and packaged output edits; excluded generated inputs do not affect source',async t=>{
 const root=await mkdtemp(join(tmpdir(),'galaxy-preview-'));t.after(()=>rm(root,{recursive:true,force:true}));
 await mkdir(join(root,'source'));await writeFile(join(root,'source/a'),'pixel-body');await writeFile(join(root,'source/generated'),'cached');
 const hash=()=>treeHash(root,['source'],name=>name==='source/generated');const original=await hash();
 await writeFile(join(root,'source/generated'),'new-cache');assert.equal(await hash(),original);
 await writeFile(join(root,'source/a'),'changed-pixel-body');assert.notEqual(await hash(),original);
});
