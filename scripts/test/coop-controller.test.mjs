import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { CoopController } from "../../apps/game/src/coop/controller.ts";
import {
  allowed,
  PROTOCOL,
  validatePayload,
} from "../../apps/game/src/coop/wire.ts";

// Declared adapter models. These tests exercise real controller/wire code, not
// native Game simulation, a browser RTC connection or network performance.
const local = {
  local_seed: 871238449,
  identity: {
    kind: "dora",
    corporation: "pan",
    sub_corporation: "renko",
    hex_id: "7f8a52daece2ed52",
  },
};
const empty = { fire: false },
  thrust = { fire: true, thrust: [100, 30] };
const pid = (n) => n.toString(16).padStart(32, "0");
const eid = (n) => n.toString(16).padStart(16, "0");
const signature = (cells, width = 4, height = 4) => {
  let hash = 0xcbf29ce484222325n;
  const byte = (b) => {
    hash = BigInt.asUintN(64, (hash ^ BigInt(b)) * 0x100000001b3n);
  };
  const word = (n) => {
    for (let i = 0; i < 4; i++) byte((n >>> (i * 8)) & 255);
  };
  for (const b of new TextEncoder().encode("galaxy-body-v1\0")) byte(b);
  word(width);
  word(height);
  word(cells.length);
  for (const [index, rgba] of cells) {
    word(index);
    word(rgba);
  }
  return hash.toString(16).padStart(16, "0");
};
const flush = async () => {
  for (let i = 0; i < 12; i++) await Promise.resolve();
};

class Network {
  transports = [];
  sent = [];
  session = 0;
  deferCreate = null;
  factory = (options) => {
    const t = new Transport(this, options);
    this.transports.push(t);
    return t;
  };
}
class Transport {
  phase = "idle";
  reason = null;
  session = null;
  participant = null;
  peers = [];
  blocked = false;
  sim = [];
  closes = 0;
  generation = 0;
  constructor(network, options) {
    this.network = network;
    this.options = options;
    this.index = network.transports.length + 1;
  }
  changed() {
    this.options.onChange?.();
  }
  async create(seed) {
    const generation = ++this.generation;
    if (this.network.deferCreate) await this.network.deferCreate;
    if (generation !== this.generation) return;
    this.session = {
      id: pid(++this.network.session + 100),
      seed,
      protocol: PROTOCOL,
    };
    this.participant = { id: pid(this.index), role: "host", status: "active" };
    this.phase = "signaling";
    this.changed();
  }
  async join() {
    const host = this.network.transports.find(
      (t) => t.participant?.role === "host",
    );
    this.session = host.session;
    this.participant = {
      id: pid(this.index),
      role: "guest",
      status: "pending",
    };
    this.phase = "signaling";
    this.peers = [
      {
        id: host.participant.id,
        transport: "ready",
        admission: "pending",
        simulation: "waiting",
        bufferedBytes: 0,
      },
    ];
    host.peers.push({
      id: this.participant.id,
      transport: "ready",
      admission: "pending",
      simulation: "waiting",
      bufferedBytes: 0,
    });
    host.changed();
    this.changed();
  }
  diagnostics() {
    return structuredClone({
      phase: this.phase,
      reason: this.reason,
      session: this.session,
      participant: this.participant,
      peers: this.peers,
      pendingRequests: 0,
      timers: 0,
      rtcErrors: [],
    });
  }
  invite() {
    return this.participant?.role === "host" ? "#" + "A".repeat(300) : null;
  }
  send(peer, kind, payload) {
    const p = this.peers.find((p) => p.id === peer);
    if (!p || this.blocked) return false;
    assert.equal(allowed(kind, this.participant.role), true);
    validatePayload(kind, payload);
    const other = this.network.transports.find(
      (t) => t.participant?.id === peer,
    );
    this.network.sent.push({
      from: this.participant.id,
      to: peer,
      kind,
      payload: structuredClone(payload),
      at: this.options.clock?.now?.() ?? this.network.now ?? 0,
    });
    if (kind === "admission") {
      p.admission = payload.accepted ? "admitted" : "rejected";
      other.peers[0].admission = p.admission;
    }
    const frame = {
        v: 1,
        session: this.session.id,
        seq: 1,
        kind,
        payload: structuredClone(payload),
      },
      sender = this.participant.id;
    queueMicrotask(() => {
      if (other?.phase !== "signaling") return;
      try {
        other.options.onFrame(sender, frame);
      } catch {
        void other.leave("invalid_frame");
      }
    });
    return true;
  }
  markSimulation(peer, running) {
    const p = this.peers.find((p) => p.id === peer);
    if (!p || p.admission !== "admitted") return false;
    p.simulation = running ? "running" : "waiting";
    this.sim.push(running);
    this.changed();
    return true;
  }
  disconnect(peer) {
    const other = this.network.transports.find(
      (t) => t.participant?.id === peer,
    );
    this.peers = this.peers.filter((p) => p.id !== peer);
    void other?.leave("channel_closed");
    this.changed();
    return true;
  }
  async leave(reason = "local_left") {
    this.generation++;
    this.closes++;
    this.phase = "ended";
    this.reason = reason;
    this.peers = [];
    this.session = this.participant = null;
    this.changed();
  }
  async end() {
    const others = this.network.transports.filter(
      (t) => t !== this && t.session?.id === this.session?.id,
    );
    for (const other of others) void other.leave("channel_error");
    await this.leave("host_ended");
  }
  dispose() {
    return this.leave("disposed");
  }
}
class Native {
  calls = [];
  active = false;
  revision = 1;
  acquisition = 0;
  tick = 0;
  participants = [];
  capture = null;
  failInput = null;
  failEnd = false;
  cellColor = 0xff556677;
  saved = {
    seed: local.local_seed,
    ship_xp: Array(10).fill(0),
    currency: [3, 2, 1],
    identity: local.identity,
  };
  co_op_batch = (text) => {
    const request = JSON.parse(text);
    assert.equal(request.v, 1);
    assert.ok(request.operations.length <= 16);
    return JSON.stringify({
      v: 1,
      results: request.operations.map((op) => {
        this.calls.push(structuredClone(op));
        if (op.op === "end" && this.failEnd)
          return { ok: false, error: "no_session" };
        if (op.op === "input" && op.peer === this.failInput)
          return { ok: false, error: "stale_sequence" };
        return { ok: true, value: this.op(op) };
      }),
    });
  };
  op(o) {
    if (o.op === "begin") {
      this.active = true;
      this.session = o.session;
      this.participants = [{ id: o.host, slot: 0, epoch: 1 }];
      return this.participants[0];
    }
    if (o.op === "end") {
      this.active = false;
      this.capture = null;
      return null;
    }
    assert.equal(o.session, this.session);
    if (o.op === "join") {
      const a = {
        id: o.participant.id,
        slot: this.participants.length,
        epoch: ++this.revision,
      };
      this.participants.push(a);
      this.capture = null;
      return a;
    }
    if (o.op === "leave") {
      this.participants = this.participants.filter((p) => p.id !== o.peer);
      this.capture = null;
      this.revision++;
      return null;
    }
    if (o.op === "host_input" || o.op === "input") return null;
    if (o.op === "acquire") {
      const cells = [
        [1, this.cellColor],
        [6, 0xff778899],
      ];
      const rows = this.participants.map((a, i) => ({
        id: eid(i + 1),
        kind: "player",
        participant: a.id,
        position: [10 + this.tick, 2],
        previous: [9 + this.tick, 2],
        velocity: [1, 0],
        heading: 0,
        previous_heading: 0,
        scale: 1,
        radius: 3,
        hull: 100,
        max_hull: 100,
        width: 4,
        height: 4,
        occupied: cells.length,
        body_signature: signature(cells),
      }));
      this.capture = {
        header: {
          acquisition: ++this.acquisition,
          snapshot: {
            protocol: PROTOCOL,
            session: this.session,
            world_seed: this.saved.seed,
            tick: this.tick,
            revision: this.revision,
            progression: false,
            dropped_notices: 0,
            participants: this.participants.map((a) => ({
              ...a,
              ...local,
              selected_ship: 0,
              position: [10, 2],
              previous: [9, 2],
              velocity: [1, 0],
              heading: 0,
              previous_heading: 0,
              hull: 100,
              max_hull: 100,
              shield: 0,
              max_shield: 0,
              throttle: 0,
              last_sequence: null,
            })),
            projectiles: [],
          },
          counts: {
            entities: rows.length,
            occupied_cells: rows.length * cells.length,
            storage_cells: rows.length * 16,
            entity_pages: 1,
            body_pages: rows.length,
          },
        },
        rows,
        cells,
      };
      return this.capture.header;
    }
    assert.equal(o.acquisition, this.capture.header.acquisition);
    assert.equal(o.revision, this.capture.header.snapshot.revision);
    const common = {
      session: this.session,
      tick: this.capture.header.snapshot.tick,
      revision: o.revision,
      offset: o.offset,
      next_offset: null,
    };
    if (o.op === "entity_page")
      return { ...common, entities: this.capture.rows };
    if (o.op === "body_page")
      return {
        ...common,
        id: o.entity,
        width: 4,
        height: 4,
        occupied: this.capture.cells.length,
        cells: this.capture.cells,
      };
    assert.fail(`Unexpected model operation ${o.op}`);
  }
}
class Replica {
  begins = [];
  pages = [];
  commits = [];
  ends = 0;
  header = null;
  accept = true;
  begin = (...args) => {
    this.begins.push(args);
    return this.accept;
  };
  stage = (h) => {
    this.header = structuredClone(h);
    return this.accept;
  };
  page = (p) => {
    this.pages.push(structuredClone(p));
    return this.accept;
  };
  commit = (...args) => {
    this.commits.push(args);
    return this.accept;
  };
  end = () => {
    this.ends++;
  };
}
async function pair({
  replica = new Replica(),
  guestSpec = local,
  hostSpec = local,
  sessionCounter = 0,
} = {}) {
  const network = new Network(),
    native = new Native(),
    guestNative = new Native();
  let now = 0;
  network.session = sessionCounter;
  native.saved.seed = hostSpec.local_seed;
  const options = {
    service: "http://127.0.0.1:4176",
    localSpec: () => structuredClone(hostSpec),
    native,
    transportFactory: network.factory,
    now: () => now,
  };
  const host = new CoopController(options),
    guest = new CoopController({
      ...options,
      localSpec: () => structuredClone(guestSpec),
      native: guestNative,
      replica,
    });
  await host.create();
  await guest.join(host.invite());
  await flush();
  const frame = async (count = 1, seconds = 1 / 60) => {
    for (let i = 0; i < count; i++) {
      now += seconds * 1_000;
      network.now = now;
      host.beforeStep(seconds, thrust);
      guest.beforeStep(seconds, empty);
      // Only the declared stand-in for Galaxy.step advances the model clock.
      native.tick++;
      native.revision++;
      host.afterStep(seconds);
      await flush();
    }
  };
  return {
    network,
    native,
    guestNative,
    replica,
    host,
    guest,
    frame,
    now: (value) => {
      now = value;
    },
  };
}

test("channel ready is separate from native admission and complete replica application", async () => {
  const p = await pair();
  assert.equal(p.host.state().phase, "hosting");
  assert.equal(p.guest.state().phase, "admitting");
  assert.equal(p.guest.state().transport_ready, 1);
  assert.equal(p.guest.state().admitted, 0);
  assert.equal(p.replica.commits.length, 0);
  await p.frame(1);
  assert.equal(p.guest.state().phase, "loading");
  await p.frame(4);
  assert.equal(p.guest.state().phase, "connected");
  assert.ok(p.replica.commits.length > 0);
  assert.equal(p.guestNative.calls.length, 0);
  assert.equal(p.guest.state().progression, false);
  assert.ok(p.network.transports[1].sim.includes(true));
  await p.host.end();
  await p.guest.dispose();
});
test("authenticated sender is the native pilot ID, and input is bounded to one local sequence per cadence", async () => {
  const p = await pair();
  await p.frame(120);
  const join = p.native.calls.find((o) => o.op === "join");
  assert.equal(join.participant.id, pid(2));
  const remote = p.native.calls.filter((o) => o.op === "input");
  assert.ok(remote.length > 20 && remote.length <= 60);
  assert.ok(
    remote.every((o) => o.peer === pid(2) && o.session === p.native.session),
  );
  assert.ok(p.native.calls.filter((o) => o.op === "host_input").length <= 60);
  assert.equal(
    p.native.calls.some((o) =>
      ["tick", "advance", "profile", "save"].includes(o.op),
    ),
    false,
  );
  await p.host.end();
  await p.guest.dispose();
});
test("acknowledged unchanged body signatures reuse cells while pose pages continue", async () => {
  const p = await pair();
  await p.frame(12);
  const before = p.network.sent.filter(
    (f) => f.kind === "body" && f.payload.cells,
  ).length;
  const poses = p.network.sent.filter(
    (f) => f.kind === "body" && f.payload.entities,
  ).length;
  await p.frame(24);
  assert.equal(
    p.network.sent.filter((f) => f.kind === "body" && f.payload.cells).length,
    before,
  );
  assert.ok(
    p.network.sent.filter((f) => f.kind === "body" && f.payload.entities)
      .length > poses,
  );
  await p.host.end();
  await p.guest.dispose();
});
test("same-count color mutation changes signature and sends actual indexed cells", async () => {
  const p = await pair();
  await p.frame(12);
  const before = p.network.sent.filter(
    (f) => f.kind === "body" && f.payload.cells,
  ).length;
  p.native.cellColor = 0xffaabbcc;
  await p.frame(24);
  const bodies = p.network.sent.filter(
    (f) => f.kind === "body" && f.payload.cells,
  );
  assert.ok(bodies.length > before);
  assert.deepEqual(bodies.at(-1).payload.cells[0], [1, 0xffaabbcc]);
  await p.host.end();
  await p.guest.dispose();
});
test("backpressure does not advance input sequence or grow a queued input history", async () => {
  const p = await pair();
  await p.frame(8);
  const transport = p.network.transports[1];
  transport.blocked = true;
  const before =
    p.network.sent.filter((f) => f.kind === "input").at(-1)?.payload.seq ?? 0;
  await p.frame(90);
  transport.blocked = false;
  await p.frame(2);
  assert.equal(
    p.network.sent
      .filter((f) => f.kind === "input")
      .find((f) => f.payload.seq > before).payload.seq,
    before + 1,
  );
  await p.host.end();
  await p.guest.dispose();
});
test("a rejected remote input retires only its authenticated pilot and preserves the host", async () => {
  const p = await pair();
  await p.frame(8);
  p.native.failInput = pid(2);
  await p.frame(6);
  assert.equal(p.host.state().phase, "hosting");
  assert.equal(p.native.active, true);
  assert.ok(p.native.calls.some((o) => o.op === "leave" && o.peer === pid(2)));
  assert.equal(p.guest.state().phase, "ended");
  await p.host.end();
  await p.guest.dispose();
});
test("host end and guest leave restore native boundaries exactly once without touching local progression", async () => {
  const p = await pair();
  const saved = structuredClone(p.native.saved);
  await p.frame(8);
  await p.host.end();
  await flush();
  await p.guest.leave();
  await p.guest.dispose();
  assert.equal(p.native.calls.filter((o) => o.op === "end").length, 1);
  assert.equal(p.replica.ends, 1);
  assert.deepEqual(p.native.saved, saved);
  assert.equal(p.host.invite(), null);
  assert.equal(p.host.state().members, 0);
  assert.equal(p.guest.state().phase, "ended");
});
test("missing native replica never claims connected and returns to a terminal state", async () => {
  const network = new Network(),
    native = new Native();
  const options = {
    service: "http://127.0.0.1:4176",
    localSpec: () => local,
    native,
    transportFactory: network.factory,
  };
  const host = new CoopController(options),
    guest = new CoopController(options);
  await host.create();
  await guest.join(host.invite());
  await flush();
  host.afterStep(0.01);
  await flush();
  assert.equal(guest.state().phase, "ended");
  assert.equal(guest.state().reason, "replica_unavailable");
  await host.end();
  await guest.dispose();
});
test("state contains only controlled presentation data and explicit copy getter owns invitation access", async () => {
  const p = await pair();
  const state = JSON.stringify(p.host.state());
  assert.equal(
    /7f8a52|AAAA|"token"|"profile"|"seed"|"session"/.test(state),
    false,
  );
  assert.match(p.host.invite(), /^#[A-F0-9]+$/);
  assert.equal(p.guest.invite(), null);
  await p.host.end();
  await p.guest.dispose();
});
test("unexpected native SCTP close is generic, not falsely attributed to host intent", async () => {
  const p = await pair();
  await p.frame(8);
  await p.network.transports[1].leave("channel_error");
  assert.equal(p.guest.state().message, "Connection ended.");
  assert.equal(p.guest.state().reason, "connection_ended");
  await p.host.end();
  await p.guest.dispose();
});
test("invalid elapsed/input cannot become an authority tick or native operation", async () => {
  const p = await pair();
  const count = p.native.calls.length;
  assert.throws(() => p.host.beforeStep(NaN, empty));
  assert.throws(() => p.host.afterStep(1));
  assert.throws(() =>
    p.host.beforeStep(0.01, { fire: true, thrust: [Infinity, 0] }),
  );
  assert.equal(p.native.calls.length, count);
  await p.host.end();
  await p.guest.dispose();
});
test("publication wire enforces finite counts, signature format, roles and token bounds", () => {
  const n = new Native();
  n.op({ op: "begin", session: pid(100), host: pid(1) });
  const h = n.op({ op: "acquire", session: n.session });
  assert.doesNotThrow(() => validatePayload("publication", h));
  assert.throws(() =>
    validatePayload("publication", {
      ...h,
      counts: { ...h.counts, storage_cells: 1_048_577 },
    }),
  );
  assert.throws(() =>
    validatePayload("publication-end", { acquisition: 0, revision: 1 }),
  );
  assert.equal(allowed("publication-ack", "host"), false);
  assert.equal(allowed("publication", "guest"), false);
  const page = n.op({
    op: "entity_page",
    session: n.session,
    acquisition: h.acquisition,
    revision: h.snapshot.revision,
    offset: 0,
  });
  assert.throws(() =>
    validatePayload("body", {
      ...page,
      entities: [{ ...page.entities[0], body_signature: "bad" }],
    }),
  );
});
test("failed native restoration remains latched and blocks a new session until explicit successful recovery", async () => {
  const p = await pair();
  await p.frame(8);
  p.native.failEnd = true;
  await p.host.leave();
  assert.equal(p.host.state().reason, "restore_failed");
  assert.equal(p.native.active, true);
  const before = p.native.calls.length;
  await assert.rejects(p.host.create(), /restore_failed/);
  p.host.beforeStep(0.01, thrust);
  p.host.afterStep(0.01);
  assert.equal(p.native.calls.length, before);
  p.native.failEnd = false;
  await p.host.leave();
  assert.equal(p.native.active, false);
  await p.host.create();
  assert.equal(p.host.state().phase, "hosting");
  await p.host.end();
  await p.guest.dispose();
});
test("failed replica restoration retains its recovery handle and cannot reopen gameplay", async () => {
  const p = await pair();
  await p.frame(8);
  const original = p.replica.end;
  p.replica.end = () => {
    throw new Error("declared restore failure");
  };
  await p.guest.leave();
  assert.equal(p.guest.state().reason, "restore_failed");
  await assert.rejects(p.guest.join("#" + "A".repeat(300)), /restore_failed/);
  p.replica.end = original;
  await p.guest.leave();
  assert.equal(p.replica.ends, 1);
  await p.host.end();
  await p.guest.dispose();
});
test("cancellation while signaling create is pending cannot begin native authority after completion", async () => {
  const network = new Network(),
    native = new Native();
  let release;
  network.deferCreate = new Promise((resolve) => {
    release = resolve;
  });
  const controller = new CoopController({
    service: "http://127.0.0.1:4176",
    localSpec: () => local,
    native,
    transportFactory: network.factory,
  });
  const pending = controller.create();
  assert.equal(controller.frameMode(), "blocked");
  assert.equal(controller.blocksSolo(), true);
  await flush();
  await controller.leave();
  release();
  await pending;
  assert.equal(controller.state().phase, "ended");
  assert.equal(controller.invite(), null);
  assert.equal(native.calls.length, 0);
  assert.equal(controller.frameMode(), "solo");
  await controller.dispose();
});
test("a stale prior-session callback cannot mutate a replacement native session", async () => {
  const p = await pair();
  await p.frame(8);
  const oldSession = p.native.session;
  await p.host.create();
  const calls = p.native.calls.length;
  p.network.transports[0].options.onFrame(pid(2), {
    v: 1,
    session: oldSession,
    seq: 1,
    kind: "input",
    payload: { epoch: 2, seq: 999, input: thrust },
  });
  p.host.beforeStep(0, empty);
  assert.equal(
    p.native.calls.slice(calls).some((o) => o.op === "input"),
    false,
  );
  assert.notEqual(p.native.session, oldSession);
  await p.host.end();
  await p.guest.dispose();
});
test("native commit refusal cannot mark a staged replica running", async () => {
  const replica = new Replica();
  replica.commit = () => false;
  const p = await pair({ replica });
  await p.frame(8);
  assert.equal(p.guest.state().phase, "ended");
  assert.equal(p.network.transports[1].sim.includes(true), false);
  assert.equal(replica.ends, 1);
  await p.host.end();
  await p.guest.dispose();
});
test("incomplete or stale publication end cannot install partially received cells", async () => {
  const p = await pair();
  await p.frame(1);
  const transport = p.network.transports[1],
    h = p.replica.header;
  assert.ok(h);
  const committed = p.replica.commits.length;
  assert.throws(
    () =>
      transport.options.onFrame(pid(1), {
        v: 1,
        session: h.snapshot.session,
        seq: 1,
        kind: "publication-end",
        payload: { acquisition: h.acquisition, revision: h.snapshot.revision },
      }),
    /publication_incomplete/,
  );
  assert.equal(p.replica.commits.length, committed);
  await p.host.end();
  await p.guest.dispose();
});
test("rolling publication quotas remain bounded at120Hz rather than allowing window-boundary bursts", async () => {
  const p = await pair();
  await p.frame(480, 1 / 120);
  const pieces = p.network.sent.filter((f) =>
    ["publication", "body", "publication-end"].includes(f.kind),
  );
  for (const piece of pieces) {
    const window = pieces.filter(
      (f) => f.at > piece.at - 1_000 && f.at <= piece.at,
    );
    assert.ok(window.length <= 80);
    assert.ok(
      window.reduce(
        (n, f) => n + Buffer.byteLength(JSON.stringify(f.payload)) + 256,
        0,
      ) <= 1_048_576,
    );
  }
  assert.ok(p.native.calls.filter((o) => o.op === "host_input").length <= 121);
  await p.host.end();
  await p.guest.dispose();
});
test("no acknowledged receipt expires only the stalled guest while host simulation remains available", async () => {
  const p = await pair();
  await p.frame(1);
  p.network.transports[1].blocked = true;
  await p.frame(6);
  p.now(31_100);
  p.host.afterStep(0.01);
  await flush();
  assert.equal(p.host.state().phase, "hosting");
  assert.equal(p.native.active, true);
  assert.ok(p.native.calls.some((o) => o.op === "leave" && o.peer === pid(2)));
  await p.host.end();
  await p.guest.dispose();
});

test("replica binds authenticated host world seed while preserving distinct guest local metadata", async () => {
  const guestSpec = {
    local_seed: 1234,
    identity: { ...local.identity, corporation: "hugo" },
  };
  const before = structuredClone(guestSpec);
  const p = await pair({ guestSpec });
  await p.frame(8);
  assert.deepEqual(p.replica.begins, [
    [p.native.session, pid(2), 2, local.local_seed],
  ]);
  assert.deepEqual(p.native.calls.find((o) => o.op === "join").participant, {
    id: pid(2),
    ...guestSpec,
  });
  assert.deepEqual(guestSpec, before);
  assert.equal(p.host.frameMode(), "host");
  assert.equal(p.guest.frameMode(), "guest");
  assert.equal(p.guest.blocksSolo(), true);
  await p.host.end();
  assert.equal(p.host.frameMode(), "solo");
  await p.guest.dispose();
});

test("one-frame click courses survive sampling cadence and backpressure, then are consumed once", async () => {
  const p = await pair();
  await p.frame(8);
  p.host.beforeStep(0.04, empty);
  const before = p.native.calls.length;
  p.host.beforeStep(0.001, { fire: false, destination: [50, 70] });
  assert.equal(p.native.calls.length, before);
  p.host.beforeStep(0.04, empty);
  assert.deepEqual(p.native.calls.at(-1).input.destination, [50, 70]);
  p.host.beforeStep(0.04, empty);
  assert.equal(p.native.calls.at(-1).input.destination, null);

  const transport = p.network.transports[1];
  transport.blocked = true;
  const inputCount = () =>
    p.network.sent.filter((f) => f.kind === "input").length;
  const count = inputCount();
  p.guest.beforeStep(0.04, { fire: false, destination: [90, 30] });
  p.guest.beforeStep(0.04, { fire: false, destination: [110, 60] });
  p.guest.beforeStep(0.04, empty);
  assert.equal(inputCount(), count);
  transport.blocked = false;
  p.guest.beforeStep(0, { fire: true, aim: [120, 60] });
  const delivered = p.network.sent
    .filter((f) => f.kind === "input")
    .at(-1).payload;
  assert.deepEqual(delivered.input.destination, [110, 60]);
  assert.deepEqual(delivered.input.aim, [120, 60]);
  assert.equal(delivered.input.fire, true);
  p.guest.beforeStep(0.04, empty);
  assert.equal(
    p.network.sent.filter((f) => f.kind === "input").at(-1).payload.input
      .destination,
    null,
  );
  await p.host.end();
  await p.guest.dispose();
});

test("cancelInput sends the existing native zero-stick course cancellation, then resumes fresh controls", async () => {
  const p = await pair();
  await p.frame(8);
  for (const controller of [p.host, p.guest]) {
    controller.beforeStep(0.001, { fire: true, destination: [500, 20] });
    controller.cancelInput();
    controller.beforeStep(0, thrust);
  }
  const zero = {
    joystick: [0, 0],
    destination: null,
    thrust: null,
    aim: null,
    fire: false,
  };
  assert.deepEqual(
    p.native.calls.filter((o) => o.op === "host_input").at(-1).input,
    zero,
  );
  assert.deepEqual(
    p.network.sent.filter((f) => f.kind === "input").at(-1).payload.input,
    zero,
  );
  p.host.beforeStep(0.04, thrust);
  p.guest.beforeStep(0.04, thrust);
  assert.deepEqual(
    p.native.calls.filter((o) => o.op === "host_input").at(-1).input.thrust,
    thrust.thrust,
  );
  assert.equal(
    p.native.calls.filter((o) => o.op === "host_input").at(-1).input.fire,
    true,
  );
  assert.deepEqual(
    p.network.sent.filter((f) => f.kind === "input").at(-1).payload.input
      .thrust,
    thrust.thrust,
  );
  await p.host.end();
  await p.guest.dispose();
});

test("a synchronous transport end followed by failed native restoration cannot submit another input", async () => {
  const p = await pair();
  await p.frame(8);
  p.native.failEnd = true;
  const transport = p.network.transports[0];
  // Deliberately omit onChange: beforeStep itself must discover this transition.
  transport.phase = "ended";
  transport.reason = "channel_error";
  const before = p.native.calls.length;
  p.host.beforeStep(0.04, thrust);
  assert.deepEqual(
    p.native.calls.slice(before).map((o) => o.op),
    ["end"],
  );
  assert.equal(p.host.state().reason, "restore_failed");
  assert.equal(p.host.frameMode(), "blocked");
  p.host.afterStep(0.04);
  assert.equal(p.native.calls.length, before + 1);
  p.native.failEnd = false;
  await p.host.leave();
  assert.equal(p.host.frameMode(), "solo");
  await p.guest.dispose();
});

test("an ambiguous successful native begin reply retains ownership until real cleanup", async () => {
  const network = new Network(),
    native = new Native();
  const delegate = native.co_op_batch;
  native.co_op_batch = (text) => {
    const response = delegate(text);
    return JSON.parse(text).operations[0].op === "begin" ? "{" : response;
  };
  const controller = new CoopController({
    service: "http://127.0.0.1:4176",
    localSpec: () => local,
    native,
    transportFactory: network.factory,
  });
  await controller.create();
  assert.equal(controller.state().phase, "ended");
  assert.deepEqual(
    native.calls.map((o) => o.op),
    ["begin", "end"],
  );
  assert.equal(native.active, false);
  assert.equal(controller.frameMode(), "solo");
  await controller.dispose();
});

test("an admitted guest drops a stale published world after 30 seconds without claiming host background simulation", async () => {
  const p = await pair();
  await p.frame(8);
  const commits = p.replica.commits.length;
  assert.ok(commits > 0);
  p.now(31_000);
  p.guest.beforeStep(0, empty);
  await flush();
  assert.equal(p.guest.state().phase, "ended");
  assert.equal(p.guest.state().reason, "publication_stalled");
  assert.equal(
    p.guest.state().message,
    "Shared-world updates stopped. Session closed.",
  );
  assert.equal(p.replica.ends, 1);
  assert.equal(p.replica.commits.length, commits);
  assert.equal(p.host.frameMode(), "host");
  await p.host.end();
  await p.guest.dispose();
});

test("unavailable concrete ports do not launch signaling or publish a connected state", async () => {
  const network = new Network(),
    native = new Native();
  const controller = new CoopController({
    service: "http://127.0.0.1:4176",
    localSpec: () => local,
    native,
    availability: { host: false, guest: false },
    transportFactory: network.factory,
  });
  await assert.rejects(controller.create(), /native_unavailable/);
  await assert.rejects(
    controller.join("#" + "A".repeat(300)),
    /replica_unavailable/,
  );
  assert.equal(network.session, 0);
  assert.equal(native.calls.length, 0);
  assert.equal(controller.frameMode(), "solo");
  await controller.dispose();
});

test("actual 91a18ab publisher rows and indexed cells reach the controller, but partial export cannot claim a complete world", async () => {
  const actual = (name) =>
    JSON.parse(
      readFileSync(
        new URL(`./fixtures/co-op-publication/${name}.json`, import.meta.url),
        "utf8",
      ),
    ).results[0].value;
  const header = actual("capture"),
    rows = actual("entities"),
    body = actual("body");
  const p = await pair({
    hostSpec: { local_seed: header.snapshot.world_seed, identity: null },
    guestSpec: { local_seed: 123, identity: null },
    sessionCounter: -1,
  });
  const transport = p.network.transports[0];
  assert.equal(p.native.session, header.snapshot.session);
  transport.send(pid(2), "admission", { accepted: true, slot: 1, epoch: 2 });
  await flush();
  for (const [kind, payload] of [
    ["publication", header],
    ["body", rows],
    ["body", body],
  ]) {
    transport.send(pid(2), kind, payload);
    await flush();
  }
  assert.deepEqual(p.replica.header, header);
  assert.deepEqual(p.replica.pages, [rows, body]);
  assert.equal(p.guest.state().phase, "loading");
  assert.equal(p.replica.commits.length, 0);
  // Original export contains one of 120 full-body pages; refusing completion
  // here is a controller boundary proof, not execution of native Replica.
  assert.throws(
    () =>
      p.network.transports[1].options.onFrame(pid(1), {
        v: 1,
        session: header.snapshot.session,
        seq: 1,
        kind: "publication-end",
        payload: {
          acquisition: header.acquisition,
          revision: header.snapshot.revision,
        },
      }),
    /publication_incomplete/,
  );
  assert.equal(p.replica.commits.length, 0);
  assert.equal(p.network.transports[1].sim.includes(true), false);
  await p.host.end();
  await p.guest.dispose();
});
