const config = await (await fetch("/qa/config", { cache: "no-store" })).json();
const $ = (id) => document.getElementById(id);
const run = [...crypto.getRandomValues(new Uint8Array(8))]
  .map((n) => n.toString(16).padStart(2, "0"))
  .join("");
const children = {
  host: { frame: $("host"), origin: config.page_origin },
  guest: { frame: $("guest"), origin: config.guest_origin },
};
let nextId = 1,
  busy = false,
  initialized = false,
  disposed = false,
  backend = "webgpu",
  scenario = "default",
  firstSession = null,
  sessionDistinct = null;
const pending = new Map(),
  checks = [];
const state = { host: null, guest: null };
const failureCodes = new Set([
  ...[
    "none",
    "open_store",
    "starter_selection",
    "import_module",
    "compile_wasm",
    "create_galaxy",
    "pause_solo",
    "register_license",
    "configure_view",
    "read_owned_loadout",
    "bind_controller",
    "save_baseline",
    "read_baseline",
    "ready",
  ].map((stage) => `initialization_${stage}_failed`),
  "initialization_timeout",
  "native_frame_failed",
  "native_action_failed",
  "fresh_save_required",
  "scenario_unavailable",
  "starter_mismatch",
  "fifth_sequence",
  "fifth_ambiguous_events",
  "fifth_queue_full",
  "fifth_native_context",
  "fifth_shot_not_retained",
  "fifth_wrong_pattern",
  "fifth_mismatch",
  "fifth_timeout",
  "fifth_not_ready",
  "approval_not_pending",
  "signaling_limit",
  "room_create_failed",
]);
const actionFailureCodes = new Set([
  ...failureCodes,
  "iframe_not_ready",
  "native_command_timeout",
  "native_timeout",
  "capture_rejected",
  "clients_not_visible",
  "page_not_visible",
  "approval_timeout",
  "approval_not_pending",
  "approval_before_rtc_failed",
  "room_case_failed",
  "room_deadline",
  "room_precondition",
]);
const roomCases = new Set();
let roomAttempted = false;
let driveActive = false,
  driveVisibilityLost = false;
function requireVisibleClients() {
  if (document.visibilityState !== "visible")
    throw new Error("page_not_visible");
  const width = window.innerWidth,
    height = window.innerHeight;
  if (
    !Number.isFinite(width) ||
    !Number.isFinite(height) ||
    width <= 0 ||
    height <= 0
  )
    throw new Error("clients_not_visible");
  for (const child of Object.values(children)) {
    const r = child.frame.getBoundingClientRect();
    if (
      ![r.left, r.top, r.right, r.bottom, r.width, r.height].every(
        Number.isFinite,
      ) ||
      r.width <= 0 ||
      r.height <= 0 ||
      r.left < 0 ||
      r.top < 0 ||
      r.right > width ||
      r.bottom > height
    )
      throw new Error("clients_not_visible");
  }
}
function noteDriveVisibility() {
  if (!driveActive) return;
  try {
    requireVisibleClients();
  } catch {
    driveVisibilityLost = true;
    $("status").textContent =
      "Native observation pending: visibility changed. Wait for commands, then leave/dispose; this action will not pass.";
  }
}
function guardDriveVisibility() {
  requireVisibleClients();
  if (driveVisibilityLost) throw new Error("clients_not_visible");
}
document.addEventListener("visibilitychange", noteDriveVisibility);
addEventListener("resize", noteDriveVisibility);
addEventListener("scroll", noteDriveVisibility);
for (const child of Object.values(children)) {
  child.ready = false;
  child.frame.src = child.origin + "/client.html";
}
addEventListener("message", (event) => {
  const pair = Object.entries(children).find(
    ([, c]) =>
      event.source === c.frame.contentWindow && event.origin === c.origin,
  );
  if (!pair) return;
  if (event.data?.type === "native-qa-client-ready") {
    pair[1].ready = true;
    return;
  }
  if (event.data?.type !== "native-qa-reply") return;
  const request = pending.get(event.data.id);
  if (!request || request.role !== pair[0]) return;
  pending.delete(event.data.id);
  clearTimeout(request.timer);
  event.data.ok
    ? request.resolve(event.data.value)
    : request.reject(
        new Error(
          failureCodes.has(event.data.error)
            ? event.data.error
            : "native_action_failed",
        ),
      );
});
function send(role, name, args = {}, timeoutMs = 75_000) {
  const id = nextId++,
    child = children[role];
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error("native_command_timeout"));
    }, timeoutMs);
    pending.set(id, { role, resolve, reject, timer });
    child.frame.contentWindow.postMessage(
      { type: "native-qa-command", id, name, args },
      child.origin,
    );
  });
}
async function read() {
  [state.host, state.guest] = await Promise.all([
    send("host", "state"),
    send("guest", "state"),
  ]);
  paint();
}
function check(name, passed) {
  checks.push({ name, passed: Boolean(passed) });
  if (!passed) throw new Error("declared_check_failed");
}
async function until(predicate) {
  const deadline = performance.now() + 65_000;
  while (true) {
    await read();
    if (predicate()) return;
    if (performance.now() > deadline) throw new Error("native_timeout");
    await new Promise((r) => setTimeout(r, 250));
  }
}
function paint() {
  $("results").textContent = JSON.stringify(
    {
      scope:
        "Actual two native Games/RTC on independent QA origins; no Internet/performance acceptance",
      backend,
      profile_mode: config.profile_mode ?? "factory",
      scenario,
      checks,
      clients: [state.host, state.guest],
      session_distinct: sessionDistinct,
    },
    null,
    2,
  );
  for (const id of [
    "initialize",
    "connect",
    "drive",
    "fifth",
    "end",
    "rejoin",
    "leave",
    "verify",
    "dispose",
    "room-decline",
    "room-replace",
    "room-lock",
  ])
    $(id).disabled = busy || disposed;
  $("initialize").disabled ||= initialized;
  $("backend").disabled = busy || initialized;
  $("scenario").disabled = busy || initialized;
  $("scenario").querySelector('[value="renko-fifth"]').disabled =
    !config.scenarios?.includes("renko-fifth");
  for (const id of ["connect", "drive", "end", "rejoin", "leave", "verify"])
    $(id).disabled ||= !initialized;
  $("drive").disabled ||= state.guest?.phase !== "connected";
  $("drive").disabled ||= scenario !== "default";
  $("fifth").disabled ||=
    !initialized ||
    scenario !== "renko-fifth" ||
    state.guest?.phase !== "connected" ||
    state.host?.fifth_shot?.state === "complete";
  for (const kind of ["decline", "replace", "lock"])
    $("room-" + kind).disabled ||=
      !initialized ||
      scenario !== "default" ||
      config.profile_mode !== "owned" ||
      roomCases.size > 0 ||
      roomAttempted;
}
async function capture() {
  if (!state.host || !state.guest) return;
  const response = await fetch("/qa/report", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      version:
        scenario === "renko-fifth"
          ? 4
          : config.profile_mode === "owned"
            ? 3
            : 2,
      ...(scenario === "renko-fifth" ? { scenario } : {}),
      ...(config.profile_mode === "owned" ? { profile_mode: "owned" } : {}),
      run_id: run,
      backend,
      checks,
      clients: [state.host, state.guest],
      session_distinct: sessionDistinct,
    }),
  });
  if (!response.ok) throw new Error("capture_rejected");
}
async function action(label, fn) {
  if (busy) return;
  busy = true;
  paint();
  $("status").textContent = label + ": running";
  try {
    await fn();
    await capture();
    $("status").textContent = label + ": complete; sanitized report captured";
  } catch (error) {
    checks.push({ name: "operation-failed", passed: false });
    await read().catch(() => {});
    await capture().catch(() => {});
    $("status").textContent =
      label +
      ": failed / " +
      (actionFailureCodes.has(error?.message)
        ? error.message
        : "native_action_failed") +
      (["clients_not_visible", "page_not_visible"].includes(error?.message)
        ? ". Keep the page foreground and both client frames fully visible; enlarge the viewport or scroll to the frames before retrying."
        : "");
  } finally {
    busy = false;
    paint();
  }
}
async function verify() {
  [state.host, state.guest] = await Promise.all([
    send("host", "verify"),
    send("guest", "verify"),
  ]);
  check(
    "saves-unchanged",
    state.host.profile_equal &&
      state.guest.profile_equal &&
      state.host.persisted_equal &&
      state.guest.persisted_equal,
  );
}
// Transport/admission values are derived in the child from the actual controller, not inferred by the parent.
async function approvePendingJoin(baseline) {
  const deadline = performance.now() + 15_000;
  let host, guest;
  while (true) {
    const remaining = deadline - performance.now();
    if (remaining <= 0) throw new Error("approval_timeout");
    [host, guest] = await Promise.all([
      send("host", "approval-state", {}, remaining),
      send("guest", "approval-state", {}, remaining),
    ]);
    if (performance.now() >= deadline) throw new Error("approval_timeout");
    if (host.pending === 1 && guest.awaiting === true) break;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  const isolated = [host, guest].every(
    (value, index) =>
      value.created === baseline[index] &&
      value.transport_ready === 0 &&
      value.admitted === 0,
  );
  check("host-approval-before-rtc", isolated);
  if (!isolated) throw new Error("approval_before_rtc_failed");
  await send("host", "approve-next");
}
async function connected(rejoin) {
  const commitsBefore = state.guest?.commits ?? 0;
  const grant = await send("host", "create");
  if (typeof grant.invite !== "string") throw new Error("invite_unavailable");
  if (firstSession) sessionDistinct = firstSession !== grant.session;
  else firstSession = grant.session;
  await read();
  const peersBefore = [state.host.created, state.guest.created];
  await send("guest", "join", { invite: grant.invite });
  await approvePendingJoin(peersBefore);
  await until(
    () =>
      state.guest.phase === "connected" &&
      state.guest.commits > commitsBefore &&
      state.host.mode === "host" &&
      state.guest.mode === "guest",
  );
  check(
    "real-channel-ready",
    state.host.transport_ready > 0 && state.guest.transport_ready > 0,
  );
  check(
    "native-admission",
    state.guest.admitted > 0 && state.host.admitted > 0,
  );
  check("native-complete-commit", state.guest.commits > commitsBefore);
  if (config.profile_mode === "owned")
    check("native-owned-commit", state.guest.owned_view === true);
  if (rejoin) check("new-session", sessionDistinct === true);
}
async function roomSend(role, name, args, deadline) {
  const remaining = deadline - performance.now();
  if (remaining <= 0) throw new Error("room_deadline");
  if (driveActive) guardDriveVisibility();
  const value = await send(
    role,
    name,
    args,
    Math.min(remaining, name === "room-state" ? 5_000 : 10_000),
  );
  if (performance.now() >= deadline) throw new Error("room_deadline");
  if (driveActive) guardDriveVisibility();
  return value;
}
async function roomRead(deadline) {
  const values = await Promise.all([
    roomSend("host", "room-state", {}, deadline),
    roomSend("guest", "room-state", {}, deadline),
  ]);
  if (driveActive) {
    // Finish each child's private command before asking for its public failure state.
    const health = await Promise.all([
      roomSend("host", "state", {}, deadline),
      roomSend("guest", "state", {}, deadline),
    ]);
    if (
      values.some(
        (value) => !Number.isFinite(value.life_ms) || value.life_ms <= 0,
      ) ||
      health.some(
        (value) =>
          value.init_stage !== "ready" ||
          value.failure_code !== null ||
          ["failed", "disposed"].includes(value.phase),
      )
    )
      throw new Error("room_case_failed");
  }
  return values;
}
async function roomUntil(predicate, deadline) {
  while (true) {
    if (driveActive) guardDriveVisibility();
    const values = await roomRead(deadline);
    if (driveActive) guardDriveVisibility();
    if (predicate(values)) return values;
    await new Promise((resolve) =>
      setTimeout(
        resolve,
        Math.min(100, Math.max(1, deadline - performance.now())),
      ),
    );
  }
}
async function roomMarkBoth(deadline) {
  await Promise.all([
    roomSend("host", "room-mark", {}, deadline),
    roomSend("guest", "room-mark", {}, deadline),
  ]);
}
async function roomPending(invite, deadline) {
  await roomSend("guest", "room-mark", {}, deadline);
  await roomSend("guest", "join", { invite }, deadline);
  const [host, guest] = await roomUntil(
    ([h, g]) => h.pending === 1 && g.awaiting,
    Math.min(deadline, performance.now() + 10_000),
  );
  check(
    "room-pending-isolated",
    [host, guest].every(
      (value) =>
        value.same_peers &&
        value.same_native &&
        value.ready === 0 &&
        value.admitted === 0,
    ),
  );
}
async function roomTerminal(reason, deadline) {
  const values = await roomUntil(([, guest]) => {
    if (
      guest.phase === "ended" &&
      guest.mode === "solo" &&
      guest.reason !== reason
    )
      throw new Error("room_case_failed");
    return (
      guest.reason === reason &&
      guest.mode === "solo" &&
      !guest.awaiting &&
      guest.pending === 0 &&
      guest.ready === 0 &&
      guest.admitted === 0 &&
      guest.created === guest.closed &&
      guest.requests === 0 &&
      guest.timers === 0
    );
  }, deadline);
  check(
    "room-policy-keeps-host",
    values[0].mode === "host" &&
      values[0].same_room &&
      values[0].same_native &&
      values[1].same_peers &&
      values[1].same_native,
  );
  return values;
}
async function roomAccept(deadline) {
  await roomSend("host", "approve-next", {}, deadline);
  return roomUntil(
    ([h, g]) =>
      h.mode === "host" &&
      g.mode === "guest" &&
      h.ready > 0 &&
      g.ready > 0 &&
      h.admitted > 0 &&
      g.admitted > 0 &&
      g.new_commits > 0 &&
      g.owned_view &&
      g.epoch_present,
    deadline,
  );
}
async function roomContinuity(deadline) {
  // Keep pre-policy ownership marks; progress must follow this post-reply read.
  const [, afterPolicy] = await roomRead(deadline);
  const commitFloor = afterPolicy.new_commits;
  if (!Number.isInteger(commitFloor) || commitFloor < 0)
    throw new Error("room_case_failed");
  const values = await roomUntil(
    ([h, g]) =>
      h.ready > 0 &&
      g.ready > 0 &&
      h.admitted > 0 &&
      g.admitted > 0 &&
      g.new_commits > commitFloor &&
      g.owned_view,
    deadline,
  );
  check(
    "room-policy-keeps-host",
    values[0].mode === "host" &&
      values[1].mode === "guest" &&
      values.every((v) => v.same_room && v.same_peers && v.same_native) &&
      values[1].same_epoch,
  );
  return values;
}
async function roomGuestLeave(deadline) {
  await roomSend("guest", "leave", {}, deadline);
  await roomUntil(
    ([h, g]) =>
      g.mode === "solo" &&
      g.created === g.closed &&
      g.requests === 0 &&
      g.timers === 0 &&
      h.ready === 0 &&
      h.admitted === 0 &&
      h.created === h.closed,
    deadline,
  );
  await roomSend("host", "room-mark", {}, deadline);
}
async function roomFinish(deadline) {
  await Promise.all([
    roomSend("host", "leave", {}, deadline),
    roomSend("guest", "leave", {}, deadline),
  ]);
  await roomUntil(
    (values) =>
      values.every(
        (v) =>
          v.mode === "solo" &&
          v.created === v.closed &&
          v.requests === 0 &&
          v.timers === 0,
      ),
    deadline,
  );
  [state.host, state.guest] = await Promise.all([
    roomSend("host", "verify", {}, deadline),
    roomSend("guest", "verify", {}, deadline),
  ]);
  check(
    "room-case-clean",
    [state.host, state.guest].every(
      (v) => v.profile_equal && v.persisted_equal,
    ),
  );
}
async function privateRoomRound(kind) {
  requireVisibleClients();
  if (
    !["decline", "replace", "lock"].includes(kind) ||
    !initialized ||
    scenario !== "default" ||
    config.profile_mode !== "owned" ||
    roomCases.size > 0 ||
    roomAttempted
  )
    throw new Error("room_precondition");
  const deadline = performance.now() + 120_000;
  driveVisibilityLost = false;
  driveActive = true;
  try {
    const before = await roomRead(deadline);
    guardDriveVisibility();
    if (
      !before.every(
        (v) =>
          v.mode === "solo" &&
          v.created === v.closed &&
          v.requests === 0 &&
          v.timers === 0 &&
          v.life_ms >= 140_000,
      )
    )
      throw new Error("room_precondition");
  } catch (error) {
    driveActive = false;
    throw error;
  }
  let failure;
  roomAttempted = true;
  try {
    const grant = await roomSend("host", "create", {}, deadline);
    if (typeof grant.invite !== "string") throw new Error("room_case_failed");
    await roomMarkBoth(deadline);
    await roomPending(grant.invite, deadline);
    if (kind === "decline") {
      await roomSend("host", "decline-next", {}, deadline);
      await roomTerminal("join_declined", deadline);
      check("room-decline-terminal", true);
    } else if (kind === "replace") {
      await roomAccept(deadline);
      await roomMarkBoth(deadline);
      const changed = await roomSend("host", "replace-code", {}, deadline);
      check(
        "room-replace-approved",
        changed.changed === true && typeof changed.invite === "string",
      );
      await roomContinuity(deadline);
      // Joining retires an existing guest session: leave before the stale-code probe.
      await roomGuestLeave(deadline);
      await roomSend("guest", "room-mark", {}, deadline);
      await roomSend("guest", "join", { invite: grant.invite }, deadline);
      await roomTerminal("stale_invite", deadline);
      check("room-stale-code", true);
      await roomPending(changed.invite, deadline);
      const rotated = await roomSend("host", "replace-code", {}, deadline);
      check("room-replace-pending", rotated.changed === true);
      await roomTerminal("code_replaced", deadline);
    } else {
      const oldRevision = (await roomRead(deadline))[0].revision;
      await roomSend("host", "set-joining-locked", { locked: true }, deadline);
      const locked = await roomTerminal("joining_locked", deadline);
      check(
        "room-lock-pending",
        locked[0].locked && locked[0].revision > oldRevision,
      );
      await roomSend("guest", "room-mark", {}, deadline);
      await roomSend("guest", "join", { invite: grant.invite }, deadline);
      await roomTerminal("joining_locked", deadline);
      check("room-locked-join", true);
      await roomSend("host", "set-joining-locked", { locked: false }, deadline);
      const unlocked = await roomRead(deadline);
      check(
        "room-unlock-rejoin",
        !unlocked[0].locked &&
          unlocked[0].revision > locked[0].revision &&
          unlocked[0].same_room,
      );
      await roomPending(grant.invite, deadline);
      await roomAccept(deadline);
      await roomMarkBoth(deadline);
      await roomSend("host", "set-joining-locked", { locked: true }, deadline);
      const retained = await roomContinuity(deadline);
      check("room-lock-approved", retained[0].locked);
      await roomMarkBoth(deadline);
      await roomSend("host", "set-joining-locked", { locked: false }, deadline);
      const retainedAfterUnlock = await roomContinuity(deadline);
      check("room-lock-approved", !retainedAfterUnlock[0].locked);
    }
  } catch (error) {
    failure = error;
  } finally {
    // Stop qualification before cleanup: hidden clients must still retire.
    driveActive = false;
    try {
      await roomFinish(performance.now() + 20_000);
    } catch (error) {
      failure ??= error;
    }
  }
  if (failure) throw failure;
  roomCases.add(kind);
}
for (const kind of ["decline", "replace", "lock"])
  $("room-" + kind).onclick = () =>
    action("Actual room " + kind, () => privateRoomRound(kind));

async function restartSession() {
  // leave is valid for both live and already-ended controllers; end is host/live-only.
  await send("guest", "leave");
  await send("host", "leave");
  await connected(true);
}
$("initialize").onclick = () =>
  action("Initialize", async () => {
    const deadline = performance.now() + 15_000;
    while (!children.host.ready || !children.guest.ready) {
      if (performance.now() > deadline) throw new Error("iframe_not_ready");
      await new Promise((r) => setTimeout(r, 30));
    }
    backend = $("backend").value;
    scenario = $("scenario").value;
    if (scenario !== "default" && !config.scenarios?.includes(scenario))
      throw new Error("scenario_unavailable");
    [state.host, state.guest] = await Promise.all([
      send("host", "initialize", { backend, run_id: run, scenario }),
      send("guest", "initialize", { backend, run_id: run, scenario }),
    ]);
    initialized = true;
    check(
      "two-native-games",
      state.host.phase === "idle" && state.guest.phase === "idle",
    );
    check(
      "independent-save-origins",
      children.host.origin !== children.guest.origin &&
        state.host.persistent &&
        state.guest.persistent,
    );
    check(
      "distinct-local-profiles",
      state.host.profile_sha !== state.guest.profile_sha,
    );
    if (config.profile_mode === "owned")
      check(
        "native-owned-export",
        state.host.owned_export === true && state.guest.owned_export === true,
      );
    await verify();
  });
$("connect").onclick = () => action("Create and join", () => connected(false));
async function drive() {
  // Browser cross-origin off-screen throttling is outside native acceptance.
  requireVisibleClients();
  driveVisibilityLost = false;
  driveActive = true;
  try {
    await Promise.all([send("host", "drive"), send("guest", "drive")]);
    guardDriveVisibility();
    await until(() => {
      guardDriveVisibility();
      return (
        state.host.movement_observed &&
        state.guest.movement_observed &&
        state.host.projectiles > 0 &&
        state.guest.projectiles > 0
      );
    });
    await verify();
    guardDriveVisibility();
    check("movement-fire-observed", true);
  } finally {
    driveActive = false;
  }
}
$("drive").onclick = () => action("Native drive/fire", drive);
async function fifthShot() {
  requireVisibleClients();
  driveVisibilityLost = false;
  driveActive = true;
  try {
    const baseline = await send("host", "fifth-depart");
    state.host = baseline.state;
    const armed = await send("guest", "fifth-arm", baseline.held);
    state.guest = armed.state;
    guardDriveVisibility();
    for (let ordinal = 1; ordinal <= 10; ordinal++) {
      guardDriveVisibility();
      const held = await send("host", "fifth-next", { ordinal });
      state.host = held.state;
      const committed = await send("guest", "fifth-accept", {
        ordinal,
        ...held.held,
      });
      state.guest = committed.state;
      guardDriveVisibility();
      paint();
    }
    await verify();
    guardDriveVisibility();
    check(
      "native-fifth-shot",
      state.host.fifth_shot.state === "complete" &&
        state.guest.fifth_shot.state === "complete",
    );
  } finally {
    driveActive = false;
  }
}
$("fifth").onclick = () => action("RENKO native fifth-shot", fifthShot);
$("end").onclick = () =>
  action("Host end", async () => {
    await send("host", "end");
    await until(
      () =>
        state.host.mode === "solo" &&
        state.guest.mode === "solo" &&
        state.host.created === state.host.closed &&
        state.guest.created === state.guest.closed &&
        state.host.requests === 0 &&
        state.guest.requests === 0 &&
        state.host.timers === 0 &&
        state.guest.timers === 0,
    );
    check("host-end-restored", true);
    await verify();
  });
$("rejoin").onclick = () => action("New session", restartSession);
$("leave").onclick = () =>
  action("Guest leave", async () => {
    await send("guest", "leave");
    await until(
      () =>
        state.guest.mode === "solo" &&
        state.guest.created === state.guest.closed &&
        state.guest.requests === 0 &&
        state.guest.timers === 0,
    );
    check("guest-leave-restored", true);
    await verify();
  });
$("verify").onclick = () => action("Read durable saves", verify);
$("dispose").onclick = () =>
  action("Dispose", async () => {
    [state.host, state.guest] = await Promise.all([
      send("host", "dispose"),
      send("guest", "dispose"),
    ]);
    check(
      "dispose-clean",
      [state.host, state.guest].every(
        (c) =>
          c.created === c.closed &&
          c.requests === 0 &&
          c.timers === 0 &&
          c.profile_equal &&
          c.persisted_equal,
      ),
    );
    disposed = true;
  });
addEventListener("pagehide", () => {
  for (const c of Object.values(children))
    c.frame.contentWindow.postMessage(
      { type: "native-qa-command", id: nextId++, name: "dispose" },
      c.origin,
    );
  for (const r of pending.values()) clearTimeout(r.timer);
  pending.clear();
});
paint();
