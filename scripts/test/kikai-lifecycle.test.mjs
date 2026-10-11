import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";
import { splitPages } from "../../apps/game/src/narrative.ts";

// Execute actual shell callbacks and the actual dialogue revision/busy guards.
// DOM, native Game, and persistence adapters isolate the asynchronous boundary;
// this does not simulate a browser renderer or prove IndexedDB durability.
const main = ts.createSourceFile(
  "main.ts",
  readFileSync(new URL("../../apps/game/src/main.ts", import.meta.url), "utf8"),
  ts.ScriptTarget.Latest,
  true,
);
const compile = (source, commonJs = false) =>
  ts.transpileModule(source, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: commonJs ? ts.ModuleKind.CommonJS : ts.ModuleKind.None,
    },
  }).outputText;
const actualFunction = (name) => {
  const node = main.statements.find(
    (item) => ts.isFunctionDeclaration(item) && item.name?.text === name,
  );
  assert.ok(node, `Missing production ${name}`);
  return node.getText(main);
};
const creation = main.statements.find(
  (item) =>
    ts.isVariableStatement(item) &&
    item.declarationList.declarations.some(
      (declaration) => declaration.name.getText(main) === "droidCreation",
    ),
);
assert.ok(creation, "Missing production creator callbacks");
const guideSource = readFileSync(
  new URL("../../assets/narrative/dora-initial-mission.json", import.meta.url),
  "utf8",
);
const dialogueSource = readFileSync(
  new URL("../../apps/game/src/dialogue.ts", import.meta.url),
  "utf8",
).replace('import "./dialogue.css";', "");

function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
const settle = () => new Promise((resolve) => setImmediate(resolve));

function panelFixture() {
  let document;
  class Element {
    children = [];
    dataset = {};
    attributes = new Map();
    events = new Map();
    isConnected = true;
    append(...children) {
      this.children.push(...children);
    }
    setAttribute(key, value) {
      this.attributes.set(key, value);
    }
    removeAttribute(key) {
      this.attributes.delete(key);
    }
    addEventListener(name, callback) {
      this.events.set(name, callback);
    }
    focus() {
      document.activeElement = this;
    }
    remove() {
      this.isConnected = false;
    }
  }
  const find = (parent, id) =>
    parent.id === id
      ? parent
      : parent.children.map((child) => find(child, id)).find(Boolean);
  document = {
    body: new Element(),
    activeElement: null,
    createElement: () => new Element(),
    getElementById: (id) => find(document.body, id),
  };
  document.activeElement = document.body;
  const exports = {};
  vm.runInNewContext(compile(dialogueSource, true), {
    exports,
    Error,
    document,
    HTMLElement: Element,
    window: { addEventListener() {}, removeEventListener() {} },
  });
  let closes = 0;
  const panel = new exports.DialoguePanel({
    onOpen() {},
    onClose() {
      closes++;
    },
  });
  return {
    panel,
    element: (id) => find(document.body, id),
    closes: () => closes,
    click: (id) => find(document.body, id).events.get("click")(),
  };
}

function guideFixture(
  training = { stage: 0, description: "Accept the flight license." },
) {
  const panel = panelFixture(),
    save = deferred(),
    toasts = [];
  let acknowledgements = 0,
    writes = 0;
  const hud = {
    onboarding: training,
    identity: {
      short_designation: "LICENSE",
      starter_ship: "PAN Pathfinder",
      starter_ability: "Flux Drive",
      corporation_role: "Passive",
    },
  };
  const context = vm.createContext({
    hud,
    launched: true,
    deathScreen: { hidden: true },
    menu: { hidden: true },
    narrative: { close() {} },
    dialoguePanel: panel.panel,
    doraGuideSource: guideSource,
    splitPages,
    galaxy: {
      acknowledge_license() {
        acknowledgements++;
        hud.onboarding = { stage: 1, description: "Leave Haven." };
      },
      hud: () => hud,
      profile: () => ({ revision: 1 }),
    },
    persist() {
      writes++;
      return save.promise;
    },
    toast: (message) => toasts.push(message),
    pixelUi: { invalidate() {} },
  });
  vm.runInContext(compile(actualFunction("showDoraGuide")), context);
  vm.runInContext("showDoraGuide()", context);
  const pages = [panel.element("dialogue-text").textContent];
  while (!panel.element("dialogue-next").hidden) {
    assert.equal(panel.element("dialogue-action").hidden, true);
    assert.equal(acknowledgements, 0);
    panel.click("dialogue-next");
    pages.push(panel.element("dialogue-text").textContent);
  }
  assert.equal(panel.element("dialogue-action").hidden, false);
  return {
    ...panel,
    save,
    toasts,
    acknowledgements: () => acknowledgements,
    writes: () => writes,
    pages,
  };
}

test("training contact presents supplied HUD text and corporation fields without issuing progress", () => {
  for (const [material_name, asteroid_name, starter_name, starter_slot] of [
    ["Phase Silicate", "porous", "Flux Drive", "passive"],
    ["Iron Filament", "metallic", "Pulse Lens", "offense"],
    ["Coolant KESSHO", "Ice asteroid", "Guard Coil", "defense"],
  ]) {
    const description = `Collect ${material_name}: 4/10. Banked 2, carried 2.`;
    const f = guideFixture({
      stage: 3,
      version: 2,
      description,
      material_name,
      asteroid_name,
      starter_name,
      starter_slot,
    });
    assert.equal(f.pages[0], description);
    const authored = f.pages.join(" ");
    for (const value of [
      material_name,
      asteroid_name,
      starter_name,
      starter_slot,
    ])
      assert.ok(authored.includes(value));
    assert.ok(!/\{(?:material|asteroid|starter|slot)\}/.test(authored));
    assert.ok(f.pages.every((page) => page.length <= 256));
    assert.equal(f.acknowledgements(), 0);
    assert.equal(f.writes(), 0);
  }
});

test("a saved guide acknowledgement cannot close a newer contact", async () => {
  const f = guideFixture();
  const operation = f.click("dialogue-action");
  await f.click("dialogue-action");
  assert.equal(f.acknowledgements(), 1);
  assert.equal(f.writes(), 1);
  f.panel.close();
  f.panel.show({ speaker: "NEW CONTACT", text: "A different live mission." });
  const token = f.panel.presentationToken;
  f.save.resolve();
  await operation;
  assert.equal(f.panel.opened, true);
  assert.equal(f.panel.presentationToken, token);
  assert.equal(
    f.element("dialogue-text").textContent,
    "A different live mission.",
  );
  assert.deepEqual(f.toasts, []);
  assert.equal(f.closes(), 1);
});

test("a saved guide acknowledgement respects Back while persistence is pending", async () => {
  const f = guideFixture();
  const operation = f.click("dialogue-action");
  f.click("dialogue-previous");
  const text = f.element("dialogue-text").textContent;
  const token = f.panel.presentationToken;
  f.save.resolve();
  await operation;
  assert.equal(f.panel.opened, true);
  assert.equal(f.panel.presentationToken, token);
  assert.equal(f.element("dialogue-text").textContent, text);
  assert.equal(f.closes(), 0);
  assert.deepEqual(f.toasts, []);
});

test("an unchanged final guide page closes only after its acknowledgement persists", async () => {
  const f = guideFixture();
  const operation = f.click("dialogue-action");
  assert.equal(f.panel.opened, true);
  assert.equal(f.closes(), 0);
  f.save.resolve();
  await operation;
  assert.equal(f.acknowledgements(), 1);
  assert.equal(f.writes(), 1);
  assert.equal(f.panel.opened, false);
  assert.equal(f.closes(), 1);
  assert.deepEqual(f.toasts, ["Leave Haven."]);
});

test("failed guide persistence keeps the page actionable without acknowledging twice", async () => {
  const f = guideFixture();
  const operation = f.click("dialogue-action");
  f.save.reject(new Error("Save is unavailable."));
  await operation;
  assert.equal(f.panel.opened, true);
  assert.equal(f.element("dialogue-text").textContent, "Save is unavailable.");
  assert.equal(f.element("dialogue-action").disabled, false);
  await f.click("dialogue-action");
  assert.equal(f.acknowledgements(), 1);
  assert.equal(f.writes(), 1);
  assert.equal(f.panel.opened, false);
});

function creationFixture() {
  let callbacks,
    registrations = 0,
    guideCalls = 0,
    focusCalls = 0;
  const writes = [],
    modes = [];
  const hud = { identity: null };
  const context = vm.createContext({
    DroidCreation: class {
      constructor(options) {
        callbacks = options;
      }
    },
    needsDroidCreation: true,
    launched: false,
    launchScreen: { hidden: false },
    galaxy: {
      register_droid(...identity) {
        registrations++;
        hud.identity = identity;
      },
      set_ui_mode: (mode) => modes.push(mode),
      hud: () => hud,
      profile: () => ({ revision: 1, identity: hud.identity }),
      set_paused() {},
    },
    input: { reset() {}, enabled: false },
    pixelUi: { invalidate() {} },
    launch: { focus: () => focusCalls++, textContent: "BEGIN EXPEDITION" },
    hud: { campaign: { credits_pending: false } },
    persist(snapshot) {
      const write = deferred();
      writes.push({ ...write, snapshot });
      return write.promise;
    },
    paintHud() {},
    showDoraGuide: () => guideCalls++,
    menuTab: "pause",
    beforeLaunchMenu: false,
    preparingEncounter: null,
    recovering: false,
    graphicsFailed: false,
    webglLifecycle: null,
    debugConsole: { opened: false },
    dialoguePanel: { opened: false },
    document: { hidden: false, body: { classList: { add() {} } } },
    portrait: false,
    matchMedia: () => ({ matches: context.portrait }),
    menu: { hidden: true },
    deathScreen: { hidden: true },
    audio: { resume: async () => {}, suspend() {} },
    lastTime: 0,
    paused: true,
    syncFlightState() {
      context.paused = false;
    },
  });
  vm.runInContext(
    compile(actualFunction("resume") + "\n" + creation.getText(main)),
    context,
  );
  return {
    context,
    callbacks,
    writes,
    modes,
    registrations: () => registrations,
    guideCalls: () => guideCalls,
    focusCalls: () => focusCalls,
  };
}
const identity = {
  kind: "dora",
  corporation: "pan",
  sub_corporation: "renko",
  hex_id: "0123456789abcdef",
};
for (const boundary of [
  "hidden",
  "portrait",
  "graphics-failed",
  "recovering",
  "context-lost",
]) {
  test(`creator restores Launch if ${boundary} prevents flight after persistence`, async () => {
    const f = creationFixture();
    f.callbacks.onOpen();
    const operation = f.callbacks.create(identity);
    assert.equal(f.context.launchScreen.hidden, true);
    assert.equal(f.context.needsDroidCreation, true);
    if (boundary === "hidden") f.context.document.hidden = true;
    if (boundary === "portrait") f.context.portrait = true;
    if (boundary === "graphics-failed") f.context.graphicsFailed = true;
    if (boundary === "recovering") f.context.recovering = true;
    if (boundary === "context-lost") f.context.webglLifecycle = { lost: true };
    f.writes[0].resolve();
    await operation;
    f.callbacks.onClose();
    await settle();
    assert.equal(f.registrations(), 1);
    assert.equal(f.context.needsDroidCreation, false);
    assert.equal(f.context.launched, false);
    assert.equal(f.context.launchScreen.hidden, false);
    assert.deepEqual(f.modes, [0, 2]);
    assert.equal(f.focusCalls(), 1);
    assert.equal(f.guideCalls(), 0);
  });
}

test("healthy creator completion launches and opens the guide once", async () => {
  const f = creationFixture();
  f.callbacks.onOpen();
  const operation = f.callbacks.create(identity);
  f.writes[0].resolve();
  await operation;
  f.callbacks.onClose();
  await settle();
  assert.equal(f.registrations(), 1);
  assert.equal(f.context.launched, true);
  assert.equal(f.context.launchScreen.hidden, true);
  assert.deepEqual(f.modes, [0, 1]);
  assert.equal(f.guideCalls(), 1);
});

test("retrying failed creator persistence preserves the issued identity and one grant", async () => {
  const f = creationFixture();
  f.callbacks.onOpen();
  const first = f.callbacks.create(identity);
  f.writes[0].reject(new Error("Save failed."));
  await assert.rejects(first, /Save failed/);
  assert.equal(f.context.needsDroidCreation, true);
  assert.equal(f.registrations(), 1);
  const retry = f.callbacks.create({ ...identity, hex_id: "fedcba9876543210" });
  assert.deepEqual(
    f.writes[1].snapshot.identity,
    f.writes[0].snapshot.identity,
  );
  f.writes[1].resolve();
  await retry;
  assert.equal(f.registrations(), 1);
  assert.equal(f.context.needsDroidCreation, false);
});
