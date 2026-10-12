import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

const source = ts.transpileModule(
  readFileSync(
    new URL("../../apps/game/src/cargo.ts", import.meta.url),
    "utf8",
  ).replace('import "./cargo.css";', ""),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  },
).outputText;
function fixture() {
  class Element {
    children = [];
    textContent = "";
    scrollTop = 0;
    append(...nodes) {
      this.children.push(...nodes);
    }
    replaceChildren(...nodes) {
      this.children = nodes;
    }
    setAttribute() {}
  }
  const exports = {};
  vm.runInNewContext(source, {
    exports,
    document: { createElement: () => new Element() },
  });
  return { ...exports, target: new Element() };
}
const hud = () => ({
  material_cargo: [10, 2, 0, 0, 0],
  material_stash: [30, 0, 4, 0, 0],
  material_training_cargo: [3, 2, 1],
  material_training_stash: [10, 0, 10],
  cargo_materials: 12,
  cargo_capacity: 16,
  quest_log: [],
  currency: [100, 200, 300],
});
const text = (node) => [node.textContent, ...node.children.map(text)].join(" ");

test("named T1 subsets sum to the true cargo and bank totals, never double counted", () => {
  const f = fixture(),
    data = hud();
  const rows = f.cargoRows(data);
  assert.equal(
    rows.reduce((n, row) => n + row.carried, 0),
    12,
  );
  assert.equal(
    rows.reduce((n, row) => n + row.banked, 0),
    34,
  );
  assert.equal(
    rows.find((row) => row.label === "OTHER T1 MATERIALS").carried,
    4,
  );
  f.paintCargo(f.target, data);
  assert.doesNotMatch(text(f.target), /100|200|300|POINTS/);
});
test("banking updates inventory while unchanged snapshots retain DOM and scroll", () => {
  const f = fixture(),
    data = hud();
  f.paintCargo(f.target, data);
  const children = f.target.children;
  f.target.scrollTop = 50;
  assert.equal(f.paintCargo(f.target, data), false);
  assert.equal(f.target.children, children);
  data.material_cargo = [0, 0, 0, 0, 0];
  data.material_training_cargo = [0, 0, 0];
  data.material_stash = [40, 2, 4, 0, 0];
  data.material_training_stash = [13, 2, 11];
  data.cargo_materials = 0;
  assert.equal(f.paintCargo(f.target, data), true);
  assert.equal(f.target.scrollTop, 50);
  assert.match(text(f.target), /CARRIED 0\/16/);
  assert.equal(
    f.cargoRows(data).reduce((n, row) => n + row.banked, 0),
    46,
  );
});
test("only active courier packets appear as mission items, with actual delivery state", () => {
  const f = fixture(),
    data = hud();
  data.quest_log = [
    {
      site_id: "courier-A",
      event: "courier",
      progress: 0,
      target: 1,
      completed: false,
    },
    {
      site_id: "courier-B",
      event: "courier",
      progress: 1,
      target: 1,
      completed: false,
    },
    {
      site_id: "claimed-C",
      event: "courier",
      progress: 1,
      target: 1,
      completed: true,
    },
    {
      site_id: "bounty-D",
      event: "bounty",
      progress: 0,
      target: 10,
      completed: false,
    },
  ];
  f.paintCargo(f.target, data);
  assert.match(text(f.target), /courier-A \/ RETURN TO HAVEN/);
  assert.match(text(f.target), /courier-B \/ DELIVERED TO HAVEN/);
  assert.doesNotMatch(text(f.target), /claimed-C|bounty-D/);
  assert.match(text(f.target), /no material cargo slots/);
});
