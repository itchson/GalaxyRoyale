import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  snapshot,
  publication,
  encode,
  MAX_FRAME_BYTES,
} from "../../apps/game/src/coop/wire.ts";
const fixture = (name) =>
  JSON.parse(
    readFileSync(
      new URL(`./fixtures/co-op-nova-fx/${name}-header.json`, import.meta.url),
      "utf8",
    ),
  );
// Real native serializer exports, not execution of native Game/Replica in Node.
test("actual legacy, active, expired and maximum Nova headers cross the strict boundary", () => {
  for (const name of ["legacy", "nova", "expired", "maximum"]) {
    const header = fixture(name);
    publication(header);
    const encoded = encode({
      v: 1,
      session: header.snapshot.session,
      seq: 1,
      kind: "publication",
      payload: header,
    });
    assert.ok(Buffer.byteLength(encoded) < MAX_FRAME_BYTES);
    assert.deepEqual(JSON.parse(encoded).payload, header);
  }
  const max = fixture("maximum").snapshot;
  assert.equal(max.participants.length, 8);
  assert.equal(max.projectiles.length, 180);
  assert.equal(max.nova_fx.records.length, 32);
  assert.equal(Object.hasOwn(fixture("legacy").snapshot, "nova_fx"), false);
  assert.equal(fixture("expired").snapshot.nova_fx.records.length, 0);
});
test("Nova metadata rejects null, missing and unknown journal/birth fields", () => {
  for (const mutate of [
    (s) => (s.nova_fx = null),
    (s) => delete s.nova_fx.dropped,
    (s) => (s.nova_fx.profile = {}),
    (s) => delete s.nova_fx.records[0].rays,
    (s) => (s.nova_fx.records[0].damage = 10),
  ]) {
    const s = fixture("nova").snapshot;
    mutate(s);
    assert.throws(() => snapshot(s));
  }
});
test("Nova metadata rejects invalid membership, positions, counts and chronology", () => {
  const changes = [
    (r) => (r.sequence = 0),
    (r) => (r.sequence = 0x1_0000_0000),
    (r) => (r.epoch = 0),
    (r) => (r.epoch += 1),
    (r) => (r.emitter = "f".repeat(32)),
    (r) => (r.emitter = "A" + r.emitter.slice(1)),
    (r) => (r.position = [Infinity, 0]),
    (r) => (r.position = [1_000_001, 0]),
    (r) => (r.position = [0]),
    (r) => (r.rays = 37),
    (r) => (r.rays = -1),
    (r) => (r.rays = 0.5),
    (r) => (r.tick = 0x1_0000_0000),
  ];
  for (const mutate of changes) {
    const s = fixture("maximum").snapshot;
    mutate(s.nova_fx.records[0]);
    assert.throws(() => snapshot(s));
  }
  for (const mutate of [
    (s) => (s.nova_fx.sequence = 0),
    (s) => (s.nova_fx.dropped = -1),
    (s) => s.nova_fx.records.push(s.nova_fx.records[0]),
    (s) => (s.nova_fx.records[1].sequence = s.nova_fx.records[0].sequence),
    (s) => (s.nova_fx.records[0].tick = s.tick + 1),
    (s) => {
      s.tick = 2;
      s.nova_fx.records[0].tick = 2;
      s.nova_fx.records[1].tick = 1;
    },
  ]) {
    const s = fixture("maximum").snapshot;
    mutate(s);
    assert.throws(() => snapshot(s));
  }
});
test("zero admitted rays and an empty high-water journal are valid visual omissions", () => {
  const s = fixture("nova").snapshot;
  s.nova_fx.records[0].rays = 0;
  snapshot(s);
  s.nova_fx.records = [];
  snapshot(s);
  s.nova_fx.sequence = 0;
  snapshot(s);
});
