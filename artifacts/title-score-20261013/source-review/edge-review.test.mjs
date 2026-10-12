/** Independent recorded-API edge checks; no native DSP/listening claim. */
import assert from "node:assert/strict";
import { test } from "node:test";
import { AudioEngine } from "../../apps/game/src/audio.ts";
import { QUIET_STATE } from "../../apps/game/src/audio-state.ts";
import { browser, fixture } from "./recorded-api-harness.mjs";

test("a 400ms title scheduler gap must not enqueue past onsets", async (t) => {
  const timers = browser();
  const engine = new AudioEngine();
  engine.setTitleScreen(true);
  await engine.unlock();
  const context = engine.context;
  context.advance(0.4);
  const before = context.sources.length;
  for (const tick of timers.values()) tick();
  const fresh = context.sources.slice(before);
  const overdue = fresh.filter((source) => source.starts[0] < context.currentTime);
  t.diagnostic(JSON.stringify({ now: context.currentTime, fresh: fresh.length,
    overdue: overdue.map((source) => ({ kind: source.kind, start: source.starts[0],
      lateBySeconds: context.currentTime - source.starts[0] })), scope: "recorded API" }));
  engine.dispose();
  assert.equal(overdue.length, 0, "missed notes should be skipped, not started immediately");
});

test("changing title while suspended must preserve the frozen beat on resume", async (t) => {
  const timers = browser();
  const engine = new AudioEngine();
  engine.setTitleScreen(true);
  await engine.unlock();
  const context = engine.context;
  context.advance(1.3);
  for (const tick of timers.values()) tick();
  const beat = engine.diagnostics().score.beat;
  engine.suspend();
  engine.update({ ...QUIET_STATE, corporation: "pan" });
  engine.setTitleScreen(false);
  await engine.resume();
  const after = engine.diagnostics().score;
  t.diagnostic(JSON.stringify({ beforeBeat: beat, afterBeat: after.beat,
    targetBpm: after.targetBpm, scope: "recorded API" }));
  engine.dispose();
  assert(Math.abs(after.beat - beat) < 1e-8);
  assert.equal(after.targetBpm, 120);
});
