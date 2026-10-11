import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

// Actual map view/listeners with a declared fake DOM field: not native/GPU layout QA.
const compiled = ts.transpileModule(
  readFileSync(
    new URL("../../apps/game/src/universe-map.ts", import.meta.url),
    "utf8",
  ).replace('import "./universe-map.css";', ""),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  },
).outputText;
function fixture() {
  const document = { activeElement: null };
  class Element {
    children = [];
    attributes = {};
    events = {};
    className = "";
    textContent = "";
    style = {};
    hidden = false;
    scrollTop = 0;
    parent = null;
    dataset = new Proxy(
      {},
      {
        get: (_, key) =>
          this.getAttribute(
            `data-${String(key).replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`)}`,
          ) ?? undefined,
        set: (_, key, value) => {
          this.setAttribute(
            `data-${String(key).replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`)}`,
            value,
          );
          return true;
        },
      },
    );
    classList = {
      add: (name) => {
        this.className += ` ${name}`;
      },
      remove: (name) => {
        this.className = this.className
          .split(" ")
          .filter((v) => v !== name)
          .join(" ");
      },
    };
    append(...children) {
      for (const child of children) {
        child.parent = this;
        this.children.push(child);
      }
    }
    replaceChildren(...children) {
      this.children = [];
      this.append(...children);
    }
    remove() {
      this.parent.children = this.parent.children.filter(
        (node) => node !== this,
      );
    }
    setAttribute(key, value) {
      this.attributes[key] = String(value);
    }
    getAttribute(key) {
      return this.attributes[key] ?? null;
    }
    removeAttribute(key) {
      delete this.attributes[key];
    }
    addEventListener(key, callback) {
      this.events[key] = callback;
    }
    getBoundingClientRect() {
      return { width: 400, height: 200, x: 0, y: 0 };
    }
    closest(tag) {
      return this.tagName === tag ? this : (this.parent?.closest(tag) ?? null);
    }
    setPointerCapture(id) {
      this.capture = id;
    }
    focus() {
      document.activeElement = this;
    }
  }
  document.createElement = (tagName) =>
    Object.assign(new Element(), { tagName });
  const exports = {};
  vm.runInNewContext(compiled, { exports, document });
  const target = new Element();
  const calls = [];
  const navigate = (position) => calls.push(Array.from(position));
  return { target, paint: exports.paintUniverseMap, calls, navigate, document };
}
function flatten(root) {
  return [root, ...root.children.flatMap(flatten)];
}
function text(root) {
  return flatten(root)
    .map((node) => node.textContent)
    .join("\n");
}
function find(f, label) {
  return flatten(f.target).find(
    (node) => node.getAttribute("aria-label") === label,
  );
}
function site(properties = {}) {
  return {
    id: "1:0",
    name: "RELAY ANCHORAGE",
    position: [1700, 0],
    radius: 180,
    event: "courier",
    status: "active",
    biome: { kind: 1, name: "Amethyst Drift" },
    progress: 0,
    target: 1,
    belt_inner: 280,
    belt_outer: 500,
    material_tier: 2,
    recommended_level: 3,
    ...properties,
  };
}
function data() {
  return {
    position: [50, 25],
    heading: 0.7,
    biome: { kind: 0, name: "Haven Expanse" },
    home_radius: 240,
    belt_inner: 700,
    belt_outer: 1400,
    sites: [site()],
  };
}
function frozen(value) {
  if (value && typeof value === "object") {
    Object.values(value).forEach(frozen);
    Object.freeze(value);
  }
  return value;
}

test("map displays only supplied charted sites and real Haven/belt geometry", () => {
  const f = fixture();
  assert.equal(f.paint(f.target, frozen(data()), f.navigate), true);
  const markers = flatten(f.target).filter(
    (node) => node.dataset.pixelKind === "map-site",
  );
  assert.equal(markers.length, 1);
  assert.equal(markers[0].dataset.pixelValue, "1");
  assert.equal(
    markers[0].getAttribute("aria-label"),
    "Select RELAY ANCHORAGE / Amethyst Drift",
  );
  assert.ok(
    text(f.target).includes("SAFE RADIUS 240 / ASTEROID BELT 700-1400"),
  );
  assert.ok(
    text(f.target).includes("SHIELD BLOCKS ASTEROIDS / BELT ORBITS OUTSIDE"),
  );
  const rings = flatten(f.target).filter(
    (node) => node.dataset.pixelKind === "map-ring",
  );
  assert.equal(rings.filter((node) => !node.hidden).length, 3);
  assert.equal(rings[0].dataset.pixelValue, "0");
  assert.equal(rings[1].dataset.pixelValue, "1");
  assert.equal(
    Number.parseFloat(rings[2].style.width) /
      Number.parseFloat(rings[1].style.width),
    2,
  );
  assert.equal(
    flatten(f.target).find((node) => node.dataset.pixelKind === "map-player")
      .dataset.pixelValue,
    "0.7",
  );
  const empty = fixture();
  const none = data();
  none.sites = [];
  empty.paint(empty.target, none, empty.navigate);
  assert.equal(
    flatten(empty.target).filter(
      (node) => node.dataset.pixelKind === "map-site",
    ).length,
    0,
  );
  assert.ok(
    text(empty.target).includes("REACH REMOTE SANCTUARIES TO CHART BEACONS"),
  );
});

test("selection reveals authoritative quest/tier data and navigates to current coordinates without mutating them", () => {
  const f = fixture();
  const map = data();
  f.paint(f.target, map, f.navigate);
  find(f, "Select RELAY ANCHORAGE / Amethyst Drift").events.click();
  assert.ok(text(f.target).includes("Amethyst Drift / COURIER / ACTIVE / 0/1"));
  assert.ok(text(f.target).includes("T2 MATERIALS / RECOMMENDED SHIP LV 3"));
  const updated = structuredClone(map);
  updated.sites[0].position = [1600, 70];
  updated.sites[0].status = "ready";
  updated.sites[0].progress = 1;
  f.paint(f.target, frozen(updated), f.navigate);
  assert.ok(text(f.target).includes("COURIER / READY / 1/1"));
  find(f, "Set course to selected location").events.click();
  assert.deepEqual(f.calls, [[1600, 70]]);
  find(f, "Select Haven").events.click();
  find(f, "Set course to selected location").events.click();
  assert.deepEqual(f.calls, [
    [1600, 70],
    [0, 0],
  ]);
  assert.deepEqual(map.sites[0].position, [1700, 0]);
});

test("unchanged refresh preserves nodes, selection, focus and scroll", () => {
  const f = fixture();
  const map = data();
  f.paint(f.target, map, f.navigate);
  const marker = find(f, "Select RELAY ANCHORAGE / Amethyst Drift");
  marker.events.click();
  marker.focus();
  f.target.scrollTop = 91;
  const nodes = flatten(f.target);
  assert.equal(f.paint(f.target, structuredClone(map), f.navigate), false);
  assert.deepEqual(flatten(f.target), nodes);
  assert.equal(f.target.scrollTop, 91);
  assert.equal(f.document.activeElement, marker);
  assert.equal(marker.dataset.active, "true");
});

test("real map listeners pan and zoom geometry without changing any world destination", () => {
  const f = fixture();
  const map = frozen(data());
  f.paint(f.target, map, f.navigate);
  const field = flatten(f.target).find((node) => node.id === "map-chart");
  const home = find(f, "Select Haven");
  const before = Number.parseFloat(home.style.left);
  field.events.pointerdown({
    button: 0,
    target: field,
    pointerId: 1,
    clientX: 100,
    clientY: 100,
  });
  field.events.pointermove({ pointerId: 1, clientX: 140, clientY: 110 });
  assert.ok(Math.abs(Number.parseFloat(home.style.left) - before - 40) < 0.01);
  field.events.pointercancel({ pointerId: 1 });
  field.events.pointermove({ pointerId: 1, clientX: 200, clientY: 100 });
  assert.ok(Math.abs(Number.parseFloat(home.style.left) - before - 40) < 0.01);
  find(f, "Zoom map in").events.click();
  assert.ok(
    flatten(f.target).every(
      (node) => !Object.values(node.style).some((v) => /NaN|Infinity/.test(v)),
    ),
  );
  find(f, "Center map on your ship").events.click();
  const player = flatten(f.target).find(
    (node) => node.dataset.pixelKind === "map-player",
  );
  assert.equal(player.style.left, "200.000px");
  assert.equal(player.style.top, "100.000px");
  let prevented = false;
  field.events.keydown({
    key: "ArrowRight",
    preventDefault: () => {
      prevented = true;
    },
  });
  assert.equal(prevented, true);
  assert.notEqual(player.style.left, "200.000px");
  assert.deepEqual(f.calls, []);
  assert.deepEqual(map.position, [50, 25]);
});

test("dense discovered maps bound real marker nodes and disclose omitted visible markers", () => {
  const f = fixture();
  const map = data();
  map.sites = Array.from({ length: 256 }, (_, index) =>
    site({
      id: `${index}:0`,
      name: `BEACON ${index}`,
      position: [index * 3, index * 2],
    }),
  );
  f.paint(f.target, frozen(map), f.navigate);
  const markers = flatten(f.target).filter(
    (node) => node.dataset.pixelKind === "map-site",
  );
  assert.equal(markers.length, 64);
  assert.ok(text(f.target).includes("64/256 VISIBLE BEACONS / 256 CHARTED"));
  assert.ok(flatten(f.target).length < 180);
});

test("unavailable site belts stay absent and invalid coordinates are not materialized", () => {
  const f = fixture();
  const map = data();
  map.sites[0].belt_inner = null;
  map.sites[0].belt_outer = null;
  map.sites[0].material_tier = null;
  map.sites[0].recommended_level = null;
  map.sites.push(site({ id: "bad", position: [Infinity, 0] }));
  f.paint(f.target, map, f.navigate);
  find(f, "Select RELAY ANCHORAGE / Amethyst Drift").events.click();
  assert.equal(
    flatten(f.target).filter((node) => node.dataset.pixelKind === "map-site")
      .length,
    1,
  );
  assert.equal(
    flatten(f.target).filter(
      (node) => node.dataset.pixelKind === "map-ring" && !node.hidden,
    ).length,
    4,
  );
  assert.ok(!text(f.target).includes("null"));
  assert.ok(!text(f.target).includes("RECOMMENDED SHIP LV"));
});
