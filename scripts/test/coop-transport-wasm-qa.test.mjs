import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { once } from "node:events";
import { createServer } from "node:net";
import { request } from "node:http";
import vm from "node:vm";
import ts from "typescript";
import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { buildIdentity, treeHash } from "../preview-build.mjs";
import { CoopController } from "../../apps/game/src/coop/controller.ts";
import {
  prepare,
  preparedNames,
  root,
  sha,
  verifyOwnedSource,
  ownedSources,
} from "../qa/co-op-transport/prepare.mjs";
import { start, validateWasmReport } from "../qa/co-op-transport/server.mjs";
import {
  FifthShotObservation,
  starterSelection,
} from "../qa/co-op-transport/fifth-shot.js";

function client(role) {
  return {
    role,
    phase: "connected",
    mode: role,
    init_stage: "ready",
    failure_code: null,
    created: 1,
    closed: 0,
    requests: 0,
    timers: 2,
    frames: 40,
    ticks: 30,
    captures: 2,
    entity_pages: 3,
    body_pages: 8,
    commits: role === "guest" ? 1 : 0,
    body_changes: 0,
    same_count_changes: 0,
    unchanged_bodies: 0,
    projectiles: 3,
    transport_ready: 1,
    admitted: 1,
    profile_equal: true,
    persisted_equal: true,
    persistent: true,
    progression_false: true,
    movement_observed: true,
    profile_sha: "a".repeat(64),
    baseline_sha: "a".repeat(64),
    persisted_sha: "a".repeat(64),
  };
}
function report() {
  return {
    version: 2,
    run_id: "b".repeat(16),
    backend: "webgpu",
    checks: [{ name: "native-complete-commit", passed: true }],
    clients: [client("host"), client("guest")],
    session_distinct: null,
  };
}
function ownedReport() {
  const value = report();
  value.version = 3;
  value.profile_mode = "owned";
  for (const c of value.clients)
    Object.assign(c, {
      owned_export: true,
      owned_view: c.role === "guest" ? true : null,
      selected_ship: 0,
      owned_slots: 1,
      owned_sha: "c".repeat(64),
    });
  return value;
}
function functionSource(source, name) {
  const ast = ts.createSourceFile(
    "qa.js",
    source,
    ts.ScriptTarget.ES2022,
    true,
    ts.ScriptKind.JS,
  );
  const node = ast.statements.find(
    (n) => ts.isFunctionDeclaration(n) && n.name?.text === name,
  );
  assert.ok(node);
  return node.getText(ast);
}
test("Actual initialization pauses fresh Galaxy before guarded license registration and marks each API stage", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  for (const existing of [false, true]) {
    const calls = [],
      stages = [];
    const profile = { seed: 5 };
    let paused = false,
      opens = 0;
    const native = {
      set_paused(value) {
        paused = value;
        calls.push("pause");
      },
      register_droid() {
        assert.equal(
          paused,
          true,
          "actual Galaxy requires paused registration",
        );
        calls.push("register");
      },
      resize: () => calls.push("resize"),
      set_ui_mode: () => {},
      set_shake: () => {},
      profile: () => profile,
      free: () => calls.push("free"),
    };
    const context = vm.createContext({
      galaxy: null,
      controller: null,
      initStage: "none",
      failureCode: null,
      appliedPause: null,
      initDeadline: 0,
      config: {
        builds: { webgpu: "/wasm/actual/galaxy.js" },
        service: "bounded",
      },
      scenario: "default",
      fifth: null,
      FifthShotObservation,
      starterSelection,
      performance: { now: () => 100 },
      setTimeout,
      clearTimeout,
      role: "host",
      canvas: {},
      crypto: { getRandomValues: (a) => a.fill(7) },
      SaveStore: class {
        persistent = true;
        open() {
          return ++opens === 1 && !existing ? null : profile;
        }
        save(value) {
          assert.equal(value, profile);
          calls.push("save");
        }
        flush() {
          calls.push("flush");
        }
      },
      loadModule: async () => ({
        default: async () => calls.push("compile"),
        Galaxy: {
          create: async () => {
            calls.push("create");
            return native;
          },
        },
      }),
      wasmPorts: {
        createGalaxyCoopPorts: () => ({
          native: {},
          replica: {},
          availability: true,
        }),
      },
      localSpec: () => ({}),
      CoopController: class {},
      digest: async () => "a".repeat(64),
      update: () => stages.push(context.initStage),
      schedule: () => calls.push("schedule"),
    });
    vm.runInContext(functionSource(source, "initStep"), context);
    vm.runInContext(
      functionSource(source, "initialize").replace(
        "import(config.builds[backend])",
        "loadModule(config.builds[backend])",
      ),
      context,
    );
    await context.initialize("webgpu", "b".repeat(16));
    assert.equal(context.initStage, "ready");
    assert.equal(context.failureCode, null);
    assert.equal(context.appliedPause, true);
    assert.ok(calls.indexOf("pause") > calls.indexOf("create"));
    assert.equal(calls.includes("save"), !existing);
    assert.equal(calls.includes("flush"), !existing);
    assert.equal(stages.includes("save_baseline"), !existing);
    if (!existing) assert.ok(calls.indexOf("save") > calls.indexOf("pause"));
    assert.equal(calls.includes("register"), !existing);
    if (!existing)
      assert.ok(calls.indexOf("pause") < calls.indexOf("register"));
    assert.ok(
      stages.includes("create_galaxy") && stages.includes("read_baseline"),
    );
  }
});

test("Actual initialization has a bounded total deadline, controlled failures and frees a late native create", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  let callback,
    delay,
    freed = 0,
    cleared = 0,
    resolve;
  const context = vm.createContext({
    initStage: "none",
    failureCode: null,
    initDeadline: 45_000,
    performance: { now: () => 44_800 },
    update: () => {},
    setTimeout: (fn, ms) => {
      callback = fn;
      delay = ms;
      return 1;
    },
    clearTimeout: () => cleared++,
  });
  vm.runInContext(functionSource(source, "initStep"), context);
  const pending = context.initStep(
    "create_galaxy",
    () =>
      new Promise((r) => {
        resolve = r;
      }),
    (native) => native.free(),
  );
  await Promise.resolve();
  assert.equal(delay, 200, "each step shares the overall deadline");
  callback();
  await assert.rejects(pending, /initialization_timeout/);
  resolve({ free: () => freed++ });
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(freed, 1);
  assert.equal(cleared, 1);
  context.failureCode = null;
  await assert.rejects(
    context.initStep("register_license", () => {
      throw new Error("raw private #credential");
    }),
    /initialization_register_license_failed/,
  );
  assert.equal(context.failureCode, "initialization_register_license_failed");
  context.performance.now = () => 45_000;
  let called = false;
  await assert.rejects(
    context.initStep("save_baseline", () => {
      called = true;
    }),
    /initialization_timeout/,
  );
  assert.equal(
    called,
    false,
    "expired startup must not run another native/save action",
  );
});

test("Actual client frame code calls Galaxy.step only, pauses solo/guest, and resets a restored frame", async () => {
  // Declared JS boundaries test the actual harness function, not native gameplay.
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  const calls = [];
  const context = vm.createContext({
    stopped: false,
    scenario: "default",
    captureHold: false,
    fifth: null,
    appliedPause: null,
    controls: { fire: true, thrust: [100, 0] },
    counters: { frames: 0 },
    update: () => {},
    resetControls() {
      context.controls = { fire: false };
      calls.push("reset");
    },
    galaxy: {
      set_paused: (v) => calls.push(["paused", v]),
      step: (ms, input) => calls.push(["step", ms, input]),
    },
    controller: {
      frameMode: () => "guest",
      beforeStep: () => {},
      afterStep: (dt) => calls.push(["after", dt]),
    },
  });
  vm.runInContext(functionSource(source, "step"), context);
  context.step(16);
  assert.equal(calls[0][1], true);
  assert.equal(calls[1][1], 16);
  assert.equal(calls[2][1], 0.016);
  calls.length = 0;
  context.step(16);
  assert.equal(calls.length, 2);
  assert.equal(
    calls[0][0],
    "step",
    "unchanged pause must not reset native accumulator",
  );
  calls.length = 0;
  let mode = "host";
  context.controller.frameMode = () => mode;
  context.step(16);
  assert.equal(calls[0][1], false);
  calls.length = 0;
  context.step(16);
  assert.equal(
    calls.length,
    2,
    "successive host frames preserve the accumulator",
  );
  calls.length = 0;
  context.controller.beforeStep = () => {
    mode = "solo";
  };
  context.step(20);
  assert.equal(calls[0], "reset");
  assert.equal(calls[1][1], true);
  assert.equal(calls[2][1], 0);
  assert.equal(calls[2][2].fire, false);
  assert.equal(calls[3][1], 0);
});
test("Actual rejoin gate requires a new complete native commit, not a cumulative old commit", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/wasm-review.js", import.meta.url),
    "utf8",
  );
  const state = {
    host: { mode: "host", transport_ready: 1, admitted: 1 },
    guest: {
      mode: "guest",
      phase: "connected",
      commits: 4,
      transport_ready: 1,
      admitted: 1,
    },
  };
  const checks = [];
  let advance = false;
  const context = vm.createContext({
    state,
    config: {},
    read: async () => {},
    approvePendingJoin: async () => {},
    firstSession: "old",
    sessionDistinct: null,
    send: async (role, name) =>
      name === "create"
        ? { invite: "private in test memory", session: "new" }
        : {},
    until: async (predicate) => {
      assert.equal(predicate(), false);
      if (advance) {
        state.guest.commits++;
        assert.equal(predicate(), true);
      } else throw new Error("still_incomplete");
    },
    check: (name, passed) => {
      assert.equal(passed, true);
      checks.push(name);
    },
  });
  vm.runInContext(functionSource(source, "connected"), context);
  await assert.rejects(context.connected(true), /still_incomplete/);
  assert.equal(checks.length, 0);
  advance = true;
  await context.connected(true);
  assert.deepEqual(checks, [
    "real-channel-ready",
    "native-admission",
    "native-complete-commit",
    "new-session",
  ]);
});

test("Actual approval QA rejects pre-approval RTC or admission and measures cumulative peer counts", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/wasm-review.js", import.meta.url),
    "utf8",
  );
  for (const violation of [null, "created", "transport_ready", "admitted"]) {
    const calls = [],
      checks = [];
    const context = vm.createContext({
      performance,
      send: async (role, name) => {
        calls.push([role, name]);
        if (name === "approve-next") return {};
        const value = {
          pending: role === "host" ? 1 : 0,
          awaiting: role === "guest",
          created: role === "host" ? 3 : 2,
          transport_ready: 0,
          admitted: 0,
        };
        if (violation && role === "guest") value[violation]++;
        return value;
      },
      check: (name, passed) => {
        checks.push([name, passed]);
        if (!passed) throw Error("declared_check_failed");
      },
    });
    vm.runInContext(functionSource(source, "approvePendingJoin"), context);
    if (violation)
      await assert.rejects(
        context.approvePendingJoin([3, 2]),
        /declared_check_failed/,
      );
    else await context.approvePendingJoin([3, 2]);
    assert.deepEqual(checks, [
      ["host-approval-before-rtc", violation === null],
    ]);
    assert.equal(
      calls.some(([role, name]) => role === "host" && name === "approve-next"),
      violation === null,
    );
  }
});

test("Actual client approval commands use current host request id without exposing credentials", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  const calls = [],
    current = {
      awaiting_approval: false,
      join_requests: [{ id: "actual-request", bearer: "never exposed" }],
      transport_ready: 0,
      admitted: 0,
    };
  const context = vm.createContext({
    role: "host",
    controller: {
      state: () => current,
      approve: async (...args) => calls.push(args),
    },
    galaxy: {},
    counters: { created: 4 },
    observations: () => ({ phase: "hosting" }),
  });
  vm.runInContext(functionSource(source, "command"), context);
  const pending = await context.command("approval-state", {});
  assert.deepEqual(JSON.parse(JSON.stringify(pending)), {
    awaiting: false,
    pending: 1,
    created: 4,
    transport_ready: 0,
    admitted: 0,
  });
  await context.command("approve-next", {});
  assert.deepEqual(calls, [["actual-request", true]]);
  context.role = "guest";
  await assert.rejects(
    context.command("approve-next", {}),
    /approval_not_pending/,
  );
  assert.equal(calls.length, 1);
  context.role = "host";
  current.join_requests.push({ id: "second" });
  await assert.rejects(
    context.command("approve-next", {}),
    /approval_not_pending/,
  );
});

test("Actual approval observation refuses late pending replies and bounds each child timeout", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/wasm-review.js", import.meta.url),
    "utf8",
  );
  let now = 0;
  const budgets = [];
  const context = vm.createContext({
    performance: { now: () => now },
    send: async (role, name, args, timeout) => {
      assert.equal(
        name,
        "approval-state",
        "a late observation must never accept",
      );
      budgets.push(timeout);
      if (role === "guest") now = 15_000;
      return {
        pending: 1,
        awaiting: true,
        created: 0,
        transport_ready: 0,
        admitted: 0,
      };
    },
    check: () => assert.fail("expired observations cannot establish isolation"),
  });
  vm.runInContext(functionSource(source, "approvePendingJoin"), context);
  await assert.rejects(context.approvePendingJoin([0, 0]), /approval_timeout/);
  assert.deepEqual(budgets, [15_000, 15_000]);
});

test("Actual child message route preserves controlled approval failure and sanitizes arbitrary errors", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  const handlerSource = source.slice(
    source.indexOf('addEventListener("message"'),
    source.indexOf('addEventListener("pagehide"'),
  );
  for (const reason of ["approval_not_pending", "private token and SDP"]) {
    const replies = [];
    let handler;
    const parent = { postMessage: (value) => replies.push(value) };
    const context = vm.createContext({
      parent,
      config: { page_origin: "http://localhost:4199" },
      commandPending: false,
      command: async () => {
        throw Error(reason);
      },
      failureCode: null,
      fifth: null,
      fifthReasons: new Set(),
      wakeApi: null,
      scenario: "default",
      update: () => {},
      phase: "hosting",
      addEventListener: (type, callback) => {
        assert.equal(type, "message");
        handler = callback;
      },
    });
    vm.runInContext(handlerSource, context);
    await handler({
      source: parent,
      origin: context.config.page_origin,
      data: { type: "native-qa-command", id: 1, name: "approve-next" },
    });
    assert.equal(replies[0].ok, false);
    assert.equal(
      replies[0].error,
      reason === "approval_not_pending" ? reason : "native_action_failed",
    );
    assert.equal(context.commandPending, false);
    const value = report();
    value.clients[0].failure_code = replies[0].error;
    assert.doesNotThrow(() => validateWasmReport(value));
  }
});

test("Actual New session orchestration uses real controller idempotent leave after prior host end", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/wasm-review.js", import.meta.url),
    "utf8",
  );
  const calls = [];
  // Actual controller lifecycle code; declared idle transport boundary, no native/Game/RTC execution.
  const controllers = Object.fromEntries(
    ["host", "guest"].map((role) => [
      role,
      new CoopController({
        service: "http://127.0.0.1:4176",
        localSpec: () => ({ local_seed: 1, identity: null }),
        native: {
          co_op_batch() {
            assert.fail(
              "an already-ended leave must not invent a native operation",
            );
          },
        },
        transportFactory(options) {
          return {
            diagnostics: () => ({
              phase: "ended",
              reason: null,
              session: null,
              participant: null,
              peers: [],
            }),
            async leave() {
              calls.push(role + "-left");
              options.onChange?.();
            },
          };
        },
      }),
    ]),
  );
  await controllers.host.leave();
  await controllers.guest.leave();
  await assert.rejects(
    controllers.host.end(),
    (error) => error.code === "host_only",
  );
  calls.length = 0;
  const context = vm.createContext({
    send: async (role, action) => {
      assert.equal(
        action,
        "leave",
        "a second end reproduces the observed host_only failure",
      );
      await controllers[role][action]();
    },
    connected: async (rejoin) => {
      assert.equal(rejoin, true);
      for (const controller of Object.values(controllers)) {
        assert.equal(controller.state().phase, "ended");
        assert.equal(controller.frameMode(), "solo");
      }
      calls.push("fresh-connect");
    },
  });
  vm.runInContext(functionSource(source, "restartSession"), context);
  await context.restartSession();
  assert.deepEqual(calls, ["guest-left", "host-left", "fresh-connect"]);
});

test("Actual Drive preflight rejects hidden, clipped, zero and nonfinite client bounds before any native command", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/wasm-review.js", import.meta.url),
    "utf8",
  );
  const visible = {
    left: 20,
    top: 224,
    right: 1260,
    bottom: 556,
    width: 1240,
    height: 332,
  };
  const context = vm.createContext({
    document: { visibilityState: "visible" },
    window: { innerWidth: 1280, innerHeight: 900 },
    children: Object.fromEntries(
      ["host", "guest"].map((role) => [
        role,
        { frame: { getBoundingClientRect: () => visible } },
      ]),
    ),
    send: () => assert.fail("invalid visibility must not send a native drive"),
  });
  for (const name of ["requireVisibleClients", "drive"])
    vm.runInContext(functionSource(source, name), context);
  assert.doesNotThrow(() => context.requireVisibleClients());
  context.document.visibilityState = "hidden";
  await assert.rejects(context.drive(), /page_not_visible/);
  context.document.visibilityState = "visible";
  for (const rect of [
    { ...visible, top: 495, bottom: 827 }, // actual short-viewport failure shape
    { ...visible, left: -1 },
    { ...visible, right: 1281 },
    { ...visible, top: -1 },
    { ...visible, width: 0 },
    { ...visible, height: 0 },
    { ...visible, bottom: Infinity },
    { ...visible, left: NaN },
  ]) {
    context.children.guest.frame.getBoundingClientRect = () => rect;
    context.window.innerHeight = rect.top === 495 ? 480 : 900;
    await assert.rejects(context.drive(), /clients_not_visible/);
  }
  context.children.guest.frame.getBoundingClientRect = () => visible;
  context.window.innerHeight = 900;
  context.window.innerWidth = NaN;
  await assert.rejects(context.drive(), /clients_not_visible/);
});

test("Actual Drive latches mid-command visibility loss and never labels interrupted observations a native pass", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/wasm-review.js", import.meta.url),
    "utf8",
  );
  const status = { textContent: "" },
    calls = [],
    pending = [];
  const context = vm.createContext({
    document: { visibilityState: "visible" },
    window: { innerWidth: 1280, innerHeight: 900 },
    children: Object.fromEntries(
      ["host", "guest"].map((role) => [
        role,
        {
          frame: {
            getBoundingClientRect: () => ({
              left: 20,
              top: 224,
              right: 620,
              bottom: 556,
              width: 600,
              height: 332,
            }),
          },
        },
      ]),
    ),
    driveActive: false,
    driveVisibilityLost: false,
    $: () => status,
    send: (role) => {
      calls.push(role);
      return new Promise((resolve) => pending.push(resolve));
    },
    until: () =>
      assert.fail(
        "interrupted native commands cannot receive a passing observation",
      ),
    check: () =>
      assert.fail("visibility is not native movement/fire acceptance"),
    verify: () => {},
  });
  for (const name of [
    "requireVisibleClients",
    "guardDriveVisibility",
    "noteDriveVisibility",
    "drive",
  ])
    vm.runInContext(functionSource(source, name), context);
  const drive = context.drive();
  assert.deepEqual(calls, ["host", "guest"]);
  assert.equal(context.driveActive, true);
  context.document.visibilityState = "hidden";
  context.noteDriveVisibility();
  assert.equal(context.driveVisibilityLost, true);
  assert.match(status.textContent, /will not pass/);
  context.document.visibilityState = "visible";
  for (const resolve of pending) resolve({});
  await assert.rejects(drive, /clients_not_visible/);
  assert.equal(context.driveActive, false);
  const previous = status.textContent;
  context.noteDriveVisibility();
  assert.equal(
    status.textContent,
    previous,
    "idle visibility changes do not change lifecycle state",
  );
  context.state = {
    host: { movement_observed: true, projectiles: 1 },
    guest: { movement_observed: true, projectiles: 1 },
  };
  context.send = async () => ({});
  context.until = async (predicate) => assert.equal(predicate(), true);
  context.verify = async () => {
    context.document.visibilityState = "hidden";
    context.noteDriveVisibility();
  };
  await assert.rejects(context.drive(), /page_not_visible/);
  assert.equal(
    context.driveActive,
    false,
    "visibility lost while reading saves still cannot pass movement",
  );
  context.document.visibilityState = "visible";
  const completed = [];
  context.verify = async () => completed.push("saves-read");
  context.check = (name, passed) => {
    assert.equal(passed, true);
    completed.push(name);
  };
  await context.drive();
  assert.deepEqual(
    completed,
    ["saves-read", "movement-fire-observed"],
    "declared observation boundary passes only after all guards",
  );
});

test("WASM report accepts finite observations, refuses capabilities/profile payloads and does not infer a native commit", () => {
  const base = report();
  assert.deepEqual(validateWasmReport(base), base);
  const incomplete = report();
  incomplete.clients[1].phase = "loading";
  incomplete.clients[1].commits = 0;
  incomplete.checks = [];
  assert.equal(validateWasmReport(incomplete).clients[1].commits, 0);
  for (const mutate of [
    (r) => (r.invite = "#SECRET"),
    (r) => (r.clients[0].token = "secret"),
    (r) => (r.clients[1].profile = { seed: 1 }),
    (r) => (r.clients[1].init_stage = "token from arbitrary error"),
    (r) => (r.clients[1].failure_code = "raw native exception / #SECRET"),
    (r) => (r.clients[0].profile_sha = "plaintext identity"),
    (r) => (r.clients[1].commits = -1),
    (r) => (r.clients[0].closed = 2),
    (r) => (r.clients[0].admitted = 8),
    (r) => r.checks.push({ name: "SDP", passed: true }),
    (r) => (r.checks[0].secret = "capability"),
    (r) => r.clients.reverse(),
    (r) => (r.clients[0].phase = "fictional_ready"),
  ]) {
    const invalid = structuredClone(base);
    mutate(invalid);
    assert.throws(() => validateWasmReport(invalid));
  }
});
test("Owned reports preserve the factory format and admit only finite, secret-free native metadata", () => {
  const good = ownedReport();
  assert.deepEqual(validateWasmReport(good), good);
  for (const mutate of [
    (r) => (r.profile_mode = "factory"),
    (r) => (r.clients[1].selected_ship = 10),
    (r) => (r.clients[1].owned_slots = 4),
    (r) => (r.clients[0].owned_sha = "raw loadout"),
    (r) => (r.clients[1].owned_view = "ready"),
    (r) => (r.clients[0].owned_loadout = {}),
    (r) => (r.clients[0].owned_export = "true"),
    (r) => delete r.clients[1].owned_export,
  ]) {
    const bad = structuredClone(good);
    mutate(bad);
    assert.throws(() => validateWasmReport(bad));
  }
  assert.deepEqual(validateWasmReport(report()), report());
});

test("Actual localSpec reads the native export only in explicit owned mode and never forwards Profile", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  const loadout = {
    version: 1,
    selected_ship: 0,
    slots: [null, { index: 1, rank: 1 }, null],
  };
  let exports = 0;
  const context = vm.createContext({
    config: { profile_mode: "factory" },
    ownedLoadout: loadout,
    canonical: (v) => v,
    galaxy: {
      profile: () => ({
        seed: 41,
        identity: null,
        currency: [99, 99, 99],
        secret: "not transport data",
      }),
    },
    wasmPorts: {
      readOwnedLoadout: () => {
        exports++;
        return structuredClone(loadout);
      },
    },
  });
  vm.runInContext(functionSource(source, "localSpec"), context);
  assert.deepEqual(JSON.parse(JSON.stringify(context.localSpec())), {
    local_seed: 41,
    identity: null,
  });
  assert.equal(exports, 0);
  context.config.profile_mode = "owned";
  assert.deepEqual(JSON.parse(JSON.stringify(context.localSpec())), {
    local_seed: 41,
    identity: null,
    owned_loadout: loadout,
  });
  assert.equal(exports, 1);
  context.wasmPorts.readOwnedLoadout = () => ({ ...loadout, selected_ship: 1 });
  assert.throws(() => context.localSpec(), /native_owned_changed/);
  context.wasmPorts = {};
  assert.throws(() => context.localSpec(), /native_owned_unavailable/);
});

test("Actual commit observations reset session IDs and cannot mistake a staged owned header for a complete native commit", async () => {
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  const loadout = {
    selected_ship: 0,
    slots: [null, { index: 1, rank: 1 }, null],
  };
  const context = vm.createContext({
    config: { profile_mode: "owned" },
    role: "guest",
    transport: undefined,
    observedSession: null,
    lastRows: new Map(),
    stagedRows: new Map(),
    firstPosition: null,
    ownedLoadout: loadout,
    ownedView: null,
    stagedOwnedView: null,
    movementObserved: null,
    canonical: (v) => v,
    counters: {
      ticks: 0,
      projectiles: 0,
      body_changes: 0,
      same_count_changes: 0,
      unchanged_bodies: 0,
    },
  });
  for (const name of ["observeHeader", "observeRows", "observeCommit"])
    vm.runInContext(functionSource(source, name), context);
  const header = (session) => ({
    snapshot: {
      session,
      tick: 1,
      projectiles: [],
      progression: false,
      participants: [
        { slot: 1, position: [0, 0], selected_ship: 0, owned_loadout: loadout },
      ],
    },
  });
  context.observeHeader(header("session-a"));
  assert.equal(
    context.ownedView,
    null,
    "header staging is not native commit acceptance",
  );
  context.observeRows([{ id: "same", occupied: 3, body_signature: "old" }]);
  context.observeCommit();
  assert.equal(context.ownedView, true);
  context.transport = {
    diagnostics: () => ({ participant: { id: "authenticated-guest" } }),
  };
  const wrongPeer = header("session-a");
  wrongPeer.snapshot.participants[0].id = "another-guest";
  context.observeHeader(wrongPeer);
  context.observeCommit();
  assert.equal(
    context.ownedView,
    false,
    "another participant's matching declaration is not the authenticated guest",
  );
  context.transport = undefined;
  context.observeHeader(header("session-a"));
  context.observeRows([{ id: "same", occupied: 3, body_signature: "old" }]);
  context.observeCommit();
  context.observeRows([{ id: "same", occupied: 3, body_signature: "changed" }]);
  context.observeCommit();
  assert.equal(
    context.counters.same_count_changes,
    1,
    "a same-session content change is counted",
  );
  context.observeHeader(header("session-b"));
  assert.equal(context.ownedView, null);
  context.observeRows([
    { id: "same", occupied: 3, body_signature: "new-session" },
  ]);
  context.observeCommit();
  assert.equal(
    context.counters.same_count_changes,
    1,
    "reused IDs in another session add no false change",
  );
});

test("Owned preparation guards exact Git adapter content, whole normal source/package stamp and independent source options", async (t) => {
  // Small declared filesystem/Git fixture exercises provenance guards, not a native build.
  const fixture = await fs.mkdtemp(
    path.join(root, "work", "coop-owned-provenance-test-"),
  );
  t.after(async () => {
    assert.ok(fixture.startsWith(path.join(root, "work") + path.sep));
    await fs.rm(fixture, { recursive: true });
  });
  const run = (...args) =>
    execFileSync("git", args, {
      cwd: fixture,
      stdio: ["ignore", "pipe", "pipe"],
    })
      .toString()
      .trim();
  for (const dir of ["crates", "assets", "dist", "work"])
    await fs.mkdir(path.join(fixture, dir), { recursive: true });
  for (const name of [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "package.json",
    "pnpm-lock.yaml",
    "tsconfig.json",
    "scripts/build.mjs",
    "scripts/build-wasm.mjs",
    "scripts/pwa.mjs",
    "scripts/wasm-artifacts.mjs",
    "scripts/preview-build.mjs",
    "scripts/game-runtime-lease.mjs",
    ...ownedSources,
  ]) {
    await fs.mkdir(path.dirname(path.join(fixture, name)), { recursive: true });
    await fs.writeFile(path.join(fixture, name), "// declared fixture\n");
  }
  await fs.writeFile(path.join(fixture, "dist/index.html"), "declared package");
  run("init", "-b", "codex/testing");
  run("add", ".");
  run(
    "-c",
    "user.name=Codex",
    "-c",
    "user.email=codex@openai.com",
    "commit",
    "-m",
    "Declared provenance fixture",
  );
  const identity = await buildIdentity(fixture);
  let stamp = {
    schemaVersion: 1,
    ...identity,
    packageHash: await treeHash(fixture, ["dist"]),
    builtAt: "2026-10-12T00:00:00.000Z",
  };
  const saveStamp = () =>
    fs.writeFile(
      path.join(fixture, "work/preview-build.json"),
      JSON.stringify(stamp),
    );
  await saveStamp();
  const options = {
    sourceRoot: fixture,
    sourceCommit: identity.head,
    dist: path.join(fixture, "dist"),
  };
  const accepted = await verifyOwnedSource(options);
  assert.equal(accepted.blobs.size, 7);
  stamp.extra = "must not enter captured provenance";
  await saveStamp();
  await assert.rejects(verifyOwnedSource(options), /record/);
  delete stamp.extra;
  await saveStamp();
  await assert.rejects(
    verifyOwnedSource({ ...options, sourceCommit: "a".repeat(40) }),
    /record/,
  );
  await assert.rejects(
    verifyOwnedSource({ ...options, dist: path.join(root, "dist") }),
    /absolute normal dist/,
  );
  await assert.rejects(
    prepare(path.join(fixture, "unused"), { sourceRoot: fixture }),
    /explicit owned-loadout/,
  );
  await fs.appendFile(path.join(fixture, ownedSources[0]), "// dirty\n");
  await assert.rejects(verifyOwnedSource(options), /changed since/);
  // Even a matching live hash cannot bless adapter bytes absent from the requested Git revision.
  stamp = { ...stamp, sourceHash: (await buildIdentity(fixture)).sourceHash };
  await saveStamp();
  await assert.rejects(verifyOwnedSource(options), /exact compiled source/);
  await fs.writeFile(
    path.join(fixture, ownedSources[0]),
    accepted.blobs.get(ownedSources[0]),
  );
  stamp = { ...stamp, sourceHash: identity.sourceHash };
  await saveStamp();
  await fs.appendFile(path.join(fixture, "dist/index.html"), "changed");
  await assert.rejects(verifyOwnedSource(options), /changed since/);
});
test("WASM allowlist fixes backend paths and cannot request unrelated files", () => {
  const builds = {
    webgpu: `/wasm/webgpu-${"a".repeat(64)}/galaxy.js`,
    webgl2: `/wasm/webgl2-${"b".repeat(64)}/galaxy.js`,
  };
  const names = preparedNames({
    version: 2,
    mode: "wasm",
    builds,
    files: { "modules/room-code.js": "c".repeat(64) },
  });
  assert.equal(names.length, 19);
  assert.ok(names.includes("modules/room-code.js"));
  assert.ok(names.includes("modules/storage.js"));
  assert.ok(!names.some((n) => n.includes("main.js")));
  for (const buildsBad of [
    { ...builds, evil: "https://secret" },
    { ...builds, webgpu: "/../../secret" },
    { ...builds, webgl2: builds.webgpu },
  ])
    assert.throws(() =>
      preparedNames({ version: 2, mode: "wasm", builds: buildsBad }),
    );
});

for (const owned of [false, true])
  test(
    `Actual supplied ${owned ? "owned source-pinned" : "factory"} normal WASM stage serves two exact QA origins, real binaries and immutable secret-free captures`,
    {
      skip: owned
        ? !(
            process.env.COOP_QA_SOURCE_ROOT &&
            process.env.COOP_QA_SOURCE_COMMIT &&
            process.env.COOP_QA_OWNED_DIST
          )
        : !process.env.COOP_QA_DIST,
    },
    async (t) => {
      // These HTTP assertions use actual normal build bytes. They do not run WASM/RTC in Node.
      const stage = await fs.mkdtemp(
        path.join(root, "work", "coop-wasm-qa-test-"),
      );
      let tool;
      t.after(async () => {
        await tool?.stop();
        assert.ok(!tool?.server.listening);
        assert.ok(stage.startsWith(path.join(root, "work") + path.sep));
        await fs.rm(stage, { recursive: true });
      });
      const manifest = await prepare(stage, {
        dist: path.resolve(
          owned ? process.env.COOP_QA_OWNED_DIST : process.env.COOP_QA_DIST,
        ),
        ...(owned
          ? {
              ownedLoadout: true,
              sourceRoot: path.resolve(process.env.COOP_QA_SOURCE_ROOT),
              sourceCommit: process.env.COOP_QA_SOURCE_COMMIT,
            }
          : {}),
      });
      assert.equal(manifest.version, owned ? 3 : 2);
      assert.equal(Object.keys(manifest.files).length, 20);
      assert.equal(manifest.artifacts.length, 8);
      for (const name of ["controller", "wasm-ports"]) {
        const js = await fs.readFile(
          path.join(stage, `modules/${name}.js`),
          "utf8",
        );
        assert.ok(!js.includes('"./wire.ts"'));
        assert.ok(!js.includes('"./transport.ts"'));
      }
      await assert.rejects(
        prepare(stage, { dist: path.resolve(process.env.COOP_QA_DIST) }),
        /fresh stage/,
      );
      const reservation = createServer();
      reservation.listen(0, "127.0.0.1");
      await once(reservation, "listening");
      const port = reservation.address().port;
      await new Promise((r) => reservation.close(r));
      tool = await start({ stage, port, maxMs: 10_000 });
      const config = await (await fetch(tool.origin + "/qa/config")).json();
      assert.equal(config.mode, "wasm");
      assert.equal(config.profile_mode, owned ? "owned" : "factory");
      if (owned) {
        assert.equal(config.source_commit, process.env.COOP_QA_SOURCE_COMMIT);
        assert.equal(config.build_stamp.head, config.source_commit);
        assert.equal(Object.keys(manifest.git_inputs).length, 7);
        const ports = await import(
          pathToFileURL(path.join(stage, "modules/wasm-ports.js")).href
        );
        // Execute the actual staged native-export adapter against a declared response boundary.
        // Native execution is reserved for the real browser; this tests request/response contract.
        const value = {
          version: 1,
          selected_ship: 0,
          owned_ships: 1,
          patches: 0,
          royal_victory: false,
          ship_xp: 0,
          hardware: [0, 0, 0, 0, 0, 0],
          slots: [null, null, { index: 2, rank: 1 }],
          trained_starter: false,
          relic_count: 0,
        };
        assert.deepEqual(
          ports.readOwnedLoadout({
            co_op_batch(encoded) {
              assert.deepEqual(JSON.parse(encoded), {
                v: 1,
                operations: [{ op: "export_loadout" }],
              });
              return JSON.stringify({ v: 1, results: [{ ok: true, value }] });
            },
          }),
          value,
        );
      }
      assert.equal(config.modules, 6);
      assert.equal(config.guest_origin, `http://guest.localhost:${port}`);
      assert.equal(config.client_role, "host");
      // Undici fetch replaces a supplied Host header; use a real HTTP request
      // to exercise the guest virtual host without depending on Node DNS aliases.
      const guestConfig = await new Promise((resolve, reject) => {
        const req = request(
          tool.service + "/qa/config",
          { headers: { Host: `guest.localhost:${port}` } },
          (res) => {
            const chunks = [];
            res.on("data", (c) => chunks.push(c));
            res.on("end", () => resolve(JSON.parse(Buffer.concat(chunks))));
            res.on("error", reject);
          },
        );
        req.on("error", reject);
        req.end();
      });
      assert.equal(guestConfig.client_role, "guest");
      const page = await fetch(tool.origin + "/");
      assert.match(await page.text(), /two.*native/i);
      assert.ok(
        page.headers
          .get("content-security-policy")
          .includes("'wasm-unsafe-eval'"),
      );
      assert.ok(
        page.headers
          .get("content-security-policy")
          .includes(config.guest_origin),
      );
      const binaryURL = config.builds.webgpu.replace(
        "galaxy.js",
        "galaxy_bg.wasm",
      );
      const binary = await fetch(tool.origin + binaryURL);
      assert.equal(binary.headers.get("content-type"), "application/wasm");
      const bytes = Buffer.from(await binary.arrayBuffer());
      assert.deepEqual(
        [...bytes.subarray(0, 8)],
        [0, 97, 115, 109, 1, 0, 0, 0],
      );
      assert.equal(sha(bytes), manifest.files[binaryURL.slice(1)]);
      assert.equal((await fetch(tool.origin + "/review.js")).status, 404);
      assert.equal(
        (await fetch(tool.origin + "/wasm/builds.json")).status,
        404,
      );
      assert.equal(
        (await fetch(tool.origin + binaryURL + "?token=forbidden")).status,
        404,
      );
      for (const origin of [tool.origin, config.guest_origin]) {
        const response = await fetch(tool.service + "/v1/sessions", {
          method: "POST",
          headers: { Origin: origin, "Content-Type": "application/json" },
          body: JSON.stringify({ seed: 123, protocol: "galaxy-coop-1" }),
        });
        assert.equal(response.status, 201);
        const grant = await response.json();
        assert.ok(grant.participant.token);
        assert.equal(
          response.headers.get("access-control-allow-origin"),
          origin,
        );
      }
      assert.equal(
        (
          await fetch(tool.service + "/v1/sessions", {
            method: "POST",
            headers: {
              Origin: "http://untrusted.localhost",
              "Content-Type": "application/json",
            },
            body: "{}",
          })
        ).status,
        403,
      );
      const post = (value) =>
        fetch(tool.origin + "/qa/report", {
          method: "POST",
          headers: { Origin: tool.origin, "Content-Type": "application/json" },
          body: JSON.stringify(value),
        });
      const actualReport = owned ? ownedReport() : report();
      assert.equal(
        (await post({ ...actualReport, invite: "private" })).status,
        400,
      );
      assert.equal(
        (await post(owned ? report() : ownedReport())).status,
        400,
        "a report cannot silently change stage modes",
      );
      assert.equal((await post(actualReport)).status, 200);
      const files = await fs.readdir(path.join(stage, "captures"));
      assert.equal(files.length, 1);
      const captured = JSON.parse(
        await fs.readFile(path.join(stage, "captures", files[0]), "utf8"),
      );
      assert.ok(captured.scope.includes("Root-driven"));
      assert.deepEqual(captured.artifacts, manifest.artifacts);
      assert.deepEqual(captured.builds, manifest.builds);
      if (owned) {
        assert.deepEqual(captured.git_inputs, manifest.git_inputs);
        assert.deepEqual(captured.build_stamp, manifest.build_stamp);
      }
      await tool.stop();
      assert.equal(tool.server.coop.pendingPolls(), 0);
      await fs.appendFile(
        path.join(stage, binaryURL.slice(1)),
        Buffer.from([0]),
      );
      await assert.rejects(start({ stage, port, maxMs: 10_000 }), /digest/);
    },
  );
