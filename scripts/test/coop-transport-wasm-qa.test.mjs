import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { once } from "node:events";
import { createServer } from "node:net";
import { request } from "node:http";
import vm from "node:vm";
import ts from "typescript";
import { CoopController } from "../../apps/game/src/coop/controller.ts";
import {
  prepare,
  preparedNames,
  root,
  sha,
} from "../qa/co-op-transport/prepare.mjs";
import { start, validateWasmReport } from "../qa/co-op-transport/server.mjs";

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
      createGalaxyCoopPorts: () => ({
        native: {},
        replica: {},
        availability: true,
      }),
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
    assert.ok(calls.indexOf("save") > calls.indexOf("pause"));
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
test("WASM allowlist fixes backend paths and cannot request unrelated files", () => {
  const builds = {
    webgpu: `/wasm/webgpu-${"a".repeat(64)}/galaxy.js`,
    webgl2: `/wasm/webgl2-${"b".repeat(64)}/galaxy.js`,
  };
  const names = preparedNames({ version: 2, mode: "wasm", builds });
  assert.equal(names.length, 18);
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

test(
  "Actual supplied normal WASM stage serves two exact QA origins, real binaries and immutable secret-free captures",
  { skip: !process.env.COOP_QA_DIST },
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
      dist: path.resolve(process.env.COOP_QA_DIST),
    });
    assert.equal(manifest.version, 2);
    assert.equal(Object.keys(manifest.files).length, 18);
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
      page.headers.get("content-security-policy").includes(config.guest_origin),
    );
    const binaryURL = config.builds.webgpu.replace(
      "galaxy.js",
      "galaxy_bg.wasm",
    );
    const binary = await fetch(tool.origin + binaryURL);
    assert.equal(binary.headers.get("content-type"), "application/wasm");
    const bytes = Buffer.from(await binary.arrayBuffer());
    assert.deepEqual([...bytes.subarray(0, 8)], [0, 97, 115, 109, 1, 0, 0, 0]);
    assert.equal(sha(bytes), manifest.files[binaryURL.slice(1)]);
    assert.equal((await fetch(tool.origin + "/review.js")).status, 404);
    assert.equal((await fetch(tool.origin + "/wasm/builds.json")).status, 404);
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
      assert.equal(response.headers.get("access-control-allow-origin"), origin);
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
    assert.equal((await post({ ...report(), invite: "private" })).status, 400);
    assert.equal((await post(report())).status, 200);
    const files = await fs.readdir(path.join(stage, "captures"));
    assert.equal(files.length, 1);
    const captured = JSON.parse(
      await fs.readFile(path.join(stage, "captures", files[0]), "utf8"),
    );
    assert.ok(captured.scope.includes("Root-driven"));
    assert.deepEqual(captured.artifacts, manifest.artifacts);
    assert.deepEqual(captured.builds, manifest.builds);
    await tool.stop();
    assert.equal(tool.server.coop.pendingPolls(), 0);
    await fs.appendFile(path.join(stage, binaryURL.slice(1)), Buffer.from([0]));
    await assert.rejects(start({ stage, port, maxMs: 10_000 }), /digest/);
  },
);
