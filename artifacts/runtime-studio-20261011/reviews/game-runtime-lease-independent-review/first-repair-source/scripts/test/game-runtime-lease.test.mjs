import test from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { acquireGameLease } from '../game-runtime-lease.mjs';
import { supervise } from '../dev.mjs';

test('OS lease rejects a concurrent runtime and is reusable after idempotent close', async t => {
  const first = await acquireGameLease(0);
  t.after(() => first.close());
  await assert.rejects(acquireGameLease(first.port), /already reserved/);
  await Promise.all([first.close(), first.close()]);
  const next = await acquireGameLease(first.port);
  await next.close();
});

test('OS releases the lease when its actual child process is killed', async t => {
  const module = new URL('../game-runtime-lease.mjs', import.meta.url).href;
  const child = spawn(process.execPath, ['--input-type=module', '-e', `import {acquireGameLease} from ${JSON.stringify(module)}; const lease=await acquireGameLease(0); console.log(lease.port); setInterval(()=>{},1000);`], { stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true });
  t.after(() => child.kill());
  const [output] = await once(child.stdout, 'data');
  const port = Number(output.toString().trim());
  assert.ok(port > 0);
  await assert.rejects(acquireGameLease(port), /already reserved/);
  const closed = once(child, 'close');
  child.kill('SIGKILL');
  await closed;
  const replacement = await acquireGameLease(port);
  await replacement.close();
});

test('failed service closes its actual sibling before supervision completes', { timeout: 10000 }, async () => {
  const run = supervise([['-e', 'setInterval(()=>{},1000)'], ['-e', 'process.exit(7)']]);
  assert.equal(await run.done, 7);
  assert.ok(run.children.every(child => child.exitCode !== null || child.signalCode !== null));
});

test('explicit stop waits for every owned child and reports a clean stop', { timeout: 10000 }, async () => {
  const run = supervise([['-e', 'setInterval(()=>{},1000)'], ['-e', 'setInterval(()=>{},1000)']]);
  run.stop();
  assert.equal(await run.done, 0);
  assert.ok(run.children.every(child => child.exitCode !== null || child.signalCode !== null));
});
