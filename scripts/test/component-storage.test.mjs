import test from "node:test";
import assert from "node:assert/strict";
import { IDBFactory } from "fake-indexeddb";
import {
  SaveStore,
  validProfile,
  normalizeProfile,
  emptyCampaign,
} from "../../apps/game/src/storage.ts";

const legacy = () => ({
  schema_version: 6,
  revision: 34,
  seed: 12345,
  currency: [11, 22, 33],
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
  identity: null,
  onboarding: null,
  campaign: emptyCampaign(),
});
const request = (r) =>
  new Promise((resolve, reject) => {
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
const done = (tx) =>
  new Promise((resolve, reject) => {
    tx.oncomplete = resolve;
    tx.onabort = () => reject(tx.error);
  });

test("schema 6 preserves progression and strips injected components without inventing ownership", () => {
  for (const injected of [
    undefined,
    null,
    "broken",
    [99, 99, 99],
    [-1],
    { offensive: 9 },
  ]) {
    const p = legacy();
    p.campaign.components = injected;
    p.campaign.stored_materials = [20, 8, 4, 2, 1];
    p.campaign.stored_training_materials = [10, 3, 2];
    p.campaign.ship_xp[0] = 130;
    const before = structuredClone(p);
    assert.ok(validProfile(p));
    const migrated = normalizeProfile(p);
    assert.equal(migrated.schema_version, 7);
    assert.deepEqual(migrated.campaign.components, [0, 0, 0]);
    assert.deepEqual(migrated, {
      ...p,
      schema_version: 7,
      campaign: { ...p.campaign, components: [0, 0, 0] },
    });
    assert.deepEqual(p, before);
  }
});

test("schema 7 requires exactly three bounded integer component counts", () => {
  const p = normalizeProfile(legacy());
  for (const invalid of [
    undefined,
    null,
    "0,0,0",
    [],
    [0, 0],
    [0, 0, 0, 0],
    Array(3),
    [0, -1, 0],
    [0, 100, 0],
    [0, 1.5, 0],
    [0, NaN, 0],
    [0, "1", 0],
  ]) {
    const malformed = {
      ...p,
      campaign: { ...p.campaign, components: invalid },
    };
    assert.equal(validProfile(malformed), false);
    assert.equal(normalizeProfile(malformed), null);
  }
  p.campaign.components = [1, 99, 23];
  assert.deepEqual(normalizeProfile(p), p);
});

test("schema 7 retains component snapshots, rejects stale replay and recovers previous valid counts", async (t) => {
  globalThis.indexedDB = new IDBFactory();
  const store = new SaveStore();
  await store.open();
  t.after(() => store.db?.close());
  const p = normalizeProfile(legacy());
  p.campaign.components = [3, 4, 5];
  await store.save(p);
  const next = structuredClone(p);
  next.revision++;
  next.campaign.components[0]--;
  await store.save(next);
  const stale = structuredClone(next);
  stale.campaign.components.fill(99);
  await store.save(stale);
  const loaded = new SaveStore();
  t.after(() => loaded.db?.close());
  assert.deepEqual(await loaded.open(), next);
  const db = await request(indexedDB.open("galaxy-royale"));
  const tx = db.transaction("profile", "readwrite");
  const commit = done(tx);
  tx.objectStore("profile").put(
    { ...next, campaign: { ...next.campaign, components: [100, 0, 0] } },
    "current",
  );
  await commit;
  db.close();
  assert.deepEqual(await loaded.open(), p);
  assert.match(loaded.message, /Recovered/);
  await assert.rejects(
    loaded.replace({ ...p, campaign: { ...p.campaign, components: null } }),
    /supported/,
  );
});

test("database 5 migrates both schema 6 snapshots and prevents a cached database 4 writer", async (t) => {
  globalThis.indexedDB = new IDBFactory();
  const opening = indexedDB.open("galaxy-royale", 4);
  opening.onupgradeneeded = () => opening.result.createObjectStore("profile");
  const old = await request(opening);
  t.after(() => old.close());
  const tx = old.transaction("profile", "readwrite");
  const commit = done(tx);
  const current = legacy(),
    previous = legacy();
  previous.revision--;
  previous.currency[0]--;
  delete current.campaign.components;
  delete previous.campaign.components;
  tx.objectStore("profile").put(current, "current");
  tx.objectStore("profile").put(previous, "previous");
  await commit;
  let blocked;
  const waiting = new Promise((resolve) => {
    blocked = resolve;
  });
  const store = new SaveStore();
  t.after(() => store.db?.close());
  const migrating = store.open(() => blocked());
  await waiting;
  old.close();
  assert.deepEqual(await migrating, normalizeProfile(current));
  assert.equal(store.db.version, 5);
  const read = store.db
    .transaction("profile", "readonly")
    .objectStore("profile");
  assert.deepEqual(
    await request(read.get("previous")),
    normalizeProfile(previous),
  );
  await assert.rejects(request(indexedDB.open("galaxy-royale", 4)), /version/i);
});
