import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

const source = readFileSync(
  new URL("../../apps/game/src/campaign.ts", import.meta.url),
  "utf8",
).replace('import "./campaign.css";', "");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  },
}).outputText;

function fixture() {
  class Element {
    children = [];
    dataset = {};
    attributes = new Map();
    events = new Map();
    textContent = "";
    disabled = false;
    hidden = false;
    constructor(tag) {
      this.tag = tag;
    }
    append(...children) {
      this.children.push(...children);
    }
    replaceChildren(...children) {
      this.children = children;
    }
    setAttribute(name, value) {
      this.attributes.set(name, value);
    }
    addEventListener(name, callback) {
      this.events.set(name, callback);
    }
    click() {
      if (!this.disabled) this.events.get("click")?.();
    }
  }
  const document = { createElement: (tag) => new Element(tag) };
  const exports = {};
  vm.runInNewContext(compiled, { exports, document });
  const target = new Element("div");
  const calls = [];
  const actions = Object.fromEntries(
    ["navigate", "challenge", "tournament", "craft", "select", "credits"].map(
      (name) => [name, (value) => calls.push({ name, value })],
    ),
  );
  const nodes = (root) => [root, ...root.children.flatMap(nodes)];
  return {
    paint: (hud) => exports.paintCampaign(target, hud, actions),
    calls,
    text: () =>
      nodes(target)
        .map((node) => node.textContent)
        .join("\n"),
    button: (label) =>
      nodes(target).find(
        (node) => node.tag === "button" && node.textContent === label,
      ),
  };
}

function completedSeason(won) {
  return {
    champions: Array.from({ length: 8 }, (_, index) => ({
      index,
      id: `champion-${index}`,
      name: `Champion ${index}`,
      species: "Fixture species",
      ship: `Ship ${index}`,
      ability: "Fixture ability",
      silhouette: "fixture",
      family: index % 3,
      category: index % 3,
      position: [index * 100, 2000],
      patch: true,
      blueprint: true,
      crafted: false,
    })),
    arena: null,
    encounter: null,
    ships: [],
    tournament_unlocked: true,
    tournament_complete: true,
    tournament_won: won,
    tournament_position: [0, 8000],
    tournament_nearby: true,
    standings: [
      {
        entrant: 8,
        name: "YOU",
        played: 8,
        wins: won ? 8 : 0,
        losses: won ? 0 : 8,
        points: won ? 24 : 0,
        tie_break: 1,
        rank: won ? 1 : 9,
      },
    ],
    matches: [
      ...Array.from({ length: 8 }, (_, a) => ({
        a,
        b: 8,
        winner: won ? 8 : a,
        source: "played",
      })),
      { a: 0, b: 1, winner: 0, source: "simulated" },
    ],
    credits_pending: won,
    selected_ship: 0,
    tournament_attempt: 0,
  };
}

test("a completed losing season offers a new season without victory or credits claims", () => {
  const f = fixture();
  const hud = completedSeason(false);
  const before = structuredClone(hud);
  f.paint(hud);
  assert.match(f.text(), /SEASON COMPLETE/);
  assert.doesNotMatch(f.text(), /LEAGUE WON|Sovereign ship unlocked/);
  assert.equal(f.button("END CREDITS"), undefined);
  const retry = f.button("NEW SEASON");
  assert.ok(retry);
  assert.equal(retry.disabled, false);
  retry.click();
  assert.deepEqual(f.calls, [{ name: "tournament", value: undefined }]);
  assert.deepEqual(
    hud,
    before,
    "painting/click dispatch cannot mutate authoritative progression",
  );
  assert.match(f.text(), /\[SIMULATED\]/);
});

test("a losing-season retry remains gated by location and an active encounter", () => {
  for (const patch of [
    { tournament_nearby: false },
    { encounter: { index: 0 } },
  ]) {
    const f = fixture();
    f.paint({ ...completedSeason(false), ...patch });
    const retry = f.button("NEW SEASON");
    assert.equal(retry.disabled, true);
    retry.click();
    assert.deepEqual(f.calls, []);
    assert.equal(f.button("END CREDITS"), undefined);
  }
});

test("only an authoritative winning season exposes credits and Sovereign victory text", () => {
  const f = fixture();
  f.paint(completedSeason(true));
  assert.match(f.text(), /LEAGUE WON \/ Sovereign ship unlocked/);
  assert.equal(f.button("NEW SEASON").disabled, true);
  f.button("NEW SEASON").click();
  assert.deepEqual(f.calls, []);
  const credits = f.button("END CREDITS");
  assert.ok(credits);
  assert.equal(credits.disabled, false);
  credits.click();
  assert.deepEqual(f.calls, [{ name: "credits", value: undefined }]);
});
