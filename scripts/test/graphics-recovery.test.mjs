import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

// Execute the production transition functions. The deferred adapter verifies
// shell ordering and user intent; real WASM/GPU behavior is checked separately.
const source = ts.createSourceFile(
  "main.ts",
  readFileSync(new URL("../../apps/game/src/main.ts", import.meta.url), "utf8"),
  ts.ScriptTarget.Latest,
  true,
);
const names = [
  "recover",
  "resize",
  "resizeGraphics",
  "resume",
  "frame",
  "restoreUi",
  "onWebglLost",
  "onWebglRestored",
];
const functions = names.map((name) => {
  const node = source.statements.find(
    (node) => ts.isFunctionDeclaration(node) && node.name?.text === name,
  );
  assert.ok(node, `Missing real ${name} transition`);
  return node.getText(source);
});
const compiled = ts.transpileModule(functions.join("\n"), {
  compilerOptions: { target: ts.ScriptTarget.ES2022 },
}).outputText;

function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
function harness() {
  const gate = deferred(),
    calls = [],
    notices = [],
    nodes = new Map();
  const element = (id) => {
    if (!nodes.has(id)) nodes.set(id, { hidden: true, textContent: "" });
    return nodes.get(id);
  };
  const classes = new Set(["pixel-ready"]);
  const profile = { revision: 1, currency: [2, 3, 4] };
  const context = {
    canvas: {},
    recovering: false,
    graphicsFailed: false,
    webglLifecycle: null,
    recoverAfterContextRestore: false,
    paused: false,
    launched: true,
    lastTime: 123,
    beforeLaunchMenu: false,
    menuTab: "settings",
    preparingEncounter: null,
    hud: { campaign: { credits_pending: false } },
    menu: { hidden: true },
    deathScreen: { hidden: true },
    launchScreen: { hidden: true },
    debugConsole: { opened: false },
    dialoguePanel: { opened: false },
    portrait: false,
    input: {
      enabled: true,
      reset() {
        calls.push("input-reset");
      },
    },
    audio: {
      suspend() {
        calls.push("audio-suspend");
      },
      async resume() {
        calls.push("audio-resume");
      },
    },
    galaxy: {
      set_paused(value) {
        calls.push(["paused", value]);
      },
      set_ui_mode(value) {
        calls.push(["ui", value]);
      },
      install_recovery(candidate) {
        calls.push(["install", candidate]);
      },
      resize(w, h) {
        calls.push(["resize", w, h]);
      },
      viewport() {
        return [533, 300];
      },
      world_viewport() {
        return [640, 360];
      },
      set_ui_insets(...values) {
        calls.push(["insets", ...values]);
      },
      profile() {
        return profile;
      },
    },
    prepareRecovery(canvas) {
      calls.push(["prepare", canvas]);
      return gate.promise;
    },
    innerWidth: 1280,
    innerHeight: 720,
    devicePixelRatio: 1,
    safeAreaProbe: {},
    getComputedStyle() {
      return {
        paddingTop: "0",
        paddingRight: "0",
        paddingBottom: "0",
        paddingLeft: "0",
      };
    },
    document: {
      hidden: false,
      body: {
        classList: {
          add(value) {
            classes.add(value);
          },
          remove(value) {
            classes.delete(value);
          },
          toggle(value, on) {
            if (on) classes.add(value);
            else classes.delete(value);
          },
        },
      },
      documentElement: { style: { setProperty() {} } },
    },
    matchMedia() {
      return { matches: context.portrait };
    },
    pixelUi: {
      invalidate() {
        calls.push("invalidate");
      },
    },
    pause() {
      context.paused = true;
      context.menu.hidden = false;
      calls.push("pause-menu");
    },
    toast(value) {
      notices.push(value);
    },
    requestAnimationFrame() {
      calls.push("raf");
    },
    element,
  };
  vm.runInNewContext(compiled, context);
  return { context, gate, calls, notices, classes, profile };
}

test("pending preparation leaves Game readable and defers GPU resize to latest dimensions", async () => {
  const h = harness(),
    c = h.context;
  const pending = c.recover("device lost");
  assert.equal(c.recovering, true);
  assert.equal(c.input.enabled, false);
  assert.equal(h.classes.has("pixel-ready"), false);
  assert.ok(h.calls.includes("audio-suspend"));
  c.innerWidth = 720;
  c.innerHeight = 360;
  c.devicePixelRatio = 2;
  c.resize();
  c.frame(1000);
  assert.equal(
    h.calls.filter((v) => Array.isArray(v) && v[0] === "resize").length,
    0,
  );
  assert.equal(
    h.calls.filter((v) => Array.isArray(v) && v[0] === "prepare").length,
    1,
  );
  const live = c.galaxy.profile();
  live.revision = 9;
  live.currency[0] = 12;
  const candidate = {};
  h.gate.resolve(candidate);
  assert.equal(await pending, true);
  assert.deepEqual(c.galaxy.profile(), { revision: 9, currency: [12, 3, 4] });
  const install = h.calls.findIndex(
    (v) => Array.isArray(v) && v[0] === "install",
  );
  const resize = h.calls.findIndex(
    (v) => Array.isArray(v) && v[0] === "resize",
  );
  assert.ok(install >= 0 && resize > install);
  assert.deepEqual(h.calls[resize], ["resize", 1440, 720]);
  assert.equal(c.recovering, false);
  assert.equal(c.paused, false);
  assert.equal(c.input.enabled, true);
  assert.equal(c.lastTime, 0);
});

test("overlapping recovery requests prepare and install exactly once", async () => {
  const h = harness(),
    c = h.context;
  const pending = c.recover("first");
  assert.equal(await c.recover("second"), false);
  h.gate.resolve({});
  assert.equal(await pending, true);
  assert.equal(
    h.calls.filter((v) => Array.isArray(v) && v[0] === "prepare").length,
    1,
  );
  assert.equal(
    h.calls.filter((v) => Array.isArray(v) && v[0] === "install").length,
    1,
  );
});

test("settlement honors latest menu, death, title, encounter, console, visibility and orientation intent", async () => {
  const mutations = [
    (c) => {
      c.paused = true;
    },
    (c) => {
      c.menu.hidden = false;
    },
    (c) => {
      c.deathScreen.hidden = false;
    },
    (c) => {
      c.launched = false;
    },
    (c) => {
      c.preparingEncounter = 4;
    },
    (c) => {
      c.debugConsole.opened = true;
    },
    (c) => {
      c.dialoguePanel.opened = true;
    },
    (c) => {
      c.document.hidden = true;
    },
    (c) => {
      c.portrait = true;
    },
  ];
  for (const mutate of mutations) {
    const h = harness(),
      c = h.context,
      pending = c.recover("loss");
    mutate(c);
    h.gate.resolve({});
    assert.equal(await pending, true);
    assert.equal(c.paused, true);
    assert.equal(c.input.enabled, false);
    assert.equal(h.calls.includes("audio-resume"), false);
  }
  const h = harness();
  h.context.paused = true;
  const pending = h.context.recover("paused loss");
  h.gate.resolve({});
  assert.equal(await pending, true);
  assert.equal(h.context.paused, true);
});

test("reconstruction rejection reports its cause, preserves Game and stops the RAF retry loop", async () => {
  const h = harness(),
    c = h.context;
  const pending = c.recover("original trigger");
  h.gate.reject(new Error("replacement adapter unavailable"));
  assert.equal(await pending, false);
  assert.equal(c.graphicsFailed, true);
  assert.equal(c.recovering, false);
  assert.equal(c.input.enabled, false);
  assert.equal(c.paused, true);
  assert.equal(h.classes.has("pixel-ready"), false);
  assert.ok(h.notices.at(-1).includes("replacement adapter unavailable"));
  assert.deepEqual(c.galaxy.profile(), h.profile);
  c.frame(1500);
  c.resize();
  assert.equal(
    h.calls.filter((v) => Array.isArray(v) && v[0] === "prepare").length,
    1,
  );
  assert.equal(
    h.calls.filter((v) => Array.isArray(v) && v[0] === "resize").length,
    0,
  );
});

test("a modal closed while preparation is pending settles to a visible pause menu", async () => {
  const h = harness(),
    c = h.context;
  const pending = c.recover("loss");
  // Console/dialogue onOpen pauses; onClose cannot resume during preparation.
  c.paused = true;
  c.debugConsole.opened = false;
  h.gate.resolve({});
  assert.equal(await pending, true);
  assert.equal(c.menu.hidden, false);
  assert.equal(c.input.enabled, false);
  assert.ok(h.calls.includes("pause-menu"));
});

test("failed reconstruction preserves an existing death report, title or other modal", async () => {
  for (const configure of [
    (c) => {
      c.deathScreen.hidden = false;
    },
    (c) => {
      c.launched = false;
      c.launchScreen.hidden = false;
    },
    (c) => {
      c.debugConsole.opened = true;
    },
    (c) => {
      c.dialoguePanel.opened = true;
    },
    (c) => {
      c.menu.hidden = false;
    },
  ]) {
    const h = harness(),
      c = h.context;
    configure(c);
    const previous = [
      c.deathScreen.hidden,
      c.launchScreen.hidden,
      c.menu.hidden,
    ];
    const pending = c.recover("loss");
    h.gate.reject(new Error("no replacement device"));
    assert.equal(await pending, false);
    assert.deepEqual(
      [c.deathScreen.hidden, c.launchScreen.hidden, c.menu.hidden],
      previous,
    );
    assert.equal(h.calls.includes("pause-menu"), false);
    assert.equal(c.graphicsFailed, true);
    assert.equal(c.input.enabled, false);
  }
});

test("an invalid install is a controlled failure without changing the live profile", async () => {
  const h = harness(),
    c = h.context;
  c.galaxy.install_recovery = () => {
    throw new Error("different recovery canvas");
  };
  const pending = c.recover("loss");
  h.gate.resolve({});
  assert.equal(await pending, false);
  assert.equal(c.graphicsFailed, true);
  assert.ok(h.notices.at(-1).includes("different recovery canvas"));
  assert.deepEqual(c.galaxy.profile(), h.profile);
});

test("Resume's awaited audio cannot re-enable input when recovery starts meanwhile", async () => {
  const h = harness(),
    c = h.context,
    audioGate = deferred();
  c.paused = true;
  c.menu.hidden = false;
  c.audio.resume = () => audioGate.promise;
  const resuming = c.resume();
  const recovery = c.recover("loss during audio activation");
  audioGate.resolve();
  await resuming;
  assert.equal(c.paused, true);
  assert.equal(c.input.enabled, false);
  h.gate.resolve({});
  assert.equal(await recovery, true);
  assert.equal(c.paused, true);
  assert.equal(c.input.enabled, false);
});

test("a real context loss gates drawing, resize and preparation until restored", async () => {
  const h = harness(),
    c = h.context;
  c.webglLifecycle = { lost: true };
  c.onWebglLost();
  c.frame(1000);
  c.resize();
  assert.equal(await c.recover("while lost"), false);
  assert.equal(c.paused, false); // Retain intent, while Rust and input are paused.
  assert.equal(c.input.enabled, false);
  assert.equal(h.classes.has("pixel-ready"), false);
  assert.equal(
    h.calls.some(
      (v) =>
        Array.isArray(v) && ["prepare", "resize", "install"].includes(v[0]),
    ),
    false,
  );
  c.webglLifecycle.lost = false;
  c.onWebglRestored();
  h.gate.resolve({});
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(c.recovering, false);
  assert.equal(c.input.enabled, true);
});

test("a context lost during preparation disposes the uninstalled candidate", async () => {
  const h = harness(),
    c = h.context;
  let freed = 0;
  const pending = c.recover("first");
  c.webglLifecycle = { lost: true };
  c.onWebglLost();
  h.gate.resolve({
    free() {
      freed++;
    },
  });
  assert.equal(await pending, false);
  assert.equal(freed, 1);
  assert.equal(c.graphicsFailed, false);
  assert.equal(
    h.calls.some((v) => Array.isArray(v) && v[0] === "install"),
    false,
  );
});

test("restoration during preparation discards stale resources and queues one fresh replacement", async () => {
  const h = harness(),
    c = h.context,
    fresh = deferred();
  let freed = 0;
  const pending = c.recover("first");
  c.webglLifecycle = { lost: true };
  c.onWebglLost();
  c.webglLifecycle.lost = false;
  c.onWebglRestored();
  c.prepareRecovery = () => {
    h.calls.push(["prepare", c.canvas]);
    return fresh.promise;
  };
  h.gate.resolve({
    free() {
      freed++;
    },
  });
  assert.equal(await pending, false);
  assert.equal(freed, 1);
  assert.equal(c.recovering, true);
  assert.equal(
    h.calls.filter((v) => Array.isArray(v) && v[0] === "prepare").length,
    2,
  );
  const candidate = {};
  fresh.resolve(candidate);
  await fresh.promise;
  await Promise.resolve();
  assert.equal(c.recovering, false);
  assert.equal(c.input.enabled, true);
  assert.equal(
    h.calls.filter((v) => Array.isArray(v) && v[0] === "install").length,
    1,
  );
  assert.equal(
    h.calls.find((v) => Array.isArray(v) && v[0] === "install")[1],
    candidate,
  );
});

test("closing the last modal after failed reconstruction exposes the pause menu", () => {
  const h = harness(),
    c = h.context;
  c.graphicsFailed = true;
  c.paused = true;
  c.input.enabled = false;
  c.restoreUi();
  assert.equal(c.menu.hidden, false);
  for (const preserve of [
    (c) => {
      c.deathScreen.hidden = false;
    },
    (c) => {
      c.launched = false;
    },
    (c) => {
      c.dialoguePanel.opened = true;
    },
    (c) => {
      c.debugConsole.opened = true;
    },
  ]) {
    const h = harness();
    h.context.graphicsFailed = true;
    preserve(h.context);
    h.context.restoreUi();
    assert.equal(h.calls.includes("pause-menu"), false);
  }
});
