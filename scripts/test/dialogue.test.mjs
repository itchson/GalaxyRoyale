import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import ts from 'typescript';
const source=readFileSync(new URL('../../apps/game/src/dialogue.ts',import.meta.url),'utf8').replace('import "./dialogue.css";','');
const compiled=ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
function fixture() {
  let document;const listeners=new Map();
  class Element {
    children=[];dataset={};hidden=false;disabled=false;attributes=new Map();events=new Map();isConnected=true;
    append(...children){this.children.push(...children);children.forEach(c=>c.parent=this);}
    setAttribute(k,v){this.attributes.set(k,v);} removeAttribute(k){this.attributes.delete(k);}
    addEventListener(name,callback){this.events.set(name,callback);} focus(){document.activeElement=this;}
    remove(){this.isConnected=false;}
  }
  const find=(root,id)=>root.id===id?root:root.children.map(child=>find(child,id)).find(Boolean);
  document={body:new Element(),activeElement:null,createElement:()=>new Element(),getElementById:id=>find(document.body,id)};
  document.activeElement=document.body;
  const exports={};vm.runInNewContext(compiled,{exports,document,HTMLElement:Element,window:{addEventListener:(n,c)=>listeners.set(n,c),removeEventListener:n=>listeners.delete(n)}});
  let opened=0,closed=0;const panel=new exports.DialoguePanel({onOpen:()=>opened++,onClose:()=>closed++});
  return {panel,document,element:id=>find(document.body,id),key:key=>listeners.get('keydown')({key,preventDefault(){},stopImmediatePropagation(){}}),opened:()=>opened,closed:()=>closed};
}
test('paged dialogue exposes navigation and voice controls without leaking modal focus',()=>{
  const f=fixture();let next=0,replay=0;
  f.panel.show({speaker:'LYRA',text:'Read the brief.',portrait:7,page:0,pageCount:3,onNext:()=>next++,onReplay:()=>replay++});
  assert.equal(f.opened(),1);assert.equal(f.element('dialogue-action').hidden,true);assert.equal(f.element('dialogue-previous').hidden,true);
  assert.equal(f.element('dialogue-next').textContent,'NEXT 1/3');
  f.element('dialogue-next').events.get('click')(); f.element('dialogue-replay').events.get('click')();
  assert.equal(next,1);assert.equal(replay,1);assert.equal(f.document.activeElement,f.element('dialogue-next'));
  f.key('Tab'); assert.equal(f.document.activeElement,f.element('dialogue-replay'));
  f.key('Tab'); assert.equal(f.document.activeElement,f.element('dialogue-close'));
  f.key('Tab'); assert.equal(f.document.activeElement,f.element('dialogue-next'));
  f.key('Escape');assert.equal(f.panel.opened,false);assert.equal(f.closed(),1);
});
test('a late asynchronous action cannot duplicate a reward or overwrite a newer dialogue page',async()=>{
  const f=fixture();let finish,calls=0;
  f.panel.show({speaker:'LYRA',text:'Challenge ready.',onAction:()=>{calls++;return new Promise((_,reject)=>finish=reject);}});
  const first=f.element('dialogue-action').events.get('click')();
  await f.element('dialogue-action').events.get('click')();assert.equal(calls,1);
  f.panel.show({speaker:'KORR',text:'New page.',onNext:()=>{}});
  finish(new Error('obsolete failure'));await first;
  assert.equal(f.element('dialogue-text').textContent,'New page.');assert.equal(f.element('dialogue-action').hidden,true);
  f.panel.close();f.panel.close();assert.equal(f.closed(),1);f.panel.dispose();assert.equal(f.panel.opened,false);
});

test('presentation tokens change on refresh and close, without changing on reads',()=>{
  const f=fixture();const before=f.panel.presentationToken;
  f.panel.show({speaker:'CONTACT',text:'Page one.'});
  const first=f.panel.presentationToken;
  assert.ok(first>before);assert.equal(f.panel.presentationToken,first);
  f.panel.show({speaker:'CONTACT',text:'Page two.'});
  const second=f.panel.presentationToken;assert.ok(second>first);
  f.panel.close();const closed=f.panel.presentationToken;assert.ok(closed>second);
  f.panel.close();assert.equal(f.panel.presentationToken,closed);
});
