import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
const root = resolve(import.meta.dirname, "../..");
const out = import.meta.dirname;
const commit = "37cdd564b55476b242f65660518d72af159e9c7b";
const hash = bytes => createHash("sha256").update(bytes).digest("hex");
const git = (...args) => execFileSync("git", args, { cwd: root });
const changed = git("diff-tree", "--no-commit-id", "--name-only", "-r", commit).toString().trim().split("\n");
const paths = [...changed, "apps/game/src/main.ts", "apps/game/src/pixel-ui.ts", "crates/game_core/src/ship_progression.rs", "crates/game_core/src/abilities.rs", "crates/game_core/src/campaign.rs", "package.json", "pnpm-lock.yaml", "tsconfig.json"];
const inputs = paths.map(path => {
  const bytes = git("show", `${commit}:${path}`), working = readFileSync(resolve(root, path));
  const snapshot = resolve(out, "committed-inputs", path);
  mkdirSync(dirname(snapshot), { recursive: true });
  writeFileSync(snapshot, bytes);
  return { path, committed_bytes: bytes.length, committed_sha256: hash(bytes), executed_working_bytes: working.length, executed_working_sha256: hash(working) };
});
const evidence = ["HANDOFF.md", "first-focused-fail.log", "new-filter-fixture-fail.log", "focused-tests.log", "full-shell-tests.log", "typecheck.log", "freeze.mjs"].map(path => {
  const bytes = readFileSync(resolve(out, path));
  return { path, bytes: bytes.length, sha256: hash(bytes) };
});
const manifest = { format: 1, commit, parent: git("rev-parse", `${commit}^`).toString().trim(), tracked_status: git("status", "--short").toString(), inputs, evidence, tests: { focused: 33, full_pass: 404, full_skip: 1, typecheck: "pass" }, limits: "Actual TS/shell functions with fake adapters, not Rust transaction, browser/native UI, GPU/phone or gameplay acceptance." };
if (manifest.tracked_status) throw new Error("Tracked source changed after commit");
writeFileSync(resolve(out, "frozen-hashes.json"), JSON.stringify(manifest, null, 2) + "\n");
console.log(JSON.stringify({ commit, input_pins: inputs.length, evidence_pins: evidence.length, handoff_sha256: evidence.find(x => x.path === "HANDOFF.md").sha256, manifest_sha256: hash(readFileSync(resolve(out, "frozen-hashes.json"))) }, null, 2));
