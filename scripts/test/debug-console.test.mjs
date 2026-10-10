import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";

const source = readFileSync(
  new URL("../../apps/game/src/debug-console.ts", import.meta.url),
  "utf8",
).replace('import "./debug-console.css";', "");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  },
}).outputText;

// A deterministic DOM/clock exercises the module's lifecycle, not CSS layout.
function harness() {
  const listeners = new Map();
  const frames = new Map();
  const timers = new Map();
  const transitions = [];
  let document;
  let now = 0;
  let nextId = 1;
  let osReduced = false;
  class Element {
    constructor(tag) {
      this.tagName = tag;
      this.children = [];
      this.attributes = new Map();
      this.listeners = new Map();
      this.classes = new Set();
      this.classList = {
        add: (...names) => names.forEach((name) => this.classes.add(name)),
        remove: (...names) =>
          names.forEach((name) => this.classes.delete(name)),
        contains: (name) => this.classes.has(name),
      };
      this.value = "";
      this.textContent = "";
      this.parent = null;
      this.isConnected = true;
      this.scrollHeight = 100;
    }
    set className(value) {
      this.classes = new Set(value.split(/\s+/));
    }
    append(...nodes) {
      for (const node of nodes) {
        node.parent = this;
        this.children.push(node);
      }
    }
    get childElementCount() {
      return this.children.length;
    }
    get firstElementChild() {
      return this.children[0] ?? null;
    }
    remove() {
      if (this.parent)
        this.parent.children = this.parent.children.filter((n) => n !== this);
      this.isConnected = false;
    }
    replaceChildren(...nodes) {
      this.children = [];
      this.append(...nodes);
    }
    setAttribute(key, value) {
      this.attributes.set(key, value);
    }
    removeAttribute(key) {
      this.attributes.delete(key);
    }
    addEventListener(name, callback) {
      const list = this.listeners.get(name) ?? [];
      list.push(callback);
      this.listeners.set(name, list);
    }
    dispatchEvent(event) {
      event.target ??= this;
      if (event.type === "galaxy-console-transition")
        transitions.push({ ...event.detail });
      for (const callback of this.listeners.get(event.type) ?? [])
        callback(event);
      return true;
    }
    focus() {
      document.activeElement = this;
    }
    blur() {
      if (document.activeElement === this)
        document.activeElement = document.body;
    }
    setSelectionRange() {}
    getBoundingClientRect() {
      return { x: 0, y: 0, width: 640, height: 180 };
    }
  }
  function find(element, id) {
    if (element.id === id) return element;
    for (const child of element.children) {
      const result = find(child, id);
      if (result) return result;
    }
    return null;
  }
  const body = new Element("body");
  document = {
    body,
    activeElement: body,
    createElement: (tag) => new Element(tag),
    getElementById: (id) => find(body, id),
  };
  const window = {
    addEventListener(name, callback, capture) {
      assert.equal(capture, true);
      const list = listeners.get(name) ?? [];
      list.push(callback);
      listeners.set(name, list);
    },
    removeEventListener(name, callback) {
      listeners.set(
        name,
        listeners.get(name).filter((f) => f !== callback),
      );
    },
    setTimeout(callback, delay) {
      const id = nextId++;
      timers.set(id, { callback, at: now + delay });
      return id;
    },
  };
  class TestEvent {
    constructor(type, options = {}) {
      this.type = type;
      Object.assign(this, options);
    }
  }
  const exports = {};
  vm.runInNewContext(compiled, {
    exports,
    document,
    window,
    HTMLElement: Element,
    Event: TestEvent,
    CustomEvent: TestEvent,
    matchMedia: () => ({ matches: osReduced }),
    clearTimeout: (id) => timers.delete(id),
    requestAnimationFrame: (callback) => {
      const id = nextId++;
      frames.set(id, callback);
      return id;
    },
    cancelAnimationFrame: (id) => frames.delete(id),
  });
  let ready = true;
  let opens = 0;
  let closes = 0;
  const calls = [];
  let run = (command) => {
    calls.push(command);
    return `RESULT ${command}`;
  };
  const terminal = new exports.DebugConsole({
    ready: () => ready,
    run: (command) => run(command),
    onOpen: () => opens++,
    onClose: () => closes++,
  });
  const screen = document.getElementById("debug-console");
  const card = screen.children[0];
  const input = document.getElementById("console-input");
  const output = document.getElementById("console-output");
  return {
    terminal,
    screen,
    card,
    input,
    output,
    document,
    frames,
    timers,
    listeners,
    transitions,
    calls,
    get opens() {
      return opens;
    },
    get closes() {
      return closes;
    },
    set ready(value) {
      ready = value;
    },
    set run(callback) {
      run = callback;
    },
    set osReduced(value) {
      osReduced = value;
    },
    key(key, extra = {}) {
      const event = {
        key,
        code: key === "`" ? "Backquote" : "",
        repeat: false,
        isComposing: false,
        preventDefault() {
          this.prevented = true;
        },
        stopImmediatePropagation() {
          this.stopped = true;
        },
        ...extra,
      };
      for (const callback of listeners.get("keydown") ?? []) callback(event);
      return event;
    },
    frame() {
      const current = [...frames];
      frames.clear();
      for (const [, callback] of current) callback(now);
    },
    advance(ms) {
      const end = now + ms;
      while (true) {
        const pending = [...timers].sort((a, b) => a[1].at - b[1].at)[0];
        if (!pending || pending[1].at > end) break;
        timers.delete(pending[0]);
        now = pending[1].at;
        pending[1].callback();
      }
      now = end;
    },
    endTransition() {
      card.dispatchEvent({
        type: "transitionend",
        target: card,
        propertyName: "transform",
      });
    },
  };
}
const settle = () => new Promise((resolve) => setImmediate(resolve));

test("drawer retains pause/capture until the closing transform finishes", () => {
  const h = harness();
  const opening = h.key("`");
  assert.equal(opening.stopped, true);
  assert.equal(opening.prevented, true);
  assert.equal(h.terminal.opened, true);
  assert.equal(h.terminal.transitioning, true);
  assert.equal(h.screen.hidden, false);
  assert.equal(h.opens, 1);
  h.frame();
  assert.ok(h.screen.classList.contains("console-visible"));
  h.endTransition();
  assert.equal(h.terminal.transitioning, false);
  h.key("Escape");
  assert.equal(h.terminal.opened, true);
  assert.equal(h.screen.hidden, false);
  assert.equal(h.closes, 0);
  const blocked = h.key("w");
  assert.equal(blocked.stopped, true);
  assert.equal(blocked.prevented, true);
  h.endTransition();
  assert.equal(h.terminal.opened, false);
  assert.equal(h.screen.hidden, true);
  assert.equal(h.closes, 1);
  assert.equal(h.document.activeElement, h.document.body);
  assert.deepEqual(h.transitions, [
    { phase: "opening", durationMs: 180 },
    { phase: "open", durationMs: 0 },
    { phase: "closing", durationMs: 180 },
    { phase: "closed", durationMs: 0 },
  ]);
});

test("closing can reopen without pause callback churn or stale timers", () => {
  const h = harness();
  h.terminal.open();
  h.frame();
  h.endTransition();
  h.terminal.close();
  h.advance(100);
  h.key("`");
  assert.equal(h.opens, 1);
  assert.equal(h.closes, 0);
  assert.ok(h.screen.classList.contains("console-visible"));
  h.advance(220);
  assert.equal(h.terminal.opened, true);
  assert.equal(h.terminal.transitioning, false);
  h.terminal.close();
  h.advance(100);
  h.terminal.close();
  h.advance(120);
  assert.equal(h.closes, 1);
  assert.equal(h.terminal.opened, false);
  h.advance(1000);
  assert.equal(h.closes, 1);
});

test("persisted and OS reduced motion both settle immediately", () => {
  for (const useOS of [false, true]) {
    const h = harness();
    if (useOS) h.osReduced = true;
    else h.document.body.classList.add("reduced-motion");
    h.terminal.open();
    assert.equal(h.terminal.transitioning, false);
    assert.equal(h.frames.size, 0);
    assert.equal(h.timers.size, 0);
    h.terminal.close();
    assert.equal(h.screen.hidden, true);
    assert.equal(h.closes, 1);
    assert.ok(h.transitions.every((event) => event.durationMs === 0));
  }
});

test("readiness and repeat guard toggles; disposal cancels unfinished opening", () => {
  const h = harness();
  h.ready = false;
  assert.equal(h.key("`").prevented, undefined);
  assert.equal(h.terminal.opened, false);
  h.ready = true;
  h.key("`");
  h.key("`", { repeat: true });
  assert.equal(h.opens, 1);
  assert.equal(h.closes, 0);
  h.terminal.dispose();
  h.frame();
  h.advance(1000);
  h.terminal.dispose();
  assert.equal(h.closes, 1);
  assert.equal(h.frames.size, 0);
  assert.equal(h.timers.size, 0);
  assert.equal(h.document.getElementById("debug-console"), null);
  assert.equal(h.listeners.get("keydown").length, 0);
});

test("commands, history/draft and text-only output remain bounded", async () => {
  const h = harness();
  h.osReduced = true;
  h.terminal.open();
  h.input.value = "  STATS  ";
  h.key("Enter");
  await settle();
  assert.equal(h.calls[0], "STATS");
  h.input.value = "NOT SUBMITTED";
  h.key("Enter", { isComposing: true });
  assert.equal(h.calls.length, 1);
  for (let index = 0; index < 33; index++) {
    h.input.value = `C${index}`;
    h.key("Enter");
    await settle();
  }
  h.input.value = "draft";
  for (let index = 0; index < 40; index++) h.key("ArrowUp");
  assert.equal(h.input.value, "C1");
  for (let index = 0; index < 40; index++) h.key("ArrowDown");
  assert.equal(h.input.value, "draft");
  h.terminal.write(
    Array.from({ length: 120 }, (_, index) => `LINE ${index}`).join("\n"),
  );
  assert.equal(h.output.childElementCount, 80);
  assert.equal(h.output.children[0].textContent, "LINE 40");
  h.terminal.write("<img src=x onerror=alert(1)>");
  assert.equal(
    h.output.children.at(-1).textContent,
    "<img src=x onerror=alert(1)>",
  );
  assert.equal(h.output.children.at(-1).childElementCount, 0);
  h.input.value = "X".repeat(500);
  h.key("Enter");
  await settle();
  assert.equal(h.calls.at(-1).length, 256);
  const previousCalls = h.calls.length;
  h.input.value = "CLEAR";
  h.key("Enter");
  assert.equal(h.output.childElementCount, 0);
  assert.equal(h.calls.length, previousCalls);
});

test("async commands cannot overlap and closing ignores submissions", async () => {
  const h = harness();
  let finish;
  h.run = (command) => {
    h.calls.push(command);
    return new Promise((resolve) => (finish = resolve));
  };
  h.terminal.open();
  h.frame();
  h.endTransition();
  h.input.value = "SLOW";
  h.key("Enter");
  h.input.value = "PRESERVED";
  h.key("Enter");
  assert.deepEqual(h.calls, ["SLOW"]);
  finish("DONE");
  await settle();
  assert.equal(h.input.value, "PRESERVED");
  h.terminal.close();
  h.key("Enter");
  assert.deepEqual(h.calls, ["SLOW"]);
  h.terminal.dispose();
});
