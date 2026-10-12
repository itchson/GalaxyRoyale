import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

const source = ts.transpileModule(
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

function fixture(scale = 1.375, flight = false, nested = {}) {
  const listeners = new Map(),
    observations = [];
  const bounds = {
    x: 8 * scale,
    y: 10 * scale,
    width: 300 * scale,
    height: 230 * scale,
  };
  const chartBounds = {
    x: 20 * scale,
    y: 36 * scale,
    width: 244.5 * scale,
    height: 150 * scale,
    ...nested.chartBounds,
  };
  let observer,
    scene,
    calls = 0;
  class Element {
    id = "";
    dataset = {};
    textContent = "";
    innerText = "";
    tagName = "SECTION";
    classList = { contains: () => false };
    getBoundingClientRect() {
      return this.bounds ?? bounds;
    }
    closest(selector) {
      if (selector === "button,label,select")
        return this instanceof Button ? this : null;
      if (selector.includes("#map-chart") && this.map) return field;
      if (
        selector.includes("#map-content") &&
        (this.inContent || this === content)
      )
        return content;
      return null;
    }
    getAttribute() {
      return null;
    }
    matches() {
      return false;
    }
    contains(other) {
      return other === this;
    }
    querySelectorAll() {
      return [];
    }
  }
  class Button extends Element {
    tagName = "BUTTON";
    disabled = false;
  }
  const root = new Element();
  root.id = flight ? "game-ui" : "map-menu";
  const content = new Element();
  content.id = "map-content";
  content.bounds = nested.contentBounds ?? bounds;
  content.parentElement = root;
  const field = new Element();
  field.id = "map-chart";
  field.map = true;
  field.bounds = chartBounds;
  field.dataset.pixelKind = "map-field";
  field.parentElement = content;
  const make = (kind, index, active = false) => {
    const element = new Button();
    element.map = true;
    element.dataset = {
      pixelKind: kind,
      pixelValue: String(index),
      active: String(active),
    };
    element.bounds = {
      x: 33.25 * scale,
      y: 42.5 * scale,
      width: 44,
      height: 44,
    };
    element.textContent = "ACCESSIBLE LABEL MUST NOT DRAW TWICE";
    return element;
  };
  const home = make("map-home", 0, true),
    site = make("map-site", 5, false);
  const player = new Element();
  player.map = true;
  player.dataset = { pixelKind: "map-player", pixelValue: String(Math.PI / 2) };
  player.bounds = {
    x: 60 * scale,
    y: 60 * scale,
    width: 12 * scale,
    height: 12 * scale,
  };
  const art = [field, home, site, player];
  const details = new Element();
  details.inContent = true;
  details.innerText = "SAFE ZONE DETAILS";
  details.bounds = nested.detailsBounds ?? bounds;
  const action = new Button();
  action.inContent = true;
  action.textContent = "SET COURSE";
  action.bounds = nested.actionBounds ?? bounds;
  const headers = [
    ["pause-button", "status-icon"],
    ["map-button", "map-icon"],
    ["flight-settings", "settings-icon"],
  ].map(([id, kind], index) => {
    const b = new Button();
    b.id = id;
    b.dataset = { pixelIcon: kind };
    b.bounds = {
      x: (210 + 24 * index) * scale,
      y: 12 * scale,
      width: 22 * scale,
      height: 22 * scale,
    };
    return b;
  });
  root.querySelectorAll = (selector) =>
    selector === "[data-pixel-kind]"
      ? flight
        ? []
        : art
      : selector === "button,.import-label"
        ? [home, site, ...(nested.actionBounds ? [action] : [])]
        : flight && selector.includes("#map-button")
          ? headers
          : nested.detailsBounds && selector.includes("h2,h3,p")
            ? [details]
            : [];
  const document = {
    querySelectorAll: () => [root],
    getElementById: (id) => (id === "menu-screen" ? root : null),
    addEventListener: (name, fn) => listeners.set(name, fn),
    activeElement: null,
    fonts: { ready: Promise.resolve() },
    body: {
      classList: { contains: (name) => name === "engine-flight" && flight },
    },
  };
  const exports = {};
  vm.runInNewContext(source, {
    exports,
    document,
    window: { addEventListener() {} },
    Element,
    HTMLElement: Element,
    HTMLButtonElement: Button,
    getComputedStyle: () => ({ display: "block", fontSize: `${8 * scale}px` }),
    performance: { now: () => 0 },
    MutationObserver: class {
      constructor(fn) {
        observer = fn;
      }
      observe(_element, opts) {
        observations.push(opts);
      }
    },
  });
  const bridge = new exports.PixelUiBridge(
    {
      set_ui_scene: (value) => {
        scene = value;
        calls++;
      },
    },
    () => scale,
  );
  bridge.sync();
  return {
    bridge,
    home,
    site,
    player,
    field,
    content,
    headers,
    listeners,
    observations,
    notify: (records) => observer(records),
    scene: () => scene,
    calls: () => calls,
  };
}

test("actual map art emits semantic Home and site buttons once with fractional chart clipping", () => {
  const f = fixture();
  const nodes = f.scene().groups[0].nodes;
  assert.deepEqual(
    Array.from(nodes, (n) => n.kind),
    ["panel", "map-field", "map-home", "map-site", "map-player"],
  );
  assert.equal(nodes.filter((n) => n.kind === "button").length, 0);
  for (const n of nodes.filter((n) => n.kind.startsWith("map-"))) {
    assert.ok(
      Math.abs(n.clip[0] - 12) < 1e-9 && Math.abs(n.clip[1] - 26) < 1e-9,
    );
    assert.ok(
      Math.abs(n.clip[2] - 244.5) < 1e-9 && Math.abs(n.clip[3] - 150) < 1e-9,
    );
  }
  const n = nodes.find((n) => n.kind === "map-site");
  assert.ok(Math.abs(n.rect[0] - 25.25) < 1e-9);
  assert.ok(Math.abs(n.rect[2] - 44 / 1.375) < 1e-9);
});

test("selected map marker remains independent of transient hover and real heading value", () => {
  const f = fixture();
  f.listeners.get("pointerover")({ target: f.site });
  f.bridge.sync();
  const nodes = f.scene().groups[0].nodes;
  assert.equal(nodes.find((n) => n.kind === "map-home").selected, true);
  assert.equal(nodes.find((n) => n.kind === "map-home").active, false);
  assert.equal(nodes.find((n) => n.kind === "map-site").active, true);
  assert.equal(nodes.find((n) => n.kind === "map-site").selected, false);
  assert.equal(nodes.find((n) => n.kind === "map-site").value, 5);
  assert.equal(nodes.find((n) => n.kind === "map-player").value, Math.PI / 2);
});

test("map style-only pan and pixel-value heading changes invalidate without text mutation", () => {
  const f = fixture();
  const calls = f.calls();
  f.bridge.sync();
  assert.equal(f.calls(), calls);
  assert.ok(f.observations[0].attributeFilter.includes("style"));
  assert.ok(f.observations[0].attributeFilter.includes("data-pixel-value"));
  f.site.bounds.x += 0.125;
  f.notify([{ attributeName: "style", target: f.site }]);
  f.bridge.sync();
  assert.equal(f.calls(), calls + 1);
  const rect = f
    .scene()
    .groups[0].nodes.find((n) => n.kind === "map-site").rect;
  assert.ok(Math.abs(rect[0] - (25.25 + 0.125 / 1.375)) < 1e-9);
  f.player.dataset.pixelValue = "0";
  f.notify([{ attributeName: "data-pixel-value", target: f.player }]);
  f.bridge.sync();
  assert.equal(
    f.scene().groups[0].nodes.find((n) => n.kind === "map-player").value,
    0,
  );
  f.notify([{ attributeName: "style", target: f.headers[0] }]);
  f.bridge.sync();
  assert.equal(f.calls(), calls + 2);
});

test("new native header semantic kinds appear through the actual flight selector", () => {
  const f = fixture(2.4, true);
  assert.deepEqual(
    Array.from(f.scene().groups[0].nodes, (n) => n.kind),
    ["status-icon", "map-icon", "settings-icon"],
  );
});

test("a wholly off-chart site is omitted and a partial hitbox keeps the exact chart clip", () => {
  const f = fixture();
  f.site.bounds.x = 1000;
  f.bridge.invalidate();
  f.bridge.sync();
  assert.equal(
    f.scene().groups[0].nodes.some((n) => n.kind === "map-site"),
    false,
  );
  f.site.bounds.x = 20 * 1.375 - 10;
  f.bridge.invalidate();
  f.bridge.sync();
  const n = f.scene().groups[0].nodes.find((n) => n.kind === "map-site");
  assert.ok(n);
  assert.equal(n.clip[0], 12);
  assert.ok(n.rect[0] < n.clip[0]);
});

test("map details and course controls are clipped to content before the saved-status footer", () => {
  const scale = 1.375;
  const f = fixture(scale, false, {
    contentBounds: {
      x: 16 * scale,
      y: 50 * scale,
      width: 260 * scale,
      height: 130 * scale,
    },
    detailsBounds: {
      x: 20 * scale,
      y: 170 * scale,
      width: 244 * scale,
      height: 40 * scale,
    },
    actionBounds: {
      x: 20 * scale,
      y: 174 * scale,
      width: 100 * scale,
      height: 24 * scale,
    },
  });
  const nodes = f.scene().groups[0].nodes;
  for (const kind of ["text", "button"]) {
    const node = nodes.find((n) => n.kind === kind);
    assert.ok(node, `${kind} remains partially visible`);
    assert.deepEqual(Array.from(node.clip), [8, 40, 260, 130]);
    assert.ok(node.rect[1] + node.rect[3] > node.clip[1] + node.clip[3]);
  }
  f.content.bounds.height = 110 * scale;
  f.bridge.invalidate();
  f.bridge.sync();
  assert.equal(
    f.scene().groups[0].nodes.some((n) => ["text", "button"].includes(n.kind)),
    false,
  );
});

test("scrolled map artwork intersects both chart and content ancestors on both axes", () => {
  const scale = 2.4;
  const f = fixture(scale, false, {
    contentBounds: {
      x: 30 * scale,
      y: 70 * scale,
      width: 200 * scale,
      height: 130 * scale,
    },
    chartBounds: {
      x: 20 * scale,
      y: 36 * scale,
      width: 244.5 * scale,
      height: 150 * scale,
    },
  });
  f.site.bounds = { x: 40 * scale, y: 64 * scale, width: 44, height: 44 };
  f.bridge.invalidate();
  f.bridge.sync();
  const nodes = f.scene().groups[0].nodes;
  for (const node of nodes.filter((n) => n.kind.startsWith("map-")))
    assert.deepEqual(Array.from(node.clip), [22, 60, 200, 116]);
  const site = nodes.find((n) => n.kind === "map-site");
  assert.ok(site, "a partly clipped marker is retained with the common clip");
  assert.ok(site.rect[1] < site.clip[1]);
  assert.equal(
    nodes.some((n) => n.kind === "map-home"),
    false,
    "a wholly header-hidden marker is omitted",
  );
  f.field.bounds.y = 0;
  f.field.bounds.height = 60 * scale;
  f.bridge.invalidate();
  f.bridge.sync();
  assert.equal(
    f.scene().groups[0].nodes.some((n) => n.kind.startsWith("map-")),
    false,
    "a wholly scrolled-away chart emits no map nodes",
  );
});
