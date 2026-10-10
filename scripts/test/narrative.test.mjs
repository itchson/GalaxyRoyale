import assert from "node:assert/strict";
import test from "node:test";
import {
  NarrativeDirector,
  BrowserNarrativePlayback,
  VoiceUnavailable,
  validateFlavor,
  validateNarrativeAudio,
} from "../../apps/game/src/narrative.ts";

const context = (id = "-1:0") => ({
  id,
  speaker: "Frontier contact",
  description: "Defeat the accepted family and return to this source.",
  personality: 3,
  status: "active",
  progress: { current: 2, total: 10 },
  facts: ["The listed reward is three offensive points and one relic."],
});
const settle = () => new Promise((resolve) => setImmediate(resolve));
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const wav = () => {
  const bytes = new Uint8Array(44);
  bytes.set(new TextEncoder().encode("RIFF"));
  bytes.set(new TextEncoder().encode("WAVE"), 8);
  return { bytes, mimeType: "audio/wav" };
};
function playback() {
  const calls = [];
  const pending = [];
  return {
    calls,
    pending,
    pauses: 0,
    resumes: 0,
    cancels: 0,
    volumes: [],
    play(request, signal) {
      calls.push(request);
      request.onStarted();
      return new Promise((resolve, reject) => {
        pending.push({ resolve, reject, signal });
        signal.addEventListener("abort", () => reject(new Error("Cancelled")), {
          once: true,
        });
      });
    },
    pause() {
      this.pauses++;
    },
    resume() {
      this.resumes++;
    },
    cancel() {
      this.cancels++;
    },
    setVolume(value) {
      this.volumes.push(value);
    },
  };
}

test("authored pages are bounded, grounded copies and require no service", () => {
  const input = context();
  input.description = "A long briefing with relevant facts. ".repeat(30);
  const director = new NarrativeDirector();
  const pages = director.open(input);
  assert.ok(
    pages.every((p) => p.text.length <= 256 && p.source === "authored"),
  );
  assert.equal(
    pages
      .filter((p) => p.role === "briefing")
      .map((p) => p.text)
      .join(" "),
    input.description.trim(),
  );
  assert.match(pages.find((p) => p.role === "state").text, /2 \/ 10/);
  input.facts[0] = "External mutation";
  pages[0].text = "External mutation";
  assert.notEqual(director.current().text, "External mutation");
  director.next();
  director.previous();
  assert.equal(director.current().role, "greeting");
  assert.equal(director.voiceStatus, "disabled");
  director.close();
  assert.equal(director.current(), null);
});

test("invalid facts, progress, characters and personalities are rejected", () => {
  for (const patch of [
    { personality: 8 },
    { status: "unknown" },
    { status: "" },
    { progress: { current: 11, total: 10 } },
    { facts: Array(2) },
    { description: "<script>bad</script>" },
    { speaker: "Invisible\u202e" },
  ])
    assert.throws(() =>
      new NarrativeDirector().open({ ...context(), ...patch }),
    );
  assert.equal(
    validateFlavor({ flavor: "A quiet channel. Steady flight, pilot." }),
    "A quiet channel. Steady flight, pilot.",
  );
  for (const flavor of [
    "You gained 20 credits.",
    "Claim your reward.",
    "<b>hello</b>",
    "https://example.com",
    "x".repeat(257),
    "\u202ehello",
  ])
    assert.equal(validateFlavor({ flavor }), null);
  assert.equal(validateFlavor("hello"), null);
});

test("valid optional flavor only updates greeting and preserves canonical state", async () => {
  const updates = [];
  let finish;
  const director = new NarrativeDirector({
    onUpdate: (pages) => updates.push(pages),
    bridge: { generate: () => new Promise((resolve) => (finish = resolve)) },
  });
  const original = director.open(context());
  await settle();
  finish({ flavor: "A quiet channel. Steady flight, pilot." });
  await settle();
  assert.equal(updates.length, 1);
  assert.equal(updates[0][0].source, "local");
  assert.deepEqual(updates[0].slice(1), original.slice(1));
  director.close();
});

test("a replaced greeting cancels replay of its old displayed text", async () => {
  const voice = playback();
  let finish;
  const director = new NarrativeDirector({
    voiceEnabled: true,
    playback: voice,
    bridge: { generate: () => new Promise((resolve) => (finish = resolve)) },
  });
  director.open(context());
  await settle();
  const speech = director.replay();
  finish({ flavor: "A quiet channel. Steady flight, pilot." });
  await settle();
  await speech;
  assert.equal(voice.pending[0].signal.aborted, true);
  assert.equal(director.current().source, "local");
  assert.equal(director.voiceStatus, "idle");
  director.close();
});

test("close/reopen aborts stale generation even when provider ignores cancellation", async () => {
  const pending = [];
  const updates = [];
  const director = new NarrativeDirector({
    onUpdate: (pages) => updates.push(pages),
    bridge: {
      generate: (request, signal) =>
        new Promise((resolve) => pending.push({ request, signal, resolve })),
    },
  });
  director.open(context("old"));
  await settle();
  director.open(context("new"));
  await settle();
  assert.equal(pending[0].signal.aborted, true);
  pending[0].resolve({ flavor: "Old channel speaking." });
  await settle();
  assert.equal(updates.length, 0);
  pending[1].resolve({ flavor: "New channel speaking." });
  await settle();
  assert.equal(updates[0][0].id.startsWith("new:"), true);
  director.close();
  director.open(context("cancel-before-dispatch"));
  director.close();
  await settle();
  assert.equal(pending.length, 2);
});

test("generation timeout and malformed response retain immediate authored content", async () => {
  let signal;
  const updates = [];
  const director = new NarrativeDirector({
    timeoutMs: 15,
    onUpdate: (pages) => updates.push(pages),
    bridge: {
      generate: (_, s) => {
        signal = s;
        return new Promise(() => {});
      },
    },
  });
  const pages = director.open(context());
  await delay(30);
  assert.equal(signal.aborted, true);
  assert.equal(updates.length, 0);
  assert.equal(director.current().text, pages[0].text);
  director.close();
  const invalid = new NarrativeDirector({
    bridge: {
      generate: async () => ({
        flavor: "You defeated a champion and received credits.",
      }),
    },
  });
  invalid.open(context());
  await settle();
  assert.equal(invalid.current().source, "authored");
  invalid.close();
});

test("voice is opt-in, explicit and supports per-page replay/pause/resume/volume", async () => {
  const voice = playback();
  const statuses = [];
  const director = new NarrativeDirector({
    playback: voice,
    onVoiceStatus: (status) => statuses.push(status),
  });
  const pages = director.open(context());
  await director.replay();
  assert.equal(voice.calls.length, 0);
  director.setVoiceEnabled(true);
  const speaking = director.replay(pages[1]);
  assert.equal(voice.calls[0].text, pages[1].text);
  assert.equal(voice.calls[0].volume, 0.7);
  assert.equal(director.voiceStatus, "speaking");
  director.pause();
  assert.equal(voice.pauses, 1);
  assert.equal(director.voiceStatus, "paused");
  director.resume();
  assert.equal(voice.resumes, 1);
  director.setVoiceVolume(0.25);
  assert.equal(voice.volumes.at(-1), 0.25);
  voice.pending[0].resolve();
  await speaking;
  assert.equal(director.voiceStatus, "idle");
  await director.replay({ ...pages[1], text: "Injected text" });
  assert.equal(voice.calls.length, 1);
  const replay = director.replay(pages[1]);
  director.close();
  await replay;
  assert.equal(voice.pending[1].signal.aborted, true);
  assert.equal(director.voiceStatus, "idle");
  assert.ok(statuses.includes("paused"));
});

test("stale synthesis never plays; timeout and invalid audio use optional platform fallback", async () => {
  const voice = playback();
  let finish;
  const director = new NarrativeDirector({
    voiceEnabled: true,
    playback: voice,
    bridge: { synthesize: () => new Promise((resolve) => (finish = resolve)) },
  });
  director.open(context());
  const first = director.replay();
  await settle();
  director.close();
  finish(wav());
  await first;
  assert.equal(voice.calls.length, 0);
  const fallback = new NarrativeDirector({
    voiceEnabled: true,
    playback: voice,
    timeoutMs: 15,
    bridge: { synthesize: () => new Promise(() => {}) },
  });
  fallback.open(context());
  const attempt = fallback.replay();
  await delay(30);
  assert.equal(voice.calls.length, 1);
  assert.equal(voice.calls[0].audio, undefined);
  voice.pending[0].resolve();
  await attempt;
  fallback.close();
  const noVoice = new NarrativeDirector({ voiceEnabled: true });
  noVoice.open(context());
  await noVoice.replay();
  assert.equal(noVoice.voiceStatus, "unavailable");
  noVoice.close();
});

test("validated audio is copied, bounded and has a matching supported header", () => {
  const audio = wav();
  const checked = validateNarrativeAudio(audio);
  assert.ok(checked);
  audio.bytes[0] = 0;
  assert.equal(checked.bytes[0], 82);
  assert.equal(
    validateNarrativeAudio({ ...wav(), mimeType: "text/html" }),
    null,
  );
  assert.equal(
    validateNarrativeAudio({
      bytes: new Uint8Array(20),
      mimeType: "audio/wav",
    }),
    null,
  );
  assert.equal(
    validateNarrativeAudio({
      bytes: new Uint8Array(8 * 1024 * 1024 + 1),
      mimeType: "audio/wav",
    }),
    null,
  );
});

test("invalid synthesis falls back without changing the displayed text request", async () => {
  const voice = playback();
  const director = new NarrativeDirector({
    voiceEnabled: true,
    playback: voice,
    bridge: {
      synthesize: async (request) => {
        request.text = "Provider attempted to replace the displayed line.";
        return { bytes: new Uint8Array(20), mimeType: "audio/wav" };
      },
    },
  });
  const pages = director.open(context());
  const replay = director.replay();
  await settle();
  assert.equal(voice.calls[0].audio, undefined);
  assert.equal(voice.calls[0].text, pages[0].text);
  voice.pending[0].resolve();
  await replay;
  director.close();
});

test("browser adapter cancels platform speech and releases local audio URLs", async () => {
  const previous = {
    Audio: globalThis.Audio,
    URL: globalThis.URL,
    speechSynthesis: globalThis.speechSynthesis,
    SpeechSynthesisUtterance: globalThis.SpeechSynthesisUtterance,
  };
  const media = [];
  const revoked = [];
  let utterance;
  let cancelled = 0;
  class FakeAudio {
    constructor() {
      media.push(this);
    }
    play() {
      this.onplaying?.();
      return Promise.resolve();
    }
    pause() {}
    removeAttribute() {}
    load() {}
  }
  class FakeUtterance {
    constructor(text) {
      this.text = text;
    }
  }
  globalThis.Audio = FakeAudio;
  globalThis.URL = {
    createObjectURL: () => "blob:owned",
    revokeObjectURL: (value) => revoked.push(value),
  };
  globalThis.SpeechSynthesisUtterance = FakeUtterance;
  globalThis.speechSynthesis = {
    getVoices: () => [],
    speak: (value) => {
      utterance = value;
      value.onstart();
    },
    cancel: () => cancelled++,
    pause() {},
    resume() {},
  };
  try {
    const adapter = new BrowserNarrativePlayback();
    const request = {
      characterId: "npc",
      speaker: "Contact",
      personality: 0,
      text: "Safe line",
      style: "calm",
      volume: 0.4,
      onStarted() {},
    };
    const local = adapter.play(
      { ...request, audio: wav() },
      new AbortController().signal,
    );
    media[0].onended();
    await local;
    assert.deepEqual(revoked, ["blob:owned"]);
    const controller = new AbortController();
    const spoken = adapter.play(request, controller.signal);
    assert.equal(utterance.text, "Safe line");
    controller.abort();
    await assert.rejects(spoken, /cancelled/);
    assert.equal(cancelled, 1);
  } finally {
    for (const [key, value] of Object.entries(previous)) {
      if (value === undefined) delete globalThis[key];
      else globalThis[key] = value;
    }
  }
});
