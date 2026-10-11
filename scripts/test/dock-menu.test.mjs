import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

// Execute actual shell transitions and registered handlers with a fake DOM and
// deferred save adapter. This is not native Game, IndexedDB or browser layout QA.
const html = readFileSync(
  new URL("../../apps/game/index.html", import.meta.url),
  "utf8",
);
const source = ts.createSourceFile(
  "main.ts",
  readFileSync(new URL("../../apps/game/src/main.ts", import.meta.url), "utf8"),
  ts.ScriptTarget.Latest,
  true,
);
const functions = [
  "paintSaveStatus",
  "persist",
  "save",
  "pause",
  "canSimulateFlight",
  "syncFlightState",
  "openMenu",
  "interactDock",
  "selectMenuTab",
  "openLaunchSettings",
  "resume",
  "navigate",
  "beginEncounter",
  "quitGame",
].map((name) => {
  const node = source.statements.find(
    (node) => ts.isFunctionDeclaration(node) && node.name?.text === name,
  );
  assert.ok(node, `Missing production ${name}`);
  return node.getText(source);
});
const constants = ["facilityPages", "facilityPositions"].map((name) => {
  const statement = source.statements.find(
    (node) =>
      ts.isVariableStatement(node) &&
      node.declarationList.declarations.some(
        (declaration) => declaration.name.getText(source) === name,
      ),
  );
  assert.ok(statement, `Missing production ${name}`);
  return statement.getText(source);
});
function registration(id) {
  const statement = source.statements.find((node) => {
    if (
      !ts.isExpressionStatement(node) ||
      !ts.isCallExpression(node.expression)
    )
      return false;
    const receiver = node.expression.expression;
    if (
      !ts.isPropertyAccessExpression(receiver) ||
      receiver.name.text !== "addEventListener"
    )
      return false;
    const target = receiver.expression;
    return (
      ts.isCallExpression(target) &&
      target.expression.getText(source) === "element" &&
      ts.isStringLiteral(target.arguments[0]) &&
      target.arguments[0].text === id
    );
  });
  assert.ok(statement, `Missing real ${id} handler`);
  return statement.getText(source);
}
const keydown = source.statements.find(
  (node) =>
    ts.isExpressionStatement(node) &&
    ts.isCallExpression(node.expression) &&
    node.expression.expression.getText(source) === "window.addEventListener" &&
    node.expression.arguments[0]?.getText(source) === '"keydown"',
);
assert.ok(keydown, "Missing real keydown handler");
const facilities = source.statements.find(
  (node) =>
    ts.isForOfStatement(node) &&
    node.expression.getText(source) === "facilityPages.entries()",
);
assert.ok(facilities, "Missing real facility handlers");
const launchHandler = source.statements.find(
  (node) =>
    ts.isExpressionStatement(node) &&
    ts.isCallExpression(node.expression) &&
    node.expression.expression.getText(source) === "launch.addEventListener",
);
assert.ok(launchHandler, "Missing real launch handler");
const compiled = ts.transpileModule(
  [
    ...constants,
    ...functions,
    ...[
      "resume",
      "menu-close",
      "pause-settings",
      "settings-backup",
      "save-indicator",
      "restart-flight",
      "quit-game",
      "map-button",
      "flight-settings",
    ].map(registration),
    facilities.getText(source),
    keydown.getText(source),
    launchHandler.getText(source),
  ].join("\n"),
  { compilerOptions: { target: ts.ScriptTarget.ES2022 } },
).outputText;

const pageIds = [
  "pause-content",
  "meishi-content",
  "cargo-content",
  "upgrade-content",
  "hangar-content",
  "campaign-content",
  "quest-content",
  "map-content",
  "settings-content",
  "archive-content",
];
function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
async function settle() {
  for (let turn = 0; turn < 8; turn++) await Promise.resolve();
}
function fixture() {
  const calls = [],
    notices = [],
    nodes = new Map(),
    windowEvents = new Map();
  const document = { hidden: false, activeElement: null };
  class Element {
    constructor(tag, id = "", attrs = "") {
      this.tag = tag;
      this.id = id;
      this.hidden = /(?:^|\s)hidden(?:\s|$)/.test(attrs);
      this.disabled = false;
      this.dataset = {};
      this.textContent = "";
      this.attributes = new Map();
      this.events = new Map();
      this.children = [];
      const classes = new Set(
        attrs.match(/\bclass="([^"]*)"/)?.[1].split(/\s+/) ?? [],
      );
      this.classList = {
        add: (value) => classes.add(value),
        remove: (value) => classes.delete(value),
        contains: (value) => classes.has(value),
        toggle(value, on) {
          if (on) classes.add(value);
          else classes.delete(value);
        },
      };
    }
    setAttribute(name, value) {
      this.attributes.set(name, value);
    }
    append(...children) {
      this.children.push(...children);
    }
    replaceChildren(...children) {
      this.children = children;
    }
    addEventListener(name, callback) {
      this.events.set(name, callback);
    }
    click() {
      if (!this.disabled) return this.events.get("click")?.();
    }
    focus() {
      document.activeElement = this;
    }
  }
  for (const match of html.matchAll(
    /<([a-z][\w-]*)\b([^>]*\bid="([^"]+)"[^>]*)>/g,
  )) {
    assert.equal(
      nodes.has(match[3]),
      false,
      `Duplicate actual HTML id ${match[3]}`,
    );
    nodes.set(match[3], new Element(match[1], match[3], match[2]));
  }
  const element = (id) => {
    assert.ok(nodes.has(id), `Missing actual HTML id ${id}`);
    return nodes.get(id);
  };
  document.createElement = (tag) => new Element(tag);
  document.body = { classList: new Element("body").classList };
  const profile = { revision: 9, currency: [3, 4, 5] };
  const hud = {
    audio: {
      area: "haven",
      biome: 0,
      combat: 0,
      hostiles: 0,
      champion: null,
      boss: false,
      hull: 1,
      speed: 0,
      throttle: 0,
      engines: 2,
    },
    dock: true,
    currency: profile.currency,
    stored_materials: 0,
    discoveries: 0,
    relics: 0,
    quest_log: [],
    boss_progress: [],
    campaign: { credits_pending: false },
  };
  const context = {
    menu: element("menu-screen"),
    deathScreen: element("death-screen"),
    launchScreen: element("launch-screen"),
    launched: true,
    needsDroidCreation: false,
    paused: false,
    beforeLaunchMenu: false,
    menuTab: "upgrades",
    lastMapPaint: 0,
    performance: { now: () => 100 },
    transitionEpoch: 0,
    preparingEncounter: null,
    recovering: false,
    graphicsFailed: false,
    webglLifecycle: null,
    waitingUpdate: null,
    lastTime: 50,
    destination: [50, 30],
    previousDock: true,
    pendingSaves: 0,
    saveFailed: false,
    hud,
    element,
    document,
    debugConsole: { opened: false },
    dialoguePanel: {
      opened: false,
      close() {
        calls.push("dialogue-close");
      },
    },
    pixelUi: { invalidate() {} },
    input: {
      enabled: true,
      setCourse(position) {
        calls.push(["set-course", ...position]);
      },
      reset() {
        calls.push("input-reset");
      },
    },
    audio: {
      update(state) {
        calls.push(["audio-update", state]);
      },
      async unlock() {},
      suspend() {
        calls.push("audio-suspend");
      },
      async resume() {
        calls.push("audio-resume");
      },
    },
    galaxy: {
      cancel_course() {
        calls.push("cancel-course");
      },
      hud() {
        return hud;
      },
      universe_map() {
        calls.push("query-discovered-map");
        return { position: [0, 0], heading: 0, sites: [] };
      },
      profile() {
        return profile;
      },
      set_paused(value) {
        calls.push(["paused", value]);
      },
      set_ui_mode(value) {
        calls.push(["ui", value]);
      },
      dock_facility_at(x, y) {
        calls.push(["facility-hit", x, y]);
        return context.hit;
      },
      return_to_home() {
        calls.push("return-home");
      },
    },
    hit: -1,
    store: {
      persistent: true,
      message: "Progress stays on this device.",
      async save(value) {
        calls.push(["save", value.revision]);
      },
      async flush() {
        calls.push("flush");
      },
    },
    paintUpgrades(dock) {
      calls.push(["paint-upgrades", dock]);
    },
    paintMeishiCard() {
      calls.push("paint-meishi");
      return true;
    },
    paintCargo() {
      return true;
    },
    paintFlightMenu() {
      calls.push("paint-flight-menu");
      return true;
    },
    paintUniverseMap(_target, data) {
      calls.push(["paint-map", data.sites.length]);
      return true;
    },
    resetForge() {},
    paintShipHangar() {
      calls.push("paint-hangar");
    },
    paintCampaign() {
      calls.push("paint-campaign");
    },
    paintCredits() {},
    paintHud() {
      calls.push("paint-hud");
    },
    campaignActions: {},
    finishCredits() {},
    interactSite() {},
    toast(value) {
      notices.push(value);
    },
    SaveConflict: class SaveConflict extends Error {},
    matchMedia() {
      return { matches: false };
    },
    HTMLButtonElement: Element,
    HTMLElement: Element,
    window: {
      close() {
        calls.push("window-close");
      },
      addEventListener(name, callback) {
        windowEvents.set(name, callback);
      },
    },
  };
  context.loadStatus = element("load-status");
  context.launch = element("launch");
  context.narrative = {
    close() {
      calls.push("narrative-close");
    },
  };
  context.menu.hidden =
    context.deathScreen.hidden =
    context.launchScreen.hidden =
      true;
  vm.runInNewContext(compiled, context);
  return {
    context,
    calls,
    notices,
    profile,
    element,
    visiblePages: () => pageIds.filter((id) => !element(id).hidden),
    press(key, properties = {}) {
      windowEvents.get("keydown")({ key, preventDefault() {}, ...properties });
    },
  };
}

test("Escape opens a live MEISHI menu, cancels controls and course, and toggles closed", async () => {
  const f = fixture(),
    c = f.context;
  f.press("Escape");
  assert.equal(c.paused, false);
  assert.equal(c.input.enabled, false);
  assert.equal(c.menu.hidden, false);
  assert.deepEqual(f.visiblePages(), ["meishi-content"]);
  assert.ok(f.calls.includes("cancel-course"));
  assert.ok(f.calls.includes("paint-meishi"));
  assert.equal(
    f.calls.some((call) => Array.isArray(call) && call[0] === "paint-upgrades"),
    false,
  );
  assert.equal(f.calls.includes("audio-suspend"), false);
  assert.equal(
    f.calls.some((call) => Array.isArray(call) && call[0] === "paused"),
    false,
  );
  assert.equal(f.element("menu-description").textContent, "FLIGHT LIVE");
  assert.doesNotMatch(
    html,
    /menu-tabs|role="tab(?:list|panel)?"|aria-labelledby="tab-/,
  );
  f.press("Escape");
  await settle();
  assert.equal(c.menu.hidden, true);
  assert.equal(c.input.enabled, true);
  assert.equal(c.paused, false);
  await settle();
});

test("opening a dedicated page paints it once without building hidden collections", () => {
  const f = fixture();
  f.context.openMenu("settings");
  assert.deepEqual(f.visiblePages(), ["settings-content"]);
  assert.equal(f.calls.includes("paint-meishi"), false);
  assert.equal(
    f.calls.some((call) => Array.isArray(call) && call[0] === "paint-upgrades"),
    false,
  );
  f.context.openMenu("upgrades");
  assert.deepEqual(f.visiblePages(), ["upgrade-content"]);
  assert.equal(
    f.calls.filter(
      (call) => Array.isArray(call) && call[0] === "paint-upgrades",
    ).length,
    1,
  );
});

test("a world facility hit opens exactly its dedicated page without changing progression", async () => {
  const expected = [
    "upgrade-content",
    "hangar-content",
    "quest-content",
    "campaign-content",
  ];
  for (let index = 0; index < expected.length; index++) {
    const f = fixture(),
      c = f.context,
      before = structuredClone(f.profile);
    c.hit = index;
    assert.equal(c.interactDock([40, -25]), true);
    assert.deepEqual(f.visiblePages(), [expected[index]]);
    assert.equal(c.menu.hidden, false);
    assert.equal(c.paused, false);
    assert.equal(c.input.enabled, false);
    assert.equal(c.document.activeElement, f.element("menu-close"));
    assert.deepEqual(
      f.calls.find(
        (value) => Array.isArray(value) && value[0] === "facility-hit",
      ),
      ["facility-hit", 40, -25],
    );
    assert.deepEqual(f.profile, before);
    await settle();
  }
});

test("misses and home/launch/pause gates leave flight untouched", () => {
  for (const configure of [
    (c) => {
      c.hit = -1;
    },
    (c) => {
      c.hit = 4;
    },
    (c) => {
      c.hud.dock = false;
      c.hit = 0;
    },
    (c) => {
      c.launched = false;
      c.hit = 0;
    },
    (c) => {
      c.paused = true;
      c.hit = 0;
    },
  ]) {
    const f = fixture(),
      c = f.context;
    configure(c);
    const before = [c.paused, c.input.enabled];
    assert.equal(c.interactDock([0, 0]), false);
    assert.equal(c.menu.hidden, true);
    assert.deepEqual([c.paused, c.input.enabled], before);
    assert.equal(
      f.calls.some((value) => Array.isArray(value) && value[0] === "save"),
      false,
    );
  }
});

test("facility overlay handlers validate the authoritative hit before opening", async () => {
  const expected = [
    "upgrade-content",
    "hangar-content",
    "quest-content",
    "campaign-content",
  ];
  for (let index = 0; index < expected.length; index++) {
    const f = fixture();
    f.context.hit = (index + 1) % 4;
    f.element(`facility-${index}`).click();
    assert.equal(f.context.menu.hidden, true);
    f.context.hit = index;
    f.element(`facility-${index}`).click();
    assert.deepEqual(f.visiblePages(), [expected[index]]);
    await settle();
  }
});

test("closing a dedicated flight page resumes; closing title settings returns to title", async () => {
  for (const page of [
    "upgrades",
    "hangar",
    "journal",
    "campaign",
    "settings",
    "archive",
  ]) {
    const f = fixture(),
      c = f.context;
    c.openMenu(page);
    await f.element("menu-close").click();
    await settle();
    assert.equal(c.menu.hidden, true);
    assert.equal(c.paused, false);
    assert.equal(c.input.enabled, true);
    assert.equal(
      f.calls.some(
        (value) =>
          Array.isArray(value) && value[0] === "paused" && value[1] === true,
      ),
      false,
    );
  }
  const f = fixture(),
    c = f.context;
  c.launched = false;
  c.paused = true;
  c.input.enabled = false;
  c.launchScreen.hidden = false;
  c.openLaunchSettings();
  assert.deepEqual(f.visiblePages(), ["settings-content"]);
  assert.equal(c.beforeLaunchMenu, true);
  await f.element("menu-close").click();
  await settle();
  assert.equal(c.menu.hidden, true);
  assert.equal(c.launchScreen.hidden, false);
  assert.equal(c.launched, false);
  assert.equal(c.paused, true);
  assert.equal(c.input.enabled, false);
  assert.equal(f.calls.includes("audio-resume"), false);
});

test("pause settings and both backup actions select only their dedicated content", async () => {
  const f = fixture(),
    c = f.context;
  c.pause(false);
  f.element("pause-settings").click();
  assert.deepEqual(f.visiblePages(), ["settings-content"]);
  f.element("settings-backup").click();
  assert.deepEqual(f.visiblePages(), ["archive-content"]);
  await c.resume();
  f.element("save-indicator").click();
  assert.deepEqual(f.visiblePages(), ["archive-content"]);
  assert.equal(c.paused, false);
  await settle();
});

test("a live menu continues after hidden/resume and lifecycle gates never restore controls beneath it", async () => {
  const f = fixture(),
    c = f.context;
  c.openMenu("settings");
  c.document.hidden = true;
  c.syncFlightState();
  assert.equal(c.paused, true);
  assert.equal(c.input.enabled, false);
  c.document.hidden = false;
  c.syncFlightState();
  assert.equal(c.paused, false);
  assert.equal(c.input.enabled, false);
  assert.equal(c.menu.hidden, false);
  // Idempotent synchronization must not keep resetting Rust's accumulator.
  const pauseCalls = f.calls.filter(
    (v) => Array.isArray(v) && v[0] === "paused",
  ).length;
  c.syncFlightState();
  c.syncFlightState();
  assert.equal(
    f.calls.filter((v) => Array.isArray(v) && v[0] === "paused").length,
    pauseCalls,
  );
  await settle();
});

test("Quit waits for a committed save and flush, then provides a normal-tab fallback", async () => {
  const f = fixture(),
    c = f.context,
    gate = deferred();
  c.launched = false;
  c.paused = true;
  c.store.flush = () => gate.promise;
  const quitting = c.quitGame();
  await settle();
  assert.equal(f.element("quit-game").disabled, true);
  assert.equal(f.calls.includes("window-close"), false);
  gate.resolve();
  await quitting;
  assert.ok(f.calls.includes("window-close"));
  assert.match(c.loadStatus.textContent, /close this tab or app/);
  assert.equal(f.element("quit-game").disabled, false);
});

test("Quit write failure keeps the title available and does not close the browser", async () => {
  const f = fixture(),
    c = f.context;
  c.launched = false;
  c.store.save = async () => {
    throw new Error("save transaction failed");
  };
  await c.quitGame();
  assert.equal(f.calls.includes("window-close"), false);
  assert.equal(f.calls.includes("flush"), false);
  assert.equal(f.element("quit-game").disabled, false);
  assert.match(c.loadStatus.textContent, /SAVE NEEDS ATTENTION/);
  assert.ok(f.notices.includes("Error: save transaction failed"));
});

test("a delayed launch audio unlock cannot dismiss newer title Settings or Quit intent", async () => {
  for (const action of ["settings", "quit"]) {
    const f = fixture(),
      c = f.context,
      gate = deferred();
    c.launched = false;
    c.paused = true;
    c.input.enabled = false;
    c.audio.unlock = () => gate.promise;
    const launching = f.element("launch").click();
    if (action === "settings") c.openLaunchSettings();
    else await c.quitGame();
    gate.resolve();
    await launching;
    assert.equal(c.launched, false);
    assert.equal(c.input.enabled, false);
    if (action === "settings") {
      assert.equal(c.beforeLaunchMenu, true);
      assert.deepEqual(f.visiblePages(), ["settings-content"]);
      assert.equal(c.menu.hidden, false);
    }
    assert.equal(f.calls.includes("audio-resume"), false);
  }
});

test("dedicated-page close cannot resume through a recovery, modal, hidden or encounter gate", async () => {
  for (const block of [
    (c) => {
      c.recovering = true;
    },
    (c) => {
      c.graphicsFailed = true;
    },
    (c) => {
      c.webglLifecycle = { lost: true };
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
      c.preparingEncounter = 5;
    },
  ]) {
    const f = fixture(),
      c = f.context;
    c.openMenu("settings");
    block(c);
    c.syncFlightState();
    f.element("menu-close").click();
    await settle();
    assert.equal(c.menu.hidden, false);
    assert.equal(c.paused, true);
    assert.equal(c.input.enabled, false);
  }
});

test("restart ends the expedition through the authoritative boundary and waits for save plus flush before resuming", async () => {
  const f = fixture(),
    c = f.context,
    saved = deferred(),
    flushed = deferred();
  c.paused = true;
  c.input.enabled = false;
  c.menu.hidden = false;
  c.selectMenuTab("pause");
  c.store.save = (value) => {
    f.calls.push(["save", value.revision]);
    return saved.promise;
  };
  c.store.flush = () => {
    f.calls.push("flush");
    return flushed.promise;
  };
  f.element("restart-flight").click();
  const homeCall = f.calls.indexOf("return-home");
  const saveCall = f.calls.findIndex(
    (v) => Array.isArray(v) && v[0] === "save",
  );
  assert.ok(homeCall >= 0 && saveCall > homeCall);
  assert.deepEqual(f.calls[saveCall], ["save", 9]);
  assert.equal(c.preparingEncounter !== null, true);
  assert.equal(c.menu.hidden, false);
  await c.resume();
  assert.equal(c.input.enabled, false);
  saved.resolve();
  await settle();
  assert.ok(f.calls.includes("flush"));
  assert.equal(c.menu.hidden, false);
  assert.equal(c.input.enabled, false);
  flushed.resolve();
  await settle();
  assert.equal(c.preparingEncounter, null);
  assert.equal(c.menu.hidden, true);
  assert.equal(c.input.enabled, true);
  const flushedCall = f.calls.indexOf("flush");
  const unpause = f.calls.findIndex(
    (value) =>
      Array.isArray(value) && value[0] === "paused" && value[1] === false,
  );
  assert.ok(unpause > flushedCall);
  assert.equal(f.calls.filter((value) => value === "return-home").length, 1);
});

test("restart save failure leaves the menu/input safe and reports the actual failure", async () => {
  const f = fixture(),
    c = f.context;
  c.paused = true;
  c.input.enabled = false;
  c.menu.hidden = false;
  c.store.save = async () => {
    c.store.message = "Save write failed.";
    throw new Error("transaction denied");
  };
  f.element("restart-flight").click();
  await settle();
  assert.equal(c.preparingEncounter, null);
  assert.equal(c.menu.hidden, false);
  assert.equal(c.input.enabled, false);
  assert.equal(f.calls.includes("flush"), false);
  assert.equal(c.saveFailed, true);
  assert.ok(f.notices.includes("Error: transaction denied"));
});

test("save indicator moves from pending to committed only after the adapter commits", async () => {
  const f = fixture(),
    c = f.context,
    gate = deferred();
  c.store.save = () => gate.promise;
  const saving = c.persist(f.profile);
  const indicator = f.element("save-indicator");
  assert.equal(indicator.dataset.pixelValue, "1");
  assert.match(indicator.title, /Saving/);
  assert.match(indicator.attributes.get("aria-label"), /Saving/);
  assert.equal(f.element("save-status").textContent, "Saving…");
  assert.equal(f.element("save-status").title, indicator.title);
  gate.resolve();
  await saving;
  assert.equal(indicator.dataset.pixelValue, "0");
  assert.match(indicator.title, /saved automatically/);
  assert.equal(f.element("save-status").textContent, "Saved on this device.");
  assert.equal(f.element("save-status").title, indicator.title);
  assert.equal(c.pendingSaves, 0);
});

test("overlapping saves do not show saved while any write is outstanding", async () => {
  const f = fixture(),
    c = f.context,
    first = deferred(),
    second = deferred();
  let writes = 0;
  c.store.save = () => (++writes === 1 ? first : second).promise;
  const one = c.persist(f.profile),
    two = c.persist({ ...f.profile, revision: 10 });
  first.resolve();
  await one;
  assert.equal(c.pendingSaves, 1);
  assert.equal(f.element("save-indicator").dataset.pixelValue, "1");
  second.resolve();
  await two;
  assert.equal(c.pendingSaves, 0);
  assert.equal(f.element("save-indicator").dataset.pixelValue, "0");
});

test("write rejection exposes attention state and a successful retry clears it", async () => {
  const f = fixture(),
    c = f.context,
    gate = deferred();
  c.store.save = () => gate.promise;
  c.store.message = "Export before closing: storage write failed.";
  const saving = c.persist(f.profile);
  gate.reject(new Error("quota exceeded"));
  await assert.rejects(saving, /quota exceeded/);
  assert.equal(f.element("save-indicator").dataset.pixelValue, "2");
  assert.equal(f.element("save-indicator").title, c.store.message);
  assert.equal(f.element("save-status").textContent, "Save needs attention.");
  assert.equal(f.element("save-status").title, c.store.message);
  assert.match(
    f.element("save-indicator").attributes.get("aria-label"),
    /needs attention/,
  );
  c.store.save = async () => {};
  await c.persist(f.profile);
  assert.equal(f.element("save-indicator").dataset.pixelValue, "0");
});

test("memory-only storage never reports a successful persistent save", async () => {
  const f = fixture(),
    c = f.context;
  c.store.persistent = false;
  c.store.message = "Storage is unavailable. Export your progress.";
  await c.persist(f.profile);
  assert.equal(f.element("save-indicator").dataset.pixelValue, "2");
  assert.equal(f.element("save-status").textContent, "Save needs attention.");
  assert.equal(f.element("save-status").title, c.store.message);
  assert.doesNotMatch(f.element("save-indicator").title, /saved automatically/);
});

test("a SaveConflict opens the live menu once without recursively scheduling rejected saves", async () => {
  const f = fixture(),
    c = f.context;
  let writes = 0;
  c.store.save = async () => {
    writes++;
    throw new c.SaveConflict("another tab owns this save");
  };
  c.save(f.profile);
  await settle();
  assert.equal(c.menu.hidden, false);
  assert.equal(c.paused, false);
  assert.equal(c.input.enabled, false);
  assert.equal(writes, 2); // initial write, then one menu-entry checkpoint
  const stable = writes;
  await settle();
  assert.equal(writes, stable);
});

test("dedicated map and settings icons replace the live page and map toggles back to flight", async () => {
  const f = fixture(),
    c = f.context;
  f.element("map-button").click();
  assert.equal(c.menuTab, "map");
  assert.deepEqual(f.visiblePages(), ["map-content"]);
  assert.equal(c.paused, false);
  assert.equal(c.input.enabled, false);
  assert.ok(f.calls.includes("query-discovered-map"));
  f.element("flight-settings").click();
  assert.deepEqual(f.visiblePages(), ["settings-content"]);
  assert.equal(c.paused, false);
  f.element("map-button").click();
  f.element("map-button").click();
  await settle();
  assert.equal(c.menu.hidden, true);
  assert.equal(c.input.enabled, true);
});

test("M uses the discovered map without a simulation pause or a repeat toggle", async () => {
  const f = fixture(),
    c = f.context;
  f.press("m");
  assert.equal(c.menuTab, "map");
  assert.equal(c.paused, false);
  assert.equal(c.input.enabled, false);
  f.press("m", { repeat: true });
  assert.equal(c.menu.hidden, false);
  f.press("m");
  await settle();
  assert.equal(c.menu.hidden, true);
  assert.equal(c.input.enabled, true);
  assert.ok(
    !f.calls.some((call) => Array.isArray(call) && call[0] === "paused"),
  );
});

test("a delayed map course cannot consume a newer Settings intent", async () => {
  const f = fixture(),
    c = f.context;
  c.openMenu("map");
  const gate = deferred();
  c.audio.resume = () => gate.promise;
  const pending = c.navigate([5000, 1000]);
  c.openMenu("settings");
  gate.resolve();
  await pending;
  assert.equal(c.menuTab, "settings");
  assert.equal(c.menu.hidden, false);
  assert.equal(c.input.enabled, false);
  assert.ok(
    !f.calls.some((call) => Array.isArray(call) && call[0] === "set-course"),
  );
  assert.ok(!f.notices.some((value) => value.startsWith("COURSE SET")));
});

test("map navigation sets a real course after resume even when optional audio is unavailable", async () => {
  const f = fixture(),
    c = f.context;
  c.openMenu("map");
  c.audio.resume = async () => {
    throw new Error("audio unavailable");
  };
  await c.navigate([5000, 1000]);
  assert.equal(c.menu.hidden, true);
  assert.equal(c.input.enabled, true);
  assert.ok(
    f.calls.some(
      (call) =>
        Array.isArray(call) &&
        call[0] === "set-course" &&
        call[1] === 5000 &&
        call[2] === 1000,
    ),
  );
});
