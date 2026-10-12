import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";

const root = new URL("../../", import.meta.url);
const own = new URL("./", import.meta.url);
const candidate = "51b529f4c82c1abb8a0c8bfb386263c5c81cc1cc";
const frozenRoot = "d028f0d7a3c589b5896f536f08e76aa504cb2923";
const base = "d53954c7956a2be71de34de12c4f309563bd8a40";
const authorPins = JSON.parse(await readFile(new URL("work/title-mix-refinement/source-pins.json", root), "utf8"));
const authorExecution = JSON.parse(await readFile(new URL("work/title-mix-refinement/execution-evidence.json", root), "utf8"));
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const normalize = (bytes) => Buffer.from(bytes.toString("utf8").replaceAll("\r\n", "\n"));
const gitBlob = (revision, path) => execFileSync("git", ["show", `${revision}:${path}`], { cwd: root });
const paths = [...Object.keys(authorPins.sources), ...Object.keys(authorExecution.unchanged_baseline_modules), ...Object.keys(authorPins.evidence).map((name) => `work/title-mix-refinement/${name}`), "work/title-mix-refinement/source-pins.json"];
const sources = {};
for (const path of paths) {
  const bytes = await readFile(new URL(path, root));
  const lf = normalize(bytes);
  const committed = gitBlob(candidate, path);
  const frozen = gitBlob(frozenRoot, path);
  const expected = authorPins.sources[path] ?? authorPins.evidence[path.replace("work/title-mix-refinement/", "")] ?? null;
  const baseline = authorExecution.unchanged_baseline_modules[path];
  sources[path] = { workingBytes: bytes.length, workingSha256: sha(bytes), normalizedLfBytes: lf.length,
    normalizedLfSha256: sha(lf), canonicalGitBytes: committed.length, canonicalGitSha256: sha(committed),
    authorExpected: expected, rawMatchesAuthor: expected ? bytes.length === expected.bytes && sha(bytes) === expected.sha256 : null,
    lfMatchesAuthor: expected ? lf.length === expected.bytes && sha(lf) === expected.sha256 : null,
    gitMatchesAuthor: expected ? committed.length === expected.bytes && sha(committed) === expected.sha256 : null,
    workingMatchesCandidateCanonical: lf.equals(normalize(committed)), candidateMatchesFrozenRootCanonical: normalize(committed).equals(normalize(frozen)),
    unchangedBaseline: baseline ? { recorded: baseline, actuallyEqualToBase: normalize(committed).equals(normalize(gitBlob(base, path))), actuallyMatchesCanonicalHash: sha(committed) === baseline.canonical_sha256 } : null };
}
const evidence = {};
for (const path of ["review.md", "focused-tests.log", "edge-review.log", "edge-review.test.mjs", "recorded-api-harness.mjs", "pin-review.mjs"]) {
  const bytes = await readFile(new URL(path, own));
  evidence[path] = { bytes: bytes.length, sha256: sha(bytes) };
}
const typecheck = normalize(await readFile(new URL("work/title-mix-refinement/typecheck.log", root))).toString("utf8").split("\n");
const mixedNewlineMatches = [];
for (let mask = 0; mask < 2 ** (typecheck.length - 1); mask++) {
  const bytes = Buffer.from(typecheck.map((line, n) => line + (n === typecheck.length - 1 ? "" : mask & (1 << n) ? "\r\n" : "\n")).join(""));
  if (sha(bytes) === authorPins.evidence["typecheck.log"].sha256) mixedNewlineMatches.push({ newlineMask: mask, bytes: bytes.length, sha256: sha(bytes) });
}
const result = { capturedAt: new Date().toISOString(), reviewedCandidate: candidate, reviewedFrozenRoot: frozenRoot, authorBase: base,
  actualHeadAtCapture: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(), node: process.version,
  sources, ownEvidence: evidence, authorTypecheckEquivalentTextMatches: mixedNewlineMatches,
  commands: [
    { command: "node --experimental-transform-types --test scripts/test/music-clock.test.mjs scripts/test/title-theme.test.mjs scripts/test/audio-state.test.mjs scripts/test/audio.test.mjs", exitCode: 0, tests: 54, pass: 54, fail: 0, log: "focused-tests.log" },
    { command: "node --experimental-transform-types --test work/title-mix-independent-review/edge-review.test.mjs", exitCode: 0, tests: 4, pass: 4, fail: 0, log: "edge-review.log" }
  ], acceptance: { sourceAndRecordedApi: "no concrete blocker found", nativeDsp: "not performed by this reviewer", listening: "open", physicalDevice: "not tested", exactSong: "open" },
  scope: "Independent title-only source/API review. No native DSP, listening, browser, gameplay, server, broad build, worker or physical-device acceptance." };
await writeFile(new URL("review-pins.json", own), `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ reviewedCandidate: candidate, frozenRoot, actualHead: result.actualHeadAtCapture,
  sources: Object.fromEntries(Object.entries(sources).map(([path, item]) => [path, { rawMatchesAuthor: item.rawMatchesAuthor, lfMatchesAuthor: item.lfMatchesAuthor, gitMatchesAuthor: item.gitMatchesAuthor, workingMatchesCandidateCanonical: item.workingMatchesCandidateCanonical, candidateMatchesFrozenRootCanonical: item.candidateMatchesFrozenRootCanonical, unchangedBaseline: item.unchangedBaseline }])), ownEvidence: evidence, authorTypecheckEquivalentTextMatches: mixedNewlineMatches }, null, 2));
