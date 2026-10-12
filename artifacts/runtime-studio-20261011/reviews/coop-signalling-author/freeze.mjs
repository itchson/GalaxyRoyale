import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";

const cwd = resolve(import.meta.dirname, "../..");
const git = (...args) => execFileSync("git", args, { cwd });
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const commit = git("rev-parse", "HEAD").toString().trim();
const parent = git("rev-parse", "HEAD^").toString().trim();
const owned = ["apps/coop/store.mjs", "apps/coop/server.mjs", "apps/coop/README.md", "scripts/test/coop-signalling.test.mjs"];
const source = [];
mkdirSync(resolve(import.meta.dirname, "source"), { recursive: true });
for (const path of [...owned, "AGENTS.md", "package.json", "pnpm-lock.yaml"]) {
  const committed = git("show", `${commit}:${path}`);
  const working = readFileSync(resolve(cwd, path));
  const snapshot = `source/${path.replaceAll("/", "__")}`;
  writeFileSync(resolve(import.meta.dirname, snapshot), committed);
  source.push({ path, committed_sha256: sha(committed), working_sha256: sha(working), bytes: committed.length, snapshot });
}
const evidence = ["first-tests.log", "second-tests.log", "third-tests.log", "copy-paste-rate-tests.log", "copy-paste-rate-fixed-tests.log", "final-tests.log", "HANDOFF.md"]
  .map((path) => { const bytes = readFileSync(resolve(import.meta.dirname, path)); return { path, sha256: sha(bytes), bytes: bytes.length }; });
const manifest = { commit, parent, node: process.version, platform: process.platform, architecture: process.arch,
  owned, source, evidence, verification: "20 focused tests passed; temporary HTTP listeners only; no RTC/gameplay/browser/public deployment acceptance" };
writeFileSync(resolve(import.meta.dirname, "frozen-hashes.json"), JSON.stringify(manifest, null, 2) + "\n");
console.log(JSON.stringify({ commit, parent, source_pins: source.length, evidence_pins: evidence.length, manifest_sha256: sha(readFileSync(resolve(import.meta.dirname, "frozen-hashes.json"))) }));
