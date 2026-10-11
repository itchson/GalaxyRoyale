import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(process.argv[2] ?? "C:/Users/mitch/Documents/ChatGPT/galaxy-royale");
const storagePath = resolve(root, "apps/game/src/storage.ts");
const { validProfile, normalizeProfile, SaveStore } = await import(pathToFileURL(storagePath));
const require = createRequire(resolve(root, "package.json"));
const { IDBFactory } = require("fake-indexeddb");
const original = JSON.parse(readFileSync(resolve(here, "rejected-original.json"), "utf8"));
const fixture = JSON.parse(readFileSync(resolve(here, "all8-earned.json"), "utf8"));
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const checks = [];
const check = (name, run) => { run(); checks.push({ name, passed: true }); };

check("Retained original is rejected by actual Root validProfile", () => {
  assert.equal(validProfile(original), false);
  assert.equal(normalizeProfile(original), null);
});
check("Minimal repair changes only eight champion ability blueprint flags", () => {
  const expected = structuredClone(original);
  for (let id = 6; id < 14; id++) expected.campaign.abilities[id].blueprint = true;
  assert.deepEqual(fixture, expected);
});
check("Repaired fixture passes actual Root validProfile and normalizes without changes", () => {
  assert.equal(validProfile(fixture), true);
  assert.deepEqual(normalizeProfile(fixture), fixture);
  assert.deepEqual(normalizeProfile(normalizeProfile(fixture)), fixture);
});
check("Every champion patch/ability inconsistency remains rejected", () => {
  for (let i = 0; i < 8; i++) {
    const broken = structuredClone(fixture);
    broken.campaign.abilities[i + 6].blueprint = false;
    assert.equal(validProfile(broken), false, `missing champion blueprint ${i}`);
    const mismatchedPatch = structuredClone(fixture);
    mismatchedPatch.campaign.patches[i] = false;
    assert.equal(validProfile(mismatchedPatch), false, `patch mismatch ${i}`);
  }
});
check("Fixture has eight supplied earned patches, no champion ownership or Royale grants", () => {
  assert.equal(fixture.campaign.patches.filter(Boolean).length, 8);
  assert.ok(fixture.campaign.abilities.slice(6, 14).every((a) => a.blueprint && a.rank === 0));
  assert.ok(fixture.campaign.crafted.every((v) => !v));
  assert.ok(fixture.campaign.royale_results.every((v) => v === 0));
  assert.equal(fixture.campaign.victory_claimed, false);
  assert.equal(fixture.campaign.credits_seen, false);
  assert.equal(fixture.campaign.selected_ship, 0);
});

const previousIndexedDB = globalThis.indexedDB;
globalThis.indexedDB = new IDBFactory();
const store = new SaveStore();
const reloaded = new SaveStore();
try {
  assert.equal(await store.open(), null);
  await store.replace(fixture);
  await store.flush();
  assert.deepEqual(await reloaded.open(), fixture);
  checks.push({ name: "Actual Root SaveStore.replace/open keeps the exact fixture on an isolated in-memory IndexedDB adapter", passed: true });
} finally {
  // These connections belong only to the test adapter, never the browser/user DB.
  store.db?.close();
  reloaded.db?.close();
  if (previousIndexedDB === undefined) delete globalThis.indexedDB;
  else globalThis.indexedDB = previousIndexedDB;
}

const report = {
  scope: "Supplied QA progression fixture only; not naturally earned. Actual Root TypeScript validator/normalizer/SaveStore executed; IndexedDB is explicitly an isolated in-memory test adapter, not a browser acceptance claim.",
  node: process.version,
  storage_source: { path: storagePath, sha256: sha(readFileSync(storagePath)) },
  fixture: { path: resolve(here, "all8-earned.json"), sha256: sha(readFileSync(resolve(here, "all8-earned.json"))) },
  rejected_original: { sha256: sha(readFileSync(resolve(here, "rejected-original.json"))) },
  checks,
};
writeFileSync(resolve(here, "validation.json"), JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify(report, null, 2));
