import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import {
  createGalaxyCoopPorts,
  readOwnedLoadout,
} from "../../apps/game/src/coop/wasm-ports.ts";
import {
  validatePayload,
  publication,
  spec,
} from "../../apps/game/src/coop/wire.ts";

// Replay exact native serializer output; this verifies the browser boundary,
// not execution of native Game/Replica or real RTC in this Node process.
const fixture = (name) =>
  JSON.parse(
    readFileSync(
      new URL(`./fixtures/co-op-publication/${name}.json`, import.meta.url),
      "utf8",
    ),
  );
const capture = fixture("capture").results[0].value;
const entities = fixture("entities").results[0].value;
const body = fixture("body").results[0].value;
const session = capture.snapshot.session;
const self = capture.snapshot.participants[1];
const ok = JSON.stringify({ v: 1, ok: true, value: null });

const wakeFixture = (name) =>
  JSON.parse(
    readFileSync(
      new URL(`./fixtures/co-op-wake/${name}.json`, import.meta.url),
      "utf8",
    ),
  );

test("exact native maximum stage and Wake request use the third delegate with matching context", () => {
  const stageRequest = wakeFixture("maximum-native-stage"),
    wakeRequest = wakeFixture("maximum-native-wake");
  const calls = [];
  const galaxy = {
    co_op_replica(text) {
      calls.push(JSON.parse(text));
      assert.ok(Buffer.byteLength(text) <= 65536);
      return ok;
    },
  };
  const port = createGalaxyCoopPorts(galaxy).replica;
  assert.equal(
    port.begin(
      stageRequest.header.snapshot.session,
      stageRequest.header.snapshot.participants[1].id,
      stageRequest.header.snapshot.participants[1].epoch,
      stageRequest.header.snapshot.world_seed,
    ),
    true,
  );
  assert.equal(port.stage(stageRequest.header), true);
  assert.equal(port.page(wakeRequest.page), true);
  assert.deepEqual(calls[1], stageRequest);
  assert.deepEqual(calls[2], wakeRequest);
  assert.equal(calls[2].op, "replica_wake");
  console.log(
    JSON.stringify({
      scope: "recorded fake native delegate requests, not native execution",
      stageBytes: Buffer.byteLength(JSON.stringify(calls[1])),
      wakeBytes: Buffer.byteLength(JSON.stringify(calls[2])),
    }),
  );
  port.end();
});

test("scalar stress Wake request requires its own maximal staged token and roster", () => {
  const request = wakeFixture("scalar-stress-native-wake");
  const header = wakeFixture("maximum-header");
  header.acquisition = request.acquisition;
  header.snapshot.tick = request.page.tick;
  header.snapshot.revision = request.page.revision;
  header.snapshot.participants.forEach((p) => (p.epoch = 0xffff_ffff));
  header.snapshot.projectiles.forEach((p) => {
    if (p.owner !== null) p.owner_epoch = 0xffff_ffff;
  });
  header.snapshot.nova_fx.records.forEach((r) => (r.epoch = 0xffff_ffff));
  const calls = [];
  const port = createGalaxyCoopPorts({
    co_op_replica: (text) => {
      calls.push(JSON.parse(text));
      return ok;
    },
  }).replica;
  const p = header.snapshot.participants[1];
  port.begin(
    header.snapshot.session,
    p.id,
    p.epoch,
    header.snapshot.world_seed,
  );
  port.stage(header);
  port.page(request.page);
  assert.deepEqual(calls.at(-1), request);
  assert.ok(Buffer.byteLength(JSON.stringify(calls.at(-1))) < 65536);
  port.end();
});

test("Wake delegate rejection retains the ownership token for commit/end recovery", () => {
  const header = wakeFixture("initial-header"),
    page = wakeFixture("initial-page"),
    calls = [];
  let reject = true;
  const port = createGalaxyCoopPorts({
    co_op_replica: (text) => {
      const r = JSON.parse(text);
      calls.push(r);
      return r.op === "replica_wake" && reject
        ? JSON.stringify({ v: 1, ok: false, error: "invalid_wake_page" })
        : ok;
    },
  }).replica;
  const p = header.snapshot.participants[1];
  port.begin(
    header.snapshot.session,
    p.id,
    p.epoch,
    header.snapshot.world_seed,
  );
  port.stage(header);
  assert.equal(port.page(page), false);
  reject = false;
  assert.equal(port.page(page), true);
  assert.deepEqual(
    calls.filter((r) => r.op === "replica_wake").map((r) => r.acquisition),
    [header.acquisition, header.acquisition],
  );
  const wrong = structuredClone(page);
  wrong.revision++;
  assert.throws(() => port.page(wrong), /replica_page/);
  const mixed = { ...page, cells: [] };
  assert.throws(() => port.page(mixed));
  port.end();
  assert.equal(calls.at(-1).op, "replica_end");
});

test("actual two-operation host response is preserved without pretending it is one result", () => {
  const raw = readFileSync(
    new URL(
      "./fixtures/co-op-wake/actual-host-batch-response.json",
      import.meta.url,
    ),
    "utf8",
  );
  const header = wakeFixture("actual-host-header"),
    page = wakeFixture("actual-host-page");
  assert.equal(header.snapshot.participants.length, 1);
  const request = {
    v: 1,
    operations: Array.from({ length: 2 }, () => ({
      op: "wake_page",
      session: header.snapshot.session,
      acquisition: header.acquisition,
      revision: header.snapshot.revision,
    })),
  };
  const ports = createGalaxyCoopPorts({
    co_op_batch: (text) => {
      assert.deepEqual(JSON.parse(text), request);
      return raw;
    },
  });
  const response = JSON.parse(
    ports.native.co_op_batch(JSON.stringify(request)),
  );
  assert.equal(response.results.length, 2);
  assert.deepEqual(
    response.results.map((r) => r.value),
    [page, page],
  );
});

const equipment = () => ({
  version: 1,
  selected_ship: 0,
  owned_ships: 1,
  patches: 0,
  royal_victory: false,
  ship_xp: 8,
  hardware: [0, 0, 0, 0, 0, 0],
  slots: [null, null, { index: 2, rank: 1 }],
  trained_starter: false,
  relic_count: 0,
});

test("equipment extraction delegates to native solo Profile, preserving receiver", () => {
  const value = equipment();
  const galaxy = {
    co_op_batch(request) {
      assert.equal(this, galaxy);
      assert.deepEqual(JSON.parse(request), {
        v: 1,
        operations: [{ op: "export_loadout" }],
      });
      return JSON.stringify({ v: 1, results: [{ ok: true, value }] });
    },
  };
  assert.deepEqual(readOwnedLoadout(galaxy), value);
});

test("equipment extraction fails closed for absent native export and malformed declarations", () => {
  assert.throws(() => readOwnedLoadout({}), /native_unavailable/);
  for (const raw of [
    "bad JSON",
    "x".repeat(65537),
    JSON.stringify({ v: 1, results: [{ ok: false, error: "active_session" }] }),
    JSON.stringify({ v: 1, results: [] }),
    JSON.stringify({
      v: 1,
      results: [{ ok: true, value: { ...equipment(), currency: [9, 9, 9] } }],
    }),
    JSON.stringify({
      v: 1,
      results: [
        { ok: true, value: { ...equipment(), hardware: [0, 0, 0, 0, 0, 6] } },
      ],
    }),
    JSON.stringify({
      v: 1,
      results: [
        {
          ok: true,
          value: { ...equipment(), slots: [null, null, { index: 2, rank: 6 }] },
        },
      ],
    }),
  ])
    assert.throws(() => readOwnedLoadout({ co_op_batch: () => raw }));
});

test("optional owned equipment preserves legacy spec and publication while rejecting inconsistent hulls", () => {
  const legacy = { local_seed: 123, identity: null };
  spec(legacy);
  spec({ ...legacy, owned_loadout: equipment() });
  assert.throws(() =>
    spec({ ...legacy, owned_loadout: { ...equipment(), ship_xp: -1 } }),
  );
  const updated = structuredClone(capture);
  const participant = updated.snapshot.participants[1];
  participant.owned_loadout = {
    ...equipment(),
    selected_ship: participant.selected_ship,
  };
  publication(updated);
  participant.owned_loadout.selected_ship =
    (participant.selected_ship + 1) % 10;
  assert.throws(() => publication(updated));
});

test("ordinary Echo evolution survives native extraction, join and publication boundaries", () => {
  for (const evolution of [1, 2, 3, 4, 5]) {
    const value = {
      ...equipment(),
      slots: [{ index: 3, rank: 4 }, null, null],
      echo_evolution: evolution,
    };
    const raw = JSON.stringify({ v: 1, results: [{ ok: true, value }] });
    const extracted = readOwnedLoadout({ co_op_batch: () => raw });
    assert.deepEqual(extracted, value);
    validatePayload("join", {
      local_seed: 123,
      identity: null,
      owned_loadout: extracted,
    });
    const updated = structuredClone(capture);
    const participant = updated.snapshot.participants[1];
    participant.owned_loadout = {
      ...extracted,
      selected_ship: participant.selected_ship,
    };
    publication(updated);
    assert.equal(participant.owned_loadout.echo_evolution, evolution);
    assert.equal(Object.hasOwn(extracted, "boss_generations"), false);
  }
});

test("Echo evolution refuses malformed tokens, wrong slots and persisted progression containers", () => {
  const valid = {
    ...equipment(),
    slots: [{ index: 3, rank: 1 }, null, null],
    echo_evolution: 2,
  };
  const invalid = [
    ...[undefined, null, 0, 6, -1, 1.5, "2", true, NaN, Infinity].map(
      (echo_evolution) => ({ ...valid, echo_evolution }),
    ),
    { ...valid, slots: [null, null, { index: 3, rank: 1 }] },
    { ...valid, slots: [{ index: 0, rank: 1 }, null, null] },
    { ...valid, boss_generations: [[0, 0, 0, 0, 0]] },
    { ...valid, currency: [10, 10, 10] },
    (() => {
      const value = { ...valid };
      delete value.relic_count;
      return value;
    })(),
  ];
  for (const value of invalid) {
    assert.throws(() =>
      spec({ local_seed: 123, identity: null, owned_loadout: value }),
    );
    // JSON's undefined/NaN handling can erase or normalize a malformed token;
    // direct shape rejection above remains the oracle for those inputs.
    if (
      value.echo_evolution !== undefined &&
      Number.isFinite(value.echo_evolution)
    ) {
      const raw = JSON.stringify({ v: 1, results: [{ ok: true, value }] });
      assert.throws(() => readOwnedLoadout({ co_op_batch: () => raw }));
    }
  }
});

test("ordinary Nova and Echo evolution remain independent across native, join and publication boundaries", () => {
  for (const nova_evolution of [1, 2, 3, 4, 5]) {
    for (const echo_evolution of [undefined, 2, 5]) {
      const value = {
        ...equipment(),
        slots: [{ index: 3, rank: 2 }, { index: 4, rank: 3 }, null],
        nova_evolution,
        ...(echo_evolution === undefined ? {} : { echo_evolution }),
      };
      const extracted = readOwnedLoadout({
        co_op_batch: () =>
          JSON.stringify({ v: 1, results: [{ ok: true, value }] }),
      });
      assert.deepEqual(extracted, value);
      validatePayload("join", {
        local_seed: 123,
        identity: null,
        owned_loadout: extracted,
      });
      const updated = structuredClone(capture);
      updated.snapshot.participants[1].owned_loadout = {
        ...extracted,
        selected_ship: updated.snapshot.participants[1].selected_ship,
      };
      publication(updated);
      assert.equal(
        updated.snapshot.participants[1].owned_loadout.nova_evolution,
        nova_evolution,
      );
      assert.equal(Object.hasOwn(extracted, "boss_generations"), false);
    }
  }
});

test("Nova evolution refuses wrong slots, malformed tokens and missing required fields even with Echo", () => {
  const valid = {
    ...equipment(),
    slots: [{ index: 3, rank: 1 }, { index: 4, rank: 1 }, null],
    echo_evolution: 2,
    nova_evolution: 2,
  };
  const invalid = [
    ...[undefined, null, 0, 6, -1, 1.5, "2", true, NaN, Infinity].map(
      (nova_evolution) => ({ ...valid, nova_evolution }),
    ),
    { ...valid, slots: [{ index: 3, rank: 1 }, null, { index: 4, rank: 1 }] },
    { ...valid, slots: [{ index: 3, rank: 1 }, { index: 1, rank: 1 }, null] },
    { ...valid, boss_generations: [[0, 0, 0, 0, 0]] },
    { ...valid, currency: [10, 10, 10] },
    (() => {
      const value = { ...valid };
      delete value.relic_count;
      return value;
    })(),
  ];
  for (const value of invalid) {
    assert.throws(() =>
      spec({ local_seed: 123, identity: null, owned_loadout: value }),
    );
    // undefined is erased by JSON; all other malformed inputs reach the native
    // extraction boundary, including non-finite numbers normalized to null.
    if (value.nova_evolution !== undefined) {
      assert.throws(() =>
        readOwnedLoadout({
          co_op_batch: () =>
            JSON.stringify({ v: 1, results: [{ ok: true, value }] }),
        }),
      );
    }
  }
  const legacy = equipment();
  spec({ local_seed: 123, identity: null, owned_loadout: legacy });
  assert.equal(Object.keys(legacy).length, 10);
});

// Native export and committed Git bytes use LF. A Windows text checkout may
// supply CRLF; this changes no JSON value or native numeric data.
const canonicalFixtureBytes = (raw) =>
  Buffer.from(raw.toString("utf8").replace(/\r\n/g, "\n"));

test("actual native legacy and evolved Echo value exports replay without invented history or rank clamping", () => {
  const root = new URL("./fixtures/co-op-echo-loadout/", import.meta.url);
  const provenance = JSON.parse(readFileSync(new URL("provenance.json", root)));
  assert.equal(
    provenance.nativeSourceCommit,
    "84c485a858464ed45e6f34ef56a6baee69757763",
  );
  assert.equal(provenance.fixtureSetup.evolved.naturallyEarned, false);
  for (const pin of provenance.files) {
    const raw = canonicalFixtureBytes(readFileSync(new URL(pin.file, root)));
    assert.equal(raw.length, pin.bytes);
    assert.equal(createHash("sha256").update(raw).digest("hex"), pin.sha256);
    const value = JSON.parse(raw);
    // Only this envelope is supplied by the boundary double. The retained value
    // bytes came from actual native Game export in two separate executions.
    const extracted = readOwnedLoadout({
      co_op_batch: () =>
        JSON.stringify({ v: 1, results: [{ ok: true, value }] }),
    });
    assert.deepEqual(extracted, value);
    spec({ local_seed: 123, identity: null, owned_loadout: extracted });
    assert.equal(Object.hasOwn(extracted, "boss_generations"), false);
    if (pin.file === "evolved.json") {
      assert.equal(extracted.echo_evolution, 5);
      assert.deepEqual(extracted.slots[0], { index: 3, rank: 5 });
      assert.equal(extracted.ship_xp, 0);
    } else {
      assert.equal(Object.hasOwn(extracted, "echo_evolution"), false);
      assert.equal(Object.keys(extracted).length, 10);
    }
  }
});

test("actual native legacy and evolved Nova exports replay with stored rank and no synthetic history", () => {
  const root = new URL("./fixtures/co-op-nova-loadout/", import.meta.url);
  const provenance = JSON.parse(readFileSync(new URL("provenance.json", root)));
  assert.equal(
    provenance.nativeSourceCommit,
    "f31873bfe3e181c57bc4ef0bdf157b14f291de11",
  );
  assert.equal(provenance.fixtureSetup.evolved.naturallyEarned, false);
  for (const pin of provenance.files) {
    const raw = canonicalFixtureBytes(readFileSync(new URL(pin.file, root)));
    assert.equal(raw.length, pin.bytes);
    assert.equal(createHash("sha256").update(raw).digest("hex"), pin.sha256);
    const value = JSON.parse(raw);
    // Actual native value bytes; only the boundary-test envelope is supplied.
    const extracted = readOwnedLoadout({
      co_op_batch: () =>
        JSON.stringify({ v: 1, results: [{ ok: true, value }] }),
    });
    assert.deepEqual(extracted, value);
    validatePayload("join", {
      local_seed: 123,
      identity: null,
      owned_loadout: extracted,
    });
    const updated = structuredClone(capture);
    updated.snapshot.participants[1].owned_loadout = {
      ...extracted,
      selected_ship: updated.snapshot.participants[1].selected_ship,
    };
    publication(updated);
    assert.equal(Object.hasOwn(extracted, "boss_generations"), false);
    if (pin.file === "evolved.json") {
      assert.equal(extracted.nova_evolution, 5);
      assert.deepEqual(extracted.slots[1], { index: 4, rank: 5 });
      assert.equal(extracted.ship_xp, 0);
    } else {
      assert.equal(Object.hasOwn(extracted, "nova_evolution"), false);
      assert.equal(Object.keys(extracted).length, 10);
    }
  }
});

test("tracked real publication DTOs retain canonical native export hashes and validate wire", () => {
  const provenance = JSON.parse(
    readFileSync(
      new URL("./fixtures/co-op-publication/provenance.json", import.meta.url),
    ),
  );
  assert.equal(provenance.commit, "91a18ab00c51efb6638404baaab6b389d58923b1");
  for (const pin of provenance.files) {
    const checkedOut = readFileSync(
      new URL(`./fixtures/co-op-publication/${pin.file}`, import.meta.url),
    );
    const raw = canonicalFixtureBytes(checkedOut);
    assert.equal(raw.length, pin.bytes);
    assert.equal(createHash("sha256").update(raw).digest("hex"), pin.sha256);
    const windowsText = Buffer.from(
      raw.toString("utf8").replace(/\n/g, "\r\n"),
    );
    assert.deepEqual(canonicalFixtureBytes(windowsText), raw);
  }
  publication(capture);
  validatePayload("body", entities);
  validatePayload("body", body);
  assert.equal(capture.counts.entities, 61);
  assert.equal(capture.counts.body_pages, 120);
  assert.equal(entities.entities.length, 61);
  assert.equal(body.cells.length, 931);
});

test("actual Galaxy batch delegate preserves receiver and native response bytes", () => {
  const raw = readFileSync(
    new URL("./fixtures/co-op-publication/capture.json", import.meta.url),
    "utf8",
  );
  const galaxy = {
    co_op_batch(request) {
      assert.equal(this, galaxy);
      assert.deepEqual(JSON.parse(request), {
        v: 1,
        operations: [{ op: "acquire", session }],
      });
      return raw;
    },
  };
  const ports = createGalaxyCoopPorts(galaxy);
  assert.deepEqual(ports.availability, { host: true, guest: false });
  assert.equal(ports.replica, undefined);
  assert.equal(
    ports.native.co_op_batch(
      JSON.stringify({ v: 1, operations: [{ op: "acquire", session }] }),
    ),
    raw,
  );
});

test("replica port maps real publisher DTOs to exact agreed native operation keys", () => {
  const calls = [];
  const galaxy = {
    co_op_replica(request) {
      assert.equal(this, galaxy);
      calls.push(JSON.parse(request));
      return ok;
    },
  };
  const replica = createGalaxyCoopPorts(galaxy).replica;
  assert.equal(
    replica.begin(session, self.id, self.epoch, capture.snapshot.world_seed),
    true,
  );
  assert.equal(replica.stage(capture), true);
  assert.equal(replica.page(entities), true);
  assert.equal(replica.page(body), true);
  assert.equal(
    replica.commit(capture.acquisition, capture.snapshot.revision),
    true,
  );
  replica.end();
  assert.deepEqual(calls, [
    {
      v: 1,
      op: "replica_begin",
      session,
      peer: self.id,
      epoch: self.epoch,
      world_seed: capture.snapshot.world_seed,
    },
    { v: 1, op: "replica_stage", header: capture },
    {
      v: 1,
      op: "replica_entities",
      acquisition: capture.acquisition,
      page: entities,
    },
    { v: 1, op: "replica_body", acquisition: capture.acquisition, page: body },
    {
      v: 1,
      op: "replica_commit",
      acquisition: capture.acquisition,
      revision: capture.snapshot.revision,
    },
    { v: 1, op: "replica_end", session },
  ]);
  // This replay supplies one of 120 body pages; the injected reply is not a
  // native completeness proof. Real commit must validate every needed body.
});

test("missing exports report unavailable without pretending that a native replica exists", () => {
  const ports = createGalaxyCoopPorts({});
  assert.deepEqual(ports.availability, { host: false, guest: false });
  assert.equal(ports.replica, undefined);
  assert.throws(() => ports.native.co_op_batch("{}"), /native_unavailable/);
});

test("rejected and malformed end replies retain the native session recovery handle", () => {
  const calls = [];
  let reply = ok;
  const replica = createGalaxyCoopPorts({
    co_op_replica(request) {
      calls.push(JSON.parse(request));
      return reply;
    },
  }).replica;
  replica.begin(session, self.id, self.epoch, capture.snapshot.world_seed);
  reply = JSON.stringify({ v: 1, ok: false, error: "stale_session" });
  assert.throws(() => replica.end(), /replica_end_rejected/);
  reply = JSON.stringify({ v: 1, ok: true, unexpected: "field" });
  assert.throws(() => replica.end(), /invalid_object/);
  reply = ok;
  replica.end();
  assert.equal(calls.filter((c) => c.op === "replica_end").length, 3);
  assert.ok(
    calls
      .filter((c) => c.op === "replica_end")
      .every((c) => c.session === session),
  );
  replica.end();
  assert.equal(calls.length, 4);
});

test("ambiguous begin response keeps a token for cleanup, explicit native rejection does not", () => {
  const calls = [];
  let reply = "{";
  const replica = createGalaxyCoopPorts({
    co_op_replica(request) {
      calls.push(JSON.parse(request));
      return reply;
    },
  }).replica;
  assert.throws(
    () =>
      replica.begin(session, self.id, self.epoch, capture.snapshot.world_seed),
    /replica_response/,
  );
  reply = ok;
  replica.end();
  assert.equal(calls.at(-1).session, session);
  reply = JSON.stringify({ v: 1, ok: false, error: "invalid_binding" });
  assert.equal(
    replica.begin(session, self.id, self.epoch, capture.snapshot.world_seed),
    false,
  );
  const count = calls.length;
  replica.end();
  assert.equal(calls.length, count);
});

test("stale acquisitions and mismatched sessions are refused before native invocation", () => {
  let calls = 0;
  const replica = createGalaxyCoopPorts({
    co_op_replica() {
      calls++;
      return ok;
    },
  }).replica;
  replica.begin(session, self.id, self.epoch, capture.snapshot.world_seed);
  replica.stage(capture);
  const before = calls;
  assert.throws(
    () => replica.commit(capture.acquisition + 1, capture.snapshot.revision),
    /replica_commit/,
  );
  assert.throws(
    () => replica.page({ ...body, session: "f".repeat(32) }),
    /replica_page/,
  );
  assert.equal(calls, before);
  replica.end();
});

test("unbounded or invalid native response envelopes never become successful operations", () => {
  let reply = ok;
  const replica = createGalaxyCoopPorts({
    co_op_replica() {
      return reply;
    },
  }).replica;
  const rejected = [
    undefined,
    "x".repeat(65_537),
    JSON.stringify({ v: 2, ok: true }),
    JSON.stringify({ v: 1, ok: false, error: "raw network error!" }),
    JSON.stringify({ v: 1, ok: true, error: "contradiction" }),
    JSON.stringify({ v: 1, ok: true, value: false }),
  ];
  for (const response of rejected) {
    reply = response;
    assert.throws(() =>
      replica.begin(session, self.id, self.epoch, capture.snapshot.world_seed),
    );
    reply = ok;
    replica.end();
  }
});

test("complete ef5b55b native operation recording maps through ports with no manufactured success replies", () => {
  const provenance = fixture("native-replica-provenance");
  assert.equal(provenance.commit, "ef5b55bd303b422aea6a2a93748370128839277c");
  const raw = canonicalFixtureBytes(
    readFileSync(
      new URL(
        `./fixtures/co-op-publication/${provenance.file}`,
        import.meta.url,
      ),
    ),
  );
  assert.equal(raw.length, provenance.bytes);
  assert.equal(
    createHash("sha256").update(raw).digest("hex"),
    provenance.sha256,
  );
  const actual = JSON.parse(raw);
  assert.match(actual.scope, /not executed WASM\/browser wrapper/);
  assert.equal(actual.operations.length, 9);
  let index = 0;
  const replica = createGalaxyCoopPorts({
    co_op_replica(text) {
      assert.ok(Buffer.byteLength(text) <= 65_536);
      const recorded = actual.operations[index++];
      assert.deepEqual(JSON.parse(text), recorded.request);
      return JSON.stringify(recorded.response);
    },
  }).replica;
  for (const { request: r } of actual.operations) {
    switch (r.op) {
      case "replica_begin":
        assert.equal(
          replica.begin(r.session, r.peer, r.epoch, r.world_seed),
          true,
        );
        break;
      case "replica_stage":
        assert.equal(replica.stage(r.header), true);
        break;
      case "replica_entities":
      case "replica_body":
        assert.equal(replica.page(r.page), true);
        break;
      case "replica_commit":
        assert.equal(replica.commit(r.acquisition, r.revision), true);
        break;
      case "replica_end":
        replica.end();
        break;
      default:
        assert.fail("Unknown recorded native operation");
    }
  }
  assert.equal(index, actual.operations.length);
  const header = actual.operations.find((o) => o.request.op === "replica_stage")
    .request.header;
  assert.equal(header.snapshot.participants.length, 2);
  assert.equal(header.counts.entities, 3);
  assert.equal(header.counts.occupied_cells, 1974);
  assert.equal(header.counts.body_pages, 4);
  // Native Result mapping is retained actual evidence. This Node process still
  // does not instantiate Game, Replica, RTC or a generated Galaxy WASM module.
});
