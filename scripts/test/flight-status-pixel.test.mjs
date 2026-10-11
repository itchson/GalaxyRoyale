import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

const code = ts.transpileModule(
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

function fixture({
  scale = 1.375,
  toastVisible = false,
  flight = true,
  slot = true,
  interactive = true,
  facilityValue = null,
  facilityFocused = false,
  slotHeight = 7,
} = {}) {
  let output, callback;
  const props = Object.fromEntries(
    ["x", "y", "width", "height"].map((axis, i) => [
      `--flight-text-${axis}`,
      slot ? `${[106, 164, 122, slotHeight][i] * scale}px` : "0px",
    ]),
  );
  class Element {
    id = "";
    dataset = {};
    tagName = "DIV";
    innerText = "";
    classList = { contains: () => false };
    parentElement = null;
    bounds = { x: 0, y: 0, width: 320 * scale, height: 180 * scale };
    getBoundingClientRect() {
      return this.bounds;
    }
    closest(selector) {
      if (selector === "button,label,select")
        return this instanceof Button ? this : null;
      if (
        selector.includes("#flight-status-strip") &&
        (this === strip || this.parentElement === strip)
      )
        return strip;
      return null;
    }
    getAttribute() {
      return null;
    }
    contains(other) {
      return this === other;
    }
    querySelectorAll() {
      return [];
    }
    querySelector() {
      return null;
    }
  }
  class Button extends Element {
    tagName = "BUTTON";
    disabled = false;
  }
  const root = new Element();
  root.id = "game-ui";
  const strip = new Element();
  strip.id = "flight-status-strip";
  strip.parentElement = root;
  const site = new Button();
  site.id = "site-button";
  site.parentElement = strip;
  site.dataset = { pixelKind: "status-text" };
  site.innerText = "FORGE / CONTACT [E]";
  // This is the actual44physical-pixel minimum, not its7logical-pixel font.
  site.bounds = {
    x: 106 * scale,
    y: 140 * scale,
    width: 122 * scale,
    height: Math.max(44, 22 * scale),
  };
  const toast = new Element();
  toast.id = "toast";
  toast.parentElement = strip;
  toast.dataset = { pixelKind: "warning-text", pixelValue: "1" };
  toast.innerText = "LOW HULL";
  toast.classList = { contains: (name) => name === "visible" && toastVisible };
  toast.bounds = {
    x: 10 * scale,
    y: 110 * scale,
    width: 230 * scale,
    height: 45 * scale,
  };
  const facility = new Button();
  facility.id = "facility-journal";
  facility.parentElement = root;
  facility.classList = { contains: (name) => name === "dock-facility" };
  facility.dataset = { pixelValue: String(facilityValue ?? 0) };
  facility.innerText = "JOURNAL";
  facility.bounds = {
    x: 140 * scale,
    y: 120 * scale,
    width: 48 * scale,
    height: 48 * scale,
  };
  root.querySelector = (selector) => (selector === "#toast" ? toast : null);
  root.querySelectorAll = (selector) =>
    selector === "[data-pixel-kind]"
      ? [site, toast]
      : selector.includes("#site-button") || selector === "#toast"
        ? [site, toast, ...(facilityValue === null ? [] : [facility])]
        : [];
  const document = {
    querySelectorAll: () => [root],
    getElementById: () => root,
    addEventListener() {},
    body: {
      classList: { contains: (name) => flight && name === "engine-flight" },
    },
    activeElement: facilityFocused ? facility : null,
    fonts: { ready: Promise.resolve() },
  };
  const exports = {};
  vm.runInNewContext(code, {
    exports,
    document,
    window: { addEventListener() {} },
    Element,
    HTMLElement: Element,
    HTMLButtonElement: Button,
    performance: { now: () => 0 },
    getComputedStyle: (element) => ({
      display: element === site && !interactive ? "none" : "block",
      fontSize: `${8 * scale}px`,
      getPropertyValue: (name) => props[name] ?? "",
    }),
    MutationObserver: class {
      constructor(fn) {
        callback = fn;
      }
      observe() {}
    },
  });
  const bridge = new exports.PixelUiBridge(
    {
      set_ui_scene: (scene) => {
        output = JSON.parse(JSON.stringify(scene));
      },
    },
    () => scale,
  );
  bridge.sync();
  return {
    nodes: () => output.groups.flatMap((group) => group.nodes),
    callback,
    bridge,
    strip,
  };
}

test("compiled bridge emits exact single-line footer glyphs and a separate real44px contact hit once", () => {
  for (const scale of [1, 1.375, 2.4]) {
    const f = fixture({ scale }),
      nodes = f.nodes();
    assert.deepEqual(
      nodes.map((node) => node.kind),
      ["status-hit", "status-text"],
    );
    const [hit, text] = nodes;
    assert.ok(
      text.rect.every(
        (value, index) => Math.abs(value - [106, 164, 122, 7][index]) < 1e-12,
      ),
    );
    assert.ok(Math.abs(hit.rect[3] - Math.max(44 / scale, 22)) < 1e-12);
    assert.equal(text.text, "FORGE / CONTACT [E]");
    assert.ok(hit.rect[3] > text.rect[3]);
  }
});

test("five-cell micro footer preserves warnings and contact input at fractional display scales", () => {
  for (const scale of [1, 1.375, 2.4, 4.6875]) {
    for (const toastVisible of [false, true]) {
      const nodes = fixture({ scale, slotHeight: 5, toastVisible }).nodes();
      const text = nodes.find(
        (node) => node.kind === (toastVisible ? "warning-text" : "status-text"),
      );
      assert.ok(
        text,
        "small native text must not silently disappear in the semantic bridge",
      );
      assert.ok(
        text.rect.every(
          (value, index) => Math.abs(value - [106, 164, 122, 5][index]) < 1e-10,
        ),
      );
      const hit = nodes.find((node) => node.kind === "status-hit");
      if (toastVisible) assert.equal(hit, undefined);
      else
        assert.ok(
          hit.rect[3] * scale >= 44,
          "smaller artwork must retain an accessible contact target",
        );
    }
  }
  assert.deepEqual(fixture({ slotHeight: 4 }).nodes(), []);
});

test("compiled bridge forwards focused facility label placement without changing hit geometry", () => {
  for (const value of [0, 1, 2]) {
    const nodes = fixture({
      facilityValue: value,
      facilityFocused: true,
    }).nodes();
    const label = nodes.find((node) => node.kind === "facility-label");
    assert.ok(label);
    assert.equal(label.value, value);
    assert.deepEqual(label.rect, [140, 120, 48, 48]);
    assert.equal(label.text, "JOURNAL");
  }
  assert.equal(
    fixture({ facilityValue: 0 })
      .nodes()
      .some((n) => n.kind === "facility-label"),
    false,
  );
});

test("visible toast wins the native slot without leaving a concealed contact hit", () => {
  const nodes = fixture({ toastVisible: true }).nodes();
  assert.deepEqual(
    nodes.map((node) => node.kind),
    ["warning-text"],
  );
  assert.deepEqual(nodes[0].rect, [106, 164, 122, 7]);
  assert.equal(nodes[0].value, 1);
  assert.equal(nodes[0].text, "LOW HULL");
});

test("unavailable or unsafe interactive context creates no invisible button geometry", () => {
  assert.deepEqual(fixture({ slot: false }).nodes(), []);
  assert.deepEqual(fixture({ interactive: false }).nodes(), []);
  const nodes = fixture({ interactive: false, toastVisible: true }).nodes();
  assert.deepEqual(
    nodes.map((node) => node.kind),
    ["warning-text"],
  );
});

test("menu recovery toast retains multiline DOM bounds instead of a flight-only font slot", () => {
  const nodes = fixture({ flight: false, toastVisible: true }).nodes();
  assert.deepEqual(
    nodes.map((node) => node.kind),
    ["text"],
  );
  assert.deepEqual(nodes[0].rect, [10, 110, 230, 45]);
});

test("footer style and interaction visibility changes invalidate actual bridge geometry", () => {
  const f = fixture();
  let calls = 0;
  f.bridge.invalidate = () => {
    calls++;
  };
  f.callback([{ attributeName: "style", target: f.strip }]);
  f.callback([{ attributeName: "data-context-interactive", target: f.strip }]);
  assert.equal(calls, 2);
});
