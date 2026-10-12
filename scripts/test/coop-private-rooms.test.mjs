import assert from "node:assert/strict";
import { once } from "node:events";
import test from "node:test";
import { makeCoopServer } from "../../apps/coop/server.mjs";
import {
  LIMITS,
  PROTOCOL,
  SessionStore,
  normalizeRoomCode,
} from "../../apps/coop/store.mjs";

const origin = "http://private-rooms-test.localhost:4173";
const metadata = { seed: 0x76543210, protocol: PROTOCOL };
async function fixture(t, options = {}) {
  let now = 1_000_000;
  const store = new SessionStore({ now: () => now, ...options });
  const server = makeCoopServer({ store, allowedOrigins: [origin] });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const base = `http://127.0.0.1:${server.address().port}`;
  t.after(async () => {
    await server.coop.stop();
    assert.equal(server.listening, false);
    assert.equal(server.coop.pendingPolls(), 0);
    assert.equal(store.listenerCount("changed"), 0);
  });
  async function request(path, body, token, method = "POST") {
    const response = await fetch(base + path, {
      method,
      headers: {
        Origin: origin,
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(body === undefined ? {} : { "Content-Type": "application/json" }),
      },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    return { status: response.status, value: await response.json() };
  }
  const events = (p, after = 0, wait = 0) =>
    request(
      `/v1/events?after=${after}&wait_ms=${wait}`,
      undefined,
      p.token,
      "GET",
    );
  const create = async () => {
    const r = await request("/v1/sessions", metadata);
    assert.equal(r.status, 201);
    return r.value;
  };
  const join = async (h) => {
    const r = await request("/v1/join", { invite: h.invite });
    assert.equal(r.status, 201);
    return r.value;
  };
  const approve = (h, g, accept = true) =>
    request(
      "/v1/approval",
      { peer: g.participant.id, accept },
      h.participant.token,
    );
  return {
    store,
    server,
    request,
    events,
    create,
    join,
    approve,
    advance: (ms) => {
      now += ms;
    },
  };
}
function error(r, status, code) {
  assert.equal(r.status, status);
  assert.equal(r.value.error.code, code);
}
const bytes = (code) => Buffer.from(code, "hex");

test("eight-hex parser has exact ASCII paste rules and raw input bound", () => {
  for (const value of [
    "#a7c2-9f34",
    "A7C29F34",
    " \t# A7-C2\r\n9-F34\v\f ",
    "#" + " ".repeat(55) + "A7C29F34",
  ])
    assert.equal(normalizeRoomCode(value), "A7C29F34");
  for (const value of [
    null,
    123,
    {},
    "",
    "#A7C29F3",
    "A7C29F344",
    "##A7C29F34",
    "A7#C29F34",
    "Ａ7C29F34",
    "\u00a0A7C29F34",
    "A7C29F34_",
    "A7C2/9F34",
    "#" + " ".repeat(56) + "A7C29F34",
  ])
    assert.equal(normalizeRoomCode(value), null, JSON.stringify(value));
});

test("real HTTP pending guests cannot negotiate or become ready until atomic host approval", async (t) => {
  const f = await fixture(t),
    h = await f.create(),
    g = await f.join(h);
  assert.equal(g.participant.status, "pending");
  assert.equal(g.session.room_code, null);
  const requests = await f.events(h.participant);
  assert.deepEqual(
    requests.value.events.map((e) => e.type),
    ["join-request"],
  );
  assert.deepEqual(requests.value.events[0].peer, {
    id: g.participant.id,
    role: "guest",
    status: "pending",
  });
  assert.equal(
    requests.value.events[0].expires_at,
    1_000_000 + LIMITS.pendingMs,
  );
  assert.equal(requests.value.events[0].room_revision, 0);
  for (const [sender, receiver, kind, payload] of [
    [h, g, "offer", { sdp: "synthetic offer" }],
    [h, g, "ice", null],
    [g, h, "answer", { sdp: "synthetic answer" }],
    [g, h, "ice", null],
  ])
    error(
      await f.request(
        "/v1/signals",
        { to: receiver.participant.id, kind, payload },
        sender.participant.token,
      ),
      403,
      "approval_required",
    );
  error(
    await f.request("/v1/heartbeat", { ready: true }, g.participant.token),
    403,
    "approval_required",
  );
  let coherent = false;
  const observer = (id) => {
    if (id !== h.session.id) return;
    const host = f.store.events(h.participant.token, requests.value.cursor);
    if (host.events.some((e) => e.type === "peer-joined")) {
      const guest = f.store.events(g.participant.token, g.cursor);
      coherent = guest.events.some((e) => e.type === "join-approved");
    }
  };
  f.store.on("changed", observer);
  const accepted = await f.approve(h, g);
  f.store.off("changed", observer);
  assert.equal(accepted.status, 200);
  assert.equal(coherent, true);
  assert.equal(accepted.value.participant.token, undefined);
  const guest = await f.events(g.participant, g.cursor);
  assert.deepEqual(
    guest.value.events.map((e) => e.type),
    ["join-approved"],
  );
  assert.deepEqual(Object.keys(guest.value.events[0]).sort(), ["seq", "type"]);
  const host = await f.events(h.participant, requests.value.cursor);
  assert.deepEqual(
    host.value.events.map((e) => e.type),
    ["peer-joined"],
  );
  error(await f.approve(h, g), 409, "request_unavailable");
  assert.equal(
    (
      await f.request(
        "/v1/signals",
        {
          to: g.participant.id,
          kind: "offer",
          payload: { sdp: "synthetic offer" },
        },
        h.participant.token,
      )
    ).status,
    200,
  );
  assert.equal(
    (
      await f.request(
        "/v1/signals",
        {
          to: h.participant.id,
          kind: "answer",
          payload: { sdp: "synthetic answer" },
        },
        g.participant.token,
      )
    ).status,
    200,
  );
  assert.equal(
    (await f.request("/v1/heartbeat", { ready: true }, g.participant.token))
      .value.participant.status,
    "active",
  );
});

test("decline and absolute approval/negotiation deadlines deliver terminal outcomes and free seats", async (t) => {
  const f = await fixture(t, { limits: { players: 2 } }),
    h = await f.create();
  const declined = await f.join(h);
  assert.equal((await f.approve(h, declined, false)).status, 200);
  assert.equal(
    (await f.events(declined.participant, declined.cursor)).value.events[0]
      .reason,
    "join_declined",
  );
  error(await f.approve(h, declined), 409, "request_unavailable");
  const timed = await f.join(h);
  f.advance(19_999);
  assert.equal(
    (await f.request("/v1/heartbeat", {}, timed.participant.token)).status,
    200,
  );
  f.advance(1);
  assert.equal(
    (await f.events(timed.participant, timed.cursor)).value.events[0].reason,
    "approval_timeout",
  );
  const approved = await f.join(h);
  await f.request("/v1/heartbeat", {}, h.participant.token);
  f.advance(19_000);
  assert.equal((await f.approve(h, approved)).status, 200);
  f.advance(1_000);
  assert.equal(
    (await f.request("/v1/heartbeat", {}, approved.participant.token)).status,
    200,
  );
  await f.request("/v1/heartbeat", {}, h.participant.token);
  f.advance(19_000);
  const ended = await f.events(approved.participant, approved.cursor);
  assert.equal(ended.value.events.at(-1).reason, "reservation_timeout");
  assert.equal(ended.value.session.state, "open");
  assert.equal((await f.join(h)).participant.status, "pending");
});

test("same-room code rotation and locking preserve approved peers while retiring every unapproved request", async (t) => {
  const f = await fixture(t),
    h = await f.create(),
    active = await f.join(h);
  await f.approve(h, active);
  await f.request("/v1/heartbeat", { ready: true }, active.participant.token);
  const negotiating = await f.join(h);
  await f.approve(h, negotiating);
  const pending = await f.join(h);
  const rotated = await f.request("/v1/code", {}, h.participant.token);
  assert.equal(rotated.status, 200);
  assert.equal(rotated.value.session.id, h.session.id);
  assert.equal(rotated.value.session.host_id, h.session.host_id);
  assert.equal(rotated.value.session.seed, h.session.seed);
  assert.equal(rotated.value.session.expires_at, h.session.expires_at);
  assert.notEqual(rotated.value.invite, h.invite);
  assert.equal(rotated.value.session.room_revision, 1);
  assert.equal(
    (await f.events(pending.participant, pending.cursor)).value.events[0]
      .reason,
    "code_replaced",
  );
  assert.equal(
    (await f.events(pending.participant, pending.cursor)).value.session
      .room_code,
    null,
  );
  error(await f.request("/v1/join", { invite: h.invite }), 410, "stale_invite");
  const lockPending = await f.join({ invite: rotated.value.invite });
  const locked = await f.request(
    "/v1/joining",
    { locked: true },
    h.participant.token,
  );
  assert.equal(locked.status, 200);
  assert.equal(locked.value.session.joining_locked, true);
  assert.equal(locked.value.session.room_revision, 2);
  assert.equal(locked.value.session.room_code, rotated.value.invite);
  assert.equal(
    (await f.events(lockPending.participant, lockPending.cursor)).value
      .events[0].reason,
    "joining_locked",
  );
  error(
    await f.request("/v1/join", { invite: rotated.value.invite }),
    409,
    "joining_locked",
  );
  for (const guest of [active, negotiating]) {
    const view = await f.events(guest.participant, guest.cursor);
    assert.equal(view.value.session.state, "open");
    assert.equal(
      view.value.events.some((e) => e.type === "participant-left"),
      false,
    );
    assert.equal(
      (
        await f.request(
          "/v1/signals",
          { to: h.participant.id, kind: "ice", payload: null },
          guest.participant.token,
        )
      ).status,
      200,
    );
  }
  const latest = (await f.request("/v1/heartbeat", {}, h.participant.token))
    .value.session;
  assert.equal(latest.room_revision, 2);
  assert.equal(latest.room_code, rotated.value.invite);
  assert.equal(
    (await f.request("/v1/joining", { locked: false }, h.participant.token))
      .value.session.room_revision,
    3,
  );
  assert.equal(
    (await f.join({ invite: rotated.value.invite })).participant.status,
    "pending",
  );
});

test("room codes and guest credentials cannot administer; exact bodies and admin attempts are bounded before parsing", async (t) => {
  const f = await fixture(t, { limits: { adminPerMinute: 2 } }),
    h = await f.create(),
    g = await f.join(h);
  for (const [path, body] of [
    ["/v1/code", {}],
    ["/v1/joining", { locked: true }],
    ["/v1/approval", { peer: g.participant.id, accept: true }],
  ]) {
    error(await f.request(path, body, g.participant.token), 403, "host_only");
    error(await f.request(path, body, h.invite), 401, "unauthorized");
    error(
      await f.request(
        path,
        body,
        h.invite.replace(/[#-]/g, "").toLowerCase().repeat(8),
      ),
      401,
      "unauthorized",
    );
  }
  error(
    await f.request("/v1/code", { seed: 7 }, h.participant.token),
    400,
    "invalid_body",
  );
  assert.equal(
    (await f.request("/v1/joining", { locked: false }, h.participant.token))
      .status,
    200,
  );
  error(
    await f.request("/v1/code", {}, h.participant.token),
    429,
    "rate_limit",
  );
  const session = (await f.events(h.participant)).value.session;
  assert.equal(session.room_code, h.invite);
  assert.equal(session.room_revision, 0);
  assert.deepEqual(Object.keys(f.store.diagnostics()).sort(), [
    "credentials",
    "rate_clients",
    "reserved_codes",
    "sessions",
  ]);
  assert.equal(JSON.stringify(session).includes(h.participant.token), false);
  f.advance(60_000); // Expired host invalidates codes rather than reviving administration.
  error(
    await f.request("/v1/code", {}, h.participant.token),
    410,
    "session_ended",
  );
});

test("crypto allocation checks active and retired collisions with finite attempts and no partial rotation or create", async (t) => {
  let mode = "sequence",
    n = 0;
  const sequence = [
    "A7C29F34",
    "A7C29F34",
    "B8D30A45",
    "A7C29F34",
    "B8D30A45",
    "C9E41B56",
  ];
  const f = await fixture(t, {
    limits: { codeAttempts: 3 },
    codeBytes: () => bytes(mode === "stuck" ? "A7C29F34" : sequence[n++]),
  });
  const h = await f.create();
  const second = await f.create();
  assert.equal(second.invite, "#B8D3-0A45");
  const rotated = await f.request("/v1/code", {}, h.participant.token);
  assert.equal(rotated.value.invite, "#C9E4-1B56");
  assert.equal(f.store.diagnostics().reserved_codes, 3);
  const pending = await f.join({ invite: rotated.value.invite });
  mode = "stuck";
  error(
    await f.request("/v1/code", {}, h.participant.token),
    503,
    "code_capacity",
  );
  assert.equal(
    (await f.events(pending.participant, pending.cursor)).value.participant
      .status,
    "pending",
  );
  const current = (await f.events(h.participant)).value.session;
  assert.equal(current.room_revision, 1);
  assert.equal(current.room_code, rotated.value.invite);
  f.advance(60_000); // Clear anonymous create rate, and retained ended rooms remain reserved.
  const before = f.store.diagnostics();
  error(await f.request("/v1/sessions", metadata), 503, "code_capacity");
  assert.deepEqual(f.store.diagnostics(), { ...before, rate_clients: 1 });
});

test("host queue preflight refuses approval, rotation and lock without revoking half a request", async (t) => {
  const f = await fixture(t, { limits: { queueEvents: 1, players: 2 } }),
    h = await f.create(),
    g = await f.join(h);
  for (const [path, body] of [
    ["/v1/approval", { peer: g.participant.id, accept: true }],
    ["/v1/approval", { peer: g.participant.id, accept: false }],
    ["/v1/code", {}],
    ["/v1/joining", { locked: true }],
  ])
    error(await f.request(path, body, h.participant.token), 429, "queue_full");
  const guest = await f.events(g.participant, g.cursor);
  assert.deepEqual(guest.value.events, []);
  assert.equal(guest.value.participant.status, "pending");
  assert.equal(guest.value.session.room_revision, 0);
  assert.equal(guest.value.session.room_code, null);
  assert.equal(guest.value.session.joining_locked, false);
  error(
    await f.request("/v1/heartbeat", { ready: true }, g.participant.token),
    403,
    "approval_required",
  );
  error(await f.request("/v1/join", { invite: h.invite }), 409, "session_full");
  const host = await f.events(h.participant);
  await f.events(h.participant, host.value.cursor);
  assert.equal((await f.approve(h, g)).status, 200);
});

test("rotation quota and room expiry are finite; retired reservations release only after room retention", () => {
  let now = 1000,
    next = 0;
  const store = new SessionStore({
    now: () => now,
    limits: { codesPerSession: 3, sessionMs: 100 },
    codeBytes: () => bytes((++next).toString(16).padStart(8, "0")),
  });
  const h = store.create(metadata, store.clientKey("owner"));
  now += 50;
  assert.equal(store.join({ invite: h.invite }).participant.status, "pending");
  const rotated = store.rotateCode(h.participant.token);
  assert.equal(rotated.session.id, h.session.id);
  assert.equal(rotated.session.expires_at, h.session.expires_at);
  assert.equal(
    store.join({ invite: rotated.invite }).participant.status,
    "pending",
  );
  store.rotateCode(h.participant.token);
  assert.throws(
    () => store.rotateCode(h.participant.token),
    (e) => e.code === "code_quota",
  );
  assert.equal(store.diagnostics().reserved_codes, 3);
  now += 50; // Rotation never extends the room's original absolute expiry.
  assert.throws(
    () => store.join({ invite: rotated.invite }),
    (e) => e.code === "session_ended",
  );
  now += LIMITS.retentionMs - 1;
  assert.equal(store.diagnostics().reserved_codes, 3);
  now++;
  assert.equal(store.diagnostics().reserved_codes, 0);
  assert.equal(store.diagnostics().credentials, 0);
});

test("current code remains valid beyond former ten-minute invite timer while host lives, then ends with room", async (t) => {
  const f = await fixture(t),
    h = await f.create();
  for (let i = 0; i < 31; i++) {
    f.advance(20_000);
    assert.equal(
      (await f.request("/v1/heartbeat", {}, h.participant.token)).status,
      200,
    );
  }
  const live = await f.join(h);
  assert.equal(live.session.id, h.session.id);
  assert.equal(live.session.expires_at, h.session.expires_at);
  assert.equal(live.session.room_revision, 0);
  f.advance(LIMITS.hostMs);
  error(
    await f.request("/v1/join", { invite: h.invite }),
    410,
    "session_ended",
  );
  assert.equal(
    (await f.events(live.participant, live.cursor)).value.events[0].reason,
    "host_timeout",
  );
});

test("unchanged unlocked policy preserves request birth revision; every new request uses the rotated policy", async (t) => {
  const f = await fixture(t),
    h = await f.create(),
    pending = await f.join(h);
  const noop = await f.request(
    "/v1/joining",
    { locked: false },
    h.participant.token,
  );
  assert.equal(noop.value.session.room_revision, 0);
  const replay = await f.events(h.participant);
  assert.equal(replay.value.events[0].room_revision, 0);
  assert.equal((await f.approve(h, pending)).status, 200);
  const rotated = await f.request("/v1/code", {}, h.participant.token);
  const next = await f.join({ invite: rotated.value.invite });
  const all = await f.events(h.participant);
  const request = all.value.events.find(
    (e) => e.type === "join-request" && e.peer.id === next.participant.id,
  );
  assert.equal(request.room_revision, rotated.value.session.room_revision);
  assert.equal(all.value.session.room_code, rotated.value.invite);
});

test("aggregate cancellation preflight preserves both pending requests and approved peers on failure", async (t) => {
  const f = await fixture(t, { limits: { queueEvents: 4 } }),
    h = await f.create(),
    active = await f.join(h);
  await f.approve(h, active);
  const first = await f.join(h),
    second = await f.join(h); // Four unacked host events.
  for (const [path, body] of [
    ["/v1/code", {}],
    ["/v1/joining", { locked: true }],
  ])
    error(await f.request(path, body, h.participant.token), 429, "queue_full");
  for (const g of [first, second, active]) {
    const view = await f.events(g.participant, g.cursor);
    assert.equal(view.value.session.state, "open");
    assert.equal(view.value.participant.status, "pending");
    assert.equal(
      view.value.events.some((e) => e.type === "participant-left"),
      false,
    );
  }
  const before = await f.events(h.participant);
  assert.equal(before.value.session.room_revision, 0);
  assert.equal(before.value.session.room_code, h.invite);
  await f.events(h.participant, before.value.cursor);
  const waiting = [first, second].map((g) =>
    f.events(g.participant, g.cursor, 1000),
  );
  assert.equal(
    (await f.request("/v1/joining", { locked: true }, h.participant.token))
      .status,
    200,
  );
  for (const notice of await Promise.all(waiting))
    assert.equal(notice.value.events[0].reason, "joining_locked");
  assert.equal(
    (await f.events(active.participant, active.cursor)).value.session.state,
    "open",
  );
});

test("admin schema, cross-room and resolved IDs reject without policy or admission changes", async (t) => {
  const f = await fixture(t),
    h = await f.create(),
    other = await f.create(),
    g = await f.join(h),
    foreign = await f.join(other);
  for (const body of [
    {},
    { locked: null },
    { locked: 1 },
    { locked: "true" },
    { locked: false, extra: true },
  ])
    assert.equal(
      (await f.request("/v1/joining", body, h.participant.token)).status,
      400,
    );
  for (const body of [
    {},
    { peer: g.participant.id },
    { peer: g.participant.id, accept: null },
    { peer: g.participant.id, accept: 1 },
    { peer: g.participant.id, accept: false, extra: true },
  ])
    assert.equal(
      (await f.request("/v1/approval", body, h.participant.token)).status,
      400,
    );
  for (const peer of [h.participant.id, foreign.participant.id, "0".repeat(32)])
    error(
      await f.request(
        "/v1/approval",
        { peer, accept: true },
        h.participant.token,
      ),
      409,
      "request_unavailable",
    );
  assert.equal((await f.events(h.participant)).value.session.room_revision, 0);
  assert.equal(
    (await f.events(g.participant, g.cursor)).value.participant.status,
    "pending",
  );
  assert.equal((await f.approve(h, g, false)).value.participant.status, "left");
});
