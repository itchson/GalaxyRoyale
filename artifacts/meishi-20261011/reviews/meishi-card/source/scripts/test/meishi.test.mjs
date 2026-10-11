import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";

const source = readFileSync(
  new URL("../../apps/game/src/meishi.ts", import.meta.url),
  "utf8",
);
const compile = (source) =>
  ts.transpileModule(source, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  }).outputText;
const compiled = compile(source.replace('import "./meishi.css";', ""));
const portraitCompiled = compile(
  readFileSync(
    new URL("../../apps/game/src/kikai-portrait.ts", import.meta.url),
    "utf8",
  ),
);

// Actual TS view/helper executed against a DOM adapter. No browser/raster claims.
function fixture() {
  let writes = 0;
  const document = { activeElement: null };
  class Element {
    children = [];
    attributes = {};
    className = "";
    scrollTop = 0;
    value = "";
    _hidden = false;
    dataset = new Proxy(
      {},
      {
        set(target, key, value) {
          writes++;
          target[key] = value;
          return true;
        },
      },
    );
    get hidden() {
      return this._hidden;
    }
    set hidden(value) {
      writes++;
      this._hidden = value;
    }
    get textContent() {
      return this.value + this.children.map((n) => n.textContent).join("");
    }
    set textContent(value) {
      writes++;
      this.value = value;
      this.children = [];
    }
    append(...children) {
      writes++;
      this.children.push(...children);
    }
    replaceChildren(...children) {
      writes++;
      this.children = children;
    }
    setAttribute(key, value) {
      writes++;
      this.attributes[key] = value;
    }
    getAttribute(key) {
      return this.attributes[key] ?? null;
    }
    focus() {
      document.activeElement = this;
    }
  }
  document.createElement = (tagName) =>
    Object.assign(new Element(), { tagName });
  const helper = {};
  vm.runInNewContext(portraitCompiled, { exports: helper });
  const exports = {};
  vm.runInNewContext(compiled, {
    exports,
    document,
    require: (name) => {
      assert.equal(name, "./kikai-portrait");
      return helper;
    },
  });
  return {
    paint: exports.paintMeishiCard,
    target: new Element(),
    external: new Element(),
    document,
    resetWrites: () => {
      writes = 0;
    },
    writes: () => writes,
  };
}
function all(node) {
  return [node, ...node.children.flatMap(all)];
}
function byClass(f, name) {
  return all(f.target).filter((n) => n.className === name);
}
function visibleText(node) {
  return node.hidden || node.getAttribute("aria-hidden") === "true"
    ? ""
    : node.value + node.children.map(visibleText).join(" ");
}
function freeze(value) {
  if (value && typeof value === "object") {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}
function hud() {
  return {
    identity: {
      kind: "dora",
      corporation: "pan",
      sub_corporation: "renko",
      short_designation: "APR#0123456789ABCDEF",
      designation: "DORA-PAN-RENKO-0123456789ABCDEF",
    },
    campaign: {
      selected_ship: 0,
      ships: [
        { id: 0, name: "PAN HIKARI" },
        { id: 9, name: "ROYALE SOVEREIGN" },
      ],
      champions: Array.from({ length: 8 }, (_, index) => ({
        index,
        name: `SECRET CHAMPION ${index}`,
        patch: false,
      })),
    },
    ship_level: 1,
    ship_xp: 7,
    ship_next_xp: 20,
    hull: 73,
    max_hull: 100,
    shield: 0,
    max_shield: 0,
    loadout: [
      null,
      null,
      {
        id: "flux-drive",
        index: 2,
        category: 2,
        name: "FLUX DRIVE",
        rank: 5,
        effective_rank: 1,
      },
    ],
    currency: [13, 29, 47],
  };
}

test("card uses authoritative native profile and exactly eight anonymous empty patch slots", () => {
  const f = fixture();
  assert.equal(f.paint(f.target, freeze(hud())), true);
  const portrait = byClass(f, "meishi-portrait")[0];
  assert.equal(portrait.hidden, false);
  assert.equal(portrait.dataset.pixelValue, "9");
  assert.equal(portrait.dataset.portraitKind, "0");
  assert.equal(portrait.dataset.portraitMain, "0");
  assert.equal(portrait.dataset.portraitSub, "1");
  assert.equal(portrait.dataset.portraitSpeaking, "false");
  assert.match(portrait.getAttribute("aria-label"), /GREEN hair, RED crystals/);
  assert.equal(byClass(f, "meishi-patch-slot").length, 8);
  assert.equal(
    byClass(f, "meishi-patch-icon").filter((n) => !n.hidden).length,
    0,
  );
  assert.match(visibleText(f.target), /MEISHI CARD/);
  assert.doesNotMatch(JSON.stringify(f.target), /SECRET CHAMPION/);
});

test("unchanged refresh preserves nodes, scroll, focus and writes nothing", () => {
  const f = fixture();
  const data = freeze(hud());
  f.paint(f.target, data);
  const before = all(f.target);
  f.target.scrollTop = 91;
  f.external.focus();
  f.resetWrites();
  assert.equal(f.paint(f.target, data), false);
  assert.equal(f.writes(), 0);
  assert.equal(f.target.scrollTop, 91);
  assert.equal(f.document.activeElement, f.external);
  all(f.target).forEach((node, index) => assert.equal(node, before[index]));
});

test("patches are indexed rather than list ordered and only earned identities reach DOM", () => {
  const f = fixture();
  const data = hud();
  data.campaign.champions.reverse();
  data.campaign.champions.find((c) => c.index === 6).patch = true;
  data.campaign.champions.push({ index: 8, name: "ROYAL NINTH", patch: true });
  f.paint(f.target, data);
  const slots = byClass(f, "meishi-patch-slot");
  assert.match(
    slots[6].getAttribute("aria-label"),
    /SECRET CHAMPION 6, earned/,
  );
  assert.equal(slots[6].children[0].dataset.pixelValue, "6");
  assert.equal(slots[6].children[0].dataset.active, "true");
  assert.equal(slots[6].children[0].hidden, false);
  assert.equal(slots[6].children[1].hidden, true);
  assert.equal(
    byClass(f, "meishi-patch-count")[0].textContent,
    "ARENA PATCHES 1/8",
  );
  assert.doesNotMatch(
    JSON.stringify(f.target),
    /SECRET CHAMPION [0-5]|SECRET CHAMPION 7|ROYAL NINTH/,
  );
});

test("an unearned champion name is never read even when source includes private metadata", () => {
  const f = fixture();
  const data = hud();
  data.campaign.champions.forEach((champion) => {
    Object.defineProperty(champion, "name", {
      get() {
        throw new Error("spoiler read");
      },
    });
  });
  assert.doesNotThrow(() => f.paint(f.target, data));
});

test("losing an earned flag clears visible art and its former name in the same stable slot", () => {
  const f = fixture();
  const data = hud();
  data.campaign.champions[2].patch = true;
  f.paint(f.target, data);
  const slot = byClass(f, "meishi-patch-slot")[2];
  data.campaign.champions[2].patch = false;
  assert.equal(f.paint(f.target, data), true);
  assert.equal(byClass(f, "meishi-patch-slot")[2], slot);
  assert.equal(slot.children[0].hidden, true);
  assert.equal(slot.children[1].hidden, false);
  assert.equal(slot.getAttribute("aria-label"), "Arena patch 3: empty");
  assert.doesNotMatch(JSON.stringify(f.target), /SECRET CHAMPION 2/);
});

test("all eight earned flags use existing campaign-patch routes without a ninth slot", () => {
  const f = fixture();
  const data = hud();
  data.campaign.champions.forEach((champion) => {
    champion.patch = true;
  });
  f.paint(f.target, freeze(data));
  byClass(f, "meishi-patch-icon").forEach((node, index) => {
    assert.equal(node.dataset.pixelKind, "campaign-patch");
    assert.equal(node.dataset.pixelValue, String(index));
    assert.equal(node.dataset.active, "true");
    assert.equal(node.hidden, false);
  });
  assert.equal(
    byClass(f, "meishi-patch-count")[0].textContent,
    "ARENA PATCHES 8/8",
  );
});

test("corporate balances follow offense/defense/passive ledger order regardless of pilot manufacturer", () => {
  const f = fixture();
  const data = hud();
  data.identity.corporation = "hugo";
  f.paint(f.target, data);
  const values = byClass(f, "meishi-points").map((node) => node.textContent);
  assert.deepEqual(values, [
    "RENKO / OFFENSIVE 13",
    "HUGO / DEFENSIVE 29",
    "PAN / PASSIVE 47",
  ]);
  data.currency = [2, 3, 5];
  assert.equal(f.paint(f.target, data), true);
  assert.equal(byClass(f, "meishi-points")[2].textContent, "PAN / PASSIVE 5");
});

test("selected RAICHO and effective equipped ranks refresh without inherited-rank or XP fabrication", () => {
  const f = fixture();
  const data = hud();
  f.paint(f.target, data);
  const icon = byClass(f, "meishi-ship-icon")[0];
  assert.equal(icon.dataset.pixelKind, "licensed-ship");
  assert.equal(icon.dataset.pixelValue, "0");
  assert.match(byClass(f, "meishi-slot")[2].textContent, /R1 EFFECTIVE/);
  assert.doesNotMatch(visibleText(f.target), /R5/);
  data.campaign.selected_ship = 9;
  data.ship_xp = 0;
  data.hull = 21;
  f.paint(f.target, data);
  assert.equal(byClass(f, "meishi-ship-icon")[0], icon);
  assert.equal(icon.dataset.pixelKind, "campaign-ship");
  assert.equal(icon.dataset.pixelValue, "8");
  assert.equal(
    byClass(f, "meishi-ship-name")[0].textContent,
    "ROYALE SOVEREIGN",
  );
  assert.match(byClass(f, "meishi-progress")[0].textContent, /LV 1 \/ XP 0/);
  assert.match(byClass(f, "meishi-vitals")[0].textContent, /HULL 21\/100/);
  data.loadout[2] = null;
  f.paint(f.target, data);
  assert.equal(byClass(f, "meishi-slot")[2].textContent, "PASSIVE / EMPTY");
});

test("legacy and malformed identity do not relabel another portrait; invalid selected hull paints no art", () => {
  const f = fixture();
  const data = hud();
  f.paint(f.target, data);
  data.identity = null;
  f.paint(f.target, data);
  assert.equal(byClass(f, "meishi-portrait")[0].hidden, true);
  assert.match(visibleText(f.target), /LEGACY PILOT/);
  assert.doesNotMatch(visibleText(f.target), /APR#|DORA-PAN/);
  assert.equal(byClass(f, "meishi-ship-icon")[0].dataset.pixelValue, "9");
  data.identity = { ...hud().identity, kind: "other" };
  data.campaign.selected_ship = 256;
  f.paint(f.target, data);
  assert.equal(byClass(f, "meishi-portrait")[0].hidden, true);
  assert.equal(byClass(f, "meishi-ship-icon")[0].hidden, true);
  assert.equal(
    byClass(f, "meishi-ship-name")[0].textContent,
    "RAICHO UNAVAILABLE",
  );
});

test("nonfinite HUD numbers are displayed as unavailable, not positive progression", () => {
  const f = fixture();
  const data = hud();
  data.hull = Infinity;
  data.ship_xp = NaN;
  data.currency = [-1, NaN, Infinity];
  f.paint(f.target, data);
  assert.match(byClass(f, "meishi-vitals")[0].textContent, /HULL --\/100/);
  assert.match(byClass(f, "meishi-progress")[0].textContent, /XP --/);
  byClass(f, "meishi-points").forEach((node) =>
    assert.match(node.textContent, / --$/),
  );
});

test("view source stays read-only and responsive CSS retains native logical patch/ship dimensions", () => {
  const css = readFileSync(
    new URL("../../apps/game/src/meishi.css", import.meta.url),
    "utf8",
  );
  assert.doesNotMatch(
    source,
    /localStorage|indexedDB|setInterval|requestAnimationFrame|innerHTML|\.style\./,
  );
  assert.match(css, /width: calc\(24px \* var\(--ui-scale, 1\)\)/);
  assert.match(css, /width: calc\(32px \* var\(--ui-scale, 1\)\)/);
  assert.match(css, /@media \(max-width: 900px\)/);
  assert.match(css, /@media \(max-width: 480px\)/);
  assert.match(css, /\.meishi-card \[hidden\]\s*\{\s*display: none;/);
  assert.doesNotMatch(css, /overflow:\s*hidden|height:\s*100vh|animation:/);
  // These are source policy checks, not computed BCR or GPU layout acceptance.
});
