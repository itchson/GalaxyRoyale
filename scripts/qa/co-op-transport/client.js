import { CoopController } from "/modules/controller.js";
import { CoopTransport } from "/modules/transport.js";
import * as wasmPorts from "/modules/wasm-ports.js";
import { SaveStore, normalizeProfile } from "/modules/storage.js";
import {
  FifthShotObservation,
  fifthReasons,
  starterSelection,
} from "/fifth-shot.js";

const config = await (await fetch("/qa/config", { cache: "no-store" })).json();
const role = config.client_role;
const status = document.getElementById("status");
const canvas = document.getElementById("canvas");
let galaxy, controller, transport, store, backend, run, baseline, timer;
let initStage = "none",
  failureCode = null,
  initDeadline = 0,
  appliedPause = null;
let phase = "uninitialized",
  mode = "solo",
  controls = { fire: false },
  pumping = false,
  stopped = false;
let lastTime = performance.now(),
  loopDeadline = 0,
  firstPosition = null;
let scenario = "default",
  fifth = null,
  captureHold = false,
  heldCapture = 0,
  latestHeader = null,
  latestCommittedHeader = null;
let lastRows = new Map(),
  stagedRows = new Map(),
  observedSession = null,
  ownedLoadout = null,
  ownedSha = null,
  ownedExport = null,
  ownedView = null,
  stagedOwnedView = null;
const counters = {
  created: 0,
  closed: 0,
  frames: 0,
  ticks: 0,
  captures: 0,
  entity_pages: 0,
  body_pages: 0,
  commits: 0,
  body_changes: 0,
  same_count_changes: 0,
  unchanged_bodies: 0,
  projectiles: 0,
};
let profileSha = null,
  persistedSha = null,
  profileEqual = null,
  persistedEqual = null,
  persistent = null,
  progressionFalse = null,
  movementObserved = null;
// QA-only private marks. Neither capabilities nor identity/epoch snapshots are reported.
let roomMark = null;
const roomNative = {
  begin: 0,
  join: 0,
  leave: 0,
  end: 0,
  replica_begin: 0,
  replica_end: 0,
};
const roomReasons = new Set([
  "join_declined",
  "code_replaced",
  "joining_locked",
  "stale_invite",
  "approval_timeout",
  "local_left",
  "host_end",
  "host_ended",
  "guest_left",
  "host_left",
]);
function observeRoomOperation(op, ok) {
  if (ok === true && Object.hasOwn(roomNative, op)) roomNative[op]++;
}
function roomFacts() {
  const d = transport?.diagnostics(),
    session = d?.session;
  const snapshot = latestCommittedHeader?.snapshot;
  const own =
    snapshot && session && snapshot.session === session.id
      ? snapshot.participants.find((p) => p.id === d.participant?.id)
      : null;
  return {
    session: session?.id ?? null,
    seed: session?.seed ?? null,
    self: d?.participant?.id ?? null,
    peer: role === "guest" ? (d?.peers[0]?.id ?? null) : null,
    epoch: own?.epoch ?? null,
    revision: session?.room_revision ?? null,
    created: counters.created,
    closed: counters.closed,
    commits: counters.commits,
    native: { ...roomNative },
  };
}
function roomState() {
  const current = controller.state(),
    d = transport.diagnostics(),
    facts = roomFacts();
  return {
    phase: current.phase,
    mode: controller.frameMode(),
    life_ms: Math.max(0, loopDeadline - performance.now()),
    reason: roomReasons.has(current.reason) ? current.reason : null,
    pending: current.join_requests.length,
    awaiting: current.awaiting_approval,
    locked: current.joining_locked,
    revision: facts.revision,
    created: counters.created,
    closed: counters.closed,
    requests: d.pendingRequests,
    timers: d.timers,
    ready: current.transport_ready,
    admitted: current.admitted,
    same_room:
      roomMark !== null &&
      ["session", "seed", "self", "peer"].every(
        (key) => facts[key] === roomMark[key],
      ),
    same_epoch:
      roomMark?.epoch !== null &&
      roomMark?.epoch !== undefined &&
      facts.epoch === roomMark.epoch,
    epoch_present: Number.isInteger(facts.epoch) && facts.epoch > 0,
    same_peers:
      roomMark !== null &&
      facts.created === roomMark.created &&
      facts.closed === roomMark.closed,
    same_native:
      roomMark !== null &&
      Object.keys(roomNative).every(
        (key) => facts.native[key] === roomMark.native[key],
      ),
    new_commits:
      roomMark === null ? 0 : Math.max(0, facts.commits - roomMark.commits),
    owned_view: ownedView === true,
  };
}
const canonical = (v) =>
  Array.isArray(v)
    ? v.map(canonical)
    : v && typeof v === "object"
      ? Object.fromEntries(
          Object.keys(v)
            .sort()
            .map((k) => [k, canonical(v[k])]),
        )
      : v;
async function digestValue(value) {
  const bytes = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(JSON.stringify(canonical(value))),
  );
  return [...new Uint8Array(bytes)]
    .map((n) => n.toString(16).padStart(2, "0"))
    .join("");
}
function localSpec() {
  const profile = galaxy.profile();
  if (config.profile_mode !== "owned")
    return { local_seed: profile.seed, identity: profile.identity ?? null };
  if (typeof wasmPorts.readOwnedLoadout !== "function")
    throw new Error("native_owned_unavailable");
  const actual = wasmPorts.readOwnedLoadout(galaxy);
  if (
    JSON.stringify(canonical(actual)) !==
    JSON.stringify(canonical(ownedLoadout))
  )
    throw new Error("native_owned_changed");
  return {
    local_seed: profile.seed,
    identity: profile.identity ?? null,
    owned_loadout: actual,
  };
}
async function digest(profile) {
  const valid = normalizeProfile(profile);
  if (!valid) throw new Error("profile_invalid");
  const bytes = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(JSON.stringify(canonical(valid))),
  );
  return [...new Uint8Array(bytes)]
    .map((n) => n.toString(16).padStart(2, "0"))
    .join("");
}
function observations() {
  const state = controller?.state(),
    d = transport?.diagnostics();
  return {
    role,
    init_stage: initStage,
    failure_code: failureCode,
    phase:
      phase === "disposed" || phase === "failed"
        ? phase
        : (state?.phase ?? phase),
    mode: controller?.frameMode() ?? mode,
    ...counters,
    requests: d?.pendingRequests ?? 0,
    timers: d?.timers ?? 0,
    transport_ready: state?.transport_ready ?? 0,
    admitted: state?.admitted ?? 0,
    profile_equal: profileEqual,
    persisted_equal: persistedEqual,
    persistent,
    progression_false: progressionFalse,
    movement_observed: movementObserved,
    profile_sha: profileSha,
    baseline_sha: baseline ?? null,
    persisted_sha: persistedSha,
    ...(config.profile_mode === "owned"
      ? {
          owned_export: ownedExport,
          owned_view: ownedView,
          selected_ship: ownedLoadout?.selected_ship ?? null,
          owned_slots:
            ownedLoadout?.slots.filter((slot) => slot !== null).length ?? null,
          owned_sha: ownedSha,
        }
      : {}),
    ...(scenario === "renko-fifth" ? { fifth_shot: fifth.report() } : {}),
  };
}
function update() {
  const s = observations();
  status.textContent = `${role.toUpperCase()} · ${s.phase} · init ${s.init_stage}${s.failure_code ? " / " + s.failure_code : ""} · mode ${s.mode} · native frames ${s.frames} · complete commits ${s.commits}`;
}
function observeHeader(header) {
  if (!header?.snapshot) return;
  const s = header.snapshot;
  latestHeader = header;
  if (observedSession !== s.session) {
    observedSession = s.session;
    lastRows = new Map();
    stagedRows = new Map();
    firstPosition = null;
    stagedOwnedView = null;
    ownedView = null;
  }
  counters.ticks = Math.max(counters.ticks, s.tick);
  counters.projectiles = Math.max(counters.projectiles, s.projectiles.length);
  progressionFalse = s.progression === false;
  const peer = transport?.diagnostics().participant?.id;
  const p = s.participants.find((p) =>
    peer ? p.id === peer : p.slot === (role === "host" ? 0 : 1),
  );
  if (config.profile_mode === "owned" && role === "guest")
    stagedOwnedView = Boolean(
      p?.owned_loadout &&
      ownedLoadout &&
      JSON.stringify(canonical(p.owned_loadout)) ===
        JSON.stringify(canonical(ownedLoadout)) &&
      p.selected_ship === ownedLoadout.selected_ship,
    );
  if (p) {
    firstPosition ??= p.position;
    movementObserved =
      Math.hypot(
        p.position[0] - firstPosition[0],
        p.position[1] - firstPosition[1],
      ) > 10 || movementObserved === true;
  }
}
function observeRows(rows) {
  for (const row of rows)
    stagedRows.set(row.id, {
      signature: row.body_signature,
      occupied: row.occupied,
    });
}
function observeCommit() {
  latestCommittedHeader = latestHeader;
  if (config.profile_mode === "owned" && role === "guest")
    ownedView = stagedOwnedView;
  for (const [id, row] of stagedRows) {
    const old = lastRows.get(id);
    if (old?.signature === row.signature) counters.unchanged_bodies++;
    else if (old) {
      counters.body_changes++;
      if (old.occupied === row.occupied) counters.same_count_changes++;
    }
  }
  lastRows = stagedRows;
  stagedRows = new Map();
}
function peerFactory(configuration) {
  const pc = new RTCPeerConnection(configuration);
  counters.created++;
  const close = pc.close.bind(pc);
  let closed = false;
  pc.close = () => {
    if (!closed) {
      closed = true;
      counters.closed++;
    }
    close();
  };
  return pc;
}
function resetControls() {
  controls = {
    fire: false,
    joystick: null,
    destination: null,
    thrust: null,
    aim: null,
  };
  controller?.cancelInput();
  galaxy?.cancel_course();
}
function step(milliseconds) {
  if (!galaxy || stopped) return;
  const before = controller?.frameMode() ?? "solo";
  controller?.beforeStep(milliseconds / 1000, controls);
  const after = controller?.frameMode() ?? "solo";
  if (after !== before) resetControls();
  // Solo Game is deliberately paused throughout QA; only actual host tick_co_op runs.
  const pause = after !== "host" || captureHold;
  if (pause !== appliedPause) {
    galaxy.set_paused(pause);
    appliedPause = pause;
  }
  const elapsed = after !== before ? 0 : milliseconds;
  const frame = galaxy.step(elapsed, controls);
  if (fifth?.state === "firing" && fifth.events(frame?.events)) {
    heldCapture = counters.captures;
    captureHold = true;
    controls = { fire: false };
    galaxy.set_paused(true);
    appliedPause = true;
  }
  controller?.afterStep(elapsed / 1000);
  counters.frames++;
  update();
}
function schedule() {
  clearTimeout(timer);
  if (stopped || performance.now() > loopDeadline) return;
  timer = setTimeout(() => {
    try {
      const now = performance.now(),
        dt = Math.min(50, Math.max(0, now - lastTime));
      lastTime = now;
      if (!pumping) step(dt);
      schedule();
    } catch {
      failureCode = "native_frame_failed";
      phase = "failed";
      stopped = true;
      update();
    }
  }, 16);
}
async function initStep(stage, action, lateCleanup = () => {}) {
  initStage = stage;
  update();
  let timeout,
    expired = false;
  const remaining = Math.max(0, initDeadline - performance.now());
  if (remaining === 0) {
    failureCode = "initialization_timeout";
    throw new Error(failureCode);
  }
  const pending = Promise.resolve()
    .then(action)
    .then((value) => {
      if (expired) lateCleanup(value);
      return value;
    });
  try {
    return await Promise.race([
      pending,
      new Promise((_, reject) => {
        timeout = setTimeout(() => {
          expired = true;
          failureCode = "initialization_timeout";
          reject(new Error("initialization_timeout"));
        }, remaining);
      }),
    ]);
  } catch {
    failureCode ??= `initialization_${stage}_failed`;
    throw new Error(failureCode);
  } finally {
    clearTimeout(timeout);
  }
}
async function initialize(
  selectedBackend,
  runId,
  selectedScenario = "default",
) {
  if (
    galaxy ||
    !["webgpu", "webgl2"].includes(selectedBackend) ||
    !/^[a-f0-9]{16}$/.test(runId)
  )
    throw new Error("initialize_invalid");
  backend = selectedBackend;
  run = runId;
  failureCode = null;
  initDeadline = performance.now() + 45_000;
  scenario = selectedScenario;
  fifth = scenario === "renko-fifth" ? new FifthShotObservation(role) : null;
  store = new SaveStore();
  const loaded = await initStep("open_store", () => store.open());
  const starter = await initStep("starter_selection", () => {
    try {
      return starterSelection(
        selectedScenario,
        role,
        loaded,
        config.profile_mode === "owned",
      );
    } catch (error) {
      failureCode = fifthReasons.has(error.message)
        ? error.message
        : "scenario_unavailable";
      throw error;
    }
  });
  persistent = store.persistent;
  const module = await initStep(
    "import_module",
    () => import(config.builds[backend]),
  );
  await initStep("compile_wasm", () =>
    module.default({
      module_or_path: config.builds[backend].replace(
        /galaxy\.js$/,
        "galaxy_bg.wasm",
      ),
    }),
  );
  galaxy = await initStep(
    "create_galaxy",
    () => module.Galaxy.create(canvas, loaded),
    (lateGalaxy) => lateGalaxy.free(),
  );
  // register_droid requires the real freshly created Galaxy to be paused.
  await initStep("pause_solo", () => {
    galaxy.set_paused(true);
    appliedPause = true;
  });
  if (!loaded) {
    const hash = [...crypto.getRandomValues(new Uint8Array(8))]
      .map((n) => n.toString(16).padStart(2, "0"))
      .join("");
    await initStep("register_license", () =>
      galaxy.register_droid(...starter, hash),
    );
  }
  await initStep("configure_view", () => {
    galaxy.resize(640, 360);
    galaxy.set_ui_mode(1);
    galaxy.set_shake(false);
  });
  if (config.profile_mode === "owned")
    await initStep("read_owned_loadout", async () => {
      if (typeof wasmPorts.readOwnedLoadout !== "function")
        throw new Error("native_owned_unavailable");
      ownedLoadout = wasmPorts.readOwnedLoadout(galaxy);
      ownedSha = await digestValue(ownedLoadout);
      ownedExport = true;
      if (scenario === "renko-fifth") {
        const expected = role === "host" ? [0, 0] : [1, 1];
        if (
          ownedLoadout.selected_ship !== 0 ||
          ownedLoadout.slots.filter(Boolean).length !== 1 ||
          ownedLoadout.slots[expected[0]]?.index !== expected[1] ||
          ownedLoadout.slots[expected[0]]?.rank !== 1
        ) {
          failureCode = "starter_mismatch";
          throw new Error("starter_mismatch");
        }
      }
    });
  const observed = {
    co_op_batch(request) {
      const response = galaxy.co_op_batch(request),
        ops = JSON.parse(request).operations,
        results = JSON.parse(response).results;
      for (let i = 0; i < ops.length; i++)
        if (results[i]?.ok) {
          observeRoomOperation(ops[i].op, results[i].ok);
          const value = results[i].value;
          if (ops[i].op === "acquire") {
            counters.captures++;
            observeHeader(value);
            stagedRows = new Map();
          }
          if (ops[i].op === "entity_page") {
            counters.entity_pages++;
            observeRows(value.entities);
          }
          if (ops[i].op === "body_page") counters.body_pages++;
        }
      return response;
    },
    co_op_replica(request) {
      const op = JSON.parse(request),
        response = galaxy.co_op_replica(request),
        result = JSON.parse(response);
      if (result.ok) {
        observeRoomOperation(op.op, result.ok);
        if (op.op === "replica_stage") {
          observeHeader(op.header);
          stagedRows = new Map();
        }
        if (op.op === "replica_entities") {
          counters.entity_pages++;
          observeRows(op.page.entities);
        }
        if (op.op === "replica_body") counters.body_pages++;
        if (op.op === "replica_commit") {
          counters.commits++;
          observeCommit();
        }
      }
      return response;
    },
  };
  initStage = "bind_controller";
  update();
  const ports = wasmPorts.createGalaxyCoopPorts(observed);
  controller = new CoopController({
    service: config.service,
    native: ports.native,
    replica: ports.replica,
    availability: ports.availability,
    localSpec,
    transportFactory: (options) => {
      transport = new CoopTransport({
        ...options,
        createPeerConnection: peerFactory,
      });
      return transport;
    },
    onChange: () => {
      const next = controller?.frameMode() ?? "solo";
      if (next !== mode) {
        mode = next;
        captureHold = false;
        resetControls();
        lastTime = performance.now();
      }
      update();
    },
  });
  if (!loaded)
    await initStep("save_baseline", async () => {
      await store.save(galaxy.profile());
      await store.flush();
    });
  await initStep("read_baseline", async () => {
    baseline = profileSha = await digest(galaxy.profile());
    persistedSha = await digest(await store.open());
  });
  profileEqual = true;
  persistedEqual = baseline === persistedSha;
  phase = "idle";
  initStage = "ready";
  loopDeadline =
    performance.now() + (scenario === "renko-fifth" ? 600_000 : 180_000);
  lastTime = performance.now();
  stopped = false;
  schedule();
  update();
}
async function verify() {
  if (!galaxy) throw new Error("not_initialized");
  profileSha = await digest(galaxy.profile());
  // Read the real origin's IndexedDB through SaveStore. No session snapshot is saved.
  const persisted = await store.open();
  persistent = store.persistent;
  persistedSha = persisted ? await digest(persisted) : null;
  profileEqual = profileSha === baseline;
  persistedEqual = persistedSha === baseline;
  return observations();
}
async function waitFifth(predicate) {
  const deadline = performance.now() + 30_000;
  while (!predicate()) {
    if (
      stopped ||
      controller.frameMode() === "blocked" ||
      controller.frameMode() === "solo"
    )
      fifth.fail("fifth_native_context");
    if (performance.now() > deadline) fifth.fail("fifth_timeout");
    await new Promise((r) => setTimeout(r, 25));
  }
}
async function fifthCommand(name, args) {
  if (scenario !== "renko-fifth" || !fifth)
    throw new Error("scenario_unavailable");
  if (name === "fifth-depart" && role === "host") {
    if (fifth.state !== "idle" && fifth.state !== "complete")
      fifth.fail("fifth_sequence");
    fifth.state = "departing";
    pumping = true;
    captureHold = false;
    try {
      let departed = false;
      for (let i = 0; i < 480; i++) {
        const origin = galaxy.hud().camera;
        controls = {
          thrust: [origin[0] + 1600, origin[1]],
          aim: [origin[0] + 1900, origin[1]],
          fire: false,
        };
        step(1000 / 60);
        const p = latestHeader?.snapshot.participants.find((p) => p.slot === 0);
        if (p && Math.hypot(...p.position) > 330) {
          departed = true;
          break;
        }
        await new Promise((r) => setTimeout(r, 16));
      }
      if (!departed) fifth.fail("fifth_timeout");
    } finally {
      pumping = false;
      resetControls();
      captureHold = true;
      galaxy.set_paused(true);
      appliedPause = true;
      lastTime = performance.now();
    }
    const before = counters.captures;
    await waitFifth(() => counters.captures > before);
    fifth.start(latestHeader.snapshot);
  } else if (name === "fifth-next" && role === "host") {
    fifth.fire(args.ordinal);
    captureHold = false;
    pumping = true;
    try {
      for (let i = 0; i < 180 && fifth.state === "firing"; i++) {
        const p = latestHeader.snapshot.participants.find((p) => p.slot === 0);
        controls = { aim: [p.position[0] + 1900, p.position[1]], fire: true };
        step(1000 / 60);
        await new Promise((r) => setTimeout(r, 16));
      }
      if (fifth.state !== "holding") fifth.fail("fifth_timeout");
    } finally {
      pumping = false;
      resetControls();
      captureHold = true;
      galaxy.set_paused(true);
      appliedPause = true;
      lastTime = performance.now();
    }
    await waitFifth(() => counters.captures > heldCapture);
    fifth.accept(latestHeader.snapshot, args.ordinal);
  } else if (["fifth-arm", "fifth-accept"].includes(name) && role === "guest") {
    if (
      ![args.tick, args.revision].every(
        (n) => Number.isInteger(n) && n >= 0 && n <= 0xffff_ffff,
      )
    )
      fifth.fail("fifth_sequence");
    await waitFifth(
      () =>
        latestCommittedHeader?.snapshot.session ===
          transport.diagnostics().session?.id &&
        latestCommittedHeader.snapshot.tick === args.tick &&
        latestCommittedHeader.snapshot.revision === args.revision,
    );
    if (name === "fifth-arm") fifth.start(latestCommittedHeader.snapshot);
    else fifth.accept(latestCommittedHeader.snapshot, args.ordinal);
  } else fifth.fail("fifth_sequence");
  return {
    state: observations(),
    held: {
      tick: latestHeader.snapshot.tick,
      revision: latestHeader.snapshot.revision,
    },
  };
}
async function command(name, args) {
  if (name === "initialize") {
    await initialize(args.backend, args.run_id, args.scenario);
    return observations();
  }
  if (name === "state") return observations();
  if (name === "dispose" && !galaxy) {
    stopped = true;
    clearTimeout(timer);
    phase = "disposed";
    return observations();
  }
  if (!controller || !galaxy) throw new Error("not_initialized");
  if (name === "room-state") return roomState();
  if (name === "room-mark") {
    roomMark = roomFacts();
    return roomState();
  }
  if (name === "replace-code") {
    const before = controller.invite();
    if (role !== "host" || !before) throw new Error("approval_not_pending");
    await controller.replaceCode();
    const invite = controller.invite();
    return { invite, changed: typeof invite === "string" && invite !== before };
  }
  if (name === "set-joining-locked") {
    if (
      role !== "host" ||
      Object.keys(args).join() !== "locked" ||
      typeof args.locked !== "boolean"
    )
      throw new Error("approval_not_pending");
    await controller.setJoiningLocked(args.locked);
    return roomState();
  }
  if (name === "approval-state") {
    const current = controller.state();
    return {
      awaiting: current.awaiting_approval,
      pending: current.join_requests.length,
      created: counters.created,
      transport_ready: current.transport_ready,
      admitted: current.admitted,
    };
  }
  if (name === "approve-next" || name === "decline-next") {
    const requests = controller.state().join_requests;
    if (role !== "host" || requests.length !== 1)
      throw new Error("approval_not_pending");
    await controller.approve(requests[0].id, name === "approve-next");
    return observations();
  }
  if (name.startsWith("fifth-")) return fifthCommand(name, args);
  if (name === "create") {
    await controller.create();
    return {
      invite: controller.invite(),
      session: transport.diagnostics().session?.id,
    };
  }
  if (name === "join") {
    await controller.join(args.invite);
    return observations();
  }
  if (name === "verify") return verify();
  if (name === "drive") {
    if (scenario !== "default") throw new Error("scenario_unavailable");
    if (
      controller.frameMode() === "solo" ||
      controller.frameMode() === "blocked"
    )
      throw new Error("not_admitted");
    pumping = true;
    try {
      for (let i = 0; i < 360; i++) {
        if (!controller.blocksSolo() || controller.frameMode() === "blocked")
          break;
        const h = galaxy.hud(),
          origin = h.camera;
        controls = {
          thrust: [origin[0] + 1600, origin[1]],
          aim: [origin[0] + 1900, origin[1]],
          fire: true,
        };
        step(1000 / 60);
        await new Promise((r) => setTimeout(r, 16));
      }
    } finally {
      pumping = false;
      resetControls();
      lastTime = performance.now();
    }
    return observations();
  }
  if (name === "end") await controller.end();
  else if (name === "leave") await controller.leave();
  else if (name === "dispose") {
    await controller.dispose();
    if (controller.blocksSolo()) throw new Error("restore_failed");
    await verify();
    stopped = true;
    clearTimeout(timer);
    galaxy.free();
    galaxy = null;
    phase = "disposed";
    update();
    return observations();
  } else throw new Error("command_invalid");
  resetControls();
  return verify();
}
let commandPending = false;
addEventListener("message", async (event) => {
  if (
    event.source !== parent ||
    event.origin !== config.page_origin ||
    event.data?.type !== "native-qa-command" ||
    !Number.isInteger(event.data.id) ||
    event.data.id < 1 ||
    event.data.id > 1_000_000
  )
    return;
  const { id, name, args = {} } = event.data;
  if (commandPending) return;
  commandPending = true;
  try {
    const value = await command(name, args);
    parent.postMessage(
      { type: "native-qa-reply", id, ok: true, value },
      config.page_origin,
    );
  } catch (error) {
    if (name === "initialize") {
      await controller?.dispose().catch(() => {});
      try {
        galaxy?.free();
      } catch {}
      galaxy = null;
      stopped = true;
      clearTimeout(timer);
    }
    failureCode ??=
      name === "initialize"
        ? `initialization_${initStage}_failed`
        : fifthReasons.has(error.message) ||
            error.message === "approval_not_pending"
          ? error.message
          : "native_action_failed";
    if (fifth && name.startsWith("fifth-")) {
      try {
        fifth.fail(failureCode);
      } catch {}
      captureHold = true;
      resetControls();
      galaxy?.set_paused(true);
      appliedPause = true;
    }
    phase = "failed";
    update();
    parent.postMessage(
      { type: "native-qa-reply", id, ok: false, error: failureCode },
      config.page_origin,
    );
  } finally {
    commandPending = false;
  }
});
addEventListener("pagehide", () => {
  stopped = true;
  clearTimeout(timer);
  void controller?.dispose();
});
parent.postMessage(
  { type: "native-qa-client-ready", role },
  config.page_origin,
);
