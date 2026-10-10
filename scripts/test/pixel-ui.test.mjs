import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

const compiled = ts.transpileModule(
  readFileSync(
    new URL("../../apps/game/src/pixel-ui.ts", import.meta.url),
    "utf8",
  ),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  },
).outputText;

function pixelScene({
  min = "",
  max = "",
  value = "50",
  icon = null,
  scale = 1,
} = {}) {
  const bounds = { x: 0, y: 0, top: 0, bottom: 200, width: 300, height: 200 };
  class Element {
    id = "";
    dataset = {};
    children = [];
    classList = { contains: () => false };
    getBoundingClientRect() {
      return this.bounds ?? bounds;
    }
    closest() {
      return null;
    }
    contains() {
      return !!this.focusedChild;
    }
    querySelectorAll() {
      return [];
    }
  }
  class Input extends Element {
    type = "range";
    min = min;
    max = max;
    value = value;
  }
  const input = new Input();
  const label = new Element();
  label.childNodes = [{ textContent: "Flight zoom" }];
  label.querySelector = () => input;
  const root = new Element();
  const art = new Element();
  if (icon) {
    art.dataset = {
      pixelKind: icon.kind,
      pixelValue: String(icon.index ?? 0),
      active: String(icon.earned ?? false),
    };
    art.bounds = {
      ...bounds,
      x: 12 * scale,
      y: 20 * scale,
      width: icon.width * scale,
      height: icon.height * scale,
    };
    art.focusedChild = icon.focusedChild;
  }
  root.querySelectorAll = (selector) =>
    selector === ".settings label"
      ? [label]
      : selector === "[data-pixel-kind]" && icon
        ? [art]
        : [];
  const document = {
    getElementById: () => null,
    addEventListener() {},
    fonts: { ready: Promise.resolve() },
    querySelectorAll: () => [root],
    activeElement: null,
    body: { classList: { contains: () => false } },
  };
  const exports = {};
  vm.runInNewContext(compiled, {
    exports,
    document,
    window: { addEventListener() {} },
    Element,
    HTMLElement: Element,
    HTMLInputElement: Input,
    HTMLButtonElement: class extends Element {},
    getComputedStyle: () => ({ display: "block", fontSize: "8px" }),
    performance: { now: () => 0 },
    MutationObserver: class {
      observe() {}
    },
  });
  let scene;
  const bridge = new exports.PixelUiBridge(
    { set_ui_scene: (value) => (scene = value) },
    () => scale,
  );
  bridge.sync();
  return scene;
}
function sliderScene(options) {
  const scene = pixelScene(options);
  const slider = scene.groups[0].nodes.find((node) => node.kind === "slider");
  assert.ok(slider, "actual semantic slider must reach the WASM scene");
  return slider;
}

test("zoom slider pixel thumb follows the semantic range at both limits and default zoom", () => {
  for (const [value, expected] of [
    ["0.6", 0],
    ["1", 0.4],
    ["1.6", 1],
  ]) {
    const slider = sliderScene({ min: "0.6", max: "1.6", value });
    assert.ok(Math.abs(slider.value - expected) < 1e-12);
    assert.equal(slider.text, "Flight zoom");
  }
});

test("fractional CSS campaign icons retain authored integer fit and measured centers", () => {
  for (const [kind, size] of [
    ["campaign-patch", 24],
    ["campaign-ship", 32],
  ]) {
    for (const scale of [1, 1.375, 2.4]) {
      for (const delta of [-0.001, 0, 0.001]) {
        const measured = size + delta;
        const scene = pixelScene({
          scale,
          icon: { kind, width: measured, height: measured },
        });
        const node = scene.groups[0].nodes.find((n) => n.kind === kind);
        assert.deepEqual(Array.from(node.rect.slice(2)), [size, size]);
        assert.ok(
          Math.abs(node.rect[0] + node.rect[2] / 2 - (12 + measured / 2)) <
            1e-10,
        );
        assert.ok(
          Math.abs(node.rect[1] + node.rect[3] / 2 - (20 + measured / 2)) <
            1e-10,
        );
      }
    }
  }
  const undersized = pixelScene({
    icon: { kind: "campaign-patch", width: 23.98, height: 24 },
  });
  assert.equal(
    undersized.groups[0].nodes.find((n) => n.kind === "campaign-patch").rect[2],
    23.98,
  );
});

test("champion emblem earned state comes from saved progress rather than focus", () => {
  for (const earned of [false, true]) {
    const scene = pixelScene({
      icon: {
        kind: "campaign-patch",
        index: 8,
        width: 24,
        height: 24,
        earned,
        focusedChild: true,
      },
    });
    const node = scene.groups[0].nodes.find((n) => n.kind === "campaign-patch");
    assert.equal(node.active, earned);
    assert.equal(node.value, 8);
  }
});

test("volume and default HTML range bounds produce finite clamped pixel positions", () => {
  assert.equal(sliderScene({ min: "0", max: "1", value: "0.65" }).value, 0.65);
  assert.equal(sliderScene({ value: "50" }).value, 0.5);
  assert.equal(sliderScene({ min: "-10", max: "10", value: "0" }).value, 0.5);
  assert.equal(sliderScene({ min: "0.6", max: "1.6", value: "10" }).value, 1);
  assert.equal(sliderScene({ min: "1", max: "1", value: "1" }).value, 0);
  assert.equal(sliderScene({ min: "oops", max: "2", value: "1" }).value, 0);
});
