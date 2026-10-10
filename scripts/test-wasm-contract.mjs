import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { IDBFactory } from "fake-indexeddb";
import { SaveStore, validProfile } from "../apps/game/src/storage.ts";

const root = resolve(import.meta.dirname, "..");
const output = resolve(root, "work", "wasm-contract");
const target = resolve(output, "target");
const backends = [
  ["webgpu", "webgpu"],
  ["webgl", "webgl2"],
];
mkdirSync(output, { recursive: true });

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`${command} failed with exit code ${result.status}.`);
}

async function checkpoint(module, stage, backend) {
  const profile = module.checkpoint_contract(stage);
  assert.equal(profile.schema_version, 3);
  assert.equal(Object.hasOwn(profile.campaign, "last_run"), true);
  assert.equal(
    profile.campaign.last_run,
    null,
    `${backend}/${stage}: Rust None must cross WASM as explicit null`,
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
  assert.deepEqual(
    profile.currency,
    [0, 0, 0],
    "QA setup must not award category points",
  );
  assert.ok(
    profile.campaign.patches.every((patch) => patch === false),
    "QA setup must not award champion ownership",
  );

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
      await checkpoint(module, "fresh", backend);
      await checkpoint(module, "champion", backend);

      const production = resolve(
        root,
        "apps",
        "game",
        "public",
        "wasm",
        backend,
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
        `PASS ${backend}: fresh + champion checkpoints validate and persist; release excludes QA probe.`,
      );
    }
  } finally {
    if (indexedDbBefore === undefined) delete globalThis.indexedDB;
    else globalThis.indexedDB = indexedDbBefore;
  }
  console.log(
    "PASS: four real WASM checkpoint/save/reload scenarios across two backends; no browser or GPU mocks.",
  );
} catch (error) {
  console.error(error);
  process.exitCode = 1;
}
