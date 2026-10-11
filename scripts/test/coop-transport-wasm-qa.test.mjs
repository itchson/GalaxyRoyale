import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { once } from "node:events";
import { createServer } from "node:net";
import { request } from "node:http";
import vm from "node:vm";
import ts from "typescript";
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
test("Actual client frame code calls Galaxy.step only, pauses solo/guest, and resets a restored frame", async () => {
  // Declared JS boundaries test the actual harness function, not native gameplay.
  const source = await fs.readFile(
    new URL("../qa/co-op-transport/client.js", import.meta.url),
    "utf8",
  );
  const calls = [];
  const context = vm.createContext({
    stopped: false,
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
  let mode = "guest";
  context.controller.frameMode = () => mode;
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
