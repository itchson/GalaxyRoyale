import fs from 'node:fs/promises';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import ts from '../../../node_modules/typescript/lib/typescript.js';

// Exact frozen function, explicit API doubles. No browser/WASM/RTC execution.
const source = await fs.readFile(new URL('./source/scripts/qa/co-op-transport/wasm-review.js', import.meta.url), 'utf8');
const parsed = ts.createSourceFile('frozen-parent.js', source, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
const fn = parsed.statements.find(s => ts.isFunctionDeclaration(s) && s.name?.text === 'wakeRound');
assert.ok(fn);
const checks = [], calls = [];
const state = {
  host: {mode:'host', phase:'hosting', failure_code:null, wake:{captured_host:true,captured_guest:true,native_render_frames:10}},
  guest: {mode:'guest', phase:'connected', failure_code:null, wake:{captured_host:true,captured_guest:true,native_render_frames:10}},
};
const context = vm.createContext({
  scenario:'configured-ion-wake-r1-e1', state, driveActive:false, driveVisibilityLost:false,
  requireVisibleClients(){}, guardDriveVisibility(){},
  async send(role, name){calls.push({role,name});return state[role];},
  async until(predicate){assert.equal(predicate(),true);},
  async verify(){
    // The declared reply state after a real frame failure/retirement while the
    // final save read is awaited. This reproduces the gate, not the native fault.
    Object.assign(state.guest, {failure_code:'wake_native_frame',phase:'failed',mode:'solo'});
    Object.assign(state.guest.wake,{state:'inconclusive',reason:'wake_native_frame'});
  },
  check(name, passed){checks.push({name,passed});},
});
vm.runInContext(fn.getText(parsed),context);
let outcome;
try { await context.wakeRound('drive');outcome='resolved'; }
catch(error){outcome=error.message;}
console.log(JSON.stringify({scope:'frozen actual wakeRound with declared native-failure reply; no listener/native/RTC execution',outcome,checks,calls,guest_after:state.guest},null,2));
assert.notEqual(outcome,'resolved','A current native-frame failure during final verification must reject the Wake phase and invoke failed cleanup before recording success.');
