// Execute the real shell adapter. Native layout/projection has separate Rust
// tests; these verify logical geometry reaches physical DOM hits at any DPR.
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
const adapter = source.statements.find(
  (n) =>
    ts.isFunctionDeclaration(n) && n.name?.text === "syncFlightStatusLayout",
);
assert.ok(adapter);
const compiled = ts.transpileModule(adapter.getText(source), {
  compilerOptions: { target: ts.ScriptTarget.ES2022 },
}).outputText;
function fixture(height, viewport, rect, text) {
  const values = new Map(),
    writes = [];
  const blockers = new Map();
  const strip = {
    dataset: {},
    style: {
      getPropertyValue: (n) => values.get(n) || "",
      setProperty: (n, v) => {
        values.set(n, v);
        writes.push([n, v]);
      },
    },
  };
  const galaxy = {
    viewport: () => new Float32Array(viewport),
    flight_status_bounds: () => new Float32Array(rect),
    flight_status_text_bounds: () => new Float32Array(text),
  };
  const context = vm.createContext({
    innerHeight: height,
    galaxy,
    flightSafeInsets: [0, 0],
    touchUi: false,
    element: (id) => {
      if (id === "flight-status-strip") return strip;
      assert.ok(
        [
          "save-indicator",
          "pause-button",
          "map-button",
          "flight-settings",
          "joystick",
          "fire-button",
        ].includes(id),
      );
      return {
        getBoundingClientRect: () =>
          blockers.get(id) ?? { width: 0, height: 0 },
      };
    },
  });
  vm.runInContext(compiled, context);
  return {
    context,
    strip,
    values,
    writes,
    blockers,
    run: () => context.syncFlightStatusLayout(),
  };
}
test("actual native strip and text bounds scale together without rounding fractional quad geometry", () => {
  const f = fixture(720, [426.6667, 240], [6, 212, 414, 22], [12, 226, 402, 7]);
  f.run();
  assert.equal(f.values.get("--flight-strip-x"), "18px");
  assert.equal(f.values.get("--flight-strip-y"), "636px");
  assert.equal(f.values.get("--flight-strip-width"), "1242px");
  assert.equal(f.values.get("--flight-strip-height"), "66px");
  assert.equal(f.values.get("--flight-text-y"), "678px");
  assert.equal(f.values.get("--flight-text-height"), "21px");
  assert.equal(f.values.get("--flight-hit-y"), "654px");
  assert.equal(f.strip.dataset.contextAvailable, "true");
  assert.equal(f.strip.dataset.contextInteractive, "true");
  const writes = f.writes.length;
  f.run();
  assert.equal(
    f.writes.length,
    writes,
    "unchanged geometry does not dirty DOM every HUD paint",
  );
  const fractional = fixture(
    844,
    [390, 180],
    [6.125, 150.5, 377.75, 22],
    [12.125, 164.5, 365.75, 7],
  );
  fractional.run();
  assert.ok(
    Math.abs(
      parseFloat(fractional.values.get("--flight-text-x")) -
        (12.125 * 844) / 180,
    ) < 1e-10,
  );
});
test("clear glyph row cannot create a touch hit overlapping the real movement/fire pads", () => {
  const f = fixture(180, [280, 180], [6, 62, 268, 22], [10, 74, 260, 7]);
  f.context.touchUi = true;
  f.blockers.set("joystick", {
    left: 14,
    top: 88,
    right: 98,
    bottom: 172,
    width: 84,
    height: 84,
  });
  f.blockers.set("fire-button", {
    left: 196,
    top: 94,
    right: 268,
    bottom: 166,
    width: 72,
    height: 72,
  });
  f.run();
  assert.equal(
    f.strip.dataset.contextAvailable,
    "true",
    "toast retains its fitting native glyph slot",
  );
  assert.equal(
    f.strip.dataset.contextInteractive,
    "false",
    "contact has no concealed competing pointer hit",
  );
  f.blockers.clear();
  f.run();
  assert.equal(f.strip.dataset.contextInteractive, "true");
  f.blockers.set("map-button", {
    left: 230,
    top: 55,
    right: 274,
    bottom: 99,
    width: 44,
    height: 44,
  });
  f.run();
  assert.equal(
    f.strip.dataset.contextInteractive,
    "false",
    "actual toolbar hits also reserve space",
  );
});
test("44px contextual hit remains centered in a touch gap and clamps only at real safe-area boundaries", () => {
  const center = fixture(
    390,
    [844, 390],
    [340, 362, 164, 22],
    [346, 376, 152, 7],
  );
  center.run();
  assert.equal(center.values.get("--flight-hit-y"), "346px");
  const raised = fixture(390, [844, 390], [6, 240, 832, 22], [12, 254, 820, 7]);
  raised.run();
  assert.equal(raised.values.get("--flight-hit-y"), "235.5px");
  raised.context.flightSafeInsets = [250, 0];
  raised.run();
  assert.equal(raised.values.get("--flight-hit-y"), "250px");
  center.context.flightSafeInsets = [0, 20];
  center.run();
  assert.equal(center.values.get("--flight-hit-y"), "326px");
});
test("native no-fit and nonfinite text geometry removes invisible contextual hit areas", () => {
  for (const text of [[], [12, 10, 0, 7], [NaN, 10, 120, 7]]) {
    const f = fixture(180, [320, 180], [6, 152, 308, 22], text);
    f.run();
    assert.equal(f.strip.dataset.contextAvailable, "false");
    for (const property of ["x", "y", "width", "height"])
      assert.equal(f.values.get(`--flight-text-${property}`), "0px");
    assert.equal(f.values.get("--flight-strip-height"), "22px");
  }
});
