import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";
const source = readFileSync(
  new URL("../../apps/game/src/dialogue.ts", import.meta.url),
  "utf8",
).replace('import "./dialogue.css";', "");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  },
}).outputText;
const portraitSource = readFileSync(
  new URL("../../apps/game/src/kikai-portrait.ts", import.meta.url),
  "utf8",
);
const portraitCompiled = ts.transpileModule(portraitSource, {
  compilerOptions: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  },
}).outputText;
function fixture() {
  let document;
  const listeners = new Map();
  class Element {
    children = [];
    dataset = {};
    hidden = false;
    disabled = false;
    attributes = new Map();
    events = new Map();
    isConnected = true;
    append(...children) {
      this.children.push(...children);
      children.forEach((c) => (c.parent = this));
    }
    setAttribute(k, v) {
      this.attributes.set(k, v);
    }
    removeAttribute(k) {
      this.attributes.delete(k);
    }
    getAttribute(k) {
      return this.attributes.get(k) ?? null;
    }
    addEventListener(name, callback) {
      this.events.set(name, callback);
    }
    focus() {
      document.activeElement = this;
    }
    remove() {
      this.isConnected = false;
    }
  }
  const find = (root, id) =>
    root.id === id
      ? root
      : root.children.map((child) => find(child, id)).find(Boolean);
  document = {
    body: new Element(),
    activeElement: null,
    createElement: () => new Element(),
    getElementById: (id) => find(document.body, id),
  };
  document.activeElement = document.body;
  const exports = {},
    portraitExports = {};
  vm.runInNewContext(portraitCompiled, { exports: portraitExports });
  vm.runInNewContext(compiled, {
    exports,
    require: (name) => {
      assert.equal(name, "./kikai-portrait");
      return portraitExports;
    },
    document,
    HTMLElement: Element,
    window: {
      addEventListener: (n, c) => listeners.set(n, c),
      removeEventListener: (n) => listeners.delete(n),
    },
  });
  let opened = 0,
    closed = 0;
  const panel = new exports.DialoguePanel({
    onOpen: () => opened++,
    onClose: () => closed++,
  });
  const nodes = (root) => [root, ...root.children.flatMap(nodes)];
  return {
    panel,
    document,
    portrait: () =>
      nodes(document.body).find((n) => n.className === "dialogue-portrait"),
    element: (id) => find(document.body, id),
    key: (key) =>
      listeners.get("keydown")({
        key,
        preventDefault() {},
        stopImmediatePropagation() {},
      }),
    opened: () => opened,
    closed: () => closed,
  };
}
test("champion dialogue uses independently manufactured detailed portraits and clears identity for the next guide", () => {
  const f = fixture();
  for (const kind of ["dora", "doro", "talo"]) {
    const identity = { kind, corporation: "renko", sub_corporation: "pan" };
    f.panel.show({
      speaker: "CHAMPION",
      text: "PAN sponsors this RENKO competitor.",
      portraitIdentity: identity,
      expression: 2,
      speaking: true,
    });
    const p = f.portrait();
    assert.equal(p.dataset.portrait, "9");
    assert.equal(
      p.dataset.portraitKind,
      String(["dora", "doro", "talo"].indexOf(kind)),
    );
    assert.equal(p.dataset.portraitMain, "1");
    assert.equal(p.dataset.portraitSub, "0");
    assert.equal(p.dataset.portraitExpression, "2");
    assert.equal(p.dataset.portraitSpeaking, "true");
    assert.match(p.getAttribute("aria-label"), /RED hair, GREEN crystals/);
  }
  f.panel.show({ speaker: "APR-1337", text: "Welcome back.", portrait: 8 });
  const p = f.portrait();
  assert.equal(p.dataset.portrait, "8");
  for (const k of [
    "pixelKind",
    "pixelValue",
    "portraitKind",
    "portraitMain",
    "portraitSub",
  ])
    assert.equal(p.dataset[k], undefined);
  assert.equal(p.getAttribute("aria-label"), null);
});
test("paged dialogue exposes navigation and voice controls without leaking modal focus", () => {
  const f = fixture();
  let next = 0,
    replay = 0;
  f.panel.show({
    speaker: "LYRA",
    text: "Read the brief.",
    portrait: 7,
    page: 0,
    pageCount: 3,
    onNext: () => next++,
    onReplay: () => replay++,
  });
  assert.equal(f.opened(), 1);
  assert.equal(f.element("dialogue-action").hidden, true);
  assert.equal(f.element("dialogue-previous").hidden, true);
  assert.equal(f.element("dialogue-next").textContent, "NEXT 1/3");
  f.element("dialogue-next").events.get("click")();
  f.element("dialogue-replay").events.get("click")();
  assert.equal(next, 1);
  assert.equal(replay, 1);
  assert.equal(f.document.activeElement, f.element("dialogue-next"));
  f.key("Tab");
  assert.equal(f.document.activeElement, f.element("dialogue-replay"));
  f.key("Tab");
  assert.equal(f.document.activeElement, f.element("dialogue-close"));
  f.key("Tab");
  assert.equal(f.document.activeElement, f.element("dialogue-next"));
  f.key("Escape");
  assert.equal(f.panel.opened, false);
  assert.equal(f.closed(), 1);
});
test("a late asynchronous action cannot duplicate a reward or overwrite a newer dialogue page", async () => {
  const f = fixture();
  let finish,
    calls = 0;
  f.panel.show({
    speaker: "LYRA",
    text: "Challenge ready.",
    onAction: () => {
      calls++;
      return new Promise((_, reject) => (finish = reject));
    },
  });
  const first = f.element("dialogue-action").events.get("click")();
  await f.element("dialogue-action").events.get("click")();
  assert.equal(calls, 1);
  f.panel.show({ speaker: "KORR", text: "New page.", onNext: () => {} });
  finish(new Error("obsolete failure"));
  await first;
  assert.equal(f.element("dialogue-text").textContent, "New page.");
  assert.equal(f.element("dialogue-action").hidden, true);
  f.panel.close();
  f.panel.close();
  assert.equal(f.closed(), 1);
  f.panel.dispose();
  assert.equal(f.panel.opened, false);
});

test("presentation tokens change on refresh and close, without changing on reads", () => {
  const f = fixture();
  const before = f.panel.presentationToken;
  f.panel.show({ speaker: "CONTACT", text: "Page one." });
  const first = f.panel.presentationToken;
  assert.ok(first > before);
  assert.equal(f.panel.presentationToken, first);
  f.panel.show({ speaker: "CONTACT", text: "Page two." });
  const second = f.panel.presentationToken;
  assert.ok(second > first);
  f.panel.close();
  const closed = f.panel.presentationToken;
  assert.ok(closed > second);
  f.panel.close();
  assert.equal(f.panel.presentationToken, closed);
});

test("overflowing dialogue text joins keyboard focus while actions remain available", () => {
  const f = fixture();
  f.panel.show({
    speaker: "OPR",
    text: "A complete long briefing.",
    onNext: () => {},
  });
  const text = f.element("dialogue-text");
  text.scrollHeight = 120;
  text.clientHeight = 40;
  assert.equal(text.tabIndex, 0);
  f.key("Tab");
  assert.equal(f.document.activeElement, f.element("dialogue-close"));
  f.key("Tab");
  assert.equal(f.document.activeElement, text);
  f.key("Tab");
  assert.equal(f.document.activeElement, f.element("dialogue-next"));
  text.scrollTop = 65;
  f.panel.show({ speaker: "OPR", text: "The next page.", onNext: () => {} });
  assert.equal(text.scrollTop, 0);
});
