/** Independent Root helper source-function checks. Declared boundaries; no native/browser/HTTP/stage/profile execution. */
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import vm from 'node:vm';
import crypto from 'node:crypto';
import ts from 'typescript';
const root = path.resolve(import.meta.dirname, '../../..');
const sha = (v) => crypto.createHash('sha256').update(v).digest('hex');
const extract = async (relative, name) => {
  const source = await fs.readFile(path.join(root, relative), 'utf8');
  const ast = ts.createSourceFile(relative, source, ts.ScriptTarget.ES2022, true, ts.ScriptKind.JS);
  const declaration = ast.statements.find((n) => ts.isFunctionDeclaration(n) && n.name?.text === name);
  assert(declaration);
  return declaration.getText(ast).replace(/^export\s+/, '');
};

test('Root actual launch rejects compiled/service metadata disagreement before the native server factory', async () => {
  const source = await extract('scripts/qa/co-op-transport/server.mjs', 'start');
  const services = ['apps/coop/server.mjs', 'apps/coop/store.mjs'];
  const pins = Object.fromEntries(services.map((s) => [s, sha(Buffer.from(s))]));
  const compiled = { 'apps/game/src/coop/transport.ts': 'c'.repeat(64) };
  const provenance = {
    gitInputs: compiled,
    compiledGitInputs: compiled,
    compiledSourceCommit: 'b'.repeat(40),
    buildStamp: { head: 'a'.repeat(40), compiledFromHead: 'b'.repeat(40) },
  };
  const base = {
    version: 3, mode: 'wasm', typescript: '5.9.3',
    source_root: root, source_commit: 'a'.repeat(40), dist: path.join(root, 'dist'),
    git_inputs: compiled, build_stamp: provenance.buildStamp,
    compiled_source_commit: provenance.compiledSourceCommit, compiled_git_inputs: compiled,
    service_inputs: pins, inputs: pins, files: {},
  };
  const cases = [
    ['valid preflight reaches declared stop boundary', () => {}, /declared_before_listener/],
    ['missing compiled revision', (m) => delete m.compiled_source_commit, /Owned prepared provenance changed/],
    ['changed compiled dependency map', (m) => { m.compiled_git_inputs = {}; }, /Owned prepared provenance changed/],
    ['missing service map', (m) => delete m.service_inputs, /declared_invalid_service_map/],
    ['missing service ordinary input', (m) => delete m.inputs[services[0]], /Prepared service provenance is incomplete/],
    ['disagreeing service ordinary input', (m) => { m.inputs[services[1]] = '0'.repeat(64); }, /Prepared service provenance is incomplete/],
  ];
  for (const [name, mutate, error] of cases) {
    const manifest = structuredClone(base);
    // Keep the service map and ordinary input map independent, as in actual JSON.
    manifest.inputs = { ...manifest.inputs };
    mutate(manifest);
    let factoryCalls = 0;
    const context = vm.createContext({
      root, path, Buffer, sha,
      stagePath: () => path.join(root, 'work', 'declared-not-created-stage'),
      preparedNames: () => [], ownedSources: [], serviceSources: services,
      verifyOwnedSource: async () => provenance,
      verifyServiceInputs: async (value) => {
        // Declared verifier return boundary only; this does not independently test prepare.mjs.
        if (JSON.stringify(value) !== JSON.stringify(pins)) throw Error('declared_invalid_service_map');
      },
      fs: { readFile: async (file) => file.endsWith('manifest.json')
        ? JSON.stringify(manifest) : Buffer.from(path.relative(root, file).replaceAll('\\', '/')) },
      makeCoopServer: () => { factoryCalls++; throw Error('declared_before_listener'); },
    });
    vm.runInContext(source, context);
    await assert.rejects(context.start({ stage: 'declared' }), error, name);
    assert.equal(factoryCalls, name.startsWith('valid') ? 1 : 0, name);
  }
});

test('Root actual approval polling waits for pending observation, checks native counters, then resolves the host request', async () => {
  const source = await extract('scripts/qa/co-op-transport/wasm-review.js', 'approvePendingJoin');
  let now = 0;
  const approvedAt = [], checks = [];
  const context = vm.createContext({
    performance: { now: () => now },
    setTimeout: (fn, ms) => { now += ms; queueMicrotask(fn); },
    send: async (role, command) => {
      if (command === 'approve-next') { approvedAt.push(now); return {}; }
      return {
        pending: role === 'host' && now >= 700 ? 1 : 0,
        awaiting: role === 'guest', created: role === 'host' ? 6 : 4,
        transport_ready: 0, admitted: 0,
      };
    },
    check: (name, passed) => { checks.push([name, passed]); assert(passed); },
  });
  vm.runInContext(source, context);
  await context.approvePendingJoin([6, 4]);
  assert.deepEqual(approvedAt, [700]);
  assert.deepEqual(checks, [['host-approval-before-rtc', true]]);
});
