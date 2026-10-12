import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

const portraitCompiled = ts.transpileModule(
  readFileSync(
    new URL("../../apps/game/src/kikai-portrait.ts", import.meta.url),
    "utf8",
  ),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  },
).outputText;

// Executes the real view with a fake DOM: not renderer, layout or physics QA.
const compiled = ts.transpileModule(
  readFileSync(
    new URL("../../apps/game/src/flight-menu.ts", import.meta.url),
    "utf8",
  ).replace('import "./flight-menu.css";', ""),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  },
).outputText;
function fixture() {
  let writes = 0;
  const document = { activeElement: null };
  class Element {
    children = [];
    attributes = {};
    className = "";
    hidden = false;
    listeners = {};
    scrollTop = 0;
    value = "";
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
    get textContent() {
      return (
        this.value + this.children.map((node) => node.textContent).join("")
      );
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
    addEventListener(type, listener) {
      (this.listeners[type] ??= []).push(listener);
    }
    click() {
      for (const listener of this.listeners.click ?? [])
        listener({ target: this });
    }
  }
  document.createElement = (tagName) =>
    Object.assign(new Element(), { tagName });
  const exports = {};
  const portraitExports = {};
  vm.runInNewContext(portraitCompiled, { exports: portraitExports });
  vm.runInNewContext(compiled, {
    exports,
    document,
    require: () => portraitExports,
  });
  return {
    paint: exports.paintFlightMenu,
    target: new Element(),
    externalButton: new Element(),
    document,
    resetWrites: () => {
      writes = 0;
    },
    writes: () => writes,
  };
}
function flatten(node) {
  return [node, ...node.children.flatMap(flatten)];
}
function visibleText(node) {
  if (node.hidden || node.getAttribute("aria-hidden") === "true") return "";
  return node.value + node.children.map(visibleText).join(" ");
}
function detailsControl(target) {
  return flatten(target).find(
    (node) => node.className === "flight-status-toggle",
  );
}
function detailsSection(target) {
  return flatten(target).find(
    (node) => node.className === "flight-status-details",
  );
}
function hud() {
  return {
    campaign: {
      selected_ship: 0,
      ships: [
        { id: 0, name: "HAVEN SCOUT" },
        { id: 9, name: "SOVEREIGN" },
      ],
    },
    ship_level: 1,
    ship_xp: 7,
    ship_next_xp: 20,
    hull: 73,
    max_hull: 100,
    shield: 4,
    max_shield: 12,
    ship_stats: {
      speed: 38.4,
      top_speed: 100,
      acceleration: 260,
      primary_damage: 18,
      fire_rate: 5.5,
      hull_regen: 0.4,
      shield_regen: 1.8,
    },
    loadout: [
      {
        id: "pulse-lens",
        name: "PULSE LENS",
        category: 0,
        index: 0,
        rank: 5,
        effective_rank: 1,
      },
      null,
      {
        id: "flux-drive",
        name: "FLUX DRIVE",
        category: 2,
        index: 2,
        rank: 1,
        effective_rank: 1,
      },
    ],
    material_cargo: [2, 1, 0, 0, 0],
    material_stash: [8, 2, 0, 0, 1],
    cargo_materials: 3,
    cargo_capacity: 16,
    currency: [9, 4, 2],
  };
}
function freeze(value) {
  if (value && typeof value === "object") {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}

test("issued license reuses the selected native appearance without resetting live-menu focus or scroll", () => {
  const f = fixture();
  const data = hud();
  data.identity = {
    kind: "talo",
    corporation: "hugo",
    sub_corporation: "pan",
    designation: "TALO-HUGO-PAN-0123456789ABCDEF",
    short_designation: "THP#0123456789ABCDEF",
    corporation_role: "Defensive",
  };
  f.paint(f.target, data);
  const portrait = flatten(f.target).find(
    (n) => n.dataset.pixelKind === "portrait",
  );
  assert.ok(portrait);
  assert.equal(portrait.hidden, false);
  assert.equal(portrait.dataset.pixelValue, "9");
  assert.equal(portrait.dataset.portraitKind, "2");
  assert.equal(portrait.dataset.portraitMain, "2");
  assert.equal(portrait.dataset.portraitSub, "0");
  assert.equal(
    portrait.attributes["aria-label"],
    "TALO profile: BLUE hair, GREEN crystals",
  );
  f.target.scrollTop = 110;
  f.externalButton.focus();
  f.resetWrites();
  assert.equal(f.paint(f.target, structuredClone(data)), false);
  assert.equal(f.writes(), 0);
  assert.equal(f.target.scrollTop, 110);
  assert.equal(f.document.activeElement, f.externalButton);
});

test("live status distinguishes earned shield recharge from a locked starter without mutating the loadout", () => {
  const f = fixture(),
    data = hud();
  data.ship_stats.shield_regen = 0;
  data.ship_stats.shield_recharge_unlocked = false;
  const before = structuredClone(data);
  f.paint(f.target, data);
  assert.match(
    f.target.textContent,
    /SHIELD RECHARGE LOCKED \/ UPGRADE AEGIS OR GUARD COIL/,
  );
  assert.doesNotMatch(f.target.textContent, /SHIELD RECHARGE 0.0\/S/);
  assert.deepEqual(data, before);
  data.ship_stats.shield_regen = 3;
  data.ship_stats.shield_recharge_unlocked = true;
  f.paint(f.target, data);
  assert.match(f.target.textContent, /SHIELD RECHARGE 3.0\/S AFTER 4S NO HIT/);
  assert.doesNotMatch(f.target.textContent, /RECHARGE LOCKED/);
});
test("status affinity follows Rust hull metadata rather than the pilot, sponsor or visible slot guesses", () => {
  const f = fixture(),
    data = hud();
  data.identity = {
    kind: "talo",
    corporation: "pan",
    sub_corporation: "hugo",
    designation: "TALO-PAN-HUGO-0123456789ABCDEF",
    short_designation: "TPH#0123456789ABCDEF",
    corporation_role: "Passive",
  };
  data.campaign.selected_ship = 2;
  data.ship_stats.affinity = {
    corporation: "renko",
    slot: 0,
    active: true,
    source: "roster_manufacturer",
  };
  const before = structuredClone(data);
  f.paint(f.target, data);
  assert.match(f.target.textContent, /RENKO \/ OFFENSE AFFINITY ACTIVE/);
  assert.doesNotMatch(f.target.textContent, /PAN \/ PASSIVE AFFINITY/);
  assert.deepEqual(data, before);
  const nodes = flatten(f.target);
  f.target.scrollTop = 67;
  f.externalButton.focus();
  f.resetWrites();
  assert.equal(f.paint(f.target, structuredClone(data)), false);
  assert.equal(f.writes(), 0);
  data.ship_stats.affinity.active = false;
  assert.equal(f.paint(f.target, data), true);
  assert.match(f.target.textContent, /RENKO \/ OFFENSE AFFINITY INACTIVE/);
  assert.equal(f.writes(), 1);
  assert.deepEqual(flatten(f.target), nodes);
  assert.equal(f.target.scrollTop, 67);
  assert.equal(f.document.activeElement, f.externalButton);
});
test("status clears nullable legacy affinity and labels Sovereign's licensed fallback explicitly", () => {
  const f = fixture(),
    data = hud();
  data.ship_stats.affinity = {
    corporation: "hugo",
    slot: 1,
    active: true,
    source: "sovereign_license",
  };
  f.paint(f.target, data);
  const row = flatten(f.target).find((n) => n.className === "flight-affinity");
  assert.equal(row.hidden, false);
  assert.equal(row.textContent, "KIKAI HUGO / DEFENSE AFFINITY ACTIVE");
  const nodes = flatten(f.target);
  data.ship_stats.affinity = null;
  f.paint(f.target, data);
  assert.equal(row.hidden, true);
  assert.equal(row.textContent, "");
  assert.deepEqual(flatten(f.target), nodes);
  delete data.ship_stats.affinity;
  f.resetWrites();
  assert.equal(f.paint(f.target, data), false);
  assert.equal(f.writes(), 0);
});
test("live status displays selected-ship XP, effective stats and ability slots", () => {
  const f = fixture();
  const data = freeze(hud());
  assert.equal(f.paint(f.target, data), true);
  const text = f.target.textContent;
  for (const expected of [
    "HAVEN SCOUT",
    "LV 1 / XP 7/20",
    "HULL 73 / 100",
    "SHIELD 4 / 12",
    "SPEED 38 / 100 U/S",
    "ACCEL 260 U/S2",
    "PRIMARY 18.0 DMG / 5.5/S",
    "HULL REPAIR 0.4/S AFTER 5S NO HIT",
    "SHIELD RECHARGE 1.8/S AFTER 4S NO HIT",
    "OFFENSE / PULSE LENS / R1",
    "DEFENSE / EMPTY",
    "PASSIVE / FLUX DRIVE / R1",
  ])
    assert.ok(text.includes(expected), expected);
  const icon = flatten(f.target).find(
    (node) => node.dataset.pixelKind === "campaign-ship",
  );
  assert.equal(icon.dataset.pixelValue, "9");
  assert.equal(icon.attributes["aria-hidden"], "true");
  assert.equal(data.loadout[0].rank, 5); // Shared owned rank is not rewritten by its effective cap.
});

test("compact default shows the short license and essentials while retaining full detail behind an accessible button", () => {
  const f = fixture(),
    data = hud();
  data.identity = {
    kind: "dora",
    corporation: "pan",
    sub_corporation: "renko",
    designation: "DORA-PAN-RENKO-7F8A52DAECE2ED52",
    short_designation: "APR#7F8A52DAECE2ED52",
    corporation_role: "Passive",
  };
  data.material_training_cargo = [1, 0, 0];
  data.material_training_stash = [2, 0, 0];
  f.paint(f.target, data);
  const toggle = detailsControl(f.target),
    details = detailsSection(f.target);
  assert.equal(toggle.tagName, "button");
  assert.equal(toggle.type, "button");
  assert.equal(toggle.getAttribute("aria-expanded"), "false");
  assert.equal(toggle.getAttribute("aria-controls"), details.id);
  assert.equal(details.hidden, true);
  const primary = visibleText(f.target);
  for (const text of [
    "MEISHI APR#7F8A52DAECE2ED52",
    "LV 1 / XP 7/20",
    "HULL 73 / 100",
    "SHIELD 4 / 12",
    "OFFENSE / PULSE LENS / R1",
    "DEFENSE / EMPTY",
    "PASSIVE / FLUX DRIVE / R1",
    "DETAILS",
  ])
    assert.ok(primary.includes(text), text);
  for (const text of [
    data.identity.designation,
    "ACCEL",
    "PRIMARY",
    "RECHARGE",
    "BANKED",
    "PHASE SILICATE",
    "T3 0",
  ])
    assert.ok(
      !primary.includes(text),
      `${text} should not crowd the default view`,
    );
  toggle.click();
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
  assert.equal(details.hidden, false);
  for (const text of [
    data.identity.designation,
    "ACCEL 260",
    "PRIMARY 18.0",
    "SHIELD RECHARGE 1.8",
  ])
    assert.ok(visibleText(f.target).includes(text), text);
  assert.doesNotMatch(
    visibleText(f.target),
    /T[1-5] 0|IRON FILAMENT 0|COOLANT KESSHO 0/,
  );
});

test("Details expansion and button focus persist through real live HUD paints, ship changes and nullable affinity", () => {
  const f = fixture(),
    data = hud();
  data.ship_stats.affinity = {
    corporation: "renko",
    slot: 0,
    active: true,
    source: "roster_manufacturer",
  };
  f.paint(f.target, data);
  const toggle = detailsControl(f.target),
    details = detailsSection(f.target),
    nodes = flatten(f.target);
  toggle.focus();
  toggle.click();
  f.target.scrollTop = 83;
  data.hull = 42;
  data.campaign.selected_ship = 9;
  data.ship_level = 2;
  data.ship_stats.affinity = null;
  f.paint(f.target, data);
  assert.equal(details.hidden, false);
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
  assert.equal(toggle.textContent, "HIDE DETAILS");
  assert.deepEqual(flatten(f.target), nodes);
  assert.equal(f.document.activeElement, toggle);
  assert.equal(f.target.scrollTop, 83);
  assert.match(visibleText(f.target), /SOVEREIGN.*HULL 42/);
  assert.doesNotMatch(visibleText(f.target), /AFFINITY/);
  f.resetWrites();
  assert.equal(f.paint(f.target, structuredClone(data)), false);
  assert.equal(f.writes(), 0);
  toggle.click();
  assert.equal(details.hidden, true);
  assert.equal(toggle.getAttribute("aria-expanded"), "false");
  assert.doesNotMatch(visibleText(f.target), /PRIMARY|BANKED/);
  assert.equal(f.document.activeElement, toggle);
  assert.equal(f.target.scrollTop, 83);
  const other = fixture();
  other.paint(other.target, data);
  assert.equal(detailsSection(other.target).hidden, true); // No global expanded state leaks to another view.
});

test("RAICHO statistics do not expose material inventory, capacity slots or MEISHI currency", () => {
  const f = fixture(),
    data = hud();
  data.material_training_cargo = [1, 0, 1];
  data.material_training_stash = [2, 3, 3];
  f.paint(f.target, data);
  detailsControl(f.target).click();
  assert.doesNotMatch(
    visibleText(f.target),
    /CARGO|CARRIED|BANKED|POINTS|SILICATE|FILAMENT|KESSHO/,
  );
  assert.match(visibleText(f.target), /OFFENSE \/ PULSE LENS/);
  assert.match(visibleText(f.target), /PRIMARY 18.0/);
  const nodes = flatten(f.target);
  f.resetWrites();
  data.cargo_materials = 0;
  data.material_cargo = [0, 0, 0, 0, 0];
  data.material_stash = [12, 9, 0, 0, 0];
  assert.equal(f.paint(f.target, data), false);
  assert.equal(f.writes(), 0);
  assert.deepEqual(flatten(f.target), nodes);
});

test("identical or sub-display-resolution live readings perform zero DOM writes", () => {
  const f = fixture();
  const data = hud();
  f.paint(f.target, data);
  f.resetWrites();
  assert.equal(f.paint(f.target, structuredClone(data)), false);
  data.ship_stats.speed += 0.01;
  assert.equal(f.paint(f.target, data), false);
  assert.equal(f.writes(), 0);
});

test("changed values update stable nodes while retaining scroll and action focus", () => {
  const f = fixture();
  const data = hud();
  f.paint(f.target, data);
  const nodes = flatten(f.target);
  f.target.scrollTop = 87;
  f.externalButton.focus();
  f.resetWrites();
  data.hull = 72;
  data.ship_stats.speed = 62;
  data.loadout[0].effective_rank = 2;
  assert.equal(f.paint(f.target, data), true);
  assert.equal(f.writes(), 3);
  assert.deepEqual(flatten(f.target), nodes);
  assert.equal(f.target.scrollTop, 87);
  assert.equal(f.document.activeElement, f.externalButton);
  assert.ok(f.target.textContent.includes("PULSE LENS / R2"));
});

test("ship switches use the new ship XP and exact shared icon mapping, including max-level caption", () => {
  const f = fixture();
  const data = hud();
  f.paint(f.target, data);
  data.campaign.selected_ship = 9;
  data.ship_level = 100;
  data.ship_xp = 99000;
  data.ship_next_xp = 101000;
  assert.equal(f.paint(f.target, data), true);
  assert.ok(f.target.textContent.includes("SOVEREIGN"));
  assert.ok(f.target.textContent.includes("LV 100 / XP 99000 / MAX LEVEL"));
  assert.ok(!f.target.textContent.includes("101000"));
  assert.equal(
    flatten(f.target).find((node) => node.dataset.pixelKind === "campaign-ship")
      .dataset.pixelValue,
    "8",
  );
});

test("HUD names remain plain text and unavailable numbers are not fabricated", () => {
  const f = fixture();
  const data = hud();
  data.campaign.ships[0].name = "<img onerror=alert(1)>";
  data.loadout[0].name = "<script>reward()</script>";
  data.ship_stats.speed = NaN;
  data.ship_stats.fire_rate = Infinity;
  assert.equal(f.paint(f.target, data), true);
  assert.ok(f.target.textContent.includes("<img onerror=alert(1)>"));
  assert.ok(f.target.textContent.includes("<script>reward()</script>"));
  assert.ok(f.target.textContent.includes("SPEED -- / 100 U/S"));
  assert.ok(f.target.textContent.includes("DMG / --/S"));
  assert.ok(
    flatten(f.target).every(
      (node) => !["img", "script"].includes(node.tagName),
    ),
  );
});

test("dedicated status actions and title Quit use supported native icons with accessible labels", () => {
  const html = readFileSync(
    new URL("../../apps/game/index.html", import.meta.url),
    "utf8",
  );
  const expected = {
    resume: "resume-icon",
    "restart-flight": "restart-icon",
    "pause-settings": "settings-icon",
    "return-title": "title-icon",
    "quit-game": "quit-icon",
  };
  for (const [id, icon] of Object.entries(expected)) {
    const button = html.match(
      new RegExp(`<button\\b[^>]*\\bid="${id}"[^>]*>`),
    )?.[0];
    assert.ok(button, `Missing ${id}`);
    assert.ok(button.includes(`data-pixel-icon="${icon}"`), `${id} icon`);
    assert.match(button, /aria-label="[^"]+"/);
    assert.match(button, /title="[^"]+"/);
  }
  assert.match(html, /id="flight-status"[^>]+aria-label="Live RAICHO status"/);
  assert.ok(!html.includes("Flight paused"));
});
