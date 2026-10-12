import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
const own = import.meta.dirname, root = path.resolve(own, '../..');
const runtimeRef = '332ddd5ec0370e13ad346ea4ea988f35b61ca3e7', snapshotRef = '60e16acaba7ceda12afd02bf497f6aadd34da885';
const git = (...args) => execFileSync('git', args, { cwd: root });
const sha = (b) => crypto.createHash('sha256').update(b).digest('hex');
git('diff', '--quiet', runtimeRef, snapshotRef, '--', 'crates', 'apps/game', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'assets');
if (git('status', '--porcelain').toString().trim()) throw Error('Tracked checkout not frozen');
if (git('rev-parse', 'codex/private-room-negative-qa').toString().trim() !== '1b14bff9e6da0a1a249f1bdc42ab49e59e74e52c') throw Error('Original branch changed');
const verified = JSON.parse(await fs.readFile(path.join(own, 'compiled-inputs.json'), 'utf8'));
if (!verified.allCompiledArchiveFilesVerified || verified.ref !== snapshotRef || verified.actualFilesVerified !== 469) throw Error('Snapshot unverified');
const sources = {};
for (const relative of ['crates/game_core/src/co_op_wake.rs', 'crates/game_core/src/co_op_wake_tests.rs', 'crates/game_core/src/co_op.rs', 'crates/game_core/src/lib.rs', 'crates/game_core/src/regions.rs', 'crates/game_core/src/co_op_publish.rs', 'crates/game_core/src/co_op_publish_tests.rs', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']) {
  const bytes = git('show', `${snapshotRef}:${relative}`);
  if (sha(bytes) !== verified.files[relative].canonicalSha256) throw Error(`Canonical source differs: ${relative}`);
  sources[relative] = { canonicalBytes: bytes.length, canonicalSha256: sha(bytes), compiled: verified.files[relative] };
}
const evidence = {};
for (const name of ['REPORT.md', 'probe.rs', 'probe-original.rs', 'probe-publication-original.rs', 'compiler.log', 'frozen-native-source.zip', 'frozen-test-fixtures.zip', 'compiled-inputs.json', 'native-probe-original.log', 'native-probe-first-runtime.log', 'native-probe.log', 'native-wake-tests.log', 'native-publication-tests.log', 'verify-snapshot.py', 'verify-snapshot-original.py', 'pin-original-failure.log', 'pin-count-original-failure.log', 'pin-evidence.mjs']) {
  const bytes = await fs.readFile(path.join(own, name)); evidence[name] = { bytes: bytes.length, sha256: sha(bytes) };
}
const binaries = (await fs.readdir(path.join(own, 'cargo-target/release/deps'))).filter((file) => /^game_core-.*\.exe$/.test(file));
if (binaries.length !== 1) throw Error('Native executable ambiguous');
const binary = await fs.readFile(path.join(own, 'cargo-target/release/deps', binaries[0]));
const records = [];
for (const log of ['native-probe.log', 'native-wake-tests.log']) {
  const raw = await fs.readFile(path.join(own, log), 'utf8');
  for (const match of raw.matchAll(/NATIVE_BUDGET (\{[^\r\n]+\})/g)) {
    const record = JSON.parse(match[1]);
    const times = record.measurements.map((row) => row.us_per_wake_stage ?? row.us_per_acquisition).sort((a,b) => a-b);
    records.push({ log, ...record, medianUs: times[2], minimumUs: times[0], maximumUs: times[4] });
  }
}
if (records.length !== 12) throw Error('Measurements incomplete');
const command = (selector) => `CARGO_TARGET_DIR=<own>/cargo-target cargo test --manifest-path <own>/snapshot/Cargo.toml -p game_core --release --offline --locked ${selector} -- --nocapture --test-threads=1`;
const result = { frozenAt: new Date().toISOString(), runtimeRef, snapshotRef, nativeSourcesEqualBetweenRefs: true, nativeSnapshotFiles: verified.actualFilesVerified, nativeBinary: { relative: 'cargo-target/release/deps/' + binaries[0], bytes: binary.length, sha256: sha(binary) }, checkoutHead: git('rev-parse','HEAD').toString().trim(), trackedClean: true, originalNegativeBranchPreserved: true, sources, evidence, records, commands: [
  { command: command('work_only_native_budget_probe'), exitCode: 0, tests: 1, pass: 1, fail: 0, log: 'native-probe.log' },
  { command: command('co_op::wake_tests::'), exitCode: 0, tests: 14, existing: 13, workOnlyProbe: 1, pass: 14, fail: 0, log: 'native-wake-tests.log' },
  { command: command('co_op_publish::tests::'), exitCode: 0, tests: 7, pass: 7, fail: 0, log: 'native-publication-tests.log' },
], failures: [ {log:'native-probe-original.log', exitCode:101, reason:'Initial source archive omitted two include fixtures; work-only probe binding needed mut'}, {log:'native-probe-first-runtime.log', exitCode:101, reason:'Work-only probe mistakenly forbade wake text in valid owned equipment JSON; replaced by exact snapshot comparison'}, {log:'pin-original-failure.log', exitCode:1, reason:'Verifier initially required literal Git bytes from git archive despite configured CRLF export; final verification distinguishes exact compiled/export bytes and canonical LF Git blobs'}, {log:'pin-count-original-failure.log', exitCode:1, reason:'Expected ZIP count included directory entries; actual 469 compiled files verified individually'} ], scope: 'Actual native configured fixtures and instrumented stage/publication measurements only. No production changes/GPU/browser/listener/RTC/FPS claim; all processes completed.' };
await fs.writeFile(path.join(own, 'source-pins.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ runtimeRef, snapshotRef, nativeFiles: result.nativeSnapshotFiles, tests: [14,7], reportSha256: evidence['REPORT.md'].sha256, nativeBinarySha256: result.nativeBinary.sha256, measurements: records.length, trackedClean:true }, null, 2));
