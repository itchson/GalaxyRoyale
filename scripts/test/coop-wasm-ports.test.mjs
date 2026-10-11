import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { createGalaxyCoopPorts } from "../../apps/game/src/coop/wasm-ports.ts";
import { validatePayload, publication } from "../../apps/game/src/coop/wire.ts";

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

// Native export and committed Git bytes use LF. A Windows text checkout may
// supply CRLF; this changes no JSON value or native numeric data.
const canonicalFixtureBytes = (raw) =>
  Buffer.from(raw.toString("utf8").replace(/\r\n/g, "\n"));

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
