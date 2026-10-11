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
  assert.equal(byClass(f, "meishi-patch-icon")[6].dataset.pixelValue, "6");
  assert.equal(byClass(f, "meishi-patch-icon")[6].dataset.active, "true");
  assert.equal(byClass(f, "meishi-patch-icon")[6].hidden, false);
  assert.equal(byClass(f, "meishi-patch-empty")[6].hidden, true);
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
  assert.equal(byClass(f, "meishi-patch-icon")[2].hidden, true);
  assert.equal(byClass(f, "meishi-patch-empty")[2].hidden, false);
  assert.equal(byClass(f, "meishi-patch-well")[2].dataset.active, "false");
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
  assert.deepEqual(values, ["RENKO / 13", "HUGO / 29", "PAN / 47"]);
  data.currency = [2, 3, 5];
  assert.equal(f.paint(f.target, data), true);
  assert.equal(byClass(f, "meishi-points")[2].textContent, "PAN / 5");
  assert.equal(
    byClass(f, "meishi-points")[2].getAttribute("aria-label"),
    "PAN / PASSIVE POINTS 5",
  );
});

test("compact ship strip retains the selected RAICHO icon, name and authoritative level", () => {
  const f = fixture();
  const data = hud();
  f.paint(f.target, data);
  const icon = byClass(f, "meishi-ship-icon")[0];
  assert.equal(icon.dataset.pixelKind, "licensed-ship");
  assert.equal(icon.dataset.pixelValue, "0");
  assert.equal(byClass(f, "meishi-ship-name")[0].textContent, "PAN HIKARI");
  const progress = byClass(f, "meishi-progress")[0];
  assert.equal(progress.textContent, "LV 1");
  assert.equal(progress.getAttribute("aria-label"), "RAICHO level 1");
  assert.equal(
    byClass(f, "meishi-raicho")[0].getAttribute("aria-label"),
    "Current RAICHO",
  );
  for (const removed of ["meishi-vitals", "meishi-slot", "meishi-equipped"])
    assert.equal(byClass(f, removed).length, 0);
  assert.doesNotMatch(visibleText(f.target), /XP|HULL|SHIELD|FLUX DRIVE|R5/);
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
  assert.equal(byClass(f, "meishi-progress")[0], progress);
  assert.equal(progress.textContent, "LV 1");
  assert.equal(progress.getAttribute("aria-label"), "RAICHO level 1");
  data.ship_level = 100;
  f.paint(f.target, data);
  assert.equal(progress.textContent, "LV 100");
  assert.equal(progress.getAttribute("aria-label"), "RAICHO level 100");
  assert.doesNotMatch(visibleText(f.target), /XP|NEXT|HULL|SHIELD|FLUX DRIVE/);
});

test("card never reads removed live XP, vitals or loadout fields", () => {
  const f = fixture(),
    data = hud();
  for (const key of [
    "ship_xp",
    "ship_next_xp",
    "hull",
    "max_hull",
    "shield",
    "max_shield",
    "loadout",
  ]) {
    Object.defineProperty(data, key, {
      enumerable: true,
      get() {
        throw new Error(`removed field read: ${key}`);
      },
    });
  }
  Object.freeze(data);
  assert.equal(f.paint(f.target, data), true);
  assert.equal(byClass(f, "meishi-progress")[0].textContent, "LV 1");
  f.resetWrites();
  assert.equal(f.paint(f.target, data), false);
  assert.equal(f.writes(), 0);
});

test("only removed live fields changing leaves the card, focus and scroll untouched", () => {
  const f = fixture(),
    data = hud();
  f.paint(f.target, data);
  const before = all(f.target);
  f.target.scrollTop = 117;
  f.external.focus();
  for (const changed of [
    {
      ship_xp: 999,
      ship_next_xp: 1320,
      hull: 1,
      shield: 0,
      loadout: [null, null, null],
    },
    {
      ship_xp: NaN,
      ship_next_xp: Infinity,
      hull: 100,
      max_hull: 150,
      shield: 80,
      max_shield: 90,
      loadout: [data.loadout[2], null, null],
    },
  ]) {
    f.resetWrites();
    assert.equal(f.paint(f.target, { ...data, ...changed }), false);
    assert.equal(f.writes(), 0);
    assert.equal(f.target.scrollTop, 117);
    assert.equal(f.document.activeElement, f.external);
    all(f.target).forEach((node, index) => assert.equal(node, before[index]));
  }
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

test("invalid card level and point numbers are unavailable, not positive progression", () => {
  const f = fixture();
  const data = hud();
  data.currency = [-1, NaN, Infinity];
  for (const level of [NaN, Infinity, -1]) {
    data.ship_level = level;
    f.paint(f.target, data);
    assert.equal(byClass(f, "meishi-progress")[0].textContent, "LV --");
    assert.equal(
      byClass(f, "meishi-progress")[0].getAttribute("aria-label"),
      "RAICHO level --",
    );
    byClass(f, "meishi-points").forEach((node) =>
      assert.match(node.textContent, / --$/),
    );
  }
});

test("view source stays read-only while the card reflows without new animation loops", () => {
  const css = readFileSync(
    new URL("../../apps/game/src/meishi.css", import.meta.url),
    "utf8",
  );
  assert.doesNotMatch(
    source,
    /localStorage|indexedDB|setInterval|requestAnimationFrame|innerHTML|\.style\./,
  );
  assert.match(css, /\.meishi-card \[hidden\]\s*\{\s*display: none;/);
  assert.doesNotMatch(css, /overflow:\s*hidden|height:\s*100vh|animation:/);
  // These are source policy checks, not computed BCR or GPU layout acceptance.
});

test("collectible decoration follows main corporation while sockets reflect only earned patches", () => {
  const f = fixture(),
    data = hud();
  for (const [corpIndex, corporation] of ["pan", "renko", "hugo"].entries()) {
    data.identity.corporation = corporation;
    data.identity.sub_corporation = ["hugo", "pan", "renko"][corpIndex];
    data.campaign.champions[3].patch = true;
    f.paint(f.target, freeze(structuredClone(data)));
    assert.equal(byClass(f, "meishi-card")[0].dataset.corporation, corporation);
    for (const name of ["meishi-portrait-frame", "meishi-seal"])
      assert.equal(byClass(f, name)[0].dataset.pixelValue, String(corpIndex));
    const wells = byClass(f, "meishi-patch-well");
    assert.equal(wells.length, 8);
    wells.forEach((well, index) => {
      assert.equal(well.dataset.pixelKind, "meishi-socket");
      assert.equal(well.dataset.pixelValue, String(corpIndex * 8 + index));
      assert.equal(well.dataset.active, String(index === 3));
      assert.equal(byClass(f, "meishi-patch-icon")[index].hidden, index !== 3);
    });
    assert.equal(
      f.paint(f.target, freeze(structuredClone(data))),
      false,
      "unchanged card does not rewrite art or text",
    );
  }
});
