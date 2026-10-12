import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';
const root = process.cwd();
const out = path.join(root, 'work/haven-fixture-review');
const commit = '5df19ae371cae78b167e64921d84040d64cb4011';
const parent = '8c284637f3dfd4f86982f2dc01b10fb8a70c8162';
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const git = (...args) => execFileSync('git', args, { cwd: root, maxBuffer: 16 * 1024 * 1024 });
const text = (...args) => git(...args).toString('utf8').trim();
assert.equal(text('rev-parse', 'HEAD'), commit);
assert.equal(text('rev-parse', `${commit}^`), parent);
assert.equal(text('status', '--porcelain'), '');
const paths = ['scripts/test/dock-menu.test.mjs', 'scripts/test/forge-integration.test.mjs'];
assert.deepEqual(text('diff', '--name-only', parent, commit).split('\n'), paths);
const source = paths.map(file => {
  const bytes = git('show', `${commit}:${file}`);
  assert.equal(fs.readFileSync(path.join(root, file), 'utf8').replaceAll('\r\n', '\n'), bytes.toString('utf8'));
  const target = path.join(out, 'snapshots', file);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, bytes);
  return { path: file, commit, bytes: bytes.length, sha256: sha(bytes) };
});
const inputs = ['apps/game/index.html', 'apps/game/src/main.ts', 'apps/game/src/input.ts', 'apps/game/src/loadout.ts', 'apps/game/src/forge-scene.ts', 'apps/game/src/hangar-scene.ts', 'apps/game/src/flight-menu.ts', 'crates/game_core/src/dock_facilities.rs', 'package.json'].map(file => {
  const bytes = git('show', `${commit}:${file}`);
  assert.deepEqual(bytes, git('show', `${parent}:${file}`));
  assert.equal(fs.readFileSync(path.join(root, file), 'utf8').replaceAll('\r\n', '\n'), bytes.toString('utf8'));
  return { path: file, commit, bytes: bytes.length, sha256: sha(bytes) };
});
const cases = bytes => [...bytes.toString('utf8').matchAll(/\btest\(\s*"([^"]+)"/g)].map(m => m[1]);
const old = paths.flatMap(file => cases(git('show', `${parent}:${file}`)));
const current = paths.flatMap(file => cases(git('show', `${commit}:${file}`)));
assert.equal(old.length, 33);
assert.equal(current.length, 39);
const renamed = new Map([
  ['the central Haven hub opens co-op without a facility hit or progression mutation', 'the central Haven hub is not a Multiplayer facility and does not alter flight or progression'],
  ['MEISHI is a standalone live card; returning to collections restores menu chrome', 'MEISHI is a standalone live card; facility scenes restore chrome without cross-collection tabs'],
  ['real Forge category survives successful repaint while a fresh open resets to overview', 'real Forge category survives successful repaint while a fresh open resets to the ability index'],
  ['Hangar page and construction/selection bindings pass full per-hull XP metadata', 'Forge construction and Hangar selection preserve full per-hull XP and crafting does not auto-select'],
]);
for (const name of old) assert.ok(current.includes(renamed.get(name) ?? name), `Dropped case: ${name}`);
const final = fs.readFileSync(path.join(out, 'committed-targeted.log'), 'utf8');
for (const name of current) assert.ok(final.includes(name), `Missing executed case: ${name}`);
assert.match(final, /# tests 39\s+# suites 0\s+# pass 39\s+# fail 0/);
assert.match(final, /# skipped 0/);
const baseline = fs.readFileSync(path.join(out, 'baseline-targeted.log'), 'utf8');
assert.match(baseline, /ℹ tests 33/);
assert.match(baseline, /ℹ pass 16/);
assert.match(baseline, /ℹ fail 17/);
const first = fs.readFileSync(path.join(out, 'migrated-first.log'), 'utf8');
assert.match(first, /# tests 36\s+# suites 0\s+# pass 32\s+# fail 4/);
const rawNames = ['baseline-targeted.log', 'migrated-first.log', 'migrated-second.log', 'committed-targeted.log', 'format.log', 'format-check.log', 'diff-check.log', 'commit.log'];
const raw = rawNames.map(file => {
  const bytes = fs.readFileSync(path.join(out, file));
  return { path: `work/haven-fixture-review/${file}`, bytes: bytes.length, sha256: sha(bytes) };
});
const verification = { commit, parent, changed_paths: paths, source_pins: source.length, unchanged_executed_inputs: inputs.length, raw_pins: raw.length, baseline: { tests: 33, passed: 16, failed: 17 }, first_migration: { tests: 36, passed: 32, failed: 4 }, committed: { tests: 39, passed: 39, failed: 0, skipped: 0 }, old_cases_preserved: 33, renamed_cases: renamed.size, added_cases: 6, tracked_clean: true, scope: 'Actual source-function/view-module execution with declared DOM/Game/save/native-circle models; no native/IndexedDB/browser/GPU/RTC/full-build acceptance.' };
fs.writeFileSync(path.join(out, 'verification.json'), `${JSON.stringify(verification, null, 2)}\n`);
const evidence = ['HANDOFF.md', 'verify.mjs', 'verification.json'].map(file => {
  const bytes = fs.readFileSync(path.join(out, file));
  return { path: `work/haven-fixture-review/${file}`, bytes: bytes.length, sha256: sha(bytes) };
});
fs.writeFileSync(path.join(out, 'manifest.json'), `${JSON.stringify({ version: 1, commit, parent, source, inputs, raw, evidence }, null, 2)}\n`);
console.log(JSON.stringify(verification));
