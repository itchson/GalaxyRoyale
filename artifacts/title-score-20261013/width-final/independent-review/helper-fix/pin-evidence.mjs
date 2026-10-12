import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
const root = new URL("../../../", import.meta.url), own = new URL("./", import.meta.url);
const files = ["scripts/qa/audio-review/review.js", "scripts/qa/audio-review/fixtures.test.mjs"];
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const normalized = (bytes) => Buffer.from(bytes.toString("utf8").replaceAll("\r\n", "\n"));
const head = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
const diff = execFileSync("git", ["diff", "--no-ext-diff", "--binary", "--", ...files], { cwd: root });
await writeFile(new URL("diff.patch", own), diff);
const sources = {};
for (const path of files) {
  const bytes = await readFile(new URL(path, root));
  const committed = execFileSync("git", ["show", `${head}:${path}`], { cwd: root });
  sources[path] = { workingBytes: bytes.length, workingSha256: sha(bytes), normalizedLfBytes: normalized(bytes).length,
    normalizedLfSha256: sha(normalized(bytes)), baseCommit: head, baseGitBytes: committed.length, baseGitSha256: sha(committed) };
}
const oldReview = await readFile(new URL("review-before.js", own));
const captureDir = "work/title-score-native-width-v3/";
const capturePinsBytes = await readFile(new URL(`${captureDir}source-pins.json`, root));
const capturePins = JSON.parse(capturePinsBytes);
const reportPath = `${captureDir}captures/qa-1791826979237-0e10d066/report.json`;
const reportBytes = await readFile(new URL(reportPath, root));
const report = JSON.parse(reportBytes);
const originalReportSha = "5369b426b6d297ce48b9dc0321af7ac3eaa5db8d996b903980c1cc6afc37bba6";
const runtime = {};
for (const [path, expected] of Object.entries(capturePins.inputs)) {
  if (path.startsWith("git:")) continue;
  const bytes = await readFile(new URL(path, root));
  runtime[path] = { bytes: bytes.length, sha256: sha(bytes), equalsPriorCaptureSource: bytes.length === expected.bytes && sha(bytes) === expected.sha256 };
}
const wavs = {};
for (const item of report.cases) {
  const path = `${captureDir}captures/qa-1791826979237-0e10d066/${item.key}.wav`;
  const bytes = await readFile(new URL(path, root));
  wavs[path] = { bytes: bytes.length, sha256: sha(bytes), equalsExistingReportHash: sha(bytes) === item.wav_sha256 };
}
const evidence = {};
for (const path of ["handoff.md", "review-before.js", "regression-before.log", "qa-tests.log", "qa-tests-with-unintended-server-checks.log", "capability-after-format.log", "format.log", "diff-check.log", "diff.patch", "pin-evidence.mjs"]) {
  const bytes = await readFile(new URL(path, own)); evidence[path] = { bytes: bytes.length, sha256: sha(bytes) };
}
const result = { capturedAt: new Date().toISOString(), baseHead: head, node: process.version, modifiedSources: sources, ownEvidence: evidence,
  commands: [
    { command: "node --test --test-name-pattern='actual renderer preserves absent native panner capability' scripts/qa/audio-review/fixtures.test.mjs", stage: "before fix", exitCode: 1, tests: 1, pass: 0, fail: 1, log: "regression-before.log" },
    { command: "node --test --test-name-pattern='^(?!actual loopback HTTP|occupied-port CLI|autoquit closes)' scripts/qa/audio-review/helper.test.mjs scripts/qa/audio-review/fixtures.test.mjs scripts/qa/audio-review/metrics.test.mjs", stage: "after fix; exclusion failed", exitCode: 0, tests: 21, pass: 21, fail: 0, log: "qa-tests-with-unintended-server-checks.log", deviation: "Three ephemeral loopback server checks inadvertently executed, then closed normally." },
    { command: "node --test --test-skip-pattern='actual loopback HTTP|occupied-port CLI|autoquit closes' scripts/qa/audio-review/helper.test.mjs scripts/qa/audio-review/fixtures.test.mjs scripts/qa/audio-review/metrics.test.mjs", stage: "after fix, before final formatting; intended no-server scope", exitCode: 0, tests: 18, pass: 18, fail: 0, excludedServerTests: 3, log: "qa-tests.log" },
    { command: "node node_modules/prettier/bin/prettier.cjs --write scripts/qa/audio-review/review.js scripts/qa/audio-review/fixtures.test.mjs", exitCode: 0, prettier: "3.9.9", log: "format.log" },
    { command: "node --test --test-name-pattern='actual renderer preserves absent native panner capability|exported offline adapter' scripts/qa/audio-review/fixtures.test.mjs", stage: "final formatted source bytes; no server tests", exitCode: 0, tests: 3, pass: 3, fail: 0, log: "capability-after-format.log" },
    { command: "git diff --check -- scripts/qa/audio-review/review.js scripts/qa/audio-review/fixtures.test.mjs", exitCode: 0, log: "diff-check.log" }
  ], priorV3Preservation: { sourcePinsPath: `${captureDir}source-pins.json`, sourcePinsBytes: capturePinsBytes.length, sourcePinsSha256: sha(capturePinsBytes),
    originalHelperSha256: capturePins.helpers["review.js"], beforeCopySha256: sha(oldReview), beforeCopyMatchesCaptureHelper: sha(oldReview) === capturePins.helpers["review.js"],
    reportPath, reportBytes: reportBytes.length, reportSha256: sha(reportBytes), reportMatchesRecordedPreAmendmentHash: sha(reportBytes) === originalReportSha,
    runtimeSources: runtime, preservedWavs: wavs, freshRenderThisCorrection: false },
  scope: "QA-only capability correction and actual helper/analytic tests. No production runtime change, commit, production bundle, broad build, browser, new native DSP render or listening/fidelity acceptance. Execution deviation is preserved explicitly." };
await writeFile(new URL("source-pins.json", own), `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ baseHead: head, sources, commands: result.commands, originalHelperVerified: result.priorV3Preservation.beforeCopyMatchesCaptureHelper,
  priorReportVerified: result.priorV3Preservation.reportMatchesRecordedPreAmendmentHash, runtimeSourcesUnchanged: Object.values(runtime).every((entry) => entry.equalsPriorCaptureSource), wavsVerified: Object.values(wavs).every((entry) => entry.equalsExistingReportHash), wavs: Object.keys(wavs).length, handoffSha256: evidence["handoff.md"].sha256 }, null, 2));
