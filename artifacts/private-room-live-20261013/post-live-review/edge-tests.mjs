/** Actual Root source functions at declared timer/message boundaries; no browser/server/stage/profile execution. */
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import vm from 'node:vm';
import ts from 'typescript';
const root = path.resolve(import.meta.dirname, '../..');
const source = await fs.readFile(path.join(root, 'scripts/qa/co-op-transport/wasm-review.js'), 'utf8');
const ast = ts.createSourceFile('wasm-review.js', source, ts.ScriptTarget.ES2022, true, ts.ScriptKind.JS);
const fn = (name) => {
  const node = ast.statements.find((n) => ts.isFunctionDeclaration(n) && n.name?.text === name);
  assert(node); return node.getText(ast);
};
const failureDeclaration = ast.statements.filter(ts.isVariableStatement).flatMap((n) => [...n.declarationList.declarations]).find((n) => n.name.getText(ast) === 'failureCodes');
assert(failureDeclaration);
const messageDeclaration = ast.statements.find((n) => ts.isExpressionStatement(n) && ts.isCallExpression(n.expression) && n.expression.expression.getText(ast) === 'addEventListener' && n.expression.arguments[0]?.getText(ast) === '"message"');
assert(messageDeclaration);

test('actual parent send bounds its timer and message route retains only controlled errors while removing pending requests', async () => {
  for (const reason of ['approval_not_pending', 'private token/SDP']) {
    const delays = [], posted = [], cleared = [];
    let handler;
    const frameWindow = { postMessage: (value) => posted.push(value) };
    const context = vm.createContext({
      nextId: 1, pending: new Map(), children: { host: { frame: { contentWindow: frameWindow }, origin: 'http://localhost:4199' } },
      setTimeout: (callback, ms) => { delays.push(ms); return 1; },
      clearTimeout: (id) => cleared.push(id),
      addEventListener: (kind, callback) => { assert.equal(kind, 'message'); handler = callback; },
    });
    vm.runInContext(`const ${failureDeclaration.getText(ast)};`, context);
    vm.runInContext(fn('send'), context);
    vm.runInContext(messageDeclaration.getText(ast), context);
    const pending = context.send('host', 'approval-state', {}, 900);
    const outcome = assert.rejects(pending, reason === 'approval_not_pending' ? /approval_not_pending/ : /native_action_failed/);
    handler({ source: frameWindow, origin: 'http://localhost:4199', data: { type: 'native-qa-reply', id: posted[0].id, ok: false, error: reason } });
    await outcome;
    assert.deepEqual(delays, [900]);
    assert.deepEqual(cleared, [1]);
    assert.equal(context.pending.size, 0);
  }
});

test('actual parent send retains its ordinary default and late replies cannot reopen a timed-out request', async () => {
  const delays = [];
  let handler, timeout;
  const frameWindow = { postMessage: () => {} };
  const context = vm.createContext({
    nextId: 1, pending: new Map(), children: { host: { frame: { contentWindow: frameWindow }, origin: 'http://localhost:4199' } },
    setTimeout: (callback, ms) => { timeout = callback; delays.push(ms); return 1; }, clearTimeout: () => {},
    addEventListener: (kind, callback) => { handler = callback; },
  });
  vm.runInContext(`const ${failureDeclaration.getText(ast)};`, context);
  vm.runInContext(fn('send'), context);
  vm.runInContext(messageDeclaration.getText(ast), context);
  const pending = context.send('host', 'state');
  const outcome = assert.rejects(pending, /native_command_timeout/);
  timeout();
  await outcome;
  handler({ source: frameWindow, origin: 'http://localhost:4199', data: { type: 'native-qa-reply', id: 1, ok: true, value: { pending: 1 } } });
  assert.deepEqual(delays, [75000]);
  assert.equal(context.pending.size, 0);
});

test('actual approval loop shrinks every observation reply budget and still accepts timely real-state snapshots', async () => {
  let now = 0;
  const budgets = [], accepted = [], checks = [];
  const context = vm.createContext({
    performance: { now: () => now },
    setTimeout: (callback, ms) => { now += ms; queueMicrotask(callback); },
    send: async (role, name, args, timeout) => {
      if (name === 'approve-next') { accepted.push(now); return {}; }
      assert.equal(name, 'approval-state'); budgets.push(timeout);
      return { pending: role === 'host' && now >= 200 ? 1 : 0, awaiting: role === 'guest', created: role === 'host' ? 3 : 2, transport_ready: 0, admitted: 0 };
    },
    check: (name, passed) => { checks.push([name, passed]); assert(passed); },
  });
  vm.runInContext(fn('approvePendingJoin'), context);
  await context.approvePendingJoin([3, 2]);
  assert.deepEqual(budgets, [15000,15000,14900,14900,14800,14800]);
  assert.deepEqual(accepted, [200]);
  assert.deepEqual(checks, [['host-approval-before-rtc', true]]);
});
