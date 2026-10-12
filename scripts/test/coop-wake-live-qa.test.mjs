import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import vm from "node:vm";
import ts from "typescript";
import { webcrypto } from "node:crypto";
import { normalizeProfile } from "../../apps/game/src/storage.ts";
import {
  WAKE_SCENARIO,
  WakeObservation,
  freshWakeOrigin,
  requireWakeStage,
  requireWakeProfile,
  requireWakeGalaxy,
  sameValue,
  validateWakeReport,
  wakeReasons,
} from "../qa/co-op-transport/wake-observation.js";
import {
  readWakeFixtures,
  verifyWakeStage,
  wakePreparedFiles,
} from "../qa/co-op-transport/wake-fixtures.mjs";
import {
  directory,
  root,
  preparedNames,
  prepare,
  sha,
} from "../qa/co-op-transport/prepare.mjs";
import {
  validateWasmReport,
  wakeFixtureRoleAllowed,
} from "../qa/co-op-transport/server.mjs";
import { starterSelection } from "../qa/co-op-transport/fifth-shot.js";

// These are listener-free helper/declared API-boundary tests, not WASM/RTC execution.
const fixture = await readWakeFixtures(directory, root);
const profiles = Object.fromEntries(
  ["host", "guest"].map((role) => [
    role,
    JSON.parse(
      fixture.files.get(`fixtures/configured-wake/${role}-profile.json`),
    ),
  ]),
);
const builds = {
  webgpu: `/wasm/webgpu-${"a".repeat(64)}/galaxy.js`,
  webgl2: `/wasm/webgl2-${"b".repeat(64)}/galaxy.js`,
};
const config = {
  mode: "wasm",
  profile_mode: "owned",
  scenarios: ["default", WAKE_SCENARIO],
  builds,
  source_commit: "c".repeat(40),
  build_stamp: { head: "c".repeat(40) },
  wake_fixture: fixture.metadata,
};
const ids = ["a".repeat(32), "b".repeat(32)],
  session = "c".repeat(32);
const header = (
  tick = 10,
  points = 2,
  positions = [
    [400, 0],
    [402, 0],
  ],
) => ({
  acquisition: tick,
  snapshot: {
    session,
    tick,
    revision: tick,
    progression: false,
    participants: positions.map((position, slot) => ({
      id: ids[slot],
      slot,
      epoch: slot + 1,
      position,
    })),
    wake_fx: { points },
  },
});
const page = (
  h,
  points = [0, 1].map((slot) => ({
    slot,
    epoch: slot + 1,
    born_tick: h.snapshot.tick,
    position: [380 + slot * 2, 0],
    life: 2,
  })),
) => ({
  session: h.snapshot.session,
  tick: h.snapshot.tick,
  revision: h.snapshot.revision,
  points,
});
function capture(observer, h, points) {
  observer.stage(h);
  if (h.snapshot.wake_fx.points) observer.page(h.acquisition, page(h, points));
  if (observer.role === "guest")
    observer.commit(h.acquisition, h.snapshot.revision);
}
function functionSource(source, name) {
  const ast = ts.createSourceFile(
    "qa.js",
    source,
    ts.ScriptTarget.ES2022,
    true,
    ts.ScriptKind.JS,
  );
  return ast.statements
    .find((n) => ts.isFunctionDeclaration(n) && n.name?.text === name)
    .getText(ast);
}
const clientSource = await fs.readFile(
  path.join(directory, "client.js"),
  "utf8",
);
const parentSource = await fs.readFile(
  path.join(directory, "wasm-review.js"),
  "utf8",
);

test("fresh-origin inventory refuses existing Galaxy databases and unsupported/malformed inventory without opening or deleting", async () => {
  const calls = [];
  const db = {
    databases: async () => [{ name: "unrelated", version: 1 }],
    open: () => calls.push("open"),
    deleteDatabase: () => calls.push("delete"),
  };
  assert.equal(await freshWakeOrigin(db), true);
  for (const database of [
    null,
    {},
    { databases: async () => undefined },
    { databases: async () => [{ name: undefined }] },
    {
      databases: async () => {
        throw Error("private");
      },
    },
  ])
    await assert.rejects(freshWakeOrigin(database), /wake_storage_unavailable/);
  db.databases = async () => [{ name: "galaxy-royale", version: 1 }];
  await assert.rejects(freshWakeOrigin(db), /wake_fresh_origin_required/);
  assert.deepEqual(calls, []);
});

test("exact native role fixtures retain production normalization, distinct derived seeds and minimal native equipment", () => {
  for (const role of ["host", "guest"]) {
    const entry = requireWakeStage(config, role);
    assert.equal(
      requireWakeProfile(role, profiles[role], normalizeProfile, entry).seed,
      entry.seed,
    );
    assert.equal(entry.native_export.slots[2].index, 5);
    assert.equal(entry.native_export.slots[2].rank, 1);
    assert.equal(
      Object.hasOwn(entry.native_export.slots[2], "evolution"),
      false,
    );
    assert.ok(profiles[role].campaign.ship_xp.every((v) => v === 0));
    assert.ok(profiles[role].currency.every((v) => v === 0));
    assert.throws(
      () =>
        requireWakeProfile(
          role,
          profiles[role === "host" ? "guest" : "host"],
          normalizeProfile,
          entry,
        ),
      /wake_fixture_mismatch/,
    );
    for (const change of [
      (p) => (p.campaign.abilities[5].rank = 5),
      (p) => (p.campaign.abilities[5].evolution = 5),
      (p) => (p.campaign.loadouts[0].slots[2] = null),
      (p) => (p.schema_version = 6),
    ]) {
      const bad = structuredClone(profiles[role]);
      change(bad);
      assert.throws(() =>
        requireWakeProfile(role, bad, normalizeProfile, entry),
      );
    }
  }
  assert.notEqual(profiles.host.seed, profiles.guest.seed);
  assert.notEqual(
    profiles.host.identity.hex_id,
    profiles.guest.identity.hex_id,
  );
});
test("actual source fixture byte/hash/export drift is refused before any normal-build preparation", async (t) => {
  const probe = await fs.mkdtemp(
    path.join(root, "work/coop-wake-live-qa/fixture-probe-"),
  );
  t.after(async () => {
    assert.ok(
      path
        .resolve(probe)
        .startsWith(path.join(root, "work/coop-wake-live-qa/fixture-probe-")),
    );
    await fs.rm(probe, { recursive: true });
  });
  for (const [name, bytes] of fixture.files) {
    await fs.mkdir(path.dirname(path.join(probe, name)), { recursive: true });
    await fs.writeFile(path.join(probe, name), bytes);
  }
  await readWakeFixtures(probe, root);
  const native = path.join(probe, "fixtures/configured-wake/host-profile.json");
  await fs.appendFile(native, " ");
  await assert.rejects(
    readWakeFixtures(probe, root),
    /differs from actual frozen native output/,
  );
  await fs.writeFile(
    native,
    fixture.files.get("fixtures/configured-wake/host-profile.json"),
  );
  const metadata = structuredClone(fixture.metadata);
  metadata.profiles.host.native_export.slots[2].rank = 5;
  await fs.writeFile(
    path.join(probe, "fixtures/configured-wake/provenance.json"),
    JSON.stringify(metadata),
  );
  await assert.rejects(
    readWakeFixtures(probe, root),
    /differs from actual frozen native output/,
  );
});

test("configured stage is opt-in normal owned WASM with exact helper/native fixture pins; historical allowlists remain unchanged", async () => {
  assert.equal(
    preparedNames({
      version: 3,
      profile_mode: "owned",
      mode: "wasm",
      builds,
      fifth_shot: true,
    }).length,
    19,
  );
  const manifest = {
    version: 3,
    profile_mode: "owned",
    mode: "wasm",
    builds,
    fifth_shot: true,
    configured_wake: true,
    wake_fixture: fixture.metadata,
    inputs: {},
    files: {},
  };
  const names = preparedNames(manifest);
  assert.equal(names.length, 23);
  for (const name of wakePreparedFiles) assert.ok(names.includes(name));
  for (const name of [
    "wasm-review.html",
    "wasm-review.js",
    "client.html",
    "client.js",
    "fifth-shot.js",
    ...wakePreparedFiles,
    "wake-fixtures.mjs",
    "server.mjs",
    "prepare.mjs",
  ]) {
    const hash = sha(await fs.readFile(path.join(directory, name)));
    manifest.inputs[`scripts/qa/co-op-transport/${name}`] = hash;
    if (!name.endsWith(".mjs")) manifest.files[name] = hash;
  }
  await verifyWakeStage(manifest, directory, root);
  for (const mutate of [
    (m) => delete m.inputs["scripts/qa/co-op-transport/client.js"],
    (m) => (m.inputs["scripts/qa/co-op-transport/server.mjs"] = "0".repeat(64)),
    (m) => (m.files["wake-observation.js"] = "0".repeat(64)),
    (m) => (m.wake_fixture.profiles.host.native_export.slots[2].rank = 5),
    (m) => (m.fifth_shot = false),
  ]) {
    const bad = structuredClone(manifest);
    mutate(bad);
    await assert.rejects(verifyWakeStage(bad, directory, root));
  }
  for (const bad of [
    { ...manifest, version: 2 },
    { ...manifest, configured_wake: false },
    { ...manifest, profile_mode: "factory" },
    {
      version: 3,
      profile_mode: "owned",
      mode: "wasm",
      builds,
      wake_fixture: fixture.metadata,
    },
  ])
    assert.throws(() => preparedNames(bad));
  await assert.rejects(
    prepare(path.join(root, "work/coop-wake-live-qa/unissued-stage"), {
      configuredWake: true,
    }),
    /normal owned WASM/,
  );
  for (const bad of [
    { ...config, mode: "diagnostic" },
    { ...config, profile_mode: "factory" },
    { ...config, scenarios: ["default"] },
    { ...config, build_stamp: { head: "d".repeat(40) } },
    { ...config, builds: { ...builds, webgpu: "/diagnostic/galaxy.js" } },
  ])
    assert.throws(
      () => requireWakeStage(bad, "host"),
      /wake_normal_wasm_required/,
    );
  assert.throws(
    () => requireWakeGalaxy({ co_op_batch() {} }),
    /wake_normal_wasm_required/,
  );
});

test("configured profile bytes are role-local HTTP inputs and never admitted from the other role or a foreign Origin", () => {
  const host = "http://localhost:4197",
    guest = "http://guest.localhost:4197";
  for (const role of ["host", "guest"]) {
    const name = `fixtures/configured-wake/${role}-profile.json`,
      own = role === "host" ? host : guest,
      other = role === "host" ? guest : host;
    assert.equal(
      wakeFixtureRoleAllowed(name, new URL(own).host, undefined, host, guest),
      true,
    );
    assert.equal(
      wakeFixtureRoleAllowed(name, new URL(own).host, own, host, guest),
      true,
    );
    assert.equal(
      wakeFixtureRoleAllowed(name, new URL(other).host, undefined, host, guest),
      false,
    );
    assert.equal(
      wakeFixtureRoleAllowed(name, new URL(own).host, other, host, guest),
      false,
    );
  }
});

test("guest wake facts remain uncommitted through header/page staging and rejection; zero commits clear actual active samples", () => {
  const guest = new WakeObservation("guest"),
    host = new WakeObservation("host"),
    h = header();
  host.stage(h);
  host.page(h.acquisition, page(h));
  guest.stage(h);
  guest.page(h.acquisition, page(h));
  assert.equal(host.report().captured_points, 2);
  assert.equal(guest.report().positive_host, false);
  assert.equal(guest.report().captured_points, 0);
  assert.equal(guest.report().complete_commits, 0);
  guest.commit(h.acquisition, h.snapshot.revision);
  assert.equal(guest.report().captured_points, 2);
  assert.equal(guest.report().complete_commits, 1);
  const empty = header(11, 0);
  guest.stage(empty);
  assert.equal(
    guest.report().captured_points,
    2,
    "staged empty header cannot clear a visible native scene",
  );
  guest.commit(empty.acquisition, empty.snapshot.revision);
  assert.equal(guest.report().captured_points, 0);
  assert.equal(
    guest.report().positive_host,
    true,
    "historical observation is distinct from current active data",
  );
  const leaked = JSON.stringify(guest.report());
  for (const privateValue of [session, ...ids, "position", "epoch", "hex_id"])
    assert.ok(!leaked.includes(privateValue));
  validateWakeReport(guest.report(), "guest");
});

test("incomplete, stale, wrong-epoch, duplicate, excessive and underflowed Wake pages cannot become native completion facts", () => {
  const incomplete = new WakeObservation("guest"),
    h = header();
  incomplete.stage(h);
  assert.throws(
    () => incomplete.commit(h.acquisition, h.snapshot.revision),
    /wake_sequence/,
  );
  assert.equal(incomplete.report().positive_guest, false);
  for (const mutate of [
    (p) => p.tick--,
    (p) => p.points[1].epoch++,
    (p) => (p.points[1] = structuredClone(p.points[0])),
    (p) => (p.points[0].life = 1e-46),
    (p) => (p.points[0].life = 2.00000001),
    (p) => (p.points[0].position[0] = 1_000_014.01),
    (p) => p.points[0].born_tick++,
    (p) => (p.points[0].token = "private"),
  ]) {
    const observer = new WakeObservation("guest"),
      bad = page(h);
    mutate(bad);
    observer.stage(h);
    assert.throws(
      () => observer.page(h.acquisition, bad),
      /wake_page_mismatch/,
    );
    assert.equal(observer.report().complete_commits, 0);
    assert.equal(observer.report().positive_host, false);
  }
  const excessive = new WakeObservation("host"),
    many = header(20, 13);
  excessive.stage(many);
  assert.throws(
    () =>
      excessive.page(
        many.acquisition,
        page(
          many,
          Array.from({ length: 13 }, (_, born_tick) => ({
            slot: 0,
            epoch: 1,
            born_tick,
            position: [400, 0],
            life: 1,
          })),
        ),
      ),
    /wake_page_mismatch/,
  );
  const edge = new WakeObservation("host");
  edge.stage(h);
  const allowed = page(h);
  allowed.points[0].position[0] = 1_000_014;
  edge.page(h.acquisition, allowed);
  assert.equal(edge.report().captured_points, 2);
});

test("native tick expiry waits past the latest observed birth; Haven retreat requires both actual committed pilot positions", () => {
  for (const role of ["host", "guest"]) {
    const observer = new WakeObservation(role);
    capture(observer, header());
    observer.stopping();
    capture(observer, header(11));
    capture(observer, header(131, 0));
    assert.equal(observer.report().expired, false);
    capture(observer, header(132, 0));
    assert.equal(observer.report().expired, true);
    observer.driving();
    capture(observer, header(140));
    observer.stopping(true);
    capture(
      observer,
      header(261, 0, [
        [0, 0],
        [200, 0],
      ]),
    );
    assert.equal(observer.report().retreated, false);
    capture(
      observer,
      header(262, 0, [
        [0, 0],
        [100, 0],
      ]),
    );
    assert.equal(observer.report().retreated, true);
    validateWakeReport(observer.report(), role);
    observer.end();
    assert.equal(observer.report().captured_points, 0);
    assert.equal(observer.report().local_view_ended, true);
  }
  const untouched = new WakeObservation("host");
  capture(untouched, header(10, 0));
  assert.throws(() => untouched.stopping(), /wake_sequence/);
});

test("leave removal binds actual owner and epoch and failure/death remains inconclusive after end", () => {
  const h = new WakeObservation("host");
  capture(h, header());
  h.leavingOwner(ids[1], 2);
  capture(h, header(11, 0));
  assert.equal(h.report().removed_owner, false);
  const replacement = header(12, 0);
  replacement.snapshot.participants[1].epoch = 3;
  capture(h, replacement);
  assert.equal(h.report().removed_owner, true);
  assert.throws(
    () =>
      h.frame({
        events: [{ kind: "death" }],
        pixels: 32,
        render_ms: 0,
        simulation_ms: 0,
      }),
    /wake_native_frame/,
  );
  h.end();
  assert.equal(h.report().state, "inconclusive");
  assert.equal(h.report().reason, "wake_native_frame");
  validateWakeReport(h.report(), "host");
});
test("a new session resets its native birth/tick fence even after end while retaining clearly historical facts", () => {
  const observer = new WakeObservation("host");
  capture(observer, header(800));
  observer.end();
  const fresh = header(4);
  fresh.snapshot.session = "d".repeat(32);
  capture(observer, fresh);
  observer.stopping();
  assert.equal(observer.report().last_birth_tick, 4);
  const empty = header(125, 0);
  empty.snapshot.session = fresh.snapshot.session;
  capture(observer, empty);
  assert.equal(observer.report().expired, true);
  assert.equal(observer.report().last_complete_tick, 125);
  observer.end();
  const soloBirth = header(1, 1);
  soloBirth.snapshot.session = "e".repeat(32);
  capture(observer, soloBirth, [
    { slot: 0, epoch: 1, born_tick: 1, position: [400, 0], life: 2 },
  ]);
  assert.throws(
    () => observer.stopping(),
    /wake_sequence/,
    "an old guest sighting cannot qualify a new session's expiry",
  );
});
test("render observations require the real finite native frame shape and cannot promote zero, missing or saturated frames", () => {
  const frame = { events: [], pixels: 1024, render_ms: 1.5, simulation_ms: 0 };
  const observer = new WakeObservation("host");
  observer.frame(frame);
  assert.equal(observer.report().native_render_frames, 1);
  assert.equal(observer.report().last_native_pixels, 1024);
  for (const bad of [
    undefined,
    { ...frame, pixels: 0 },
    { ...frame, pixels: Infinity },
    { ...frame, render_ms: NaN },
    {
      ...frame,
      events: Array.from({ length: 128 }, () => ({ kind: "impact" })),
    },
  ]) {
    const rejected = new WakeObservation("guest");
    assert.throws(() => rejected.frame(bad), /wake_native_frame/);
    assert.equal(rejected.report().native_render_frames, 0);
  }
});

function initializationContext({
  inventory = [],
  loaded = null,
  nativeChange = () => {},
  role = "host",
} = {}) {
  const calls = [],
    profile = structuredClone(profiles[role]);
  let opens = 0,
    observed;
  const native = {
    profile: () => profile,
    co_op_batch: () => "{}",
    co_op_replica: () => "{}",
    hud: () => ({}),
    step: () => ({}),
    set_paused: () => calls.push("pause"),
    cancel_course() {},
    resize() {},
    set_ui_mode() {},
    set_shake() {},
    set_reduced_motion() {},
    free: () => calls.push("free"),
  };
  nativeChange(native, profile);
  const context = vm.createContext({
    galaxy: null,
    controller: null,
    scenario: "default",
    fifth: null,
    wake: null,
    wakeApi: null,
    config,
    role,
    canvas: {},
    failureCode: null,
    initStage: "none",
    initDeadline: 0,
    performance: { now: () => 0 },
    setTimeout,
    clearTimeout,
    crypto: webcrypto,
    Uint8Array,
    TextDecoder,
    indexedDB: {
      databases: async () => {
        calls.push("inventory");
        return inventory;
      },
    },
    loadWake: async () => ({
      WakeObservation,
      requireWakeStage,
      freshWakeOrigin,
      requireWakeProfile,
      requireWakeGalaxy,
      sameValue,
      wakeReasons,
    }),
    SaveStore: class {
      persistent = true;
      constructor() {
        calls.push("store");
      }
      open() {
        calls.push("open");
        return ++opens === 1 ? loaded : profile;
      }
      save(value) {
        assert.ok(sameValue(value, profiles[role]));
        calls.push("save");
      }
      flush() {
        calls.push("flush");
      }
    },
    normalizeProfile,
    starterSelection,
    update() {},
    schedule() {},
    digest: async () => "a".repeat(64),
    digestValue: async () => "b".repeat(64),
    fetch: async () => ({
      ok: true,
      arrayBuffer: async () =>
        fixture.files.get(`fixtures/configured-wake/${role}-profile.json`),
    }),
    loadModule: async () => {
      calls.push("import");
      return {
        default: async () => calls.push("compile"),
        Galaxy: {
          create: async (_canvas, value) => {
            assert.ok(sameValue(value, profiles[role]));
            calls.push("create");
            return native;
          },
        },
      };
    },
    wasmPorts: {
      readOwnedLoadout: () => fixture.metadata.profiles[role].native_export,
      createGalaxyCoopPorts: (value) => {
        observed = value;
        return {};
      },
    },
    localSpec() {},
    CoopController: class {},
  });
  vm.runInContext(functionSource(clientSource, "initStep"), context);
  vm.runInContext(
    functionSource(clientSource, "initialize")
      .replace('import("/wake-observation.js")', "loadWake()")
      .replace("import(config.builds[backend])", "loadModule()"),
    context,
  );
  return { context, calls, getObserved: () => observed };
}
test("actual configured initialization preflight precedes SaveStore/native creation and constructor mismatch prevents persistence", async () => {
  for (const options of [
    { inventory: [{ name: "galaxy-royale" }] },
    { inventory: null },
  ]) {
    const { context, calls } = initializationContext(options);
    await assert.rejects(
      context.initialize("webgpu", "a".repeat(16), WAKE_SCENARIO),
    );
    assert.deepEqual(calls, ["inventory"]);
    assert.equal(context.galaxy, null);
    assert.ok(wakeReasons.has(context.failureCode));
  }
  const retained = initializationContext({ loaded: profiles.host });
  await assert.rejects(
    retained.context.initialize("webgpu", "a".repeat(16), WAKE_SCENARIO),
    /wake_fresh_origin_required/,
  );
  assert.ok(!retained.calls.includes("import"));
  const mutated = initializationContext({
    nativeChange: (_native, p) => p.currency[0]++,
  });
  await assert.rejects(
    mutated.context.initialize("webgpu", "a".repeat(16), WAKE_SCENARIO),
  );
  assert.ok(!mutated.calls.includes("save"));
  const valid = initializationContext();
  await valid.context.initialize("webgpu", "a".repeat(16), WAKE_SCENARIO);
  assert.deepEqual(valid.calls, [
    "inventory",
    "store",
    "open",
    "import",
    "compile",
    "create",
    "pause",
    "save",
    "flush",
    "open",
  ]);
  assert.equal(valid.context.wake.report().native_export_match, true);
});
function observationBindings(context) {
  Object.assign(context, {
    counters: { captures: 0, entity_pages: 0, body_pages: 0, commits: 0 },
    observeRoomOperation() {},
    observeHeader() {},
    observeRows() {},
    observeCommit() {},
    stagedRows: new Map(),
  });
}
test("actual generated-native wrapper requires successful replica_wake/commit/end replies before visible Wake facts", async () => {
  let accepted = true;
  const value = initializationContext({
    role: "guest",
    nativeChange: (native) => {
      native.co_op_replica = () =>
        JSON.stringify(
          accepted
            ? { v: 1, ok: true, value: null }
            : { v: 1, ok: false, error: "native_rejected" },
        );
    },
  });
  await value.context.initialize("webgl2", "a".repeat(16), WAKE_SCENARIO);
  observationBindings(value.context);
  const observed = value.getObserved(),
    h = header(),
    invoke = (operation) =>
      observed.co_op_replica(JSON.stringify({ v: 1, ...operation }));
  invoke({ op: "replica_stage", header: h });
  accepted = false;
  invoke({ op: "replica_wake", acquisition: h.acquisition, page: page(h) });
  assert.equal(value.context.wake.report().native_pages, 0);
  accepted = true;
  invoke({ op: "replica_wake", acquisition: h.acquisition, page: page(h) });
  accepted = false;
  invoke({
    op: "replica_commit",
    acquisition: h.acquisition,
    revision: h.snapshot.revision,
  });
  assert.equal(value.context.wake.report().captured_points, 0);
  assert.equal(value.context.wake.report().complete_commits, 0);
  accepted = true;
  invoke({
    op: "replica_commit",
    acquisition: h.acquisition,
    revision: h.snapshot.revision,
  });
  assert.equal(value.context.wake.report().captured_points, 2);
  accepted = false;
  invoke({ op: "replica_end", session });
  assert.equal(value.context.wake.report().local_view_ended, false);
  accepted = true;
  invoke({ op: "replica_end", session });
  assert.equal(value.context.wake.report().local_view_ended, true);
  assert.equal(value.context.wake.report().captured_points, 0);
});
test("actual last-guest leave wrapper performs only a bounded real-native removal capture/page without inventing a guest commit", async () => {
  const calls = [],
    afterLeave = header(11, 1);
  afterLeave.snapshot.participants = [afterLeave.snapshot.participants[0]];
  const value = initializationContext({
    nativeChange: (native) => {
      native.co_op_batch = (raw) => {
        const operations = JSON.parse(raw).operations;
        assert.equal(operations.length, 1);
        const op = operations[0];
        calls.push(op);
        return JSON.stringify({
          v: 1,
          results: [
            {
              ok: true,
              value:
                op.op === "acquire"
                  ? afterLeave
                  : op.op === "wake_page"
                    ? page(afterLeave, [
                        {
                          slot: 0,
                          epoch: 1,
                          born_tick: 10,
                          position: [380, 0],
                          life: 1.9,
                        },
                      ])
                    : null,
            },
          ],
        });
      };
    },
  });
  await value.context.initialize("webgpu", "a".repeat(16), WAKE_SCENARIO);
  observationBindings(value.context);
  vm.runInContext(
    functionSource(clientSource, "observeWakeRemoval"),
    value.context,
  );
  capture(value.context.wake, header());
  value.getObserved().co_op_batch(
    JSON.stringify({
      v: 1,
      operations: [{ op: "leave", session, peer: ids[1], epoch: 2 }],
    }),
  );
  assert.deepEqual(
    calls.map((v) => v.op),
    ["leave", "acquire", "wake_page"],
  );
  assert.equal(value.context.wake.report().removed_owner, true);
  assert.equal(value.context.wake.report().captured_guest, false);
  assert.equal(value.context.wake.report().captured_host, true);
  assert.equal(value.context.wake.report().complete_commits, 0);
  assert.equal(value.context.counters.commits, 0);
});

test("actual Wake movement uses fire-off native steps/current commits and caps phase work; it never creates trail points", async () => {
  const observer = new WakeObservation("host"),
    calls = [],
    h = header();
  capture(observer, h);
  const context = vm.createContext({
    scenario: WAKE_SCENARIO,
    role: "host",
    wake: observer,
    controller: { frameMode: () => "host", blocksSolo: () => true },
    galaxy: { hud: () => ({ time: 10 }) },
    stopped: false,
    loopDeadline: 600_000,
    performance: { now: () => 0 },
    controls: {},
    pumping: false,
    wakeCruise: null,
    resetControls: () => calls.push("cancel"),
    step: () => calls.push(structuredClone(context.controls)),
    observations: () => observer.report(),
    setTimeout: (fn) => {
      fn();
      return 0;
    },
  });
  for (const name of ["wakeContext", "wakeCommand"])
    vm.runInContext(functionSource(clientSource, name), context);
  const result = await context.wakeCommand("wake-drive", {});
  assert.equal(result.captured_points, 2);
  assert.deepEqual(calls[1], { joystick: [1, 0], fire: false });
  assert.equal(context.pumping, false);
  assert.equal(context.wakeCruise.time, 10);
  assert.equal(observer.current.points.length, 2);
  context.step = () => calls.push(structuredClone(context.controls));
  await assert.rejects(context.wakeCommand("wake-retreat", {}), /wake_timeout/);
  assert.equal(calls.filter((v) => v?.destination).length, 960);
  assert.ok(
    calls.filter((v) => typeof v === "object").every((v) => v.fire === false),
  );
  assert.equal(context.pumping, false);
  assert.equal(context.wakeCruise, null);
});

test("actual retirement latches failure, waits for native restore, frees once even after read failure, and refuses blocked release", async () => {
  let resolveDispose,
    disposed = 0,
    freed = 0;
  const context = vm.createContext({
    wakeRetiring: null,
    wakeCruise: {},
    stopped: false,
    timer: 1,
    failureCode: null,
    phase: "hosting",
    wake: new WakeObservation("host"),
    wakeApi: { wakeReasons: new Set(["wake_timeout"]) },
    resetControls() {},
    clearTimeout() {},
    update() {},
    controller: {
      dispose: () => {
        disposed++;
        return new Promise((r) => (resolveDispose = r));
      },
      blocksSolo: () => false,
    },
    galaxy: { free: () => freed++ },
    verify: async () => {
      throw Error("read failed");
    },
  });
  vm.runInContext(functionSource(clientSource, "retireWake"), context);
  const first = context.retireWake("wake_timeout"),
    second = context.retireWake("wake_timeout");
  assert.equal(context.stopped, true);
  assert.equal(freed, 0);
  assert.equal(disposed, 1);
  resolveDispose();
  await assert.rejects(first, /read failed/);
  await assert.rejects(second, /read failed/);
  assert.equal(freed, 1);
  assert.equal(context.galaxy, null);
  context.wakeRetiring = null;
  context.galaxy = { free: () => freed++ };
  context.controller = { dispose: async () => {}, blocksSolo: () => true };
  await assert.rejects(context.retireWake("wake_timeout"), /restore_failed/);
  assert.equal(
    freed,
    1,
    "blocked native owner is not freed on a fabricated restore",
  );
});

function report() {
  const clients = ["host", "guest"].map((role) => {
    const observer = new WakeObservation(role);
    capture(observer, header());
    observer.frame({
      events: [],
      pixels: 1024,
      render_ms: 1,
      simulation_ms: 1,
    });
    Object.assign(observer.value, {
      fresh_origin: true,
      fixture_valid: true,
      native_export_match: true,
    });
    return {
      role,
      phase: role === "host" ? "hosting" : "connected",
      mode: role,
      init_stage: "ready",
      failure_code: null,
      ...Object.fromEntries(
        [
          "created",
          "closed",
          "requests",
          "timers",
          "frames",
          "ticks",
          "captures",
          "entity_pages",
          "body_pages",
          "commits",
          "body_changes",
          "same_count_changes",
          "unchanged_bodies",
          "projectiles",
          "transport_ready",
          "admitted",
        ].map((k) => [k, 0]),
      ),
      ...Object.fromEntries(
        [
          "profile_equal",
          "persisted_equal",
          "persistent",
          "progression_false",
          "movement_observed",
        ].map((k) => [k, true]),
      ),
      ...Object.fromEntries(
        ["profile_sha", "baseline_sha", "persisted_sha", "owned_sha"].map(
          (k) => [k, "a".repeat(64)],
        ),
      ),
      owned_export: true,
      owned_view: role === "guest",
      selected_ship: 0,
      owned_slots: 1,
      wake: observer.report(),
    };
  });
  return {
    version: 5,
    scenario: WAKE_SCENARIO,
    fixture_provenance: fixture.metadata.provenance,
    profile_mode: "owned",
    backend: "webgl2",
    run_id: "a".repeat(16),
    session_distinct: null,
    checks: [
      { name: "wake-configured-profiles", passed: true },
      { name: "wake-native-owners", passed: true },
    ],
    clients,
  };
}
test("version-five sanitized reports separate configured/native facts from earned/visual claims and reject unsupported success", () => {
  const good = report();
  assert.equal(validateWasmReport(good), good);
  for (const mutate of [
    (r) => (r.version = 3),
    (r) => (r.scenario = "default"),
    (r) => (r.fixture_provenance = "earned"),
    (r) => (r.clients[0].wake.fresh_origin = null),
    (r) => (r.clients[1].wake.complete_commits = 0),
    (r) => (r.clients[1].wake.visual_review = "passed"),
    (r) => (r.clients[0].wake.epoch = 2),
    (r) => (r.clients[0].profile = profiles.host),
    (r) => r.checks.push({ name: "wake-authoritative-expiry", passed: true }),
    (r) => (r.clients[0].wake.peak_points = 25),
    (r) => (r.clients[0].wake.state = "inconclusive"),
  ]) {
    const bad = structuredClone(good);
    mutate(bad);
    assert.throws(() => validateWasmReport(bad));
  }
  const early = report();
  early.clients[1].init_stage = "wake_preflight";
  early.clients[1].phase = "failed";
  early.clients[1].wake = null;
  early.checks = [{ name: "operation-failed", passed: false }];
  assert.equal(
    validateWasmReport(early),
    early,
    "an honest preflight failure remains capturable",
  );
});

test("actual parent Wake phase guards visibility through final verification and performs explicit failed cleanup", async () => {
  let hidden = true,
    sent = [],
    passed = 0;
  const state = { host: { mode: "host" }, guest: { phase: "connected" } };
  const context = vm.createContext({
    scenario: WAKE_SCENARIO,
    state,
    driveActive: false,
    driveVisibilityLost: false,
    requireVisibleClients: () => {
      if (hidden) throw Error("page_not_visible");
    },
    guardDriveVisibility: () => {
      if (hidden) throw Error("page_not_visible");
    },
    send: async (role, name, _args, timeout) => {
      sent.push({ role, name, timeout });
      return state[role];
    },
    until: async (predicate) => {
      for (const value of Object.values(state))
        Object.assign(value, {
          failure_code: null,
          wake: {
            captured_host: true,
            captured_guest: true,
            native_render_frames: 1,
          },
        });
      assert.equal(predicate(), true);
    },
    verify: async () => {
      hidden = true;
    },
    check: () => passed++,
  });
  vm.runInContext(functionSource(parentSource, "wakeRound"), context);
  await assert.rejects(context.wakeRound("drive"), /page_not_visible/);
  assert.deepEqual(sent, []);
  hidden = false;
  await assert.rejects(context.wakeRound("drive"), /page_not_visible/);
  assert.equal(passed, 0);
  assert.deepEqual(
    sent.map((v) => v.name),
    ["wake-drive", "wake-drive", "dispose", "dispose"],
  );
  assert.ok(sent.slice(-2).every((v) => v.timeout === 20_000));
  assert.equal(context.driveActive, false);
  assert.equal(context.disposed, true);
});
