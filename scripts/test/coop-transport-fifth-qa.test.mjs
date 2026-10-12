import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import vm from "node:vm";
import ts from "typescript";
import {
  FifthShotObservation,
  starterSelection,
  validateFifthReport,
} from "../qa/co-op-transport/fifth-shot.js";
import { validateWasmReport } from "../qa/co-op-transport/server.mjs";
import { preparedNames } from "../qa/co-op-transport/prepare.mjs";

// Configured observation inputs exercise the verifier, not native/WASM/RTC execution.
const owner = "a".repeat(32),
  session = "b".repeat(32);
const snapshot = (projectiles = [], overrides = {}) => ({
  session,
  progression: false,
  participants: [
    {
      id: owner,
      slot: 0,
      epoch: 1,
      identity: { corporation: "renko" },
      selected_ship: 0,
      hull: 20,
      heading: 0,
    },
  ],
  projectiles,
  ...overrides,
});
const shot = (n, bonus = false, overrides = {}) => ({
  id: n.toString(16).padStart(16, "0"),
  owner,
  owner_epoch: 1,
  damage: bonus ? 21.78 : 14.52,
  remaining_hits: 1,
  velocity: [bonus ? 504 : 434, -3],
  ...overrides,
});
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
test("starter choice keeps defaults and refuses nonempty saves before optional RENKO registration", () => {
  assert.deepEqual(starterSelection("default", "host", { seed: 1 }, false), [
    "dora",
    "pan",
    "renko",
  ]);
  assert.deepEqual(starterSelection("default", "guest", null, false), [
    "doro",
    "hugo",
    "pan",
  ]);
  assert.deepEqual(starterSelection("renko-fifth", "host", null, true), [
    "dora",
    "renko",
    "renko",
  ]);
  for (const loaded of [{}, { identity: null }, undefined, false])
    assert.throws(
      () => starterSelection("renko-fifth", "host", loaded, true),
      /fresh_save_required/,
    );
  assert.throws(
    () => starterSelection("renko-fifth", "host", null, false),
    /scenario_unavailable/,
  );
  assert.throws(
    () => starterSelection("unknown", "host", null, true),
    /scenario_unavailable/,
  );
});
test("verifier counts admitted events, binds owner/epoch, proves fifth and tenth and refuses replay", () => {
  const host = new FifthShotObservation("host"),
    guest = new FifthShotObservation("guest");
  host.start(snapshot());
  guest.start(snapshot());
  let id = 0;
  for (let ordinal = 1; ordinal <= 10; ordinal++) {
    host.fire(ordinal);
    assert.equal(host.events([{ kind: "impact" }]), false);
    assert.equal(host.events([{ kind: "shot" }]), true);
    const rows = [shot(++id)];
    if (ordinal % 5 === 0) rows.push(shot(++id, true));
    // Unrelated enemy projectiles cannot impersonate the authenticated owner.
    rows.push(shot(++id, false, { owner: null, owner_epoch: null }));
    host.accept(snapshot(rows), ordinal);
    guest.accept(snapshot(rows), ordinal);
  }
  assert.equal(host.report().state, "complete");
  assert.equal(host.report().primaries, 10);
  assert.equal(
    guest.report().primaries,
    0,
    "guest does not invent admitted event counts",
  );
  assert.deepEqual(host.report().ordinals, guest.report().ordinals);
  assert.deepEqual(
    host.report().ordinals.map((r) => r.projectiles),
    [1, 1, 1, 1, 2, 1, 1, 1, 1, 2],
  );
  assert.ok(!JSON.stringify(host.report()).includes(owner));
  assert.throws(() => host.accept(snapshot([shot(1)]), 10), /fifth_sequence/);
});
test("ambiguous, saturated, missing, reused, mismatched and stale observations remain inconclusive", () => {
  for (const events of [
    [{ kind: "shot" }, { kind: "shot" }],
    Array.from({ length: 128 }, () => ({ kind: "impact" })),
    [{ kind: "death" }],
  ]) {
    const h = new FifthShotObservation("host");
    h.start(snapshot());
    h.fire(1);
    assert.throws(() => h.events(events));
    assert.equal(h.report().state, "inconclusive");
  }
  for (const s of [
    snapshot(),
    snapshot([shot(1, false, { owner_epoch: 2 })]),
    snapshot([shot(1)], { session: "c".repeat(32) }),
    snapshot([shot(1, false, { damage: NaN })]),
    snapshot([shot(1), shot(2, true)]),
  ]) {
    const h = new FifthShotObservation("host");
    h.start(snapshot());
    h.fire(1);
    h.events([{ kind: "shot" }]);
    assert.throws(() => h.accept(s, 1));
    assert.equal(h.report().state, "inconclusive");
  }
  const g = new FifthShotObservation("guest");
  g.start(snapshot());
  g.accept(snapshot([shot(1)]), 1);
  assert.throws(
    () => g.accept(snapshot([shot(1)]), 2),
    /fifth_shot_not_retained/,
  );
});
test("fifth damage alone cannot pass incorrect speed/direction or epoch ownership", () => {
  for (const overrides of [
    { velocity: [504, 67] },
    { velocity: [364, -3] },
    { damage: 29.04 },
    { remaining_hits: 2 },
  ]) {
    const g = new FifthShotObservation("guest");
    g.start(snapshot());
    for (let i = 1; i <= 4; i++) g.accept(snapshot([shot(i)]), i);
    assert.throws(
      () => g.accept(snapshot([shot(5), shot(6, true, overrides)]), 5),
      /fifth_mismatch/,
    );
  }
});
test("actual client frame holds native immediately after one admitted event while continuing bounded publication", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  const fifth = new FifthShotObservation("host");
  fifth.start(snapshot());
  fifth.fire(1);
  const calls = [],
    context = vm.createContext({
      galaxy: {
        set_paused: (v) => calls.push(["pause", v]),
        step: () => {
          calls.push(["step"]);
          return { events: [{ kind: "shot" }] };
        },
      },
      controller: {
        frameMode: () => "host",
        beforeStep: () => {},
        afterStep: () => calls.push(["publish"]),
      },
      stopped: false,
      scenario: "renko-fifth",
      captureHold: false,
      appliedPause: false,
      fifth,
      heldCapture: 0,
      counters: { captures: 7, frames: 0 },
      controls: { fire: true },
      resetControls: () => {},
      update: () => {},
    });
  vm.runInContext(functionSource(source, "step"), context);
  context.step(1000 / 60);
  assert.deepEqual(calls, [["step"], ["pause", true], ["publish"]]);
  assert.equal(context.captureHold, true);
  assert.equal(context.heldCapture, 7);
  assert.equal(context.controls.fire, false);
  calls.length = 0;
  context.step(16);
  assert.deepEqual(
    calls,
    [["step"], ["publish"]],
    "hold does not repeatedly reset native accumulator",
  );
  assert.equal(
    fifth.primaries,
    1,
    "paused frame does not count another synthetic event from a boundary double",
  );
});
test("report validation is bounded, secret-free and refuses fake completion or inferred guest event counts", () => {
  const idle = new FifthShotObservation("host").report();
  for (const value of [
    { ...idle, state: "complete" },
    { ...idle, reason: "private error" },
    { ...idle, token: "secret" },
    { ...idle, primaries: Infinity },
    { ...idle, state: "inconclusive" },
  ])
    assert.throws(() => validateFifthReport(value, "host"));
  assert.throws(() => validateFifthReport({ ...idle, primaries: 1 }, "guest"));
  const builds = {
    webgpu: `/wasm/webgpu-${"a".repeat(64)}/galaxy.js`,
    webgl2: `/wasm/webgl2-${"b".repeat(64)}/galaxy.js`,
  };
  assert.equal(
    preparedNames({ version: 3, profile_mode: "owned", mode: "wasm", builds })
      .length,
    18,
    "immutable old stages retain their allowlist",
  );
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
  assert.throws(() =>
    validateWasmReport({
      version: 4,
      scenario: "renko-fifth",
      token: "secret",
    }),
  );
});
test("actual fresh-scenario initialize refuses a retained save before import, registration or save", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  let imported = false;
  const context = vm.createContext({
    galaxy: null,
    config: { profile_mode: "owned" },
    role: "host",
    failureCode: null,
    initStage: "none",
    scenario: "default",
    fifth: null,
    initDeadline: 0,
    starterSelection,
    FifthShotObservation,
    fifthReasons: new Set(["fresh_save_required"]),
    SaveStore: class {
      open() {
        return { seed: 7 };
      }
    },
    performance: { now: () => 0 },
    update: () => {},
    setTimeout,
    clearTimeout,
    loadModule: () => {
      imported = true;
      throw new Error("must not import");
    },
  });
  vm.runInContext(functionSource(source, "initStep"), context);
  vm.runInContext(
    functionSource(source, "initialize").replace(
      "import(config.builds[backend])",
      "loadModule()",
    ),
    context,
  );
  await assert.rejects(
    context.initialize("webgpu", "a".repeat(16), "renko-fifth"),
    /fresh_save_required/,
  );
  assert.equal(imported, false);
  assert.equal(context.galaxy, null);
  assert.equal(context.failureCode, "fresh_save_required");
});
test("actual guest acceptance waits for complete commit in the current session, not a matching staged or old frame", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  const fifth = new FifthShotObservation("guest");
  const target = { snapshot: { ...snapshot(), tick: 8, revision: 9 } };
  const context = vm.createContext({
    scenario: "renko-fifth",
    role: "guest",
    fifth,
    latestHeader: target,
    latestCommittedHeader: {
      snapshot: {
        ...snapshot(),
        session: "c".repeat(32),
        tick: 8,
        revision: 9,
      },
    },
    transport: { diagnostics: () => ({ session: { id: session } }) },
    observations: () => ({ fifth_shot: fifth.report() }),
    waitFifth: async (predicate) => {
      assert.equal(
        predicate(),
        false,
        "same tick from retired session is not complete current data",
      );
      context.latestCommittedHeader = {
        snapshot: { ...snapshot(), tick: 7, revision: 9 },
      };
      assert.equal(
        predicate(),
        false,
        "new staged header is not the guest commit",
      );
      context.latestCommittedHeader = target;
      assert.equal(predicate(), true);
    },
  });
  vm.runInContext(functionSource(source, "fifthCommand"), context);
  await context.fifthCommand("fifth-arm", { tick: 8, revision: 9 });
  assert.equal(fifth.report().state, "holding");
});
test("version four capture requires both complete observers for a passing cadence check and admits no extra telemetry", () => {
  const observation = {};
  for (const role of ["host", "guest"]) {
    const value = new FifthShotObservation(role);
    value.start(snapshot());
    let id = 0;
    for (let ordinal = 1; ordinal <= 10; ordinal++) {
      if (role === "host") {
        value.fire(ordinal);
        value.events([{ kind: "shot" }]);
      }
      const rows = [shot(++id)];
      if (ordinal % 5 === 0) rows.push(shot(++id, true));
      value.accept(snapshot(rows), ordinal);
    }
    observation[role] = value.report();
  }
  const client = (role) => ({
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
      ].map((key) => [key, 0]),
    ),
    ...Object.fromEntries(
      [
        "profile_equal",
        "persisted_equal",
        "persistent",
        "progression_false",
        "movement_observed",
      ].map((key) => [key, true]),
    ),
    ...Object.fromEntries(
      ["profile_sha", "baseline_sha", "persisted_sha", "owned_sha"].map(
        (key) => [key, "c".repeat(64)],
      ),
    ),
    owned_export: true,
    owned_view: role === "guest",
    selected_ship: 0,
    owned_slots: 1,
    fifth_shot: observation[role],
  });
  const report = {
    version: 4,
    scenario: "renko-fifth",
    profile_mode: "owned",
    run_id: "d".repeat(16),
    backend: "webgl2",
    session_distinct: null,
    checks: [{ name: "native-fifth-shot", passed: true }],
    clients: [client("host"), client("guest")],
  };
  assert.equal(validateWasmReport(report), report);
  for (const mutate of [
    (r) => {
      r.clients[1].fifth_shot = new FifthShotObservation("guest").report();
    },
    (r) => {
      r.clients[0].fifth_shot.ordinals[4].owner = owner;
    },
    (r) => {
      r.clients[0].fifth_shot.ordinals[4].damage_ratio = 2;
    },
    (r) => {
      r.clients[1].fifth_shot.primaries = 10;
    },
    (r) => {
      r.profile = {};
    },
    (r) => {
      r.scenario = "default";
    },
  ]) {
    const invalid = structuredClone(report);
    mutate(invalid);
    assert.throws(() => validateWasmReport(invalid));
  }
});
test("actual fifth orchestration preserves visibility gating through final save verification", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/wasm-review.js", import.meta.url),
    "utf8",
  );
  let hidden = true,
    passed = 0,
    commands = 0;
  const context = vm.createContext({
    driveActive: false,
    driveVisibilityLost: false,
    state: {},
    requireVisibleClients() {
      if (hidden) throw new Error("page_not_visible");
    },
    guardDriveVisibility() {
      if (hidden) throw new Error("page_not_visible");
    },
    send: async () => {
      commands++;
      return {
        state: { fifth_shot: { state: "complete" } },
        held: { tick: 8, revision: 9 },
      };
    },
    paint: () => {},
    verify: async () => {
      hidden = true;
    },
    check: () => passed++,
  });
  vm.runInContext(functionSource(source, "fifthShot"), context);
  await assert.rejects(context.fifthShot(), /page_not_visible/);
  assert.equal(commands, 0, "foreground preflight precedes real commands");
  hidden = false;
  await assert.rejects(context.fifthShot(), /page_not_visible/);
  assert.equal(
    passed,
    0,
    "hidden final verification cannot publish a passing cadence check",
  );
  assert.equal(context.driveActive, false);
});
