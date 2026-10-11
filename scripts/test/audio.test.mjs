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

const dynamic = (patch = {}) => ({ ...QUIET_STATE, area: "reach", ...patch });
function stepAudio(context, timers, seconds = 0.025) {
  context.advance(seconds);
  for (const tick of timers.values()) tick();
  for (const [id, tail] of timers.timeouts)
    if (tail.due <= context.currentTime) {
      timers.timeouts.delete(id);
      tail.fn();
    }
}
test("source state cannot create audio before gesture, after suspension, or disposal", async () => {
  const count = Context.created.length;
  const timers = browser();
  const engine = new AudioEngine();
  engine.update(dynamic({ hull: 0.1, throttle: 1, engines: 2 }));
  engine.play(cue("ability-nova"));
  assert.equal(Context.created.length, count);
  assert.equal(engine.diagnostics().context, "locked");
  await engine.unlock();
  const context = engine.context;
  stepAudio(context, timers);
  assert.equal(engine.diagnostics().voices.continuous, 2);
  engine.suspend();
  const sources = context.sources.length;
  engine.update(dynamic({ hull: 0.1, throttle: 1, engines: 2 }));
  engine.play(cue("death"));
  assert.equal(context.sources.length, sources);
  assert.equal(engine.diagnostics().voices.sfx, 0);
  engine.dispose();
  engine.update(dynamic({ throttle: 1, engines: 2 }));
  await engine.unlock();
  assert.equal(Context.created.length, count + 1);
  assert.equal(engine.diagnostics().context, "locked");
  assert.equal(engine.diagnostics().disposed, true);
});
test("continuous engine and hull alarm reuse one voice each inside the shared16 cap", async () => {
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ hull: 0.1, speed: 0.6, throttle: 1, engines: 2 }));
  stepAudio(context, timers, 0.1);
  const engineVoice = engine.engineVoice,
    alarmVoice = engine.alarmVoice;
  assert(engineVoice && alarmVoice);
  assert.equal(engineVoice.sources.length, 3);
  assert.equal(alarmVoice.sources.length, 2);
  const continuousSources = context.sources.filter(
    (s) => engineVoice.sources.includes(s) || alarmVoice.sources.includes(s),
  );
  assert(
    continuousSources.every(
      (s) => s.starts.length === 1 && s.stops.length === 0,
    ),
  );
  for (let i = 0; i < 200; i++) {
    context.advance(1 / 60);
    engine.update(dynamic({ hull: 0.1, speed: 0.6, throttle: 1, engines: 2 }));
    assert.equal(engine.engineVoice, engineVoice);
    assert.equal(engine.alarmVoice, alarmVoice);
    assert.equal(engine.diagnostics().voices.continuous, 2);
  }
  for (let i = 0; i < 100; i++) {
    engine.play(cue(i % 2 ? "shot" : "ability-nova"));
    assert(engine.diagnostics().voices.sfx <= 16);
  }
  assert.equal(engine.diagnostics().voices.sfx, 16);
  assert(!engineVoice.released && !alarmVoice.released);
  engine.update(dynamic({ hull: 0.8 }));
  for (let i = 0; i < 120; i++) stepAudio(context, timers);
  assert.equal(engine.engineVoice, null);
  assert.equal(engine.alarmVoice, null);
  assert(
    continuousSources.every(
      (s) => s.onended === null && s.connections.size === 0,
    ),
  );
  assert(
    engineVoice.nodes.every((n) => n.disconnects === 1) &&
      alarmVoice.nodes.every((n) => n.disconnects === 1),
  );
  engine.dispose();
});
test("adaptive score follows actual biome/champion states and stays within32 music voices", async () => {
  const { engine, context, timers } = await fixture();
  engine.setLevels(1, 0.4, 0.8);
  engine.update(
    dynamic({
      biome: 4,
      combat: 0.8,
      hostiles: 5,
      hull: 0.2,
      throttle: 0.7,
      speed: 0.4,
      engines: 2,
    }),
  );
  for (let i = 0; i < 240; i++) stepAudio(context, timers);
  let d = engine.diagnostics();
  assert.equal(d.themes[0].index, 4);
  assert(d.combat > 0.75 && d.lowHull > 0);
  const instruments = [];
  for (let champion = 0; champion < 8; champion++) {
    engine.update(
      dynamic({ area: "arena", biome: 3, champion, combat: 0.6, hostiles: 1 }),
    );
    for (let i = 0; i < 220; i++) {
      stepAudio(context, timers);
      d = engine.diagnostics();
      assert(
        d.voices.music <= 32 && d.voices.sfx <= 16 && d.themes.length <= 2,
      );
    }
    assert.equal(d.themes[0].index, 6 + champion);
    instruments.push(d.themes[0].name);
  }
  assert.equal(new Set(instruments).size, 8);
  engine.update({ ...QUIET_STATE });
  for (let i = 0; i < 260; i++) stepAudio(context, timers);
  d = engine.diagnostics();
  assert.equal(d.themes[0].index, 0);
  assert(d.combat < 0.012 && d.lowHull < 0.002);
  assert.equal(d.voices.continuous, 0);
  engine.dispose();
});
test("distinct ability and shield timbres use one dry bounded voice each", async () => {
  const { engine, context } = await fixture();
  const signatures = new Set();
  for (const kind of [
    "shot",
    "hit",
    "player-hit",
    "shield",
    "boss-charge",
    "ability-echo",
    "ability-nova",
    "ability-wake",
    "ability-signature-6",
    "ability-signature-7",
    "ability-signature-8",
    "ability-signature-9",
    "ability-signature-12",
    "ability-signature-13",
    "ability-royal",
    "ability-repair",
  ]) {
    const voice = play(engine, kind, kind === "hit" ? 0.25 : 1);
    assert(voice && !voice.released);
    isDrySfx(engine, voice);
    assert(voice.sources.length <= 4);
    signatures.add(
      JSON.stringify(
        voice.sources.map((s) => [
          s.kind,
          s.type,
          s.frequency.events.map((e) => e.value),
        ]),
      ),
    );
    context.advance(1);
    assert.equal(sfx(engine).length, 0);
  }
  assert.equal(signatures.size, 16);
  const before = context.sources.length;
  engine.play(cue("unknown"));
  assert.equal(context.sources.length, before);
  engine.dispose();
});
test("wake sound rate limiting never schedules delayed debt and diagnostics are observational", async () => {
  const { engine, context } = await fixture();
  engine.play(cue("ability-wake"));
  const sources = context.sources.length;
  for (let i = 0; i < 100; i++) engine.play(cue("ability-wake"));
  assert.equal(context.sources.length, sources);
  const voices = engine.voices.size;
  for (let i = 0; i < 100; i++) engine.diagnostics();
  assert.equal(context.sources.length, sources);
  assert.equal(engine.voices.size, voices);
  context.advance(0.59);
  engine.play(cue("ability-wake"));
  assert.equal(context.sources.length, sources);
  context.advance(0.011);
  engine.play(cue("ability-wake"));
  assert.equal(context.sources.length, sources + 2);
  engine.dispose();
});
test("continuous effects honor independent mute buses, interruption and one-context recovery", async () => {
  const count = Context.created.length;
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ hull: 0.1, throttle: 1, speed: 0.5, engines: 2 }));
  stepAudio(context, timers, 0.1);
  const firstEngine = engine.engineVoice,
    firstAlarm = engine.alarmVoice;
  engine.setLevels(0, 0.8, 0.3);
  assert.equal(engine.master.gain.value, 0);
  engine.setLevels(0.7, 0, 0);
  assert.equal(engine.music.gain.value, 0);
  assert.equal(engine.sfx.gain.value, 0);
  assert.deepEqual([...firstEngine.gain.connections], [engine.sfx]);
  assert.deepEqual([...firstAlarm.gain.connections], [engine.sfx]);
  context.changeState("interrupted");
  assert.equal(timers.size, 0);
  assert.equal(engine.voices.size, 0);
  assert(firstEngine.released && firstAlarm.released);
  await engine.unlock();
  stepAudio(context, timers, 0.1);
  assert.equal(Context.created.length, count + 1);
  assert.equal(timers.size, 1);
  assert.equal(engine.diagnostics().voices.continuous, 2);
  const graph = [...engine.graph];
  const voices = [...engine.voices];
  engine.dispose();
  assert(
    voices.every(
      (v) => v.released && v.nodes.every((n) => n.connections.size === 0),
    ),
  );
  assert(graph.every((n) => n.connections.size === 0));
  assert.equal(timers.size, 0);
});
test("an older failed resume cannot suspend a newer successful audio intent", async () => {
  const { engine, context, timers } = await fixture();
  engine.suspend();
  let rejectOld;
  const old = new Promise((_, reject) => {
    rejectOld = reject;
  });
  let calls = 0;
  context.resume = async () => {
    if (++calls === 1) await old;
    else context.changeState("running");
  };
  const first = engine.resume();
  await engine.resume();
  rejectOld(new Error("older gesture failed"));
  await first;
  assert.equal(engine.paused, false);
  assert.equal(engine.musicWet.gain.value, 0.12);
  assert.equal(timers.size, 1);
  assert.equal(engine.diagnostics().enabled, true);
  engine.dispose();
});

test("healing during an OS interruption cannot revive a stale hull alarm", async () => {
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ hull: 0.1, throttle: 1, engines: 2 }));
  stepAudio(context, timers, 0.2);
  assert(engine.alarmVoice && engine.engineVoice);
  context.changeState("interrupted");
  engine.update(dynamic({ hull: 0.8, engines: 2 }));
  await engine.resume();
  stepAudio(context, timers, 0.1);
  assert.equal(engine.alarmVoice, null);
  assert.equal(engine.engineVoice, null);
  assert.equal(engine.diagnostics().lowHull, 0);
  engine.dispose();
});

test("death finish retains only the real existing cue and suspends after its bounded tail", async () => {
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ hull: 0.1, throttle: 1, speed: 0.5, engines: 2 }));
  stepAudio(context, timers, 0.1);
  const others = [...engine.voices];
  const death = play(engine, "death");
  engine.finishDeath();
  engine.finishDeath();
  assert.equal(engine.paused, true);
  assert.equal(timers.size, 0);
  assert.equal(timers.timeouts.size, 1);
  assert.equal(engine.voices.size, 1);
  assert(engine.voices.has(death));
  assert(others.every((v) => v.released));
  assert(!death.released);
  assert.equal(engine.engineVoice, null);
  assert.equal(engine.alarmVoice, null);
  assert.equal(engine.diagnostics().deathTail, true);
  assert.equal(engine.music.gain.value, 0);
  assert.equal(engine.musicWet.gain.value, 0);
  assert.equal(context.state, "running");
  const sources = context.sources.length;
  engine.update(dynamic({ hull: 0.1, throttle: 1, engines: 2 }));
  engine.play(cue("shot"));
  assert.equal(context.sources.length, sources);
  const tail = [...timers.timeouts.values()][0];
  assert(tail.ms <= 800 && tail.ms >= 700);
  stepAudio(context, timers, 0.6);
  assert(!death.released);
  stepAudio(context, timers, 0.2);
  assert(death.released);
  assert.equal(engine.voices.size, 0);
  assert.equal(context.state, "suspended");
  assert.equal(engine.diagnostics().deathTail, false);
  engine.dispose();
});
test("retry cancels a death timeout and a stale callback cannot suspend resumed music", async () => {
  const { engine, context, timers } = await fixture();
  const death = play(engine, "death");
  engine.finishDeath();
  const old = [...timers.timeouts.values()][0].fn;
  engine.update({ ...QUIET_STATE });
  await engine.resume();
  assert(death.released);
  assert.equal(timers.timeouts.size, 0);
  assert.equal(timers.size, 1);
  assert.equal(context.state, "running");
  old();
  assert.equal(timers.size, 1);
  assert.equal(context.state, "running");
  assert.equal(engine.musicWet.gain.value, 0.12);
  assert.equal(engine.paused, false);
  stepAudio(context, timers, 1);
  assert(engine.diagnostics().voices.music > 0);
  engine.dispose();
});
test("hidden/full suspend and disposal cut death tails without creating a context", async () => {
  const count = Context.created.length;
  const { engine, context, timers } = await fixture();
  const death = play(engine, "death");
  engine.finishDeath();
  const old = [...timers.timeouts.values()][0].fn;
  engine.suspend();
  assert(death.released);
  assert.equal(timers.timeouts.size, 0);
  assert.equal(context.state, "suspended");
  assert.equal(engine.voices.size, 0);
  await engine.resume();
  old();
  assert.equal(context.state, "running");
  play(engine, "death");
  engine.finishDeath();
  engine.dispose();
  assert.equal(timers.timeouts.size, 0);
  assert.equal(timers.size, 0);
  assert.equal(context.state, "closed");
  assert.equal(Context.created.length, count + 1);
  const empty = new AudioEngine();
  empty.finishDeath();
  assert.equal(Context.created.length, count + 1);
  assert.equal(empty.voices.size, 0);
  empty.dispose();
});

test("a late resume state event preserves only the bounded death tail and obeys hard suspend", async () => {
  const { engine, context, timers } = await fixture();
  let resolve;
  context.deferredResume = new Promise((r) => {
    resolve = r;
  });
  const pending = engine.resume();
  const death = play(engine, "death");
  engine.finishDeath();
  resolve();
  await pending;
  assert.equal(engine.voices.size, 1);
  assert(engine.voices.has(death));
  assert.equal(timers.size, 0);
  assert.equal(timers.timeouts.size, 1);
  assert.equal(engine.paused, true);
  assert.equal(context.state, "running");
  engine.suspend();
  context.changeState("running");
  assert.equal(context.state, "suspended");
  assert.equal(engine.voices.size, 0);
  assert.equal(timers.timeouts.size, 0);
  engine.dispose();
});
