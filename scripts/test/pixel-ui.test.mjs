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
  fontSize = "8px",
  buttons = [],
  scrollBounds = null,
  rootBounds = null,
  consoleCommand = null,
  scrollId = "archive-content",
  flight = false,
  hover = null,
  pressed = null,
  releasePress = false,
  leaveHover = false,
  settingGeometry = null,
  paragraph = null,
} = {}) {
  const bounds = rootBounds ?? {
    x: 0,
    y: 0,
    top: 0,
    bottom: 200,
    width: 300,
    height: 200,
  };
  let activePointer = null;
  class Element {
    id = "";
    dataset = {};
    children = [];
    classList = { contains: () => false };
    getBoundingClientRect() {
      return this.bounds ?? bounds;
    }
    closest(selector) {
      if (selector === "button,label,select") return this;
      if (selector === ".menu-tabs" && this.nav) return root;
      if (
        selector.includes(
          scrollId.startsWith(".") ? scrollId : `#${scrollId}`,
        ) &&
        this.inScroll &&
        scrollBounds
      )
        return { getBoundingClientRect: () => scrollBounds };
      return null;
    }
    getAttribute(name) {
      return this.attributes?.[name] ?? null;
    }
    contains(element) {
      return element === this || !!this.focusedChild;
    }
    matches(selector) {
      return selector === ":active" && buttonElements?.[activePointer] === this;
    }
    querySelectorAll() {
      return [];
    }
  }
  class Input extends Element {
    disabled = false;
    type = "range";
    min = min;
    max = max;
    value = value;
  }
  class Button extends Element {}
  const input = new Input();
  const label = new Element();
  const settingName = new Element();
  settingName.textContent = "Flight zoom";
  if (settingGeometry) {
    label.bounds = settingGeometry.row;
    input.bounds = settingGeometry.input;
    input.disabled = settingGeometry.disabled ?? false;
    settingName.bounds = settingGeometry.name;
  }
  label.childNodes = [{ textContent: "Flight zoom" }];
  label.querySelector = (selector) =>
    selector === ".setting-name"
      ? settingGeometry
        ? settingName
        : null
      : input;
  const root = new Element();
  const text = new Element();
  if (paragraph) {
    text.id = paragraph.id ?? "dialogue-text";
    text.tagName = "P";
    text.innerText = paragraph.text ?? "A complete briefing to scroll.";
    text.bounds = paragraph.bounds;
    text.clientHeight = paragraph.clientHeight ?? paragraph.bounds.height;
    text.scrollHeight = paragraph.scrollHeight;
    text.scrollTop = paragraph.scrollTop ?? 0;
    text.parentElement = root;
    text.closest = (selector) =>
      selector.includes("#dialogue-text") && text.id === "dialogue-text"
        ? text
        : null;
  }
  if (flight) root.id = "game-ui";
  const events = new Map();
  if (consoleCommand) {
    root.classList = { contains: (name) => name === "console-card" };
    input.id = "console-input";
    input.type = "text";
    input.value = consoleCommand.text;
    input.selectionStart = consoleCommand.selection;
    root.querySelector = () => input;
  }
  const buttonElements = buttons.map((props) => {
    const button = new Button();
    Object.assign(button, props);
    button.classList = {
      contains: (name) => props.classes?.includes(name) ?? false,
    };
    return button;
  });
  const art = new Element();
  if (icon) {
    art.dataset = {
      pixelKind: icon.kind,
      pixelValue: String(icon.index ?? 0),
      active: String(icon.earned ?? false),
      ...icon.dataset,
    };
    art.bounds = {
      ...bounds,
      x: 12 * scale,
      y: 20 * scale,
      width: icon.width * scale,
      height: icon.height * scale,
    };
    art.focusedChild = icon.focusedChild;
    art.inScroll = icon.inScroll;
  }
  root.querySelectorAll = (selector) =>
    selector === ".settings label"
      ? [label]
      : selector === "[data-pixel-kind]" && icon
        ? [art]
        : selector === "button,.import-label"
          ? buttonElements
          : flight && selector.includes(".dock-facility")
            ? buttonElements
            : paragraph && selector.startsWith("h2,h3,p,")
              ? [text]
              : [];
  const document = {
    getElementById: () => null,
    addEventListener(name, callback) {
      events.set(name, callback);
    },
    fonts: { ready: Promise.resolve() },
    querySelectorAll: () => [root],
    activeElement: consoleCommand ? input : null,
    body: {
      classList: { contains: (name) => flight && name === "engine-flight" },
    },
  };
  const exports = {};
  vm.runInNewContext(compiled, {
    exports,
    document,
    window: { addEventListener() {} },
    Element,
    HTMLElement: Element,
    HTMLInputElement: Input,
    HTMLButtonElement: Button,
    getComputedStyle: () => ({ display: "block", fontSize }),
    performance: { now: () => 0 },
    MutationObserver: class {
      observe() {}
    },
  });
  let scene;
  const snapshots = [];
  const bridge = new exports.PixelUiBridge(
    {
      set_ui_scene: (value) => {
        scene = value;
        snapshots.push(value);
      },
    },
    () => scale,
  );
  bridge.sync();
  if (pressed !== null) {
    activePointer = pressed;
    events.get("pointerdown")();
    bridge.sync();
    if (releasePress) {
      activePointer = null;
      events.get("pointercancel")();
      bridge.sync();
    }
  }
  if (hover !== null) {
    events.get("pointerover")({ target: buttonElements[hover] });
    bridge.sync();
    if (leaveHover) {
      events.get("pointerout")();
      bridge.sync();
    }
  }
  if (consoleCommand?.moveSelection !== undefined) {
    input.selectionStart = consoleCommand.moveSelection;
    events.get("selectionchange")();
    bridge.sync();
  }
  for (const scroll of paragraph?.scrollTo ?? []) {
    text.scrollTop = scroll;
    events.get("scroll")();
    bridge.sync();
  }
  if (paragraph?.scrollTo || releasePress) return snapshots;
  return scene;
}
function sliderScene(options) {
  const scene = pixelScene(options);
  const slider = scene.groups[0].nodes.find((node) => node.kind === "slider");
  assert.ok(slider, "actual semantic slider must reach the WASM scene");
  return slider;
}

test("live license portraits deliver all 27 independent appearance choices to native rendering", () => {
  for (const kind of [0, 1, 2])
    for (const main of [0, 1, 2])
      for (const sub of [0, 1, 2]) {
        const node = pixelScene({
          icon: {
            kind: "portrait",
            index: 9,
            width: 96,
            height: 128,
            dataset: {
              portraitKind: String(kind),
              portraitMain: String(main),
              portraitSub: String(sub),
              portraitExpression: "2",
              portraitSpeaking: "true",
            },
          },
        }).groups[0].nodes.find((n) => n.kind === "portrait");
        assert.equal(node.value, 9);
        assert.equal(node.portrait_kind, kind);
        assert.equal(node.portrait_main, main);
        assert.equal(node.portrait_sub, sub);
        assert.equal(node.portrait_expression, 2);
        assert.equal(node.portrait_speaking, true);
      }
});

test("missing, fractional or malformed profile tokens never invent a fallback identity", () => {
  for (const invalid of [undefined, "", "-1", "3", "1.5", "01", "NaN"]) {
    const node = pixelScene({
      icon: {
        kind: "portrait",
        index: 9,
        width: 96,
        height: 128,
        dataset: {
          portraitKind: invalid,
          portraitMain: "0",
          portraitSub: "1",
        },
      },
    }).groups[0].nodes.find((n) => n.kind === "portrait");
    for (const field of ["portrait_kind", "portrait_main", "portrait_sub"]) {
      assert.equal(field in node, false);
    }
  }
  const guide = pixelScene({
    icon: {
      kind: "portrait",
      index: 8,
      width: 96,
      height: 128,
      dataset: {
        portraitKind: "2",
        portraitMain: "2",
        portraitSub: "2",
      },
    },
  }).groups[0].nodes.find((n) => n.kind === "portrait");
  assert.equal(
    "portrait_kind" in guide,
    false,
    "fixed APR guide is not recolored by profile fields",
  );
});

test("short-window license body clips the full portrait geometry above the fixed footer", () => {
  const options = {
    scrollId: ".droid-body",
    scrollBounds: { x: 0, y: 70, width: 300, height: 40 },
    icon: {
      kind: "portrait",
      index: 9,
      width: 96,
      height: 128,
      inScroll: true,
      dataset: { portraitKind: "1", portraitMain: "0", portraitSub: "2" },
    },
  };
  const node = pixelScene(options).groups[0].nodes.find(
    (n) => n.kind === "portrait",
  );
  assert.deepEqual(Array.from(node.rect), [12, 20, 96, 128]);
  assert.deepEqual(Array.from(node.clip), [0, 70, 300, 40]);
  assert.equal(node.portrait_kind, 1);
  const hidden = pixelScene({
    ...options,
    scrollBounds: { x: 0, y: 160, width: 300, height: 40 },
  });
  assert.equal(
    hidden.groups[0].nodes.some((n) => n.kind === "portrait"),
    false,
  );
});

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

test("native controls receive physical active state and cancellation without changing geometry", () => {
  const states = pixelScene({
    buttons: [
      { dataset: { pixelIcon: "settings-icon" }, textContent: "Settings" },
    ],
    pressed: 0,
    releasePress: true,
  });
  const control = (scene) =>
    scene.groups[0].nodes.find((n) => n.kind === "settings-icon");
  assert.deepEqual(
    states.map((s) => control(s).selected),
    [false, true, false],
  );
  assert.deepEqual(control(states[0]).rect, control(states[1]).rect);
  assert.deepEqual(control(states[1]).rect, control(states[2]).rect);
});
test("semantic actions carry persistent selection independently of focus and category colors", () => {
  const actions = [
    { nav: true, dataset: { active: "true" }, textContent: "SAVE" },
    { nav: true, focusedChild: true, textContent: "SETTINGS" },
    { classes: ["primary-button"], textContent: "RETURN TO FLIGHT" },
    { id: "return-title", textContent: "TITLE SCREEN" },
    ...["offense", "defense", "passive"].map((category) => ({
      dataset: { pixelAction: `${category}-button` },
      disabled: true,
      textContent: "UPGRADE",
    })),
  ];
  const nodes = pixelScene({ buttons: actions }).groups[0].nodes.filter(
    (n) => n.kind.includes("button") || n.kind === "nav-tab",
  );
  assert.deepEqual(
    Array.from(nodes, (n) => n.kind),
    [
      "nav-tab",
      "nav-tab",
      "primary-button",
      "exit-button",
      "offense-button",
      "defense-button",
      "passive-button",
    ],
  );
  assert.equal(nodes[0].selected, true);
  assert.equal(nodes[0].active, false);
  assert.equal(nodes[1].selected, false);
  assert.equal(nodes[1].active, true);
  assert.ok(nodes.slice(4).every((n) => n.disabled));
});

test("scroll clipping intersects panel on both axes and rejects wholly obscured actions", () => {
  const rootBounds = { x: 40, y: 20, width: 200, height: 180 };
  const scrollBounds = { x: 10, y: 60, width: 260, height: 100 };
  const nodes = pixelScene({
    rootBounds,
    scrollBounds,
    buttons: [
      {
        inScroll: true,
        bounds: { x: 35, y: 80, width: 60, height: 24 },
        textContent: "RESTORE",
      },
      {
        inScroll: true,
        bounds: { x: 280, y: 80, width: 40, height: 24 },
        textContent: "OUTSIDE",
      },
      {
        inScroll: true,
        bounds: { x: 80, y: 30, width: 40, height: 24 },
        textContent: "ABOVE",
      },
    ],
  }).groups[0].nodes.filter((n) => n.kind === "button");
  assert.equal(nodes.length, 1);
  assert.equal(nodes[0].text, "RESTORE");
  assert.deepEqual(Array.from(nodes[0].clip), [0, 40, 200, 100]);
});

test("console command preserves spaces and sends the actual codepoint caret, including selection-only edits", () => {
  for (const [text, selection, expected] of [
    ["  WARP  12 34 ", 9, 9],
    ["A🚀 B", 3, 2],
    ["HELP", null, 4],
  ]) {
    const node = pixelScene({
      consoleCommand: { text, selection },
    }).groups[0].nodes.find((n) => n.kind === "input");
    assert.equal(node.text, text);
    assert.equal(node.value, expected);
    assert.equal(node.active, true);
  }
  const node = pixelScene({
    consoleCommand: { text: "HELP", selection: 4, moveSelection: 1 },
  }).groups[0].nodes.find((n) => n.kind === "input");
  assert.equal(node.value, 1);
});

test("pause actions inherit their own scroll viewport rather than bleeding into the footer", () => {
  const node = pixelScene({
    scrollId: "pause-content",
    scrollBounds: { x: 0, y: 60, width: 300, height: 60 },
    buttons: [
      {
        inScroll: true,
        bounds: { x: 0, y: 100, width: 300, height: 44 },
        textContent: "RESTART",
      },
    ],
  }).groups[0].nodes.find((n) => n.kind === "button");
  assert.deepEqual(Array.from(node.clip), [0, 60, 300, 60]);
});

test("settings transfer independently measured input and label rectangles through UI scaling", () => {
  const slider = sliderScene({
    scale: 2,
    rootBounds: { x: 40, y: 20, width: 500, height: 360 },
    settingGeometry: {
      row: { x: 60, y: 80, width: 220, height: 80 },
      name: { x: 68, y: 86, width: 180, height: 20 },
      input: { x: 68, y: 116, width: 192, height: 28 },
      disabled: true,
    },
  });
  assert.deepEqual(Array.from(slider.label_rect), [14, 33, 90, 10]);
  assert.deepEqual(Array.from(slider.control_rect), [14, 48, 96, 14]);
  assert.deepEqual(Array.from(slider.rect), [10, 30, 110, 40]);
  assert.equal(slider.disabled, true);
  assert.equal(slider.text, "Flight zoom");
});

test("physical menu font caps preserve sub-unit native glyph scale on large viewports", () => {
  for (const [scale, fontSize, expected] of [
    [2.4, "16px", 16 / 2.4 / 8],
    [4, "16px", 0.5],
    [8, "16px", 0.25],
  ]) {
    const slider = sliderScene({ scale, fontSize });
    assert.ok(Math.abs(slider.font - expected) < 1e-6);
    assert.ok(Math.abs(slider.font * scale * 8 - 16) < 1e-6);
  }
});

test("facility pixel labels require hover or deliberate focus while controls keep their hit geometry", () => {
  const facility = {
    classes: ["dock-facility"],
    tagName: "BUTTON",
    innerText: "SHIP FORGE",
  };
  const nodes = (options) =>
    pixelScene({ flight: true, buttons: [facility], ...options }).groups[0]
      .nodes;
  assert.equal(nodes({}).filter((n) => n.kind === "facility-label").length, 0);
  const hovered = nodes({ hover: 0 }).find((n) => n.kind === "facility-label");
  assert.equal(hovered.text, "SHIP FORGE");
  assert.equal(hovered.active, true);
  assert.equal(
    nodes({ hover: 0, leaveHover: true }).filter(
      (n) => n.kind === "facility-label",
    ).length,
    0,
  );
  const focused = nodes({
    buttons: [{ ...facility, focusedChild: true }],
  }).find((n) => n.kind === "facility-label");
  assert.equal(focused.active, true);
  assert.deepEqual(Array.from(hovered.rect), Array.from(focused.rect));
});

test("dialogue self-scroll moves full text through a fixed clip without touching actions", () => {
  for (const scale of [1, 1.375, 2.4]) {
    const frames = pixelScene({
      scale,
      rootBounds: { x: 20, y: 20, width: 300, height: 200 },
      paragraph: {
        bounds: { x: 30, y: 60, width: 250, height: 40 },
        scrollHeight: 120,
        scrollTo: [40, 80],
      },
      buttons: [
        {
          bounds: { x: 30, y: 110, width: 250, height: 44 },
          innerText: "NEXT",
        },
      ],
    });
    assert.equal(
      frames.length,
      3,
      "actual captured scroll events invalidate the bridge",
    );
    for (const [index, frame] of frames.entries()) {
      const nodes = frame.groups[0].nodes;
      const text = nodes.find((node) => node.kind === "text");
      const action = nodes.find((node) => node.kind === "button");
      assert.deepEqual(Array.from(text.rect), [
        10 / scale,
        (40 - index * 40) / scale,
        250 / scale,
        120 / scale,
      ]);
      assert.deepEqual(Array.from(text.clip), [
        10 / scale,
        40 / scale,
        250 / scale,
        40 / scale,
      ]);
      assert.deepEqual(Array.from(action.rect), [
        10 / scale,
        90 / scale,
        250 / scale,
        44 / scale,
      ]);
      assert.ok(text.clip[1] + text.clip[3] < action.rect[1]);
      assert.equal(text.text, "A complete briefing to scroll.");
      assert.equal(
        text.font,
        frames[0].groups[0].nodes.find((node) => node.kind === "text").font,
        "scroll preserves native font/width/wrapping inputs",
      );
    }
  }
});

test("dialogue scrolling clamps invalid/overshoot metrics and leaves other paragraphs untouched", () => {
  const bounds = { x: 10, y: 30, width: 200, height: 40 };
  for (const [scrollHeight, scrollTop, y, height] of [
    [120, -10, 30, 120],
    [120, 999, -50, 120],
    [120, NaN, 30, 120],
    [Infinity, 10, 30, 40],
    [-1, 10, 30, 40],
  ]) {
    const node = pixelScene({
      paragraph: { bounds, scrollHeight, scrollTop },
    }).groups[0].nodes.find((node) => node.kind === "text");
    assert.deepEqual(Array.from(node.rect), [10, y, 200, height]);
    assert.deepEqual(Array.from(node.clip), [10, 30, 200, 40]);
  }
  const other = pixelScene({
    paragraph: {
      id: "menu-description",
      bounds,
      scrollHeight: 120,
      scrollTop: 40,
    },
  }).groups[0].nodes.find((node) => node.kind === "text");
  assert.deepEqual(Array.from(other.rect), [10, 30, 200, 40]);
});
