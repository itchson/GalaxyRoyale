/** Actual AudioEngine against a deterministic recorded Web Audio API.
 * This checks graph/resource/scheduling behavior, not browser DSP or audibility.
 */
import assert from "node:assert/strict";
import { test } from "node:test";
import { AudioEngine } from "../../apps/game/src/audio.ts";
import { QUIET_STATE } from "../../apps/game/src/audio-state.ts";

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
  cancelScheduledValues(t) {
    assert(Number.isFinite(t) && t >= 0);
    this.events.push({ kind: "cancel", time: t });
    return this;
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
  createWaveShaper() {
    return new Node(this, "ceiling");
  }
  createStereoPanner() {
    const n = new Node(this, "panner");
    n.pan = new Param();
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
  timers.timeouts = new Map();
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
    setTimeout: (fn, ms) => {
      assert(Number.isFinite(ms) && ms >= 0 && ms <= 800);
      const id = nextId++;
      timers.timeouts.set(id, {
        fn,
        due: Context.created.at(-1).currentTime + ms / 1000,
        ms,
      });
      return id;
    },
    clearTimeout: (id) => timers.timeouts.delete(id),
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

// Evaluate recorded set/linear/exponential gain automation, not audio samples.
function gainAt(param, time) {
  let events = [];
  for (const event of param.events) {
    if (event.kind === "cancel")
      events = events.filter((e) => e.time < event.time);
    else events.push(event);
  }
  events = events
    .map((event, order) => ({ ...event, order }))
    .sort((a, b) => a.time - b.time || a.order - b.order);
  let previous = events[0];
  for (const event of events.slice(1)) {
    if (time < event.time) {
      const fraction = Math.min(
        1,
        Math.max(0, (time - previous.time) / (event.time - previous.time)),
      );
      if (event.kind === "linear")
        return previous.value + (event.value - previous.value) * fraction;
      if (event.kind === "exponential")
        return previous.value * (event.value / previous.value) ** fraction;
      return previous.value;
    }
    previous = event;
  }
  return previous.value;
}
function sourceEnvelope(source) {
  let node = [...source.connections][0];
  while (node.kind !== "gain") node = [...node.connections][0];
  return node.gain;
}
async function titleFixture(stereo = true) {
  const timers = browser(),
    engine = new AudioEngine();
  if (!stereo)
    window.AudioContext = class extends Context {
      constructor(...args) {
        super(...args);
        this.createStereoPanner = undefined;
      }
    };
  engine.setTitleScreen(true);
  engine.setLevels(0.75, 0.4, 0.75);
  await engine.unlock();
  return { engine, context: engine.context, timers };
}


export { Context, browser, fixture, titleFixture, gainAt, sourceEnvelope };
