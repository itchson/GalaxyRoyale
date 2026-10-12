import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer as createNetServer} from 'node:net';
import {mkdtemp,readFile,writeFile,mkdir,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createServer,preview,createLogger} from 'file:///C:/Users/mitch/Documents/ChatGPT/galaxy-royale/node_modules/vite/dist/node/index.js';

const base=import.meta.dirname;
const repair=await import(pathToFileURL(join(base,'repair-source/scripts/game-runtime-lease.mjs')));
async function availablePort(){
 const server=createNetServer();await new Promise((yes,no)=>{server.once('error',no);server.listen({host:'127.0.0.1',port:0},yes);});
 const port=server.address().port;await new Promise((yes,no)=>server.close(error=>error?no(error):yes()));return port;
}
async function baseline(port){
 const source=await readFile(join(base,'baseline-source/scripts/game-runtime-lease.mjs'),'utf8');
 // Exact committed baseline hook; only its fixed production port is changed for isolation.
 return import('data:text/javascript;base64,'+Buffer.from(source.replace('GAME_LEASE_PORT = 4172','GAME_LEASE_PORT = '+port)).toString('base64'));
}
async function fixture(t){
 const root=await mkdtemp(join(tmpdir(),'galaxy-vite-review-'));t.after(()=>rm(root,{recursive:true,force:true}));
 const port=await availablePort();let httpPort;do{httpPort=await availablePort();}while(httpPort===port);
 const errors=[];const logger=createLogger('silent');logger.error=message=>errors.push(message);
 return {root,port,httpPort,errors,options(plugin){return {configFile:false,root,plugins:[plugin],customLogger:logger,logLevel:'silent',appType:'custom',server:{host:'127.0.0.1',port:httpPort,strictPort:true,hmr:false,watch:null}};}};
}
async function canReserve(port){const lease=await repair.acquireGameLease(port);await lease.close();}
async function cleanupOriginalHttp(server){await new Promise(resolve=>server.httpServer.close(()=>resolve()));await new Promise(resolve=>setImmediate(resolve));}

test('baseline actual Vite close before listen leaks the HTTP-event-only reservation',{timeout:15000},async t=>{
 const f=await fixture(t);const original=await baseline(f.port);const server=await createServer(f.options(original.singleGameRuntime()));
 t.after(async()=>{await server.close();await cleanupOriginalHttp(server);});
 await server.close();await assert.rejects(canReserve(f.port),/already reserved/);
 console.log('BASELINE_CONFIRMED never-listened Vite.close did not release lease');
 await cleanupOriginalHttp(server);await canReserve(f.port);
});
test('baseline actual Vite restart conflicts with its still-open old reservation',{timeout:15000},async t=>{
 const f=await fixture(t);const original=await baseline(f.port);const server=await createServer(f.options(original.singleGameRuntime()));t.after(()=>server.close());
 await server.listen();await server.restart();
 assert.ok(f.errors.some(x=>x.includes('already reserved')));assert.ok(f.errors.some(x=>x.includes('server restart failed')));
 assert.ok(server.httpServer.listening,'old server remains active after failed restart');
 console.log('BASELINE_CONFIRMED replacement configure runs before old.close');
 await server.close();await canReserve(f.port);
});
test('repair actual Vite never-listened close is idempotent and releases lease',{timeout:15000},async t=>{
 const f=await fixture(t);const server=await createServer(f.options(repair.singleGameRuntime(f.port)));t.after(()=>server.close());
 await assert.rejects(canReserve(f.port),/already reserved/);
 await Promise.all([server.close(),server.close()]);await canReserve(f.port);
});
test('repair actual Vite performs two restarts and retains cross-process OS exclusion',{timeout:15000},async t=>{
 const f=await fixture(t);const server=await createServer(f.options(repair.singleGameRuntime(f.port)));t.after(()=>server.close());await server.listen();
 for(let i=0;i<2;i++){await server.restart();assert.equal(server.httpServer.listening,true);await assert.rejects(canReserve(f.port),/already reserved/);}
 assert.deepEqual(f.errors,[]);await server.close();await canReserve(f.port);
});
test('repair independently imported plugin owners share until final explicit close',{timeout:15000},async t=>{
 const f=await fixture(t);const url=pathToFileURL(join(base,'repair-source/scripts/game-runtime-lease.mjs'));url.search='rebundle=1';const rebundled=await import(url);
 const first=await createServer(f.options(repair.singleGameRuntime(f.port)));t.after(()=>first.close());await first.listen();
 const next=await createServer(f.options(rebundled.singleGameRuntime(f.port)));t.after(()=>next.close());
 await first.close();await assert.rejects(canReserve(f.port),/already reserved/);
 await next.close();await canReserve(f.port);
});
test('repair actual preview close releases reservation',{timeout:15000},async t=>{
 const f=await fixture(t);const dist=join(f.root,'dist');await mkdir(dist);await writeFile(join(dist,'index.html'),'owned review fixture');
 const server=await preview({configFile:false,root:f.root,plugins:[repair.singleGameRuntime(f.port)],logLevel:'silent',build:{outDir:dist},preview:{host:'127.0.0.1',port:f.httpPort,strictPort:true}});t.after(()=>server.close());
 await assert.rejects(canReserve(f.port),/already reserved/);await Promise.all([server.close(),server.close()]);await canReserve(f.port);
});
test('repair failed preview HTTP listen releases lease without claiming the occupied owner',{timeout:15000},async t=>{
 const f=await fixture(t);const blocker=createNetServer();await new Promise((yes,no)=>{blocker.once('error',no);blocker.listen({host:'127.0.0.1',port:f.httpPort},yes);});t.after(()=>new Promise(resolve=>blocker.close(resolve)));
 const dist=join(f.root,'dist');await mkdir(dist);await writeFile(join(dist,'index.html'),'owned review fixture');
 await assert.rejects(preview({configFile:false,root:f.root,plugins:[repair.singleGameRuntime(f.port)],logLevel:'silent',build:{outDir:dist},preview:{host:'127.0.0.1',port:f.httpPort,strictPort:true}}),/already in use/);
 await new Promise(resolve=>setImmediate(resolve));assert.equal(blocker.listening,true);await canReserve(f.port);
});
