import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import {
  ownedLoadout,
  validatePayload,
  publication,
} from "../../apps/game/src/coop/wire.ts";
import { readOwnedLoadout } from "../../apps/game/src/coop/wasm-ports.ts";
const folder = new URL("./fixtures/co-op-wake-loadout/", import.meta.url);
const provenance = JSON.parse(readFileSync(new URL("provenance.json", folder)));
const exportValue = (value) =>
  readOwnedLoadout({
    co_op_batch: () => JSON.stringify({ v: 1, results: [{ ok: true, value }] }),
  });
const evolved = () =>
  JSON.parse(readFileSync(new URL("ordinary-evolution-5.json", folder)));
const capture = JSON.parse(
  readFileSync(
    new URL("./fixtures/co-op-publication/capture.json", import.meta.url),
  ),
).results[0].value;

test("actual native Wake exports retain exact bytes, omission and stored versus effective rank across browser boundaries", () => {
  assert.equal(
    provenance.nativeSourceCommit,
    "87de1f7afad1a53183fcca91e146c9a59ee012cd",
  );
  assert.equal(provenance.fixtureSetup.ordinary.naturallyEarned, false);
  for (const pin of provenance.files) {
    const raw = readFileSync(new URL(pin.file, folder));
    assert.equal(raw.length, pin.bytes);
    assert.equal(createHash("sha256").update(raw).digest("hex"), pin.sha256);
    const value = JSON.parse(raw),
      extracted = exportValue(value);
    assert.deepEqual(extracted, value);
    validatePayload("join", {
      local_seed: 123,
      identity: null,
      owned_loadout: extracted,
    });
    const frame = structuredClone(capture),
      peer = frame.snapshot.participants[1];
    peer.owned_loadout = { ...extracted, selected_ship: peer.selected_ship };
    publication(frame);
    if (pin.file === "legacy.json")
      assert.deepEqual(extracted.slots, [null, null, null]);
    else assert.deepEqual(extracted.slots[2], { index: 5, rank: 5 });
    assert.equal(extracted.ship_xp, 0);
    assert.equal(Object.hasOwn(extracted, "boss_generations"), false);
    assert.equal(Object.hasOwn(extracted, "wake_points"), false);
    if (pin.file === "legacy.json" || pin.file === "ordinary-evolution-1.json")
      assert.equal(Object.hasOwn(extracted, "wake_evolution"), false);
    else
      assert.equal(
        extracted.wake_evolution,
        Number(pin.file.match(/(\d)\.json$/)[1]),
      );
  }
});

test("ordinary Wake evolution coexists independently with equipped Echo and Nova claims", () => {
  for (const wake of [undefined, 1, 2, 3, 4, 5])
    for (const echo of [undefined, 1, 5])
      for (const nova of [undefined, 1, 5]) {
        const value = {
          ...evolved(),
          slots: [
            { index: 3, rank: 2 },
            { index: 4, rank: 3 },
            { index: 5, rank: 4 },
          ],
        };
        delete value.wake_evolution;
        for (const [key, claim] of [
          ["wake_evolution", wake],
          ["echo_evolution", echo],
          ["nova_evolution", nova],
        ])
          if (claim !== undefined) value[key] = claim;
        ownedLoadout(value);
        assert.deepEqual(exportValue(value), value);
        validatePayload("join", {
          local_seed: 123,
          identity: null,
          owned_loadout: value,
        });
        const frame = structuredClone(capture),
          peer = frame.snapshot.participants[1];
        peer.owned_loadout = { ...value, selected_ship: peer.selected_ship };
        publication(frame);
        assert.equal(
          Object.keys(value).length,
          10 +
            Number(wake !== undefined) +
            Number(echo !== undefined) +
            Number(nova !== undefined),
        );
      }
});

test("Wake claims reject malformed tokens, wrong equipment, missing fields and arbitrary world/progression state", () => {
  const valid = evolved();
  const invalid = [
    ...[undefined, null, 0, 6, 255, 256, -1, 1.5, "2", true, NaN, Infinity].map(
      (wake_evolution) => ({ ...valid, wake_evolution }),
    ),
    { ...valid, slots: [{ index: 5, rank: 1 }, null, null] },
    { ...valid, slots: [null, null, { index: 2, rank: 1 }] },
    { ...valid, slots: [null, null, null] },
    { ...valid, wake_points: [] },
    { ...valid, boss_generations: [[0, 0, 0, 0, 4]] },
    { ...valid, currency: [0, 0, 10] },
    (() => {
      const v = { ...valid };
      delete v.relic_count;
      return v;
    })(),
  ];
  for (const value of invalid) {
    assert.throws(() => ownedLoadout(value));
    assert.throws(() =>
      validatePayload("join", {
        local_seed: 123,
        identity: null,
        owned_loadout: value,
      }),
    );
    // JSON erases undefined; every other invalid primitive reaches the strict bridge.
    if (value.wake_evolution !== undefined)
      assert.throws(() => exportValue(value));
  }
});
