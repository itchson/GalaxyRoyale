import assert from "node:assert/strict";
import { test } from "node:test";
import { TitleTrack, TITLE_THEME } from "../../apps/game/src/title-theme.ts";

// Recorded Web Audio calls test lifecycle/loop contracts, not audible DSP.
function fixture(
  theme = { url: "/audio/theme.mp3", loopStart: 1, loopEnd: 4 },
) {
  const sources = [],
    gains = [];
  const context = {
    state: "running",
    currentTime: 10,
    decodeAudioData: async () => ({
      duration: 5,
      length: 240000,
      numberOfChannels: 2,
    }),
    createGain() {
      const node = {
        gain: {
          events: [],
          value: 0,
          setValueAtTime(v, t) {
            this.value = v;
            this.events.push(["set", v, t]);
          },
          setTargetAtTime(v, t, c) {
            this.value = v;
            this.events.push(["target", v, t, c]);
          },
          cancelScheduledValues(t) {
            this.events.push(["cancel", t]);
          },
        },
        connect() {},
        disconnect() {
          this.disconnected = true;
        },
      };
      gains.push(node);
      return node;
    },
    createBufferSource() {
      const source = {
        connect() {},
        disconnect() {
          this.disconnected = true;
        },
        start(...args) {
          this.starts = args;
        },
        stop(...args) {
          this.stops = args;
        },
      };
      sources.push(source);
      return source;
    },
  };
  return { context, sources, gains, track: new TitleTrack(context, {}, theme) };
}
async function withFetch(fn) {
  const previous = globalThis.fetch;
  let calls = 0;
  globalThis.fetch = async () => {
    calls++;
    return new Response(new Uint8Array([1, 2, 3]));
  };
  try {
    await fn(() => calls);
  } finally {
    globalThis.fetch = previous;
  }
}

test("unconfigured title requests nothing and starts no source", async () =>
  withFetch(async (calls) => {
    assert.equal(
      TITLE_THEME.url,
      null,
      "composer asset is intentionally pending",
    );
    const { track, sources } = fixture(TITLE_THEME);
    track.update(true, 1);
    await track.load();
    assert.equal(calls(), 0);
    assert.equal(sources.length, 0);
    assert.equal(track.status, "absent");
    track.dispose();
  }));
test("one source loops the authored range and never starts during loading", async () =>
  withFetch(async (calls) => {
    const { track, sources } = fixture();
    track.update(true, 0.45);
    assert.equal(sources.length, 0);
    await Promise.all([track.load(), track.load()]);
    assert.equal(calls(), 1);
    assert.equal(track.status, "ready");
    assert.equal(sources.length, 1);
    assert.equal(sources[0].loop, true);
    assert.deepEqual([sources[0].loopStart, sources[0].loopEnd], [1, 4]);
    assert.deepEqual(sources[0].starts, [10, 1]);
    track.update(true, 0);
    assert.equal(sources.length, 1);
    track.dispose();
    assert(sources[0].disconnected);
  }));
test("flight transition fades out and rapid title reentry disconnects old source", async () =>
  withFetch(async () => {
    const { track, sources, gains } = fixture();
    await track.load();
    track.update(true, 0.4);
    track.update(false, 0.4);
    assert.deepEqual(sources[0].stops, [11.2]);
    assert(gains[0].gain.events.some((e) => e[0] === "target" && e[1] === 0));
    track.update(true, 0.4);
    assert.equal(sources.length, 2);
    assert(sources[0].disconnected);
    assert(!sources[1].disconnected);
    track.update(false, 0);
    track.stop();
    assert(sources[1].disconnected);
    track.dispose();
  }));
test("suspended context does not start after decode; next resume uses retained buffer", async () =>
  withFetch(async () => {
    const { track, context, sources } = fixture();
    context.state = "suspended";
    track.update(true, 0.4);
    await track.load();
    assert.equal(sources.length, 0);
    context.state = "running";
    track.update(true, 0.4);
    assert.equal(sources.length, 1);
    track.dispose();
  }));
test("invalid loop bounds and decoded memory budget fail silently without sources", async () =>
  withFetch(async () => {
    for (const theme of [
      { url: "/a", loopStart: 5, loopEnd: 4 },
      { url: "/a", loopStart: NaN, loopEnd: null },
      { url: "/a", loopStart: 0, loopEnd: 8 },
    ]) {
      const { track, sources } = fixture(theme);
      track.update(true, 1);
      await track.load();
      assert.equal(track.status, "unavailable");
      assert.equal(sources.length, 0);
      track.dispose();
    }
    const { track, context, sources } = fixture();
    context.decodeAudioData = async () => ({
      duration: 100,
      length: 20000000,
      numberOfChannels: 2,
    });
    track.update(true, 1);
    await track.load();
    assert.equal(track.status, "unavailable");
    assert.equal(sources.length, 0);
    track.dispose();
  }));
test("late decode after disposal cannot connect or play", async () =>
  withFetch(async () => {
    let resolveDecode;
    const { track, context, sources, gains } = fixture();
    context.decodeAudioData = () =>
      new Promise((resolve) => {
        resolveDecode = resolve;
      });
    track.update(true, 1);
    const loading = track.load();
    while (!resolveDecode)
      await new Promise((resolve) => setImmediate(resolve));
    track.dispose();
    resolveDecode({ duration: 5, length: 240000, numberOfChannels: 2 });
    await loading;
    assert.equal(sources.length, 0);
    assert(gains[0].disconnected);
  }));
