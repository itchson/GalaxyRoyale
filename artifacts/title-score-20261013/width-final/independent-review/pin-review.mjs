import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
const root = new URL("../../", import.meta.url), own = new URL("./", import.meta.url);
const candidate = "a1c16b68c5312822ae4efc214036825ac6471d8f";
const frozenRoot = "f4b8882b568339f0aa164c6aaf99a174b59cc105";
const base = "d028f0d7a3c589b5896f536f08e76aa504cb2923";
const authorDir = "work/title-instrument-width/";
const authorPins = JSON.parse(await readFile(new URL(`${authorDir}source-pins.json`, root), "utf8"));
const authorExecution = JSON.parse(await readFile(new URL(`${authorDir}execution-evidence.json`, root), "utf8"));
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const normalize = (bytes) => Buffer.from(bytes.toString("utf8").replaceAll("\r\n", "\n"));
const gitBlob = (revision, path) => execFileSync("git", ["show", `${revision}:${path}`], { cwd: root });
const sources = {};
const paths = [...Object.keys(authorPins.sources), ...Object.keys(authorExecution.unchanged_baseline_modules), ...Object.keys(authorPins.evidence).map((name) => `${authorDir}${name}`), `${authorDir}source-pins.json`, "scripts/qa/audio-review/review.js"];
for (const path of paths) {
  const bytes = await readFile(new URL(path, root)), lf = normalize(bytes);
  const committed = gitBlob(candidate, path), frozen = gitBlob(frozenRoot, path);
  const expected = authorPins.sources[path] ?? authorPins.evidence[path.replace(authorDir, "")] ?? null;
  const baseline = authorExecution.unchanged_baseline_modules[path];
  sources[path] = { workingBytes: bytes.length, workingSha256: sha(bytes), normalizedLfBytes: lf.length, normalizedLfSha256: sha(lf),
    candidateGitBytes: committed.length, candidateGitSha256: sha(committed), frozenRootGitBytes: frozen.length, frozenRootGitSha256: sha(frozen),
    authorExpected: expected, rawMatchesAuthor: expected ? bytes.length === expected.bytes && sha(bytes) === expected.sha256 : null,
    lfMatchesAuthor: expected ? lf.length === expected.bytes && sha(lf) === expected.sha256 : null,
    gitMatchesAuthor: expected ? committed.length === expected.bytes && sha(committed) === expected.sha256 : null,
    workingMatchesCandidateCanonical: lf.equals(normalize(committed)), workingMatchesFrozenRootCanonical: lf.equals(normalize(frozen)),
    candidateMatchesFrozenRootCanonical: normalize(committed).equals(normalize(frozen)),
    unchangedBaseline: baseline ? { recorded: baseline, actuallyEqualToBase: normalize(committed).equals(normalize(gitBlob(base, path))), actuallyMatchesCanonicalHash: sha(committed) === baseline.canonical_sha256 } : null };
}
const evidence = {};
for (const path of ["review.md", "focused-tests.log", "edge-review.log", "edge-review.test.mjs", "recorded-api-harness.mjs", "pin-review.mjs"]) {
  const bytes = await readFile(new URL(path, own)); evidence[path] = { bytes: bytes.length, sha256: sha(bytes) };
}
const typecheck = normalize(await readFile(new URL(`${authorDir}typecheck.log`, root))).toString("utf8").split("\n");
const mixedNewlineMatches = [];
for (let mask = 0; mask < 2 ** (typecheck.length - 1); mask++) {
  const bytes = Buffer.from(typecheck.map((line, n) => line + (n === typecheck.length - 1 ? "" : mask & (1 << n) ? "\r\n" : "\n")).join(""));
  if (sha(bytes) === authorPins.evidence["typecheck.log"].sha256) mixedNewlineMatches.push({ newlineMask: mask, bytes: bytes.length, sha256: sha(bytes) });
}
const result = { capturedAt: new Date().toISOString(), reviewedCandidate: candidate, reviewedFrozenRoot: frozenRoot, authorBase: base,
  actualHeadAtCapture: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(), node: process.version,
  sources, ownEvidence: evidence, authorTypecheckEquivalentTextMatches: mixedNewlineMatches,
  commands: [
    { command: "node --experimental-transform-types --test scripts/test/music-clock.test.mjs scripts/test/title-theme.test.mjs scripts/test/audio-state.test.mjs scripts/test/audio.test.mjs", exitCode: 0, tests: 58, pass: 58, fail: 0, log: "focused-tests.log" },
    { command: "node --experimental-transform-types --test work/title-width-independent-review/edge-review.test.mjs", exitCode: 0, tests: 3, pass: 3, fail: 0, log: "edge-review.log" }
  ], acceptance: { sourceAndRecordedApi: "no concrete runtime blocker found", qaLegacyMissingPanner: "adapter limitation recorded", nativeDsp: "not performed by this reviewer", listening: "open", physicalDevice: "not tested", referenceFidelity: "open" },
  scope: "Independent title width source/API review. Native DSP, listening, reference fidelity and physical-device acceptance remain separate." };
await writeFile(new URL("review-pins.json", own), `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ reviewedCandidate: candidate, frozenRoot, actualHead: result.actualHeadAtCapture,
  sources: Object.fromEntries(Object.entries(sources).map(([path, item]) => [path, { rawMatchesAuthor: item.rawMatchesAuthor, lfMatchesAuthor: item.lfMatchesAuthor, gitMatchesAuthor: item.gitMatchesAuthor, workingMatchesCandidateCanonical: item.workingMatchesCandidateCanonical, workingMatchesFrozenRootCanonical: item.workingMatchesFrozenRootCanonical, candidateMatchesFrozenRootCanonical: item.candidateMatchesFrozenRootCanonical, unchangedBaseline: item.unchangedBaseline }])), ownEvidence: evidence, authorTypecheckEquivalentTextMatches: mixedNewlineMatches }, null, 2));
