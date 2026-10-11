import test from "node:test";
import assert from "node:assert/strict";
import { IDBFactory } from "fake-indexeddb";
import {
  SaveStore,
  SaveConflict,
  validProfile,
  normalizeProfile,
  emptyCampaign,
} from "../../apps/game/src/storage.ts";

const old = () => ({
  schema_version: 4,
  revision: 21,
  seed: 12345,
  currency: [10, 20, 30],
  ranks: Array(9).fill(0),
  unlocked: [false, false, false],
  kills: [0, 0, 0],
  boss_kills: [0, 0, 0],
  boss_generations: Array.from({ length: 3 }, () => Array(5).fill(0)),
  pity: [0, 0, 0],
  pending: [false, false, false],
  discoveries: [],
  quests: [],
  relics: [],
  campaign: emptyCampaign(),
});
function licensed(corporation = "pan") {
  const p = normalizeProfile(old());
  p.identity = {
    kind: "talo",
    corporation,
    sub_corporation: "renko",
    hex_id: "0123456789abcdef",
  };
  p.onboarding = { stage: 0, flags: 0 };
  p.campaign.abilities.slice(0, 3).forEach((a) => {
    a.blueprint = false;
  });
  const ability = { pan: 2, renko: 0, hugo: 1 }[corporation];
  p.campaign.abilities[ability] = { blueprint: true, rank: 1 };
  p.campaign.loadouts[0].slots[ability] = ability;
  return p;
}
const request = (r) =>
  new Promise((resolve, reject) => {
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });

test("schema4 upgrades preserve actual progress without a license or starter grant", () => {
  const p = old();
  p.campaign.ship_xp[0] = 120;
  p.campaign.stored_materials[0] = 7;
  const migrated = normalizeProfile(p);
  assert.equal(migrated.schema_version, 5);
  assert.equal(migrated.identity, null);
  assert.equal(migrated.onboarding, null);
  assert.deepEqual(migrated.campaign, p.campaign);
  assert.deepEqual(migrated.currency, p.currency);
  assert.equal(migrated.revision, p.revision);
});
test("three valid licensed starters retain one real equipped item through normalization", () => {
  for (const corporation of ["pan", "renko", "hugo"]) {
    const p = licensed(corporation);
    assert.equal(validProfile(p), true);
    assert.deepEqual(normalizeProfile(p), p);
    assert.equal(p.campaign.abilities.filter((a) => a.rank > 0).length, 1);
  }
});
test("identity and onboarding validation rejects malformed keys and impossible stages", () => {
  for (const mutate of [
    (p) => delete p.identity,
    (p) => (p.identity.kind = ["dora"]),
    (p) => (p.identity.corporation = "ranko"),
    (p) => delete p.identity.sub_corporation,
    (p) => (p.identity.hex_id = "1337"),
    (p) => (p.identity.hex_id = "0123456789ABCDEF"),
    (p) => (p.onboarding.stage = 6),
    (p) => (p.onboarding.flags = 128),
    (p) => {
      p.identity = null;
    },
  ]) {
    const p = licensed();
    mutate(p);
    assert.equal(validProfile(p), false);
  }
  for (const [flags, stage] of [
    [0, 0],
    [1, 1],
    [3, 2],
    [7, 3],
    [15, 4],
    [31, 5],
    [63, 5],
    [127, 6],
    [96, 0],
  ]) {
    const p = licensed();
    p.onboarding = { flags, stage };
    assert.equal(validProfile(p), true);
  }
});
test("licensed save reload and backup recovery preserve hash and exactly one starter", async (t) => {
  globalThis.indexedDB = new IDBFactory();
  const first = new SaveStore();
  await first.open();
  t.after(() => first.db?.close());
  const p = licensed();
  await first.save(p);
  const next = structuredClone(p);
  next.revision++;
  next.onboarding = { flags: 1, stage: 1 };
  await first.save(next);
  const db = await request(indexedDB.open("galaxy-royale"));
  const tx = db.transaction("profile", "readwrite");
  tx.objectStore("profile").put(
    { ...next, identity: { ...next.identity, hex_id: "broken" } },
    "current",
  );
  await new Promise((resolve, reject) => {
    tx.oncomplete = resolve;
    tx.onabort = () => reject(tx.error);
  });
  db.close();
  const recovered = new SaveStore();
  t.after(() => recovered.db?.close());
  assert.deepEqual(await recovered.open(), p);
  assert.match(recovered.message, /Recovered/);
  await recovered.save({ ...p, revision: p.revision + 2 });
  assert.equal(
    (await recovered.open()).campaign.abilities.filter((a) => a.rank > 0)
      .length,
    1,
  );
});
test("schema5 barrier waits for cached schema4 connection and rejects its subsequent writer", async (t) => {
  globalThis.indexedDB = new IDBFactory();
  const opening = indexedDB.open("galaxy-royale", 2);
  opening.onupgradeneeded = () => opening.result.createObjectStore("profile");
  const cached = await request(opening);
  t.after(() => cached.close());
  const tx = cached.transaction("profile", "readwrite");
  tx.objectStore("profile").put(old(), "current");
  await new Promise((resolve) => {
    tx.oncomplete = resolve;
  });
  let blocked;
  const waited = new Promise((resolve) => (blocked = resolve));
  const current = new SaveStore();
  t.after(() => current.db?.close());
  const load = current.open(() => blocked());
  await waited;
  cached.close();
  const migrated = await load;
  assert.equal(migrated.identity, null);
  assert.equal(current.db.version, 3);
  await assert.rejects(request(indexedDB.open("galaxy-royale", 2)), /version/i);
});
