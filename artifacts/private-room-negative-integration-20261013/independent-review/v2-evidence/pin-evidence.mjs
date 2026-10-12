import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
const own = import.meta.dirname, root = path.resolve(own, '../../..');
const base = '1b14bff9e6da0a1a249f1bdc42ab49e59e74e52c';
const head = '7c188d4198901ead15e4bffa2b0063b9c95f7674';
const git = (...args) => execFileSync('git', args, { cwd: root });
const sha = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const allowed = ['scripts/qa/co-op-transport/wasm-review.js', 'scripts/test/coop-private-room-negative-qa.test.mjs'];
if (git('rev-parse', 'HEAD').toString().trim() !== head || git('rev-parse', 'codex/private-room-negative-qa').toString().trim() !== base) throw Error('Frozen refs changed');
if (git('status', '--porcelain').toString().trim()) throw Error('Tracked source not frozen');
const changed = git('diff', '--name-only', base, head).toString().trim().split(/\r?\n/);
if (JSON.stringify(changed.sort()) !== JSON.stringify([...allowed].sort())) throw Error('Changed ownership escaped');
git('diff', '--quiet', base, head, '--', 'apps', 'crates', 'assets', 'Cargo.toml', 'Cargo.lock', 'package.json', 'pnpm-lock.yaml', 'tsconfig.json');
const sources = {};
for (const relative of [...allowed, 'scripts/qa/co-op-transport/client.js', 'apps/game/src/coop/controller.ts']) {
  const working = await fs.readFile(path.join(root, relative)), canonical = git('show', `${head}:${relative}`), lf = Buffer.from(working.toString('utf8').replaceAll('\r\n', '\n'));
  if (sha(lf) !== sha(canonical)) throw Error(`Canonical mismatch: ${relative}`);
  sources[relative] = { workingBytes: working.length, workingSha256: sha(working), lfBytes: lf.length, lfSha256: sha(lf), canonicalGitBytes: canonical.length, canonicalGitSha256: sha(canonical) };
}
const original = await fs.readFile(path.join(own, 'original-parent.js'));
if (sha(original) !== sha(git('show', `${base}:${allowed[0]}`))) throw Error('Original parent snapshot changed');
await fs.writeFile(path.join(own, 'candidate.diff.patch'), git('diff', '--no-ext-diff', base, head, '--', ...allowed));
const evidence = {};
for (const name of ['HANDOFF.md', 'original-parent.js', 'original-regression.log', 'pre-visibility-parent.js', 'visibility-original-regression.log', 'pre-health-parent.js', 'health-preflight-original-regression.log', 'focused-tests.log', 'existing-wasm-functions.log', 'existing-fifth-functions.log', 'format.log', 'syntax-check.log', 'diff-check.log', 'commit.log', 'candidate.diff.patch', 'pin-evidence.mjs']) {
  const bytes = await fs.readFile(path.join(own, name)); evidence[name] = { bytes: bytes.length, sha256: sha(bytes) };
}
const commands = [
  { command: 'node --experimental-transform-types --test scripts/test/coop-private-room-negative-qa.test.mjs', exitCode: 0, tests: 17, pass: 17, fail: 0, log: 'focused-tests.log' },
  { command: "node --experimental-transform-types --test --test-name-pattern='^Actual initialization|^Actual client frame|^Actual rejoin|^Actual approval|^Actual client approval|^Actual child message|^Actual New session|^Actual Drive|^Actual localSpec|^Actual commit|^WASM report|^Owned reports|^WASM allowlist' scripts/test/coop-transport-wasm-qa.test.mjs", exitCode: 0, tests: 16, pass: 16, fail: 0, log: 'existing-wasm-functions.log' },
  { command: 'node --experimental-transform-types --test scripts/test/coop-transport-fifth-qa.test.mjs', exitCode: 0, tests: 10, pass: 10, fail: 0, log: 'existing-fifth-functions.log' },
];
const result = { frozenAt: new Date().toISOString(), node: process.version, checkout: root, branch: git('branch', '--show-current').toString().trim(), base, head, originalBranchPreserved: true, trackedClean: true, productionUnchanged: true, sources, evidence, commands, originalFailures: ['original-regression.log', 'visibility-original-regression.log', 'health-preflight-original-regression.log'], scope: 'Author QA source-function checks only; no build/stage/listener/browser/native/RTC/live acceptance. Raw stopped/pagehide liveness not directly observable from current protocol.' };
await fs.writeFile(path.join(own, 'source-pins.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ base, head, changed, tests: commands.map((entry) => entry.tests), trackedClean: true, handoffSha256: evidence['HANDOFF.md'].sha256, parentSha256: sources[allowed[0]].canonicalGitSha256, testSha256: sources[allowed[1]].canonicalGitSha256 }, null, 2));
