import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
const own = import.meta.dirname, root = path.resolve(own, '../../..');
const sha = (v) => crypto.createHash('sha256').update(v).digest('hex');
const owned = ['scripts/qa/co-op-transport/prepare.mjs', 'scripts/test/coop-private-room-provenance-qa.test.mjs'];
const sources = {};
for (const relative of [...owned, 'scripts/test/coop-transport-fifth-qa.test.mjs', 'scripts/test/coop-transport-wasm-qa.test.mjs']) {
  const bytes = await fs.readFile(path.join(root, relative));
  const lf = Buffer.from(bytes.toString('utf8').replaceAll('\r\n', '\n'));
  sources[relative] = { owner: owned.includes(relative) ? 'This bounded amendment' : 'Root/source check only', bytes: bytes.length, sha256: sha(bytes), lfBytes: lf.length, lfSha256: sha(lf) };
}
const git = (...args) => execFileSync('git', args, { cwd: root });
await fs.writeFile(path.join(own, 'prepare.diff.patch'), git('diff', '--no-ext-diff', '--', owned[0]));
await fs.copyFile(path.join(root, owned[1]), path.join(own, 'final-provenance-test.mjs'));
const evidence = {};
for (const name of ['HANDOFF.md', 'original-regression.log', 'current-provenance-tests.log', 'current-fifth-tests.log', 'current-wasm-allowlist.log', 'format.log', 'diff-check.log', 'prepare.diff.patch', 'final-provenance-test.mjs', 'pin-evidence.mjs']) {
  const bytes = await fs.readFile(path.join(own, name)); evidence[name] = { bytes: bytes.length, sha256: sha(bytes) };
}
const result = { frozenAt: new Date().toISOString(), head: git('rev-parse', 'HEAD').toString().trim(), node: process.version, sources, evidence,
  commands: [
    { command: "node --test --test-name-pattern='^historical|^room-code admission' scripts/test/coop-private-room-provenance-qa.test.mjs", phase: 'before preparedNames correction', exitCode: 1, tests: 3, pass: 0, fail: 3, log: 'original-regression.log' },
    { command: 'node --test scripts/test/coop-private-room-provenance-qa.test.mjs', exitCode: 0, tests: 9, pass: 9, fail: 0, log: 'current-provenance-tests.log' },
    { command: 'node --experimental-transform-types --test scripts/test/coop-transport-fifth-qa.test.mjs', exitCode: 0, tests: 10, pass: 10, fail: 0, log: 'current-fifth-tests.log' },
    { command: "node --experimental-transform-types --test --test-name-pattern='^WASM allowlist' scripts/test/coop-transport-wasm-qa.test.mjs", exitCode: 0, tests: 1, pass: 1, fail: 0, log: 'current-wasm-allowlist.log' },
  ], scope: 'QA-only legacy/current allowlist correction after Root live shutdown. Declared fixtures and isolated transport preparation only; no frozen capture changes/listener/browser/build/native/RTC/save/negative live pass.' };
await fs.writeFile(path.join(own, 'source-pins.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ head: result.head, sources: Object.fromEntries(owned.map((p) => [p, sources[p]])), tests: [9, 10, 1], originalFailedRegressionPreserved: true }, null, 2));
