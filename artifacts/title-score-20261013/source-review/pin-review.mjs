import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";

const root = new URL("../../", import.meta.url);
const own = new URL("./", import.meta.url);
const frozen = JSON.parse(await readFile(new URL("work/title-score-engine/source-pins.json", root), "utf8"));
const hash = (buffer) => createHash("sha256").update(buffer).digest("hex");
const originalCandidate = "4edfd189d4e05f844e91a67a0f93f721800e0565";
const candidate = "8727e7e0295095e338cf907b623174807d14e736";
const paths = [...Object.keys(frozen.sources), "scripts/test/audio-state.test.mjs", ...Object.keys(frozen.evidence).map((path) => `work/title-score-engine/${path}`), "work/title-score-engine/source-pins.json"];
const sources = {};
for (const path of paths) {
  const bytes = await readFile(new URL(path, root));
  const normalized = Buffer.from(bytes.toString("utf8").replaceAll("\r\n", "\n"));
  const committed = execFileSync("git", ["show", `${candidate}:${path}`], { cwd: root });
  const expected = frozen.sources[path] ?? frozen.evidence[path.replace("work/title-score-engine/", "")];
  sources[path] = { bytes: bytes.length, sha256: hash(bytes), lfBytes: normalized.length,
    lfSha256: hash(normalized), candidateBytes: committed.length, candidateSha256: hash(committed),
    frozenExpected: expected ?? null, rawMatchesFrozen: expected ? bytes.length === expected.bytes && hash(bytes) === expected.sha256 : null,
    lfMatchesFrozen: expected ? normalized.length === expected.bytes && hash(normalized) === expected.sha256 : null,
    commitMatchesFrozen: expected ? committed.length === expected.bytes && hash(committed) === expected.sha256 : null,
    workspaceMatchesCandidateLf: normalized.equals(Buffer.from(committed.toString("utf8").replaceAll("\r\n", "\n"))) };
}
const evidence = {};
for (const path of ["review.md", "focused-tests.log", "edge-review.log", "edge-review-original-failure.log", "focused-tests-original.log", "review-pins-original.json", "edge-review.test.mjs", "recorded-api-harness.mjs", "pin-review.mjs"] ) {
  const bytes = await readFile(new URL(path, own));
  evidence[path] = { bytes: bytes.length, sha256: hash(bytes) };
}
const typecheckText = (await readFile(new URL("work/title-score-engine/typecheck.log", root), "utf8")).replaceAll("\r\n", "\n");
const typecheckLines = typecheckText.split("\n");
const typecheckNewlineMatches = [];
for (let mask = 0; mask < 2 ** (typecheckLines.length - 1); mask++) {
  const candidateBytes = Buffer.from(typecheckLines.map((line, i) => line + (i === typecheckLines.length - 1 ? "" : mask & (1 << i) ? "\r\n" : "\n")).join(""));
  if (hash(candidateBytes) === frozen.evidence["typecheck.log"].sha256)
    typecheckNewlineMatches.push({ newlineMask: mask, bytes: candidateBytes.length, sha256: hash(candidateBytes) });
}
const result = { capturedAt: new Date().toISOString(), candidateCommit: candidate, originalCandidateCommit: originalCandidate,
  currentHead: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
  node: process.version, sourcePins: sources, ownEvidence: evidence,
  frozenTypecheckEquivalentTextMatches: typecheckNewlineMatches,
  commands: [
    { command: "node --experimental-transform-types --test scripts/test/music-clock.test.mjs scripts/test/title-theme.test.mjs scripts/test/audio-state.test.mjs scripts/test/audio.test.mjs", exitCode: 0, tests: 49, pass: 49, fail: 0, log: "focused-tests.log" },
    { command: "node --experimental-transform-types --test work/title-score-independent-review/edge-review.test.mjs", exitCode: 0, tests: 2, pass: 2, fail: 0, log: "edge-review.log" }
  ], scope: "Source review and recorded WebAudio API tests only. No native DSP, listening, browser, gameplay, device, or full build validation." };
await writeFile(new URL("review-pins.json", own), `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ sources: Object.entries(sources).map(([path, value]) => ({ path, rawMatchesFrozen: value.rawMatchesFrozen, lfMatchesFrozen: value.lfMatchesFrozen, commitMatchesFrozen: value.commitMatchesFrozen, workspaceMatchesCandidateLf: value.workspaceMatchesCandidateLf })), ownEvidence: evidence, frozenTypecheckEquivalentTextMatches: typecheckNewlineMatches }, null, 2));
