/** Independent source-function counterexamples only; no browser/native/RTC/listener. */
import test from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import { createRequire } from 'node:module';
import { execFileSync } from 'node:child_process';
const author='C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale';
const require=createRequire(author+'/package.json');
const ts=require('typescript');
const source=execFileSync('git',['show','1b14bff9e6da0a1a249f1bdc42ab49e59e74e52c:scripts/qa/co-op-transport/wasm-review.js'],{cwd:author,encoding:'utf8'});
const ast=ts.createSourceFile('frozen-parent.js',source,ts.ScriptTarget.ES2022,true,ts.ScriptKind.JS);
const declaration=(name)=>{
 const node=ast.statements.find(n=>ts.isFunctionDeclaration(n)&&n.name?.text===name);
 assert(node,name);return node.getText(ast);
};
test('original continuity must require a commit beyond the post-policy reply floor',async()=>{
 const postPolicyFloor=5;
 const values=[{mode:'host',ready:1,admitted:1,same_room:true,same_peers:true,same_native:true},
  {mode:'guest',ready:1,admitted:1,new_commits:postPolicyFloor,owned_view:true,same_room:true,same_peers:true,same_native:true,same_epoch:true}];
 const context=vm.createContext({
  roomUntil:async predicate=>{if(!predicate(values))throw new Error('no_later_commit');return values;},
  check:(name,passed)=>{if(!passed)throw Error('ownership_failed');},
 });
 vm.runInContext(declaration('roomContinuity'),context);
 // All observed commits already existed at the post-reply floor. Acceptance
 // here is the original source defect, not a simulated RTC pass.
 await assert.rejects(context.roomContinuity(1000),/no_later_commit/);
});
test('original roomUntil must reject visibility lost during the awaited observation',async()=>{
 let visible=true;
 const context=vm.createContext({
  requireVisibleClients:()=>{if(!visible)throw Error('page_not_visible');},
  roomRead:async()=>{visible=false;return [{mode:'host'},{mode:'solo'}];},
  performance:{now:()=>0},setTimeout,
 });
 vm.runInContext(declaration('roomUntil'),context);
 await assert.rejects(context.roomUntil(()=>true,1000),/page_not_visible/);
});
