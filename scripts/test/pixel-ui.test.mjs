import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import ts from 'typescript';

const compiled=ts.transpileModule(readFileSync(new URL('../../apps/game/src/pixel-ui.ts',import.meta.url),'utf8'),{
  compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}
}).outputText;

function sliderScene({min='',max='',value='50'}={}) {
  const bounds={x:0,y:0,top:0,bottom:200,width:300,height:200};
  class Element {
    id='';dataset={};children=[];
    classList={contains:()=>false};
    getBoundingClientRect(){return bounds;}
    closest(){return null;}
    contains(){return false;}
    querySelectorAll(){return [];}
  }
  class Input extends Element {type='range';min=min;max=max;value=value;}
  const input=new Input();
  const label=new Element();label.childNodes=[{textContent:'Flight zoom'}];
  label.querySelector=()=>input;
  const root=new Element();
  root.querySelectorAll=selector=>selector==='.settings label'?[label]:[];
  const document={getElementById:()=>null,addEventListener(){},fonts:{ready:Promise.resolve()},
    querySelectorAll:()=>[root],activeElement:null,body:{classList:{contains:()=>false}}};
  const exports={};
  vm.runInNewContext(compiled,{exports,document,window:{addEventListener(){}},
    Element,HTMLElement:Element,HTMLInputElement:Input,HTMLButtonElement:class extends Element{},
    getComputedStyle:()=>({display:'block',fontSize:'8px'}),performance:{now:()=>0},
    MutationObserver:class {observe(){}}});
  let scene;
  const bridge=new exports.PixelUiBridge({set_ui_scene:value=>scene=value},()=>1);
  bridge.sync();
  const slider=scene.groups[0].nodes.find(node=>node.kind==='slider');
  assert.ok(slider,'actual semantic slider must reach the WASM scene');
  return slider;
}

test('zoom slider pixel thumb follows the semantic range at both limits and default zoom',()=>{
  for(const [value,expected]of [['0.6',0],['1',0.4],['1.6',1]]) {
    const slider=sliderScene({min:'0.6',max:'1.6',value});
    assert.ok(Math.abs(slider.value-expected)<1e-12);
    assert.equal(slider.text,'Flight zoom');
  }
});

test('volume and default HTML range bounds produce finite clamped pixel positions',()=>{
  assert.equal(sliderScene({min:'0',max:'1',value:'0.65'}).value,0.65);
  assert.equal(sliderScene({value:'50'}).value,0.5);
  assert.equal(sliderScene({min:'-10',max:'10',value:'0'}).value,0.5);
  assert.equal(sliderScene({min:'0.6',max:'1.6',value:'10'}).value,1);
  assert.equal(sliderScene({min:'1',max:'1',value:'1'}).value,0);
  assert.equal(sliderScene({min:'oops',max:'2',value:'1'}).value,0);
});
