import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import path from 'node:path';

const reviewed = 'C:/Users/mitch/.codex/worktrees/galaxy-hud-navigation/galaxy-royale';
const output = path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1'));
const commit = 'db15b91f3a1614ebd245fbab47787de737535882';
const parent = '8c284637f3dfd4f86982f2dc01b10fb8a70c8162';
const git = (...args) => execFileSync('git', ['-C', reviewed, ...args], { maxBuffer: 32 * 1024 * 1024 });
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
assert.equal(git('rev-parse', `${commit}^`).toString().trim(), parent);
const sources = [
  'crates/engine_core/src/frame.rs', 'crates/engine_core/src/lib.rs',
  'crates/engine_core/src/pixel_rig.rs', 'crates/engine_core/src/shadow.rs',
  'crates/game_core/src/hub_shadows.rs', 'crates/game_core/src/lib.rs',
  'crates/game_web/src/dock_art.rs', 'crates/game_web/src/dock_art_tests.rs',
  'crates/game_web/src/lib.rs', 'crates/game_web/src/lighting.rs',
  'crates/game_web/src/pixel.wgsl', 'crates/game_web/src/renderer.rs',
  'crates/game_web/src/shadow_uniforms.rs',
];
assert.deepEqual(git('diff-tree', '--no-commit-id', '--name-only', '-r', commit).toString().trim().split(/\r?\n/).sort(), sources.toSorted());
const inputs = [
  'crates/engine_core/src/camera.rs', 'crates/engine_core/src/transform.rs',
  'crates/engine_core/src/pixels.rs', 'crates/game_core/src/campaign.rs',
  'crates/game_core/src/identity.rs', 'crates/game_core/src/raicho_fine.rs',
  'crates/game_core/src/co_op_replica.rs', 'crates/game_web/src/browser.rs',
  'crates/game_web/Cargo.toml', 'Cargo.toml', 'Cargo.lock',
];
function pinSource(file, unchanged) {
  const bytes = git('show', `${commit}:${file}`);
  if (unchanged) assert.deepEqual(bytes, git('show', `${parent}:${file}`));
  const location = path.join(output, 'source', file);
  mkdirSync(path.dirname(location), { recursive: true });
  writeFileSync(location, bytes);
  return { path: file, git: `${commit}:${file}`, sha256: sha(bytes), bytes: bytes.length, snapshot: path.relative(output, location).replaceAll('\\', '/') };
}
const sourcePins = sources.map(file => pinSource(file, false));
const inputPins = inputs.map(file => pinSource(file, true));
const authorBase = path.join(reviewed, 'work/hub-shadow-prototype');
const rawNames = [
  'native-fields.json', 'native-deck-packets.json', 'native-export-committed.log',
  'deck-export-committed.log', 'native-portable-final.log', 'dock-tests-final.log',
  'lighting-tests-final.log', 'packing-tests-final.log', 'game-shadow-final.log',
  'clippy-portable-final.log', 'clippy-web-native-final-qualified.log',
  'clippy-webgl-final-qualified.log', 'clippy-webgpu-final-qualified.log',
  'clippy-native-first.log', 'clippy-wasm-webgl-first.log',
  'clippy-web-native-baseline-exception.log', 'native-all-first.log',
  'fmt-result.json', 'naga-glsl.log', 'software-proof.log', 'software-proof.py',
  'native-software-ray-contact.png', 'naga-portable/vs_main.glsl',
  'naga-portable/fs_main.glsl', 'naga-portable/src/main.rs',
  'naga-portable/Cargo.toml', 'naga-portable/Cargo.lock',
];
const rawPins = rawNames.map(file => {
  const absolute = path.join(authorBase, file);
  const bytes = readFileSync(absolute);
  return { path: absolute.replaceAll('\\', '/'), sha256: sha(bytes), bytes: bytes.length };
});
const native = JSON.parse(readFileSync(path.join(authorBase, 'native-fields.json'))).cases;
const packets = JSON.parse(readFileSync(path.join(authorBase, 'native-deck-packets.json'))).cases;
assert.equal(native.length, 6); assert.equal(packets.length, 6);
const popcount = word => { let v = word >>> 0, n = 0; while(v) { v &= v - 1; n++; } return n; };
// The second exporter reads JSON as serde_json::Value and reserializes its f64
// numbers. Decimal tails may differ by one f64 ULP while native f32 fields are
// bit-identical. Normalize only the declared f32 fields; mask/index/RGBA data
// still require exact integer equality.
function nativeSemantic(n) {
  return { ...n, position: n.position.map(Math.fround),
    basis_x: n.basis_x.map(Math.fround), basis_y: n.basis_y.map(Math.fround),
    heading: Math.fround(n.heading), height: Math.fround(n.height),
    light: { position: n.light.position.map(Math.fround), color: n.light.color.map(Math.fround),
      radius: Math.fround(n.light.radius), intensity: Math.fround(n.light.intensity) } };
}
function blocked(n, receiver) {
  const t = n.height / (n.light.radius * 0.35);
  const p = receiver.map((r, i) => r * (1 - t) + n.light.position[i] * t - n.position[i]);
  const x = n.basis_x, y = n.basis_y, det = x[0] * y[1] - x[1] * y[0];
  const local = [(y[1] * p[0] - y[0] * p[1]) / det, (-x[1] * p[0] + x[0] * p[1]) / det];
  const cell = [Math.floor(local[0] + n.width / 2), Math.floor(local[1] + n.height_cells / 2)];
  if(cell[0] < 0 || cell[1] < 0 || cell[0] >= n.width || cell[1] >= n.height_cells) return false;
  const bit = cell[1] * 128 + cell[0];
  return Boolean(n.mask[Math.floor(bit / 32)] & (1 << (bit % 32)));
}
let comparisons = 0;
const cases = [];
for(let i = 0; i < native.length; i++) {
  const n = native[i], p = packets[i];
  assert.deepEqual(nativeSemantic(p.native), nativeSemantic(n));
  assert.equal(n.width, 48); assert.equal(n.height_cells, 48);
  assert.equal(n.height, 6); assert.equal(n.mask.length, 4096);
  assert.deepEqual(n.light.position, [0, 0]); assert.equal(n.light.radius, 120);
  assert.equal(n.cells.length, n.damaged ? 618 : 662);
  assert.equal(n.mask.reduce((sum, w) => sum + popcount(w), 0), n.cells.length);
  const expected = new Array(4096).fill(0);
  for(const [index, rgba] of n.cells) {
    assert(Number.isInteger(index) && index >= 0 && index < 48 * 48);
    assert((rgba >>> 24) >= 128);
    const bit = Math.floor(index / 48) * 128 + index % 48;
    expected[Math.floor(bit / 32)] = (expected[Math.floor(bit / 32)] | (1 << (bit % 32))) >>> 0;
  }
  assert.deepEqual(n.mask, expected);
  assert.equal(p.pixels.length, 28549);
  assert.equal(p.pixels.filter(cell => cell.receiver).length, 25919);
  assert.deepEqual(p.descriptor_header, [1, 1, 4, 1]);
  assert.equal(p.descriptor_x[3], 6); assert.equal(p.descriptor_y[3], 0);
  let blockedCount = 0;
  for(const cell of p.pixels) {
    assert(cell.center.concat(cell.basis_x, cell.basis_y, cell.material).every(Number.isFinite));
    const actual = cell.receiver && blocked(n, cell.center);
    assert.equal(actual, cell.blocked_center);
    blockedCount += Number(actual); comparisons++;
  }
  assert.equal(blockedCount, p.center_blocked);
  cases.push({ position: n.position, heading: n.heading, damaged: n.damaged, occupied: n.cells.length, receivers: 25919, blocked_center: blockedCount });
  if(i % 2 === 1) {
    const previous = native[i - 1];
    assert.deepEqual(previous.position, n.position); assert.equal(previous.heading, n.heading);
    const retained = new Map(previous.cells);
    for(const [index, rgba] of n.cells) assert.equal(retained.get(index), rgba);
    assert.equal(previous.cells.length - n.cells.length, 44);
    assert.deepEqual(p.pixels.map(({ blocked_center, ...rest }) => rest), packets[i - 1].pixels.map(({ blocked_center, ...rest }) => rest));
  }
}
assert.equal(comparisons, 171294);
const portableLog = readFileSync(path.join(authorBase, 'native-portable-final.log'), 'utf8');
assert(portableLog.includes('53 passed; 0 failed; 1 ignored'));
assert(portableLog.includes('3 passed; 0 failed; 0 ignored'));
assert(portableLog.includes('515 passed; 0 failed; 13 ignored'));
for(const [file, count] of [['dock-tests-final.log', 7], ['lighting-tests-final.log', 9], ['packing-tests-final.log', 2], ['game-shadow-final.log', 2]]) {
  assert(readFileSync(path.join(authorBase, file), 'utf8').includes(`${count} passed; 0 failed;`));
}
for(const file of ['native-export-committed.log', 'deck-export-committed.log']) {
  assert(readFileSync(path.join(authorBase, file), 'utf8').includes('1 passed; 0 failed;'));
}
assert.equal(JSON.parse(readFileSync(path.join(authorBase, 'fmt-result.json'))).exit_code, 0);
const nagaLog = readFileSync(path.join(authorBase, 'naga-glsl.log'), 'utf8');
for(const line of ['empty optional capabilities', 'ObjectTable: 1024B', 'Caster: 48B', 'ShadowDescriptors: 400B', 'ShadowMasks: 16384B', 'GLSL ES300 vs_main:', 'GLSL ES300 fs_main:']) assert(nagaLog.includes(line));
for(const file of ['naga-portable/vs_main.glsl', 'naga-portable/fs_main.glsl']) assert(readFileSync(path.join(authorBase, file), 'utf8').startsWith('#version 300 es'));

const authorManifest = path.join(authorBase, 'MANIFEST.json');
const authorHandoff = path.join(authorBase, 'HANDOFF.md');
assert(existsSync(authorManifest) && existsSync(authorHandoff), 'Author handoff must be frozen before review publication');
const manifestBytes = readFileSync(authorManifest);
const handoffBytes = readFileSync(authorHandoff);
assert.equal(sha(handoffBytes), '8b065e405cf1c24d6b23f0f00136c728e373153386426163eaa0413326fbacfc');
assert.equal(sha(manifestBytes), '626cdf5ea5e9c443495b464632ab9771827c6a8ce795e7ebfaeef83ce94353d8');
const authored = JSON.parse(manifestBytes);
assert.equal(authored.version, 1); assert.equal(authored.commit, commit); assert.equal(authored.parent, parent);
assert.equal(authored.source.length, 13); assert.equal(authored.inputs.length, 4); assert.equal(authored.evidence.length, 34);
for(const pin of [...authored.source, ...authored.inputs]) {
  const checkout = readFileSync(path.join(reviewed, pin.path));
  const bytes = git('show', `${commit}:${pin.path}`);
  assert.equal(checkout.length, pin.bytes); assert.equal(sha(checkout), pin.sha256);
  assert.deepEqual(Buffer.from(checkout.toString().replaceAll('\r\n', '\n')), bytes);
  if(pin.git_blob) assert.equal(git('rev-parse', `${commit}:${pin.path}`).toString().trim(), pin.git_blob);
}
for(const pin of [...authored.evidence, authored.handoff]) {
  const bytes = readFileSync(path.join(reviewed, pin.path));
  assert.equal(bytes.length, pin.bytes); assert.equal(sha(bytes), pin.sha256);
}
const amendedPath = path.join(authorBase, 'MANIFEST-GIT-V2.json');
const amendedBytes = readFileSync(amendedPath);
assert.equal(sha(amendedBytes), '9e5dcac8778aac1deb01012734d9ddfaf54c6d0c66857004d7c2e98a50dda5d9');
const amended = JSON.parse(amendedBytes);
assert.equal(amended.version, 2); assert.equal(amended.commit, commit); assert.equal(amended.parent, parent);
assert.equal(amended.source.length, 13); assert.equal(amended.inputs.length, 4); assert.equal(amended.evidence.length, 38);
for(const pin of amended.source) {
  const bytes = git('cat-file', 'blob', `${commit}:${pin.path}`);
  assert.equal(bytes.length, pin.bytes); assert.equal(sha(bytes), pin.sha256);
  assert.equal(git('rev-parse', `${commit}:${pin.path}`).toString().trim(), pin.git_blob);
  const original = authored.source.find(p => p.path === pin.path);
  assert.deepEqual(pin.working_checkout, { bytes: original.bytes, sha256: original.sha256 });
}
for(const pin of [...amended.inputs, ...amended.evidence]) {
  const bytes = readFileSync(path.join(reviewed, pin.path));
  assert.equal(bytes.length, pin.bytes); assert.equal(sha(bytes), pin.sha256);
}
const verification = { schema: 1, commit, parent, source_pins: sourcePins.length, unchanged_inputs: inputPins.length, raw_pins: rawPins.length, author_pins_verified: { original_source: 13, original_inputs: 4, original_evidence: 34, original_handoff: 1, canonical_git_source: 13, amended_raw_evidence: 38 }, native_fixture_cases: cases, independent_center_ray_comparisons: comparisons, author_portable_native: { passed: 571, ignored: 14, failed: 0 }, author_scoped_web: { dock: 7, lighting: 9, packing: 2, game_shadow: 2 }, author_hand_off_sha256: sha(handoffBytes), author_manifest_sha256: sha(manifestBytes), author_canonical_manifest_sha256: sha(amendedBytes), reviewer_execution: 'Git/source/evidence pin verification and analytical native-packet oracle only; no native build/test rerun, browser, device, listener, GPU, or worker.' };
writeFileSync(path.join(output, 'verification.json'), JSON.stringify(verification, null, 2) + '\n');
const manifest = { schema: 1, commit, parent, source: sourcePins, unchanged_inputs: inputPins, author_raw: rawPins, author_handoff: { path: authorHandoff.replaceAll('\\', '/'), sha256: sha(handoffBytes), bytes: handoffBytes.length }, author_manifest: { path: authorManifest.replaceAll('\\', '/'), sha256: sha(manifestBytes), bytes: manifestBytes.length }, author_canonical_manifest: { path: amendedPath.replaceAll('\\', '/'), sha256: sha(amendedBytes), bytes: amendedBytes.length } };
writeFileSync(path.join(output, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
console.log(JSON.stringify(verification));
