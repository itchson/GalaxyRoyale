import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const root = "C:/Users/mitch/Documents/ChatGPT/galaxy-royale";
const commit = "69db89ccf5895390807383fcdd37c3413ba2dc68";
const out = import.meta.dirname;
const sha = (data) => createHash("sha256").update(data).digest("hex");
const files = [
  "apps/game/index.html",
  "apps/game/src/main.ts",
  "apps/game/src/style.css",
  "apps/game/src/meishi.ts",
  "apps/game/src/meishi.css",
  "apps/game/src/pixel-ui.ts",
  "apps/game/src/loadout.ts",
  "apps/game/src/loadout.css",
  "apps/game/src/campaign.ts",
  "scripts/test/meishi.test.mjs",
  "scripts/test/loadout.test.mjs",
  "scripts/test/flight-menu.test.mjs",
  "scripts/test/pixel-ui.test.mjs",
  "scripts/test/dock-menu.test.mjs",
];
const inputs = files.map((file) => {
  const committed = execFileSync("git", ["show", `${commit}:${file}`], {
    cwd: root,
    maxBuffer: 8 * 1024 * 1024,
  });
  const live = readFileSync(resolve(root, file));
  const snapshot = resolve(out, "committed-inputs", file);
  mkdirSync(resolve(snapshot, ".."), { recursive: true });
  writeFileSync(snapshot, committed);
  if (committed.toString("utf8") !== live.toString("utf8").replaceAll("\r\n", "\n")) {
    throw new Error(`Runtime source moved: ${file}`);
  }
  return {
    file,
    committed_sha256: sha(committed),
    committed_bytes: committed.length,
    observed_working_sha256: sha(live),
    observed_working_bytes: live.length,
    working_equal_after_crlf_normalization: true,
  };
});
const evidence = [
  "REVIEW.md",
  "scroll-probe-prepare.mjs",
  "scroll-probe.mjs",
  "scroll-probe.log",
  "inputs/apps/game/src/loadout.ts",
].map((file) => {
  const bytes = readFileSync(resolve(out, file));
  return { file, sha256: sha(bytes), bytes: bytes.length };
});
const tested = readFileSync(resolve(out, "inputs/apps/game/src/loadout.ts"), "utf8");
const final = readFileSync(resolve(root, "apps/game/src/loadout.ts"), "utf8");
if (tested.replaceAll(/\s+/g, "") !== final.replaceAll(/\s+/g, "")) {
  throw new Error("Tested scroll source changed beyond whitespace");
}
const manifest = {
  root_commit: commit,
  initial_route_commit: "e6be2e847bedd1e477e4f3576b7e6bf654f7492b",
  route_runtime_commit: "2bab5b1e3b8a14d813f4827ff5623d09d1af7427",
  native_surface_commit: "293f41c",
  keyboard_scroll_commit: "69db89ccf5895390807383fcdd37c3413ba2dc68",
  scope: "Read-only source route review and three actual painter fake-DOM scroll probes; no browser/native execution",
  final_scroll_source_changes_from_executed_snapshot: "Prettier wraps previousScroll assignment only; no token changes",
  inputs,
  evidence,
};
writeFileSync(resolve(out, "source-hashes.json"), JSON.stringify(manifest, null, 2) + "\n");
console.log(JSON.stringify({ commit, inputs: inputs.length, evidence: evidence.length,
  report_sha256: evidence[0].sha256,
  manifest_sha256: sha(readFileSync(resolve(out, "source-hashes.json"))) }, null, 2));
