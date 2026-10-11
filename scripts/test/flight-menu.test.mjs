import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

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
    focus() {
      document.activeElement = this;
    }
  }
  document.createElement = (tagName) =>
    Object.assign(new Element(), { tagName });
  const exports = {};
  vm.runInNewContext(compiled, { exports, document });
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

test("live status displays selected-ship XP, effective stats, slots and actual cargo tiers", () => {
  const f = fixture();
  const data = freeze(hud());
  assert.equal(f.paint(f.target, data), true);
  const text = f.target.textContent;
  for (const expected of [
    "HAVEN SCOUT",
    "LV 1 / XP 7 / NEXT 20",
    "HULL 73 / 100",
    "SHIELD 4 / 12",
    "SPEED 38 / 100 U/S",
    "ACCEL 260 U/S2",
    "PRIMARY 18.0 DMG / 5.5/S",
    "HULL REPAIR 0.4/S AFTER 5S NO HIT",
    "SHIELD REGEN 1.8/S AFTER 4S NO HIT",
    "OFFENSE / PULSE LENS / ACTIVE R1",
    "DEFENSE / EMPTY",
    "PASSIVE / FLUX DRIVE / ACTIVE R1",
    "CARGO 3 / 16 / AT RISK",
    "T1 2 / T2 1 / T3 0 / T4 0 / T5 0",
    "BANKED / T1 8 / T2 2 / T3 0 / T4 0 / T5 1",
    "POINTS / OFF 9 / DEF 4 / PASS 2",
  ])
    assert.ok(text.includes(expected), expected);
  const icon = flatten(f.target).find(
    (node) => node.dataset.pixelKind === "campaign-ship",
  );
  assert.equal(icon.dataset.pixelValue, "9");
  assert.equal(icon.attributes["aria-hidden"], "true");
  assert.equal(data.loadout[0].rank, 5); // Shared owned rank is not rewritten by its effective cap.
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
  assert.ok(f.target.textContent.includes("ACTIVE R2"));
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
  assert.match(html, /id="flight-status"[^>]+aria-label="Live ship status"/);
  assert.ok(!html.includes("Flight paused"));
});
