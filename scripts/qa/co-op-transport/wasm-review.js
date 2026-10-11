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
  firstSession = null,
  sessionDistinct = null;
const pending = new Map(),
  checks = [];
const state = { host: null, guest: null };
const failureCodes = new Set([
  ...[
    "none",
    "open_store",
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
]);
const actionFailureCodes = new Set([
  ...failureCodes,
  "iframe_not_ready",
  "native_command_timeout",
  "native_timeout",
  "capture_rejected",
  "clients_not_visible",
  "page_not_visible",
]);
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
      "Drive pending: visibility changed. Wait for commands, then leave/dispose; this drive will not pass.";
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
function send(role, name, args = {}) {
  const id = nextId++,
    child = children[role];
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error("native_command_timeout"));
    }, 75_000);
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
    "end",
    "rejoin",
    "leave",
    "verify",
    "dispose",
  ])
    $(id).disabled = busy || disposed;
  $("initialize").disabled ||= initialized;
  $("backend").disabled = busy || initialized;
  for (const id of ["connect", "drive", "end", "rejoin", "leave", "verify"])
    $(id).disabled ||= !initialized;
  $("drive").disabled ||= state.guest?.phase !== "connected";
}
async function capture() {
  if (!state.host || !state.guest) return;
  const response = await fetch("/qa/report", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      version: config.profile_mode === "owned" ? 3 : 2,
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
async function connected(rejoin) {
  const commitsBefore = state.guest?.commits ?? 0;
  const grant = await send("host", "create");
  if (typeof grant.invite !== "string") throw new Error("invite_unavailable");
  if (firstSession) sessionDistinct = firstSession !== grant.session;
  else firstSession = grant.session;
  await send("guest", "join", { invite: grant.invite });
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
    [state.host, state.guest] = await Promise.all([
      send("host", "initialize", { backend, run_id: run }),
      send("guest", "initialize", { backend, run_id: run }),
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
