import test from "node:test";
import assert from "node:assert/strict";
import { IDBFactory } from "fake-indexeddb";
import {
  SaveStore,
  SaveConflict,
  validProfile,
  normalizeProfile,
  emptyCampaign as currentEmptyCampaign,
  defaultSettings,
  normalizeSettings,
  loadSettings,
  saveSettings,
} from "../../apps/game/src/storage.ts";

// Keep real schema-three fixtures to exercise backups produced by the prior release.
const emptyCampaign = () => {
  const { abilities, loadouts, ...campaign } = currentEmptyCampaign();
  return { ...campaign, cargo_materials: 0, stored_materials: 0 };
};

const emptyGenerations = () =>
  Array.from({ length: 3 }, () => Array(5).fill(0));
const profile = (revision = 0, currency = [0, 0, 0]) => ({
  schema_version: 3,
  revision,
  seed: 0x47524f59,
  currency,
  ranks: Array(9).fill(0),
  unlocked: [false, false, false],
  kills: [0, 0, 0],
  boss_kills: [0, 0, 0],
  boss_generations: emptyGenerations(),
  pity: [0, 0, 0],
  pending: [false, false, false],
  discoveries: [],
  quests: [],
  relics: [],
  campaign: emptyCampaign(),
});
const previousProfile = (revision = 0, currency = [0, 0, 0]) => {
  const { campaign, boss_generations, ...older } = profile(revision, currency);
  return { ...older, schema_version: 2 };
};
const legacyProfile = (revision = 0, currency = [0, 0, 0]) => {
  const { discoveries, quests, relics, campaign, boss_generations, ...legacy } =
    profile(revision, currency);
  return { ...legacy, schema_version: 1 };
};
const quest = (site_id = "-2:3", overrides = {}) => ({
  site_id,
  event: "bounty",
  family: 1,
  target: 3,
  progress: 1,
  completed: false,
  ...overrides,
});
const request = (r) =>
  new Promise((resolve, reject) => {
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
const committed = (tx) =>
  new Promise((resolve, reject) => {
    tx.oncomplete = resolve;
    tx.onabort = () => reject(tx.error);
  });
async function connection() {
  return request(indexedDB.open("galaxy-royale"));
}
async function snapshots() {
  const db = await connection();
  const tx = db.transaction("profile", "readonly");
  const store = tx.objectStore("profile");
  const result = await Promise.all([
    request(store.get("current")),
    request(store.get("previous")),
  ]);
  db.close();
  return result;
}
async function corrupt(current, previous) {
  const db = await connection();
  const tx = db.transaction("profile", "readwrite");
  const done = committed(tx);
  const store = tx.objectStore("profile");
  store.put(current, "current");
  if (previous !== undefined) store.put(previous, "previous");
  await done;
  db.close();
}
async function fixture(t) {
  globalThis.indexedDB = new IDBFactory();
  const store = new SaveStore();
  await store.open();
  t.after(() => store.db?.close());
  return store;
}

test("profile boundary rejects malformed arrays, impossible signature ranks and unsupported versions", () => {
  assert.equal(validProfile(profile()), true);
  const unlocked = profile(10, [10, 20, 30]);
  unlocked.unlocked = [true, true, true];
  unlocked.ranks = [1, 2, 3, 4, 5, 1, 1, 2, 3];
  assert.equal(validProfile(unlocked), true);
  for (const mutation of [
    (p) => (p.schema_version = 99),
    (p) => (p.revision = NaN),
    (p) => (p.revision = Number.MAX_SAFE_INTEGER + 1),
    (p) => (p.seed = 0),
    (p) => (p.currency = [-1, 0, 0]),
    (p) => (p.currency = [0, 0]),
    (p) => (p.currency = [0, 0, 2 ** 32]),
    (p) => (p.currency = Array(3)),
    (p) => (p.ranks[0] = 6),
    (p) => (p.ranks[6] = 1),
    (p) => (p.unlocked[0] = true),
    (p) => (p.unlocked = Array(3)),
    (p) => (p.pending = [false, false, 1]),
  ]) {
    const p = profile();
    mutation(p);
    assert.equal(validProfile(p), false, JSON.stringify(p));
  }
  assert.equal(validProfile(null), false);
  assert.equal(validProfile("profile"), false);
});

test("saving full snapshots never adds currency twice and ignores equal or older revisions", async (t) => {
  const store = await fixture(t);
  const first = profile(1, [10, 20, 30]);
  const saving = store.save(first);
  first.currency[0] = 999;
  await saving;
  await store.save(profile(1, [10000, 10000, 10000]));
  await store.save(profile(0, [10000, 10000, 10000]));
  const spent = profile(2, [0, 20, 30]);
  spent.ranks[0] = 1;
  spent.ranks[6] = 2;
  spent.unlocked[0] = true;
  await store.save(spent);
  await store.flush();
  const [current, previous] = await snapshots();
  assert.deepEqual(current.currency, [0, 20, 30]);
  assert.equal(current.ranks[0], 1);
  assert.equal(current.revision, 2);
  assert.deepEqual(previous.currency, [10, 20, 30]);
  assert.equal(current.unlocked[0], true);
  assert.equal(current.ranks[6], 2);
  const reopened = new SaveStore();
  t.after(() => reopened.db?.close());
  const loaded = await reopened.open();
  assert.deepEqual(loaded, current);
  loaded.currency[0] = 555;
  await reopened.save({ ...loaded, revision: 3, currency: [0, 20, 30] });
  const [, backup] = await snapshots();
  assert.deepEqual(
    backup.currency,
    [0, 20, 30],
    "returned profile must not mutate the internal previous snapshot",
  );
});

test("reload recovers a valid previous snapshot when current is corrupted", async (t) => {
  const store = await fixture(t);
  await store.save(profile(1, [10, 0, 0]));
  await store.save(profile(2, [20, 0, 0]));
  await corrupt({ schema_version: 1, currency: ["broken"] });
  const recovered = new SaveStore();
  t.after(() => recovered.db?.close());
  assert.deepEqual(
    await recovered.open(),
    normalizeProfile(profile(1, [10, 0, 0])),
  );
  assert.match(recovered.message, /Recovered/);
  await recovered.save(profile(2, [11, 0, 0]));
  const [current, previous] = await snapshots();
  assert.deepEqual(current.currency, [11, 0, 0]);
  assert.deepEqual(previous.currency, [10, 0, 0]);
});

test("two corrupted snapshots produce a visible new-profile recovery state", async (t) => {
  await fixture(t);
  await corrupt(
    { schema_version: 2 },
    { schema_version: 1, currency: [null, null, null] },
  );
  const recovered = new SaveStore();
  t.after(() => recovered.db?.close());
  assert.equal(await recovered.open(), null);
  assert.match(recovered.message, /invalid/);
});

test("import is ordered after pending saves, captures its input and prevents old-session saves overwriting it", async (t) => {
  const store = await fixture(t);
  await store.save(profile(7, [70, 0, 0]));
  const pending = store.save(profile(8, [80, 0, 0]));
  const imported = profile(1, [3, 4, 5]);
  imported.seed = 42;
  const importing = store.replace(imported);
  imported.currency[0] = 999;
  const oldSessionSave = store.save(profile(9, [900, 0, 0]));
  await Promise.all([pending, importing, oldSessionSave]);
  await store.flush();
  const [current, previous] = await snapshots();
  assert.equal(current.seed, 42);
  assert.equal(current.revision, 1);
  assert.deepEqual(current.currency, [3, 4, 5]);
  assert.equal(previous.revision, 8);
  const reopened = new SaveStore();
  t.after(() => reopened.db?.close());
  const loaded = await reopened.open();
  assert.equal(loaded.seed, 42);
  await reopened.save({ ...loaded, revision: 2, currency: [2, 4, 5] });
  assert.deepEqual((await snapshots())[0].currency, [2, 4, 5]);
});

test("invalid imports preserve current and previous; unavailable storage cannot report a persisted import", async (t) => {
  const store = await fixture(t);
  await store.save(profile(1, [10, 0, 0]));
  await store.save(profile(2, [20, 0, 0]));
  const before = await snapshots();
  await assert.rejects(store.replace({ schema_version: 2 }), /supported/);
  assert.deepEqual(await snapshots(), before);
  const original = globalThis.indexedDB;
  globalThis.indexedDB = undefined;
  const memory = new SaveStore();
  assert.equal(await memory.open(), null);
  assert.match(memory.message, /unavailable/);
  await memory.save(profile(1, [5, 0, 0]));
  await assert.rejects(memory.replace(profile(2, [99, 0, 0])), /unavailable/i);
  globalThis.indexedDB = original;
});

test("another open session cannot overwrite newer progression or duplicate currency", async (t) => {
  const first = await fixture(t);
  await first.save(profile(1, [10, 0, 0]));
  const second = new SaveStore();
  t.after(() => second.db?.close());
  await second.open();
  await first.save(profile(2, [20, 0, 0]));
  await assert.rejects(
    second.save(profile(3, [100000, 0, 0])),
    (error) =>
      error instanceof SaveConflict &&
      /another session|changed|reload/i.test(error.message),
  );
  assert.equal((await snapshots())[0].revision, 2);
  assert.deepEqual((await snapshots())[0].currency, [20, 0, 0]);
});

test("failed database writes leave the accepted snapshot intact and report the failure", async (t) => {
  const store = await fixture(t);
  await store.save(profile(1, [10, 0, 0]));
  store.db.close();
  await assert.rejects(store.save(profile(2, [20, 0, 0])));
  assert.match(store.message, /failed/);
  assert.equal((await snapshots())[0].revision, 1);
  assert.deepEqual((await snapshots())[0].currency, [10, 0, 0]);
});

test("v1 current and previous snapshots migrate to v4 without changing progression or awarding currency", async (t) => {
  await fixture(t);
  const current = legacyProfile(11, [99, 17, 4]);
  current.ranks[0] = 3;
  current.ranks[6] = 2;
  current.unlocked[0] = true;
  current.kills = [100, 2, 3];
  current.pending[2] = true;
  const previous = legacyProfile(10, [89, 17, 4]);
  const before = structuredClone(current);
  await corrupt(current, previous);
  const migrated = new SaveStore();
  t.after(() => migrated.db?.close());
  const loaded = await migrated.open();
  assert.equal(loaded.schema_version, 4);
  assert.equal(loaded.revision, 11);
  assert.deepEqual(loaded.currency, [99, 17, 4]);
  assert.deepEqual(loaded.ranks, current.ranks);
  assert.deepEqual(loaded.unlocked, current.unlocked);
  assert.deepEqual(loaded.pending, current.pending);
  assert.deepEqual(current, before, "migration does not mutate caller input");
  assert.deepEqual(loaded.discoveries, []);
  assert.deepEqual(loaded.quests, []);
  assert.deepEqual(loaded.relics, []);
  assert.match(migrated.message, /upgraded/);
  const [persisted, backup] = await snapshots();
  assert.deepEqual(persisted, normalizeProfile(current));
  assert.deepEqual(backup, normalizeProfile(previous));
  await migrated.save({ ...loaded, currency: [9999, 17, 4] });
  assert.deepEqual(
    (await snapshots())[0].currency,
    [99, 17, 4],
    "same revision never adds a migration reward",
  );
  await migrated.save({
    ...loaded,
    revision: 12,
    discoveries: ["-2:3"],
    quests: [quest("-2:3", { progress: 3, completed: true })],
    relics: ["-2:3"],
    currency: [109, 17, 4],
  });
  const reopened = new SaveStore();
  t.after(() => reopened.db?.close());
  const progress = await reopened.open();
  await reopened.save({ ...progress, currency: [119, 17, 4] });
  assert.deepEqual(
    (await snapshots())[0].currency,
    [109, 17, 4],
    "completed quest snapshot cannot replay currency at equal revision",
  );
});

test("legacy recovery and legacy import both retain revision, ranks and currency in v4 snapshots", async (t) => {
  const store = await fixture(t);
  await store.save(profile(5, [50, 0, 0]));
  const legacy = legacyProfile(4, [40, 1, 2]);
  legacy.ranks[4] = 2;
  await corrupt({ schema_version: 99 }, legacy);
  const recovery = new SaveStore();
  t.after(() => recovery.db?.close());
  const recovered = await recovery.open();
  assert.deepEqual(recovered, normalizeProfile(legacy));
  assert.match(recovery.message, /Recovered/);
  await recovery.save({ ...recovered, revision: 5, currency: [41, 1, 2] });
  const imported = legacyProfile(2, [12, 13, 14]);
  imported.ranks[1] = 4;
  await recovery.replace(imported);
  imported.currency[0] = 900;
  const [current, previous] = await snapshots();
  assert.equal(current.schema_version, 4);
  assert.equal(current.revision, 2);
  assert.equal(current.ranks[1], 4);
  assert.deepEqual(current.currency, [12, 13, 14]);
  assert.deepEqual(current.quests, []);
  assert.equal(previous.revision, 5);
});

test("discoveries, quests and relics persist exactly across reload without replaying completion rewards", async (t) => {
  const store = await fixture(t);
  const explored = profile(1, [5, 6, 7]);
  explored.discoveries = ["-2:3", "0:-1"];
  explored.quests = [
    quest(),
    quest("0:-1", {
      event: "courier",
      family: 0,
      target: 1,
      progress: 1,
      completed: true,
    }),
  ];
  explored.relics = ["0:-1"];
  const saving = store.save(explored);
  explored.quests[0].progress = 3;
  explored.discoveries.push("8:8");
  await saving;
  const reopened = new SaveStore();
  t.after(() => reopened.db?.close());
  const loaded = await reopened.open();
  assert.deepEqual(loaded.discoveries, ["-2:3", "0:-1"]);
  assert.deepEqual(loaded.quests, [
    quest(),
    quest("0:-1", {
      event: "courier",
      family: 0,
      target: 1,
      progress: 1,
      completed: true,
    }),
  ]);
  assert.deepEqual(loaded.relics, ["0:-1"]);
  await reopened.save({ ...loaded, revision: 1, currency: [5000, 6000, 7000] });
  assert.deepEqual((await snapshots())[0].currency, [5, 6, 7]);
});

test("v2 rejects malformed, oversized and duplicate site collections and quest bounds", () => {
  assert.equal(validProfile(legacyProfile()), true);
  assert.equal(
    validProfile({
      ...profile(),
      discoveries: ["-2:3"],
      quests: [quest()],
      relics: ["-2:3"],
    }),
    true,
  );
  for (const mutation of [
    (p) => delete p.discoveries,
    (p) => (p.discoveries = ["not-a-grid"]),
    (p) => (p.discoveries = ["1:2", "1:2"]),
    (p) => (p.discoveries = Array(1)),
    (p) => (p.discoveries = Array.from({ length: 257 }, (_, i) => `${i}:0`)),
    (p) => (p.relics = ["1:2", "1:2"]),
    (p) => (p.relics = ["1.5:2"]),
    (p) => (p.relics = Array.from({ length: 257 }, (_, i) => `${i}:0`)),
    (p) => (p.quests = Array(1)),
    (p) => (p.quests = [quest(), quest()]),
    (p) => (p.quests = [quest(), quest("-2:3", { event: "courier" })]),
    (p) => (p.quests = Array.from({ length: 257 }, (_, i) => quest(`${i}:0`))),
    ...[
      { site_id: "x:1" },
      { event: "escort" },
      { family: 3 },
      { family: -1 },
      { family: 0.5 },
      { target: 0 },
      { target: 13 },
      { progress: -1 },
      { progress: 4 },
      { completed: 1 },
    ].map((fields) => (p) => (p.quests = [quest("-2:3", fields)])),
  ]) {
    const p = profile();
    mutation(p);
    assert.equal(validProfile(p), false, JSON.stringify(p));
    assert.equal(normalizeProfile(p), null);
  }
});

test("settings migrate old values and round-trip motion and cosmetic density with safe validation", (t) => {
  const original = globalThis.localStorage;
  let stored = JSON.stringify({
    master: 0.2,
    music: 0.3,
    sfx: 0.4,
    shake: false,
  });
  globalThis.localStorage = {
    getItem: () => stored,
    setItem: (_key, value) => (stored = value),
  };
  t.after(() => (globalThis.localStorage = original));
  assert.deepEqual(loadSettings(), {
    master: 0.2,
    music: 0.3,
    sfx: 0.4,
    shake: false,
    reduced_motion: false,
    cosmetic_density: "full",
    dialogue_voice: false,
    voice_volume: 0.7,
    zoom: 1,
  });
  const reduced = {
    ...defaultSettings,
    reduced_motion: true,
    cosmetic_density: "reduced",
  };
  saveSettings(reduced);
  assert.deepEqual(loadSettings(), reduced);
  assert.deepEqual(
    normalizeSettings({
      master: 10,
      music: -1,
      sfx: "loud",
      shake: "no",
      reduced_motion: 1,
      cosmetic_density: "ultra",
    }),
    { ...defaultSettings, master: 1, music: 0 },
  );
  stored = "{malformed";
  assert.deepEqual(loadSettings(), defaultSettings);
  stored = "null";
  assert.deepEqual(loadSettings(), defaultSettings);
  assert.equal(normalizeSettings({ zoom: 100 }).zoom, 1.6);
  assert.equal(normalizeSettings({ zoom: -1 }).zoom, 0.6);
  assert.equal(normalizeSettings({ zoom: NaN }).zoom, 1);
  assert.equal(normalizeSettings({ voice_volume: 4 }).voice_volume, 1);
  assert.equal(normalizeSettings({ voice_volume: -3 }).voice_volume, 0);
});

test("v2 expedition progression migrates to a blank campaign with no new reward or XP", async (t) => {
  await fixture(t);
  const older = previousProfile(19, [40, 12, 7]);
  older.discoveries = ["-2:3"];
  older.quests = [quest("-2:3", { progress: 3, completed: true })];
  older.relics = ["-2:3"];
  await corrupt(older, previousProfile(18, [37, 12, 7]));
  const migrated = new SaveStore();
  t.after(() => migrated.db?.close());
  const loaded = await migrated.open();
  assert.equal(loaded.schema_version, 4);
  assert.equal(loaded.revision, 19);
  assert.deepEqual(loaded.currency, older.currency);
  assert.deepEqual(loaded.quests, older.quests);
  assert.deepEqual(loaded.campaign, currentEmptyCampaign());
  assert.deepEqual((await snapshots())[0], loaded);
  assert.equal((await snapshots())[1].schema_version, 4);
});

test("campaign boundary rejects fabricated ownership, malformed league, cargo and run summaries", () => {
  for (const mutation of [
    (c) => delete c.ship_xp,
    (c) => (c.ship_xp = Array(10)),
    (c) => (c.ship_xp[1] = -1),
    (c) => (c.patches = Array(8)),
    (c) => (c.patches[0] = true),
    (c) => (c.blueprints[0] = true),
    (c) => (c.crafted[0] = true),
    (c) => (c.selected_ship = 1),
    (c) => (c.selected_ship = 9),
    (c) => (c.royale_results = [1, 0, 0, 0, 0, 0, 0, 0]),
    (c) => (c.royale_results = Array(8)),
    (c) => (c.tournament_attempt = -1),
    (c) => (c.victory_claimed = true),
    (c) => (c.credits_seen = true),
    (c) => (c.cargo_materials = 17),
    (c) => (c.cargo_materials = -1),
    (c) => (c.stored_materials = 2 ** 32),
    (c) => (c.last_run = {}),
    (c) =>
      (c.last_run = {
        time: NaN,
        distance_peak: 1,
        enemies: 0,
        asteroids: 0,
        champions: 0,
        points_gained: [0, 0, 0],
        xp_gained: 0,
        materials_gained: 0,
      }),
  ]) {
    const candidate = profile();
    mutation(candidate.campaign);
    assert.equal(validProfile(candidate), false);
  }
  const candidate = profile();
  candidate.campaign.patches[0] = true;
  candidate.campaign.blueprints[0] = true;
  candidate.campaign.crafted[0] = true;
  candidate.campaign.selected_ship = 1;
  assert.equal(validProfile(candidate), true);
});

test("ship, campaign and run snapshots survive reload without replaying rewards or material costs", async (t) => {
  const store = await fixture(t);
  const candidate = profile(20, [30, 40, 50]);
  candidate.campaign.patches.fill(true);
  candidate.campaign.blueprints.fill(true);
  candidate.campaign.crafted[2] = true;
  candidate.campaign.selected_ship = 3;
  candidate.campaign.royale_results.fill(1);
  candidate.campaign.victory_claimed = true;
  candidate.campaign.ship_xp[3] = 125;
  candidate.campaign.ship_xp[9] = 2000;
  candidate.campaign.stored_materials = 12;
  candidate.campaign.last_run = {
    time: 120,
    distance_peak: 6500,
    enemies: 8,
    asteroids: 4,
    champions: 1,
    points_gained: [2, 3, 4],
    xp_gained: 40,
    materials_gained: 6,
  };
  const saving = store.save(candidate);
  candidate.campaign.stored_materials = 999;
  await saving;
  const reopened = new SaveStore();
  t.after(() => reopened.db?.close());
  const loaded = await reopened.open();
  assert.deepEqual(loaded.campaign.stored_materials, [12, 0, 0, 0, 0]);
  assert.equal(loaded.campaign.selected_ship, 3);
  assert.equal(loaded.campaign.victory_claimed, true);
  await reopened.save({
    ...loaded,
    currency: [3000, 4000, 5000],
    campaign: { ...loaded.campaign, stored_materials: [12000, 0, 0, 0, 0] },
  });
  assert.deepEqual((await snapshots())[0].currency, [30, 40, 50]);
  assert.deepEqual(
    (await snapshots())[0].campaign.stored_materials,
    [12, 0, 0, 0, 0],
  );
  await reopened.save({
    ...loaded,
    revision: 21,
    ranks: [...loaded.campaign.loadouts[9].hardware, ...loaded.ranks.slice(6)],
    campaign: { ...loaded.campaign, credits_seen: true, selected_ship: 9 },
  });
  assert.equal((await snapshots())[0].campaign.selected_ship, 9);
  assert.equal((await snapshots())[1].campaign.selected_ship, 3);
});

test("old boss aggregates migrate into generation one for every supported save format without rewards", () => {
  for (const make of [legacyProfile, previousProfile, profile]) {
    for (const supplied of ["missing", "zero"]) {
      const old = make(73, [11, 22, 33]);
      old.boss_kills = [7, 4, 2];
      old.kills = [240, 96, 48];
      old.pending = [true, false, true];
      if (supplied === "missing") delete old.boss_generations;
      else old.boss_generations = emptyGenerations();
      const before = structuredClone(old);
      assert.equal(validProfile(old), true);
      const migrated = normalizeProfile(old);
      assert.deepEqual(migrated.boss_generations, [
        [7, 0, 0, 0, 0],
        [4, 0, 0, 0, 0],
        [2, 0, 0, 0, 0],
      ]);
      for (const key of [
        "revision",
        "currency",
        "ranks",
        "unlocked",
        "kills",
        "boss_kills",
        "pending",
        "pity",
      ])
        assert.deepEqual(
          migrated[key],
          before[key],
          `${make.name}/${supplied}/${key} must retain progression`,
        );
      assert.deepEqual(
        old,
        before,
        "migration must not rewrite its caller-owned input",
      );
      assert.deepEqual(
        normalizeProfile(migrated),
        migrated,
        "normalization must be idempotent",
      );
    }
  }
  const mixed = profile();
  mixed.boss_kills = [15, 3, 0];
  mixed.boss_generations = [
    [8, 4, 2, 1, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
  ];
  assert.deepEqual(
    normalizeProfile(mixed).boss_generations,
    [
      [8, 4, 2, 1, 0],
      [3, 0, 0, 0, 0],
      [0, 0, 0, 0, 0],
    ],
    "each legacy family migrates independently",
  );
});

test("generation boundary rejects malformed or impossible promoted counters and accepts saturated aggregates", () => {
  const valid = profile();
  valid.boss_generations = [
    [16, 8, 4, 2, 1],
    [6, 3, 1, 0, 0],
    [0, 0, 0, 0, 0],
  ];
  valid.boss_kills = [31, 10, 0];
  assert.equal(validProfile(valid), true);
  for (const mutation of [
    (p) => (p.boss_generations = null),
    (p) => (p.boss_generations = undefined),
    (p) => (p.boss_generations = []),
    (p) => (p.boss_generations = Array(3)),
    (p) => (p.boss_generations[0] = Array(5)),
    (p) => (p.boss_generations[0] = [16, 8, 4, 2]),
    (p) => p.boss_generations[0].push(0),
    (p) => (p.boss_generations[0][0] = -1),
    (p) => (p.boss_generations[0][0] = 0.5),
    (p) => (p.boss_generations[0][0] = NaN),
    (p) => (p.boss_generations[0][0] = Infinity),
    (p) => (p.boss_generations[0][0] = 2 ** 32),
    (p) => (p.boss_generations[0][0] = "16"),
    (p) => (p.boss_kills[0] = 30),
    (p) => {
      p.boss_generations[0] = [1, 1, 0, 0, 0];
      p.boss_kills[0] = 2;
    },
    (p) => {
      p.boss_generations[0] = [8, 4, 3, 0, 0];
      p.boss_kills[0] = 15;
    },
    (p) => {
      p.boss_generations[0] = [8, 4, 2, 2, 0];
      p.boss_kills[0] = 16;
    },
    (p) => {
      p.boss_generations[0] = [8, 4, 2, 1, 1];
      p.boss_kills[0] = 16;
    },
    (p) => {
      p.boss_generations[2] = [1, 0, 0, 0, 0];
    },
  ]) {
    const candidate = structuredClone(valid);
    mutation(candidate);
    assert.equal(validProfile(candidate), false);
    assert.equal(normalizeProfile(candidate), null);
  }
  const saturated = profile();
  saturated.boss_generations[0] = [
    0xffffffff, 0x7fffffff, 0x3fffffff, 0x1fffffff, 0x0fffffff,
  ];
  saturated.boss_kills[0] = 0xffffffff;
  assert.equal(
    validProfile(saturated),
    true,
    "aggregate uses the same u32 saturation as Rust counters",
  );
});

test("same-schema legacy current and backup persist canonical generation matrices atomically with unchanged revisions", async (t) => {
  await fixture(t);
  const old = profile(19, [90, 12, 7]);
  delete old.boss_generations;
  old.boss_kills = [7, 4, 2];
  old.kills = [240, 96, 48];
  old.pending = [true, false, true];
  const backup = profile(18, [87, 12, 7]);
  backup.boss_kills = [6, 4, 2];
  await corrupt(old, backup);
  const store = new SaveStore();
  t.after(() => store.db?.close());
  const loaded = await store.open();
  assert.deepEqual(loaded, normalizeProfile(old));
  const [current, previous] = await snapshots();
  assert.deepEqual(current, normalizeProfile(old));
  assert.deepEqual(previous, normalizeProfile(backup));
  assert.equal(current.revision, 19);
  assert.equal(previous.revision, 18);
  await store.save({ ...loaded, currency: [9000, 12, 7] });
  assert.deepEqual(
    (await snapshots())[0],
    current,
    "same revision cannot turn migration into a reward",
  );
});

test("generation counters, surplus base kills and pending reservations survive reload without duplicate advancement", async (t) => {
  const store = await fixture(t);
  const fought = profile(100, [30, 40, 50]);
  fought.boss_generations = [
    [16, 8, 4, 2, 0],
    [4, 1, 0, 0, 0],
    [0, 0, 0, 0, 0],
  ];
  fought.boss_kills = [30, 5, 0];
  fought.kills = [453, 120, 23];
  fought.pending = [true, true, false];
  const expected = normalizeProfile(fought);
  const saving = store.save(fought);
  fought.boss_generations[0][0] = 999;
  await saving;
  const reopened = new SaveStore();
  t.after(() => reopened.db?.close());
  const loaded = await reopened.open();
  assert.deepEqual(loaded, expected);
  loaded.boss_generations[0][0] = 999;
  const equal = structuredClone(expected);
  equal.boss_generations[0] = [29, 1, 0, 0, 0];
  equal.currency = [3000, 4000, 5000];
  await reopened.save(equal);
  assert.deepEqual(
    (await snapshots())[0],
    expected,
    "equal revision cannot replay a boss award or replace its promotion state",
  );
  const next = structuredClone(expected);
  next.revision = 101;
  next.boss_generations[0][4] = 1;
  next.boss_kills[0] = 31;
  await reopened.save(next);
  assert.deepEqual((await snapshots())[0], normalizeProfile(next));
  assert.deepEqual((await snapshots())[1], expected);
});

test("another session cannot overwrite a newer boss defeat with stale counters even at a newer proposed revision", async (t) => {
  const first = await fixture(t);
  const baseline = profile(50, [8, 9, 10]);
  baseline.boss_generations[0] = [8, 4, 2, 1, 0];
  baseline.boss_kills[0] = 15;
  baseline.pending[0] = true;
  await first.save(baseline);
  const second = new SaveStore();
  t.after(() => second.db?.close());
  await second.open();
  const won = structuredClone(baseline);
  won.revision = 51;
  won.boss_generations[0][0] = 9;
  won.boss_kills[0] = 16;
  won.pending[0] = false;
  won.currency[0] = 9;
  await first.save(won);
  const stale = structuredClone(baseline);
  stale.revision = 52;
  stale.currency[0] = 999;
  await assert.rejects(
    second.save(stale),
    (error) => error instanceof SaveConflict,
  );
  assert.deepEqual((await snapshots())[0], normalizeProfile(won));
  assert.deepEqual((await snapshots())[1], normalizeProfile(baseline));
});

test("impossible generation current recovers its prior legacy snapshot and invalid imports cannot replace it", async (t) => {
  const store = await fixture(t);
  const legacy = profile(9, [9, 10, 11]);
  delete legacy.boss_generations;
  legacy.boss_kills[1] = 4;
  legacy.kills[1] = 121;
  legacy.pending[1] = true;
  const invalid = profile(10, [10, 10, 11]);
  invalid.boss_generations[1] = [4, 3, 0, 0, 0];
  invalid.boss_kills[1] = 7;
  await corrupt(invalid, legacy);
  const recovered = new SaveStore();
  t.after(() => recovered.db?.close());
  assert.deepEqual(await recovered.open(), normalizeProfile(legacy));
  assert.match(recovered.message, /Recovered/);
  const before = await snapshots();
  await assert.rejects(recovered.replace(invalid), /supported/);
  assert.deepEqual(await snapshots(), before);
  const imported = profile(3, [1, 2, 3]);
  imported.seed = 42;
  imported.boss_generations[2] = [8, 4, 2, 1, 0];
  imported.boss_kills[2] = 15;
  await recovered.replace(imported);
  imported.boss_generations[2][0] = 999;
  assert.equal((await snapshots())[0].boss_generations[2][0], 8);
  await store.flush();
});
