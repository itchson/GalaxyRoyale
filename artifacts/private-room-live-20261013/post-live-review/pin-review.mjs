import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
const own = import.meta.dirname, root = path.resolve(own, '../..');
const sha = (v) => crypto.createHash('sha256').update(v).digest('hex');
const reviewed = ['scripts/qa/co-op-transport/client.js', 'scripts/qa/co-op-transport/wasm-review.js', 'scripts/qa/co-op-transport/server.mjs', 'scripts/test/coop-transport-wasm-qa.test.mjs'];
const git = (...args) => execFileSync('git', args, { cwd: root });
const sources = {};
for (const relative of reviewed) {
  const bytes = await fs.readFile(path.join(root, relative)), canonical = git('show', `HEAD:${relative}`), lf = Buffer.from(bytes.toString('utf8').replaceAll('\r\n', '\n'));
  sources[relative] = { workingBytes: bytes.length, workingSha256: sha(bytes), lfBytes: lf.length, lfSha256: sha(lf), canonicalHeadBytes: canonical.length, canonicalHeadSha256: sha(canonical) };
}
await fs.writeFile(path.join(own, 'root-helper.diff.patch'), git('diff', '--no-ext-diff', '--', ...reviewed));
const evidence = {};
for (const name of ['REVIEW.md', 'focused-tests.log', 'edge-tests.mjs', 'edge-tests.log', 'root-helper.diff.patch', 'pin-review.mjs']) { const bytes = await fs.readFile(path.join(own, name)); evidence[name] = { bytes: bytes.length, sha256: sha(bytes) }; }
const result = { frozenAt: new Date().toISOString(), head: git('rev-parse', 'HEAD').toString().trim(), node: process.version, sources, evidence,
  commands: [
    { command: "node --experimental-transform-types --test --test-name-pattern='^Actual approval observation refuses late|^Actual child message route preserves' scripts/test/coop-transport-wasm-qa.test.mjs", exitCode: 0, tests: 2, pass: 2, fail: 0, log: 'focused-tests.log' },
    { command: 'node --test work/private-room-post-live-review/edge-tests.mjs', exitCode: 0, tests: 3, pass: 3, fail: 0, log: 'edge-tests.log' },
  ], scope: 'Independent post-live source/function review only. Prior live captures retain prior-source provenance. No helper edit/listener/browser/stage/build/native/RTC/save operation or independent full-suite claim.' };
await fs.writeFile(path.join(own, 'source-pins.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ head: result.head, sources: Object.fromEntries(reviewed.map((name) => [name, sources[name].workingSha256])), reviewSha256: evidence['REVIEW.md'].sha256, tests: [2,3] }, null, 2));
