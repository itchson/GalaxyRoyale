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
  assert.match(allText(f.target), /SHIP LV 3 \/ 60 XP/);
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
  assert.match(allText(f.target), /PASSIVE \/ 50 PTS/);
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
test("busy mutation and outside-dock clearing cannot dispatch; loss uses real tier disclosure", () => {
  const f = fixture();
  const h = hud();
  h.dock = false;
  h.material_loss = [1, 0, 2, 0, 0];
  h.material_loss_reason = "reload";
  h.loadout[0] = ability(0, 0, { owned: true, rank: 1, effective_rank: 1 });
  f.exports.paintForge(f.target, h, { ...f.actions, busy: true });
  assert.match(
    allText(f.target),
    /MATERIALS LOST \/ RELOAD \/ T1 1 \/ T2 0 \/ T3 2/,
  );
  f.find("manage-0").events.click();
  assert.equal(f.find("empty-0").disabled, true);
  assert.equal(f.find("craft-ability-0").disabled, true);
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
  assert.match(allText(f.target), /SHIP LV 3 \/ 60 XP/);
  assert.match(allText(f.target), /SHIP LV 1 \/ 0 XP/);
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
  assert.equal(f.find("craft-ship-9").textContent, "LOCKED SHIP");
  assert.ok(!allText(f.target).includes("POINTS 0 / HAVE"));
  assert.ok(!allText(f.target).includes("MATERIALS 0 / BANKED"));
  royal.owned = true;
  royal.can_select = true;
  f.exports.paintShipHangar(f.target, h, { select() {}, craft() {} });
  assert.match(allText(f.target), /SHIP LV 1 \/ 0 XP \/ NEXT 20 XP/);
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

test("Forge prioritizes all three slot controls while retaining cargo guidance and loss details", () => {
  const f = fixture();
  const h = hud();
  h.material_loss = [2, 0, 3, 0, 0];
  h.material_loss_reason = "reload";
  f.exports.paintForge(f.target, h, f.actions);
  const nodes = flatten(f.target);
  const guidance = nodes.findIndex((node) =>
    node.textContent.startsWith("CARGO:"),
  );
  assert.ok(guidance > 0);
  for (const category of [0, 1, 2]) {
    const control = f.find(`manage-${category}`);
    assert.ok(nodes.indexOf(control) < guidance);
    assert.equal(control.disabled, false);
  }
  assert.match(
    allText(f.target),
    /Bank by arriving at Haven\. Lost on death, restart, title or reload/,
  );
  assert.match(
    allText(f.target),
    /MATERIALS LOST \/ RELOAD \/ T1 2 \/ T2 0 \/ T3 3/,
  );
  assert.match(allText(f.target), /DOCK STASH \/ T1 10 \/ T2 2/);
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
