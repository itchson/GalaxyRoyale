import test from 'node:test';
import assert from 'node:assert/strict';
import { FlightInput } from '../../apps/game/src/input.ts';

class Surface extends EventTarget {
  style = {}; classList = { add(){}, remove(){} };
  setPointerCapture() {}
  getBoundingClientRect() { return {left:0,top:0,width:100,height:100}; }
}
function pointer(target,type,props) {
  const event=new Event(type,{cancelable:true});
  Object.assign(event,{pointerId:1,pointerType:'touch',clientX:50,clientY:50,button:0,buttons:1,...props});
  target.dispatchEvent(event);
}
function setup() {
  globalThis.window=new EventTarget();
  const canvas=new Surface(), area=new Surface(), stick=new Surface(), fire=new Surface();
  let selected;const input=new FlightInput(canvas,area,stick,fire,world=>selected=world);
  input.enabled=true;input.camera=[100,200];input.viewport=[800,360];
  return {input,canvas,area,fire,selected:()=>selected};
}
test('simultaneous joystick and fire use independent pointer channels',()=>{
  const {input,area,fire}=setup();
  pointer(area,'pointerdown',{pointerId:3,clientX:84});
  pointer(fire,'pointerdown',{pointerId:7});
  assert.deepEqual(input.sample(),{joystick:[1,0],destination:null,fire:true});
  pointer(area,'pointercancel',{pointerId:3});
  assert.deepEqual(input.sample(),{joystick:null,destination:null,fire:true});
  pointer(fire,'pointerup',{pointerId:7});assert.equal(input.sample().fire,false);
});
test('blur resets every interrupted input, including mouse buttons',()=>{
  const {input,area,fire,canvas}=setup();
  pointer(area,'pointerdown',{clientX:70});pointer(fire,'pointerdown',{pointerId:2});
  pointer(canvas,'pointerdown',{pointerType:'mouse',button:2,buttons:2});
  window.dispatchEvent(new Event('blur'));
  assert.deepEqual(input.sample(),{joystick:null,destination:null,fire:false});
});
test('desktop destination maps from unshaken camera and is submitted once',()=>{
  const {input,canvas,selected}=setup();
  pointer(canvas,'pointerdown',{pointerType:'mouse',clientX:75,clientY:25});
  assert.deepEqual(selected(),[300,110]);
  assert.deepEqual(input.sample().destination,[300,110]);
  assert.equal(input.sample().destination,null);
  pointer(canvas,'pointermove',{pointerType:'mouse',clientX:90,clientY:50,buttons:3});
  assert.deepEqual(input.sample(),{joystick:null,destination:[420,200],fire:true});
  pointer(canvas,'pointercancel',{pointerType:'mouse'});
  assert.deepEqual(input.sample(),{joystick:null,destination:null,fire:false});
});
test('disabled or cancelled control cannot continue firing',()=>{
  const {input,fire}=setup();pointer(fire,'pointerdown',{});
  input.enabled=false;assert.equal(input.sample().fire,false);
  pointer(fire,'lostpointercapture',{});input.enabled=true;assert.equal(input.sample().fire,false);
});
test('right mouse fires and Space is not a weapon shortcut',()=>{
  const {input,canvas}=setup();
  const space=new Event('keydown',{cancelable:true}); Object.assign(space,{key:' '});window.dispatchEvent(space);
  assert.equal(input.sample().fire,false);
  pointer(canvas,'pointerdown',{pointerType:'mouse',button:2,buttons:2});assert.equal(input.sample().fire,true);
  pointer(canvas,'pointerup',{pointerType:'mouse',button:2,buttons:0});assert.equal(input.sample().fire,false);
});
