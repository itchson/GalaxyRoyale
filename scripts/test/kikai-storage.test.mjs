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
  p.onboarding = {
    version: 2,
    stage: 0,
    flags: 0,
    asteroids_destroyed: 0,
    materials_collected: 0,
    legacy_complete: false,
  };
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
  assert.equal(migrated.schema_version, 6);
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
    [127, 6],
    [4, 0],
  ]) {
    const p = licensed();
    p.onboarding = {
      version: 2,
      flags,
      stage,
      asteroids_destroyed: flags & 4 ? 10 : 0,
      materials_collected: flags & 8 ? 10 : 0,
      legacy_complete: false,
    };
    if (flags === 127) p.campaign.abilities[2].rank = 2;
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
  next.onboarding = { ...p.onboarding, flags: 1, stage: 1 };
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
test("schema6 barrier waits for cached schema5 connection and rejects its subsequent writer", async (t) => {
  globalThis.indexedDB = new IDBFactory();
  const opening = indexedDB.open("galaxy-royale", 3);
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
  assert.equal(current.db.version, 4);
  await assert.rejects(request(indexedDB.open("galaxy-royale", 3)), /version/i);
});

test("schema5 tutorial migration preserves the license and earned assets without invented training counts", () => {
  for (const flags of [0, 1, 3, 7, 15, 31, 63, 127]) {
    const p = licensed("renko");
    p.schema_version = 5;
    const stages = { 0: 0, 1: 1, 3: 2, 7: 3, 15: 4, 31: 5, 63: 5, 127: 6 };
    p.onboarding = { flags, stage: stages[flags] };
    delete p.campaign.cargo_training_materials;
    delete p.campaign.stored_training_materials;
    p.campaign.ship_xp[0] = 80;
    p.campaign.stored_materials[0] = 14;
    const original = structuredClone(p);
    const migrated = normalizeProfile(p);
    assert.equal(validProfile(migrated), true);
    assert.deepEqual(p, original);
    assert.deepEqual(migrated.identity, p.identity);
    assert.deepEqual(migrated.currency, p.currency);
    assert.deepEqual(migrated.campaign.ship_xp, p.campaign.ship_xp);
    assert.deepEqual(migrated.campaign.abilities, p.campaign.abilities);
    assert.equal(migrated.onboarding.flags, flags === 127 ? 127 : flags & 3);
    assert.equal(migrated.onboarding.legacy_complete, flags === 127);
    assert.equal(migrated.onboarding.asteroids_destroyed, 0);
    assert.equal(migrated.onboarding.materials_collected, 0);
    assert.deepEqual(migrated.campaign.stored_training_materials, [0, 0, 0]);
  }
});

test("named training materials are bounded Tier1 subsets and impossible mission checkpoints are rejected", () => {
  const p = licensed();
  p.campaign.cargo_materials[0] = 8;
  p.campaign.stored_materials[0] = 15;
  p.campaign.cargo_training_materials = [3, 2, 1];
  p.campaign.stored_training_materials = [10, 2, 0];
  p.onboarding = {
    version: 2,
    stage: 3,
    flags: 7,
    asteroids_destroyed: 10,
    materials_collected: 6,
    legacy_complete: false,
  };
  assert.equal(validProfile(p), true);
  assert.deepEqual(normalizeProfile(p), p);
  for (const mutate of [
    (p) => (p.campaign.cargo_training_materials = [5, 3, 1]),
    (p) => (p.campaign.stored_training_materials = [14, 2, 0]),
    (p) => delete p.campaign.cargo_training_materials,
    (p) => (p.campaign.cargo_training_materials = Array(3)),
    (p) => (p.campaign.stored_training_materials = Array(3)),
    (p) => (p.campaign.cargo_training_materials = [-1, 0, 0]),
    (p) => (p.campaign.cargo_training_materials = [1.5, 0, 0]),
    (p) => (p.onboarding.version = 1),
    (p) => (p.onboarding.asteroids_destroyed = 11),
    (p) => (p.onboarding.asteroids_destroyed = 9),
    (p) => (p.onboarding.materials_collected = 10),
    (p) => (p.onboarding = { ...p.onboarding, flags: 31, stage: 5 }),
    (p) =>
      (p.onboarding = {
        ...p.onboarding,
        flags: 63,
        stage: 5,
        materials_collected: 10,
      }),
    (p) =>
      (p.onboarding = {
        ...p.onboarding,
        flags: 127,
        stage: 6,
        materials_collected: 10,
      }),
    (p) => (p.onboarding.legacy_complete = true),
  ]) {
    const bad = structuredClone(p);
    mutate(bad);
    assert.equal(validProfile(bad), false);
  }
});

test("named material and mission snapshots recover together without duplicating a starter upgrade", async (t) => {
  globalThis.indexedDB = new IDBFactory();
  const store = new SaveStore();
  await store.open();
  t.after(() => store.db?.close());
  const collected = licensed("hugo");
  collected.onboarding = {
    version: 2,
    stage: 4,
    flags: 15,
    asteroids_destroyed: 10,
    materials_collected: 10,
    legacy_complete: false,
  };
  collected.campaign.stored_materials[0] = 10;
  collected.campaign.stored_training_materials = [0, 0, 10];
  await store.save(collected);
  const upgraded = structuredClone(collected);
  upgraded.revision++;
  upgraded.onboarding = { ...upgraded.onboarding, stage: 6, flags: 127 };
  upgraded.campaign.abilities[1].rank = 2;
  upgraded.campaign.stored_materials[0] = 0;
  upgraded.campaign.stored_training_materials = [0, 0, 0];
  await store.save(upgraded);
  const reload = new SaveStore();
  t.after(() => reload.db?.close());
  assert.deepEqual(await reload.open(), upgraded);
  const db = await request(indexedDB.open("galaxy-royale"));
  const tx = db.transaction("profile", "readwrite");
  tx.objectStore("profile").put(
    {
      ...upgraded,
      campaign: { ...upgraded.campaign, stored_training_materials: [0, 0, 10] },
    },
    "current",
  );
  await new Promise((resolve, reject) => {
    tx.oncomplete = resolve;
    tx.onabort = () => reject(tx.error);
  });
  db.close();
  const recovery = new SaveStore();
  t.after(() => recovery.db?.close());
  assert.deepEqual(await recovery.open(), collected);
  assert.match(recovery.message, /Recovered/);
  assert.equal((await recovery.open()).campaign.abilities[1].rank, 1);
});
