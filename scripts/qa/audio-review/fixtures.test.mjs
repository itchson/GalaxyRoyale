/** Actual page declarations/state functions; failure injection never returns PCM. */
import assert from "node:assert/strict";
import { test, before } from "node:test";
import fs from "node:fs/promises";
import path from "node:path";
import vm from "node:vm";
import { pathToFileURL } from "node:url";
import { execFileSync } from "node:child_process";
import ts from "typescript";
import { prepare } from "./prepare.mjs";
import { PROJECT_ROOT, TOOL_DIRECTORY } from "./config.mjs";
import { CAPTURE_LIMIT } from "./server.mjs";

const PARENT = "e9e6069b4f3420d45b4ee6e5405cc7975145eec1";
let current, previous, AudioEngine, mixer, output;
function declarations(text, bindings) {
  const parsed = ts.createSourceFile("review.js", text, ts.ScriptTarget.ES2022);
  const names = new Set([
    "fixtures",
    "currentScorePhases",
    "fixtureState",
    "fixtureEvents",
    "createOfflineAdapter",
    "renderFixture",
  ]);
  const selected = parsed.statements.filter((node) =>
    ts.isVariableStatement(node)
      ? node.declarationList.declarations.some((d) =>
          names.has(d.name.getText(parsed)),
        )
      : ts.isFunctionDeclaration(node) && names.has(node.name?.text),
  );
  // Execute the actual page functions without its DOM/import side effects.
  const source = selected
    .map((node) => node.getText(parsed).replace(/^export\s+/, ""))
    .join("\n");
  return vm.runInNewContext(
    `${source}\n({fixtures,fixtureState,fixtureEvents,renderFixture,createOfflineAdapter:typeof createOfflineAdapter==='function'?createOfflineAdapter:null,phases:typeof currentScorePhases==='undefined'?null:currentScorePhases})`,
    { ...bindings },
  );
}
const plain = (value) => JSON.parse(JSON.stringify(value));
before(async () => {
  await fs.mkdir(path.join(PROJECT_ROOT, "work"), { recursive: true });
  output = await fs.mkdtemp(
    path.join(PROJECT_ROOT, "work", "audio-fixture-test-"),
  );
  await prepare(output);
  const module = (name) =>
    import(pathToFileURL(path.join(output, "modules", name)).href);
  ({ AudioEngine } = await module("audio.js"));
  const { AudioEngine: BaselineEngine } = await module("baseline-audio.js");
  mixer = await module("audio-state.js");
  const bindings = {
    AudioEngine,
    BaselineEngine,
    QUIET_STATE: mixer.QUIET_STATE,
  };
  current = declarations(
    await fs.readFile(path.join(TOOL_DIRECTORY, "review.js"), "utf8"),
    bindings,
  );
  previous = declarations(
    execFileSync(
      "git",
      ["show", `${PARENT}:scripts/qa/audio-review/review.js`],
      { cwd: PROJECT_ROOT },
    ).toString(),
    bindings,
  );
});

test("the four earlier declared algorithms are unchanged across every scheduler time", () => {
  assert.equal(current.fixtures.length, 7);
  assert.deepEqual(
    plain(current.fixtures.slice(0, 4)),
    plain(previous.fixtures),
  );
  for (let i = 0; i < 4; i++) {
    const fixture = current.fixtures[i];
    assert.equal(fixture.Engine, previous.fixtures[i].Engine);
    assert.deepEqual(
      plain(current.fixtureEvents(fixture)),
      plain(previous.fixtureEvents(previous.fixtures[i])),
    );
    for (let tick = 0; tick <= fixture.seconds * 40; tick++)
      assert.deepEqual(
        plain(current.fixtureState(fixture, tick / 40)),
        plain(previous.fixtureState(previous.fixtures[i], tick / 40)),
      );
  }
});

test("title draft and Continue/chase fixtures declare native synthesis, bounded captures and no source recording", () => {
  const [title, transition] = current.fixtures.slice(5);
  assert.equal(title.key, "title-score");
  assert.equal(title.titleOnly, true);
  assert.equal(title.seconds, 48);
  assert.equal(transition.key, "title-transition");
  assert.equal(transition.titleTransition, true);
  assert.equal(transition.seconds, 32);
  for (const fixture of [title, transition]) {
    assert.equal(fixture.Engine, AudioEngine);
    assert.equal(fixture.seconds * 48000 * 2 * 4 + 44 < CAPTURE_LIMIT, true);
    assert.deepEqual(plain(current.fixtureEvents(fixture)), []);
  }
  assert.equal(current.fixtureState(transition, 6).area, "haven");
  assert.equal(current.fixtureState(transition, 11).area, "reach");
  assert.equal(current.fixtureState(transition, 15).hostiles, 6);
  assert.equal(current.fixtureState(transition, 20).hostiles, 0);
  assert.equal(current.fixtureState(transition, 26).area, "haven");
});

test("current score declares three home identities, exploration, real boss inputs and healed thrust release", () => {
  const fixture = current.fixtures[4];
  assert.equal(fixture.key, "current-score");
  assert.equal(fixture.Engine, AudioEngine);
  assert.equal(fixture.seconds, 24);
  const phases = current.phases;
  assert.equal(phases.length, 7);
  assert.equal(phases[0].start, 0);
  assert.equal(phases.at(-1).end, fixture.seconds);
  assert.equal(new Set(phases.map((p) => p.key)).size, 7);
  for (const [index, phase] of phases.entries()) {
    assert(phase.start < phase.end);
    if (index) assert.equal(phase.start, phases[index - 1].end);
    const state = current.fixtureState(fixture, phase.start);
    assert.deepEqual(plain(mixer.cleanState(state)), plain(state));
    assert.deepEqual(
      plain(current.fixtureState(fixture, phase.end - 0.001)),
      plain(state),
    );
    if (index < 3) {
      assert.equal(state.corporation, ["pan", "renko", "hugo"][index]);
      assert.equal(state.area, "haven");
      let mix = mixer.newMix();
      for (let tick = 0; tick < 120; tick++)
        mix = mixer.advanceMix(mix, state, 0.025);
      assert.equal(mixer.audibleThemes(mix)[0][0], 15 + index);
      assert.equal(mix.boss, 0);
    }
  }
  const quiet = current.fixtureState(fixture, 9);
  assert.equal(quiet.biome, 0);
  assert.equal(quiet.combat, 0);
  assert(quiet.throttle > 0 && quiet.engines === 2);
  const combat = current.fixtureState(fixture, 12);
  assert(combat.combat > 0 && combat.hostiles > 0 && !combat.boss);
  const boss = current.fixtureState(fixture, 16);
  assert(boss.boss && boss.hostiles > 0 && boss.combat > 0 && boss.hull < 0.25);
  const release = current.fixtureState(fixture, 20);
  assert.equal(release.hull, 1);
  assert.equal(release.throttle, 0);
  assert.equal(release.speed, 0);
  assert.equal(release.combat, 0);
  assert.equal(release.hostiles, 0);
  assert.equal(release.boss, false);
  assert.equal(release.engines, 2);
  assert(fixture.seconds * 48000 * 2 * 4 + 44 < CAPTURE_LIMIT);
});

test("new declared cues are bounded, chronological and end before the native release tail", () => {
  const fixture = current.fixtures[4],
    events = current.fixtureEvents(fixture);
  assert.equal(events.length, 18);
  assert.equal(
    events.reduce((sum, e) => sum + e.count, 0),
    18,
  );
  for (const [i, event] of events.entries()) {
    assert(
      Number.isFinite(event.at) &&
        event.at > 0 &&
        event.at < fixture.seconds - 2,
    );
    assert.equal(event.count, 1);
    if (i) assert(events[i - 1].at < event.at);
  }
  assert(events.some((e) => e.kind === "boss" && e.at === 16));
  assert(!events.some((e) => e.kind === "death"));
});

test("missing native DSP rejects visibly before mutating adapter globals", async () => {
  const isolated = {
    AudioContext: class {},
    setInterval() {},
    clearInterval() {},
  };
  const saved = { ...isolated };
  const api = declarations(
    await fs.readFile(path.join(TOOL_DIRECTORY, "review.js"), "utf8"),
    {
      AudioEngine,
      BaselineEngine: AudioEngine,
      QUIET_STATE: mixer.QUIET_STATE,
      window: isolated,
    },
  );
  await assert.rejects(
    api.renderFixture(api.fixtures[4]),
    /Native OfflineAudioContext is unavailable/,
  );
  assert.deepEqual(isolated, saved);
});

test("pre-render injected unlock failure disposes callbacks and restores every adapter owner without samples", async () => {
  // Deliberate failure doubles, not a rendered graph or fake PCM evidence.
  let disposed = 0,
    rendered = 0;
  const isolated = {
    AudioContext: class OriginalContext {},
    setInterval() {
      throw new Error("original interval must not run");
    },
    clearInterval() {},
    OfflineAudioContext: class {
      startRendering() {
        rendered++;
        throw new Error("no PCM may be produced");
      }
    },
  };
  const saved = { ...isolated };
  class FailingEngine {
    setLevels() {}
    update() {}
    async unlock() {
      this.id = isolated.setInterval(() => {}, 25);
      throw new Error("declared unlock failure");
    }
    dispose() {
      isolated.clearInterval(this.id);
      disposed++;
    }
  }
  const api = declarations(
    await fs.readFile(path.join(TOOL_DIRECTORY, "review.js"), "utf8"),
    {
      AudioEngine,
      BaselineEngine: AudioEngine,
      QUIET_STATE: mixer.QUIET_STATE,
      window: isolated,
    },
  );
  await assert.rejects(
    api.renderFixture({ ...api.fixtures[4], Engine: FailingEngine }),
    /declared unlock failure/,
  );
  assert.equal(rendered, 0);
  assert.equal(disposed, 1);
  assert.deepEqual(isolated, saved);
});

test("actual renderer preserves absent native panner capability before an explicit no-PCM probe exit", async () => {
  let observed,
    disposed = 0,
    rendered = 0;
  const isolated = {
    AudioContext: class OriginalContext {},
    setInterval() {},
    clearInterval() {},
    OfflineAudioContext: class {
      startRendering() {
        rendered++;
        throw new Error("PCM is forbidden in the capability probe");
      }
    },
  };
  const saved = { ...isolated };
  class CapabilityProbeEngine {
    setLevels() {}
    update() {}
    async unlock() {
      const host = new isolated.AudioContext();
      observed = typeof host.createStereoPanner;
      assert.equal(
        observed,
        "undefined",
        "the production feature guard must enter its mono route",
      );
      throw new Error("declared no-PCM capability probe");
    }
    dispose() {
      disposed++;
    }
  }
  const api = declarations(
    await fs.readFile(path.join(TOOL_DIRECTORY, "review.js"), "utf8"),
    {
      AudioEngine,
      BaselineEngine: AudioEngine,
      QUIET_STATE: mixer.QUIET_STATE,
      window: isolated,
    },
  );
  await assert.rejects(
    api.renderFixture({ ...api.fixtures[4], Engine: CapabilityProbeEngine }),
    /declared no-PCM capability probe/,
  );
  assert.equal(observed, "undefined");
  assert.equal(rendered, 0);
  assert.equal(disposed, 1);
  assert.deepEqual(isolated, saved);
});

test("exported offline adapter preserves real receivers, return identity and exact node/source registration", () => {
  const nodes = [],
    sources = [],
    calls = [];
  const panner = { kind: "native-panner" },
    oscillator = { kind: "native-oscillator" };
  const bufferSource = { kind: "native-buffer-source" },
    buffer = { kind: "native-buffer" };
  const real = {
    get sampleRate() {
      assert.equal(this, real);
      return 48000;
    },
    createStereoPanner(...args) {
      assert.equal(this, real);
      calls.push(args);
      return panner;
    },
    createOscillator() {
      assert.equal(this, real);
      return oscillator;
    },
    createBufferSource() {
      assert.equal(this, real);
      return bufferSource;
    },
    createBuffer() {
      assert.equal(this, real);
      return buffer;
    },
  };
  const host = current.createOfflineAdapter(real, nodes, sources);
  const detachedFactory = host.createStereoPanner;
  assert.equal(typeof detachedFactory, "function");
  assert.equal(detachedFactory("declared argument"), panner);
  assert.deepEqual(calls, [["declared argument"]]);
  assert.equal(host.createOscillator(), oscillator);
  assert.equal(host.createBufferSource(), bufferSource);
  assert.equal(host.createBuffer(), buffer);
  assert.equal(host.sampleRate, 48000);
  assert.deepEqual(nodes, [panner, oscillator, bufferSource]);
  assert.deepEqual(sources, [oscillator, bufferSource]);
});

test("exported offline adapter retains absent and noncallable factory capabilities without phantom nodes", () => {
  for (const value of [undefined, null, 17]) {
    const real = value === undefined ? {} : { createStereoPanner: value };
    const nodes = [],
      sources = [];
    const host = current.createOfflineAdapter(real, nodes, sources);
    assert.equal(host.createStereoPanner, value);
    assert.equal(typeof host.createStereoPanner === "function", false);
    assert.deepEqual(nodes, []);
    assert.deepEqual(sources, []);
  }
});
