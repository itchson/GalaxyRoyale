import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer as createNetServer} from 'node:net';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {mkdtemp,writeFile,mkdir,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve,dirname,basename} from 'node:path';
import {createServer,preview,createLogger} from 'vite';
import {acquireGameLease,singleGameRuntime} from '../game-runtime-lease.mjs';

const exec=promisify(execFile);
async function availablePort(){
  const server=createNetServer();
  await new Promise((yes,no)=>{server.once('error',no);server.listen({host:'127.0.0.1',port:0},yes);});
  const port=server.address().port;
  await new Promise((yes,no)=>server.close(error=>error?no(error):yes()));
  return port;
}
async function fixture(t){
  const root=resolve(await mkdtemp(join(tmpdir(),'galaxy-vite-lifecycle-')));
  t.after(async()=>{
    // Delete only the explicitly owned temporary fixture, never a checkout.
    assert.equal(dirname(root),resolve(tmpdir()));
    assert.ok(basename(root).startsWith('galaxy-vite-lifecycle-'));
    await rm(root,{recursive:true,force:true});
  });
  const port=await availablePort();
  let httpPort;do{httpPort=await availablePort();}while(httpPort===port);
  const errors=[];
  const logger=createLogger('silent');logger.error=message=>errors.push(message);
  return {root,port,httpPort,errors,options(plugin){return {
    configFile:false,root,plugins:[plugin],customLogger:logger,logLevel:'silent',appType:'custom',
    server:{host:'127.0.0.1',port:httpPort,strictPort:true,hmr:false,watch:null},
  };}};
}
async function canReserve(port){const lease=await acquireGameLease(port);await lease.close();}
async function childCannotReserve(port){
  const module=new URL('../game-runtime-lease.mjs',import.meta.url).href;
  const code=`import {acquireGameLease} from ${JSON.stringify(module)};
    try {const lease=await acquireGameLease(${port});await lease.close();process.exitCode=0;}
    catch(error){process.exitCode=error.cause?.code==='EADDRINUSE'?2:3;}`;
  await assert.rejects(exec(process.execPath,['--input-type=module','-e',code],{timeout:3000,windowsHide:true}),error=>error.code===2);
}

test('actual Vite never-listened close is idempotent and releases its lease',{timeout:15000},async t=>{
  const f=await fixture(t);
  const server=await createServer(f.options(singleGameRuntime(f.port)));t.after(()=>server.close());
  await assert.rejects(canReserve(f.port),/already reserved/);
  await Promise.all([server.close(),server.close()]);
  await canReserve(f.port);
});

test('actual Vite restarts twice without releasing its cross-process reservation',{timeout:15000},async t=>{
  const f=await fixture(t);
  const server=await createServer(f.options(singleGameRuntime(f.port)));t.after(()=>server.close());
  await server.listen();
  for(let i=0;i<2;i++){
    await server.restart();
    assert.equal(server.httpServer.listening,true);
    await assert.rejects(canReserve(f.port),/already reserved/);
  }
  await childCannotReserve(f.port);
  assert.deepEqual(f.errors,[]);
  await server.close();await canReserve(f.port);
});

test('rebundled Vite plugin owners keep the lease until their final explicit close',{timeout:15000},async t=>{
  const f=await fixture(t);
  const url=new URL('../game-runtime-lease.mjs',import.meta.url);url.search='lifecycle-rebundle=1';
  const rebundled=await import(url);
  const first=await createServer(f.options(singleGameRuntime(f.port)));t.after(()=>first.close());await first.listen();
  const next=await createServer(f.options(rebundled.singleGameRuntime(f.port)));t.after(()=>next.close());
  await first.close();await assert.rejects(canReserve(f.port),/already reserved/);
  await next.close();await canReserve(f.port);
});

test('actual Vite preview close releases its lease',{timeout:15000},async t=>{
  const f=await fixture(t);const dist=join(f.root,'dist');await mkdir(dist);await writeFile(join(dist,'index.html'),'owned lifecycle fixture');
  const server=await preview({configFile:false,root:f.root,plugins:[singleGameRuntime(f.port)],logLevel:'silent',build:{outDir:dist},preview:{host:'127.0.0.1',port:f.httpPort,strictPort:true}});t.after(()=>server.close());
  await assert.rejects(canReserve(f.port),/already reserved/);
  await Promise.all([server.close(),server.close()]);await canReserve(f.port);
});

test('failed Vite preview listen releases the lease and preserves the occupied port owner',{timeout:15000},async t=>{
  const f=await fixture(t);
  const blocker=createNetServer();
  await new Promise((yes,no)=>{blocker.once('error',no);blocker.listen({host:'127.0.0.1',port:f.httpPort},yes);});
  t.after(()=>new Promise(resolve=>blocker.close(resolve)));
  const dist=join(f.root,'dist');await mkdir(dist);await writeFile(join(dist,'index.html'),'owned lifecycle fixture');
  await assert.rejects(preview({configFile:false,root:f.root,plugins:[singleGameRuntime(f.port)],logLevel:'silent',build:{outDir:dist},preview:{host:'127.0.0.1',port:f.httpPort,strictPort:true}}),/already in use/);
  await new Promise(resolve=>setImmediate(resolve));
  assert.equal(blocker.listening,true);
  await canReserve(f.port);
});
