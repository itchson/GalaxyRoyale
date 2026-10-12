import fs from 'node:fs/promises';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import ts from '../../node_modules/typescript/lib/typescript.js';
// Exact frozen handler; reply and cleanup states are explicitly declared doubles.
const source = await fs.readFile(new URL('./source/scripts/qa/co-op-transport/wasm-review.js', import.meta.url), 'utf8');
const parsed = ts.createSourceFile('frozen-parent.js', source, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
const statement = parsed.statements.find(s => ts.isExpressionStatement(s)
  && ts.isBinaryExpression(s.expression)
  && s.expression.left.getText(parsed) === '$("leave").onclick');
assert.ok(statement);
const checks = [], calls = [], button = {};
const state = {
  host:{mode:'host',phase:'hosting',failure_code:null,wake:{state:'live',removed_owner:false}},
  guest:{mode:'guest',phase:'connected',failure_code:null,created:1,closed:0,requests:1,timers:1,wake:{state:'live',local_view_ended:false}},
};
const context = vm.createContext({
  state, scenario:'configured-ion-wake-r1-e1',
  $:()=>button, async action(_label, fn){return fn();},
  async send(role,name){
    calls.push({role,name});
    // A current failed/retired reply arrives during the leave operation. Prior
    // successful native teardown facts remain observable, but the run failed.
    Object.assign(state.guest,{mode:'solo',phase:'failed',failure_code:'wake_native_frame',closed:1,requests:0,timers:0});
    Object.assign(state.guest.wake,{state:'inconclusive',reason:'wake_native_frame',local_view_ended:true});
    state.host.wake.removed_owner=true;
    return state[role];
  },
  async until(predicate){assert.equal(predicate(),true);},
  async verify(){},
  check(name,passed){checks.push({name,passed});},
});
vm.runInContext(statement.getText(parsed),context);
let outcome;
try {await button.onclick();outcome='resolved';}
catch(error){outcome=error.message;}
console.log(JSON.stringify({scope:'exact frozen leave handler; failed current reply double, no browser/native/RTC execution',outcome,checks,calls,guest_after:state.guest},null,2));
assert.notEqual(outcome,'resolved','A current inconclusive native failure must not qualify the newly requested Wake leave/removal gate even when earlier teardown facts are true.');
