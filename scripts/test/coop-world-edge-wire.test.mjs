/** Actual native DTO replay + declared boundary mutations; no native/browser/RTC execution here. */
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import crypto from "node:crypto";
import {
  encode,
  decode,
  publication,
  snapshot,
  validatePayload,
} from "../../apps/game/src/coop/wire.ts";
import { createGalaxyCoopPorts } from "../../apps/game/src/coop/wasm-ports.ts";
const directory = new URL("./fixtures/co-op-world-edge/", import.meta.url);
const bytes = (name) => fs.readFileSync(new URL(name, directory));
const fixture = (name) => JSON.parse(bytes(name));
const provenance = fixture("native-provenance.json");
const captures = provenance.cases.map((c) => ({
  ...c,
  header: fixture(c.name + ".header.json"),
  pages: fixture(c.name + ".entities.json"),
}));
const edge = captures[0],
  outward = captures.find((c) => c.name === "outward-projectile");
const row = (kind) =>
  edge.pages.flatMap((p) => p.entities).find((r) => r.kind === kind);
const page = (entity) => ({
  ...edge.pages[0],
  entities: [entity],
  next_offset: null,
});
const frame = (kind, payload, seq = 1) => ({
  v: 1,
  session: edge.header.snapshot.session,
  seq,
  kind,
  payload,
});
const nextF32 = (limit) => {
  const a = new Float32Array([limit]);
  const bits = new Uint32Array(a.buffer);
  bits[0]++;
  return a[0];
};
const nextDouble = (limit) => {
  const b = new DataView(new ArrayBuffer(8));
  b.setFloat64(0, limit);
  b.setBigUint64(0, b.getBigUint64(0) + 1n);
  return b.getFloat64(0);
};

test("exact native edge header/entity bytes retain source provenance and complete unfiltered row counts", () => {
  const canonicalProvenance = Buffer.from(
    bytes("native-provenance.json").toString("utf8").replaceAll("\r\n", "\n"),
  );
  assert.equal(
    crypto.createHash("sha256").update(canonicalProvenance).digest("hex"),
    "a1db3db204c4d284ae62ea3deaf13576233a50d94e507dfa11181d0877191246",
  );
  assert.equal(
    provenance.nativeRuntimeCommit,
    "1b8c3ec7d20fba1dc69f14a3cc2f9b22945cd3f4",
  );
  assert.equal(
    provenance.exporterCommit,
    "84701ff7e0118654bc171276b122fc72df8a032b",
  );
  assert.equal(captures.length, 9);
  for (const c of captures) {
    for (const suffix of ["header", "entities"]) {
      const name = c.name + "." + suffix + ".json",
        actual = Buffer.from(
          bytes(name).toString("utf8").replaceAll("\r\n", "\n"),
        );
      assert.equal(actual.length, provenance.files[name].bytes, name);
      assert.equal(
        crypto.createHash("sha256").update(actual).digest("hex"),
        provenance.files[name].sha256,
        name,
      );
    }
    const rows = c.pages.flatMap((p) => p.entities);
    assert.equal(rows.length, c.header.counts.entities);
    assert.equal(rows.length, 87);
    assert.equal(
      rows.filter(
        (r) =>
          r.kind === "asteroid" &&
          r.position.some((v) => Math.abs(v) > 1000000),
      ).length,
      c.outsideAsteroids,
    );
    assert.equal(c.ownedLoadout, false);
    assert(
      c.header.snapshot.participants.every(
        (p) => p.owned_loadout === undefined,
      ),
    );
  }
});

test("actual eight unfiltered native edge/corner headers and every entity page pass public wire encode/decode unchanged", () => {
  for (const c of captures) {
    publication(c.header);
    assert.deepEqual(
      decode(encode(frame("publication", c.header))).payload,
      c.header,
    );
    for (const p of c.pages) {
      validatePayload("body", p);
      assert.deepEqual(decode(encode(frame("body", p))).payload, p);
    }
  }
});

test("every projectile velocity stays inside the native one-million component domain", () => {
  for (const sign of [-1, 1])
    for (const component of [0, 1]) {
      const valid = structuredClone(outward.header.snapshot);
      valid.projectiles[0].velocity[component] = sign * 1000000;
      snapshot(valid);
      const invalid = structuredClone(valid);
      invalid.projectiles[0].velocity[component] = sign * nextF32(1000000);
      assert.throws(() => snapshot(invalid), /invalid_vector/);
    }
});

test("declared raw limits and next-f32/f64 variations preserve separate body/player/velocity domains", () => {
  // Mutations are boundary fixtures, not newly executed native bodies.
  for (const kind of ["asteroid", "enemy", "boss", "segment", "player"]) {
    const base = structuredClone(
      row(kind === "player" ? "player" : "asteroid"),
    );
    base.kind = kind;
    const maximum = kind === "player" ? 1000000 : 1100000;
    for (const field of ["position", "previous", "velocity"]) {
      const limit = field === "velocity" ? 1000000 : maximum;
      for (const sign of [-1, 1])
        for (const component of [0, 1]) {
          const valid = structuredClone(base);
          valid[field][component] = sign * limit;
          validatePayload("body", page(valid));
          for (const excess of [nextF32(limit), nextDouble(limit)]) {
            assert(excess > limit);
            if (excess === nextDouble(limit))
              assert.equal(
                Math.fround(excess),
                limit,
                "raw-double excess rounds inside f32 but must still fail JS",
              );
            const invalid = structuredClone(valid);
            invalid[field][component] = sign * excess;
            assert.throws(
              () => validatePayload("body", page(invalid)),
              /invalid_vector/,
            );
          }
        }
      for (const nonfinite of [NaN, Infinity, -Infinity]) {
        const invalid = structuredClone(base);
        invalid[field][0] = nonfinite;
        assert.throws(
          () => validatePayload("body", page(invalid)),
          /nonfinite_payload/,
        );
      }
    }
  }
});

test("actual native outward shot retains its native owner/epoch and declared raw pose variations cannot widen speed or pilot positions", () => {
  snapshot(outward.header.snapshot);
  const actual = outward.header.snapshot.projectiles.find(
    (p) => p.position[0] > 1000000,
  );
  const owner = outward.header.snapshot.participants.find(
    (p) => p.id === actual.owner,
  );
  assert.equal(actual.owner_epoch, owner.epoch);
  assert.equal(actual.owner_epoch, 2);
  assert(actual.previous[0] > 1000000);
  assert.equal(outward.ownedLoadout, false);
  for (const field of ["position", "previous", "velocity"]) {
    const limit = field === "velocity" ? 1000000 : 1100000;
    for (const sign of [-1, 1])
      for (const component of [0, 1]) {
        const valid = structuredClone(outward.header.snapshot);
        valid.projectiles[0][field][component] = sign * limit;
        snapshot(valid);
        for (const excess of [nextF32(limit), nextDouble(limit)]) {
          const invalid = structuredClone(valid);
          invalid.projectiles[0][field][component] = sign * excess;
          assert.throws(() => snapshot(invalid), /invalid_vector/);
        }
      }
  }
  for (const field of ["position", "previous", "velocity"])
    for (const sign of [-1, 1]) {
      const invalid = structuredClone(outward.header.snapshot);
      invalid.participants[1][field][0] = sign * nextF32(1000000);
      assert.throws(() => snapshot(invalid), /invalid_vector/);
    }
  for (const field of ["aim", "thrust", "destination"])
    for (const sign of [-1, 1]) {
      const input = {
        epoch: 2,
        seq: 1,
        input: { fire: false, [field]: [sign * 1000000, 0] },
      };
      validatePayload("input", input);
      for (const excess of [nextF32(1000000), nextDouble(1000000)]) {
        const invalid = structuredClone(input);
        invalid.input[field][0] = sign * excess;
        assert.throws(() => validatePayload("input", invalid), /invalid_input/);
      }
    }
});

test("existing public shape/frame bounds remain fail-closed for malformed edge data", () => {
  const base = structuredClone(row("asteroid"));
  for (const variation of [
    (r) => (r.profile = {}),
    (r) => (r.unknown = 1),
    (r) => (r.width = 513),
    (r) => (r.occupied = r.width * r.height + 1),
    (r) => (r.body_signature = "bad"),
    (r) => (r.hull = 100001),
    (r) => (r.scale = 101),
  ]) {
    const invalid = structuredClone(base);
    variation(invalid);
    assert.throws(() => validatePayload("body", page(invalid)));
  }
  const header = structuredClone(edge.header);
  header.counts.storage_cells = 1048577;
  assert.throws(() => publication(header), /invalid_publication_counts/);
  const badPage = {
    ...edge.pages[0],
    entities: Array.from({ length: 65 }, () => structuredClone(base)),
  };
  assert.throws(() => validatePayload("body", badPage));
  assert.throws(() => decode("x".repeat(65537)), /frame_bound/);
});

test("actual exported adapter gates malformed raw data before native invocation and forwards real edge DTOs unchanged", () => {
  // Declared unit replies test adapter invocation only. No native result is inferred.
  const calls = [];
  const galaxy = {
    co_op_replica(request) {
      assert.equal(this, galaxy);
      calls.push(JSON.parse(request));
      return JSON.stringify({ v: 1, ok: true, value: null });
    },
  };
  const port = createGalaxyCoopPorts(galaxy).replica;
  const self = edge.header.snapshot.participants[1];
  assert.equal(
    port.begin(
      edge.header.snapshot.session,
      self.id,
      self.epoch,
      edge.header.snapshot.world_seed,
    ),
    true,
  );
  const badHeader = structuredClone(outward.header);
  badHeader.snapshot.projectiles[0].velocity[0] = nextF32(1000000);
  const beforeStage = calls.length;
  assert.throws(() => port.stage(badHeader), /invalid_vector/);
  assert.equal(calls.length, beforeStage);
  assert.equal(port.stage(edge.header), true);
  assert.deepEqual(calls.at(-1).header, edge.header);
  const bad = structuredClone(edge.pages[0]);
  bad.entities.find((r) => r.kind === "asteroid").position[0] =
    nextDouble(1100000);
  const beforePage = calls.length;
  assert.throws(() => port.page(bad), /invalid_vector/);
  assert.equal(calls.length, beforePage);
  for (const p of edge.pages) {
    assert.equal(port.page(p), true);
    assert.deepEqual(calls.at(-1).page, p);
  }
  // Body pages/complete commit are native evidence in the exporter, not mocked here.
});
