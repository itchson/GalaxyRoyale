import assert from "node:assert/strict";
import { once } from "node:events";
import { request as httpRequest } from "node:http";
import test from "node:test";
import { setTimeout as delay } from "node:timers/promises";
import { makeCoopServer } from "../../apps/coop/server.mjs";
import { LIMITS, PROTOCOL, SessionStore } from "../../apps/coop/store.mjs";

const ORIGIN = "http://coop-test.localhost:4173";
const metadata = { seed: 0x1234abcd, protocol: PROTOCOL };
// Synthetic transport data. These tests do not create an RTC peer or game actor.
const offer = { sdp: "v=0\r\ns=declared signaling fixture\r\n" };
const candidate = {
  candidate: "candidate:synthetic-test",
  sdpMid: "0",
  sdpMLineIndex: 0,
};

async function fixture(t, options = {}) {
  let time = 1_000_000;
  const store = new SessionStore({ now: () => time, ...options });
  const server = makeCoopServer({ store, allowedOrigins: [ORIGIN] });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const base = `http://127.0.0.1:${server.address().port}`;
  const baselines = new Map();
  t.after(async () => {
    await server.coop.stop();
    assert.equal(server.listening, false);
    assert.equal(server.coop.pendingPolls(), 0);
    assert.equal(store.listenerCount("changed"), 0);
  });
  const request = async (
    path,
    {
      token,
      body,
      method = body === undefined ? "GET" : "POST",
      origin = ORIGIN,
      headers = {},
      signal,
      raw,
    } = {},
  ) => {
    const response = await fetch(base + path, {
      method,
      signal,
      headers: {
        ...(origin === null ? {} : { Origin: origin }),
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(body !== undefined || raw !== undefined
          ? { "Content-Type": "application/json" }
          : {}),
        ...headers,
      },
      body: raw ?? (body === undefined ? undefined : JSON.stringify(body)),
    });
    return {
      status: response.status,
      headers: response.headers,
      value: response.status === 204 ? null : await response.json(),
    };
  };
  const create = async (body = metadata) => {
    const result = await request("/v1/sessions", { body });
    assert.equal(result.status, 201);
    return result.value;
  };
  const approve = async (host, guest) => {
    // Process and acknowledge the request before adding negotiation notices.
    const requests = await events(host.participant);
    await events(host.participant, requests.value.cursor);
    baselines.set(host.participant.id, requests.value.cursor);
    const accepted = await request("/v1/approval", {
      token: host.participant.token,
      body: { peer: guest.participant.id, accept: true },
    });
    assert.equal(accepted.status, 200);
    const notice = await events(guest.participant, guest.cursor);
    assert.deepEqual(
      notice.value.events.map((e) => e.type),
      ["join-approved"],
    );
    guest.cursor = notice.value.cursor;
    await events(guest.participant, guest.cursor);
    baselines.set(guest.participant.id, guest.cursor);
  };
  const join = async (host, approved = true) => {
    const result = await request("/v1/join", { body: { invite: host.invite } });
    assert.equal(result.status, 201);
    if (approved) await approve(host, result.value);
    return result.value;
  };
  const events = (participant, after = null, wait = 0, signal) =>
    request(
      `/v1/events?after=${after ?? baselines.get(participant.id) ?? 0}&wait_ms=${wait}`,
      {
        token: participant.token,
        signal,
      },
    );
  return {
    store,
    server,
    base,
    request,
    create,
    join,
    approve,
    events,
    advance: (ms) => {
      time += ms;
    },
  };
}
async function waitFor(predicate) {
  for (let i = 0; i < 100; i++) {
    if (predicate()) return;
    await delay(5);
  }
  assert.fail("Expected bounded asynchronous condition was not reached.");
}
function error(result, status, code) {
  assert.equal(result.status, status);
  assert.equal(result.value.error.code, code);
}

test("HTTP exact Origin, CORS and fixed routes reject untrusted or forged requests", async (t) => {
  const f = await fixture(t);
  assert.equal((await f.request("/health", { origin: null })).status, 200);
  error(
    await f.request("/v1/sessions", { body: metadata, origin: null }),
    403,
    "origin",
  );
  error(
    await f.request("/v1/sessions", {
      body: metadata,
      origin: ORIGIN + ".evil",
    }),
    403,
    "origin",
  );
  error(
    await f.request("/v1/sessions", { body: { ...metadata, role: "host" } }),
    400,
    "invalid_body",
  );
  error(
    await f.request("/v1/sessions", { body: { ...metadata, profile: {} } }),
    400,
    "invalid_body",
  );
  const preflight = await f.request("/v1/events", {
    method: "OPTIONS",
    headers: {
      "Access-Control-Request-Method": "GET",
      "Access-Control-Request-Headers": "authorization, content-type",
    },
  });
  assert.equal(preflight.status, 204);
  assert.equal(preflight.headers.get("access-control-allow-origin"), ORIGIN);
  assert.equal(
    preflight.headers.has("access-control-allow-credentials"),
    false,
  );
  error(
    await f.request("/v1/events", {
      method: "OPTIONS",
      headers: {
        "Access-Control-Request-Method": "GET",
        "Access-Control-Request-Headers": "x-forward-url",
      },
    }),
    403,
    "cors_headers",
  );
  error(await f.request("/forward?url=https://example.invalid"), 404, "route");
  assert.throws(() => makeCoopServer({ allowedOrigins: [ORIGIN + "/path"] }));
  assert.throws(() => new SessionStore({ limits: { players: 9 } }));
  assert.throws(() => new SessionStore({ limits: { unknown: 1 } }));
});

test("short room lookup codes reject malformed, expiry and service restart changes", async (t) => {
  const f = await fixture(t, { limits: { inviteMs: 100 } });
  const host = await f.create();
  assert.match(host.invite, /^#[0-9A-F]{4}-[0-9A-F]{4}$/);
  assert.equal(host.session.room_code, host.invite);
  assert.equal(host.session.room_revision, 0);
  assert.equal(host.session.joining_locked, false);
  assert.match(host.participant.token, /^[0-9a-f]{64}$/);
  assert.match(host.participant.id, /^[0-9a-f]{32}$/);
  assert.equal(host.session.seed, metadata.seed);
  const encoded = host.invite.replace(/[#-]/g, "");
  for (const copied of [
    host.invite,
    encoded,
    " \n" + host.invite.toLowerCase() + "\t",
    " " + encoded.toLowerCase() + " ",
  ])
    assert.equal(
      (await f.request("/v1/join", { body: { invite: copied } })).status,
      201,
    );
  for (const malformed of [
    "##" + encoded,
    "#" + encoded.slice(1),
    "#" + "A".repeat(9),
    " ".repeat(65),
    "#" + encoded + "!",
    "\u00a0" + encoded,
  ])
    error(
      await f.request("/v1/join", { body: { invite: malformed } }),
      401,
      "invalid_invite",
    );
  const restarted = await fixture(t);
  error(
    await restarted.request("/v1/join", { body: { invite: host.invite } }),
    401,
    "invalid_invite",
  );
  error(await restarted.events(host.participant), 401, "unauthorized");
  f.advance(100);
  error(
    await f.request("/v1/join", { body: { invite: host.invite } }),
    410,
    "invite_expired",
  );
});

test("concurrent HTTP admission reserves only host plus seven, including all pending guests", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const results = await Promise.all(
    Array.from({ length: 8 }, () =>
      f.request("/v1/join", { body: { invite: host.invite } }),
    ),
  );
  assert.equal(results.filter((r) => r.status === 201).length, 7);
  error(
    results.find((r) => r.status !== 201),
    409,
    "session_full",
  );
  const guests = results.filter((r) => r.status === 201).map((r) => r.value);
  assert.equal(new Set([host, ...guests].map((r) => r.participant.id)).size, 8);
  assert.ok(guests.every((r) => r.participant.status === "pending"));
  assert.equal(f.store.diagnostics().credentials, 8);
  await f.approve(host, guests[0]);
  const ready = await f.request("/v1/heartbeat", {
    token: guests[0].participant.token,
    body: { ready: true },
  });
  assert.equal(ready.value.participant.status, "active");
  const notices = await f.events(host.participant);
  assert.equal(
    notices.value.events.filter((e) => e.type === "peer-joined").length,
    1,
  );
  assert.equal(
    notices.value.events.filter((e) => e.type === "peer-ready").length,
    1,
  );
  assert.equal(notices.value.participant.token, undefined);
});

test("reservations and active guests expire separately; heartbeat does not extend pending admission", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const guest = await f.join(host, false);
  f.advance(19_999);
  assert.equal(
    (
      await f.request("/v1/heartbeat", {
        token: guest.participant.token,
        body: {},
      })
    ).status,
    200,
  );
  f.advance(1);
  error(
    await f.request("/v1/heartbeat", {
      token: guest.participant.token,
      body: { ready: true },
    }),
    410,
    "participant_left",
  );
  assert.equal(
    (await f.events(guest.participant)).value.events[0].reason,
    "approval_timeout",
  );
  const active = await f.join(host);
  await f.request("/v1/heartbeat", { token: host.participant.token, body: {} });
  await f.request("/v1/heartbeat", {
    token: active.participant.token,
    body: { ready: true },
  });
  f.advance(25_000);
  await f.request("/v1/heartbeat", { token: host.participant.token, body: {} });
  f.advance(20_000);
  assert.equal(
    (await f.events(active.participant)).value.events[0].reason,
    "guest_timeout",
  );
  assert.equal((await f.events(host.participant)).value.session.state, "open");
  assert.equal((await f.join(host)).participant.status, "pending");
});

test("guest cannot end, replace, forge metadata, address strangers or bypass offer roles", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const a = await f.join(host);
  const b = await f.join(host);
  const other = await f.create({ seed: 9, protocol: PROTOCOL });
  error(
    await f.request("/v1/end", { token: a.participant.token, body: {} }),
    403,
    "host_only",
  );
  error(
    await f.request("/v1/replace", {
      token: a.participant.token,
      body: metadata,
    }),
    403,
    "host_only",
  );
  error(
    await f.request("/v1/heartbeat", {
      token: a.participant.token,
      body: { seed: 99, role: "host" },
    }),
    400,
    "invalid_body",
  );
  for (const peer of [a.participant, b.participant, other.participant])
    error(
      await f.request("/v1/signals", {
        token: a.participant.token,
        body: { to: peer.id, kind: "ice", payload: null },
      }),
      403,
      "invalid_peer",
    );
  error(
    await f.request("/v1/signals", {
      token: a.participant.token,
      body: { to: host.participant.id, kind: "offer", payload: offer },
    }),
    403,
    "signal_role",
  );
  error(
    await f.request("/v1/signals", {
      token: a.participant.token,
      body: { to: host.participant.id, kind: "answer", payload: offer },
    }),
    409,
    "no_offer",
  );
  assert.equal(
    (
      await f.request("/v1/signals", {
        token: host.participant.token,
        body: { to: a.participant.id, kind: "offer", payload: offer },
      })
    ).status,
    200,
  );
  assert.equal(
    (
      await f.request("/v1/signals", {
        token: a.participant.token,
        body: { to: host.participant.id, kind: "answer", payload: offer },
      })
    ).status,
    200,
  );
  error(
    await f.request("/v1/signals", {
      token: a.participant.token,
      body: { to: host.participant.id, kind: "answer", payload: offer },
    }),
    409,
    "no_offer",
  );
  assert.equal(
    (await f.events(host.participant)).value.session.seed,
    metadata.seed,
  );
});

test("host end publishes every final long-poll notice atomically, even with guests pending", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const guests = await Promise.all(
    Array.from({ length: 7 }, () => f.join(host)),
  );
  const polls = guests.map((g) => f.events(g.participant, g.cursor, 1000));
  await waitFor(() => f.server.coop.pendingPolls() === 7);
  assert.equal(
    (await f.request("/v1/end", { token: host.participant.token, body: {} }))
      .status,
    200,
  );
  for (const result of await Promise.all(polls)) {
    assert.equal(result.status, 200);
    assert.equal(result.value.session.state, "ended");
    assert.deepEqual(
      result.value.events.map((e) => [e.type, e.reason]),
      [["session-ended", "host_end"]],
    );
  }
  error(
    await f.request("/v1/join", { body: { invite: host.invite } }),
    410,
    "session_ended",
  );
  error(
    await f.request("/v1/heartbeat", {
      token: guests[0].participant.token,
      body: {},
    }),
    410,
    "session_ended",
  );
});

test("host leave, timeout and absolute expiry retain a bounded terminal notice then erase credentials", async (t) => {
  for (const reason of ["host_left", "host_timeout", "expired"]) {
    const f = await fixture(
      t,
      reason === "expired" ? { limits: { sessionMs: 100 } } : {},
    );
    const host = await f.create();
    const guest = await f.join(host);
    if (reason === "host_left")
      await f.request("/v1/leave", { token: host.participant.token, body: {} });
    else f.advance(reason === "expired" ? 100 : 30_000);
    const ended = await f.events(guest.participant, guest.cursor);
    assert.equal(ended.value.events[0].reason, reason);
    f.advance(60_000);
    error(await f.events(guest.participant), 401, "unauthorized");
    assert.equal(f.store.diagnostics().sessions, 0);
    assert.equal(f.store.diagnostics().credentials, 0);
  }
});

test("replacement ends only the previous session and failed replacement preserves it", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const guest = await f.join(host);
  error(
    await f.request("/v1/replace", {
      token: host.participant.token,
      body: { seed: -1, protocol: PROTOCOL },
    }),
    400,
    "invalid_metadata",
  );
  assert.equal((await f.events(host.participant)).value.session.state, "open");
  await f.request("/v1/heartbeat", {
    token: guest.participant.token,
    body: { ready: true },
  });
  for (let i = 0; i < 3; i++) {
    f.advance(20_000);
    assert.equal(
      (
        await f.request("/v1/heartbeat", {
          token: host.participant.token,
          body: {},
        })
      ).status,
      200,
    );
    assert.equal(
      (
        await f.request("/v1/heartbeat", {
          token: guest.participant.token,
          body: {},
        })
      ).status,
      200,
    );
  }
  const next = await f.request("/v1/replace", {
    token: host.participant.token,
    body: { seed: 7, protocol: PROTOCOL },
  });
  assert.equal(next.status, 201);
  assert.notEqual(next.value.session.id, host.session.id);
  assert.notEqual(next.value.participant.token, host.participant.token);
  assert.equal(next.value.session.seed, 7);
  assert.equal(
    (await f.events(guest.participant)).value.events[0].reason,
    "replaced",
  );
  const limited = await fixture(t, { limits: { sessions: 1 } });
  const first = await limited.create();
  error(
    await limited.request("/v1/replace", {
      token: first.participant.token,
      body: metadata,
    }),
    429,
    "session_quota",
  );
  assert.equal(
    (await limited.events(first.participant)).value.session.state,
    "open",
  );
});

test("authenticated replacement shares create rates across new bearer credentials", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const replaced = await f.request("/v1/replace", {
    token: host.participant.token,
    body: metadata,
  });
  assert.equal(replaced.status, 201);
  error(
    await f.request("/v1/replace", {
      token: replaced.value.participant.token,
      body: metadata,
    }),
    429,
    "rate_limit",
  );
  assert.equal(
    (await f.events(replaced.value.participant)).value.session.state,
    "open",
  );
  assert.equal(f.store.diagnostics().sessions, 2);
  const global = await fixture(t, { limits: { createPerMinute: 1 } });
  const first = await global.create();
  error(
    await global.request("/v1/replace", {
      token: first.participant.token,
      body: metadata,
    }),
    429,
    "rate_limit",
  );
  assert.equal(
    (await global.events(first.participant)).value.session.state,
    "open",
  );
});

test("event replay and acknowledgment enforce queue capacity without silently dropping signals", async (t) => {
  const f = await fixture(t, { limits: { queueEvents: 4 } });
  const host = await f.create();
  const guest = await f.join(host);
  const send = () =>
    f.request("/v1/signals", {
      token: host.participant.token,
      body: { to: guest.participant.id, kind: "ice", payload: candidate },
    });
  for (let n = 0; n < 4; n++) assert.equal((await send()).status, 200);
  error(await send(), 429, "queue_full");
  const first = await f.events(guest.participant, guest.cursor);
  assert.equal(first.value.events.length, 4);
  assert.deepEqual(
    (await f.events(guest.participant, guest.cursor)).value,
    first.value,
  );
  assert.equal(
    (await f.events(guest.participant, first.value.cursor)).value.events.length,
    0,
  );
  error(await f.events(guest.participant, guest.cursor), 409, "cursor_expired");
  assert.equal((await send()).status, 200);
  assert.equal(
    (await f.events(guest.participant, first.value.cursor)).value.events.length,
    1,
  );
  error(await f.events(guest.participant, 999), 400, "invalid_cursor");
});

test("body, SDP and ICE byte bounds and schemas reject oversized or forwarding payloads", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const guest = await f.join(host);
  const send = (kind, payload) =>
    f.request("/v1/signals", {
      token: host.participant.token,
      body: { to: guest.participant.id, kind, payload },
    });
  assert.equal(
    (await send("offer", { sdp: "a".repeat(LIMITS.sdpBytes) })).status,
    200,
  );
  error(
    await send("offer", { sdp: "é".repeat(LIMITS.sdpBytes / 2 + 1) }),
    400,
    "invalid_sdp",
  );
  assert.equal(
    (await send("ice", { candidate: "a".repeat(LIMITS.iceBytes) })).status,
    200,
  );
  error(
    await send("ice", { candidate: "a".repeat(LIMITS.iceBytes + 1) }),
    400,
    "invalid_ice",
  );
  error(
    await send("ice", { candidate: "x", url: "https://example.invalid" }),
    400,
    "invalid_body",
  );
  error(
    await send("ice", { candidate: "x", sdpMLineIndex: 33 }),
    400,
    "invalid_ice",
  );
  error(await send("fetch", {}), 400, "invalid_signal");
  error(
    await f.request("/v1/signals", {
      token: host.participant.token,
      method: "POST",
      raw: "x".repeat(LIMITS.bodyBytes + 1),
    }),
    413,
    "body_limit",
  );
  error(
    await f.request("/v1/signals", {
      token: host.participant.token,
      method: "POST",
      raw: "{broken",
    }),
    400,
    "invalid_json",
  );
  error(
    await f.request("/v1/signals", {
      token: host.participant.token,
      body: {},
      headers: { "Content-Type": "text/plain" },
    }),
    415,
    "content_type",
  );
});

test("unauthenticated failures consume bounded admission rates; session and client quotas are finite", async (t) => {
  const f = await fixture(t);
  for (let i = 0; i < 2; i++)
    error(
      await f.request("/v1/sessions", {
        body: { seed: -1, protocol: PROTOCOL },
      }),
      400,
      "invalid_metadata",
    );
  error(await f.request("/v1/sessions", { body: metadata }), 429, "rate_limit");
  f.advance(60_000);
  const host = await f.create();
  for (let i = 0; i < 20; i++)
    error(
      await f.request("/v1/join", { body: { invite: "invalid" } }),
      401,
      "invalid_invite",
    );
  error(
    await f.request("/v1/join", { body: { invite: host.invite } }),
    429,
    "rate_limit",
  );
  const limited = await fixture(t, { limits: { clientSessions: 1 } });
  await limited.create();
  error(
    await limited.request("/v1/sessions", { body: metadata }),
    429,
    "client_sessions",
  );
  const store = new SessionStore({ limits: { rateClients: 2 }, now: () => 1 });
  store.admitAnonymous("join", store.clientKey("a"));
  store.admitAnonymous("join", store.clientKey("b"));
  assert.throws(
    () => store.admitAnonymous("join", store.clientKey("c")),
    (e) => e.code === "rate_clients",
  );
  const global = new SessionStore({
    limits: { createPerMinute: 2 },
    now: () => 1,
  });
  global.admitAnonymous("create", global.clientKey("a"));
  global.admitAnonymous("create", global.clientKey("b"));
  assert.throws(
    () => global.admitAnonymous("create", global.clientKey("c")),
    (e) => e.code === "rate_limit",
  );
});

test("per-sender and per-session signaling rates refuse excess without adding events", async (t) => {
  for (const limits of [
    { signalPerMinute: 2 },
    { sessionSignalsPerMinute: 2 },
  ]) {
    const f = await fixture(t, { limits });
    const host = await f.create();
    const guest = await f.join(host);
    const send = () =>
      f.request("/v1/signals", {
        token: host.participant.token,
        body: { to: guest.participant.id, kind: "ice", payload: null },
      });
    assert.equal((await send()).status, 200);
    assert.equal((await send()).status, 200);
    error(await send(), 429, "rate_limit");
    assert.equal((await f.events(guest.participant)).value.events.length, 2);
  }
});

test("one bounded long poll per participant, abort cleanup and finite timeout", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const guest = await f.join(host);
  const controller = new AbortController();
  const pending = f.events(
    guest.participant,
    guest.cursor,
    1000,
    controller.signal,
  );
  // Install rejection observer before intentionally aborting.
  const aborted = assert.rejects(pending, (e) => e.name === "AbortError");
  await waitFor(() => f.server.coop.pendingPolls() === 1);
  error(
    await f.events(guest.participant, guest.cursor, 1000),
    409,
    "concurrent_poll",
  );
  controller.abort();
  await aborted;
  await waitFor(() => f.server.coop.pendingPolls() === 0);
  assert.equal(f.store.listenerCount("changed"), 0);
  const timed = await f.events(guest.participant, guest.cursor, 25);
  assert.equal(timed.status, 200);
  assert.deepEqual(timed.value.events, []);
  for (const query of [
    "after=-1",
    "after=0&after=0",
    "wait_ms=20001",
    "after=01",
    "url=http://example.invalid",
  ])
    assert.equal(
      (
        await f.request("/v1/events?" + query, {
          token: guest.participant.token,
        })
      ).status,
      400,
    );
});

test("service stop resolves existing polls and closes every test listener", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const guest = await f.join(host);
  const pending = f.events(guest.participant, guest.cursor, 1000);
  await waitFor(() => f.server.coop.pendingPolls() === 1);
  const stopped = f.server.coop.stop();
  const response = await pending;
  await stopped;
  assert.equal(response.value.events[0].reason, "service_shutdown");
  assert.equal(f.server.listening, false);
  assert.equal(f.server.coop.pendingPolls(), 0);
  assert.throws(
    () => f.store.create(metadata, f.store.clientKey("late request")),
    (e) => e.code === "service_closed",
  );
  assert.throws(
    () => f.store.join({ invite: host.invite }),
    (e) => e.code === "service_closed",
  );
});

test("pure queued transport is cloned and retained departed credentials stay bounded", () => {
  let now = 1000;
  const store = new SessionStore({
    now: () => now,
    limits: { retainedParticipants: 2 },
  });
  const host = store.create(metadata, store.clientKey("test"));
  const guest = store.join({ invite: host.invite });
  store.approval(host.participant.token, {
    peer: guest.participant.id,
    accept: true,
  });
  const approved = store.events(guest.participant.token, guest.cursor);
  guest.cursor = approved.cursor;
  store.events(guest.participant.token, guest.cursor);
  const payload = { candidate: "original" };
  store.signal(host.participant.token, {
    to: guest.participant.id,
    kind: "ice",
    payload,
  });
  payload.candidate = "mutated";
  assert.equal(
    store.events(guest.participant.token, guest.cursor).events[0].payload
      .candidate,
    "original",
  );
  store.leave(guest.participant.token);
  assert.throws(
    () => store.join({ invite: host.invite }),
    (e) => e.code === "participant_quota",
  );
  now += 59_999; // Maintain the host with legal monotonic checkpoints.
  // A backwards clock cannot postpone the already observed timeout.
  assert.equal(
    store.events(host.participant.token, 0).session.reason,
    "host_timeout",
  );
  now = 1000;
  assert.equal(store.events(host.participant.token, 0).session.state, "ended");
});

test("byte quota and membership overflow preserve terminal notices instead of orphaning peers", async (t) => {
  const f = await fixture(t, { limits: { queueBytes: 256 } });
  const host = await f.create();
  const guest = await f.join(host);
  error(
    await f.request("/v1/signals", {
      token: host.participant.token,
      body: {
        to: guest.participant.id,
        kind: "offer",
        payload: { sdp: "x".repeat(256) },
      },
    }),
    429,
    "queue_full",
  );
  assert.deepEqual(
    (await f.events(guest.participant, guest.cursor)).value.events,
    [],
  );
  // An unacknowledged host membership queue has no spare control capacity.
  await f.request("/v1/leave", { token: guest.participant.token, body: {} });
  const result = await f.events(guest.participant, guest.cursor);
  assert.equal(result.value.session.reason, "signaling_overflow");
  assert.equal(result.value.events[0].type, "session-ended");
  assert.throws(() => new SessionStore({ limits: { queueBytes: 255 } }));
});

test("ready and guest leave are once-only, free capacity and prune old credentials while host stays live", async (t) => {
  const f = await fixture(t, { limits: { retainedParticipants: 2 } });
  const host = await f.create();
  const guest = await f.join(host);
  for (let i = 0; i < 2; i++)
    assert.equal(
      (
        await f.request("/v1/heartbeat", {
          token: guest.participant.token,
          body: { ready: true },
        })
      ).status,
      200,
    );
  const ready = await f.events(host.participant);
  assert.equal(
    ready.value.events.filter((e) => e.type === "peer-ready").length,
    1,
  );
  assert.equal(
    (await f.request("/v1/leave", { token: guest.participant.token, body: {} }))
      .status,
    200,
  );
  error(
    await f.request("/v1/leave", { token: guest.participant.token, body: {} }),
    410,
    "participant_left",
  );
  error(
    await f.request("/v1/join", { body: { invite: host.invite } }),
    429,
    "participant_quota",
  );
  for (let i = 0; i < 3; i++) {
    f.advance(20_000);
    assert.equal(
      (
        await f.request("/v1/heartbeat", {
          token: host.participant.token,
          body: {},
        })
      ).status,
      200,
    );
  }
  error(await f.events(guest.participant), 401, "unauthorized");
  assert.equal((await f.join(host)).participant.status, "pending");
  assert.equal(f.store.diagnostics().credentials, 2);
});

test("chunked request stream enforces the body bound without a Content-Length header", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const result = await new Promise((resolve, reject) => {
    const req = httpRequest(
      f.base + "/v1/signals",
      {
        method: "POST",
        headers: {
          Origin: ORIGIN,
          Authorization: `Bearer ${host.participant.token}`,
          "Content-Type": "application/json",
        },
      },
      (res) => {
        const chunks = [];
        res.on("data", (chunk) => chunks.push(chunk));
        res.on("end", () =>
          resolve({
            status: res.statusCode,
            value: JSON.parse(Buffer.concat(chunks).toString()),
          }),
        );
        res.on("error", reject);
      },
    );
    req.on("error", reject);
    req.write("x".repeat(12_000));
    req.write("x".repeat(12_000));
    req.end("x".repeat(1_000));
  });
  error(result, 413, "body_limit");
  assert.equal(f.store.diagnostics().credentials, 1);
});

test("maintenance wakes an outstanding poll on host expiry with its retained end notice", async (t) => {
  const f = await fixture(t);
  const host = await f.create();
  const guest = await f.join(host);
  const pending = f.events(guest.participant, guest.cursor, 1000);
  await waitFor(() => f.server.coop.pendingPolls() === 1);
  f.advance(30_000); // Injected clock; the real service maintenance interval triggers the wake.
  const result = await pending;
  assert.equal(result.status, 200);
  assert.equal(result.value.session.reason, "host_timeout");
  assert.deepEqual(
    result.value.events.map((e) => [e.type, e.reason]),
    [["session-ended", "host_timeout"]],
  );
  assert.equal(f.server.coop.pendingPolls(), 0);
});
