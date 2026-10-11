import { AudioEngine } from "./modules/audio.js";
import { AudioEngine as BaselineEngine } from "./modules/baseline-audio.js";
import { QUIET_STATE } from "./modules/audio-state.js";
import { pcmMetrics, highFrequencyRatio, wav } from "./metrics.js";

const button = document.getElementById("run"),
  status = document.getElementById("status");
const reportNode = document.getElementById("report"),
  rows = document.getElementById("rows");
const downloads = document.getElementById("downloads");
let active = false;
const objectUrls = [];
const fixtures = [
  {
    key: "prior-engine",
    title: "Prior engine, default buses",
    Engine: BaselineEngine,
    seconds: 6,
    engine: true,
  },
  {
    key: "warm-engine",
    title: "Warm engine, default buses",
    Engine: AudioEngine,
    seconds: 6,
    engine: true,
  },
  {
    key: "representative-mix",
    title: "Eight-theme score, propulsion, low hull and cues",
    Engine: AudioEngine,
    seconds: 11,
  },
  {
    key: "max-bus-burst",
    title: "Maximum buses with declared capped cue bursts",
    Engine: AudioEngine,
    seconds: 5,
    burst: true,
  },
];
const sha = async (bytes) =>
  [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))]
    .map((n) => n.toString(16).padStart(2, "0"))
    .join("");
function fixtureState(fixture, time) {
  if (fixture.engine)
    return {
      ...QUIET_STATE,
      area: "reach",
      engines: 2,
      throttle: time < 4 ? 1 : 0,
      speed: time < 4 ? 1 : 0,
    };
  return {
    ...QUIET_STATE,
    area: "arena",
    champion: fixture.burst ? 3 : Math.min(7, Math.floor(time / 1.25)),
    combat: 0.8,
    hostiles: 12,
    hull: 0.08,
    throttle: 0.9,
    speed: 0.7,
    engines: 2,
  };
}
function fixtureEvents(fixture) {
  if (fixture.engine) return [];
  if (fixture.burst)
    return [
      { at: 1, kind: "explosion", count: 16 },
      { at: 2, kind: "death", count: 16 },
      { at: 3, kind: "upgrade", count: 16 },
    ];
  const kinds = [
    "shot",
    "shield",
    "player-hit",
    "mote-charge",
    "mote-pulse",
    "ability-echo",
    "ability-nova",
    "ability-signature-7",
  ];
  return Array.from({ length: 34 }, (_, i) => ({
    at: 0.25 + i * 0.3,
    kind: kinds[i % kinds.length],
    count: 1,
  }));
}
async function renderFixture(fixture) {
  const Constructor =
    window.OfflineAudioContext ?? window.webkitOfflineAudioContext;
  if (!Constructor)
    throw new Error(
      "Native OfflineAudioContext is unavailable; no fallback samples are produced.",
    );
  const real = new Constructor(2, Math.ceil(48000 * fixture.seconds), 48000);
  const intervals = new Map();
  let nextInterval = 1;
  const savedContext = window.AudioContext,
    savedInterval = window.setInterval,
    savedClear = window.clearInterval;
  const nodes = [],
    sources = [];
  const methods = new Set([
    "createGain",
    "createBiquadFilter",
    "createDelay",
    "createDynamicsCompressor",
    "createWaveShaper",
    "createBuffer",
    "createOscillator",
    "createBufferSource",
  ]);
  // This adapter is scoped exclusively to this isolated page/render. All graph
  // work and PCM rendering is native, not a recording or replacement DSP.
  const host = new Proxy(real, {
    get(target, name) {
      if (name === "state") return "running";
      if (name === "resume" || name === "suspend" || name === "close")
        return async () => {};
      if (name === "onstatechange") return null;
      if (methods.has(name))
        return (...args) => {
          const value = target[name](...args);
          if (name !== "createBuffer") nodes.push(value);
          if (name === "createOscillator" || name === "createBufferSource")
            sources.push(value);
          return value;
        };
      return Reflect.get(target, name, target);
    },
    set(_target, name) {
      if (name === "onstatechange") return true;
      throw new Error(`Unexpected offline adapter write ${String(name)}`);
    },
  });
  window.AudioContext = class {
    constructor() {
      return host;
    }
  };
  window.setInterval = (fn, ms) => {
    if (ms !== 25) throw new Error(`Unexpected scheduler interval ${ms}`);
    const id = nextInterval++;
    intervals.set(id, fn);
    return id;
  };
  window.clearInterval = (id) => intervals.delete(id);
  let engine;
  const events = fixtureEvents(fixture);
  let nextEvent = 0,
    cueCalls = 0,
    maxMusic = 0,
    maxEffects = 0,
    maxContinuous = 0,
    checkpoints = 0;
  const dispatched = [];
  const sourceStates = [];
  try {
    engine = new fixture.Engine();
    engine.setLevels(
      fixture.burst ? 1 : 0.75,
      fixture.engine ? 0 : fixture.burst ? 1 : 0.4,
      fixture.burst ? 1 : 0.75,
    );
    engine.update(fixtureState(fixture, 0));
    await engine.unlock();
    const times = [];
    for (let time = 0.025; time < fixture.seconds - 0.005; time += 0.025)
      times.push(real.suspend(time));
    const rendering = real.startRendering();
    for (const checkpoint of times) {
      await checkpoint;
      if (real.state !== "suspended")
        throw new Error("Scheduler did not reach a real native suspension.");
      const time = real.currentTime;
      engine.update(fixtureState(fixture, time));
      while (nextEvent < events.length && events[nextEvent].at <= time) {
        const event = events[nextEvent++];
        for (let i = 0; i < event.count; i++)
          engine.play({ kind: event.kind, x: 0, y: 0, intensity: 1 });
        cueCalls += event.count;
        dispatched.push({
          kind: event.kind,
          calls: event.count,
          requested_time: event.at,
          actual_native_time: time,
        });
      }
      for (const callback of [...intervals.values()]) callback();
      const d = engine.diagnostics();
      if (d.voices.music > 32 || d.voices.sfx > 16)
        throw new Error("Actual voice limit exceeded.");
      maxMusic = Math.max(maxMusic, d.voices.music);
      maxEffects = Math.max(maxEffects, d.voices.sfx);
      maxContinuous = Math.max(maxContinuous, d.voices.continuous);
      if (checkpoints % 40 === 0)
        sourceStates.push({
          time,
          champion: d.champion,
          hull: d.hull,
          engine: d.engine,
          engineDuck: d.engineDuck ?? 1,
        });
      checkpoints++;
      await real.resume();
    }
    const buffer = await rendering;
    const encoded = wav(buffer);
    const summary = {
      key: fixture.key,
      title: fixture.title,
      seconds: fixture.seconds,
      channels: buffer.numberOfChannels,
      sample_rate: buffer.sampleRate,
      native_state_after_render: real.state,
      native_suspend_checkpoints: checkpoints,
      graph_nodes_created: nodes.length,
      sources_created: sources.length,
      maximum_voices: {
        music: maxMusic,
        effects: maxEffects,
        continuous: maxContinuous,
      },
      events_dispatched: nextEvent,
      events_declared: events.length,
      cue_calls: cueCalls,
      dispatched,
      full: pcmMetrics(buffer),
      steady: pcmMetrics(buffer, 1, Math.min(3, fixture.seconds)),
      tail: pcmMetrics(buffer, fixture.seconds - 0.5, fixture.seconds),
      steady_power_above_1khz_ratio: fixture.engine
        ? highFrequencyRatio(buffer)
        : null,
      wav_bytes: encoded.byteLength,
      wav_sha256: await sha(encoded),
      sourceStates,
    };
    engine.dispose();
    summary.cleanup = {
      ...engine.diagnostics().voices,
      scheduler_callbacks: intervals.size,
    };
    if (
      summary.full.nonfinite ||
      Object.values(summary.cleanup).some((n) => n !== 0)
    )
      throw new Error("Nonfinite output or incomplete cleanup.");
    return { summary, encoded };
  } finally {
    engine?.dispose();
    window.AudioContext = savedContext;
    window.setInterval = savedInterval;
    window.clearInterval = savedClear;
  }
}
function download(label, bytes, name, type) {
  const a = document.createElement("a");
  a.textContent = label;
  a.download = name;
  a.href = URL.createObjectURL(new Blob([bytes], { type }));
  objectUrls.push(a.href);
  const p = document.createElement("p");
  p.append(a);
  downloads.append(p);
}
button.addEventListener("click", async () => {
  if (active) return;
  active = true;
  button.disabled = true;
  rows.replaceChildren();
  downloads.replaceChildren();
  for (const url of objectUrls.splice(0)) URL.revokeObjectURL(url);
  const run = `qa-${Date.now()}-${crypto.randomUUID().slice(0, 8)}`;
  const report = {
    run,
    scope:
      "native OfflineAudioContext PCM, declared fixtures; no human/mobile/gameplay acceptance",
    user_agent: navigator.userAgent,
    created_utc: new Date().toISOString(),
    cases: [],
  };
  try {
    report.sources = await (
      await fetch("./source-pins.json", { cache: "no-store" })
    ).json();
    for (const fixture of fixtures) {
      status.textContent = `Rendering ${fixture.title} through native DSP…`;
      const { summary, encoded } = await renderFixture(fixture);
      report.cases.push(summary);
      const row = document.createElement("tr");
      for (const value of [
        summary.title,
        summary.full.peak.toFixed(6),
        summary.full.rms.toFixed(6),
        `${summary.full.over_unity} / ${summary.full.nonfinite}`,
        `${summary.maximum_voices.music} / ${summary.maximum_voices.effects}`,
      ]) {
        const cell = document.createElement("td");
        cell.textContent = value;
        row.append(cell);
      }
      rows.append(row);
      download(
        `Download ${fixture.key} float WAV`,
        encoded,
        `${run}-${fixture.key}.wav`,
        "audio/wav",
      );
      const capture = await fetch(`./capture/${run}/${fixture.key}.wav`, {
        method: "POST",
        body: encoded,
      });
      if (!capture.ok)
        throw new Error(`Local WAV preservation failed: ${capture.status}`);
    }
    const baseline = report.cases[0],
      warm = report.cases[1];
    report.engine_comparison = {
      steady_rms_change_db:
        20 * Math.log10(warm.steady.rms / baseline.steady.rms),
      high_frequency_power_ratio_before: baseline.steady_power_above_1khz_ratio,
      high_frequency_power_ratio_after: warm.steady_power_above_1khz_ratio,
    };
    report.status = report.cases.some((c) => c.full.over_unity > 0)
      ? "Completed; over-unity samples require review"
      : "Completed; no over-unity/nonfinite samples in these declared fixtures";
    reportNode.textContent = JSON.stringify(report, null, 2);
    const bytes = JSON.stringify(report, null, 2) + "\n";
    const capture = await fetch(`./capture/${run}/report.json`, {
      method: "POST",
      body: bytes,
      headers: { "Content-Type": "application/json" },
    });
    if (!capture.ok)
      throw new Error(`Local report preservation failed: ${capture.status}`);
    download(
      "Download DSP report",
      bytes,
      `${run}-report.json`,
      "application/json",
    );
    status.textContent = `${report.status}. Capture ${run} preserved locally. Listening remains pending.`;
    document.body.dataset.run = run;
    document.body.dataset.status = "complete";
  } catch (error) {
    status.textContent = `FAILED: ${error instanceof Error ? error.message : String(error)}`;
    reportNode.textContent = JSON.stringify(report, null, 2);
    document.body.dataset.status = "failed";
  } finally {
    active = false;
    button.disabled = false;
  }
});
