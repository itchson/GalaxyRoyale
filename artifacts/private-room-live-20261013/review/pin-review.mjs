import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
const own = import.meta.dirname, root = path.resolve(own, '../../..');
const sha = (value) => crypto.createHash('sha256').update(value).digest('hex');
const reviewed = [
  'scripts/qa/co-op-transport/client.js',
  'scripts/qa/co-op-transport/wasm-review.js',
  'scripts/qa/co-op-transport/server.mjs',
  'scripts/test/coop-transport-qa.test.mjs',
  'scripts/test/coop-transport-wasm-qa.test.mjs',
];
const contracts = [
  'apps/game/src/coop/controller.ts', 'apps/game/src/coop/transport.ts',
  'apps/game/src/storage.ts', 'apps/coop/server.mjs', 'apps/coop/store.mjs',
];
const git = (...args) => execFileSync('git', args, { cwd: root });
const sources = {};
for (const relative of [...reviewed, ...contracts]) {
  const working = await fs.readFile(path.join(root, relative));
  const normalized = Buffer.from(working.toString('utf8').replaceAll('\r\n', '\n'));
  const canonical = git('show', `HEAD:${relative}`);
  sources[relative] = {
    scope: reviewed.includes(relative) ? 'Root changes independently reviewed' : 'Read-only production contract',
    workingBytes: working.length, workingSha256: sha(working),
    workingLfBytes: normalized.length, workingLfSha256: sha(normalized),
    canonicalHeadBytes: canonical.length, canonicalHeadSha256: sha(canonical),
  };
}
await fs.writeFile(path.join(own, 'root-helper.diff.patch'), git('diff', '--no-ext-diff', '--', ...reviewed));
const evidence = {};
for (const name of ['REVIEW.md', 'focused-functions.log', 'source-function-edges.mjs', 'source-function-edges.log', 'root-helper.diff.patch', 'pin-review.mjs']) {
  const bytes = await fs.readFile(path.join(own, name));
  evidence[name] = { bytes: bytes.length, sha256: sha(bytes) };
}
const result = {
  capturedAt: new Date().toISOString(), node: process.version,
  head: git('rev-parse', 'HEAD').toString().trim(), sources, evidence,
  commands: [
    { command: "node --experimental-transform-types --test --test-name-pattern='^Actual initialization|^Actual rejoin gate|^Actual approval QA|^Actual client approval commands' scripts/test/coop-transport-wasm-qa.test.mjs", exitCode: 0, tests: 5, pass: 5, log: 'focused-functions.log' },
    { command: 'node --test work/private-room-live-plan/root-helper-review/source-function-edges.mjs', exitCode: 0, tests: 2, pass: 2, log: 'source-function-edges.log' },
  ],
  authorPatchIndependentAcceptance: false,
  scope: 'Independent Root helper source/function review only; no listener/browser/build/stage/native/RTC/save mutation or live pass.',
};
await fs.writeFile(path.join(own, 'source-pins.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ head: result.head, tests: [5, 2], reviewed: Object.fromEntries(reviewed.map((name) => [name, sources[name].workingSha256])), reportSha256: evidence['REVIEW.md'].sha256 }, null, 2));
