import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
const own = import.meta.dirname, root = path.resolve(own, '../..');
const base = 'fbb8b1c640f0e11d495cf00c11bf557500701674';
const git = (...args) => execFileSync('git', args, { cwd: root });
const sha = (value) => crypto.createHash('sha256').update(value).digest('hex');
const head = git('rev-parse', 'HEAD').toString().trim();
const files = git('diff', '--name-only', base, head).toString().trim().split(/\r?\n/);
if (files.length !== 6 || files.some((name) => !name.startsWith('scripts/qa/co-op-transport/') && name !== 'scripts/test/coop-private-room-negative-qa.test.mjs')) throw Error('Changed scope escaped ownership');
if (git('status', '--porcelain').toString().trim()) throw Error('Tracked candidate not frozen');
git('diff', '--quiet', base, head, '--', 'apps/game', 'apps/coop', 'crates', 'assets', 'Cargo.toml', 'Cargo.lock', 'package.json', 'pnpm-lock.yaml', 'tsconfig.json');
const sources = {};
for (const relative of files) {
  const bytes = await fs.readFile(path.join(root, relative)), canonical = git('show', `${head}:${relative}`), lf = Buffer.from(bytes.toString('utf8').replaceAll('\r\n', '\n'));
  if (sha(lf) !== sha(canonical)) throw Error(`Working/canonical mismatch: ${relative}`);
  sources[relative] = { workingBytes: bytes.length, workingSha256: sha(bytes), lfBytes: lf.length, lfSha256: sha(lf), canonicalGitBytes: canonical.length, canonicalGitSha256: sha(canonical) };
}
await fs.writeFile(path.join(own, 'candidate.diff.patch'), git('diff', '--no-ext-diff', base, head, '--', ...files));
const evidence = {};
for (const name of ['HANDOFF.md', 'frozen-plan.md', 'negative-tests-original.log', 'negative-tests-expanded.log', 'negative-tests-final.log', 'existing-wasm-functions.log', 'existing-fifth-functions.log', 'format.log', 'final-format.log', 'syntax-check.log', 'diff-check.log', 'original-commit-failure.log', 'commit.log', 'studio-start-input.json', 'studio-start-result.json', 'studio-run-input.json', 'studio-run-result.json', 'candidate.diff.patch', 'pin-evidence.mjs']) {
  const bytes = await fs.readFile(path.join(own, name)); evidence[name] = { bytes: bytes.length, sha256: sha(bytes) };
}
const result = { frozenAt: new Date().toISOString(), node: process.version, checkout: root, branch: git('branch', '--show-current').toString().trim(), base, head, trackedClean: true,
  priorHavenBranch: git('rev-parse', 'codex/haven-scene-fixtures').toString().trim(), productionRuntimeUnchanged: true, sources, evidence,
  commands: [
    { command: 'node --test scripts/test/coop-private-room-negative-qa.test.mjs', exitCode: 0, tests: 11, pass: 11, fail: 0, log: 'negative-tests-final.log' },
    { command: "node --experimental-transform-types --test --test-name-pattern='^Actual initialization|^Actual client frame|^Actual rejoin|^Actual approval|^Actual client approval|^Actual child message|^Actual New session|^Actual Drive|^Actual localSpec|^Actual commit|^WASM report|^Owned reports|^WASM allowlist' scripts/test/coop-transport-wasm-qa.test.mjs", exitCode: 0, tests: 16, pass: 16, fail: 0, log: 'existing-wasm-functions.log' },
    { command: 'node --experimental-transform-types --test scripts/test/coop-transport-fifth-qa.test.mjs', exitCode: 0, tests: 10, pass: 10, fail: 0, log: 'existing-fifth-functions.log' },
  ], scope: 'Author QA source/function implementation only. No independent acceptance, normal build/stage/listener/browser/native/RTC/stored-profile/live negative execution; prior captures preserved.' };
await fs.writeFile(path.join(own, 'source-pins.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ base, head, branch: result.branch, changedFiles: files.length, trackedClean: result.trackedClean, tests: [11,16,10], reportSha256: evidence['HANDOFF.md'].sha256 }, null, 2));
