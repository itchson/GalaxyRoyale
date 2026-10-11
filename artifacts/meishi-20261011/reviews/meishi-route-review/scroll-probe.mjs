import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

// Actual view code, fake DOM only: not Rust authority, browser layout or saves.
const source = readFileSync(
  new URL("./inputs/apps/game/src/loadout.ts", import.meta.url),
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


// Actual painter/Root scroll change; fake DOM with declared scroll clamping.
// This is not browser BCR, layout, rendering or native input evidence.
function boundedScroll(f) {
  let offset = 0, max = 200;
  Object.defineProperty(f.target, "scrollTop", {
    configurable: true,
    get: () => offset,
    set: value => { offset = Math.max(0, Math.min(max, value)); },
  });
  const replace = f.target.replaceChildren.bind(f.target);
  f.target.replaceChildren = (...nodes) => { replace(...nodes); offset = 0; };
  return value => { max = value; offset = Math.min(offset, max); };
}
const results = [];
{
  const f = fixture(), h = hud(); boundedScroll(f);
  const before = copy(h);
  f.exports.paintForge(f.target, h, f.actions);
  f.find("browse-abilities").events.click();
  f.target.scrollTop = 63;
  f.find("ability-filter-blueprint").events.click();
  assert.equal(f.target.scrollTop, 63);
  assert.equal(f.document.activeElement, f.find("ability-filter-blueprint"));
  assert.deepEqual(h, before);
  results.push("ability filter retains prior scroll/focus");
}
{
  const f = fixture(), h = hud(); boundedScroll(f);
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-0").events.click();
  f.target.scrollTop = 71;
  f.find("ability-category-2").events.click();
  assert.equal(f.target.scrollTop, 71);
  assert.equal(f.document.activeElement, f.find("ability-category-2"));
  f.find("back").events.click();
  assert.equal(f.target.scrollTop, 0);
  assert.equal(f.document.activeElement, f.find("manage-2"));
  results.push("same collection category retains scroll; Back resets overview");
}
{
  const f = fixture(), h = hud(); const max = boundedScroll(f);
  f.exports.paintShipHangar(f.target, h, { craft() {}, select() {} });
  f.target.scrollTop = 89;
  f.find("hull-filter-owned").events.click();
  assert.equal(f.target.scrollTop, 89);
  assert.equal(f.document.activeElement, f.find("hull-filter-owned"));
  max(20);
  f.find("hull-filter-blueprint").events.click();
  assert.equal(f.target.scrollTop, 20);
  assert.equal(f.document.activeElement, f.find("hull-filter-blueprint"));
  results.push("hull filters retain legal scroll and accept clamped shorter content");
}
console.log(JSON.stringify({ passed: results.length, results, scope: "Actual Root loadout.ts with declared fake DOM scroll bounds, no browser/render proof" }, null, 2));
