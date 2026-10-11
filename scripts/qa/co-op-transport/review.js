import { CoopTransport } from "/modules/transport.js";
const $ = (id) => document.getElementById(id);
const config = await (await fetch("/qa/config", { cache: "no-store" })).json();
let host = null,
  guest = null,
  busy = false,
  firstSession = null;
let created = 0,
  closed = 0,
  hostFrames = [],
  guestFrames = [];
const runId = crypto
  .getRandomValues(new Uint32Array(2))
  .reduce((s, x) => s + x.toString(16).padStart(8, "0"), "");
const checks = [];
const identity = {
  kind: "dora",
  corporation: "pan",
  sub_corporation: "renko",
  hex_id: "7f8a52daece2ed52",
};
function check(name, passed, counts = {}) {
  checks.push({ name, passed, counts });
  if (!passed) throw new Error("Declared check failed");
}
async function until(condition) {
  const deadline = performance.now() + 18000;
  while (!condition()) {
    if (performance.now() > deadline)
      throw new Error("Native RTC condition timed out");
    await new Promise((r) => setTimeout(r, 30));
  }
}
function factory(configuration) {
  const pc = new RTCPeerConnection(configuration);
  created++;
  const nativeClose = pc.close.bind(pc);
  let done = false;
  pc.close = () => {
    if (!done) {
      done = true;
      closed++;
    }
    nativeClose();
  };
  return pc;
}
function state() {
  const clean = (client) => {
    const d = client?.diagnostics();
    return d
      ? {
          phase: d.phase,
          reason: d.reason,
          participantStatus: d.participant?.status ?? null,
          peers: d.peers.map((p) => ({
            transport: p.transport,
            admission: p.admission,
            simulation: p.simulation,
            bufferedBytes: p.bufferedBytes,
          })),
          pendingRequests: d.pendingRequests,
          timers: d.timers,
          rtcErrors: d.rtcErrors,
        }
      : null;
  };
  return { host: clean(host), guest: clean(guest), created, closed };
}
function paint() {
  $("results").textContent = JSON.stringify(
    {
      scope: "same-page native RTC loopback, no simulation",
      checks,
      state: state(),
    },
    null,
    2,
  );
  $("send").disabled =
    busy ||
    !host?.diagnostics().peers.some((p) => p.transport === "ready") ||
    checks.some((c) => c.name === "input-delivered");
  $("end").disabled = busy || host?.diagnostics().phase !== "signaling";
  $("rejoin").disabled =
    busy ||
    host?.diagnostics().phase !== "ended" ||
    guest?.diagnostics().phase !== "ended";
  $("connect").disabled =
    busy || (host !== null && host.diagnostics().phase !== "ended");
  $("dispose").disabled = busy;
}
async function capture() {
  const response = await fetch("/qa/report", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      version: 1,
      run_id: runId,
      checks,
      created,
      closed,
      secure_context: isSecureContext,
      rtc_available: typeof RTCPeerConnection === "function",
      rtc_errors: [
        ...(host?.diagnostics().rtcErrors ?? []),
        ...(guest?.diagnostics().rtcErrors ?? []),
      ].slice(-8),
    }),
  });
  if (!response.ok) throw new Error("Sanitized report capture failed");
}
async function action(label, fn) {
  if (busy) return;
  busy = true;
  paint();
  $("status").textContent = `${label}: running`;
  try {
    await fn();
    await capture();
    $("status").textContent = `${label}: complete; sanitized report saved`;
  } catch {
    checks.push({ name: "operation-failed", passed: false, counts: {} });
    await capture().catch(() => {});
    $("status").textContent =
      `${label}: failed; inspect declared checks (no secret diagnostics)`;
  } finally {
    busy = false;
    paint();
  }
}
async function connect(rejoining) {
  if (location.origin !== config.page_origin)
    throw new Error("Open the configured localhost page origin.");
  if (typeof RTCPeerConnection !== "function" || !isSecureContext)
    throw new Error("Native RTC unavailable");
  hostFrames = [];
  guestFrames = [];
  if (!host)
    host = new CoopTransport({
      service: config.service,
      createPeerConnection: factory,
      onChange: paint,
      onFrame: (peer, frame) => hostFrames.push({ peer, frame }),
    });
  if (!guest)
    guest = new CoopTransport({
      service: config.service,
      createPeerConnection: factory,
      onChange: paint,
      onFrame: (peer, frame) => guestFrames.push({ peer, frame }),
    });
  await host.create(rejoining ? 456 : 123);
  await guest.join(host.invite());
  await until(
    () =>
      host.diagnostics().peers[0]?.transport === "ready" &&
      guest.diagnostics().peers[0]?.transport === "ready",
  );
  const session = host.diagnostics().session.id;
  check(rejoining ? "new-session-ready" : "real-channel-ready", true, {
    created,
    closed,
  });
  check(
    rejoining ? "rejoin-admission-pending" : "admission-pending",
    guest.diagnostics().peers[0].admission === "pending" &&
      guest.diagnostics().peers[0].simulation === "waiting",
  );
  if (rejoining) check("distinct-session", session !== firstSession);
  else firstSession = session;
}
$("connect").onclick = () => action("Connect", () => connect(false));
$("send").onclick = () =>
  action("Declared join/input fixture", async () => {
    const guestId = guest.diagnostics().participant.id,
      hostId = host.diagnostics().participant.id;
    check(
      "join-sent",
      guest.send(hostId, "join", { local_seed: 987, identity }),
    );
    await until(() => hostFrames.some((x) => x.frame.kind === "join"));
    check(
      "authenticated-join",
      hostFrames.find((x) => x.frame.kind === "join").peer === guestId,
    );
    check(
      "input-before-admission-refused",
      !guest.send(hostId, "input", { epoch: 1, seq: 1, input: { fire: true } }),
    );
    check(
      "fixture-admission-sent",
      host.send(guestId, "admission", { accepted: true, slot: 1, epoch: 1 }),
    );
    await until(() => guest.diagnostics().peers[0]?.admission === "admitted");
    check(
      "input-sent",
      guest.send(hostId, "input", {
        epoch: 1,
        seq: 1,
        input: { fire: true, thrust: [12, 34], aim: [56, 78] },
      }),
    );
    await until(() => hostFrames.some((x) => x.frame.kind === "input"));
    const received = hostFrames.find((x) => x.frame.kind === "input");
    check(
      "input-delivered",
      received.peer === guestId &&
        received.frame.payload.seq === 1 &&
        received.frame.payload.input.thrust[1] === 34,
    );
    check(
      "simulation-still-waiting",
      host.diagnostics().peers[0].simulation === "waiting" &&
        guest.diagnostics().peers[0].simulation === "waiting",
    );
  });
$("end").onclick = () =>
  action("Host end", async () => {
    await host.end();
    await until(() => guest.diagnostics().phase === "ended");
    await until(
      () =>
        host.diagnostics().pendingRequests === 0 &&
        guest.diagnostics().pendingRequests === 0,
    );
    check(
      "host-end-clean",
      created === closed &&
        !host.diagnostics().peers.length &&
        !guest.diagnostics().peers.length &&
        host.diagnostics().timers === 0 &&
        guest.diagnostics().timers === 0,
      { created, closed },
    );
  });
$("rejoin").onclick = () => action("Rejoin", () => connect(true));
$("dispose").onclick = () =>
  action("Dispose", async () => {
    await Promise.all([host?.dispose(), guest?.dispose()]);
    await until(
      () =>
        (!host || host.diagnostics().pendingRequests === 0) &&
        (!guest || guest.diagnostics().pendingRequests === 0),
    );
    check(
      "dispose-clean",
      created === closed &&
        (!host || host.diagnostics().timers === 0) &&
        (!guest || guest.diagnostics().timers === 0),
      { created, closed },
    );
  });
addEventListener("pagehide", () => {
  void host?.dispose();
  void guest?.dispose();
});
$("status").textContent =
  `Ready. Native RTC ${typeof RTCPeerConnection === "function" ? "available" : "unavailable"}. Prepared ${config.modules} actual source modules.`;
paint();
