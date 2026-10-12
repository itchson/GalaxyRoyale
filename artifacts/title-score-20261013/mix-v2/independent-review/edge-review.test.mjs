/** Independent title pad lifecycle checks; recorded API only, no DSP claim. */
import assert from "node:assert/strict";
import { test } from "node:test";
import { AudioEngine } from "../../apps/game/src/audio.ts";
import { QUIET_STATE } from "../../apps/game/src/audio-state.ts";
import { browser } from "./recorded-api-harness.mjs";

const valueAt = (points, time) => {
  let last = points[0];
  for (const point of points.slice(1)) {
    if (time < point.time) {
      const fraction = Math.max(0, Math.min(1, (time - last.time) / (point.time - last.time)));
      return last.value + (point.value - last.value) * fraction;
    }
    last = point;
  }
  return last.value;
};
const envelope = (source) => {
  let node = [...source.connections][0];
  while (node.kind !== "gain") node = [...node.connections][0];
  return node.gain;
};
async function titleFixture() {
  const timers = browser();
  const engine = new AudioEngine();
  engine.setTitleScreen(true);
  await engine.unlock();
  return { timers, engine, context: engine.context };
}

test("Continue during title pad attack replaces source stops and survives the former deadline", async (t) => {
  const { engine, context } = await titleFixture();
  const pad = [...engine.voices].find((voice) => voice.titlePad);
  const oldStop = pad.sources[0].stopAt;
  context.advance(0.05);
  assert(context.currentTime > pad.start && context.currentTime < pad.start + 0.15);
  const before = valueAt(pad.titlePad.points, context.currentTime);
  engine.update({ ...QUIET_STATE, corporation: "hugo" });
  engine.setTitleScreen(false);
  const newStop = pad.sources[0].stopAt;
  assert(newStop > oldStop);
  assert.equal(pad.titlePad.points[0].value, before);
  assert(pad.sources.every((source) => source.stops.length === 2 && source.stopAt === newStop));
  assert(pad.sources.every((source) => envelope(source).events.map((event) => event.kind).join() === "set,linear"));
  context.advance(oldStop + 0.001 - context.currentTime);
  assert(!pad.released && pad.sources.every((source) => !source.ended));
  const nodes = [...pad.nodes];
  context.advance(newStop + 0.001 - context.currentTime);
  assert(pad.released && pad.sources.every((source) => source.onended === null));
  assert(nodes.every((node) => node.disconnects === 1));
  t.diagnostic(JSON.stringify({ oldStop, newStop, innerEnvelope: "set,linear", scope: "recorded API" }));
  engine.dispose();
});

test("Continue during the last 40ms of the original title pad release preserves its current gate", async (t) => {
  const { engine, context } = await titleFixture();
  const pad = [...engine.voices].find((voice) => voice.titlePad);
  const end = pad.end - 0.025;
  context.advance(end - 0.04);
  const before = valueAt(pad.titlePad.points, context.currentTime);
  assert(before > 0.0001 && before < pad.titlePad.level);
  engine.update({ ...QUIET_STATE, corporation: "pan" });
  engine.setTitleScreen(false);
  const first = pad.titlePad.points[0];
  assert.equal(first.time, context.currentTime);
  assert(Math.abs(first.value - before) < 1e-12);
  assert.equal(pad.titlePad.points.length, 2, "a late pad release never regains full level");
  assert(Math.abs(pad.sources[0].stopAt - (end + 0.025)) < 1e-12);
  context.advance(end + 0.026 - context.currentTime);
  assert(pad.released);
  t.diagnostic(JSON.stringify({ retimeAt: first.time, gainBefore: before, gainAfter: first.value, scope: "recorded API" }));
  engine.dispose();
});

test("hard suspend during a deferred title resume releases sustained pads and rejects stale scheduling", async () => {
  const { engine, context, timers } = await titleFixture();
  context.advance(0.1);
  const pads = [...engine.voices].filter((voice) => voice.titlePad);
  assert(pads.length > 0);
  engine.suspend();
  assert(pads.every((pad) => pad.released && pad.nodes.every((node) => node.disconnects === 1)));
  let resolveResume;
  context.deferredResume = new Promise((resolve) => { resolveResume = resolve; });
  const pending = engine.resume();
  engine.suspend();
  resolveResume();
  await pending;
  assert.equal(engine.voices.size, 0);
  assert.equal(timers.size, 0);
  assert.equal(engine.diagnostics().enabled, false);
  engine.dispose();
});

test("repeated title/Continue/chase changes leave sustained pads finite and active sources bounded", async (t) => {
  const { engine, context, timers } = await titleFixture();
  let maxVoices = 0, maxSources = 0;
  for (let i = 0; i < 800; i++) {
    if (i % 11 === 0) engine.setTitleScreen(i % 44 < 22);
    if (i % 7 === 0) engine.update({ ...QUIET_STATE, area: "reach", corporation: "renko", combat: i % 28 < 14 ? 0.8 : 0, hostiles: i % 28 < 14 ? 2 : 0 });
    context.advance(0.025);
    for (const tick of timers.values()) tick();
    const music = [...engine.voices].filter((voice) => voice.lane === "music");
    const activeSources = music.reduce((sum, voice) => sum + voice.sources.length, 0);
    maxVoices = Math.max(maxVoices, music.length);
    maxSources = Math.max(maxSources, activeSources);
    assert(music.length <= 32 && activeSources <= 256);
    for (const pad of music.filter((voice) => voice.titlePad)) {
      assert(pad.titlePad.points.length <= 4);
      assert(Number.isFinite(pad.end) && pad.end >= context.currentTime);
      assert(pad.sources.every((source) => Number.isFinite(source.stopAt)));
      assert(pad.titlePad.points.every((point, n, points) => Number.isFinite(point.value) && point.value >= 0.0001 && (n === 0 || point.time >= points[n - 1].time)));
    }
  }
  engine.suspend();
  assert.equal(engine.voices.size, 0);
  assert.equal(timers.size, 0);
  t.diagnostic(JSON.stringify({ seconds: 20, maxVoices, maxSources, scope: "active recorded API resources" }));
  engine.dispose();
});
