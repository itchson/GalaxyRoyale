import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { IDBFactory } from "fake-indexeddb";
import { SaveStore, validProfile } from "../apps/game/src/storage.ts";
import { readWasmBuilds } from "./wasm-artifacts.mjs";

const root = resolve(import.meta.dirname, "..");
const output = resolve(root, "work", "wasm-contract");
const target = resolve(output, "target");
const backends = [
  ["webgpu", "webgpu"],
  ["webgl", "webgl2"],
];
const zeroGenerations = () => Array.from({ length: 3 }, () => Array(5).fill(0));
const zeroMaterials = () => Array(5).fill(0);
const runFixture = {
  time: 3,
  distance_peak: 1800,
  enemies: 2,
  asteroids: 3,
  champions: 0,
  points_gained: [4, 5, 6],
  xp_gained: 12,
  materials_gained: 15,
  materials_lost: [1, 2, 3, 4, 5],
};
const stages = {
  fresh: {
    currency: [0, 0, 0],
    boss_kills: [0, 0, 0],
    boss_generations: zeroGenerations(),
  },
  champion: {
    currency: [0, 0, 0],
    boss_kills: [0, 0, 0],
    boss_generations: zeroGenerations(),
  },
  generations: {
    currency: [3, 4, 5],
    kills: [470, 158, 660],
    boss_kills: [34, 10, 48],
    boss_generations: [
      [18, 9, 4, 2, 1],
      [6, 3, 1, 0, 0],
      [26, 13, 6, 2, 1],
    ],
    pending: [true, false, true],
  },
  "legacy-bosses": {
    currency: [7, 8, 9],
    kills: [120, 60, 220],
    boss_kills: [4, 2, 8],
    boss_generations: [
      [4, 0, 0, 0, 0],
      [2, 0, 0, 0, 0],
      [8, 0, 0, 0, 0],
    ],
    pending: [true, false, true],
  },
  "legacy-cargo": {
    currency: [20, 30, 40],
    ranks: [2, 1, 3, 0, 0, 0, 4, 2, 1],
    unlocked: [true, true, true],
    campaign: {
      selected_ship: 2,
      cargo_materials: zeroMaterials(),
      stored_materials: [12, 0, 0, 0, 0],
    },
  },
  inventory: {
    currency: [37, 97, 97],
    ranks: [2, 0, 0, 0, 0, 0, 1, 0, 0],
    unlocked: [true, true, true],
    campaign: {
      selected_ship: 2,
      cargo_materials: zeroMaterials(),
      stored_materials: [37, 47, 50, 50, 50],
    },
  },
  // These two typed serializer fixtures are not pickup/earned reward evidence.
  "tiered-cargo": {
    currency: [0, 0, 0],
    campaign: {
      cargo_materials: [1, 2, 3, 4, 5],
      stored_materials: [10, 20, 30, 40, 50],
    },
  },
  "cargo-reload": {
    currency: [0, 0, 0],
    campaign: {
      cargo_materials: zeroMaterials(),
      stored_materials: [10, 20, 30, 40, 50],
    },
  },
  "loss-summary": { currency: [0, 0, 0], campaign: { last_run: runFixture } },
};
mkdirSync(output, { recursive: true });

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`${command} failed with exit code ${result.status}.`);
}

async function checkpoint(module, stage, backend) {
  const profile = module.checkpoint_contract(stage);
  const expected = stages[stage];
  assert.equal(profile.schema_version, 4);
  assert.equal(Object.hasOwn(profile.campaign, "last_run"), true);
  assert.notEqual(
    profile.campaign.last_run,
    undefined,
    `${backend}/${stage}: Rust Option must never omit the owned field`,
  );
  assert.deepEqual(
    profile.campaign.last_run,
    stage === "loss-summary" ? runFixture : null,
  );
  assert.equal(
    validProfile(profile),
    true,
    `${backend}/${stage}: actual Rust checkpoint must pass the TypeScript boundary`,
  );
  const jsonCopy = JSON.parse(JSON.stringify(profile));
  assert.equal(
    Object.hasOwn(jsonCopy.campaign, "last_run"),
    true,
    "exported JSON must retain the optional field rather than omit undefined",
  );
  assert.deepEqual(jsonCopy, profile);
  for (const [field, value] of Object.entries(expected)) {
    if (field === "campaign") {
      for (const [key, nested] of Object.entries(value))
        assert.deepEqual(
          profile.campaign[key],
          nested,
          `${backend}/${stage}: exact campaign.${key}`,
        );
    } else
      assert.deepEqual(
        profile[field],
        value,
        `${backend}/${stage}: real Rust normalization must preserve ${field}`,
      );
  }
  if (stage === "legacy-bosses" || stage === "legacy-cargo")
    assert.ok(
      profile.revision >= 9,
      "runtime migration may bump revision but must not lose the existing checkpoint",
    );
  if (!["inventory", "legacy-cargo"].includes(stage))
    assert.ok(
      profile.campaign.patches.every((patch) => patch === false),
      "QA diagnostic must not award champion ownership",
    );
  assert.equal(profile.campaign.abilities.length, 17);
  assert.equal(profile.campaign.loadouts.length, 10);
  const selected = profile.campaign.loadouts[profile.campaign.selected_ship];
  assert.deepEqual(
    profile.ranks.slice(0, 6),
    selected.hardware,
    "selected hull owns the hardware compatibility mirror",
  );
  assert.deepEqual(
    profile.ranks.slice(6),
    profile.campaign.abilities.slice(3, 6).map((a) => a.rank),
  );
  assert.deepEqual(
    profile.unlocked,
    profile.campaign.abilities.slice(3, 6).map((a) => a.blueprint),
  );
  for (const loadout of profile.campaign.loadouts) {
    assert.equal(loadout.hardware.length, 6);
    assert.equal(loadout.slots.length, 3);
    assert.ok(
      loadout.slots.every(
        (id) => id === null || (Number.isInteger(id) && id >= 0 && id < 17),
      ),
      "empty Rust slots must serialize as null",
    );
  }
  for (const key of ["cargo_materials", "stored_materials"])
    assert.equal(profile.campaign[key].length, 5);
  if (stage === "inventory") {
    assert.deepEqual(
      profile.campaign.abilities.map((a) => a.rank),
      [2, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 5, 5, 5],
    );
    assert.ok(profile.campaign.abilities.every((a) => a.blueprint));
    assert.deepEqual(selected.hardware, [2, 0, 0, 0, 0, 0]);
    assert.deepEqual(selected.slots, [7, 1, 2]);
    assert.equal(profile.campaign.ship_xp[2], 60);
    assert.equal(
      profile.campaign.ship_xp[9],
      0,
      "royal inventory fixture must not fabricate Sovereign XP",
    );
    assert.deepEqual(
      profile.campaign.loadouts[9].slots,
      [null, null, null],
      "v4 normalization must retain deliberately empty royal slots",
    );
    assert.ok(
      profile.campaign.loadouts
        .filter((_, i) => i !== 2)
        .every((l) => l.hardware.every((r) => r === 0)),
      "hardware purchases must not leak into other hulls",
    );
  }
  if (stage === "legacy-cargo") {
    assert.deepEqual(selected.hardware, [2, 1, 3, 0, 0, 0]);
    assert.deepEqual(selected.slots, [7, 4, 5]);
    assert.deepEqual(profile.campaign.loadouts[0].slots, [3, 4, 5]);
    assert.equal(profile.campaign.ship_xp[2], 60);
    assert.equal(profile.campaign.abilities[7].rank, 1);
  }

  // Exercise real SaveStore transactions in an isolated fake-indexeddb database.
  globalThis.indexedDB = new IDBFactory();
  const store = new SaveStore();
  const reloaded = new SaveStore();
  try {
    assert.equal(await store.open(), null);
    await store.save(profile);
    await store.flush();
    const loaded = await reloaded.open();
    assert.deepEqual(
      loaded,
      profile,
      `${backend}/${stage}: WASM checkpoint must survive save/reload exactly`,
    );
    await reloaded.save(loaded);
    assert.deepEqual(
      await reloaded.open(),
      profile,
      "an equal-revision replay must retain the same checkpoint without inventing rewards",
    );
  } finally {
    store.db?.close();
    reloaded.db?.close();
  }
}

try {
  const indexedDbBefore = globalThis.indexedDB;
  try {
    for (const [feature, backend] of backends) {
      console.log(`Building isolated ${backend} WASM checkpoint contract.`);
      run("cargo", [
        "build",
        "--locked",
        "--release",
        "--target",
        "wasm32-unknown-unknown",
        "--target-dir",
        target,
        "-p",
        "game_web",
        "--no-default-features",
        "--features",
        `${feature},qa-contract`,
      ]);
      const bindingOutput = resolve(output, backend);
      mkdirSync(bindingOutput, { recursive: true });
      run("wasm-bindgen", [
        resolve(target, "wasm32-unknown-unknown", "release", "game_web.wasm"),
        "--target",
        "web",
        "--out-dir",
        bindingOutput,
        "--out-name",
        "galaxy",
      ]);
      const module = await import(
        pathToFileURL(resolve(bindingOutput, "galaxy.js"))
      );
      module.initSync({
        module: readFileSync(resolve(bindingOutput, "galaxy_bg.wasm")),
      });
      assert.equal(
        typeof module.checkpoint_contract,
        "function",
        "QA build must expose the real shared-serializer contract probe",
      );
      for (const stage of Object.keys(stages))
        await checkpoint(module, stage, backend);

      const production = dirname(
        resolve(
          root,
          "apps/game/public",
          readWasmBuilds(root)[backend].slice(1),
        ),
      );
      assert.ok(
        existsSync(resolve(production, "galaxy.js")),
        "Production WASM package is required; run pnpm build:wasm first.",
      );
      const release = await import(
        pathToFileURL(resolve(production, "galaxy.js"))
      );
      assert.equal(
        Object.hasOwn(release, "checkpoint_contract"),
        false,
        "QA fixture must never appear in the production JavaScript API",
      );
      const wasmModule = new WebAssembly.Module(
        readFileSync(resolve(production, "galaxy_bg.wasm")),
      );
      assert.ok(
        !WebAssembly.Module.exports(wasmModule).some((value) =>
          value.name.includes("checkpoint_contract"),
        ),
        "QA fixture must never appear in the production WASM exports",
      );
      console.log(
        `PASS ${backend}: ${Object.keys(stages).length} actual Rust serializer/save/reload cases; release excludes QA probe.`,
      );
    }
  } finally {
    if (indexedDbBefore === undefined) delete globalThis.indexedDB;
    else globalThis.indexedDB = indexedDbBefore;
  }
  console.log(
    `PASS: ${Object.keys(stages).length * backends.length} headless WASM serializer/SaveStore cases. Supplied inventory/history fixtures are not earned reward proof; IndexedDB uses fake-indexeddb. No browser/GPU/device acceptance.`,
  );
} catch (error) {
  console.error(error);
  process.exitCode = 1;
}
