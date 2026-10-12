import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

// Actual view code, fake DOM only: not Rust authority, browser layout or saves.
const source = readFileSync(
  new URL("../../apps/game/src/loadout.ts", import.meta.url),
  "utf8",
).replace('import "./loadout.css";', "");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  },
}).outputText;
function fixture() {
  let document;
  class Element {
    children = [];
    dataset = {};
    attributes = {};
    events = {};
    className = "";
    textContent = "";
    disabled = false;
    scrollTop = 0;
    parent = null;
    classList = { add: (name) => (this.className += ` ${name}`) };
    append(...children) {
      for (const child of children) {
        child.parent = this;
        this.children.push(child);
      }
    }
    replaceChildren(...children) {
      this.children = [];
      this.append(...children);
    }
    get lastElementChild() {
      return this.children.at(-1);
    }
    contains(element) {
      return (
        element === this ||
        this.children.some((child) => child.contains(element))
      );
    }
    setAttribute(key, value) {
      this.attributes[key] = value;
    }
    addEventListener(key, value) {
      this.events[key] = value;
    }
    focus() {
      document.activeElement = this;
    }
    querySelectorAll() {
      return flatten(this).filter(
        (node) => node.tagName === "button" && node.dataset.loadoutAction,
      );
    }
  }
  document = {
    activeElement: null,
    createElement: (tag) => Object.assign(new Element(), { tagName: tag }),
  };
  const exports = {};
  vm.runInNewContext(compiled, { exports, document, HTMLElement: Element });
  const target = new Element();
  const calls = [];
  const actions = {
    purchase: (id) => calls.push(["purchase", id]),
    craftAbility: (id) => calls.push(["craft", id]),
    craftComponent: (id) => calls.push(["component", id]),
    upgradeAbility: (id) => calls.push(["upgrade", id]),
    equipAbility: (slot, id) => calls.push(["equip", slot, id]),
  };
  return {
    exports,
    target,
    document,
    actions,
    calls,
    find: (key) =>
      flatten(target).find((node) => node.dataset.loadoutAction === key),
  };
}
function flatten(root) {
  return [root, ...root.children.flatMap(flatten)];
}
function copy(value) {
  return JSON.parse(JSON.stringify(value));
}
function allText(root) {
  return flatten(root)
    .map((node) => node.textContent)
    .join("\n");
}
function ship(id, selected = false) {
  return {
    id,
    name: id ? "DAWNNEEDLE" : "HAVEN SCOUT",
    ability: "Chassis and blueprint.",
    family: 0,
    owned: true,
    selected,
    category: 0,
    craftable: true,
    level: id ? 1 : 3,
    xp: id ? 0 : 60,
    next_xp: id ? 20 : 120,
    rank_cap: id ? 1 : 2,
    point_cost: 10,
    material_tier: 1,
    material_cost: 6,
    can_craft: false,
    can_select: !selected,
    craft_reason: "Already owned.",
    select_reason: "",
  };
}
function ability(index, category, properties = {}) {
  return {
    id: `ability-${index}`,
    index,
    name: `ABILITY ${index}`,
    category,
    rank: 0,
    effective_rank: 0,
    description: "Actual authored effect.",
    blueprint: true,
    owned: false,
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
    ...properties,
  };
}
function hud() {
  return {
    dock: true,
    currency: [3, 4, 5],
    material_stash: [10, 2, 0, 0, 0],
    material_cargo: [0, 0, 0, 0, 0],
    material_loss: [0, 0, 0, 0, 0],
    material_loss_reason: "",
    campaign: { selected_ship: 0, ships: [ship(0, true), ship(1)] },
    ship_level: 3,
    ship_xp: 60,
    ship_next_xp: 120,
    rank_cap: 2,
    abilities: [ability(0, 0), ability(1, 1), ability(2, 2)],
    components: [],
    loadout: [null, null, null],
    upgrades: Array.from({ length: 6 }, (_, index) => ({
      id: `hardware-${index}`,
      name: `HARDWARE ${index}`,
      category: Math.floor(index / 2),
      rank: 0,
      effective_rank: 0,
      max_rank: 5,
      description: "Ship-owned stat.",
      required_level: 1,
      material_tier: 1,
      material_cost: 1,
      point_cost: 10,
      can_purchase: false,
      purchase_reason: "Insufficient category points.",
    })),
  };
}

test("first-upgrade recipe shows the actual named banked subset and calls the ordinary native upgrade action", () => {
  const f = fixture(),
    data = hud();
  data.ship_level = 1;
  data.rank_cap = 1;
  data.material_stash = [100, 0, 0, 0, 0];
  data.material_training_stash = [0, 7, 0];
  data.abilities[0] = {
    ...data.abilities[0],
    owned: true,
    rank: 1,
    effective_rank: 1,
    equipped: true,
    training_recipe: true,
    required_training_material: {
      id: "iron-filament",
      name: "Iron Filament",
      amount: 10,
    },
    material_tier: 1,
    material_cost: 10,
    point_cost: 0,
    required_level: 1,
    can_upgrade: false,
    upgrade_reason: "Bank 10 Iron Filament.",
  };
  data.loadout[0] = data.abilities[0];
  f.exports.paintForge(f.target, data, f.actions);
  f.find("manage-0").events.click();
  assert.match(
    allText(f.target),
    /FIRST UPGRADE \/ IRON FILAMENT 10 \/ BANKED 7/,
  );
  assert.equal(f.find("upgrade-ability-0").disabled, true);
  data.material_training_stash[1] = 10;
  data.abilities[0].can_upgrade = true;
  data.abilities[0].upgrade_reason = "";
  f.exports.paintForge(f.target, data, f.actions);
  f.find("upgrade-ability-0").events.click();
  assert.deepEqual(f.calls, [["upgrade", "ability-0"]]);
});

test("component workshop reveals only discovered recipes and delegates exact authoritative actions", () => {
  const f = fixture(),
    data = hud();
  f.exports.paintForge(f.target, data, f.actions);
  assert.equal(f.find("browse-components"), undefined);
  data.components = [
    {
      id: "pulse-conductor",
      index: 0,
      name: "Pulse Conductor",
      category: 0,
      description: "Stabilizes advanced offensive abilities.",
      count: 2,
      max_count: 99,
      required_level: 3,
      materials: [2, 1, 0, 0, 0],
      can_craft: false,
      craft_reason: "Return to Haven.",
    },
  ];
  f.exports.paintForge(f.target, data, f.actions);
  f.find("browse-components").events.click();
  assert.equal(f.target.scrollTop, 0);
  assert.equal(f.document.activeElement, f.find("components-back"));
  assert.match(allText(f.target), /BANKED 2\/99/);
  assert.match(allText(f.target), /T1 MATERIALS 2 \/ BANKED 10/);
  assert.match(allText(f.target), /T2 MATERIALS 1 \/ BANKED 2/);
  assert.doesNotMatch(allText(f.target), /Aegis Lattice|Flux Bearing|POINTS/);
  assert.equal(
    flatten(f.target).filter((n) => n.dataset.pixelKind === "craft-component")
      .length,
    1,
  );
  f.find("component-pulse-conductor").events.click();
  assert.deepEqual(f.calls, []);
  data.components[0].can_craft = true;
  data.components[0].craft_reason = "";
  f.exports.paintForge(f.target, data, f.actions);
  f.find("component-pulse-conductor").focus();
  f.target.scrollTop = 21;
  f.find("component-pulse-conductor").events.click();
  assert.deepEqual(f.calls, [["component", "pulse-conductor"]]);
  data.components[0].count = 3;
  f.exports.paintForge(f.target, data, f.actions);
  assert.equal(f.document.activeElement, f.find("component-pulse-conductor"));
  assert.equal(f.target.scrollTop, 21);
  assert.match(allText(f.target), /BANKED 3\/99/);
  f.exports.paintForge(f.target, data, { ...f.actions, busy: true });
  assert.equal(f.find("component-pulse-conductor").disabled, true);
  f.find("components-back").events.click();
  assert.equal(f.document.activeElement, f.find("browse-components"));
});

test("advanced ability recipes include banked crafted components without changing starter training", () => {
  const f = fixture(),
    data = hud();
  Object.assign(data.abilities[0], {
    owned: true,
    rank: 2,
    point_cost: 10,
    material_tier: 3,
    material_cost: 2,
    required_component: {
      id: "pulse-conductor",
      name: "Pulse Conductor",
      amount: 1,
      owned: 0,
    },
    can_upgrade: false,
    upgrade_reason: "Craft 1 Pulse Conductor.",
  });
  f.exports.paintForge(f.target, data, f.actions);
  f.find("manage-0").events.click();
  assert.match(allText(f.target), /RENKO POINTS 10 \/ HAVE 3/);
  assert.match(allText(f.target), /T3 MATERIALS 2 \/ BANKED 0/);
  assert.match(allText(f.target), /PULSE CONDUCTOR 1 \/ BANKED 0/);
  assert.equal(f.find("upgrade-ability-0").disabled, true);
});

test("Forge separates three automatic slots and six ship hardware actions", () => {
  const f = fixture();
  f.exports.paintForge(f.target, hud(), f.actions);
  assert.equal(
    flatten(f.target).filter((node) =>
      node.dataset.loadoutAction?.startsWith("manage-"),
    ).length,
    3,
  );
  assert.equal(
    flatten(f.target).filter((node) =>
      node.dataset.loadoutAction?.startsWith("hardware-"),
    ).length,
    6,
  );
  assert.match(allText(f.target), /RAICHO LV 3 \/ 60 XP/);
  assert.ok(!allText(f.target).includes("PLAYER LEVEL"));
});
test("category editor has contextual Back and explicit craft versus equip", () => {
  const f = fixture();
  const h = hud();
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-0").events.click();
  assert.ok(f.find("back"));
  assert.equal(
    f.target.children[0].children[0].children[0],
    f.find("back"),
    "the contextual exit precedes the ship summary in the scroll region",
  );
  assert.ok(f.find("craft-ability-0"));
  assert.equal(f.find("craft-ability-1"), undefined);
  f.find("craft-ability-0").events.click();
  assert.deepEqual(f.calls, [["craft", "ability-0"]]);
  f.find("back").events.click();
  assert.equal(f.document.activeElement, f.find("manage-0"));
  assert.equal(f.find("back"), undefined);
  assert.equal(f.target.scrollTop, 0);
});
test("availability is authoritative even when visible balances look sufficient or insufficient", () => {
  const f = fixture();
  const h = hud();
  h.currency = [0, 100, 100];
  h.material_stash = [0, 0, 0, 0, 0];
  h.abilities[0].can_craft = true;
  h.abilities[1].can_craft = false;
  h.abilities[1].craft_reason = "Incompatible hull family.";
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-0").events.click();
  assert.equal(f.find("craft-ability-0").disabled, false);
  f.find("back").events.click();
  f.find("manage-1").events.click();
  assert.equal(f.find("craft-ability-1").disabled, true);
  f.find("craft-ability-1").events.click();
  assert.equal(f.calls.length, 0);
  assert.match(allText(f.target), /Incompatible hull family/);
});
test("owned rank cap and replacement retain inventory and send exact category", () => {
  const f = fixture();
  const h = hud();
  const first = ability(0, 0, {
    owned: true,
    equipped: true,
    rank: 5,
    effective_rank: 2,
    can_upgrade: false,
    upgrade_reason: "Max rank.",
  });
  h.abilities[0] = first;
  h.loadout[0] = first;
  h.abilities.push(
    ability(7, 0, {
      owned: true,
      rank: 1,
      effective_rank: 1,
      can_equip: true,
      equip_reason: "",
      can_upgrade: true,
      upgrade_reason: "",
      material_tier: 2,
      material_cost: 2,
      point_cost: 10,
    }),
  );
  const original = copy(h);
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-0").events.click();
  assert.match(allText(f.target), /OWNED RANK 5 \/ ACTIVE RANK 2/);
  assert.equal(f.find("equip-ability-7").textContent, "REPLACE");
  f.find("equip-ability-7").events.click();
  f.find("upgrade-ability-7").events.click();
  f.find("empty-0").events.click();
  assert.deepEqual(f.calls, [
    ["equip", 0, "ability-7"],
    ["upgrade", "ability-7"],
    ["equip", 0, null],
  ]);
  assert.deepEqual(h, original);
  assert.match(allText(f.target), /T2 MATERIALS 2 \/ BANKED 2/);
});
test("actual HUD repaint preserves category, while ship change/reset returns overview", () => {
  const f = fixture();
  const h = hud();
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-2").events.click();
  f.target.scrollTop = 77;
  const updated = copy(h);
  updated.currency[2] = 50;
  f.exports.paintForge(f.target, updated, f.actions);
  assert.ok(f.find("back"));
  assert.equal(f.target.scrollTop, 77);
  assert.match(allText(f.target), /PAN POINTS 3 \/ HAVE 50/);
  updated.campaign.selected_ship = 1;
  updated.ship_level = 1;
  updated.ship_xp = 0;
  f.exports.paintForge(f.target, updated, f.actions);
  assert.equal(f.find("back"), undefined);
  assert.equal(f.target.scrollTop, 0);
  f.find("manage-1").events.click();
  f.exports.resetForge(f.target);
  f.exports.paintForge(f.target, updated, f.actions);
  assert.equal(f.find("back"), undefined);
});
test("busy mutation and outside-dock clearing cannot dispatch; cargo stays in its separate page", () => {
  const f = fixture();
  const h = hud();
  h.dock = false;
  h.material_loss = [1, 0, 2, 0, 0];
  h.material_loss_reason = "reload";
  h.abilities[0] = ability(0, 0, {
    owned: true,
    equipped: true,
    rank: 1,
    effective_rank: 1,
  });
  h.loadout[0] = h.abilities[0];
  f.exports.paintForge(f.target, h, { ...f.actions, busy: true });
  assert.match(allText(f.target), /Return to Haven to craft, upgrade or equip/);
  assert.doesNotMatch(allText(f.target), /MATERIALS LOST|CARRIED|DOCK STASH/);
  f.find("manage-0").events.click();
  assert.equal(f.find("empty-0").disabled, true);
  assert.equal(f.find("upgrade-ability-0").disabled, true);
  f.find("empty-0").events.click();
  assert.equal(f.calls.length, 0);
});
test("Hangar shows each hull's XP, construction recipe and exact selection callback", () => {
  const f = fixture();
  const h = hud();
  const calls = [];
  const locked = ship(2);
  locked.owned = false;
  locked.can_craft = false;
  locked.craft_reason = "Defeat the champion to earn its blueprint.";
  h.campaign.ships.push(locked);
  f.exports.paintShipHangar(f.target, h, {
    select: (id) => calls.push(["select", id]),
    craft: (id) => calls.push(["craft", id]),
  });
  assert.match(allText(f.target), /RAICHO LV 3 \/ 60 XP/);
  assert.match(allText(f.target), /RAICHO LV 1 \/ 0 XP/);
  assert.equal(f.find("select-ship-0").disabled, true);
  assert.equal(f.find("craft-ship-2").disabled, true);
  f.find("select-ship-1").events.click();
  assert.deepEqual(calls, [["select", 1]]);
  assert.match(allText(f.target), /T1 MATERIALS 6 \/ BANKED 10/);
});
test("all game-provided names and errors are plain text, including malicious markup", () => {
  const f = fixture();
  const h = hud();
  h.abilities[0].name = '<img src=x onerror="bad()">';
  h.abilities[0].craft_reason = "<script>bad()</script>";
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-0").events.click();
  assert.ok(allText(f.target).includes(h.abilities[0].name));
  assert.ok(allText(f.target).includes(h.abilities[0].craft_reason));
  assert.ok(
    flatten(f.target).every((node) => !Object.hasOwn(node, "innerHTML")),
  );
});
test("victory-only hull has no fake construction recipe and retains real LV1 reward XP", () => {
  const f = fixture();
  const h = hud();
  const royal = {
    ...ship(9),
    name: "ROYALE SOVEREIGN",
    owned: false,
    point_cost: 0,
    material_cost: 0,
    can_craft: false,
    craft_reason: "Win the tournament.",
    level: 1,
    xp: 0,
    next_xp: 20,
  };
  h.campaign.ships = [royal];
  f.exports.paintShipHangar(f.target, h, { select() {}, craft() {} });
  assert.equal(f.find("craft-ship-9").textContent, "LOCKED RAICHO");
  assert.ok(!allText(f.target).includes("POINTS 0 / HAVE"));
  assert.ok(!allText(f.target).includes("MATERIALS 0 / BANKED"));
  royal.owned = true;
  royal.can_select = true;
  f.exports.paintShipHangar(f.target, h, { select() {}, craft() {} });
  assert.match(allText(f.target), /RAICHO LV 1 \/ 0 XP \/ NEXT 20 XP/);
});
test("legacy stored hardware rank and active ship cap are shown separately without inventing power", () => {
  const f = fixture();
  const h = hud();
  h.upgrades[0].rank = 5;
  h.upgrades[0].effective_rank = 1;
  h.upgrades[0].can_purchase = false;
  h.upgrades[0].purchase_reason = "Maximum stored hardware rank.";
  f.exports.paintForge(f.target, h, f.actions);
  assert.match(allText(f.target), /OWNED RANK 5 \/ ACTIVE RANK 1 \/ MAX 5/);
  assert.equal(f.find("hardware-hardware-0").disabled, true);
  assert.equal(h.upgrades[0].rank, 5);
  assert.equal(h.upgrades[0].effective_rank, 1);
});

test("Forge prioritizes slots and collection access without repeating cargo inventory", () => {
  const f = fixture();
  const h = hud();
  h.material_loss = [2, 0, 3, 0, 0];
  h.material_loss_reason = "reload";
  f.exports.paintForge(f.target, h, f.actions);
  const nodes = flatten(f.target);
  const browse = nodes.indexOf(f.find("browse-abilities"));
  assert.ok(browse > 0);
  for (const category of [0, 1, 2]) {
    const control = f.find(`manage-${category}`);
    assert.ok(nodes.indexOf(control) < browse);
    assert.equal(control.disabled, false);
  }
  assert.doesNotMatch(
    allText(f.target),
    /CARGO:|MATERIALS LOST|DOCK STASH|CARRIED/,
  );
  assert.match(allText(f.target), /T1 MATERIALS 1 \/ BANKED 10/);
});

test("Forge excludes an undiscovered ability rather than showing its locked name", () => {
  const f = fixture();
  const h = hud();
  h.abilities[0].name = "UNDISCOVERED SECRET";
  h.abilities[0].blueprint = false;
  h.abilities[0].owned = false;
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-0").events.click();
  assert.ok(!allText(f.target).includes("UNDISCOVERED SECRET"));
});

test("ability filters run after discovery and distinguish ownership from blueprint and compatibility", () => {
  const f = fixture(),
    h = hud();
  h.abilities[0] = ability(0, 0, {
    owned: true,
    rank: 3,
    effective_rank: 1,
    equipped: true,
  });
  h.abilities[1] = ability(1, 1, {
    blueprint: false,
    owned: true,
    rank: 1,
    can_craft: false,
    compatible: false,
    equip_reason: "Incompatible hull.",
  });
  h.abilities.push(
    ability(8, 0, {
      blueprint: false,
      owned: false,
      name: "UNFOUND SECRET",
      compatible: true,
    }),
  );
  h.loadout[0] = h.abilities[0];
  const original = copy(h);
  const ids = () =>
    flatten(f.target)
      .filter((node) => node.dataset.abilityId)
      .map((node) => node.dataset.abilityId);
  f.exports.paintForge(f.target, h, f.actions);
  f.find("browse-abilities").events.click();
  assert.match(allText(f.target), /Craft to collect/);
  assert.match(allText(f.target), /3 FOUND \/ 2 OWNED/);
  assert.match(allText(f.target), /BLUEPRINT \/ NOT CRAFTED/);
  assert.match(allText(f.target), /OWNED RANK 3 \/ ACTIVE RANK 1/);
  assert.match(allText(f.target), /DEFENSIVE \/ INCOMPATIBLE WITH ACTIVE HULL/);
  assert.deepEqual(ids(), ["ability-0", "ability-2", "ability-1"]);
  f.find("ability-filter-owned").events.click();
  assert.deepEqual(ids(), ["ability-0", "ability-1"]);
  assert.equal(
    f.find("ability-filter-owned").attributes["aria-pressed"],
    "true",
  );
  f.find("ability-filter-blueprint").events.click();
  assert.deepEqual(ids(), ["ability-2"]);
  f.find("ability-filter-compatible").events.click();
  assert.deepEqual(ids(), ["ability-0", "ability-2"]);
  f.find("ability-category-1").events.click();
  assert.deepEqual(ids(), []);
  assert.match(allText(f.target), /No found abilities match these filters/);
  assert.doesNotMatch(allText(f.target), /UNFOUND SECRET/);
  assert.deepEqual(h, original);
  assert.deepEqual(f.calls, []);
});

test("ability filters preserve focus and scroll across real HUD replacements without stealing outside focus", () => {
  const f = fixture(),
    h = hud();
  h.abilities[0] = ability(0, 0, { owned: true, rank: 1 });
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-0").events.click();
  f.find("ability-filter-owned").events.click();
  f.target.scrollTop = 87;
  f.find("ability-filter-owned").focus();
  const changed = copy(h);
  changed.abilities[0].rank = 2;
  f.exports.paintForge(f.target, changed, f.actions);
  assert.equal(f.target.scrollTop, 87);
  assert.equal(f.document.activeElement, f.find("ability-filter-owned"));
  assert.equal(f.find("ability-category-0").attributes["aria-pressed"], "true");
  assert.match(allText(f.target), /OWNED RANK 2/);
  const outside = {};
  f.document.activeElement = outside;
  f.exports.paintForge(f.target, changed, f.actions);
  assert.equal(f.document.activeElement, outside);
  f.find("back").events.click();
  assert.equal(f.document.activeElement, f.find("manage-0"));
});

test("successful crafting exposes Equip and retains action focus without automatically equipping", () => {
  const f = fixture(),
    h = hud();
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-0").events.click();
  f.find("craft-ability-0").focus();
  f.find("craft-ability-0").events.click();
  assert.deepEqual(f.calls, [["craft", "ability-0"]]);
  const changed = copy(h);
  Object.assign(changed.abilities[0], {
    owned: true,
    rank: 1,
    effective_rank: 1,
    can_equip: true,
    can_craft: false,
    equip_reason: "",
  });
  f.target.scrollTop = 44;
  f.exports.paintForge(f.target, changed, f.actions);
  assert.equal(f.target.scrollTop, 44);
  assert.equal(f.find("craft-ability-0"), undefined);
  assert.equal(f.document.activeElement, f.find("equip-ability-0"));
  assert.equal(changed.loadout[0], null);
  assert.deepEqual(f.calls, [["craft", "ability-0"]]);
});

test("hull filters keep found, owned and earned blueprint states distinct and preserve real recipe gates", () => {
  const f = fixture(),
    h = hud(),
    calls = [];
  const blueprint = {
    ...ship(2),
    name: "FOUND BLUEPRINT",
    owned: false,
    craftable: true,
    can_craft: false,
    craft_reason: "Need 6 banked T1 materials.",
  };
  const found = {
    ...ship(3),
    name: "FOUND ARENA HULL",
    owned: false,
    craftable: false,
    can_craft: false,
    craft_reason: "Defeat its champion first.",
  };
  h.campaign.ships.push(blueprint, found);
  h.material_cargo = [9, 0, 0, 0, 0];
  h.material_stash = [2, 0, 0, 0, 0];
  const original = copy(h);
  const actions = {
    craft: (id) => calls.push(["craft", id]),
    select: (id) => calls.push(["select", id]),
  };
  const ids = () =>
    flatten(f.target)
      .filter((node) => node.dataset.shipId)
      .map((node) => node.dataset.shipId);
  f.exports.paintShipHangar(f.target, h, actions);
  assert.match(allText(f.target), /Each RAICHO keeps its own XP and slots/);
  assert.match(allText(f.target), /4 FOUND \/ 2 OWNED/);
  assert.match(allText(f.target), /FOUND \/ BLUEPRINT REQUIRED/);
  assert.match(allText(f.target), /RENKO POINTS 10 \/ HAVE 3/);
  assert.match(allText(f.target), /T1 MATERIALS 6 \/ BANKED 2/);
  assert.doesNotMatch(
    allText(f.target),
    /CARGO:|CARRIED|DOCK STASH|MATERIALS LOST/,
  );
  f.find("hull-filter-owned").events.click();
  assert.deepEqual(ids(), ["0", "1"]);
  f.find("hull-filter-blueprint").events.click();
  assert.deepEqual(ids(), ["2"]);
  assert.match(allText(f.target), /BLUEPRINT \/ NOT CRAFTED/);
  assert.equal(f.find("craft-ship-2").disabled, true);
  f.find("craft-ship-2").events.click();
  assert.deepEqual(calls, []);
  const changed = copy(h);
  changed.campaign.ships[2].can_craft = true;
  f.exports.paintShipHangar(f.target, changed, actions);
  f.find("craft-ship-2").events.click();
  assert.deepEqual(calls, [["craft", 1]]);
  assert.deepEqual(h, original);
});

test("hull filter state persists through checkpoint repaint and does not retain a second inventory", () => {
  const f = fixture(),
    h = hud();
  const actions = { craft() {}, select() {} };
  f.exports.paintShipHangar(f.target, h, actions);
  f.find("hull-filter-blueprint").events.click();
  assert.match(allText(f.target), /No found hulls match this filter/);
  f.find("hull-filter-blueprint").focus();
  f.target.scrollTop = 63;
  const changed = copy(h);
  changed.campaign.ships.push({
    ...ship(4),
    owned: false,
    name: "NEWLY FOUND",
    craftable: true,
  });
  f.exports.paintShipHangar(f.target, changed, actions);
  assert.equal(f.target.scrollTop, 63);
  assert.equal(f.document.activeElement, f.find("hull-filter-blueprint"));
  assert.match(allText(f.target), /NEWLY FOUND/);
  assert.equal(
    f.find("hull-filter-blueprint").attributes["aria-pressed"],
    "true",
  );
  changed.campaign.ships.pop();
  f.exports.paintShipHangar(f.target, changed, actions);
  assert.doesNotMatch(allText(f.target), /NEWLY FOUND/);
});

test("collection costs and level requirements remain supplied by the HUD, including named recipes", () => {
  const f = fixture(),
    h = hud();
  Object.assign(h.abilities[1], {
    point_cost: 17,
    material_cost: 4,
    material_tier: 3,
    required_level: 10,
    can_craft: false,
    craft_reason: "Reach RAICHO level 10.",
  });
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-1").events.click();
  assert.match(allText(f.target), /REQUIRES RAICHO LV 10/);
  assert.match(allText(f.target), /HUGO POINTS 17 \/ HAVE 4/);
  assert.match(allText(f.target), /T3 MATERIALS 4 \/ BANKED 0/);
  assert.equal(f.find("craft-ability-1").disabled, true);
});

test("in-place collection filters and categories retain the visible scroll position", () => {
  const f = fixture(),
    h = hud();
  f.exports.paintForge(f.target, h, f.actions);
  f.find("browse-abilities").events.click();
  f.target.scrollTop = 123;
  f.find("ability-filter-owned").events.click();
  assert.equal(f.target.scrollTop, 123);
  assert.equal(f.document.activeElement, f.find("ability-filter-owned"));
  f.find("ability-category-2").events.click();
  assert.equal(f.target.scrollTop, 123);
  assert.equal(f.document.activeElement, f.find("ability-category-2"));
  f.exports.paintShipHangar(f.target, h, { craft() {}, select() {} });
  f.target.scrollTop = 63;
  f.find("hull-filter-owned").events.click();
  assert.equal(f.target.scrollTop, 63);
  assert.equal(f.document.activeElement, f.find("hull-filter-owned"));
});

test("native collection hierarchy uses real equipped flags and keeps point issuer separate from hull maker", () => {
  const f = fixture(),
    h = hud();
  h.abilities[2] = ability(2, 2, {
    owned: true,
    rank: 5,
    effective_rank: 2,
    equipped: true,
  });
  h.loadout[2] = h.abilities[2];
  const before = copy(h);
  f.exports.paintForge(f.target, h, f.actions);
  const issuers = flatten(f.target).filter(
    (n) => n.dataset.pixelKind === "corporation-symbol",
  );
  assert.deepEqual(
    issuers.slice(0, 3).map((n) => n.dataset.pixelValue),
    ["1", "2", "0"],
  );
  f.find("manage-2").events.click();
  const cards = flatten(f.target).filter((n) => n.dataset.abilityId);
  assert.equal(cards.length, 1);
  assert.equal(cards[0].dataset.pixelKind, "collection-entry");
  assert.equal(cards[0].dataset.active, "true");
  const state = flatten(cards[0]).find(
    (n) => n.dataset.pixelKind === "collection-state",
  );
  assert.equal(state.dataset.pixelValue, "3");
  assert.match(state.textContent, /OWNED RANK 5 \/ ACTIVE RANK 2 \/ EQUIPPED/);
  f.exports.paintShipHangar(f.target, h, { craft() {}, select() {} });
  const hulls = flatten(f.target).filter((n) => n.dataset.shipId);
  assert.equal(hulls[0].dataset.active, "true");
  assert.equal(hulls[1].dataset.active, "false");
  assert.ok(hulls.every((n) => n.dataset.pixelValue === "-1"));
  assert.equal(
    flatten(f.target).filter(
      (n) => n.dataset.pixelKind === "corporation-symbol",
    ).length,
    0,
  );
  assert.deepEqual(h, before);
});
