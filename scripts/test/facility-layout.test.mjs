import assert from "node:assert/strict";
import test from "node:test";
import vm from "node:vm";
import { readFileSync } from "node:fs";
import ts from "typescript";
const source = ts.createSourceFile(
  "main.ts",
  readFileSync(new URL("../../apps/game/src/main.ts", import.meta.url), "utf8"),
  ts.ScriptTarget.Latest,
  true,
);
const fn = source.statements.find(
  (n) => ts.isFunctionDeclaration(n) && n.name?.text === "paintFacilities",
);
assert.ok(fn);
const compiled = ts.transpileModule(fn.getText(source), {
  compilerOptions: { target: ts.ScriptTarget.ES2022 },
}).outputText;
function fixture(logicalHeight, worldHeight, stripTop) {
  const nodes = Array.from({ length: 4 }, () => ({
    hidden: false,
    style: {},
    dataset: {},
  }));
  const controls = new Map();
  const width = 1280,
    height = 720;
  const worldWidth = (worldHeight * width) / height;
  const context = {
    innerHeight: height,
    flightSafeInsets: [0, 0],
    touchUi: false,
    launched: true,
    paused: false,
    menu: { hidden: true },
    hud: { dock: true, camera: [0, 0] },
    facilityPositions: [
      [-120, -80],
      [120, -80],
      [-120, 95],
      [120, 95],
    ],
    galaxy: {
      world_viewport: () => [worldWidth, worldHeight],
      viewport: () => [(logicalHeight * width) / height, logicalHeight],
      flight_navigation_bounds: () => new Float32Array(),
      flight_status_bounds: () =>
        new Float32Array([
          6,
          stripTop,
          (logicalHeight * width) / height - 12,
          22,
        ]),
    },
    canvas: {
      getBoundingClientRect: () => ({
        left: 0,
        top: 0,
        right: width,
        bottom: height,
        width,
        height,
      }),
    },
    element: (id) =>
      id.startsWith("facility-")
        ? nodes[Number(id.slice(-1))]
        : {
            getBoundingClientRect: () =>
              controls.get(id) ?? { width: 0, height: 0 },
          },
    pixelUi: { invalidate() {} },
  };
  vm.runInNewContext(compiled, context);
  return { context, nodes, controls };
}
test("same Haven chooses above-footer labels at GPU2.4scale and below at GL2scale without moving hits", () => {
  const gpu = fixture(300, 300, 272);
  gpu.context.paintFacilities();
  assert.deepEqual(
    gpu.nodes.map((n) => n.dataset.labelPosition),
    ["below", "below", "above", "above"],
  );
  assert.equal(gpu.nodes[2].style.top, "588px");
  assert.ok(Math.abs(parseFloat(gpu.nodes[2].style.height) - 115.2) < 1e-6);
  assert.ok(gpu.nodes.every((n) => !n.hidden));
  const gl = fixture(360, 360, 332);
  gl.context.paintFacilities();
  assert.deepEqual(
    gl.nodes.map((n) => n.dataset.labelPosition),
    ["below", "below", "below", "below"],
  );
  assert.equal(gl.nodes[2].style.top, "550px");
  assert.equal(gl.nodes[2].style.height, "96px");
});
test("blocked text has no unsafe substitute and a blocked real hit is hidden", () => {
  const f = fixture(300, 300, 272);
  f.controls.set("map-button", {
    left: 210,
    right: 490,
    top: 500,
    bottom: 525,
    width: 280,
    height: 25,
  });
  f.context.paintFacilities();
  assert.equal(f.nodes[2].hidden, false);
  assert.equal(f.nodes[2].dataset.labelPosition, "hidden");
  assert.equal(f.nodes[2].dataset.pixelValue, "2");
  f.controls.set("map-button", {
    left: 280,
    right: 420,
    top: 550,
    bottom: 620,
    width: 140,
    height: 70,
  });
  f.context.paintFacilities();
  assert.equal(f.nodes[2].hidden, true);
});
test("invalid world extents and live modal gates never publish interactive facility boxes", () => {
  const f = fixture(300, 300, 272);
  f.context.galaxy.world_viewport = () => [0, 0];
  f.context.paintFacilities();
  assert.ok(f.nodes.every((n) => n.hidden));
  const live = fixture(300, 300, 272);
  live.context.menu.hidden = false;
  live.context.paintFacilities();
  assert.ok(live.nodes.every((n) => n.hidden));
});

test("actual exported minimap geometry reserves facility art and labels at UI scale", () => {
  const f = fixture(300, 300, 272);
  f.context.galaxy.flight_navigation_bounds = () =>
    new Float32Array([120, 220, 44, 44]);
  f.context.paintFacilities();
  assert.equal(f.nodes[2].hidden, true);
  assert.equal(f.nodes[3].hidden, false);
  f.context.galaxy.flight_navigation_bounds = () =>
    new Float32Array([NaN, 220, 44, 44]);
  f.context.paintFacilities();
  assert.ok(f.nodes.every((n) => !n.hidden));
});
