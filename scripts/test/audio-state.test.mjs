/** Deterministic score/mix tests; no claim of DSP or human listening. */
import assert from "node:assert/strict";
import { test } from "node:test";
import {
  advanceMix,
  audibleThemes,
  cleanState,
  newMix,
  QUIET_STATE,
  THEMES,
} from "../../apps/game/src/audio-state.ts";
const state = (patch = {}) => ({ ...QUIET_STATE, ...patch });
function settle(raw, previous = newMix(), ticks = 300) {
  for (let i = 0; i < ticks; i++) previous = advanceMix(previous, raw, 1 / 60);
  return previous;
}
test("sanctuary exit selects authored Expanse exploration, then blends back home", () => {
  const home = settle(state());
  assert.equal(audibleThemes(home)[0][0], 0);
  const first = advanceMix(home, state({ area: "belt" }), 1 / 60);
  assert(first.themes[0] > 0.98 && first.themes[14] > 0);
  const belt = settle(state({ area: "belt" }), first);
  assert.equal(audibleThemes(belt)[0][0], 14);
  assert.equal(belt.combat, 0, "No distance-only danger soundtrack");
  assert.notDeepEqual(THEMES[14].motif, THEMES[0].motif);
  const reach = settle(state({ area: "reach" }), belt);
  assert.equal(audibleThemes(reach)[0][0], 14);
  assert.equal(audibleThemes(settle(state(), reach))[0][0], 0);
});
test("six biome identities and eight champion motifs remain distinct and bounded", () => {
  for (let biome = 0; biome < 6; biome++) {
    const mix = settle(state({ area: "reach", biome }));
    assert.equal(audibleThemes(mix)[0][0], biome === 0 ? 14 : biome);
  }
  const motifs = new Set();
  for (let champion = 0; champion < 8; champion++) {
    const mix = settle(
      state({ area: "arena", biome: 3, champion, combat: 0.6, hostiles: 1 }),
    );
    assert.equal(audibleThemes(mix)[0][0], 6 + champion);
    motifs.add(JSON.stringify(THEMES[6 + champion].motif));
  }
  assert.equal(motifs.size, 8);
  assert.equal(THEMES.length, 18);
  let mix = newMix();
  for (let i = 0; i < 1000; i++) {
    mix = advanceMix(
      mix,
      state({ area: "arena", champion: i % 8, combat: (i % 11) / 10 }),
      1 / 60,
    );
    assert(audibleThemes(mix).length <= 2);
    assert(Math.abs(mix.themes.reduce((a, b) => a + b, 0) - 1) < 1e-12);
    assert(mix.themes.every((n) => Number.isFinite(n) && n >= 0 && n <= 1));
  }
});
test("real aggro rises smoothly and sanctuary/recovery fades it without false distance", () => {
  const fight = state({ area: "reach", combat: 0.75, hostiles: 4 });
  const first = advanceMix(newMix(), fight, 1 / 60);
  assert(first.combat > 0 && first.combat < 0.04);
  const loud = settle(fight);
  assert(Math.abs(loud.combat - 0.75) < 0.0001);
  assert.equal(
    settle(state({ area: "reach", combat: 0 }), loud, 180).combat < 0.12,
    true,
  );
  assert.equal(cleanState(state({ area: "site", combat: 1 })).combat, 0);
  assert.equal(cleanState(state({ area: "haven", combat: 1 })).combat, 0);
});
test("low hull alarm recedes on healing and engine requires actual live ports", () => {
  const low = settle(
    state({ area: "reach", hull: 0.1, throttle: 1, speed: 0.7, engines: 2 }),
  );
  assert(low.alarm > 0.85 && low.engine > 0.95);
  const healed = settle(state({ area: "reach", hull: 0.7 }), low, 120);
  assert(healed.alarm < 0.001 && healed.engine < 0.001);
  assert.equal(
    settle(state({ hull: 0, throttle: 1, speed: 1, engines: 2 })).engine,
    0,
  );
  assert.equal(settle(state({ throttle: 1, speed: 1, engines: 0 })).engine, 0);
  assert(
    Math.abs(settle(state({ throttle: 1, engines: 1 })).engine - 0.44) < 0.001,
  );
  assert(settle(state({ hull: 0.25 })).alarm > 0.14);
});
test("nonfinite boundaries, no-op time and source input normalization are deterministic", () => {
  const dirty = state({
    area: "invalid",
    biome: 99,
    champion: 8,
    combat: NaN,
    hostiles: Infinity,
    hull: Infinity,
    speed: -1,
    throttle: NaN,
    engines: -1,
  });
  const clean = cleanState(dirty);
  assert.deepEqual(clean, { ...QUIET_STATE, hull: 0 });
  const original = newMix();
  for (const dt of [NaN, Infinity, 0, -1])
    assert.deepEqual(advanceMix(original, dirty, dt), original);
  assert.deepEqual(
    settle(state({ area: "arena", champion: 3, combat: 0.7 })),
    settle(state({ area: "arena", champion: 3, combat: 0.7 })),
  );
});

test("three original corporate homes preserve old identities and fade through real sanctuary exit", () => {
  const motifs = new Set();
  for (const [corporation, index] of [
    ["pan", 15],
    ["renko", 16],
    ["hugo", 17],
  ]) {
    const home = settle(state({ corporation }));
    assert.equal(audibleThemes(home)[0][0], index);
    assert.equal(home.combat, 0);
    assert.equal(home.boss, 0);
    motifs.add(JSON.stringify(THEMES[index].motif));
    const first = advanceMix(
      home,
      state({ corporation, area: "belt" }),
      1 / 60,
    );
    assert(first.themes[index] > 0.95);
    assert(first.themes[14] > 0);
    const outer = settle(
      state({ corporation, area: "reach", biome: 2 }),
      first,
    );
    assert.equal(audibleThemes(outer)[0][0], 2);
    assert.equal(
      audibleThemes(settle(state({ corporation }), outer))[0][0],
      index,
    );
    assert.equal(cleanState(state({ corporation })).corporation, corporation);
  }
  assert.equal(motifs.size, 3);
  for (const corporation of [null, undefined, "PAN", "invalid", 3])
    assert.equal(audibleThemes(settle(state({ corporation })))[0][0], 0);
  assert.equal(
    cleanState(state({ corporation: "pan", sub_corporation: "hugo" }))
      .sub_corporation,
    undefined,
  );
  assert.equal(THEMES[6].name, "Lyra Voss / Needle dance");
  assert.equal(THEMES[14].name, "Expanse / Scouting signal");
});

test("boss arrangement requires a real hostile and aggro, with bounded smooth rise and decay", () => {
  for (const raw of [
    state({ boss: true }),
    state({ area: "reach", boss: true, combat: 1, hostiles: 0 }),
    state({ area: "reach", boss: true, combat: 0, hostiles: 1 }),
    state({ area: "haven", boss: true, combat: 1, hostiles: 1 }),
    state({ area: "site", boss: true, combat: 1, hostiles: 1 }),
  ])
    assert.equal(settle(raw).boss, 0);
  const active = state({ area: "reach", boss: true, combat: 0.8, hostiles: 1 });
  const first = advanceMix(newMix(), active, 1 / 60);
  assert(first.boss > 0 && first.boss < 0.05);
  const full = settle(active);
  assert(full.boss > 0.999);
  const fading = advanceMix(
    full,
    state({ area: "reach", combat: 0.8, hostiles: 4 }),
    1 / 60,
  );
  assert(fading.boss > 0.98 && fading.boss < full.boss);
  assert(
    settle(state({ area: "reach", combat: 0.8, hostiles: 4 }), fading, 900)
      .boss < 0.0001,
  );
  for (const seconds of [0, -1, NaN, Infinity])
    assert.deepEqual(advanceMix(full, active, seconds), full);
});
