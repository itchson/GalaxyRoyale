import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";
import { dirname } from "node:path";
const root = "C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale";
const out = resolve(import.meta.dirname);
const commit = process.argv[2];
if (!/^[0-9a-f]{40}$/.test(commit ?? "")) throw new Error("Exact final Art commit required");
const hash = (b) => createHash("sha256").update(b).digest("hex");
const git = (...args) => execFileSync("git", args, { cwd: root, maxBuffer: 40 * 1024 * 1024 });
const paths = git("ls-tree", "-r", "--name-only", commit, "assets/pipeline/pan-coop-dock/native256", "scripts/qa/pan-coop-dock/native256", "scripts/qa/pan-coop-dock/convert_256.py", "scripts/qa/pan-coop-dock/test_256.py", "scripts/qa/pan-coop-dock/verify_256.py", "scripts/qa/pan-coop-dock/archive_256.py").toString().trim().split("\n");
paths.push("scripts/qa/pan-coop-dock/convert.py", "scripts/asset-pack.py", "assets/pipeline/pan-coop-dock/source.png", "crates/engine_core/src/frame.rs", "crates/engine_core/src/pixels.rs", "crates/engine_core/src/pixel_rig.rs", "crates/engine_core/src/camera.rs", "crates/engine_core/src/transform.rs");
const inputs = paths.map(path => {
  const bytes = git("show", `${commit}:${path}`);
  const snapshot = resolve(out, "committed-inputs", path);
  mkdirSync(dirname(snapshot), { recursive: true });
  writeFileSync(snapshot, bytes);
  return { path, bytes: bytes.length, sha256: hash(bytes) };
});
const evidence = ["python-tests.log", "native-tests.log", "clippy.log", "verify.log", "body-run.log", "rig-run.log", "python-tests-archive-fixture-error.log"].map(name => {
  const bytes = readFileSync(resolve(root, "work/pan-coop-dock256", name));
  mkdirSync(resolve(out, "peer-logs"), { recursive: true });
  writeFileSync(resolve(out, "peer-logs", name), bytes);
  return { path: `peer-logs/${name}`, bytes: bytes.length, sha256: hash(bytes), attribution: "Art owner executed; independently read, not rerun" };
});
for (const path of ["inspect.mjs", "packet-inspection.json", "inspection.log", "REVIEW.md"]) {
  const bytes = readFileSync(resolve(out, path));
  evidence.push({ path, bytes: bytes.length, sha256: hash(bytes), attribution: "Independent source/retained-packet review" });
}
writeFileSync(resolve(out, "frozen-hashes.json"), JSON.stringify({ format: 1, commit, root, inputs, evidence, scope: "Read-only source, source-art and actual retained native packets. No native rebuild, Game, GPU, services, co-op or phone acceptance." }, null, 2) + "\n");
console.log(JSON.stringify({ commit, input_pins: inputs.length, evidence_pins: evidence.length, report_sha256: evidence.find(p => p.path === "REVIEW.md").sha256, manifest_sha256: hash(readFileSync(resolve(out, "frozen-hashes.json"))) }, null, 2));
