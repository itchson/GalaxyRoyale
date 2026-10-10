/** Actual AudioEngine against a deterministic recorded Web Audio API.
 * This checks graph/resource/scheduling behavior, not browser DSP or audibility.
 */
import assert from "node:assert/strict";
import { test } from "node:test";
import { AudioEngine } from "../../apps/game/src/audio.ts";

class Param {
  value = 0;
  events = [];
  record(kind, value, time, extra) {
    assert(Number.isFinite(value));
    assert(Number.isFinite(time) && time >= 0);
    this.value = value;
    this.events.push({ kind, value, time, extra });
    return this;
  }
  setValueAtTime(v, t) {
    return this.record("set", v, t);
  }
  linearRampToValueAtTime(v, t) {
    return this.record("linear", v, t);
  }
  exponentialRampToValueAtTime(v, t) {
    assert(v > 0);
    return this.record("exponential", v, t);
  }
  setTargetAtTime(v, t, constant) {
    assert(Number.isFinite(constant) && constant > 0);
    return this.record("target", v, t, constant);
  }
}
class Node {
  connections = new Set();
  disconnects = 0;
  constructor(context, kind) {
    this.context = context;
    this.kind = kind;
    context.nodes.push(this);
  }
  connect(target) {
    this.connections.add(target);
    return target;
  }
  disconnect() {
    this.connections.clear();
    this.disconnects++;
  }
}
class Source extends Node {
  frequency = new Param();
  onended = null;
  starts = [];
  stops = [];
  ended = false;
  constructor(context, kind) {
    super(context, kind);
    context.sources.push(this);
  }
  start(t = this.context.currentTime) {
    assert(this.starts.length === 0);
    assert(Number.isFinite(t) && t >= 0);
    this.starts.push(t);
  }
  stop(t = this.context.currentTime) {
    assert(Number.isFinite(t) && t >= 0);
    this.stops.push(t);
    this.stopAt = t;
  }
}
class Context {
  static created = [];
  state = "suspended";
  sampleRate = 48000;
  currentTime = 0;
  nodes = [];
  sources = [];
  onstatechange = null;
  rejectResume = false;
  deferredResume = null;
  constructor() {
    this.destination = new Node(this, "destination");
    Context.created.push(this);
  }
  gainNode() {
    const n = new Node(this, "gain");
    n.gain = new Param();
    return n;
  }
  createGain() {
    return this.gainNode();
  }
  createOscillator() {
    return new Source(this, "oscillator");
  }
  createBufferSource() {
    return new Source(this, "buffer");
  }
  createBiquadFilter() {
    const n = new Node(this, "filter");
    n.frequency = new Param();
    n.Q = new Param();
    return n;
  }
  createDelay() {
    const n = new Node(this, "delay");
    n.delayTime = new Param();
    return n;
  }
  createDynamicsCompressor() {
    const n = new Node(this, "limiter");
    for (const field of ["threshold", "knee", "ratio", "attack", "release"])
      n[field] = new Param();
    return n;
  }
  createBuffer(channels, length, sampleRate) {
    assert.equal(channels, 1);
    assert.equal(sampleRate, this.sampleRate);
    const values = new Float32Array(length);
    return {
      getChannelData: (channel) => {
        assert.equal(channel, 0);
        return values;
      },
    };
  }
  changeState(state) {
    this.state = state;
    this.onstatechange?.();
  }
  async resume() {
    if (this.rejectResume) throw new Error("gesture required");
    if (this.deferredResume) await this.deferredResume;
    this.changeState("running");
  }
  async suspend() {
    this.changeState("suspended");
  }
  async close() {
    this.changeState("closed");
  }
  advance(seconds) {
    this.currentTime += seconds;
    for (const source of this.sources) {
      if (!source.ended && source.stopAt <= this.currentTime) {
        source.ended = true;
        source.onended?.();
      }
    }
  }
}
function browser() {
  const timers = new Map();
  let nextId = 1;
  globalThis.window = {
    AudioContext: Context,
    setInterval: (fn, ms) => {
      assert.equal(ms, 25);
      const id = nextId++;
      timers.set(id, fn);
      return id;
    },
    clearInterval: (id) => timers.delete(id),
  };
  return timers;
}
async function fixture() {
  const timers = browser();
  const engine = new AudioEngine();
  engine.setLevels(1, 0, 1);
  await engine.unlock();
  return { engine, context: engine.context, timers };
}
const cue = (kind, intensity = 1) => ({ kind, x: 10, y: -20, intensity });
const sfx = (engine) => [...engine.voices].filter((v) => v.lane === "sfx");
function play(engine, kind, intensity = 1) {
  engine.play(cue(kind, intensity));
  return sfx(engine).at(-1);
}
function isDrySfx(engine, voice) {
  assert.deepEqual([...voice.gain.connections], [engine.sfx]);
  assert.deepEqual([...engine.sfx.connections], [engine.master]);
  assert.equal(engine.music.gain.value, 0);
}

test("charge is one dry voice, two distinct rising tones, and ends by 125 ms", async () => {
  const { engine, context } = await fixture();
  const voice = play(engine, "mote-charge");
  assert.equal(sfx(engine).length, 1);
  assert.equal(voice.sources.length, 2);
  isDrySfx(engine, voice);
  assert.deepEqual(
    voice.sources.map((s) => s.type),
    ["square", "triangle"],
  );
  assert.deepEqual(
    voice.sources.map((s) => s.frequency.events.map((e) => e.value)),
    [
      [330, 660],
      [990, 1320],
    ],
  );
  assert.equal(voice.sources[1].starts[0], 0.025);
  assert(Math.max(...voice.sources.map((s) => s.stopAt)) <= 0.125);
  context.advance(0.126);
  assert.equal(sfx(engine).length, 0);
  assert(voice.nodes.every((n) => n.disconnects === 1));
  engine.dispose();
});

test("pulse is one dry voice with falling tones and original bandpass noise", async () => {
  const { engine, context } = await fixture();
  const voice = play(engine, "mote-pulse", 1 / 6);
  assert.equal(voice.sources.length, 3);
  assert.equal(voice.pending, 3);
  assert.equal(sfx(engine).length, 1);
  assert(Math.abs(voice.gain.gain.value - (0.75 + 0.25 / 6)) < 1e-12);
  isDrySfx(engine, voice);
  assert.deepEqual(
    voice.sources
      .slice(0, 2)
      .map((s) => s.frequency.events.map((e) => e.value)),
    [
      [720, 180],
      [92, 52],
    ],
  );
  const noise = voice.sources[2];
  assert.equal(noise.kind, "buffer");
  assert.equal(noise.buffer, engine.noise);
  const filter = [...noise.connections][0];
  assert.equal(filter.type, "bandpass");
  assert.equal(filter.frequency.value, 1750);
  assert(Math.max(...voice.sources.map((s) => s.stopAt)) <= 0.12);
  context.advance(0.121);
  assert.equal(sfx(engine).length, 0);
  engine.dispose();
});

test("100 mixed cues retain the existing 16-voice effects cap and release stolen nodes", async () => {
  const { engine, context } = await fixture();
  const seen = [];
  for (let i = 0; i < 100; i++) {
    const voice = play(
      engine,
      i % 3 === 0 ? "shot" : i % 3 === 1 ? "mote-charge" : "mote-pulse",
    );
    seen.push(voice);
    assert.equal(sfx(engine).length, Math.min(16, i + 1));
    assert(sfx(engine).reduce((total, v) => total + v.sources.length, 0) <= 48);
    assert(sfx(engine).reduce((total, v) => total + v.nodes.length, 0) <= 144);
  }
  assert.equal(seen.filter((v) => v.released).length, 84);
  assert(
    seen
      .filter((v) => v.released)
      .every(
        (v) =>
          v.nodes.every((n) => n.connections.size === 0) &&
          v.sources.every((s) => s.onended === null),
      ),
  );
  context.advance(1);
  assert.equal(sfx(engine).length, 0);
  engine.dispose();
});

test("separate volume buses and master mute remain authoritative for new cues", async () => {
  const { engine } = await fixture();
  engine.setLevels(0, 0.6, 0.3);
  play(engine, "mote-pulse");
  assert.deepEqual(
    [engine.master.gain.value, engine.music.gain.value, engine.sfx.gain.value],
    [0, 0.6, 0.3],
  );
  engine.setLevels(1, 0.8, 0);
  const voice = play(engine, "mote-charge");
  assert.equal(engine.sfx.gain.value, 0);
  assert.deepEqual([...voice.gain.connections], [engine.sfx]);
  engine.setLevels(Number.NaN, 2, -1);
  assert.deepEqual(
    [engine.master.gain.value, engine.music.gain.value, engine.sfx.gain.value],
    [0, 1, 0],
  );
  engine.dispose();
});

test("suspend, interrupted recovery, repeat gesture and disposal preserve one context", async () => {
  const count = Context.created.length;
  const { engine, context, timers } = await fixture();
  const voice = play(engine, "mote-charge");
  engine.suspend();
  assert.equal(engine.voices.size, 0);
  assert.equal(timers.size, 0);
  assert(voice.nodes.every((n) => n.connections.size === 0));
  const sources = context.sources.length;
  engine.play(cue("mote-pulse"));
  assert.equal(context.sources.length, sources);
  context.rejectResume = true;
  await engine.resume();
  assert.equal(timers.size, 0);
  assert.equal(engine.paused, true);
  context.rejectResume = false;
  await engine.unlock();
  await engine.unlock();
  assert.equal(Context.created.length, count + 1);
  assert.equal(timers.size, 1);
  context.changeState("suspended");
  assert.equal(timers.size, 0);
  context.changeState("running");
  assert.equal(timers.size, 1);
  const graph = [...engine.graph];
  play(engine, "mote-pulse");
  engine.dispose();
  engine.dispose();
  assert.equal(engine.voices.size, 0);
  assert.equal(timers.size, 0);
  assert.equal(context.state, "closed");
  assert(graph.every((n) => n.connections.size === 0));
  const disposedSources = context.sources.length;
  await engine.unlock();
  engine.play(cue("mote-pulse"));
  assert.equal(Context.created.length, count + 1);
  assert.equal(context.sources.length, disposedSources);
});

test("suspend during asynchronous resume prevents late scheduling and audio debt", async () => {
  const { engine, context, timers } = await fixture();
  engine.suspend();
  let resolve;
  context.deferredResume = new Promise((r) => {
    resolve = r;
  });
  const resume = engine.resume();
  engine.suspend();
  resolve();
  await resume;
  assert.equal(engine.paused, true);
  assert.equal(timers.size, 0);
  const sources = context.sources.length;
  engine.play(cue("mote-pulse"));
  assert.equal(context.sources.length, sources);
  engine.dispose();
});
