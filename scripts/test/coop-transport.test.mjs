import assert from "node:assert/strict";
import test from "node:test";
import { once } from "node:events";
import { setTimeout as delay } from "node:timers/promises";
import { CoopTransport } from "../../apps/game/src/coop/transport.ts";
import {
  CHANNEL,
  PROTOCOL,
  MAX_BUFFERED_BYTES,
  decode,
  encode,
} from "../../apps/game/src/coop/wire.ts";
import { Signaling } from "../../apps/game/src/coop/signaling.ts";
import { makeCoopServer } from "../../apps/coop/server.mjs";
import { SessionStore } from "../../apps/coop/store.mjs";

const ORIGIN = "http://transport-test.localhost:4173";
const sid = "a".repeat(32),
  pid = "b".repeat(32);
const neutral = { local_seed: 123, identity: null };
const identity = {
  kind: "dora",
  corporation: "pan",
  sub_corporation: "renko",
  hex_id: "7f8a52daece2ed52",
};
const input = {
  epoch: 1,
  seq: 1,
  input: {
    joystick: null,
    destination: null,
    aim: null,
    thrust: [100, 100],
    fire: true,
  },
};
const frame = (kind, payload, seq = 1) => ({
  v: 1,
  session: sid,
  seq,
  kind,
  payload,
});
const emptySnapshot = (session = sid, seed = 123) => ({
  protocol: PROTOCOL,
  session,
  world_seed: seed,
  tick: 0,
  revision: 1,
  progression: false,
  dropped_notices: 0,
  participants: [],
  projectiles: [],
});
async function until(predicate) {
  for (let i = 0; i < 200; i++) {
    if (predicate()) return;
    await delay(10);
  }
  assert.fail("Timed out waiting for declared transport condition");
}

// Injected deterministic RTC API model. This is lifecycle/HTTP evidence, not an
// actual browser peer connection, SDP interpretation, NAT or game authority proof.
class Channel {
  label = CHANNEL;
  ordered = true;
  maxRetransmits = null;
  maxPacketLifeTime = null;
  readyState = "connecting";
  bufferedAmount = 0;
  bufferedAmountLowThreshold = 0;
  sent = [];
  closed = 0;
  send(text) {
    if (this.readyState !== "open") throw new Error("closed test channel");
    this.sent.push(text);
    queueMicrotask(() => {
      if (this.other?.readyState === "open")
        this.other.onmessage?.({ data: text });
    });
  }
  close() {
    if (this.readyState === "closed") return;
    this.closed++;
    this.readyState = "closed";
    this.onclose?.();
    queueMicrotask(() => {
      if (this.other?.readyState !== "closed") this.other?.close();
    });
  }
}
class Network {
  pcs = [];
  autoOpen = true;
  deliverOpenChannel = false;
  make = () => {
    const n = this,
      pc = {
        index: n.pcs.length,
        connectionState: "new",
        localDescription: null,
        remoteDescription: null,
        sctp: { maxMessageSize: 65536 },
        candidates: [],
        closes: 0,
        channel: null,
        createDataChannel(label, options) {
          assert.equal(label, CHANNEL);
          assert.equal(options.ordered, true);
          return (this.channel = new Channel());
        },
        async createOffer() {
          return { type: "offer", sdp: `offer.${this.index}` };
        },
        async createAnswer() {
          return { type: "answer", sdp: `answer.${this.index}` };
        },
        async setLocalDescription(value) {
          this.localDescription = value;
          queueMicrotask(() => {
            this.onicecandidate?.({
              candidate: {
                toJSON: () => ({
                  candidate: "declared-api-candidate",
                  sdpMid: "0",
                  sdpMLineIndex: 0,
                }),
              },
            });
            this.onicecandidate?.({ candidate: null });
          });
        },
        async setRemoteDescription(value) {
          this.remoteDescription = value;
          const partner = n.pcs[Number(value.sdp.split(".")[1])];
          if (value.type === "offer") {
            this.partner = partner;
            partner.partner = this;
            this.channel = new Channel();
            this.channel.other = partner.channel;
            partner.channel.other = this.channel;
            if (!n.deliverOpenChannel)
              this.ondatachannel?.({ channel: this.channel });
          } else if (n.autoOpen) n.open(this);
        },
        async addIceCandidate(value) {
          assert.ok(this.remoteDescription);
          this.candidates.push(value);
        },
        close() {
          if (this.connectionState === "closed") return;
          this.closes++;
          this.connectionState = "closed";
          this.channel?.close();
          this.onconnectionstatechange?.();
        },
      };
    n.pcs.push(pc);
    return pc;
  };
  open(pc) {
    const pair = [pc, pc.partner];
    for (const p of pair) {
      p.connectionState = "connected";
      p.channel.readyState = "open";
    }
    for (const p of pair) {
      if (this.deliverOpenChannel && p.remoteDescription?.type === "offer")
        p.ondatachannel?.({ channel: p.channel });
      p.onconnectionstatechange?.();
      p.channel.onopen?.({});
    }
  }
}
class ManualClock {
  time = 0;
  tasks = new Map();
  now = () => this.time;
  setTimeout = (fn, ms) => {
    const timer = {};
    this.tasks.set(timer, { fn, ms });
    return timer;
  };
  clearTimeout = (timer) => this.tasks.delete(timer);
  fireDelay(ms) {
    const task = [...this.tasks].find(([, t]) => t.ms === ms);
    if (!task) return false;
    this.tasks.delete(task[0]);
    this.time += ms;
    task[1].fn();
    return true;
  }
}
async function fixture(t, storeOptions = {}, network = new Network()) {
  let time = 1_000_000;
  const store = new SessionStore({ now: () => time, ...storeOptions });
  const server = makeCoopServer({ store, allowedOrigins: [ORIGIN] });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const service = `http://127.0.0.1:${server.address().port}`,
    clients = [];
  const fetcher = (url, init) =>
    fetch(url, { ...init, headers: { ...init.headers, Origin: ORIGIN } });
  const client = (options = {}) => {
    const c = new CoopTransport({
      service,
      fetch: fetcher,
      createPeerConnection: network.make,
      ...options,
    });
    clients.push(c);
    return c;
  };
  t.after(async () => {
    await Promise.allSettled(clients.map((c) => c.dispose()));
    await server.coop.stop();
    assert.equal(server.listening, false);
    assert.equal(server.coop.pendingPolls(), 0);
    assert.equal(store.listenerCount("changed"), 0);
    for (const c of clients) {
      assert.equal(c.diagnostics().peers.length, 0);
      assert.equal(c.diagnostics().timers, 0);
      assert.equal(c.diagnostics().pendingRequests, 0);
    }
    assert.ok(network.pcs.every((p) => p.closes === 1));
  });
  const pair = async (options = {}) => {
    const received = [];
    const host = client({
      onFrame: (peer, frame) => received.push({ peer, frame }),
      ...options,
    });
    await host.create(123);
    const guest = client();
    await guest.join(host.invite());
    await approveGuest(host, guest);
    await until(
      () =>
        host.diagnostics().peers[0]?.transport === "ready" &&
        guest.diagnostics().peers[0]?.transport === "ready",
    );
    const guestId = guest.diagnostics().participant.id,
      hostId = host.diagnostics().participant.id;
    return { host, guest, guestId, hostId, received };
  };
  return {
    store,
    server,
    service,
    fetcher,
    network,
    clients,
    client,
    pair,
    advance: (ms) => {
      time += ms;
      store.sweep();
    },
  };
}
async function approveGuest(host, guest) {
  const peer = guest.diagnostics().participant.id;
  await until(() => host.diagnostics().joinRequests.some((r) => r.id === peer));
  await host.approve(peer, true);
}
async function admit(p) {
  assert.equal(
    p.host.send(p.guestId, "admission", { accepted: true, slot: 1, epoch: 1 }),
    true,
  );
  await until(() => p.guest.diagnostics().peers[0]?.admission === "admitted");
}

test("versioned frame bounds reject corruption, persistent profiles and nonfinite input", () => {
  assert.throws(
    () => encode(frame("join", { local_seed: 0, identity: null })),
    { code: "invalid_local_seed" },
  );
  assert.deepEqual(
    decode(encode(frame("join", neutral))),
    frame("join", neutral),
  );
  for (const invalid of [
    { ...frame("join", neutral), v: 2 },
    { ...frame("join", neutral), from: pid },
    frame("input", { ...input, input: { fire: true, aim: [Infinity, 0] } }),
    frame("join", { ...neutral, profile: {} }),
    frame("body", { save: {} }),
    frame("body", { data: Array(1025).fill(0) }),
    frame("body", { data: "a".repeat(16385) }),
  ])
    assert.throws(() => encode(invalid));
  assert.throws(() => decode(new ArrayBuffer(10)));
  assert.throws(() => decode(" ".repeat(65537)));
  assert.throws(() =>
    encode(
      frame("join", {
        local_seed: 1,
        identity: { ...identity, hex_id: "UPPERCASEINVALID" },
      }),
    ),
  );
  assert.throws(() => encode(frame("join", new Date())));
  assert.throws(() =>
    encode(
      frame(
        "join",
        Object.assign(Object.create({ toJSON: () => neutral }), neutral),
      ),
    ),
  );
});
test("native snapshot contract rejects duplicate slots, wrong profile semantics and nonfinite geometry", () => {
  const p = {
    id: pid,
    slot: 1,
    epoch: 1,
    local_seed: 456,
    identity,
    selected_ship: 0,
    position: [0, 0],
    previous: [0, 0],
    velocity: [0, 0],
    heading: 0,
    previous_heading: 0,
    hull: 90,
    max_hull: 90,
    shield: 0,
    max_shield: 0,
    throttle: 0,
    last_sequence: null,
  };
  assert.ok(
    encode(frame("snapshot", { ...emptySnapshot(), participants: [p] })),
  );
  for (const bad of [
    { ...emptySnapshot(), progression: true },
    { ...emptySnapshot(), participants: [p, { ...p, id: "c".repeat(32) }] },
    { ...emptySnapshot(), participants: [{ ...p, position: [NaN, 0] }] },
    { ...emptySnapshot(), participants: Array(9).fill(p) },
  ])
    assert.throws(() => encode(frame("snapshot", bad)));
});
test("HTTP reader refuses credential redirects and bounded oversized responses", async () => {
  const c = new Signaling(
    "http://localhost:4176",
    async () => new Response("x".repeat(300001)),
  );
  await assert.rejects(
    c.request("/v1/join", null, {}, new AbortController().signal),
    { code: "response_bound" },
  );
  const redirect = new Signaling("http://localhost:4176", async () => ({
    redirected: true,
  }));
  await assert.rejects(
    redirect.request("/v1/join", null, {}, new AbortController().signal),
    { code: "service_redirect" },
  );
  for (const url of [
    "http://user:secret@localhost:4176",
    "http://localhost:4176/path",
    "http://localhost:4176?token=bad",
  ])
    assert.throws(() => new Signaling(url));
});
test("body pages retain exact cell indices and reject duplicates, oversize and wrong query kinds", () => {
  const page = {
    session: sid,
    tick: 1,
    revision: 2,
    id: "c".repeat(16),
    width: 48,
    height: 48,
    occupied: 2,
    offset: 0,
    next_offset: null,
    cells: [
      [10, 0xffffffff],
      [11, 0x123456ff],
    ],
  };
  assert.deepEqual(
    decode(encode(frame("body", page))).payload.cells,
    page.cells,
  );
  assert.ok(
    encode(
      frame("body-query", {
        kind: "body",
        id: page.id,
        expected_revision: 2,
        offset: 0,
      }),
    ),
  );
  for (const bad of [
    {
      ...page,
      cells: [
        [10, 1],
        [10, 2],
      ],
    },
    { ...page, cells: [[2304, 1]] },
    { ...page, cells: Array(1025).fill([1, 1]) },
    { ...page, next_offset: 0 },
  ])
    assert.throws(() => encode(frame("body", bad)));
  assert.throws(() =>
    encode(
      frame("body-query", { kind: "save", expected_revision: 2, offset: 0 }),
    ),
  );
  assert.throws(() =>
    encode(
      frame("body-query", { kind: "body", id: page.id, tick: 1, offset: 0 }),
    ),
  );
  assert.throws(() => encode(frame("body", { ...page, revision: undefined })));
  assert.ok(
    encode(
      frame("body-query", {
        kind: "entities",
        expected_revision: 3,
        offset: 64,
      }),
    ),
  );
});
test("snapshot carries same-tick revision and permits finite authoritative projectiles beyond world edge", () => {
  const shot = {
    id: "c".repeat(16),
    owner: null,
    owner_epoch: null,
    position: [1002048, -1002048],
    previous: [1000000, -1000000],
    velocity: [400, -400],
    damage: 10,
    life: 2,
    remaining_hits: 1,
  };
  assert.ok(
    encode(
      frame("snapshot", {
        ...emptySnapshot(),
        revision: 2,
        projectiles: [shot],
      }),
    ),
  );
  for (const bad of [
    { ...emptySnapshot(), revision: undefined },
    { ...emptySnapshot(), dropped_notices: -1 },
    { ...emptySnapshot(), projectiles: [{ ...shot, position: [Infinity, 0] }] },
  ])
    assert.throws(() => encode(frame("snapshot", bad)));
});
test("actual naturally spawned Coil metadata preserves the segment unknown-maximum sentinel only", () => {
  // Exact first segment row from peer's public Game6093496 API export after
  // 1460 real ticks. Whole page SHA2562f98830878dc86104acca83267414d4afc889f590020e7a295799378478e33bb.
  const segment = {
    id: "0000000200000049",
    kind: "segment",
    participant: null,
    position: [2760.7256, -413.8149],
    previous: [2761.1062, -413.58252],
    velocity: [-22.836912, -13.9434805],
    heading: 2.4269783,
    previous_heading: 2.3893313,
    scale: 0.85,
    radius: 6.990192,
    hull: 15,
    max_hull: 0,
    width: 16,
    height: 16,
    occupied: 87,
  };
  const page = {
    session: "000000000000000000000000000003e8",
    tick: 1460,
    revision: 1460,
    offset: 0,
    next_offset: null,
    entities: [segment],
  };
  const nativeFrame = {
    v: 1,
    session: page.session,
    seq: 1,
    kind: "body",
    payload: page,
  };
  assert.deepEqual(decode(encode(nativeFrame)).payload, page);
  for (const invalid of [
    { ...segment, kind: "enemy" },
    { ...segment, max_hull: 14 },
    { ...segment, hull: -1 },
    { ...segment, hull: Infinity },
    { ...segment, max_hull: -1 },
  ])
    assert.throws(() =>
      encode({ ...nativeFrame, payload: { ...page, entities: [invalid] } }),
    );
});
test("actual HTTP membership never implies a ready channel or Rust admission", async (t) => {
  const n = new Network();
  n.autoOpen = false;
  const f = await fixture(t, {}, n);
  const host = f.client(),
    guest = f.client();
  await host.create(123);
  await guest.join(host.invite());
  await until(() => host.diagnostics().joinRequests.length === 1);
  assert.equal(n.pcs.length, 0);
  assert.equal(guest.diagnostics().awaitingApproval, true);
  assert.equal(
    guest.send(host.diagnostics().participant.id, "join", neutral),
    false,
  );
  await approveGuest(host, guest);
  await until(
    () =>
      n.pcs.length === 2 &&
      n.pcs.some((p) => p.remoteDescription?.type === "answer"),
  );
  assert.equal(guest.diagnostics().participant.status, "pending");
  assert.equal(host.diagnostics().peers[0].transport, "negotiating");
  assert.equal(guest.diagnostics().peers[0].admission, "pending");
  const peer = host.diagnostics().peers[0].id;
  assert.equal(host.send(peer, "snapshot", emptySnapshot()), false);
  n.open(n.pcs[0]);
  await until(() => guest.diagnostics().peers[0]?.transport === "ready");
  assert.equal(guest.diagnostics().peers[0].admission, "pending");
  assert.equal(
    guest.markSimulation(guest.diagnostics().peers[0].id, true),
    false,
  );
});
test("already-open incoming native channel plus later open event sends exactly one hello", async (t) => {
  const n = new Network();
  n.deliverOpenChannel = true;
  const f = await fixture(t, {}, n),
    p = await f.pair();
  await delay(30);
  assert.equal(p.host.diagnostics().peers[0].transport, "ready");
  assert.equal(p.guest.diagnostics().peers[0].transport, "ready");
  assert.ok(
    n.pcs.every(
      (pc) =>
        pc.channel.sent.filter((s) => JSON.parse(s).kind === "hello").length ===
        1,
    ),
  );
  assert.ok(n.pcs.every((pc) => pc.closes === 0));
});
test("RTC diagnostics retain only allowlisted native error detail, never messages or candidates", async (t) => {
  const f = await fixture(t),
    p = await f.pair();
  const pc = f.network.pcs.find((pc) => pc.remoteDescription?.type === "offer");
  pc.channel.onerror({
    error: { errorDetail: "sctp-failure", message: "DO-NOT-RETAIN-SDP-TOKEN" },
  });
  await until(() => p.guest.diagnostics().phase === "ended");
  assert.deepEqual(p.guest.diagnostics().rtcErrors, ["sctp-failure"]);
  assert.ok(!JSON.stringify(p.guest.diagnostics()).includes("DO-NOT-RETAIN"));
});
test("real HTTP offer/answer/ICE path binds peer IDs and admits only explicit frames", async (t) => {
  const f = await fixture(t),
    p = await f.pair();
  const local = {
      seed: 456,
      identity: structuredClone(identity),
      currency: [1, 2, 3],
    },
    before = structuredClone(local);
  assert.equal(
    p.guest.send(p.hostId, "join", {
      local_seed: local.seed,
      identity: local.identity,
    }),
    true,
  );
  await until(() => p.received.some((x) => x.frame.kind === "join"));
  assert.equal(p.received[0].peer, p.guestId);
  assert.equal(p.received[0].frame.payload.local_seed, 456);
  assert.equal(p.guest.send(p.hostId, "input", input), false);
  await admit(p);
  assert.equal(p.guest.send(p.hostId, "input", input), true);
  await until(() => p.received.some((x) => x.frame.kind === "input"));
  assert.equal(p.received.at(-1).peer, p.guestId);
  assert.equal(p.received.at(-1).frame.payload.input.fire, true);
  assert.equal(p.host.send(p.guestId, "input", input), false);
  assert.equal(
    p.guest.send(p.hostId, "admission", { accepted: true, slot: 1, epoch: 1 }),
    false,
  );
  assert.equal(p.guest.markSimulation(p.hostId, true), true);
  assert.deepEqual(local, before);
  // Admission can precede asynchronous trickled ICE delivery through HTTP.
  await until(() => f.network.pcs.every((pc) => pc.candidates.length === 2));
  assert.ok(f.network.pcs.every((pc) => pc.candidates.length === 2));
  const diagnostics = JSON.stringify(p.host.diagnostics());
  assert.ok(
    !diagnostics.includes("token") &&
      !diagnostics.includes("invite") &&
      !diagnostics.includes("sdp"),
  );
});
test("host plus seven guest channels are bounded; ninth reservation fails", async (t) => {
  const f = await fixture(t),
    host = f.client();
  await host.create(123);
  const guests = [];
  for (let i = 0; i < 7; i++) {
    const guest = f.client();
    guests.push(guest);
    await guest.join(host.invite());
    await approveGuest(host, guest);
  }
  await until(
    () =>
      host.diagnostics().peers.length === 7 &&
      host.diagnostics().peers.every((p) => p.transport === "ready"),
  );
  const ninth = f.client();
  await assert.rejects(ninth.join(host.invite()), {
    code: "session_full",
  });
  assert.equal(ninth.diagnostics().phase, "ended");
  assert.equal(f.network.pcs.length, 14);
  assert.equal(
    guests[0].send(guests[1].diagnostics().participant.id, "join", neutral),
    false,
  );
});
test("backpressure refuses without advancing wire sequence, queueing or replaying stale input", async (t) => {
  const f = await fixture(t),
    p = await f.pair();
  await admit(p);
  const channel = f.network.pcs.find(
    (pc) => pc.remoteDescription?.type === "offer",
  ).channel;
  channel.bufferedAmount = MAX_BUFFERED_BYTES;
  assert.equal(p.guest.send(p.hostId, "input", input), false);
  assert.equal(channel.sent.length, 1);
  channel.bufferedAmount = 0;
  assert.equal(p.guest.send(p.hostId, "input", { ...input, seq: 2 }), true);
  await until(() => p.received.length === 1);
  assert.equal(p.received[0].frame.seq, 2);
  assert.equal(p.received[0].frame.payload.seq, 2);
  assert.equal(channel.sent.length, 2);
});
for (const [name, corrupt] of [
  [
    "forged peer hello",
    (session) => ({
      v: 1,
      session,
      seq: 2,
      kind: "hello",
      payload: { protocol: PROTOCOL, peer_id: "0".repeat(32) },
    }),
  ],
  [
    "replayed sequence",
    (session) => ({ v: 1, session, seq: 1, kind: "join", payload: neutral }),
  ],
  ["wrong session", () => frame("join", neutral, 2)],
  [
    "guest snapshot authority",
    (session) => ({
      v: 1,
      session,
      seq: 2,
      kind: "snapshot",
      payload: emptySnapshot(session),
    }),
  ],
])
  test(`malformed ${name} closes only that host peer and leaves no duplicate RTC`, async (t) => {
    const f = await fixture(t),
      p = await f.pair();
    const channel = f.network.pcs.find(
      (pc) => pc.remoteDescription?.type === "offer",
    ).channel;
    channel.send(JSON.stringify(corrupt(p.guest.diagnostics().session.id)));
    await until(
      () =>
        p.guest.diagnostics().phase === "ended" &&
        p.host.diagnostics().peers.length === 0,
    );
    assert.equal(p.host.diagnostics().phase, "signaling");
    assert.ok(f.network.pcs.every((pc) => pc.closes === 1));
  });
test("input epoch and immutable snapshot seed are checked before sends", async (t) => {
  const f = await fixture(t),
    p = await f.pair();
  await admit(p);
  assert.equal(p.guest.send(p.hostId, "input", { ...input, epoch: 0 }), false);
  assert.equal(
    p.host.send(
      p.guestId,
      "snapshot",
      emptySnapshot(p.host.diagnostics().session.id, 999),
    ),
    false,
  );
  assert.equal(
    p.host.send(
      p.guestId,
      "snapshot",
      emptySnapshot(p.host.diagnostics().session.id, 123),
    ),
    true,
  );
  assert.equal(
    p.host.send(p.guestId, "admission", { accepted: true, slot: 2, epoch: 2 }),
    false,
  );
});
test("explicit rejection frees the guest reservation without ending the host", async (t) => {
  const f = await fixture(t),
    p = await f.pair();
  assert.equal(
    p.host.send(p.guestId, "admission", {
      accepted: false,
      reason: "not admitted by authority",
    }),
    true,
  );
  await until(
    () =>
      p.guest.diagnostics().phase === "ended" &&
      p.host.diagnostics().peers.length === 0,
  );
  assert.equal(p.guest.diagnostics().reason, "admission_rejected");
  assert.equal(p.host.diagnostics().phase, "signaling");
});
test("host end notifies guest, aborts polls and allows a distinct-session rejoin", async (t) => {
  const f = await fixture(t),
    p = await f.pair(),
    first = p.host.diagnostics().session.id,
    oldInvite = p.host.invite();
  await p.host.end();
  await until(() => p.guest.diagnostics().phase === "ended");
  assert.equal(p.host.invite(), null);
  assert.equal(p.host.diagnostics().peers.length, 0);
  await assert.rejects(p.guest.join(oldInvite));
  await p.host.create(456);
  await p.guest.join(p.host.invite());
  await approveGuest(p.host, p.guest);
  await until(() => p.guest.diagnostics().peers[0]?.transport === "ready");
  assert.notEqual(p.host.diagnostics().session.id, first);
  assert.equal(p.guest.diagnostics().session.seed, 456);
  assert.equal(p.guest.diagnostics().peers[0].admission, "pending");
});
test("expiry ends channels using real polling terminal notices", async (t) => {
  const f = await fixture(t, { limits: { sessionMs: 5000 } }),
    p = await f.pair();
  f.advance(5000);
  await until(
    () =>
      p.host.diagnostics().phase === "ended" &&
      p.guest.diagnostics().phase === "ended",
  );
  assert.equal(p.host.diagnostics().reason, "expired");
  // Channel-close can arrive before the independently polled service notice.
  assert.ok(
    ["expired", "channel_closed", "session_ended"].includes(
      p.guest.diagnostics().reason,
    ),
    p.guest.diagnostics().reason,
  );
});
test("failed negotiation does not fabricate readiness and guest leave frees its reservation", async (t) => {
  const f = await fixture(t),
    host = f.client();
  await host.create(123);
  const guest = f.client({
    createPeerConnection: () => {
      throw new Error("declared RTC unavailable");
    },
  });
  await guest.join(host.invite());
  await approveGuest(host, guest);
  await until(() => guest.diagnostics().phase === "ended");
  assert.equal(guest.diagnostics().reason, "rtc_unavailable");
  await until(() => host.diagnostics().peers.length === 0);
  assert.equal(guest.diagnostics().phase, "ended");
});
test("late create result after disposal cannot reopen state and gets a leave cleanup", async (t) => {
  const f = await fixture(t);
  let release,
    received = false;
  const gate = new Promise((resolve) => {
    release = resolve;
  });
  const host = f.client({
    fetch: async (url, init) => {
      const response = await f.fetcher(url, init);
      if (url.endsWith("/v1/sessions")) {
        const buffered = new Response(await response.text(), {
          status: response.status,
          headers: response.headers,
        });
        received = true;
        await gate;
        return buffered;
      }
      return response;
    },
  });
  const start = host.create(123);
  await until(() => received);
  await host.dispose();
  release();
  await start;
  await delay(50);
  assert.equal(host.diagnostics().phase, "ended");
  assert.equal(host.invite(), null);
  assert.equal(f.network.pcs.length, 0);
});
test("guest cannot end a host; explicit leave and repeated disposal clean every handler", async (t) => {
  const f = await fixture(t),
    p = await f.pair();
  assert.throws(() => p.guest.end(), { code: "host_only" });
  await p.guest.leave();
  await p.guest.dispose();
  await until(() => p.host.diagnostics().peers.length === 0);
  for (const pc of f.network.pcs) {
    assert.equal(pc.onicecandidate, null);
    assert.equal(pc.ondatachannel, null);
    assert.equal(pc.channel.onmessage, null);
  }
});
test("a retired peer's already-queued ICE cannot terminate an unrelated host peer", async (t) => {
  const f = await fixture(t),
    p = await f.pair();
  const second = f.client();
  await second.join(p.host.invite());
  await approveGuest(p.host, second);
  await until(
    () =>
      p.host.diagnostics().peers.length === 2 &&
      p.host.diagnostics().peers.every((x) => x.transport === "ready"),
  );
  const pc = f.network.pcs.find((pc) => pc.remoteDescription?.type === "offer");
  // Queue a real HTTP signal while its channel is still live, then close it.
  pc.onicecandidate({
    candidate: {
      toJSON: () => ({
        candidate: "declared-late-api-candidate",
        sdpMid: "0",
        sdpMLineIndex: 0,
      }),
    },
  });
  pc.channel.send(
    JSON.stringify({
      v: 1,
      session: p.host.diagnostics().session.id,
      seq: 1,
      kind: "join",
      payload: neutral,
    }),
  );
  await until(
    () =>
      p.guest.diagnostics().phase === "ended" &&
      p.host.diagnostics().peers.length === 1,
  );
  await delay(40);
  assert.equal(p.host.diagnostics().phase, "signaling");
  assert.equal(second.diagnostics().peers[0].transport, "ready");
});
test("SCTP message bound and send rate refuse without an application backlog", async (t) => {
  const f = await fixture(t),
    p = await f.pair();
  const pc = f.network.pcs.find((pc) => pc.remoteDescription?.type === "offer");
  pc.sctp.maxMessageSize = 8;
  assert.equal(p.guest.send(p.hostId, "join", neutral), false);
  pc.sctp.maxMessageSize = 65536;
  for (let i = 0; i < 119; i++)
    assert.equal(p.guest.send(p.hostId, "join", neutral), true);
  assert.equal(p.guest.send(p.hostId, "join", neutral), false);
  await until(() => p.received.length === 119);
  assert.equal(pc.channel.sent.length, 120);
});
test("unopened negotiation deadline frees membership and every tracked clock task", async (t) => {
  const n = new Network();
  n.autoOpen = false;
  const f = await fixture(t, {}, n),
    clock = new ManualClock();
  const host = f.client();
  await host.create(123);
  let releaseLeaveNotice;
  const leaveNotice = new Promise((resolve) => {
    releaseLeaveNotice = resolve;
  });
  const guest = f.client({
    clock,
    fetch: async (url, init) => {
      const response = await f.fetcher(url, init);
      // Local teardown precedes the best-effort service acknowledgement. Hold
      // that response deliberately instead of depending on HTTP scheduling.
      if (String(url).endsWith("/v1/leave")) await leaveNotice;
      return response;
    },
  });
  try {
    await guest.join(host.invite());
    await approveGuest(host, guest);
    await until(
      () =>
        n.pcs.length === 2 &&
        n.pcs.some((pc) => pc.remoteDescription?.type === "answer"),
    );
    assert.equal(clock.fireDelay(15000), true);
    await until(
      () =>
        guest.diagnostics().phase === "ended" &&
        host.diagnostics().peers.length === 0 &&
        guest.diagnostics().pendingRequests === 0,
    );
    assert.equal(guest.diagnostics().reason, "negotiation_timeout");
    assert.equal(guest.diagnostics().timers, 0);
    assert.equal(clock.tasks.size, 1); // The separately bounded leave notice.
    releaseLeaveNotice();
    await until(() => clock.tasks.size === 0);
    assert.equal(clock.tasks.size, 0);
  } finally {
    releaseLeaveNotice();
  }
});

test("pending request waits without RTC and decline retains its exact terminal outcome", async (t) => {
  const f = await fixture(t),
    host = f.client(),
    guest = f.client();
  await host.create(123);
  await guest.join(
    host.invite().toLowerCase().replace("#", " ").replace("-", "\t"),
  );
  await until(() => host.diagnostics().joinRequests.length === 1);
  const peer = guest.diagnostics().participant.id;
  assert.equal(guest.diagnostics().awaitingApproval, true);
  assert.equal(f.network.pcs.length, 0);
  assert.equal(host.diagnostics().peers.length, 0);
  assert.equal(guest.invite(), null);
  await host.approve(peer, false);
  await until(() => guest.diagnostics().phase === "ended");
  assert.equal(guest.diagnostics().reason, "join_declined");
  assert.equal(host.diagnostics().joinRequests.length, 0);
  assert.equal(f.network.pcs.length, 0);
  assert.equal(host.diagnostics().phase, "signaling");
});

test("approval timeout is independent of approved negotiation reservation timeout", async (t) => {
  const n = new Network();
  n.autoOpen = false;
  const f = await fixture(t, {}, n),
    hostClock = new ManualClock();
  let heartbeats = 0;
  const host = f.client({
      clock: hostClock,
      fetch: async (url, init) => {
        const response = await f.fetcher(url, init);
        if (String(url).endsWith("/v1/heartbeat")) heartbeats++;
        return response;
      },
    }),
    unapproved = f.client();
  await host.create(123);
  await until(() => heartbeats === 1);
  await unapproved.join(host.invite());
  await until(() => host.diagnostics().joinRequests.length === 1);
  f.advance(20_000);
  await until(() => unapproved.diagnostics().phase === "ended");
  assert.equal(unapproved.diagnostics().reason, "approval_timeout");
  assert.equal(n.pcs.length, 0);
  // Keep the owner live through a real HTTP heartbeat; do not widen limits.
  assert.equal(hostClock.fireDelay(5000), true);
  await until(() => heartbeats === 2);
  const approved = f.client();
  await approved.join(host.invite());
  await approveGuest(host, approved);
  await until(
    () =>
      n.pcs.length === 2 &&
      n.pcs.some((p) => p.remoteDescription?.type === "answer"),
  );
  f.advance(20_000);
  await until(() => approved.diagnostics().phase === "ended");
  assert.equal(approved.diagnostics().reason, "reservation_timeout");
});

test("same-world rotate/lock preserves approved channels and retires only pending requests", async (t) => {
  const f = await fixture(t),
    p = await f.pair();
  await admit(p);
  const before = p.host.diagnostics(),
    oldCode = p.host.invite(),
    oldPcs = [...f.network.pcs];
  const waiting = f.client();
  await waiting.join(oldCode);
  await until(() => p.host.diagnostics().joinRequests.length === 1);
  await p.host.replaceCode();
  await until(() => waiting.diagnostics().phase === "ended");
  assert.equal(waiting.diagnostics().reason, "code_replaced");
  const fresh = p.host.invite();
  assert.notEqual(fresh, oldCode);
  assert.match(fresh, /^#[A-F0-9]{4}-[A-F0-9]{4}$/);
  const stale = f.client();
  await assert.rejects(stale.join(oldCode), { code: "stale_invite" });
  const next = f.client();
  await next.join(fresh);
  await until(() => p.host.diagnostics().joinRequests.length === 1);
  await p.host.setJoiningLocked(true);
  await until(() => next.diagnostics().phase === "ended");
  assert.equal(next.diagnostics().reason, "joining_locked");
  const locked = f.client();
  await assert.rejects(locked.join(fresh), { code: "joining_locked" });
  assert.deepEqual(p.host.diagnostics().session.id, before.session.id);
  assert.equal(p.host.diagnostics().session.seed, before.session.seed);
  assert.equal(p.host.diagnostics().peers[0].admission, "admitted");
  assert.equal(p.guest.diagnostics().peers[0].transport, "ready");
  assert.deepEqual(f.network.pcs, oldPcs);
  assert.ok(oldPcs.every((pc) => pc.closes === 0));
  const diagnostic = JSON.stringify(p.host.diagnostics());
  assert.equal(diagnostic.includes(fresh), false);
  assert.equal(diagnostic.includes("room_code"), false);
  await p.host.setJoiningLocked(false);
  const retry = f.client();
  await retry.join(fresh);
  await approveGuest(p.host, retry);
  await until(() => retry.diagnostics().peers[0]?.transport === "ready");
});

test("hostile pre-approval SDP event cannot create guest RTC or bypass approval", async (t) => {
  const f = await fixture(t),
    host = f.client();
  await host.create(123);
  let guestPcs = 0;
  const guest = f.client({
    createPeerConnection: (...args) => {
      guestPcs++;
      return f.network.make(...args);
    },
    fetch: async (url, init) => {
      const response = await f.fetcher(url, init);
      if (String(url).includes("/v1/events")) {
        const body = await response.json();
        body.events = body.events.map((e) =>
          e.type === "join-approved"
            ? {
                seq: e.seq,
                type: "signal",
                from: host.diagnostics().participant.id,
                kind: "offer",
                payload: { sdp: "declared-hostile-service-fixture" },
              }
            : e,
        );
        return new Response(JSON.stringify(body), {
          status: response.status,
          headers: response.headers,
        });
      }
      return response;
    },
  });
  await guest.join(host.invite());
  await approveGuest(host, guest);
  await until(() => guest.diagnostics().phase === "ended");
  assert.equal(guest.diagnostics().reason, "approval_required");
  assert.equal(guestPcs, 0);
});

test("newest whole policy wins delayed rotation and heartbeat responses", async (t) => {
  const f = await fixture(t);
  let releaseCode,
    releaseHeartbeat,
    codeHeld = false,
    heartbeatHeld = false;
  const codeGate = new Promise((resolve) => {
    releaseCode = resolve;
  });
  const heartbeatGate = new Promise((resolve) => {
    releaseHeartbeat = resolve;
  });
  const host = f.client({
    fetch: async (url, init) => {
      const response = await f.fetcher(url, init);
      const holdCode = String(url).endsWith("/v1/code");
      const holdHeartbeat =
        String(url).endsWith("/v1/heartbeat") && !heartbeatHeld;
      if (holdCode || holdHeartbeat) {
        const buffered = new Response(await response.text(), {
          status: response.status,
          headers: response.headers,
        });
        if (holdCode) {
          codeHeld = true;
          await codeGate;
        } else {
          heartbeatHeld = true;
          await heartbeatGate;
        }
        return buffered;
      }
      return response;
    },
  });
  try {
    await host.create(123);
    await until(() => heartbeatHeld);
    const initial = host.invite(),
      rotate = host.replaceCode();
    await until(() => codeHeld);
    await host.setJoiningLocked(true);
    const newest = host.invite();
    assert.notEqual(newest, initial);
    assert.equal(host.diagnostics().session.room_revision, 2);
    assert.equal(host.diagnostics().session.joining_locked, true);
    releaseCode();
    await rotate;
    releaseHeartbeat();
    await delay(30);
    assert.equal(host.invite(), newest);
    assert.equal(host.diagnostics().session.room_revision, 2);
    assert.equal(host.diagnostics().session.joining_locked, true);
    assert.equal(f.network.pcs.length, 0);
  } finally {
    releaseCode();
    releaseHeartbeat();
  }
});

test("delayed old join-request cannot recreate an approval row after a newer policy", async (t) => {
  const f = await fixture(t);
  let release,
    held = false;
  const gate = new Promise((resolve) => {
    release = resolve;
  });
  const host = f.client({
    fetch: async (url, init) => {
      if (String(url).includes("/v1/events") && !held) {
        held = true;
        await gate;
      }
      return f.fetcher(url, init);
    },
  });
  try {
    await host.create(123);
    await until(() => held);
    const guest = f.client();
    await guest.join(host.invite());
    await host.replaceCode();
    // The real service now returns current policy with the unacked OLD request
    // and its terminal peer-left event, rather than an invented client event.
    release();
    await until(() => guest.diagnostics().phase === "ended");
    await delay(30);
    assert.equal(host.diagnostics().session.room_revision, 1);
    assert.equal(host.diagnostics().joinRequests.length, 0);
    assert.equal(f.network.pcs.length, 0);
    assert.equal(host.diagnostics().phase, "signaling");
  } finally {
    release();
  }
});

test("late admin response after leave cannot reopen code, policy, requests or RTC", async (t) => {
  const f = await fixture(t);
  let release,
    held = false;
  const gate = new Promise((resolve) => {
    release = resolve;
  });
  const host = f.client({
    fetch: async (url, init) => {
      const response = await f.fetcher(url, init);
      if (String(url).endsWith("/v1/code")) {
        const buffered = new Response(await response.text(), {
          status: response.status,
          headers: response.headers,
        });
        held = true;
        await gate;
        return buffered;
      }
      return response;
    },
  });
  try {
    await host.create(123);
    const change = host.replaceCode();
    await until(() => held);
    await host.leave();
    release();
    await change;
    assert.equal(host.diagnostics().phase, "ended");
    assert.equal(host.diagnostics().session, null);
    assert.equal(host.invite(), null);
    assert.equal(host.diagnostics().joinRequests.length, 0);
    assert.equal(f.network.pcs.length, 0);
  } finally {
    release();
  }
});

async function terminalRace(
  t,
  { holdRead = false, unavailableRead = false } = {},
) {
  const f = await fixture(t),
    host = f.client(),
    clock = new ManualClock();
  await host.create(123);
  let releasePoll,
    releaseRead,
    pollHeld = false,
    readHeld = false,
    readCount = 0,
    heartbeats = 0;
  const pollGate = new Promise((resolve) => {
    releasePoll = resolve;
  });
  const readGate = new Promise((resolve) => {
    releaseRead = resolve;
  });
  const guest = f.client({
    clock,
    fetch: async (url, init) => {
      const isRead = String(url).includes("wait_ms=0");
      if (isRead) {
        readCount++;
        if (unavailableRead)
          return new Response(
            JSON.stringify({
              error: {
                code: "unauthorized",
                message: "DECLARED-UNAVAILABLE-RETENTION",
              },
            }),
            { status: 401 },
          );
      }
      const response = await f.fetcher(url, init);
      if (String(url).endsWith("/v1/heartbeat")) heartbeats++;
      if (String(url).includes("/v1/events")) {
        const text = await response.text();
        const body = JSON.parse(text);
        const buffered = new Response(text, {
          status: response.status,
          headers: response.headers,
        });
        if (isRead && holdRead) {
          readHeld = true;
          await readGate;
        } else if (
          !isRead &&
          !pollHeld &&
          body.participant?.status === "left"
        ) {
          pollHeld = true;
          await pollGate;
        }
        return buffered;
      }
      return response;
    },
  });
  t.after(() => {
    releasePoll();
    releaseRead();
  });
  await guest.join(host.invite());
  await until(() => heartbeats === 1);
  await until(() => host.diagnostics().joinRequests.length === 1);
  await host.approve(guest.diagnostics().participant.id, false);
  await until(() => pollHeld);
  assert.equal(clock.fireDelay(5000), true);
  await until(() => readCount === 1);
  if (holdRead) await until(() => readHeld);
  return {
    ...f,
    host,
    guest,
    clock,
    releasePoll,
    releaseRead,
    readCount: () => readCount,
  };
}

test("bounded retained terminal read wins failed heartbeat and ignores later old poll", async (t) => {
  const f = await terminalRace(t);
  await until(() => f.guest.diagnostics().phase === "ended");
  assert.equal(f.guest.diagnostics().reason, "join_declined");
  assert.equal(f.readCount(), 1);
  f.releasePoll();
  await delay(25);
  assert.equal(f.guest.diagnostics().reason, "join_declined");
  assert.equal(f.guest.diagnostics().timers, 0);
  assert.equal(f.network.pcs.length, 0);
});

test("terminal poll wins a competing retained read without reason rollback", async (t) => {
  const f = await terminalRace(t, { holdRead: true });
  f.releasePoll();
  await until(() => f.guest.diagnostics().phase === "ended");
  assert.equal(f.guest.diagnostics().reason, "join_declined");
  f.releaseRead();
  await delay(25);
  assert.equal(f.guest.diagnostics().reason, "join_declined");
  assert.equal(f.readCount(), 1);
  assert.equal(f.guest.diagnostics().pendingRequests, 0);
});

test("leave and replacement while terminal recovery is held cannot end the new request", async (t) => {
  const f = await terminalRace(t, { holdRead: true });
  await f.guest.leave();
  await f.guest.join(f.host.invite());
  await until(() => f.host.diagnostics().joinRequests.length === 1);
  const replacement = f.guest.diagnostics().participant.id;
  f.releaseRead();
  f.releasePoll();
  await delay(25);
  assert.equal(f.guest.diagnostics().phase, "signaling");
  assert.equal(f.guest.diagnostics().participant.id, replacement);
  assert.equal(f.guest.diagnostics().awaitingApproval, true);
  assert.equal(f.network.pcs.length, 0);
});

test("unavailable retained terminal read falls back once and tears down all resources", async (t) => {
  const f = await terminalRace(t, { unavailableRead: true });
  await until(() => f.guest.diagnostics().phase === "ended");
  assert.equal(f.guest.diagnostics().reason, "participant_left");
  assert.equal(f.readCount(), 1);
  f.releasePoll();
  await delay(25);
  assert.equal(f.guest.diagnostics().reason, "participant_left");
  assert.equal(f.guest.diagnostics().pendingRequests, 0);
  assert.equal(f.guest.diagnostics().timers, 0);
  assert.equal(f.network.pcs.length, 0);
});

test("dispose during terminal recovery ignores both held replies and clears its tracked tasks", async (t) => {
  const f = await terminalRace(t, { holdRead: true });
  await f.guest.dispose();
  f.releaseRead();
  f.releasePoll();
  await delay(25);
  assert.equal(f.guest.diagnostics().phase, "ended");
  assert.equal(f.guest.diagnostics().reason, "disposed");
  assert.equal(f.guest.diagnostics().session, null);
  assert.equal(f.guest.diagnostics().pendingRequests, 0);
  assert.equal(f.guest.diagnostics().timers, 0);
  assert.equal(f.network.pcs.length, 0);
});
test("reconnection cancels loss timeout; actual failed connection closes once", async (t) => {
  const clock = new ManualClock(),
    f = await fixture(t),
    p = await f.pair({ clock });
  const pc = f.network.pcs.find(
    (pc) => pc.remoteDescription?.type === "answer",
  );
  pc.connectionState = "disconnected";
  pc.onconnectionstatechange();
  assert.ok([...clock.tasks.values()].some((t) => t.ms === 5000));
  pc.connectionState = "connected";
  pc.onconnectionstatechange();
  // The separate heartbeat5s timer remains; the loss timer is removed.
  assert.equal(
    [...clock.tasks.values()].filter((t) => t.ms === 5000).length,
    1,
  );
  pc.connectionState = "failed";
  pc.onconnectionstatechange();
  await until(
    () =>
      p.guest.diagnostics().phase === "ended" &&
      p.host.diagnostics().peers.length === 0,
  );
  assert.equal(p.host.diagnostics().phase, "signaling");
  assert.equal(pc.closes, 1);
});
