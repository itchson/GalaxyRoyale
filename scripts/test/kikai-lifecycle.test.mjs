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
const portraitExports = {};
vm.runInNewContext(
  compile(
    readFileSync(
      new URL("../../apps/game/src/kikai-portrait.ts", import.meta.url),
      "utf8",
    ),
    true,
  ),
  { exports: portraitExports },
);

function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
const settle = () => new Promise((resolve) => setImmediate(resolve));

function panelFixture(callbacks = {}) {
  let document;
  class Element {
    children = [];
    dataset = {};
    attributes = new Map();
    events = new Map();
    isConnected = true;
    classList = { add() {}, remove() {}, toggle() {} };
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
    hidden: false,
    body: new Element(),
    activeElement: null,
    createElement: () => new Element(),
    getElementById: (id) => find(document.body, id),
  };
  document.activeElement = document.body;
  const exports = {};
  vm.runInNewContext(compile(dialogueSource, true), {
    exports,
    require: (name) => {
      assert.equal(name, "./kikai-portrait");
      return portraitExports;
    },
    Error,
    document,
    HTMLElement: Element,
    window: { addEventListener() {}, removeEventListener() {} },
  });
  let closes = 0;
  const panel = new exports.DialoguePanel({
    onOpen() {
      callbacks.onOpen?.();
    },
    onClose() {
      closes++;
      callbacks.onClose?.();
    },
  });
  return {
    panel,
    document,
    Element,
    element: (id) => find(document.body, id),
    closes: () => closes,
    click: (id) => find(document.body, id).events.get("click")(),
  };
}

function guideFixture(
  training = { stage: 0, description: "Accept the flight license." },
  configure = () => {},
) {
  const callbacks = {},
    panel = panelFixture(callbacks),
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
    coop: null,
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
  configure(context, panel, callbacks);
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
    context,
    save,
    toasts,
    acknowledgements: () => acknowledgements,
    writes: () => writes,
    pages,
  };
}

function conflictFixture() {
  const saves = [],
    modes = [],
    pauses = [];
  const profile = {
    revision: 7,
    currency: [3, 4, 5],
    ship_xp: [0],
    materials: [2],
  };
  const f = guideFixture(undefined, (context, panel, callbacks) => {
    const nodes = new Map();
    const html = readFileSync(
      new URL("../../apps/game/index.html", import.meta.url),
      "utf8",
    );
    for (const match of html.matchAll(/\bid="([^"]+)"/g)) {
      const node = new panel.Element();
      node.id = match[1];
      nodes.set(node.id, node);
    }
    const element = (id) => {
      assert.ok(nodes.has(id), `Missing actual HTML id ${id}`);
      return nodes.get(id);
    };
    class SaveConflict extends Error {}
    Object.assign(context, {
      document: panel.document,
      element,
      SaveConflict,
      menu: element("menu-screen"),
      deathScreen: element("death-screen"),
      needsDroidCreation: false,
      pendingSaves: 0,
      saveFailed: false,
      transitionEpoch: 0,
      destination: [10, 20],
      waitingUpdate: null,
      beforeLaunchMenu: false,
      preparingEncounter: null,
      recovering: false,
      graphicsFailed: false,
      webglLifecycle: null,
      debugConsole: { opened: false },
      paused: false,
      lastTime: 10,
      menuTab: "pause",
      input: { enabled: true, reset() {} },
      audio: {
        setTitleScreen() {},
        update() {},
        resume: async () => {},
        suspend() {},
      },
      matchMedia: () => ({ matches: false }),
      paintUpgrades() {},
      paintMeishiCard() {
        return true;
      },
      paintCargo() {
        return true;
      },
      paintFlightMenu() {},
      store: {
        persistent: true,
        message: "A newer save exists. Reload to continue.",
        save(snapshot) {
          assert.deepEqual(snapshot, profile);
          const write = deferred();
          saves.push(write);
          return write.promise;
        },
      },
    });
    context.menu.hidden = true;
    context.deathScreen.hidden = true;
    Object.assign(context.galaxy, {
      set_paused: (value) => pauses.push(value),
      set_ui_mode: (mode) => modes.push(mode),
      cancel_course() {},
      profile: () => profile,
    });
    const options = main.statements
      .filter(ts.isVariableStatement)
      .flatMap((statement) => [...statement.declarationList.declarations])
      .find((node) => node.name.getText(main) === "dialoguePanel")?.initializer
      ?.arguments?.[0];
    assert.ok(options && ts.isObjectLiteralExpression(options));
    for (const name of ["onOpen", "onClose"]) {
      const property = options.properties.find(
        (node) => node.name?.getText(main) === name,
      );
      assert.ok(property && ts.isPropertyAssignment(property));
      callbacks[name] = vm.runInContext(
        compile(`(${property.initializer.getText(main)})`),
        context,
      );
    }
    vm.runInContext(
      compile(
        [
          "paintSaveStatus",
          "persist",
          "save",
          "pause",
          "canSimulateFlight",
          "syncFlightState",
          "restoreUi",
          "selectMenuTab",
        ]
          .map(actualFunction)
          .join("\n"),
      ),
      context,
    );
    // The first flight checkpoint is already pending when the contact opens.
    vm.runInContext("save(galaxy.profile())", context);
  });
  return { ...f, saves, profile, modes, pauses };
}

for (const pendingAcknowledgement of [false, true]) {
  test(`delayed flight SaveConflict closes the guide before showing the live menu${pendingAcknowledgement ? " with its action pending" : ""}`, async () => {
    const f = conflictFixture();
    const before = JSON.stringify(f.profile);
    const token = f.panel.presentationToken;
    assert.equal(f.panel.opened, true);
    assert.equal(f.context.menu.hidden, true);
    assert.equal(f.context.paused, true);
    const action = pendingAcknowledgement
      ? f.click("dialogue-action")
      : undefined;
    f.saves[0].reject(new f.context.SaveConflict("Newer checkpoint"));
    await settle();
    assert.equal(f.panel.opened, false);
    assert.equal(f.element("dialogue-screen").hidden, true);
    assert.ok(f.panel.presentationToken > token);
    assert.equal(f.context.menu.hidden, false);
    assert.equal(f.context.document.activeElement.id, "menu-close");
    assert.equal(f.context.input.enabled, false);
    assert.equal(
      f.context.paused,
      false,
      "the menu leaves the universe running",
    );
    assert.equal(f.modes.at(-1), 3);
    assert.equal(f.acknowledgements(), pendingAcknowledgement ? 1 : 0);
    assert.equal(
      JSON.stringify(f.profile),
      before,
      "menu transition grants no progression",
    );
    assert.ok(f.toasts.includes(f.context.store.message));
    const closedToken = f.panel.presentationToken;
    // The menu's checkpoint may also conflict; it must not open recursively.
    const menuSave = f.saves.at(-1);
    menuSave.reject(new f.context.SaveConflict("Still newer"));
    if (action) {
      f.saves[1].resolve();
      await action;
    }
    await settle();
    assert.equal(f.saves.length, pendingAcknowledgement ? 3 : 2);
    assert.equal(f.context.pendingSaves, 0);
    assert.equal(f.panel.opened, false);
    assert.equal(f.panel.presentationToken, closedToken);
    assert.equal(f.context.document.activeElement.id, "menu-close");
    assert.equal(f.acknowledgements(), pendingAcknowledgement ? 1 : 0);
    assert.equal(f.closes(), 1);
    assert.equal(JSON.stringify(f.profile), before);
  });
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
    coop: null,
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
    audio: { setTitleScreen() {}, resume: async () => {}, suspend() {} },
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
