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

test("authored title shares music controls/context and silences gameplay effects", async () => {
  const { engine, context, timers } = await fixture();
  engine.setLevels(0.6, 0.5, 0.7);
  engine.play({ kind: "shot", x: 0, y: 0, intensity: 1 });
  assert([...engine.voices].some((v) => v.lane === "sfx"));
  engine.setTitleScreen(true);
  assert.equal(engine.music.gain.value, 0.5);
  assert.equal(engine.musicWet.gain.value, 0.12);
  assert.equal(engine.master.gain.value, 0.6);
  assert.equal([...engine.voices].filter((v) => v.lane === "sfx").length, 0);
  const count = context.sources.length;
  context.advance(0.5);
  for (const tick of timers.values()) tick();
  assert(
    context.sources.length > count,
    "native title notes use the existing scheduler",
  );
  assert([...engine.voices].some((v) => v.cue === "music-title"));
  const sourceCount = context.sources.length;
  engine.play({ kind: "shot", x: 0, y: 0, intensity: 1 });
  assert.equal(
    context.sources.length,
    sourceCount,
    "title has no gameplay effects",
  );
  engine.setTitleScreen(false);
  assert.equal(engine.music.gain.value, 0.5);
  assert.equal(engine.musicWet.gain.value, 0.12);
  assert.equal(engine.context, context, "no second AudioContext");
  engine.suspend();
  assert.equal(timers.size, 0);
  engine.dispose();
});
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
  assert(Math.abs(voice.gain.gain.value - (0.75 + 0.25 / 6) * 0.85) < 1e-12);
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

test("thrust uses a calm sine fifth with one shared lowpass and duck stage", async () => {
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ throttle: 1, speed: 1, engines: 2 }));
  for (let i = 0; i < 80; i++) stepAudio(context, timers);
  const voice = engine.engineVoice;
  assert(voice && !voice.released);
  assert.deepEqual(
    voice.sources.slice(0, 2).map((s) => s.type),
    ["sine", "sine"],
  );
  const gains = voice.sources.map((s) => [...s.connections][0]);
  assert.deepEqual(
    gains.map((n) => n.gain.value),
    [0.045, 0.014, 0.008],
  );
  assert(gains.every((n) => [...n.connections][0] === engine.engineFilter));
  assert.equal(engine.engineFilter.type, "lowpass");
  assert.equal(engine.engineFilter.Q.value, 0.5);
  assert.equal(engine.engineFilter.frequency.value, 440);
  assert.deepEqual(
    [...engine.engineFilter.connections],
    [engine.engineDuckGain],
  );
  assert.deepEqual([...engine.engineDuckGain.connections], [voice.gain]);
  assert.deepEqual([...voice.gain.connections], [engine.sfx]);
  assert(voice.gain.gain.value >= 0.225 && voice.gain.gain.value <= 0.24);
  assert(
    Math.abs(
      voice.sources[0].frequency.value - 440 * 2 ** ((47 - 69) / 12) * 1.04,
    ) < 1e-9,
  );
  assert(
    Math.abs(
      voice.sources[1].frequency.value - 440 * 2 ** ((54 - 69) / 12) * 1.04,
    ) < 1e-9,
  );
  assert.equal(voice.sources[2].loop, true);
  assert(voice.sources.every((s) => s.starts.length === 1));
  engine.dispose();
});

test("throttle and speed retarget smooth continuous parameters without restarting the engine", async () => {
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ throttle: 0.2, speed: 0.1, engines: 2 }));
  stepAudio(context, timers, 0.1);
  const voice = engine.engineVoice;
  engine.update(dynamic({ throttle: 1, speed: 0.9, engines: 2 }));
  stepAudio(context, timers);
  assert.equal(engine.engineVoice, voice);
  assert.equal(voice.gain.gain.events.at(-1).kind, "target");
  assert.equal(voice.gain.gain.events.at(-1).extra, 0.18);
  for (const source of engine.engineOscillators) {
    assert.equal(source.frequency.events.at(-1).kind, "target");
    assert.equal(source.frequency.events.at(-1).extra, 0.24);
    assert.equal(source.starts.length, 1);
  }
  assert.equal(engine.engineFilter.frequency.events.at(-1).extra, 0.18);
  const before = voice.gain.gain.value;
  engine.update(dynamic({ engines: 2 }));
  stepAudio(context, timers, 0.025);
  assert(voice.gain.gain.value > 0 && voice.gain.gain.value < before);
  for (let i = 0; i < 100; i++) stepAudio(context, timers);
  assert(voice.released && engine.engineVoice === null);
  assert(voice.nodes.every((n) => n.disconnects === 1));
  engine.dispose();
});

test("only admitted important cues briefly duck propulsion without pumping music", async () => {
  const { engine, context, timers } = await fixture();
  engine.setLevels(0.8, 0.45, 0.7);
  engine.update(dynamic({ throttle: 1, speed: 0.5, engines: 2 }));
  stepAudio(context, timers, 0.1);
  const voice = engine.engineVoice;
  const musicAutomation = engine.music.gain.events.length;
  for (let i = 0; i < 30; i++) {
    engine.play(cue("shot"));
    engine.play(cue("hit", 0.25));
  }
  assert.equal(engine.engineDuckGain.gain.value, 1);
  engine.play(cue("unknown"));
  assert.equal(engine.diagnostics().engineDuck, 1);
  engine.play(cue("hit", 0.9));
  assert.equal(engine.diagnostics().engineDuck, 0.35);
  assert.equal(engine.engineDuckGain.gain.events.at(-1).extra, 0.015);
  const heldUntil = engine.engineDuckUntil;
  stepAudio(context, timers, 0.1);
  assert.equal(engine.engineVoice, voice);
  assert.equal(engine.engineDuckGain.gain.value, 0.35);
  while (context.currentTime <= heldUntil + 0.025) stepAudio(context, timers);
  assert.equal(engine.engineDuckGain.gain.value, 1);
  assert.equal(engine.engineDuckGain.gain.events.at(-1).extra, 0.22);
  assert.equal(engine.music.gain.events.length, musicAutomation);
  assert.equal(engine.music.gain.value, 0.45);
  assert(engine.diagnostics().voices.sfx <= 16);
  engine.dispose();
});

test("low-hull priority and strongest cue holds fade away after actual healing", async () => {
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ hull: 0.08, throttle: 1, engines: 2 }));
  for (let i = 0; i < 100; i++) stepAudio(context, timers);
  assert(
    engine.diagnostics().engineDuck >= 0.6 &&
      engine.diagnostics().engineDuck < 0.601,
  );
  engine.play(cue("shield"));
  const shieldTarget = engine.diagnostics().engineDuck;
  engine.play(cue("mote-charge"));
  assert.equal(engine.diagnostics().engineDuck, shieldTarget);
  assert(shieldTarget >= 0.21 && shieldTarget < 0.211);
  engine.update(dynamic({ hull: 0.8, throttle: 1, engines: 2 }));
  for (let i = 0; i < 160; i++) stepAudio(context, timers);
  assert.equal(engine.alarmVoice, null);
  assert(engine.diagnostics().engineDuck > 0.999);
  assert.equal(engine.engineDuckDepth, 1);
  engine.dispose();
});

test("hidden stop, retry and death cleanup cannot carry a propulsion duck into the next life", async () => {
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ throttle: 1, engines: 2 }));
  stepAudio(context, timers, 0.1);
  const old = engine.engineVoice;
  engine.play(cue("explosion"));
  assert(engine.engineDuckUntil > context.currentTime);
  engine.suspend();
  assert(old.released && old.nodes.every((n) => n.disconnects === 1));
  assert.equal(engine.engineDuckGain, null);
  assert.equal(engine.engineDuckUntil, 0);
  assert.equal(engine.diagnostics().engineDuck, 1);
  await engine.resume();
  stepAudio(context, timers, 0.1);
  assert.equal(engine.engineDuckGain.gain.value, 1);
  assert.equal(engine.diagnostics().voices.continuous, 1);
  engine.play(cue("death"));
  engine.finishDeath();
  assert.equal(engine.engineVoice, null);
  assert.equal(engine.engineDuckGain, null);
  assert.equal(engine.engineDuckUntil, 0);
  engine.dispose();
  assert.equal(engine.voices.size, 0);
});

// Sum recorded source envelope maxima along the real graph, ignoring filters,
// phase cancellation and compressor dynamics. This is scalar gain evidence,
// not a sample-peak, RMS or audibility measurement.
function envelopeSum(voice) {
  const paramPeak = (p) => Math.max(p.value, ...p.events.map((e) => e.value));
  const pathGain = (node) => {
    if (node === voice.gain) return paramPeak(node.gain);
    const next = [...node.connections][0];
    assert(next, "every live source reaches its real voice bus");
    return (node.kind === "gain" ? paramPeak(node.gain) : 1) * pathGain(next);
  };
  return voice.sources.reduce((sum, source) => {
    const peak = source.kind === "buffer" ? 0.7 : 1;
    return sum + peak * pathGain(source);
  }, 0);
}
test("real gain envelopes leave a quieter engine bed and finite bounded stacked mix", async (t) => {
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ throttle: 1, speed: 1, engines: 2 }));
  for (let i = 0; i < 80; i++) stepAudio(context, timers);
  const engineEnvelope = envelopeSum(engine.engineVoice);
  const historicalEngineEnvelope = 0.42 * (0.08 + 0.055 + 0.045 * 0.7);
  const priorWarmEnvelope = 0.32 * (0.055 + 0.025 + 0.02 * 0.7);
  assert(engineEnvelope <= 0.015504001);
  assert(engineEnvelope / historicalEngineEnvelope < 0.223);
  assert(engineEnvelope / priorWarmEnvelope < 0.516);
  let maximumCue = { kind: "", envelope: 0 };
  for (const kind of [
    "mote-charge",
    "mote-pulse",
    "shot",
    "hit",
    "player-hit",
    "explosion",
    "death",
    "pickup",
    "boss",
    "boss-charge",
    "shield",
    "upgrade",
    "dock",
    "discovery",
    "champion",
    "royale-round",
    "credits",
    "ability-echo",
    "ability-nova",
    "ability-wake",
    "ability-royal",
    "ability-repair",
    ...[6, 7, 8, 9, 12, 13].map((i) => `ability-signature-${i}`),
  ]) {
    const voice = play(engine, kind);
    const value = envelopeSum(voice);
    assert(
      Number.isFinite(value) && value > 0 && value <= 0.55,
      `${kind}: ${value}`,
    );
    if (value > maximumCue.envelope) maximumCue = { kind, envelope: value };
    context.advance(1);
  }
  for (let i = 0; i < 16; i++) play(engine, "death");
  const stackedEffects = sfx(engine).reduce(
    (total, v) => total + envelopeSum(v),
    0,
  );
  assert.equal(sfx(engine).length, 16);
  assert(stackedEffects <= maximumCue.envelope * 16 + 1e-8);
  const limiter = engine.graph.find((n) => n.kind === "limiter");
  assert.equal(limiter.threshold.value, -12);
  assert.equal(limiter.ratio.value, 12);
  assert.equal(limiter.attack.value, 0.003);
  assert.equal(limiter.release.value, 0.18);
  t.diagnostic(
    JSON.stringify({
      engineEnvelope,
      historicalEngineEnvelope,
      priorWarmEnvelope,
      historicalScalarChangeDb:
        20 * Math.log10(engineEnvelope / historicalEngineEnvelope),
      priorWarmScalarChangeDb:
        20 * Math.log10(engineEnvelope / priorWarmEnvelope),
      maximumCue,
      stackedEffects,
      defaultBusScalar: stackedEffects * 0.75 * 0.75,
    }),
  );
  engine.dispose();
});

test("combined adaptive score, alarm, propulsion and real cues retain finite bus gains and voice limits", async (t) => {
  const { engine, context, timers } = await fixture();
  engine.setLevels(0.75, 0.4, 0.75);
  let musicEnvelope = 0,
    effectsEnvelope = 0,
    combinedEnvelope = 0;
  let maximumMusicVoices = 0,
    maximumSfxVoices = 0;
  const kinds = [
    "shot",
    "shield",
    "player-hit",
    "mote-charge",
    "ability-echo",
    "ability-nova",
    "explosion",
  ];
  for (let i = 0; i < 960; i++) {
    engine.update(
      dynamic({
        area: "arena",
        champion: Math.floor(i / 120),
        hull: 0.08,
        combat: 1,
        hostiles: 12,
        boss: true,
        throttle: 1,
        speed: 1,
        engines: 2,
      }),
    );
    stepAudio(context, timers);
    if (i % 12 === 0)
      engine.play(cue(kinds[Math.floor(i / 12) % kinds.length]));
    let music = 0,
      effects = 0;
    for (const voice of engine.voices) {
      const value = envelopeSum(voice);
      assert(Number.isFinite(value));
      if (voice.lane === "music") music += value;
      else effects += value;
    }
    const d = engine.diagnostics();
    assert(
      d.voices.music <= 32 && d.voices.sfx <= 16 && d.voices.continuous === 2,
    );
    maximumMusicVoices = Math.max(maximumMusicVoices, d.voices.music);
    maximumSfxVoices = Math.max(maximumSfxVoices, d.voices.sfx);
    musicEnvelope = Math.max(musicEnvelope, music);
    effectsEnvelope = Math.max(effectsEnvelope, effects);
    combinedEnvelope = Math.max(
      combinedEnvelope,
      0.75 * (0.4 * music + 0.75 * effects),
    );
  }
  // Account separately for music's unchanged .12 wet/.16 feedback scalar.
  const conservativeEchoEnvelope =
    0.75 *
    (0.4 * musicEnvelope * (1 + 0.12 / (1 - 0.16)) + 0.75 * effectsEnvelope);
  assert(
    Number.isFinite(conservativeEchoEnvelope) &&
      conservativeEchoEnvelope > combinedEnvelope,
  );
  t.diagnostic(
    JSON.stringify({
      samples: 960,
      seconds: 24,
      maximumMusicVoices,
      maximumSfxVoices,
      musicEnvelope,
      effectsEnvelope,
      combinedEnvelope,
      conservativeEchoEnvelope,
      scope: "recorded graph envelope sums; no DSP peak/RMS",
    }),
  );
  engine.dispose();
});

test("all real buses reach one final bounded soft ceiling after compression", async () => {
  const { engine, context } = await fixture();
  const limiter = engine.graph.find((n) => n.kind === "limiter");
  const ceiling = engine.graph.find((n) => n.kind === "ceiling");
  assert(ceiling);
  assert.equal(engine.graph.filter((n) => n.kind === "ceiling").length, 1);
  assert.deepEqual([...engine.master.connections], [limiter]);
  assert.deepEqual([...limiter.connections], [ceiling]);
  assert.deepEqual([...ceiling.connections], [context.destination]);
  assert.equal(ceiling.oversample, "none");
  assert(ceiling.curve instanceof Float32Array);
  assert.equal(ceiling.curve.length, 4097);
  let previous = -Infinity;
  for (let i = 0; i < ceiling.curve.length; i++) {
    const input = (i / 4096) * 2 - 1,
      output = ceiling.curve[i];
    assert(Number.isFinite(output) && Math.abs(output) < 0.915);
    assert(output >= previous);
    assert(Math.abs(output + ceiling.curve[4096 - i]) < 1e-7);
    if (Math.abs(input) <= 0.78) assert(Math.abs(output - input) < 1e-7);
    else assert(Math.abs(output) <= Math.abs(input) + 1e-7);
    previous = output;
  }
  engine.setLevels(1, 1, 1);
  for (let i = 0; i < 100; i++) engine.play(cue("death"));
  assert.equal(sfx(engine).length, 16);
  assert.deepEqual([...engine.sfx.connections], [engine.master]);
  engine.suspend();
  await engine.resume();
  assert.equal(
    engine.graph.find((n) => n.kind === "ceiling"),
    ceiling,
  );
  assert.equal(context.nodes.filter((n) => n.kind === "ceiling").length, 1);
  engine.dispose();
  assert.equal(ceiling.disconnects, 1);
  assert.equal(ceiling.connections.size, 0);
});

test("calm thrust pitch stays within a tiny musical bend while breathing reuses all sources", async () => {
  const { engine, context, timers } = await fixture();
  engine.update(dynamic({ throttle: 1, speed: 1, engines: 2 }));
  for (let i = 0; i < 80; i++) stepAudio(context, timers);
  const voice = engine.engineVoice;
  const sources = [...voice.sources];
  const firstNote = 440 * 2 ** ((47 - 69) / 12),
    fifth = 440 * 2 ** ((54 - 69) / 12);
  let minGain = 1,
    maxGain = 0;
  for (let i = 0; i < 200; i++) {
    stepAudio(context, timers);
    const gain = voice.gain.gain.value;
    minGain = Math.min(minGain, gain);
    maxGain = Math.max(maxGain, gain);
    assert(gain >= 0.2255 && gain <= 0.24);
    assert.equal(engine.engineVoice, voice);
    assert.deepEqual(voice.sources, sources);
  }
  assert(maxGain - minGain > 0.014);
  for (const throttle of [0, 0.25, 0.5, 0.75, 1])
    for (const speed of [0, 0.25, 0.5, 0.75, 1]) {
      engine.update(dynamic({ throttle, speed, engines: 2 }));
      stepAudio(context, timers);
      const frequencies = engine.engineOscillators.map(
        (s) => s.frequency.value,
      );
      const bend = 1 + speed * 0.025 + throttle * 0.015;
      assert(Math.abs(frequencies[0] - firstNote * bend) < 1e-9);
      assert(Math.abs(frequencies[1] - fifth * bend) < 1e-9);
      assert(1200 * Math.log2(frequencies[0] / firstNote) < 70);
      assert.equal(voice.sources.length, 3);
      assert(voice.sources.every((s) => s.starts.length === 1));
      assert.equal(engine.diagnostics().voices.continuous, 1);
    }
  engine.suspend();
  assert(voice.released);
  assert(voice.nodes.every((n) => n.disconnects === 1));
  await engine.resume();
  stepAudio(context, timers);
  assert.equal(engine.diagnostics().voices.continuous, 1);
  assert.notEqual(engine.engineVoice, voice);
  engine.dispose();
});

test("corporate home and quiet Iron exploration produce actual softer score nodes", async () => {
  const { engine, context, timers } = await fixture();
  engine.setLevels(0.75, 0.4, 0.75);
  for (const [corporation, index] of [
    ["pan", 15],
    ["renko", 16],
    ["hugo", 17],
  ]) {
    engine.update(dynamic({ area: "haven", corporation }));
    for (let i = 0; i < 220; i++) stepAudio(context, timers);
    assert.equal(engine.diagnostics().themes[0].index, index);
    const existing = new Set(engine.voices);
    engine.scoreStep(context.currentTime, 0, 60 / 112 / 4);
    const actual = [...engine.voices].filter((v) => !existing.has(v));
    assert(actual.some((v) => v.theme === index));
    assert(actual.every((v) => v.cue !== "music-combat"));
    assert(
      actual
        .flatMap((v) => v.sources)
        .every((s) => s.type === "sine" || s.type === "triangle"),
    );
    assert.equal(engine.diagnostics().corporation, corporation);
  }
  engine.update(dynamic({ area: "reach", biome: 3 }));
  for (let i = 0; i < 220; i++) stepAudio(context, timers);
  const before = new Set(engine.voices);
  engine.scoreStep(context.currentTime, 0, 60 / 112 / 4);
  const quiet = [...engine.voices].filter((v) => !before.has(v));
  assert(quiet.some((v) => v.theme === 3));
  assert(quiet.flatMap((v) => v.sources).every((s) => s.type !== "square"));
  assert.equal(engine.mix.boss, 0);
  engine.dispose();
});

test("one shared filtered power-chord voice follows aggro and adds only the actual boss counter-note", async () => {
  const { engine, context, timers } = await fixture();
  engine.setLevels(0.75, 0.4, 0.75);
  const length = 60 / 112 / 4;
  engine.update(dynamic({ combat: 0.8, hostiles: 4 }));
  for (let i = 0; i < 120; i++) stepAudio(context, timers);
  const before = new Set(engine.voices);
  engine.scoreStep(context.currentTime, 0, length);
  const combat = [...engine.voices].filter(
    (v) => !before.has(v) && v.cue === "music-combat",
  );
  assert.equal(combat.length, 1);
  assert.equal(combat[0].sources.length, 2);
  assert(combat[0].sources.every((s) => s.type === "triangle"));
  const filters = combat[0].nodes.filter((n) => n.kind === "filter");
  assert.equal(filters.length, 2);
  assert(
    filters.every(
      (n) =>
        n.type === "lowpass" && n.Q.value === 0.5 && n.frequency.value === 1000,
    ),
  );
  const curves = combat[0].nodes.filter((n) => n.kind === "ceiling");
  assert.equal(curves.length, 2);
  assert(
    curves.every((n) => n.oversample === "none" && n.curve.length === 257),
  );
  assert(
    curves.every((n) =>
      [...n.curve].every((v) => Number.isFinite(v) && Math.abs(v) <= 1),
    ),
  );
  assert.deepEqual([...combat[0].gain.connections], [engine.music]);
  assert.equal(engine.sfx.gain.value, 0.75);
  assert.equal(engine.music.gain.value, 0.4);
  engine.update(dynamic({ combat: 0.8, hostiles: 1, boss: true }));
  for (let i = 0; i < 100; i++) stepAudio(context, timers);
  assert(engine.diagnostics().bossLayer > 0.99);
  const previous = new Set(engine.voices);
  engine.scoreStep(context.currentTime, 4, length);
  const boss = [...engine.voices].filter(
    (v) => !previous.has(v) && v.cue === "music-combat",
  );
  assert.equal(boss.length, 1);
  assert.equal(boss[0].sources.length, 3);
  assert(
    boss[0].nodes
      .filter((n) => n.kind === "filter")
      .every((n) => n.frequency.value > 1600),
  );
  for (let i = 0; i < 1000; i++) {
    if (i % 40 === 0)
      engine.update(
        dynamic({
          area: i % 80 ? "reach" : "arena",
          champion: i % 8,
          combat: 0.8,
          hostiles: 1,
          boss: true,
          corporation: "pan",
          throttle: 1,
          engines: 2,
          hull: 0.1,
        }),
      );
    stepAudio(context, timers);
    assert(
      engine.diagnostics().voices.music <= 32 &&
        engine.diagnostics().voices.sfx <= 16,
    );
    assert(engine.diagnostics().themes.length <= 2);
  }
  const nodes = [...engine.voices].flatMap((v) => v.nodes);
  engine.suspend();
  assert.equal(engine.mix.boss, 0);
  assert.equal(engine.voices.size, 0);
  assert(nodes.every((n) => n.disconnects === 1));
  engine.update(dynamic({ area: "haven", boss: true, combat: 1, hostiles: 1 }));
  await engine.resume();
  for (let i = 0; i < 100; i++) stepAudio(context, timers);
  assert.equal(engine.mix.boss, 0);
  assert.equal(engine.mix.combat, 0);
  assert(![...engine.voices].some((v) => v.cue === "music-combat"));
  engine.dispose();
});

test("Continue at different phrase positions preserves beat and shares the selected corporate arrangement", async (t) => {
  for (const [corporation, index, position] of [
    ["pan", 15, 0.3],
    ["renko", 16, 0.925],
    ["hugo", 17, 6.8],
  ]) {
    const timers = browser(),
      engine = new AudioEngine();
    engine.setTitleScreen(true);
    engine.setLevels(0.75, 0.4, 0.75);
    await engine.unlock();
    const context = engine.context,
      clock = engine.clock;
    for (let elapsed = 0; elapsed < position; elapsed += 0.025)
      stepAudio(context, timers);
    const beat = engine.diagnostics().score.beat;
    const future = [...engine.voices].filter(
      (v) => v.lane === "music" && v.start > context.currentTime,
    );
    engine.update({ ...QUIET_STATE, corporation });
    engine.setTitleScreen(false);
    const arrival = engine.diagnostics();
    assert(
      Math.abs(arrival.score.beat - beat) < 1e-8,
      "Continue retains the heard phrase phase",
    );
    assert.equal(engine.clock, clock);
    assert.equal(arrival.score.targetBpm, 120);
    assert.equal(arrival.score.rampEndBeat - arrival.score.rampStartBeat, 8);
    assert(
      future.every((v) => v.released),
      "queued title notes are canceled before replacement",
    );
    for (let i = 0; i < 180; i++) {
      stepAudio(context, timers);
      const d = engine.diagnostics();
      assert(
        d.voices.music <= 32 && d.voices.sfx <= 16 && d.score.futureVoices <= 8,
      );
      assert(
        [...engine.voices]
          .filter((v) => v.lane === "music")
          .every((v) => v.gain.connections.has(engine.music)),
      );
    }
    const settled = engine.diagnostics();
    assert.equal(settled.themes[0].index, index);
    assert(Math.abs(settled.score.bpm - 120) < 1e-8);
    assert(settled.title.blend < 0.001);
    assert.equal(timers.size, 1, "title/hub share one scheduler");
    t.diagnostic(
      JSON.stringify({
        corporation,
        atSeconds: position,
        beat,
        settledBpm: settled.score.bpm,
        scope: "recorded WebAudio calls; no DSP/listening",
      }),
    );
    engine.dispose();
  }
});

test("real combat intent retimes queued notes; release hold and re-chase never restart the clock", async () => {
  const { engine, context, timers } = await fixture();
  for (let i = 0; i < 16; i++) stepAudio(context, timers);
  const clock = engine.clock,
    beat = engine.diagnostics().score.beat;
  const oldFuture = [...engine.voices].filter(
    (v) => v.lane === "music" && v.start > context.currentTime,
  );
  engine.update(dynamic({ combat: 0.8, hostiles: 3 }));
  const chase = engine.diagnostics().score;
  assert.equal(engine.clock, clock);
  assert(Math.abs(chase.beat - beat) < 1e-8);
  assert.equal(chase.targetBpm, 166);
  assert.equal(chase.rampEndBeat - chase.rampStartBeat, 4);
  const boundary = clock.timeAtBeat(chase.rampStartBeat);
  const canceled = oldFuture.filter((v) => v.start >= boundary - 1e-8);
  assert(
    canceled.length > 0,
    "fixture contains notes beyond the changed boundary",
  );
  assert(
    canceled.every(
      (v) =>
        v.released &&
        v.sources.every((s) => s.connections.size === 0 && s.onended === null),
    ),
  );
  const audiblePads = [...engine.voices].filter(
    (v) => v.harmonicEndBeat !== null && v.start < context.currentTime,
  );
  assert(audiblePads.length > 0);
  assert(
    audiblePads.every((v) =>
      v.sources.every(
        (s) =>
          Math.abs(s.stopAt - (clock.timeAtBeat(v.harmonicEndBeat) + 0.025)) <
          1e-8,
      ),
    ),
    "audible chord tails stop at the retimed harmonic boundary",
  );
  for (let i = 0; i < 160; i++) stepAudio(context, timers);
  assert(Math.abs(engine.diagnostics().score.bpm - 166) < 1e-8);
  engine.update(dynamic());
  const release = engine.diagnostics().score;
  assert.equal(release.targetBpm, 120);
  assert.equal(release.rampEndBeat - release.rampStartBeat, 8);
  assert(
    Math.abs(release.rampStartBeat - (Math.ceil(release.beat - 1e-9) + 2)) <
      1e-8,
  );
  for (let i = 0; i < 6; i++) stepAudio(context, timers);
  engine.update(dynamic({ combat: 0.8, hostiles: 1 }));
  assert.equal(engine.diagnostics().score.targetBpm, 166);
  for (let i = 0; i < 100; i++) stepAudio(context, timers);
  assert(
    Math.abs(engine.diagnostics().score.bpm - 166) < 1e-8,
    "pending release cannot slow re-chase",
  );
  assert.equal(engine.clock, clock);
  assert.equal(timers.size, 1);
  engine.dispose();
});

test("scheduler stalls skip missed notes and bound only the next lookahead queue", async () => {
  for (const title of [false, true]) {
    for (const delay of [0.4, 20]) {
      const { engine, context, timers } = await fixture();
      if (title) engine.setTitleScreen(true);
      const clock = engine.clock;
      context.advance(delay);
      const before = context.sources.length;
      for (const tick of timers.values()) tick();
      const fresh = context.sources.slice(before);
      assert(fresh.length > 0);
      assert(
        fresh.every((source) => source.starts[0] >= context.currentTime),
        "no burst of overdue note debt",
      );
      assert(
        fresh.every((source) => source.starts[0] < context.currentTime + 0.18),
      );
      const d = engine.diagnostics();
      assert(
        d.score.nextStep <=
          Math.ceil(clock.beatAtTime(context.currentTime + 0.18) * 4),
      );
      assert(d.score.futureVoices <= 8 && d.voices.music <= 32);
      engine.dispose();
    }
  }
});

test("native title mute, interruption and late resume retain one clock and no stale scheduled notes", async () => {
  const count = Context.created.length,
    timers = browser(),
    engine = new AudioEngine();
  engine.setTitleScreen(true);
  await engine.unlock();
  const context = engine.context,
    clock = engine.clock;
  for (let i = 0; i < 60; i++) stepAudio(context, timers);
  engine.setLevels(0, 0, 0);
  assert.equal(engine.master.gain.value, 0);
  assert.equal(engine.music.gain.value, 0);
  const beat = engine.diagnostics().score.beat,
    bpm = engine.diagnostics().score.bpm;
  const scheduled = [...engine.voices];
  engine.suspend();
  assert.equal(engine.voices.size, 0);
  assert.equal(timers.size, 0);
  assert(
    scheduled.every(
      (v) => v.released && v.nodes.every((n) => n.connections.size === 0),
    ),
  );
  await engine.resume();
  assert.equal(engine.clock, clock);
  assert.equal(Context.created.length, count + 1);
  assert(Math.abs(engine.diagnostics().score.beat - beat) < 1e-8);
  assert(Math.abs(engine.diagnostics().score.bpm - bpm) < 1e-8);
  assert.equal(engine.music.gain.value, 0);
  assert.equal(timers.size, 1);
  engine.setLevels(0.75, 0.4, 0.75);
  stepAudio(context, timers, 0.1);
  assert([...engine.voices].some((v) => v.cue === "music-title"));
  engine.dispose();
  assert.equal(engine.voices.size, 0);
  assert.equal(engine.clock, null);
  assert.equal(timers.size, 0);
});

test("rapid title, chase, release and area changes keep music/effects budgets and finite gain events", async (t) => {
  const { engine, context, timers } = await fixture();
  let maxMusic = 0,
    maxFuture = 0;
  for (let i = 0; i < 1200; i++) {
    if (i % 7 === 0) engine.setTitleScreen(i % 49 < 14);
    if (i % 5 === 0)
      engine.update(
        dynamic({
          area: i % 31 === 0 ? "haven" : "reach",
          biome: i % 6,
          corporation: ["pan", "renko", "hugo"][i % 3],
          combat: i % 25 < 15 ? 0.8 : 0,
          hostiles: i % 25 < 15 ? 3 : 0,
          boss: i % 60 < 10,
          engines: 2,
          throttle: 0.7,
          hull: 0.1,
        }),
      );
    if (i % 11 === 0) engine.play(cue("ability-nova"));
    stepAudio(context, timers);
    const d = engine.diagnostics();
    maxMusic = Math.max(maxMusic, d.voices.music);
    maxFuture = Math.max(maxFuture, d.score.futureVoices);
    assert(
      d.voices.music <= 32 && d.voices.sfx <= 16 && d.score.futureVoices <= 32,
    );
    assert(
      [...engine.voices]
        .filter((v) => v.lane === "music" && v.start > context.currentTime)
        .every((v) => v.start < context.currentTime + 0.18 + 1e-9),
      "future note queue stays inside real lookahead",
    );
    assert(
      d.score.segments <= 16 &&
        Number.isFinite(d.score.bpm) &&
        Number.isFinite(d.score.beat),
    );
    if (engine.titleScreen)
      assert.equal(
        d.voices.continuous,
        0,
        "title cannot restart faded engine/alarm layers",
      );
    assert.equal(timers.size, 1);
  }
  t.diagnostic(
    JSON.stringify({
      seconds: 30,
      maxMusic,
      maxFuture,
      scope: "recorded API calls; no native DSP or audible acceptance",
    }),
  );
  engine.suspend();
  assert.equal(engine.voices.size, 0);
  engine.dispose();
});
