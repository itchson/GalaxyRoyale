/** Width-specific independent recorded API checks, no PCM/listening claim. */
import assert from "node:assert/strict";
import { test } from "node:test";
import { QUIET_STATE } from "../../apps/game/src/audio-state.ts";
import { TITLE_THEME } from "../../apps/game/src/title-theme.ts";
import { titleFixture } from "./recorded-api-harness.mjs";

test("all eight stereo pad layers share a replaced stop and own every panner through cleanup", async (t) => {
  const { engine, context } = await titleFixture();
  const pad = [...engine.voices].find((voice) => voice.titlePad);
  const panners = pad.nodes.filter((node) => node.kind === "panner");
  assert.equal(pad.sources.length, 8);
  assert.equal(panners.length, 8);
  const oldStop = pad.end;
  context.advance(0.05);
  engine.update({ ...QUIET_STATE, corporation: "pan" });
  engine.setTitleScreen(false);
  const newStop = pad.end;
  assert(newStop > oldStop);
  assert(pad.sources.every((source) => source.stopAt === newStop && source.stops.length === 2));
  context.advance(oldStop + 0.001 - context.currentTime);
  assert(!pad.released && panners.every((node) => node.connections.has(pad.gain)));
  context.advance(newStop + 0.001 - context.currentTime);
  assert(pad.released && pad.nodes.every((node) => node.disconnects === 1));
  assert(pad.sources.every((source) => source.onended === null));
  t.diagnostic(JSON.stringify({ oldStop, newStop, panners: panners.length, scope: "recorded API" }));
  engine.dispose();
});

for (const stereo of [true, false]) {
  test(`one complete authored title loop remains finite with ${stereo ? "stereo" : "mono fallback"} routes`, async (t) => {
    const { engine, context, timers } = await titleFixture(stereo);
    const seconds = (TITLE_THEME.loopSteps / 4) * 60 / TITLE_THEME.bpm + 0.3;
    let maxVoices = 0, maxSources = 0, maxNodes = 0, maxPerVoice = 0, maxPanners = 0;
    const seenSteps = new Set();
    for (let elapsed = 0; elapsed < seconds; elapsed += 0.025) {
      context.advance(0.025);
      for (const tick of timers.values()) tick();
      const music = [...engine.voices].filter((voice) => voice.lane === "music");
      const diagnostics = engine.diagnostics();
      assert(music.length <= 32 && diagnostics.score.futureVoices <= 32);
      assert(music.every((voice) => voice.sources.length <= 9));
      assert(music.every((voice) => Number.isFinite(voice.end)));
      assert(music.filter((voice) => voice.start > context.currentTime).every((voice) => voice.start < context.currentTime + 0.18));
      for (const voice of music) {
        maxPerVoice = Math.max(maxPerVoice, voice.sources.length);
        const panners = voice.nodes.filter((node) => node.kind === "panner");
        maxPanners = Math.max(maxPanners, panners.length);
        assert(stereo || panners.length === 0);
        assert(panners.every((node) => Math.abs(node.pan.value) <= 0.32));
        assert(panners.every((node) => node.connections.has(voice.gain)));
        assert(voice.sources.every((source) => Number.isFinite(source.stopAt) && source.stopAt <= voice.end + 1e-9));
      }
      maxVoices = Math.max(maxVoices, music.length);
      maxSources = Math.max(maxSources, music.reduce((sum, voice) => sum + voice.sources.length, 0));
      maxNodes = Math.max(maxNodes, music.reduce((sum, voice) => sum + voice.nodes.length, 0));
      seenSteps.add(diagnostics.score.nextStep);
    }
    assert(engine.diagnostics().score.nextStep > TITLE_THEME.loopSteps);
    const heldNodes = [...engine.voices].flatMap((voice) => voice.nodes);
    engine.suspend();
    assert.equal(engine.voices.size, 0);
    assert.equal(timers.size, 0);
    assert(heldNodes.every((node) => node.disconnects === 1));
    t.diagnostic(JSON.stringify({ seconds, stereo, maxVoices, maxSources, maxNodes, maxPerVoice, maxPanners, observedStepPositions: seenSteps.size, scope: "sampled active recorded API resources; no DSP" }));
    engine.dispose();
  });
}
