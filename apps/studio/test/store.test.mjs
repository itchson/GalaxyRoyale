import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, dirname, resolve, basename } from 'node:path';
import { ProjectStore, artifactPath } from '../store.mjs';
import { makeServer } from '../server.mjs';

const initial = () => ({schemaVersion:1,revision:1,updatedAt:new Date().toISOString(),tasks:[{id:'one',title:'Validate controller',description:'Meaningful fixture',department:'QA',owner:'Tester',status:'active',priority:'normal',dependencies:[],acceptanceCriteria:[{text:'Input cancellation clears thrust',met:false}],evidence:[],blocker:'',revision:1}],reviews:[],decisions:[],builds:[],feedback:[],activity:[]});
async function fixture(t) {
  const root=await mkdtemp(join(tmpdir(),'galaxy-studio-'));
  assert.equal(resolve(dirname(root)), resolve(tmpdir()));
  assert.ok(basename(root).startsWith('galaxy-studio-'));
  await mkdir(join(root,'work'));await writeFile(join(root,'work','result.log'),'controller cancellation validated\n');
  const file=join(root,'project.json');await writeFile(file,JSON.stringify(initial()));
  const store=await new ProjectStore(file,root).load();
  t.after(()=>rm(root,{recursive:true,force:true}));
  return {root,file,store};
}
const update = patch => ({id:'one',expectedRevision:1,author:'QA',source:'Automated fixture',patch});

test('serialized writes reject stale revisions without losing accepted state; restart and backup preserve documents',async t=>{
  const {file,root,store}=await fixture(t);
  const results=await Promise.allSettled([store.mutate('task.update',update({title:'Reviewed controller'})),store.mutate('task.update',update({title:'Stale change'}))]);
  assert.equal(results[0].status,'fulfilled');assert.equal(results[1].status,'rejected');assert.equal(results[1].reason.status,409);
  const loaded=await new ProjectStore(file,root).load();assert.equal(loaded.project.tasks[0].title,'Reviewed controller');assert.equal(loaded.project.revision,2);assert.equal(loaded.project.activity.length,1);
  const backup=JSON.parse(await readFile(`${file}.backup`,'utf8'));assert.equal(backup.tasks[0].title,'Validate controller');assert.equal(backup.revision,1);
});

test('completion needs actual criteria and evidence, including existing artifacts',async t=>{
  const {store}=await fixture(t);
  await assert.rejects(store.mutate('task.update',update({status:'complete'})),/acceptance criteria/);
  await assert.rejects(store.mutate('task.update',update({status:'complete',acceptanceCriteria:[{text:'Input cancellation clears thrust',met:true}]})),/evidence/);
  await assert.rejects(store.mutate('task.update',update({status:'complete',acceptanceCriteria:[{text:'Input cancellation clears thrust',met:true}],evidence:[{label:'Results',path:'work/missing.log'}]})),/does not exist/);
  assert.equal(store.project.revision,1);
  await store.mutate('task.update',update({status:'complete',acceptanceCriteria:[{text:'Input cancellation clears thrust',met:true}],evidence:[{label:'Controller results',path:'work/result.log'}]}));
  assert.equal(store.project.tasks[0].status,'complete');assert.equal(store.project.tasks[0].revision,2);
});

test('artifact allowlist rejects traversal, secrets, missing files and directory requests',async t=>{
  const {root}=await fixture(t);await writeFile(join(root,'.env'),'fixture only');
  assert.equal(await artifactPath(root,'work/result.log'),join(root,'work','result.log'));
  await assert.rejects(artifactPath(root,'work/../.env'),/inside/);
  await assert.rejects(artifactPath(root,'../outside.log'),/inside/);
  await assert.rejects(artifactPath(root,'work'),/inside/);
  await assert.rejects(artifactPath(root,'work/not-created.log'),/does not exist/);
});

test('HTTP API returns stale-edit 409 and blocks foreign origins and non-JSON mutations',async t=>{
  const {store}=await fixture(t);const server=makeServer(store);
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));t.after(()=>new Promise(resolve=>server.close(resolve)));
  const base=`http://127.0.0.1:${server.address().port}`;
  const options={method:'PATCH',headers:{'Content-Type':'application/json'},body:JSON.stringify(update({title:'Saved via HTTP'}))};
  assert.equal((await fetch(base+'/api/tasks/one',options)).status,200);
  const stale=await fetch(base+'/api/tasks/one',options);assert.equal(stale.status,409);assert.match((await stale.json()).error,/revision/);
  assert.equal((await fetch(base+'/api/tasks/one',{...options,headers:{...options.headers,Origin:'https://foreign.example'}})).status,403);
  assert.equal((await fetch(base+'/api/tasks/one',{...options,headers:{'Content-Type':'text/plain'}})).status,415);
  assert.equal((await fetch(base+'/artifacts?path=work%2F..%2F.env')).status,400);
  assert.equal((await fetch(base+'/artifacts?path=work%2Fresult.log')).status,200);
  const snapshot=await (await fetch(base+'/api/project')).json();assert.equal(snapshot.tasks[0].title,'Saved via HTTP');
});

test('dependency cycles are rejected and feedback is queued until explicitly acknowledged or resolved',async t=>{
  const {store}=await fixture(t);
  const created=await store.mutate('task.create',{author:'QA',source:'Automated fixture',task:{title:'Follow-up',dependencies:['one']}});
  await assert.rejects(store.mutate('task.update',update({dependencies:[created.item.id]})),/cycle/);
  const feedback=await store.mutate('feedback',{author:'User',source:'Automated fixture',record:{title:'Controller note',body:'Record cancellation issue.'}});
  assert.equal(feedback.item.status,'queued');
  await assert.rejects(store.mutate('feedback.update',{id:feedback.item.id,expectedRevision:1,author:'Producer',source:'Automated fixture',status:'resolved'}),/response/);
  const resolved=await store.mutate('feedback.update',{id:feedback.item.id,expectedRevision:1,author:'Producer',source:'Automated fixture',status:'resolved',response:'Linked to controller validation task.'});
  assert.equal(resolved.item.status,'resolved');assert.equal(resolved.item.revision,2);
});

const runRecord = overrides => ({author:'Producer',source:'Actual fixture observation',record:{agentId:'/root/qa',agentLabel:'QA specialist',state:'running',observedAt:new Date().toISOString(),taskIds:['one'],summary:'Examining the controller cancellation behavior.',evidence:[{label:'Assignment observation',details:'Producer assigned controller QA in an actual session exchange.'}],...overrides}});

test('run records persist as append-only observations without inferring task completion or agent presence',async t=>{
  const {store,file,root}=await fixture(t);
  assert.deepEqual(store.snapshot().runRecords,[],'older snapshots load with empty run history');
  const first=await store.mutate('run',runRecord());
  const second=await store.mutate('run',runRecord({state:'completed',summary:'Controller cancellation QA finished.',evidence:[{label:'QA output',path:'work/result.log'}]}));
  assert.equal(store.project.runRecords.length,2);assert.notEqual(first.item.id,second.item.id);
  assert.equal(store.project.runRecords[0].state,'running');assert.equal(store.project.runRecords[1].state,'completed');
  assert.equal(store.project.tasks[0].status,'active','run completion does not complete its tasks');
  assert.equal(store.project.activity.at(-1).kind,'run');
  const restarted=await new ProjectStore(file,root).load();assert.deepEqual(restarted.snapshot().runRecords,store.snapshot().runRecords);
  await assert.rejects(store.mutate('run.update',{author:'QA',source:'Automated fixture'}),/Unknown operation/);
});

test('run observations validate state, identity, timestamp, task references and artifact evidence',async t=>{
  const {store}=await fixture(t);
  for(const invalid of [
    {state:'live'}, {agentId:'QA agent'}, {agentId:''}, {agentLabel:''},
    {observedAt:'yesterday'}, {observedAt:'2026-02-30T00:00:00Z'}, {observedAt:new Date(Date.now()+60000).toISOString()},
    {taskIds:['missing']}, {taskIds:['one','one']}, {taskIds:[]}, {evidence:[]},
    {evidence:[{label:'Missing output',path:'work/not-created.log'}]}
  ])await assert.rejects(store.mutate('run',runRecord(invalid)),error=>error.status===400 || error.status===404);
  assert.equal(store.project.revision,1);assert.equal(store.project.runRecords.length,0);
  await store.mutate('run',runRecord({state:'interrupted',summary:'Worker returned a process failure; scope is unfinished.'}));
  assert.equal(store.project.runRecords[0].state,'interrupted');
});

test('HTTP run endpoint appends observations and rejects edit attempts',async t=>{
  const {store}=await fixture(t);const server=makeServer(store);
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));t.after(()=>new Promise(resolve=>server.close(resolve)));
  const base=`http://127.0.0.1:${server.address().port}`;
  const result=await fetch(base+'/api/runs',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(runRecord())});
  assert.equal(result.status,201);const {item}=await result.json();assert.equal(item.agentId,'/root/qa');
  assert.equal((await fetch(base+'/api/runs/'+item.id,{method:'PATCH',headers:{'Content-Type':'application/json'},body:JSON.stringify(runRecord({state:'completed'}))})).status,404);
  const snapshot=await (await fetch(base+'/api/project')).json();assert.equal(snapshot.runRecords.length,1);assert.equal(snapshot.runRecords[0].state,'running');
});

test('review, decision and build records retain their source and explicit check coverage after restart',async t=>{
  const {store,file,root}=await fixture(t);const context={author:'QA',source:'Automated isolated record-contract fixture'};
  await store.mutate('review',{...context,record:{title:'Actual fixture exchange',body:'QA reported test evidence; Producer requested a follow-up.',participants:['QA','Producer'],taskIds:['one']}});
  await store.mutate('decision',{...context,record:{title:'Fixture decision',body:'Use the verified controller behavior.',status:'accepted'}});
  await assert.rejects(store.mutate('build',{...context,record:{title:'Unsupported check',body:'Fixture only.',revision:'fixture',checks:[{name:'Controller',result:'probably_pass'}]}}),/explicitly/);
  await store.mutate('build',{...context,record:{title:'Fixture validation',body:'Record the checks that actually ran in this isolated fixture.',revision:'fixture',checks:[{name:'Controller fixture',result:'pass'},{name:'Phone acceptance',result:'not_run',details:'No physical device is present.'}],evidence:[{label:'Fixture output',path:'work/result.log'}],knownIssues:'Physical phone acceptance remains untested.'}});
  const restarted=await new ProjectStore(file,root).load();assert.equal(restarted.project.reviews[0].source,context.source);assert.deepEqual(restarted.project.reviews[0].participants,['QA','Producer']);assert.equal(restarted.project.decisions[0].status,'accepted');assert.equal(restarted.project.builds[0].checks[1].result,'not_run');assert.equal(restarted.project.builds[0].evidence[0].path,'work/result.log');
});
