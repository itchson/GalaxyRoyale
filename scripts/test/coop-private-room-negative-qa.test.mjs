/** QA source/function tests with declared boundaries; no browser/native/HTTP/stored Profile. */
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import vm from "node:vm";
import ts from "typescript";
import {
  validateWasmReport,
  wasmCheckNames,
} from "../qa/co-op-transport/server.mjs";
const sources = {};
for (const [key, relative] of Object.entries({
  child: "../qa/co-op-transport/client.js",
  parent: "../qa/co-op-transport/wasm-review.js",
  fixtures: "./coop-transport-wasm-qa.test.mjs",
}))
  sources[key] = await fs.readFile(new URL(relative, import.meta.url), "utf8");
function declaration(key, name) {
  const ast = ts.createSourceFile(
    "qa.js",
    sources[key],
    ts.ScriptTarget.ES2022,
    true,
    ts.ScriptKind.JS,
  );
  const node = ast.statements.find(
    (n) => ts.isFunctionDeclaration(n) && n.name?.text === name,
  );
  assert(node, name);
  return node.getText(ast);
}
const plain = (value) => JSON.parse(JSON.stringify(value));
const roomNative = () => ({
  begin: 0,
  join: 0,
  leave: 0,
  end: 0,
  replica_begin: 0,
  replica_end: 0,
});

test("actual negative child commands resolve the one current id, rotate only the host and require an exact boolean lock", async () => {
  const calls = [],
    current = { join_requests: [{ id: "current-request" }] };
  let invite = "private-old-code";
  const context = vm.createContext({
    role: "host",
    galaxy: {},
    controller: {
      state: () => current,
      approve: async (...args) => calls.push(["approve", ...args]),
      invite: () => invite,
      replaceCode: async () => {
        calls.push(["replace"]);
        invite = "private-new-code";
      },
      setJoiningLocked: async (...args) => calls.push(["lock", ...args]),
    },
    observations: () => ({ phase: "hosting" }),
    roomState: () => ({ locked: false }),
  });
  vm.runInContext(declaration("child", "command"), context);
  await context.command("decline-next", {});
  await context.command("approve-next", {});
  assert.deepEqual(calls.slice(0, 2), [
    ["approve", "current-request", false],
    ["approve", "current-request", true],
  ]);
  const replacement = await context.command("replace-code", {});
  assert.deepEqual(plain(replacement), {
    invite: "private-new-code",
    changed: true,
  });
  await context.command("set-joining-locked", { locked: true });
  await context.command("set-joining-locked", { locked: false });
  for (const args of [
    { locked: "true" },
    {},
    { locked: true, extra: "capability" },
  ])
    await assert.rejects(
      context.command("set-joining-locked", args),
      /approval_not_pending/,
    );
  context.role = "guest";
  for (const name of ["decline-next", "replace-code", "set-joining-locked"])
    await assert.rejects(
      context.command(name, { locked: true }),
      /approval_not_pending/,
    );
  context.role = "host";
  current.join_requests = [{ id: "one" }, { id: "two" }];
  await assert.rejects(
    context.command("decline-next", {}),
    /approval_not_pending/,
  );
  assert.deepEqual(calls, [
    ["approve", "current-request", false],
    ["approve", "current-request", true],
    ["replace"],
    ["lock", true],
    ["lock", false],
  ]);
});

test("actual private room marks require a current committed epoch and retain no raw identities in observations", () => {
  const native = roomNative();
  let diagnostic = {
    session: null,
    participant: null,
    peers: [],
    pendingRequests: 0,
    timers: 0,
  };
  const current = {
    phase: "idle",
    reason: null,
    join_requests: [],
    awaiting_approval: false,
    joining_locked: false,
    transport_ready: 0,
    admitted: 0,
  };
  const context = vm.createContext({
    role: "guest",
    roomMark: null,
    roomNative: native,
    roomReasons: new Set(["join_declined"]),
    latestCommittedHeader: null,
    counters: { created: 0, closed: 0, commits: 0 },
    ownedView: null,
    loopDeadline: 180000,
    performance: { now: () => 0 },
    transport: { diagnostics: () => diagnostic },
    controller: { state: () => current, frameMode: () => "solo" },
  });
  for (const name of ["observeRoomOperation", "roomFacts", "roomState"])
    vm.runInContext(declaration("child", name), context);
  context.roomMark = context.roomFacts();
  assert.equal(
    context.roomState().same_room,
    true,
    "solo mark is valid without a snapshot",
  );
  diagnostic = {
    ...diagnostic,
    session: { id: "private-session", seed: 123, room_revision: 1 },
    participant: { id: "private-self" },
    peers: [{ id: "private-host" }],
  };
  context.latestCommittedHeader = {
    snapshot: {
      session: "private-session",
      participants: [{ id: "private-self", epoch: 7 }],
    },
  };
  context.counters.commits = 2;
  context.roomMark = context.roomFacts();
  context.counters.commits++;
  assert.equal(context.roomState().same_epoch, true);
  assert.equal(context.roomState().new_commits, 1);
  const safe = JSON.stringify(context.roomState());
  for (const raw of [
    "private-session",
    "private-self",
    "private-host",
    '"seed"',
    '"epoch"',
    '"invite"',
    '"native"',
  ])
    assert(!safe.includes(raw));
  context.observeRoomOperation("replica_end", false);
  context.observeRoomOperation("unrecognized", true);
  assert.equal(context.roomState().same_native, true);
  context.observeRoomOperation("replica_end", true);
  assert.equal(context.roomState().same_native, false);
  context.counters.created++;
  assert.equal(context.roomState().same_peers, false);
  context.latestCommittedHeader.snapshot.session = "old-session";
  assert.equal(context.roomState().same_epoch, false);
  assert.equal(context.roomState().epoch_present, false);
  current.reason = "raw token/SDP";
  assert.equal(context.roomState().reason, null);
});

test("actual initialization wires private ownership counts to successful real bridge calls only", async () => {
  let observed,
    acceptEnd = false;
  const nativeCounts = roomNative();
  const native = {
    set_paused: () => {},
    resize: () => {},
    set_ui_mode: () => {},
    set_shake: () => {},
    profile: () => ({ seed: 5 }),
    co_op_batch: (request) =>
      JSON.stringify({
        v: 1,
        results: JSON.parse(request).operations.map((op) => ({
          ok: op.op !== "end" || acceptEnd,
          value: null,
        })),
      }),
    co_op_replica: (request) =>
      JSON.stringify({ v: 1, ok: JSON.parse(request).op !== "replica_end" }),
  };
  const context = vm.createContext({
    galaxy: null,
    controller: null,
    initStage: "none",
    failureCode: null,
    appliedPause: null,
    initDeadline: 0,
    config: {
      builds: { webgpu: "/wasm/declared/galaxy.js" },
      service: "declared",
    },
    scenario: "default",
    fifth: null,
    performance: { now: () => 100 },
    setTimeout,
    clearTimeout,
    role: "host",
    canvas: {},
    roomNative: nativeCounts,
    starterSelection: () => ["dora", "pan", "hugo"],
    SaveStore: class {
      persistent = true;
      open() {
        return { seed: 5 };
      }
    },
    loadModule: async () => ({
      default: async () => {},
      Galaxy: { create: async () => native },
    }),
    wasmPorts: {
      createGalaxyCoopPorts: (value) => {
        observed = value;
        return { native: {}, replica: {}, availability: {} };
      },
    },
    localSpec: () => ({}),
    CoopController: class {},
    digest: async () => "a".repeat(64),
    update: () => {},
    schedule: () => {},
  });
  for (const name of ["observeRoomOperation", "initStep"])
    vm.runInContext(declaration("child", name), context);
  vm.runInContext(
    declaration("child", "initialize").replace(
      "import(config.builds[backend])",
      "loadModule(config.builds[backend])",
    ),
    context,
  );
  await context.initialize("webgpu", "b".repeat(16));
  observed.co_op_batch(
    JSON.stringify({
      v: 1,
      operations: [{ op: "begin" }, { op: "join" }, { op: "end" }],
    }),
  );
  observed.co_op_replica(JSON.stringify({ op: "replica_begin" }));
  observed.co_op_replica(JSON.stringify({ op: "replica_end" }));
  assert.deepEqual(nativeCounts, {
    begin: 1,
    join: 1,
    leave: 0,
    end: 0,
    replica_begin: 1,
    replica_end: 0,
  });
  acceptEnd = true;
  observed.co_op_batch(JSON.stringify({ v: 1, operations: [{ op: "end" }] }));
  assert.equal(nativeCounts.end, 1);
});

test("actual pending-only gate refuses RTC/native changes and never sends acceptance", async () => {
  for (const violation of [
    null,
    "same_peers",
    "same_native",
    "ready",
    "admitted",
  ]) {
    const calls = [],
      values = [0, 1].map(() => ({
        same_peers: true,
        same_native: true,
        ready: 0,
        admitted: 0,
      }));
    if (violation)
      values[1][violation] = violation.startsWith("same") ? false : 1;
    const context = vm.createContext({
      performance: { now: () => 0 },
      roomSend: async (...args) => calls.push(args),
      roomUntil: async () => values,
      check: (name, passed) => {
        assert.equal(name, "room-pending-isolated");
        if (!passed) throw Error("isolation_failed");
      },
    });
    vm.runInContext(declaration("parent", "roomPending"), context);
    if (violation)
      await assert.rejects(
        context.roomPending("private-code", 1000),
        /isolation_failed/,
      );
    else await context.roomPending("private-code", 1000);
    assert.deepEqual(
      calls.map((c) => c[1]),
      ["room-mark", "join"],
    );
  }
});

test("actual terminal and approved continuity gates reject mismatched reasons, ownership changes and old commits", async () => {
  const host = {
    mode: "host",
    same_room: true,
    same_native: true,
    same_peers: true,
    ready: 1,
    admitted: 1,
  };
  const guest = {
    phase: "ended",
    mode: "solo",
    reason: "join_declined",
    awaiting: false,
    pending: 0,
    ready: 0,
    admitted: 0,
    created: 2,
    closed: 2,
    requests: 0,
    timers: 0,
    same_peers: true,
    same_native: true,
  };
  const context = vm.createContext({
    roomUntil: async (predicate) => {
      assert(predicate([host, guest]));
      return [host, guest];
    },
    check: (name, passed) => {
      if (!passed) throw Error("ownership_failed");
    },
  });
  vm.runInContext(declaration("parent", "roomTerminal"), context);
  await context.roomTerminal("join_declined", 1000);
  await assert.rejects(
    context.roomTerminal("code_replaced", 1000),
    /room_case_failed/,
  );
  host.same_room = false;
  await assert.rejects(
    context.roomTerminal("join_declined", 1000),
    /ownership_failed/,
  );
  host.same_room = true;
  Object.assign(guest, {
    mode: "guest",
    ready: 1,
    admitted: 1,
    owned_view: true,
    new_commits: 1,
    same_room: true,
    same_epoch: true,
  });
  vm.runInContext(declaration("parent", "roomContinuity"), context);
  await context.roomContinuity(1000);
  guest.same_epoch = false;
  await assert.rejects(context.roomContinuity(1000), /ownership_failed/);
  guest.same_epoch = true;
  guest.new_commits = 0;
  await assert.rejects(context.roomContinuity(1000), /assertion|false/i);
});

test("actual room reply budget is finite and late replies cannot qualify after the action deadline", async () => {
  let now = 0;
  const budgets = [];
  const context = vm.createContext({
    performance: { now: () => now },
    send: async (role, name, args, budget) => {
      budgets.push(budget);
      now = 1000;
      return {};
    },
  });
  vm.runInContext(declaration("parent", "roomSend"), context);
  await assert.rejects(
    context.roomSend("guest", "room-state", {}, 1000),
    /room_deadline/,
  );
  assert.deepEqual(budgets, [1000]);
  await assert.rejects(
    context.roomSend("host", "replace-code", {}, 1000),
    /room_deadline/,
  );
  assert.equal(budgets.length, 1);
});

test("actual replacement round tests approved continuity before guest leave and stale-code join, with one host create", async () => {
  const calls = [];
  const clean = {
    mode: "solo",
    created: 0,
    closed: 0,
    requests: 0,
    timers: 0,
    life_ms: 180000,
  };
  const context = vm.createContext({
    requireVisibleClients: () => {},
    initialized: true,
    scenario: "default",
    config: { profile_mode: "owned" },
    roomCases: new Set(),
    roomAttempted: false,
    performance: { now: () => 10 },
    roomRead: async () => [clean, clean],
    roomMarkBoth: async () => calls.push(["mark"]),
    roomSend: async (role, name, args) => {
      calls.push([role, name, args]);
      return name === "create"
        ? { invite: "private-old" }
        : { invite: "private-new", changed: true };
    },
    roomPending: async (invite) => calls.push(["pending", invite]),
    roomAccept: async () => calls.push(["accept"]),
    roomContinuity: async () => calls.push(["continuity"]),
    roomGuestLeave: async () => calls.push(["guest-left"]),
    roomTerminal: async (reason) => calls.push(["terminal", reason]),
    roomFinish: async () => calls.push(["clean"]),
    check: (name, passed) => assert(passed, name),
  });
  vm.runInContext(declaration("parent", "privateRoomRound"), context);
  await context.privateRoomRound("replace");
  assert.equal(calls.filter((c) => c[1] === "create").length, 1);
  const continuation = calls.findIndex((c) => c[0] === "continuity");
  const leave = calls.findIndex((c) => c[0] === "guest-left");
  const stale = calls.findIndex(
    (c) => c[1] === "join" && c[2].invite === "private-old",
  );
  assert(continuation < leave && leave < stale);
  assert.deepEqual(
    calls.filter((c) => c[0] === "terminal"),
    [
      ["terminal", "stale_invite"],
      ["terminal", "code_replaced"],
    ],
  );
  assert.equal(calls.at(-1)[0], "clean");
  assert(context.roomCases.has("replace"));
});

test("actual failed negative round retains its original failure, runs bounded cleanup and never records completion", async () => {
  let cleaned = 0;
  const clean = {
    mode: "solo",
    created: 0,
    closed: 0,
    requests: 0,
    timers: 0,
    life_ms: 180000,
  };
  const context = vm.createContext({
    requireVisibleClients: () => {},
    initialized: true,
    scenario: "default",
    config: { profile_mode: "owned" },
    roomCases: new Set(),
    roomAttempted: false,
    performance: { now: () => 50 },
    roomRead: async () => [clean, clean],
    roomSend: async () => ({ invite: "private" }),
    roomMarkBoth: async () => {},
    roomPending: async () => {
      throw Error("original_actual_failure");
    },
    roomFinish: async (deadline) => {
      cleaned++;
      assert.equal(deadline, 20050);
      throw Error("cleanup_failure");
    },
  });
  vm.runInContext(declaration("parent", "privateRoomRound"), context);
  await assert.rejects(
    context.privateRoomRound("decline"),
    /original_actual_failure/,
  );
  assert.equal(cleaned, 1);
  assert.equal(context.roomCases.size, 0);
  assert.equal(context.roomAttempted, true);
});

test("actual round refuses insufficient native lifetime or a second completed case before any room mutation", async () => {
  const clean = {
    mode: "solo",
    created: 0,
    closed: 0,
    requests: 0,
    timers: 0,
    life_ms: 139999,
  };
  const context = vm.createContext({
    requireVisibleClients: () => {},
    initialized: true,
    scenario: "default",
    config: { profile_mode: "owned" },
    roomCases: new Set(),
    roomAttempted: false,
    performance: { now: () => 0 },
    roomRead: async () => [clean, clean],
    roomSend: () =>
      assert.fail("precondition failure must not create or mutate a room"),
  });
  vm.runInContext(declaration("parent", "privateRoomRound"), context);
  await assert.rejects(
    context.privateRoomRound("decline"),
    /room_precondition/,
  );
  clean.life_ms = 180000;
  context.roomCases.add("replace");
  await assert.rejects(
    context.privateRoomRound("decline"),
    /room_precondition/,
  );
});

test("actual lock round cancels pending and locked lookup before unlocked acceptance, then retains the approved guest across both policies", async () => {
  const calls = [];
  let locked = false,
    revision = 1,
    readCount = 0;
  const clean = {
    mode: "solo",
    created: 0,
    closed: 0,
    requests: 0,
    timers: 0,
    life_ms: 180000,
  };
  const policy = () => ({ mode: "host", locked, revision, same_room: true });
  const context = vm.createContext({
    requireVisibleClients: () => {},
    initialized: true,
    scenario: "default",
    config: { profile_mode: "owned" },
    roomCases: new Set(),
    roomAttempted: false,
    performance: { now: () => 10 },
    roomRead: async () => (readCount++ === 0 ? [clean, clean] : [policy(), {}]),
    roomMarkBoth: async () => calls.push(["mark"]),
    roomSend: async (role, name, args) => {
      calls.push([name, args]);
      if (name === "set-joining-locked") {
        locked = args.locked;
        revision++;
      }
      return { invite: "private" };
    },
    roomPending: async () => calls.push(["pending"]),
    roomTerminal: async (reason) => {
      calls.push(["terminal", reason]);
      return [policy(), {}];
    },
    roomAccept: async () => calls.push(["accept"]),
    roomContinuity: async () => {
      calls.push(["continuity"]);
      return [policy(), {}];
    },
    roomFinish: async () => calls.push(["clean"]),
    check: (name, passed) => assert(passed, name),
  });
  vm.runInContext(declaration("parent", "privateRoomRound"), context);
  await context.privateRoomRound("lock");
  assert.deepEqual(
    calls.filter((c) => c[0] === "set-joining-locked").map((c) => c[1].locked),
    [true, false, true, false],
  );
  assert.deepEqual(
    calls.filter((c) => c[0] === "terminal"),
    [
      ["terminal", "joining_locked"],
      ["terminal", "joining_locked"],
    ],
  );
  assert.equal(calls.filter((c) => c[0] === "create").length, 1);
  assert.equal(calls.filter((c) => c[0] === "accept").length, 1);
  assert.equal(calls.filter((c) => c[0] === "continuity").length, 2);
  assert(
    calls.findIndex((c) => c[0] === "terminal") <
      calls.findIndex((c) => c[0] === "accept"),
  );
  assert.equal(calls.at(-1)[0], "clean");
});

test("actual report allowlist admits bounded named room checks while excluding private command replies", () => {
  const context = vm.createContext({});
  for (const name of ["client", "report"])
    vm.runInContext(declaration("fixtures", name), context);
  for (const name of [...wasmCheckNames].filter((n) => n.startsWith("room-"))) {
    const report = context.report();
    report.checks = [{ name, passed: true }];
    assert.doesNotThrow(() => validateWasmReport(report));
    report.clients[0].room_state = { invite: "private", session: "private" };
    assert.throws(() => validateWasmReport(report));
  }
  const report = context.report();
  report.checks = [{ name: "room-private-secret", passed: true }];
  assert.throws(() => validateWasmReport(report));
});
