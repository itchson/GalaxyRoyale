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
const compiledModules = Object.fromEntries(
  ["loadout", "forge-scene", "hangar-scene", "flight-menu"].map((name) => [
    name,
    ts.transpileModule(
      readFileSync(
        new URL(`../../apps/game/src/${name}.ts`, import.meta.url),
        "utf8",
      ),
      {
        compilerOptions: {
          target: ts.ScriptTarget.ES2022,
          module: ts.ModuleKind.CommonJS,
        },
      },
    ).outputText,
  ]),
);
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
const backHandler = main.statements.find(
  (node) =>
    ts.isExpressionStatement(node) &&
    /^element\("ship-hangar-back"\)\.addEventListener/.test(node.getText(main)),
);
assert.ok(backHandler, "Missing actual ship detail back handler");
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
      "refreshHangar",
    ].map(fn),
    declaration("campaignActions"),
    "globalThis.hangarActions = campaignActions;",
    backHandler.getText(main),
  ].join("\n"),
  { compilerOptions: { target: ts.ScriptTarget.ES2022 } },
).outputText;
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
    hull: 100,
    max_hull: 100,
    shield: 0,
    max_shield: 0,
    ship_stats: {
      speed: 0,
      top_speed: 90,
      acceleration: 300,
      primary_damage: 10,
      fire_rate: 4,
      hull_regen: 0,
      shield_regen: 0,
      shield_recharge_unlocked: false,
    },
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
    components: [],
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
    scrollLeft = 0;
    parentElement = null;
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
      for (const item of items) {
        item.remove();
        item.parentElement = this;
        this.children.push(item);
      }
    }
    replaceChildren(...items) {
      for (const child of this.children) child.parentElement = null;
      this.children = [];
      this.append(...items);
    }
    remove() {
      if (this.parentElement)
        this.parentElement.children = this.parentElement.children.filter(
          (child) => child !== this,
        );
      this.parentElement = null;
    }
    get firstElementChild() {
      return this.children[0];
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
      if (key.startsWith("data-"))
        this.dataset[
          key.slice(5).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase())
        ] = value;
    }
    getAttribute(key) {
      if (key.startsWith("data-"))
        return (
          this.dataset[
            key
              .slice(5)
              .replace(/-([a-z])/g, (_, letter) => letter.toUpperCase())
          ] ?? null
        );
      return this.attributes[key] ?? null;
    }
    addEventListener(name, callback) {
      this.events[name] = callback;
    }
    focus() {
      document.activeElement = this;
    }
    scrollIntoView() {}
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
    node.id = match[3];
    node.hidden = /(?:^|\s)hidden(?:\s|$)/.test(match[2]);
    nodes.set(match[3], node);
  }
  const element = (id) => {
    assert.ok(nodes.has(id), `Missing actual DOM ID ${id}`);
    return nodes.get(id);
  };
  const loaded = {};
  function load(name) {
    if (loaded[name]) return loaded[name];
    assert.ok(compiledModules[name], `Unexpected runtime view module ${name}`);
    const exports = (loaded[name] = {});
    vm.runInNewContext(compiledModules[name], {
      exports,
      document,
      HTMLElement: Element,
      require(specifier) {
        if (specifier.endsWith(".css")) return {};
        // Cosmetic portrait boundary only; the fixture's legacy HUD has no portrait.
        if (specifier === "./kikai-portrait")
          return { paintLicensePortrait: () => false };
        assert.ok(specifier.startsWith("./"));
        return load(specifier.slice(2));
      },
    });
    return exports;
  }
  const exports = Object.assign({}, ...Object.keys(compiledModules).map(load));
  assert.match(
    html,
    /id="ship-content"[\s\S]*?id="ship-hangar-back"[\s\S]*?<div id="ship-statistics"><\/div>/,
  );
  element("ship-content").append(
    element("ship-hangar-back"),
    element("ship-statistics"),
  );
  let authoritativeHud = initialHud();
  let authoritativeProfile = {
    schema_version: 4,
    revision: 9,
    marker: "before",
  };
  const c = {
    coop: null,
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
      setTitleScreen(active) {
        this.titleScreen = active;
      },
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
    paintForgeScene(target, value, actions, hulls, changed) {
      calls.push(["paint-forge", value]);
      exports.paintForgeScene(target, value, actions, hulls, changed);
    },
    paintMeishiCard() {
      return true;
    },
    paintVault() {
      return true;
    },
    paintFlightMenu(target, value, expanded) {
      return exports.paintFlightMenu(target, value, expanded);
    },
    resetForgeScene(target) {
      calls.push(["reset-forge", target]);
      exports.resetForgeScene(target);
    },
    paintHangarScene(target, value, actions) {
      calls.push(["paint-hangar", value]);
      return exports.paintHangarScene(target, value, actions);
    },
    paintHud() {
      calls.push(["paint-hud"]);
    },
    refreshMissions() {},
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
    "craft_component",
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
    section: (label) =>
      flatten(element("upgrade-content")).find(
        (node) => node.tagName === "BUTTON" && node.textContent === label,
      ),
  };
}

test("Forge forwards exact craft/upgrade/equip/empty/hardware arguments, refreshes HUD, then saves the actual snapshot", async () => {
  const operations = [
    ["purchase", ["rate"], "RAICHO HARDWARE UPGRADED"],
    ["craftAbility", ["pulse-lens"], "ABILITY CRAFTED"],
    ["craftComponent", ["pulse-conductor"], "COMPONENT STORED AT HAVEN"],
    ["upgradeAbility", ["pulse-lens"], "ABILITY UPGRADED"],
    ["equipAbility", [0, "pulse-lens"], "ABILITY EQUIPPED"],
    ["equipAbility", [2, null], "ABILITY SLOT EMPTY"],
  ];
  const methods = {
    purchase: "purchase",
    craftAbility: "craft_ability",
    craftComponent: "craft_component",
    upgradeAbility: "upgrade_ability",
    equipAbility: "equip_ability",
  };
  for (const [name, args, notice] of operations) {
    const f = fixture();
    let bindings;
    f.c.paintForgeScene = (target, value, actions, hulls, changed) => {
      bindings = actions;
      f.exports.paintForgeScene(target, value, actions, hulls, changed);
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
  f.c.paintForgeScene = (target, value, bindings, hulls, changed) => {
    actions = bindings;
    f.exports.paintForgeScene(target, value, bindings, hulls, changed);
  };
  f.c.paintUpgrades(true);
  actions.upgradeAbility("guard-coil");
  await settle();
  assert.deepEqual(f.calls, [["upgrade_ability", "guard-coil"]]);
  assert.deepEqual(f.notices, ["Error: Tier 2 dock material required."]);
});
test("real Forge category survives successful repaint while a fresh open resets to the ability index", async () => {
  const f = fixture();
  f.c.dialoguePanel.opened = true;
  f.c.paused = true;
  f.c.openMenu("upgrades");
  assert.equal(f.c.dialoguePanel.opened, false);
  assert.equal(f.c.menu.hidden, false);
  assert.equal(f.c.document.activeElement, f.element("menu-close"));
  assert.equal(f.c.input.enabled, false);
  assert.equal(f.c.paused, false, "the live Forge does not pause the universe");
  assert.ok(f.action("ability-category-all"));
  assert.equal(f.action("manage-0"), undefined);
  f.action("ability-category-0").click();
  assert.equal(
    f.action("ability-category-0").getAttribute("aria-pressed"),
    "true",
  );
  const resets = f.calls.filter((item) => item[0] === "reset-forge").length;
  f.action("craft-pulse-lens").click();
  await settle();
  assert.equal(
    f.action("ability-category-0").getAttribute("aria-pressed"),
    "true",
  );
  assert.equal(
    f.calls.filter((item) => item[0] === "reset-forge").length,
    resets,
  );
  assert.match(
    contents(f.element("upgrade-content")),
    /RENKO POINTS 3 \/ HAVE 0/,
  );
  f.c.openMenu("upgrades");
  assert.equal(f.action("back"), undefined);
  assert.equal(f.action("manage-0"), undefined);
  assert.equal(
    f.action("ability-category-all").getAttribute("aria-pressed"),
    "true",
  );
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
test("Forge construction and Hangar selection preserve full per-hull XP and crafting does not auto-select", async () => {
  const f = fixture();
  f.c.selectMenuTab("hangar");
  assert.match(contents(f.element("hangar-content")), /LV 3 \/ 60 XP/);
  assert.match(contents(f.element("hangar-content")), /LV 1 \/ 0 XP/);
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
  assert.equal(
    f.calls.some((item) => item[0] === "switch_ship"),
    false,
  );
  assert.equal(f.c.hud.campaign.selected_ship, 0);
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
    /DAWNNEEDLE \/ LV 1 \/ RANK CAP/,
  );
  assert.equal(f.c.hud.ship_xp, 0);
  assert.equal(f.c.hud.ship_next_xp, 20);
  assert.match(contents(f.element("hangar-content")), /LV 3 \/ 60 XP/);
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
  f.action("craft-pulse-lens").click();
  await settle();
  assert.equal(f.c.hud, f.c.nextHud);
  assert.equal(f.calls.filter((item) => item[0] === "save").length, 1);
  assert.equal(f.calls.filter((item) => item[0] === "audio").length, 0);
  assert.ok(f.notices.some((value) => value.startsWith("ABILITY CRAFTED")));
});

test("component and hardware callbacks retain their actual Forge mode, navigation nodes and focused action", async () => {
  const f = fixture();
  f.c.hud.components = [
    {
      id: "pulse-conductor",
      index: 0,
      name: "PULSE CONDUCTOR",
      description: "Crafted offense part.",
      category: 0,
      count: 2,
      max_count: 20,
      required_level: 1,
      materials: [2, 0, 0, 0, 0],
      can_craft: true,
      craft_reason: "",
    },
  ];
  f.c.nextHud = copy(f.c.hud);
  f.c.nextHud.components[0].count = 3;
  f.c.paintUpgrades(true);
  const nav = f.section("ITEMS & MATERIALS");
  nav.click();
  const crafting = f.action("component-pulse-conductor");
  crafting.focus();
  crafting.click();
  await settle();
  assert.equal(f.section("ITEMS & MATERIALS"), nav);
  assert.equal(nav.getAttribute("aria-pressed"), "true");
  assert.match(
    contents(f.element("upgrade-content")),
    /COMPONENT WORKSHOP[\s\S]*BANKED 3\/20/,
  );
  assert.equal(
    f.c.document.activeElement.dataset.loadoutAction,
    "component-pulse-conductor",
  );
  assert.notEqual(f.c.document.activeElement, crafting);
  f.section("HARDWARE").click();
  assert.equal(f.action("component-pulse-conductor"), undefined);
  assert.ok(f.action("hardware-rate"));
  f.c.nextHud = copy(f.c.hud);
  f.c.nextHud.upgrades[0].rank = 1;
  f.action("hardware-rate").focus();
  f.action("hardware-rate").click();
  await settle();
  assert.equal(f.section("HARDWARE").getAttribute("aria-pressed"), "true");
  assert.equal(
    f.c.document.activeElement.dataset.loadoutAction,
    "hardware-rate",
  );
  assert.equal(f.action("component-pulse-conductor"), undefined);
  assert.equal(f.calls.filter((call) => call[0] === "save").length, 2);
});

test("the actual Forge hull blueprint action crafts once without exposing selection or auto-switching", async () => {
  const f = fixture();
  f.c.hud.campaign.ships.push({
    ...ship(2, false, 0, 1),
    owned: false,
    craftable: true,
    can_craft: true,
    can_select: false,
    craft_reason: "",
  });
  f.c.nextHud = copy(f.c.hud);
  f.c.nextHud.campaign.ships[2].owned = true;
  f.c.nextHud.campaign.ships[2].can_craft = false;
  f.c.openMenu("upgrades");
  await settle();
  f.section("RAICHO BLUEPRINTS").click();
  assert.equal(f.action("select-ship-1"), undefined);
  assert.ok(f.action("craft-ship-2"));
  f.calls.length = 0;
  f.action("craft-ship-2").click();
  await settle();
  assert.deepEqual(f.calls.slice(0, 4), [
    ["craft_ship", 1],
    ["hud", f.c.nextHud],
    ["profile", f.c.nextProfile],
    ["save", f.c.nextProfile],
  ]);
  assert.equal(
    f.calls.some((call) => call[0] === "switch_ship"),
    false,
  );
  assert.equal(f.c.hud.campaign.selected_ship, 0);
  assert.equal(f.action("craft-ship-2"), undefined);
  assert.equal(f.action("select-ship-2"), undefined);
  assert.equal(
    f.section("RAICHO BLUEPRINTS").getAttribute("aria-pressed"),
    "true",
  );
});

test("actual Hangar legacy art and separate expanded statistics keep the persistent back control", async () => {
  const f = fixture();
  f.c.openMenu("hangar");
  const hero = flatten(f.element("hangar-content")).find(
    (node) => node.dataset.pixelKind === "garage-bay",
  );
  assert.ok(hero);
  assert.equal(hero.children[0].dataset.pixelKind, "campaign-ship");
  assert.equal(hero.children[0].dataset.pixelValue, "9");
  const back = f.element("ship-hangar-back");
  hero.click();
  assert.equal(f.element("ship-content").hidden, false);
  assert.match(contents(f.element("ship-statistics")), /LV 3 \/ XP 60\/120/);
  assert.match(
    contents(f.element("ship-statistics")),
    /OFFENSE \/ EMPTY[\s\S]*DEFENSE \/ EMPTY[\s\S]*PASSIVE \/ EMPTY/,
  );
  assert.equal(f.element("ship-content").contains(back), true);
  f.c.hud.hull = 43;
  f.c.selectMenuTab("ship");
  assert.match(contents(f.element("ship-statistics")), /HULL 43 \/ 100/);
  assert.equal(f.element("ship-content").contains(back), true);
  assert.equal(f.element("ship-hangar-back"), back);
  back.click();
  assert.equal(f.element("hangar-content").hidden, false);
  assert.equal(f.element("ship-content").hidden, true);
  assert.equal(f.c.paused, false);
  assert.equal(f.c.input.enabled, false);
  await settle();
});
