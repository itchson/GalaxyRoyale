import test from "node:test";
import assert from "node:assert/strict";
import { IDBFactory } from "fake-indexeddb";
import {
  SaveStore,
  SaveConflict,
  emptyCampaign,
  normalizeProfile,
  validProfile,
} from "../../apps/game/src/storage.ts";

function legacy() {
  const { abilities, loadouts, ...campaign } = emptyCampaign();
  return {
    schema_version: 3,
    revision: 10,
    seed: 0x47524f59,
    currency: [30, 20, 10],
    ranks: [2, 0, 1, 0, 0, 0, 0, 0, 0],
    unlocked: [false, false, false],
    kills: [0, 0, 0],
    boss_kills: [0, 0, 0],
    boss_generations: Array.from({ length: 3 }, () => Array(5).fill(0)),
    pending: [false, false, false],
    pity: [0, 0, 0],
    discoveries: [],
    quests: [],
    relics: [],
    campaign: { ...campaign, cargo_materials: 0, stored_materials: 0 },
  };
}
function fresh() {
  return normalizeProfile(legacy());
}
function mirror(p) {
  p.ranks = [
    ...p.campaign.loadouts[p.campaign.selected_ship].hardware,
    ...p.campaign.abilities.slice(3, 6).map((a) => a.rank),
  ];
  p.unlocked = p.campaign.abilities.slice(3, 6).map((a) => a.blueprint);
  return p;
}
function ownedChampion(p, index) {
  p.campaign.patches[index] =
    p.campaign.blueprints[index] =
    p.campaign.crafted[index] =
      true;
  p.campaign.abilities[index + 6] = { blueprint: true, rank: 1 };
  return p;
}
const request = (r) =>
  new Promise((resolve, reject) => {
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
async function snapshots() {
  const db = await request(indexedDB.open("galaxy-royale"));
  const store = db.transaction("profile", "readonly").objectStore("profile");
  const result = await Promise.all([
    request(store.get("current")),
    request(store.get("previous")),
  ]);
  db.close();
  return result;
}
async function fixture(t) {
  globalThis.indexedDB = new IDBFactory();
  const store = new SaveStore();
  await store.open();
  t.after(() => store.db?.close());
  return store;
}

test("schema-five migration waits for a real old database connection and prevents old-version writers", async (t) => {
  globalThis.indexedDB = new IDBFactory();
  const opening = indexedDB.open("galaxy-royale", 1);
  opening.onupgradeneeded = () => opening.result.createObjectStore("profile");
  const old = await request(opening);
  t.after(() => old.close());
  const baseline = legacy();
  baseline.campaign.cargo_materials = 7;
  baseline.campaign.stored_materials = 9;
  const tx = old.transaction("profile", "readwrite");
  const done = new Promise((resolve, reject) => {
    tx.oncomplete = resolve;
    tx.onabort = () => reject(tx.error);
  });
  tx.objectStore("profile").put(baseline, "current");
  await done;
  const store = new SaveStore();
  t.after(() => store.db?.close());
  let notice;
  const blocked = new Promise((resolve) => (notice = resolve));
  let settled = false;
  const loading = store
    .open((message) => notice(message))
    .then((profile) => {
      settled = true;
      return profile;
    });
  assert.match(await blocked, /Close other.*tabs/i);
  assert.equal(settled, false);
  assert.equal(old.version, 1);
  old.close();
  const loaded = await loading;
  assert.deepEqual(loaded, normalizeProfile(baseline));
  assert.equal(store.db.version, 5);
  assert.deepEqual(loaded.campaign.stored_materials, [16, 0, 0, 0, 0]);
  await assert.rejects(request(indexedDB.open("galaxy-royale", 1)), /version/i);
  assert.deepEqual(
    (await snapshots())[0],
    loaded,
    "failed old-version open cannot overwrite the new inventory",
  );
});

test("an actual version-change event closes obsolete connections and rejects queued and imported writes", async (t) => {
  const store = await fixture(t),
    p = fresh();
  await store.save(p);
  let release;
  store.queue = new Promise((resolve) => (release = resolve));
  const queued = store.save({
    ...p,
    revision: p.revision + 1,
    currency: [900, 900, 900],
  });
  const rejection = assert.rejects(
    queued,
    (error) => error instanceof SaveConflict,
  );
  const next = await request(indexedDB.open("galaxy-royale", 6));
  t.after(() => next.close());
  release();
  await rejection;
  assert.equal(store.persistent, false);
  assert.match(store.message, /format changed.*Reload/i);
  await assert.rejects(
    store.save({ ...p, revision: p.revision + 1, currency: [900, 900, 900] }),
    (error) => error instanceof SaveConflict,
  );
  await assert.rejects(
    store.replace(p),
    (error) => error instanceof SaveConflict,
  );
  assert.deepEqual((await snapshots())[0], p);
  const oldClient = new SaveStore();
  await assert.rejects(
    oldClient.open(),
    (error) =>
      error instanceof SaveConflict &&
      /newer game version/i.test(error.message),
  );
  await assert.rejects(
    oldClient.save(p),
    (error) => error instanceof SaveConflict,
  );
  await assert.rejects(
    oldClient.replace(p),
    (error) => error instanceof SaveConflict,
  );
  assert.deepEqual((await snapshots())[0], p);
});

test("legacy migration banks scalar materials once and isolates old hardware and XP per hull", () => {
  const p = legacy();
  p.campaign.cargo_materials = 7;
  p.campaign.stored_materials = 19;
  p.campaign.patches[2] =
    p.campaign.blueprints[2] =
    p.campaign.crafted[2] =
      true;
  p.campaign.selected_ship = 3;
  p.campaign.ship_xp[0] = 60;
  p.campaign.ship_xp[3] = 125;
  p.ranks[7] = 3;
  p.unlocked[1] = true;
  const before = structuredClone(p),
    migrated = normalizeProfile(p);
  assert.equal(validProfile(migrated), true);
  assert.deepEqual(migrated.campaign.stored_materials, [26, 0, 0, 0, 0]);
  assert.deepEqual(migrated.campaign.cargo_materials, [0, 0, 0, 0, 0]);
  assert.deepEqual(migrated.campaign.loadouts[3].hardware, p.ranks.slice(0, 6));
  assert.deepEqual(migrated.campaign.loadouts[0].hardware, [0, 0, 0, 0, 0, 0]);
  assert.deepEqual(migrated.campaign.ship_xp, p.campaign.ship_xp);
  assert.deepEqual(migrated.campaign.abilities[4], {
    blueprint: true,
    rank: 3,
  });
  assert.deepEqual(migrated.campaign.loadouts[3].slots, [8, 4, null]);
  assert.deepEqual(
    normalizeProfile(migrated),
    migrated,
    "normalization cannot bank materials twice",
  );
  assert.deepEqual(p, before, "caller-owned legacy input stays untouched");
});

test("legacy saturation and unknown new-looking fields cannot poison the supported migration", () => {
  const p = legacy();
  p.campaign.cargo_materials = 16;
  p.campaign.stored_materials = 0xffffffff;
  p.campaign.abilities = null;
  p.campaign.loadouts = null;
  p.campaign.injected = "ignored";
  assert.equal(validProfile(p), true);
  const normalized = normalizeProfile(p);
  assert.equal(validProfile(normalized), true);
  assert.deepEqual(
    normalized.campaign.stored_materials,
    [0xffffffff, 0, 0, 0, 0],
  );
  assert.equal(normalized.campaign.abilities.length, 17);
  assert.equal(normalized.campaign.loadouts.length, 10);
  assert.equal(Object.hasOwn(normalized.campaign, "injected"), false);
});

test("schema four accepts learned uncrafted blueprints and rejects impossible equipment and mirrors", () => {
  const p = ownedChampion(fresh(), 1);
  p.campaign.abilities[3] = { blueprint: true, rank: 0 };
  mirror(p);
  assert.equal(
    validProfile(p),
    true,
    "blueprint availability does not fabricate a crafted ability",
  );
  for (const change of [
    (p) => p.campaign.abilities.pop(),
    (p) => (p.campaign.abilities[0].blueprint = false),
    (p) => (p.campaign.abilities[3] = { blueprint: false, rank: 1 }),
    (p) => (p.campaign.abilities[0].rank = 6),
    (p) => (p.campaign.loadouts[0].slots = [0, null]),
    (p) => p.campaign.loadouts[0].hardware.push(0),
    (p) => (p.campaign.loadouts[0].slots[0] = 3),
    (p) => (p.campaign.abilities[0].rank = 1),
    (p) => (p.unlocked[0] = false),
    (p) => (p.campaign.loadouts[0].slots[0] = 7),
    (p) => (p.campaign.loadouts[1].hardware[0] = 1),
    (p) => (p.campaign.loadouts[1].slots[0] = 7),
    (p) => (p.campaign.loadouts[2].slots[1] = 7),
    (p) => (p.campaign.abilities[6] = { blueprint: true, rank: 1 }),
    (p) => (p.campaign.abilities[14] = { blueprint: true, rank: 5 }),
    (p) => (p.ranks[0] = 0),
  ]) {
    const candidate = structuredClone(p);
    change(candidate);
    // The owned starter rank by itself is valid; exercise its wrong-category slot.
    if (candidate.campaign.abilities[0].rank === 1)
      candidate.campaign.loadouts[0].slots[1] = 0;
    assert.equal(validProfile(candidate), false, JSON.stringify(candidate));
    assert.equal(normalizeProfile(candidate), null);
  }
});

test("save normalization preserves live tiered cargo and clones all mutable inventory state", () => {
  const p = ownedChampion(fresh(), 1);
  p.campaign.abilities[0].rank = 1;
  p.campaign.loadouts[0].slots = [0, null, null];
  p.campaign.loadouts[2].slots = [7, null, null];
  p.campaign.cargo_materials = [2, 3, 1, 0, 0];
  p.campaign.stored_materials = [4, 5, 6, 7, 8];
  const normalized = normalizeProfile(p);
  assert.equal(validProfile(normalized), true);
  assert.deepEqual(
    normalized.campaign.cargo_materials,
    p.campaign.cargo_materials,
    "only Game constructor abandons a live run",
  );
  normalized.campaign.cargo_materials[0] = 9;
  normalized.campaign.abilities[0].rank = 5;
  normalized.campaign.loadouts[0].slots[0] = null;
  normalized.campaign.loadouts[2].hardware[0] = 4;
  assert.deepEqual(p.campaign.cargo_materials, [2, 3, 1, 0, 0]);
  assert.equal(p.campaign.abilities[0].rank, 1);
  assert.equal(p.campaign.loadouts[0].slots[0], 0);
  assert.equal(p.campaign.loadouts[2].hardware[0], 0);
  for (const cargo of [
    [17, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1],
    [1, -1, 0, 0, 0],
    Array(5),
  ]) {
    const invalid = structuredClone(p);
    invalid.campaign.cargo_materials = cargo;
    assert.equal(validProfile(invalid), false);
  }
});

test("crafting and slot snapshots retain previous inventory and block stale-session material duplication", async (t) => {
  const first = await fixture(t),
    baseline = fresh();
  baseline.campaign.stored_materials = [6, 2, 0, 0, 0];
  await first.save(baseline);
  const second = new SaveStore();
  t.after(() => second.db?.close());
  await second.open();
  const crafted = structuredClone(baseline);
  crafted.revision++;
  crafted.currency[0] -= 3;
  crafted.campaign.stored_materials[0] -= 2;
  crafted.campaign.abilities[0].rank = 1;
  await first.save(crafted);
  const equipped = structuredClone(crafted);
  equipped.revision++;
  equipped.campaign.loadouts[0].slots[0] = 0;
  await first.save(equipped);
  const stale = structuredClone(baseline);
  stale.revision = equipped.revision + 1;
  await assert.rejects(
    second.save(stale),
    (error) => error instanceof SaveConflict,
  );
  assert.deepEqual(await snapshots(), [equipped, crafted]);
  const reopened = new SaveStore();
  t.after(() => reopened.db?.close());
  assert.deepEqual(await reopened.open(), equipped);
  await reopened.save({ ...equipped, currency: [3000, 20, 10] });
  assert.deepEqual((await snapshots())[0], equipped);
});

test("corrupt slot recovery retains banked assets, per-ship XP, and recorded death material loss", async (t) => {
  const store = await fixture(t),
    p = fresh();
  p.campaign.ship_xp[0] = 125;
  p.campaign.stored_materials = [4, 3, 2, 1, 0];
  p.campaign.last_run = {
    time: 60,
    distance_peak: 2400,
    enemies: 8,
    asteroids: 3,
    champions: 0,
    points_gained: [1, 2, 3],
    xp_gained: 40,
    materials_gained: 7,
    materials_lost: [2, 3, 1, 1, 0],
  };
  await store.save(p);
  const newer = structuredClone(p);
  newer.revision++;
  await store.save(newer);
  const corrupt = structuredClone(newer);
  corrupt.campaign.loadouts[0].slots[0] = 16;
  const db = await request(indexedDB.open("galaxy-royale"));
  const tx = db.transaction("profile", "readwrite");
  const done = new Promise((resolve, reject) => {
    tx.oncomplete = resolve;
    tx.onabort = () => reject(tx.error);
  });
  tx.objectStore("profile").put(corrupt, "current");
  await done;
  db.close();
  const recovered = new SaveStore();
  t.after(() => recovered.db?.close());
  assert.deepEqual(await recovered.open(), p);
  assert.match(recovered.message, /Recovered/);
  const before = await snapshots();
  await assert.rejects(recovered.replace(corrupt), /supported/);
  assert.deepEqual(await snapshots(), before);
});
