import test from 'node:test';
import assert from 'node:assert/strict';
import {pixelInputWindow} from '../../apps/game/src/pixel-ui.ts';

test('native glyph window keeps the entire encrypted join value outside geometry',()=>{
 const value='A'.repeat(512)+'B'.repeat(513);
 const first=pixelInputWindow(value,0),last=pixelInputWindow(value,value.length);
 assert.deepEqual(first,{text:'A'.repeat(256),caret:0});
 assert.deepEqual(last,{text:'B'.repeat(256),caret:256});
 assert.equal(value.length,1025);
 assert.equal(value.slice(512),'B'.repeat(513));
});
test('caret-centered window supports middle edits instead of showing a stale prefix',()=>{
 const value=Array.from({length:800},(_,i)=>String(i%10)).join('');
 const shown=pixelInputWindow(value,400);
 assert.equal(shown.text,value.slice(272,528));
 assert.equal(shown.caret,128);
});
test('native cursor coordinates count glyphs while DOM selections count UTF16',()=>{
 const value='🛰'.repeat(150)+'X'.repeat(150);
 const shown=pixelInputWindow(value,300);
 assert.equal(shown.caret,128);
 assert.equal(Array.from(shown.text).length,256);
 assert.equal(Array.from(shown.text)[127],'🛰');
 assert.equal(Array.from(shown.text)[128],'X');
});
test('absent and invalid selection uses the value end without truncating editable data',()=>{
 for(const caret of [null,Number.NaN,Number.POSITIVE_INFINITY]){
  assert.deepEqual(pixelInputWindow('JOIN',caret),{text:'JOIN',caret:4});
 }
 assert.deepEqual(pixelInputWindow('JOIN',-1),{text:'JOIN',caret:0});
 assert.deepEqual(pixelInputWindow('JOIN',100),{text:'JOIN',caret:4});
});
