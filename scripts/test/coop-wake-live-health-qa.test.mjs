import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import vm from "node:vm";
import ts from "typescript";
import { execFileSync } from "node:child_process";

// Actual harness functions with declared API replies. No native/WASM/RTC fault
// or execution is simulated as acceptance; these test qualification and cleanup.
const file = "scripts/qa/co-op-transport/wasm-review.js";
const source = await fs.readFile(
  new URL("../../" + file, import.meta.url),
  "utf8",
);
const frozen = execFileSync(
  "git",
  ["show", "1f28378871406383435f7ba103436d58026968d3:" + file],
  { encoding: "utf8" },
);
function parse(text) {
  return ts.createSourceFile(
    "qa.js",
    text,
    ts.ScriptTarget.ES2022,
    true,
    ts.ScriptKind.JS,
  );
}
function bind(context, text, names) {
  const parsed = parse(text);
  for (const name of names) {
    const fn = parsed.statements.find(
      (n) => ts.isFunctionDeclaration(n) && n.name?.text === name,
    );
    if (fn) vm.runInContext(fn.getText(parsed), context);
  }
}
function client(role, kind = "drive") {
  return {
    mode: role,
    phase: role === "host" ? "hosting" : "connected",
    init_stage: "ready",
    failure_code: null,
    created: 1,
    closed: 0,
    requests: 0,
    timers: 1,
    profile_equal: true,
    persisted_equal: true,
    wake: {
      state:
        kind === "stop" ? "expired" : kind === "retreat" ? "retreated" : "live",
      reason: null,
      captured_host: true,
      captured_guest: true,
      captured_points: ["stop", "retreat"].includes(kind) ? 0 : 2,
      native_render_frames: 10,
      expired: kind === "stop",
      retreated: kind === "retreat",
      reduced_motion: true,
      removed_owner: false,
      local_view_ended: false,
    },
  };
}
function failure(value) {
  Object.assign(value, {
    mode: "solo",
    phase: "failed",
    failure_code: "wake_native_frame",
  });
  Object.assign(value.wake, {
    state: "inconclusive",
    reason: "wake_native_frame",
  });
}
function model({
  text = source,
  kind = "drive",
  faultAt = null,
  mutate = failure,
  cleanupReject = false,
} = {}) {
  const state = { host: client("host", kind), guest: client("guest", kind) };
  const historical = { name: "earlier-complete-operation", passed: true },
    checks = [historical],
    calls = [];
  let verifyCalls = 0;
  const button = {};
  const context = vm.createContext({
    state,
    scenario: "configured-ion-wake-r1-e1",
    disposed: false,
    driveActive: false,
    driveVisibilityLost: false,
    requireVisibleClients() {},
    guardDriveVisibility() {},
    $: () => button,
    action: async (_label, fn) => fn(),
    send: async (role, name, _args, timeout) => {
      calls.push({ role, name, timeout });
      if (name === "leave") {
        Object.assign(state.guest, {
          mode: "solo",
          phase: "ended",
          closed: 1,
          requests: 0,
          timers: 0,
        });
        state.guest.wake.state = "ended";
        state.guest.wake.local_view_ended = true;
        state.host.wake.removed_owner = true;
        if (faultAt === "leave") mutate(state.guest);
      }
      if (name === "dispose") {
        if (cleanupReject) throw Error("native_command_timeout");
        Object.assign(state[role], {
          mode: "solo",
          phase: "disposed",
          closed: 1,
          requests: 0,
          timers: 0,
        });
      }
      return state[role];
    },
    until: async (predicate) => {
      assert.equal(predicate(), true);
      await Promise.resolve();
      if (faultAt === "observation") mutate(state.guest);
    },
    verify: async (timeout, record) => {
      verifyCalls++;
      calls.push({ name: "verify", timeout, record });
      await Promise.resolve();
      if (faultAt === "verify") mutate(state.guest);
      return true;
    },
    check: (name, passed) => {
      checks.push({ name, passed });
      if (!passed) throw Error("declared_check_failed");
    },
  });
  bind(context, text, [
    "wakeHealth",
    "cleanupWakeFailure",
    "wakeRound",
    "wakeLeave",
  ]);
  const statement = parse(text).statements.find(
    (n) =>
      ts.isExpressionStatement(n) &&
      ts.isBinaryExpression(n.expression) &&
      n.expression.left.getText() === '$("leave").onclick',
  );
  vm.runInContext(statement.getText(), context);
  return {
    context,
    state,
    checks,
    historical,
    calls,
    button,
    verifyCalls: () => verifyCalls,
  };
}
async function rejected(run, action) {
  await assert.rejects(action(), /wake_context/);
  assert.deepEqual(
    run.checks,
    [run.historical],
    "a new failure cannot erase or append earlier success",
  );
  assert.deepEqual(
    run.calls.filter((c) => c.name === "dispose").map((c) => c.role),
    ["host", "guest"],
  );
  assert.ok(
    run.calls
      .filter((c) => c.name === "dispose")
      .every((c) => c.timeout === 20_000),
  );
  assert.equal(run.context.driveActive, false);
}

test("exact frozen functions reproduce both reviewer qualification gaps, distinct from import setup failures", async () => {
  const phase = model({ text: frozen, faultAt: "verify" });
  await phase.context.wakeRound("drive");
  assert.ok(
    phase.checks.some((c) => c.name === "wake-native-owners" && c.passed),
  );
  assert.equal(phase.state.guest.wake.state, "inconclusive");
  assert.ok(!phase.calls.some((c) => c.name === "dispose"));
  const leave = model({ text: frozen, faultAt: "leave" });
  await leave.button.onclick();
  assert.ok(
    leave.checks.some((c) => c.name === "wake-owner-removal" && c.passed),
  );
  assert.equal(leave.state.guest.failure_code, "wake_native_frame");
});
test("post-final-verify health rejects a delayed native-frame failure and performs bounded cleanup without new saves/owner passes", async () => {
  const run = model({ faultAt: "verify" });
  await rejected(run, () => run.context.wakeRound("drive"));
  assert.equal(
    run.verifyCalls(),
    1,
    "regression reaches the awaited final read while initially healthy",
  );
  assert.equal(run.context.disposed, true);
  assert.deepEqual(
    run.calls.find((c) => c.name === "verify"),
    { name: "verify", timeout: 5_000, record: false },
  );
});
test("post-final-verify context/predicate gate rejects clean retirement or changed captured owner data", async () => {
  for (const mutate of [
    (guest) => Object.assign(guest, { mode: "solo", phase: "ended" }),
    (guest) => (guest.wake.captured_guest = false),
    (guest) => (guest.init_stage = "open_store"),
  ]) {
    const run = model({ faultAt: "verify", mutate });
    await rejected(run, () => run.context.wakeRound("drive"));
    assert.equal(run.verifyCalls(), 1);
  }
});
test("each Wake phase revalidates its own final predicate; old expiry/retreat or reduced-motion facts cannot qualify changed state", async () => {
  for (const [kind, mutate] of [
    ["stop", (guest) => (guest.wake.captured_points = 1)],
    ["retreat", (guest) => (guest.wake.state = "live")],
    ["reduced-motion", (guest) => (guest.wake.reduced_motion = false)],
  ]) {
    const run = model({ kind, faultAt: "verify", mutate });
    await rejected(run, () => run.context.wakeRound(kind));
    assert.equal(run.verifyCalls(), 1);
  }
});
test("failure after observation is rejected before the final save read and cleanup timeout does not become disposal success", async () => {
  const run = model({ faultAt: "observation", cleanupReject: true });
  await rejected(run, () => run.context.wakeRound("drive"));
  assert.equal(run.verifyCalls(), 0);
  assert.equal(run.context.disposed, false);
});
test("healthy final Wake phase snapshots qualify only after current verification and preserve earlier checks", async () => {
  for (const kind of ["drive", "stop", "retreat", "reduced-motion"]) {
    const run = model({ kind });
    await run.context.wakeRound(kind);
    assert.equal(run.verifyCalls(), 1);
    assert.equal(run.checks[0], run.historical);
    assert.deepEqual(
      run.checks.slice(1).map((c) => c.name),
      [
        "saves-unchanged",
        {
          drive: "wake-native-owners",
          stop: "wake-authoritative-expiry",
          retreat: "wake-native-retreat",
          "reduced-motion": "wake-reduced-motion",
        }[kind],
      ],
    );
    assert.ok(!run.calls.some((c) => c.name === "dispose"));
  }
});
test("configured Guest leave rejects failure in the actual reply before teardown facts can become passes", async () => {
  const run = model({ faultAt: "leave" });
  await rejected(run, () => run.button.onclick());
  assert.equal(run.verifyCalls(), 0);
  assert.equal(
    run.state.host.wake.removed_owner,
    true,
    "actual-style prior removal facts remain historical",
  );
  assert.equal(run.state.guest.wake.local_view_ended, true);
});
test("configured Guest leave rejects delayed removal/final-read failure, lost removal predicate or host context", async () => {
  for (const options of [
    { faultAt: "observation" },
    { faultAt: "verify" },
    {
      faultAt: "verify",
      mutate: (guest) => (guest.wake.local_view_ended = false),
    },
  ]) {
    const run = model(options);
    await rejected(run, () => run.button.onclick());
  }
  const host = model({
    faultAt: "verify",
    mutate: () =>
      Object.assign(host.state.host, { mode: "solo", phase: "ended" }),
  });
  await rejected(host, () => host.button.onclick());
});
test("healthy configured Guest leave checks clean restore/removal only after final non-recording read", async () => {
  const run = model();
  await run.button.onclick();
  assert.equal(run.verifyCalls(), 1);
  assert.deepEqual(
    run.checks.slice(1).map((c) => c.name),
    ["saves-unchanged", "guest-leave-restored", "wake-owner-removal"],
  );
  assert.ok(!run.calls.some((c) => c.name === "dispose"));
});
test("ordinary Guest leave retains its existing handler behavior and adds no configured qualification", async () => {
  const old = model({ text: frozen }),
    current = model();
  old.context.scenario = current.context.scenario = "default";
  await old.button.onclick();
  await current.button.onclick();
  assert.deepEqual(current.checks, old.checks);
  assert.deepEqual(current.calls, old.calls);
  assert.ok(!current.checks.some((c) => c.name.startsWith("wake-")));
});
test("actual verify supports a non-recording read so failed phase health cannot append a generic pass first", async () => {
  const state = {},
    checks = [],
    context = vm.createContext({
      state,
      send: async (role) => ({
        role,
        profile_equal: true,
        persisted_equal: true,
        failure_code: "wake_native_frame",
      }),
      check: (name, passed) => checks.push({ name, passed }),
    });
  bind(context, source, ["verify"]);
  assert.equal(await context.verify(5_000, false), true);
  assert.deepEqual(checks, []);
  assert.equal(state.guest.failure_code, "wake_native_frame");
  await context.verify();
  assert.deepEqual(
    checks,
    [{ name: "saves-unchanged", passed: true }],
    "standalone save fact behavior stays unchanged",
  );
});
