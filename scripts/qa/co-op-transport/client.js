import { CoopController } from "/modules/controller.js";
import { CoopTransport } from "/modules/transport.js";
import * as wasmPorts from "/modules/wasm-ports.js";
import { SaveStore, normalizeProfile } from "/modules/storage.js";

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
  };
}
function update() {
  const s = observations();
  status.textContent = `${role.toUpperCase()} · ${s.phase} · init ${s.init_stage}${s.failure_code ? " / " + s.failure_code : ""} · mode ${s.mode} · native frames ${s.frames} · complete commits ${s.commits}`;
}
function observeHeader(header) {
  if (!header?.snapshot) return;
  const s = header.snapshot;
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
  const pause = after !== "host";
  if (pause !== appliedPause) {
    galaxy.set_paused(pause);
    appliedPause = pause;
  }
  const elapsed = after !== before ? 0 : milliseconds;
  galaxy.step(elapsed, controls);
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
async function initialize(selectedBackend, runId) {
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
  store = new SaveStore();
  const loaded = await initStep("open_store", () => store.open());
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
      galaxy.register_droid(
        role === "host" ? "dora" : "doro",
        role === "host" ? "pan" : "hugo",
        role === "host" ? "renko" : "pan",
        hash,
      ),
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
    });
  const observed = {
    co_op_batch(request) {
      const response = galaxy.co_op_batch(request),
        ops = JSON.parse(request).operations,
        results = JSON.parse(response).results;
      for (let i = 0; i < ops.length; i++)
        if (results[i]?.ok) {
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
        resetControls();
        lastTime = performance.now();
      }
      update();
    },
  });
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
  loopDeadline = performance.now() + 180_000;
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
async function command(name, args) {
  if (name === "initialize") {
    await initialize(args.backend, args.run_id);
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
  } catch {
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
        : "native_action_failed";
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
