import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

// Execute real shell functions plus the real view with fake DOM, Game and save
// adapters. This does not verify Rust recipes, IndexedDB or browser rendering.
const main = ts.createSourceFile(
  "main.ts",
  readFileSync(new URL("../../apps/game/src/main.ts", import.meta.url), "utf8"),
  ts.ScriptTarget.Latest,
  true,
);
const html = readFileSync(
  new URL("../../apps/game/index.html", import.meta.url),
  "utf8",
);
const viewSource = readFileSync(
  new URL("../../apps/game/src/loadout.ts", import.meta.url),
  "utf8",
).replace('import "./loadout.css";', "");
function fn(name) {
  const node = main.statements.find(
    (node) => ts.isFunctionDeclaration(node) && node.name?.text === name,
  );
  assert.ok(node, `Missing actual function ${name}`);
  return node.getText(main);
}
function declaration(name) {
  const node = main.statements.find(
    (node) =>
      ts.isVariableStatement(node) &&
      node.declarationList.declarations.some(
        (item) => item.name.getText(main) === name,
      ),
  );
  assert.ok(node, `Missing actual declaration ${name}`);
  return node.getText(main);
}
const compiledMain = ts.transpileModule(
  [
    ...[
      "paintSaveStatus",
      "persist",
      "save",
      "forgeMutation",
      "paintUpgrades",
      "pause",
      "canSimulateFlight",
      "syncFlightState",
      "openMenu",
      "selectMenuTab",
    ].map(fn),
    declaration("campaignActions"),
    "globalThis.hangarActions = campaignActions;",
  ].join("\n"),
  { compilerOptions: { target: ts.ScriptTarget.ES2022 } },
).outputText;
const compiledView = ts.transpileModule(viewSource, {
  compilerOptions: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  },
}).outputText;
const flatten = (root) => [root, ...root.children.flatMap(flatten)];
const copy = (value) => JSON.parse(JSON.stringify(value));
const contents = (root) =>
  flatten(root)
    .map((item) => item.textContent)
    .join("\n");
function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
async function settle() {
  for (let i = 0; i < 8; i++) await Promise.resolve();
}
function ship(id, selected, xp, level) {
  return {
    id,
    name: id ? "DAWNNEEDLE" : "HAVEN SCOUT",
    ability: "Hull blueprint.",
    family: 0,
    owned: true,
    selected,
    category: 0,
    craftable: false,
    xp,
    level,
    next_xp: level === 1 ? 20 : 120,
    rank_cap: level === 1 ? 1 : 2,
    point_cost: 10,
    material_tier: 1,
    material_cost: 6,
    can_craft: false,
    can_select: !selected,
    craft_reason: "Already owned.",
    select_reason: "",
  };
}
function initialHud() {
  return {
    dock: true,
    ship_level: 3,
    ship_xp: 60,
    ship_next_xp: 120,
    rank_cap: 2,
    currency: [3, 4, 5],
    material_stash: [10, 2, 0, 0, 0],
    material_cargo: [0, 0, 0, 0, 0],
    material_loss: [0, 0, 0, 0, 0],
    material_loss_reason: "",
    quest_log: [],
    discoveries: 0,
    relics: 0,
    boss_progress: [],
    campaign: {
      selected_ship: 0,
      ships: [ship(0, true, 60, 3), ship(1, false, 0, 1)],
      credits_pending: false,
    },
    loadout: [null, null, null],
    abilities: [0, 1, 2].map((category) => ({
      id: ["pulse-lens", "guard-coil", "flux-drive"][category],
      index: category,
      name: ["PULSE LENS", "GUARD COIL", "FLUX DRIVE"][category],
      category,
      description: "Authored automatic effect.",
      blueprint: true,
      owned: false,
      rank: 0,
      effective_rank: 0,
      max_rank: 5,
      compatible: true,
      equipped: false,
      required_level: 1,
      material_tier: 1,
      material_cost: 2,
      point_cost: 3,
      can_craft: true,
      can_equip: false,
      can_upgrade: false,
      craft_reason: "",
      equip_reason: "Not owned.",
      upgrade_reason: "Not owned.",
    })),
    upgrades: [
      {
        id: "rate",
        name: "FIRE RATE",
        category: 0,
        rank: 0,
        effective_rank: 0,
        max_rank: 5,
        description: "Ship-owned hardware.",
        required_level: 1,
        material_tier: 1,
        material_cost: 1,
        point_cost: 10,
        can_purchase: true,
        purchase_reason: "",
      },
    ],
  };
}
function fixture() {
  const calls = [],
    notices = [],
    nodes = new Map();
  const document = { hidden: false, activeElement: null };
  class Element {
    children = [];
    dataset = {};
    attributes = {};
    events = {};
    textContent = "";
    className = "";
    disabled = false;
    hidden = false;
    scrollTop = 0;
    classList = {
      add: (name) => {
        this.className += ` ${name}`;
      },
      remove: (name) => {
        this.className = this.className
          .split(" ")
          .filter((item) => item !== name)
          .join(" ");
      },
      toggle: (name, on) => {
        on ? this.classList.add(name) : this.classList.remove(name);
      },
    };
    append(...items) {
      this.children.push(...items);
    }
    replaceChildren(...items) {
      this.children = items;
    }
    get lastElementChild() {
      return this.children.at(-1);
    }
    contains(item) {
      return (
        this === item || this.children.some((child) => child.contains(item))
      );
    }
    querySelectorAll() {
      return flatten(this).filter(
        (item) => item.tagName === "BUTTON" && item.dataset.loadoutAction,
      );
    }
    setAttribute(key, value) {
      this.attributes[key] = value;
    }
    addEventListener(name, callback) {
      this.events[name] = callback;
    }
    focus() {
      document.activeElement = this;
    }
    click() {
      if (!this.disabled) this.events.click?.();
    }
  }
  document.createElement = (name) =>
    Object.assign(new Element(), { tagName: name.toUpperCase() });
  document.body = document.createElement("body");
  for (const match of html.matchAll(
    /<([a-z][\w-]*)\b([^>]*\bid="([^"]+)"[^>]*)>/g,
  )) {
    const node = document.createElement(match[1]);
    node.hidden = /(?:^|\s)hidden(?:\s|$)/.test(match[2]);
    nodes.set(match[3], node);
  }
  const element = (id) => {
    assert.ok(nodes.has(id), `Missing actual DOM ID ${id}`);
    return nodes.get(id);
  };
  const exports = {};
  vm.runInNewContext(compiledView, { exports, document, HTMLElement: Element });
  let authoritativeHud = initialHud();
  let authoritativeProfile = {
    schema_version: 4,
    revision: 9,
    marker: "before",
  };
  const c = {
    document,
    HTMLElement: Element,
    element,
    hud: authoritativeHud,
    launched: true,
    needsDroidCreation: false,
    paused: false,
    preparingEncounter: null,
    recovering: false,
    graphicsFailed: false,
    webglLifecycle: null,
    debugConsole: { opened: false },
    dialoguePanel: {
      opened: false,
      close() {
        this.opened = false;
      },
    },
    matchMedia: () => ({ matches: false }),
    beforeLaunchMenu: false,
    transitionEpoch: 0,
    destination: null,
    menuTab: "upgrades",
    menu: element("menu-screen"),
    launchScreen: element("launch-screen"),
    deathScreen: element("death-screen"),
    waitingUpdate: null,
    pendingSaves: 0,
    saveFailed: false,
    store: {
      persistent: true,
      message: "Progress stays on this device.",
      async save(profile) {
        calls.push(["save", profile]);
      },
    },
    input: { enabled: true, reset() {} },
    pixelUi: { invalidate() {} },
    audio: {
      update() {},
      suspend() {},
      async resume() {},
      play(cue) {
        calls.push(["audio", cue.kind]);
      },
    },
    galaxy: {
      hud() {
        calls.push(["hud", authoritativeHud]);
        return authoritativeHud;
      },
      profile() {
        calls.push(["profile", authoritativeProfile]);
        return authoritativeProfile;
      },
      set_paused() {},
      cancel_course() {},
      set_ui_mode() {},
    },
    paintForge(target, value, actions) {
      calls.push(["paint-forge", value]);
      exports.paintForge(target, value, actions);
    },
    paintFlightMenu() {},
    resetForge(target) {
      calls.push(["reset-forge", target]);
      exports.resetForge(target);
    },
    paintShipHangar(target, value, actions) {
      calls.push(["paint-hangar", value]);
      exports.paintShipHangar(target, value, actions);
    },
    paintHud() {
      calls.push(["paint-hud"]);
    },
    paintCampaign() {},
    paintCredits() {},
    finishCredits() {},
    openCredits() {},
    navigate() {},
    beginChampion() {},
    beginTournament() {},
    toast(message) {
      notices.push(message);
    },
    SaveConflict: class SaveConflict extends Error {},
  };
  for (const method of [
    "purchase",
    "craft_ability",
    "upgrade_ability",
    "equip_ability",
    "craft_ship",
    "switch_ship",
  ]) {
    c.galaxy[method] = (...args) => {
      calls.push([method, ...args]);
      if (c.mutationError) throw c.mutationError;
      authoritativeHud = c.nextHud;
      authoritativeProfile = c.nextProfile;
    };
  }
  c.nextHud = copy(authoritativeHud);
  c.nextHud.currency[0] = 0;
  c.nextHud.marker = "fresh HUD";
  c.nextProfile = {
    schema_version: 4,
    revision: 10,
    marker: "real post-mutation snapshot",
  };
  vm.runInNewContext(compiledMain, c);
  return {
    c,
    calls,
    notices,
    element,
    exports,
    action: (key) =>
      flatten(element("upgrade-content")).find(
        (node) => node.dataset.loadoutAction === key,
      ),
    hangarAction: (key) =>
      flatten(element("hangar-content")).find(
        (node) => node.dataset.loadoutAction === key,
      ),
  };
}

test("Forge forwards exact craft/upgrade/equip/empty/hardware arguments, refreshes HUD, then saves the actual snapshot", async () => {
  const operations = [
    ["purchase", ["rate"], "RAICHO HARDWARE UPGRADED"],
    ["craftAbility", ["pulse-lens"], "ABILITY CRAFTED"],
    ["upgradeAbility", ["pulse-lens"], "ABILITY UPGRADED"],
    ["equipAbility", [0, "pulse-lens"], "ABILITY EQUIPPED"],
    ["equipAbility", [2, null], "ABILITY SLOT EMPTY"],
  ];
  const methods = {
    purchase: "purchase",
    craftAbility: "craft_ability",
    upgradeAbility: "upgrade_ability",
    equipAbility: "equip_ability",
  };
  for (const [name, args, notice] of operations) {
    const f = fixture();
    let bindings;
    f.c.paintForge = (target, value, actions) => {
      bindings = actions;
      f.exports.paintForge(target, value, actions);
    };
    f.c.paintUpgrades(true);
    bindings[name](...args);
    await settle();
    assert.deepEqual(f.calls.slice(0, 4), [
      [methods[name], ...args],
      ["hud", f.c.nextHud],
      ["profile", f.c.nextProfile],
      ["save", f.c.nextProfile],
    ]);
    assert.equal(f.c.hud, f.c.nextHud);
    assert.ok(f.notices.some((value) => value.startsWith(notice)));
    assert.ok(
      !f.notices.some((value) => /saved|banked|deposited/i.test(value)),
    );
  }
});
test("backend rejection has no HUD fetch, snapshot save, audio cue or invented success", async () => {
  const f = fixture();
  f.c.mutationError = new Error("Tier 2 dock material required.");
  let actions;
  f.c.paintForge = (target, value, bindings) => {
    actions = bindings;
    f.exports.paintForge(target, value, bindings);
  };
  f.c.paintUpgrades(true);
  actions.upgradeAbility("guard-coil");
  await settle();
  assert.deepEqual(f.calls, [["upgrade_ability", "guard-coil"]]);
  assert.deepEqual(f.notices, ["Error: Tier 2 dock material required."]);
});
test("real Forge category survives successful repaint while a fresh open resets to overview", async () => {
  const f = fixture();
  f.c.dialoguePanel.opened = true;
  f.c.paused = true;
  f.c.openMenu("upgrades");
  assert.equal(f.c.dialoguePanel.opened, false);
  assert.equal(f.c.menu.hidden, false);
  assert.equal(f.c.document.activeElement, f.element("menu-close"));
  assert.equal(f.c.input.enabled, false);
  assert.equal(f.c.paused, false, "the live Forge does not pause the universe");
  f.action("manage-0").click();
  assert.ok(f.action("back"));
  const resets = f.calls.filter((item) => item[0] === "reset-forge").length;
  f.action("craft-pulse-lens").click();
  await settle();
  assert.ok(f.action("back"));
  assert.equal(
    f.calls.filter((item) => item[0] === "reset-forge").length,
    resets,
  );
  assert.match(contents(f.element("upgrade-content")), /OFFENSIVE \/ 0 PTS/);
  f.c.openMenu("upgrades");
  assert.equal(f.action("back"), undefined);
  assert.ok(f.action("manage-0"));
  assert.equal(
    f.calls.filter((item) => item[0] === "reset-forge").length,
    resets + 1,
  );
});
test("mutation success is not a committed-save claim while deferred persistence fails", async () => {
  const f = fixture();
  const pending = deferred();
  f.c.store.save = (profile) => {
    f.calls.push(["save", profile]);
    return pending.promise;
  };
  f.c.paintUpgrades(true);
  f.action("manage-0").click();
  f.action("craft-pulse-lens").click();
  assert.equal(f.c.pendingSaves, 1);
  assert.equal(f.element("save-indicator").dataset.pixelValue, "1");
  assert.equal(f.c.hud, f.c.nextHud);
  assert.ok(f.notices.some((value) => value.startsWith("ABILITY CRAFTED")));
  f.c.store.message = "Storage write failed. Export your progression.";
  pending.reject(new Error("QuotaExceeded"));
  await settle();
  assert.equal(f.c.saveFailed, true);
  assert.equal(f.element("save-indicator").dataset.pixelValue, "2");
  assert.equal(f.c.hud, f.c.nextHud);
  assert.ok(f.notices.includes(f.c.store.message));
  assert.equal(f.calls.filter((item) => item[0] === "save").length, 1);
});
test("Hangar page and construction/selection bindings pass full per-hull XP metadata", async () => {
  const f = fixture();
  f.c.selectMenuTab("hangar");
  assert.match(contents(f.element("hangar-content")), /RAICHO LV 3 \/ 60 XP/);
  assert.match(contents(f.element("hangar-content")), /RAICHO LV 1 \/ 0 XP/);
  f.calls.length = 0;
  f.c.hangarActions.craft(7);
  await settle();
  assert.deepEqual(f.calls.slice(0, 4), [
    ["craft_ship", 7],
    ["hud", f.c.nextHud],
    ["profile", f.c.nextProfile],
    ["save", f.c.nextProfile],
  ]);
  assert.equal(
    f.calls.find((item) => item[0] === "paint-hangar")[1],
    f.c.nextHud,
  );
  const next = copy(f.c.nextHud);
  next.campaign.selected_ship = 1;
  next.campaign.ships[0].selected = false;
  next.campaign.ships[1].selected = true;
  next.ship_level = 1;
  next.ship_xp = 0;
  next.ship_next_xp = 20;
  f.c.nextHud = next;
  f.calls.length = 0;
  f.c.hangarActions.select(1);
  await settle();
  assert.equal(f.calls[0][0], "switch_ship");
  assert.equal(f.calls[0][1], 1);
  assert.equal(f.calls.find((item) => item[0] === "paint-hangar")[1], next);
  assert.match(
    contents(f.element("upgrade-content")),
    /DAWNNEEDLE\nRAICHO LV 1 \/ 0 XP/,
  );
  assert.match(contents(f.element("hangar-content")), /RAICHO LV 3 \/ 60 XP/);
});
test("rejected ship construction or switching cannot save or report success", async () => {
  for (const [action, method, argument] of [
    ["craft", "craft_ship", 0],
    ["select", "switch_ship", 1],
  ]) {
    const f = fixture();
    f.c.mutationError = new Error("Return physically to Haven.");
    f.c.hangarActions[action](argument);
    await settle();
    assert.deepEqual(f.calls, [[method, argument]]);
    assert.deepEqual(f.notices, ["Error: Return physically to Haven."]);
  }
});
test("an unavailable audio context cannot reject a successful mutation/save flow", async () => {
  const f = fixture();
  f.c.audio.resume = async () => {
    throw new Error("Audio context unavailable.");
  };
  f.c.paintUpgrades(true);
  f.action("manage-0").click();
  f.action("craft-pulse-lens").click();
  await settle();
  assert.equal(f.c.hud, f.c.nextHud);
  assert.equal(f.calls.filter((item) => item[0] === "save").length, 1);
  assert.equal(f.calls.filter((item) => item[0] === "audio").length, 0);
  assert.ok(f.notices.some((value) => value.startsWith("ABILITY CRAFTED")));
});
