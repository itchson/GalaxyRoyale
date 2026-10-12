import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";
const compiled = ts.transpileModule(
  readFileSync(
    new URL("../../apps/game/src/hangar-scene.ts", import.meta.url),
    "utf8",
  ),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  },
).outputText;

test("legacy unregistered Scout uses its actual art before and after a licensed identity transition", () => {
  const f = fixture(),
    hud = makeHud({ identity: null });
  hud.campaign.ships[0].name = "HAVEN SCOUT";
  f.paint(hud);
  const hero = f.class("hangar-showcase-art"),
    row = f.node("hangar-hull-art", f.hull(0));
  for (const art of [hero, row]) {
    assert.equal(art.dataset.pixelKind, "campaign-ship");
    assert.equal(art.dataset.pixelValue, "9");
  }
  hud.identity = { corporation: "hugo" };
  f.paint(hud);
  assert.equal(f.class("hangar-showcase-art"), hero);
  for (const art of [hero, row]) {
    assert.equal(art.dataset.pixelKind, "licensed-ship");
    assert.equal(art.dataset.pixelValue, "2");
  }
});

// Actual semantic module with a declared DOM stand-in. No native pixels,
// screenshots, gameplay selection, GPU layout or physical browser are claimed.
function fixture() {
  let writes = 0,
    created = 0;
  const document = { activeElement: null };
  class Element {
    children = [];
    parentElement = null;
    attributes = {};
    listeners = {};
    _text = "";
    _hidden = false;
    _disabled = false;
    _scrollLeft = 0;
    className = "";
    id = "";
    focused = [];
    scrolled = [];
    _dataset = {};
    dataset = new Proxy(this._dataset, {
      set: (object, key, value) => {
        writes++;
        object[key] = value;
        this.attributes[
          "data-" + key.replace(/[A-Z]/g, (c) => "-" + c.toLowerCase())
        ] = value;
        return true;
      },
    });
    get textContent() {
      return (
        this._text + this.children.map((child) => child.textContent).join(" ")
      );
    }
    set textContent(value) {
      writes++;
      this._text = value;
      this.replaceChildren();
    }
    get hidden() {
      return this._hidden;
    }
    set hidden(value) {
      writes++;
      this._hidden = value;
    }
    get disabled() {
      return this._disabled;
    }
    set disabled(value) {
      writes++;
      this._disabled = value;
    }
    get scrollLeft() {
      return this._scrollLeft;
    }
    set scrollLeft(value) {
      writes++;
      this._scrollLeft = value;
    }
    append(...children) {
      writes++;
      for (const child of children) {
        this.children.push(child);
        child.parentElement = this;
      }
    }
    replaceChildren(...children) {
      writes++;
      if (this.contains(document.activeElement)) document.activeElement = null;
      for (const child of this.children) child.parentElement = null;
      this.children = [];
      this.append(...children);
    }
    contains(target) {
      return (
        this === target || this.children.some((child) => child.contains(target))
      );
    }
    setAttribute(key, value) {
      writes++;
      this.attributes[key] = value;
      if (key.startsWith("data-"))
        this._dataset[
          key.slice(5).replace(/-([a-z])/g, (_, c) => c.toUpperCase())
        ] = value;
    }
    getAttribute(key) {
      return this.attributes[key] ?? null;
    }
    addEventListener(name, listener) {
      (this.listeners[name] ??= []).push(listener);
    }
    dispatch(name, fields = {}) {
      const event = {
        target: this,
        key: "",
        isComposing: false,
        altKey: false,
        ctrlKey: false,
        metaKey: false,
        prevented: false,
        preventDefault() {
          this.prevented = true;
        },
        ...fields,
      };
      for (const listener of this.listeners[name] ?? []) listener(event);
      return event;
    }
    click() {
      if (!this.disabled) this.dispatch("click");
    }
    focus(options) {
      document.activeElement = this;
      this.focused.push(options);
    }
    scrollIntoView(options) {
      this.scrolled.push(options);
    }
  }
  document.createElement = (tagName) => {
    created++;
    return Object.assign(new Element(), { tagName });
  };
  const exports = {};
  vm.runInNewContext(compiled, {
    exports,
    document,
    require: (name) => {
      assert.equal(name, "./hangar-scene.css");
      return {};
    },
  });
  const target = new Element(),
    calls = [];
  const actions = {
    select: (id) => calls.push(["select", id]),
    details: () => calls.push(["details"]),
  };
  const nodes = (node = target) => [
    node,
    ...node.children.flatMap((child) => nodes(child)),
  ];
  return {
    target,
    document,
    actions,
    calls,
    paint: (hud = makeHud(), chosenActions = actions) =>
      exports.paintHangarScene(target, hud, chosenActions),
    class: (name) =>
      nodes().find((node) => node.className.split(" ").includes(name)),
    hull: (id) => nodes().find((node) => node.id === `hangar-hull-${id}`),
    node: (name, root) => nodes(root).find((node) => node.className === name),
    reset: () => {
      writes = 0;
    },
    writes: () => writes,
    created: () => created,
  };
}
const ship = (id, patch = {}) => ({
  id,
  name: id === 0 ? "PAN HIKARI" : `RAICHO ${id}`,
  owned: true,
  selected: id === 0,
  level: 1,
  xp: 0,
  can_select: id !== 0,
  ...patch,
});
const makeHud = (patch = {}) => ({
  identity: { corporation: "pan" },
  dock: true,
  currency: [500, 600, 700],
  material_stash: [99],
  material_cargo: [99],
  material_loss: [],
  material_loss_reason: "",
  campaign: {
    selected_ship: 0,
    ships: [
      ship(0),
      ship(1),
      ship(2, { name: "UNDISCOVERED SECRET", owned: false }),
    ],
  },
  ...patch,
});

test("only owned HUD hulls appear; no blueprint, currency, cargo or ability inventory leaks into the garage", () => {
  const f = fixture();
  f.paint();
  assert.ok(f.hull(0));
  assert.ok(f.hull(1));
  assert.equal(f.hull(2), undefined);
  assert.equal(f.target.textContent.includes("UNDISCOVERED SECRET"), false);
  assert.doesNotMatch(
    f.target.textContent,
    /CRAFT|MATERIAL|CARGO|500|600|700|ABILITY|OFFENSIVE|DEFENSIVE|PASSIVE/,
  );
});
test("all three starter corporations use licensed art and matching native garage palette", () => {
  for (const [corporation, value] of ["pan", "renko", "hugo"].map(
    (corporation, value) => [corporation, value],
  )) {
    const f = fixture();
    f.paint(makeHud({ identity: { corporation } }));
    const hero = f.class("hangar-showcase"),
      art = f.class("hangar-showcase-art");
    assert.equal(hero.dataset.pixelKind, "garage-bay");
    assert.equal(hero.dataset.pixelValue, String(value));
    assert.equal(art.dataset.pixelKind, "licensed-ship");
    assert.equal(art.dataset.pixelValue, String(value));
    assert.equal(
      f.node("hangar-hull-art", f.hull(0)).dataset.pixelValue,
      String(value),
    );
  }
});
test("Sovereign is campaign art index8 and uses the actual selected HUD, not row order", () => {
  const f = fixture();
  f.paint(
    makeHud({
      campaign: {
        selected_ship: 9,
        ships: [
          ship(4),
          ship(9, { name: "SOVEREIGN", selected: true, level: 19, xp: 417 }),
          ship(0),
        ],
      },
    }),
  );
  const art = f.class("hangar-showcase-art");
  assert.equal(art.dataset.pixelKind, "campaign-ship");
  assert.equal(art.dataset.pixelValue, "8");
  assert.equal(f.class("hangar-active-name").textContent, "SOVEREIGN");
  assert.equal(f.class("hangar-active-progress").textContent, "LV 19 / 417 XP");
  assert.equal(f.hull(9).getAttribute("aria-pressed"), "true");
});
test("clicking a selectable hull calls only the existing native select action, with no optimistic showcase mutation", () => {
  const f = fixture();
  const hud = makeHud();
  f.paint(hud);
  f.hull(0).click();
  f.hull(1).click();
  assert.deepEqual(f.calls, [["select", 1]]);
  assert.equal(f.class("hangar-active-name").textContent, "PAN HIKARI");
  assert.equal(f.hull(1).getAttribute("aria-pressed"), "false");
  const row = f.hull(1);
  row.focus();
  f.class("hangar-index").scrollLeft = 93;
  hud.campaign.selected_ship = 1;
  f.paint(hud);
  assert.equal(f.hull(1), row);
  assert.equal(f.document.activeElement, row);
  assert.equal(f.class("hangar-index").scrollLeft, 93);
  assert.equal(f.class("hangar-active-name").textContent, "RAICHO 1");
  assert.equal(f.hull(1).getAttribute("aria-pressed"), "true");
});
test("away from Haven, busy or native can_select=false cannot mutate a hull", () => {
  for (const [hud, actionsPatch] of [
    [makeHud({ dock: false }), {}],
    [makeHud(), { busy: true }],
    [
      makeHud({
        campaign: {
          selected_ship: 0,
          ships: [ship(0), ship(1, { can_select: false })],
        },
      }),
      {},
    ],
  ]) {
    const f = fixture();
    f.paint(hud, { ...f.actions, ...actionsPatch });
    assert.equal(f.hull(1).disabled, true);
    f.hull(1).dispatch("click");
    assert.deepEqual(f.calls, []);
  }
});
test("showcase details stay available away from Haven and during a selection save", () => {
  const f = fixture();
  f.paint(makeHud({ dock: false }), { ...f.actions, busy: true });
  assert.equal(f.class("hangar-showcase").disabled, false);
  f.class("hangar-showcase").click();
  assert.deepEqual(f.calls, [["details"]]);
});
test("unchanged relevant HUD retains nodes/focus/scroll and produces no DOM writes during repeated world updates", () => {
  const f = fixture();
  const hud = makeHud();
  f.paint(hud);
  const row = f.hull(1),
    index = f.class("hangar-index");
  row.focus();
  index.scrollLeft = 121;
  const created = f.created();
  f.reset();
  for (let tick = 0; tick < 80; tick++) {
    hud.material_stash = [tick];
    hud.position = [tick, -tick];
    hud.camera = [tick / 2, -tick];
    hud.campaign.ships[2].name = `SECRET ${tick}`;
    hud.currency = [tick, tick, tick];
    assert.equal(f.paint(hud), false);
  }
  assert.equal(f.writes(), 0);
  assert.equal(f.created(), created);
  assert.equal(f.hull(1), row);
  assert.equal(f.document.activeElement, row);
  assert.equal(index.scrollLeft, 121);
});
test("hull XP changes update their own summary while preserving the active scene and index identities", () => {
  const f = fixture();
  const hud = makeHud();
  f.paint(hud);
  const row = f.hull(1),
    hero = f.class("hangar-showcase"),
    index = f.class("hangar-index");
  row.focus();
  index.scrollLeft = 57;
  hud.campaign.ships[1].xp = 41;
  hud.campaign.ships[1].level = 3;
  assert.equal(f.paint(hud), true);
  assert.equal(f.node("hangar-hull-progress", row).textContent, "LV 3 / 41 XP");
  assert.equal(f.class("hangar-active-progress").textContent, "LV 1 / 0 XP");
  assert.equal(f.class("hangar-showcase"), hero);
  assert.equal(f.hull(1), row);
  assert.equal(f.document.activeElement, row);
  assert.equal(index.scrollLeft, 57);
});
test("owned membership/order changes preserve surviving focus and scroll; stale removed tiles cannot act", () => {
  const f = fixture();
  const hud = makeHud();
  f.paint(hud);
  const row = f.hull(1),
    index = f.class("hangar-index");
  row.focus();
  index.scrollLeft = 37;
  hud.campaign.ships = [ship(3), ship(1), ship(0)];
  f.paint(hud);
  assert.equal(f.hull(1), row);
  assert.equal(f.document.activeElement, row);
  assert.equal(index.scrollLeft, 37);
  assert.equal(row.focused.at(-1).preventScroll, true);
  hud.campaign.ships = [ship(0)];
  f.paint(hud);
  row.dispatch("click");
  assert.equal(f.hull(1), undefined);
  assert.deepEqual(f.calls, []);
});
test("keyboard horizontal index navigation skips unavailable hulls and never selects a ship", () => {
  const f = fixture();
  f.paint(
    makeHud({
      campaign: {
        selected_ship: 0,
        ships: [ship(0), ship(1, { can_select: false }), ship(3), ship(9)],
      },
    }),
  );
  const index = f.class("hangar-index");
  f.hull(0).focus();
  assert.equal(
    index.dispatch("keydown", { key: "ArrowRight" }).prevented,
    true,
  );
  assert.equal(f.document.activeElement, f.hull(3));
  index.dispatch("keydown", { key: "End" });
  assert.equal(f.document.activeElement, f.hull(9));
  index.dispatch("keydown", { key: "Home" });
  assert.equal(f.document.activeElement, f.hull(0));
  index.dispatch("keydown", { key: "ArrowRight", isComposing: true });
  assert.equal(f.document.activeElement, f.hull(0));
  assert.deepEqual(f.calls, []);
  assert.equal(f.hull(3).scrolled[0].inline, "nearest");
});
test("unknown selected hull or invalid native hull data cannot fabricate an active showcase", () => {
  for (const campaign of [
    { selected_ship: 2, ships: [ship(0), ship(2, { owned: false })] },
    { selected_ship: 1, ships: [ship(1, { xp: Infinity })] },
    { selected_ship: 0, ships: [] },
  ]) {
    const f = fixture();
    f.paint(makeHud({ campaign }));
    assert.equal(f.class("hangar-showcase").disabled, true);
    assert.equal(f.class("hangar-showcase-art").hidden, true);
    f.class("hangar-showcase").dispatch("click");
    assert.deepEqual(f.calls, []);
    assert.equal(f.class("hangar-active-name").textContent, "NO ACTIVE RAICHO");
  }
});
test("fresh callback ownership is respected even when the relevant HUD did not change", () => {
  const f = fixture();
  const hud = makeHud();
  f.paint(hud);
  assert.equal(
    f.paint(hud, {
      select: (id) => f.calls.push(["new-select", id]),
      details: () => f.calls.push(["new-details"]),
    }),
    false,
  );
  f.hull(1).click();
  f.class("hangar-showcase").click();
  assert.deepEqual(f.calls, [["new-select", 1], ["new-details"]]);
});
test("remounting a cleared target builds a fresh scene and detached old controls cannot act", () => {
  const f = fixture();
  f.paint();
  const hero = f.class("hangar-showcase"),
    row = f.hull(1);
  f.target.replaceChildren();
  assert.equal(f.paint(), true);
  assert.notEqual(f.class("hangar-showcase"), hero);
  hero.dispatch("click");
  row.dispatch("click");
  assert.deepEqual(f.calls, []);
});
