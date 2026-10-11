import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import vm from "node:vm";
import ts from "typescript";
import { WikiView, captureWikiSearch, restoreWikiSearch } from "../src/wiki.ts";

const source = await readFile(
  new URL("../src/main.ts", import.meta.url),
  "utf8",
);
const ast = ts.createSourceFile(
  "main.ts",
  source,
  ts.ScriptTarget.Latest,
  true,
);
const declaration = ast.statements.find(
  (node) => ts.isFunctionDeclaration(node) && node.name?.text === "render",
);
assert.ok(declaration, "Exercise the actual shared workspace render");
const render = ts.transpileModule(declaration.getText(ast), {
  compilerOptions: { target: ts.ScriptTarget.ES2022 },
}).outputText;

class SearchInput extends EventTarget {
  id = "wiki-search";
  selectionStart = 0;
  selectionEnd = 0;
  selectionDirection = "none";
  constructor(document, value) {
    super();
    this.document = document;
    this.value = value;
    this.focusCalls = [];
    this.selectionCalls = [];
  }
  focus(options) {
    this.document.activeElement = this;
    this.focusCalls.push(options);
    // Focusing a replacement input can put its caret at the end.
    this.selectionStart = this.selectionEnd = this.value.length;
    this.selectionDirection = "none";
  }
  setSelectionRange(start, end, direction) {
    this.selectionStart = Math.min(start, this.value.length);
    this.selectionEnd = Math.min(end, this.value.length);
    this.selectionDirection = direction;
    this.selectionCalls.push([start, end, direction]);
  }
}

function fixture() {
  const document = { activeElement: null };
  const wiki = new WikiView();
  wiki.query = "Korr-9 and Lyra";
  wiki.catalog = {
    schemaVersion: 1,
    description: "Actual Wiki reader fixture; no browser acceptance",
    categories: [],
    entries: [],
  };
  const app = {
    input: new SearchInput(document, wiki.query),
    replacements: 0,
    set innerHTML(html) {
      assert.ok(html.includes('id="wiki-search"'));
      this.input = new SearchInput(document, wiki.query);
      this.replacements++;
    },
    querySelector(selector) {
      return selector === "#wiki-search" ? this.input : null;
    },
    querySelectorAll() {
      return [];
    },
  };
  const context = vm.createContext({
    app,
    wiki,
    document,
    captureWikiSearch,
    restoreWikiSearch,
    project: null,
    view: "wiki",
    nav: [["wiki", "DevBible", "ref"]],
    connected: true,
    history: { replaceState() {} },
  });
  vm.runInContext(render, context);
  return { app, document, wiki, render: () => context.render() };
}

test("actual whole-workspace render preserves a focused backward range and middle caret across refreshes", () => {
  const f = fixture();
  const original = f.app.input;
  f.document.activeElement = original;
  original.selectionStart = 2;
  original.selectionEnd = 8;
  original.selectionDirection = "backward";
  f.render(); // Same render used by refreshed snapshots and SSE.
  assert.notEqual(f.app.input, original);
  assert.equal(f.document.activeElement, f.app.input);
  assert.deepEqual(f.app.input.selectionCalls, [[2, 8, "backward"]]);
  assert.deepEqual(f.app.input.focusCalls, [{ preventScroll: true }]);
  f.app.input.selectionStart = f.app.input.selectionEnd = 4;
  f.app.input.selectionDirection = "none";
  f.render();
  assert.deepEqual(f.app.input.selectionCalls, [[4, 4, "none"]]);
  assert.equal(f.wiki.query, "Korr-9 and Lyra");
});

test("actual Wiki input edits keep Unicode UTF-16 selection and focus after shared rerender", () => {
  const f = fixture();
  f.render(); // Installs the actual WikiView.bind input handler.
  const input = f.app.input;
  input.value = "Lyra 🚀 Voss";
  input.selectionStart = 5;
  input.selectionEnd = 7;
  input.selectionDirection = "forward";
  f.document.activeElement = input;
  input.dispatchEvent(new Event("input"));
  assert.equal(f.wiki.query, "Lyra 🚀 Voss");
  assert.notEqual(f.app.input, input);
  assert.equal(f.document.activeElement, f.app.input);
  assert.deepEqual(f.app.input.selectionCalls, [[5, 7, "forward"]]);
});

test("refresh does not steal focus from another control and missing/null selection is safe", () => {
  const f = fixture();
  const other = { id: "wiki-status" };
  f.document.activeElement = other;
  f.render();
  assert.equal(f.document.activeElement, other);
  assert.deepEqual(f.app.input.focusCalls, []);
  f.document.activeElement = f.app.input;
  f.app.input.selectionStart = f.app.input.selectionEnd = null;
  f.app.input.selectionDirection = null;
  f.render();
  assert.equal(f.document.activeElement, f.app.input);
  assert.deepEqual(f.app.input.selectionCalls, []);
  assert.equal(captureWikiSearch({ querySelector: () => null }, other), null);
  assert.doesNotThrow(() =>
    restoreWikiSearch({ querySelector: () => null }, { start: 2, end: 4 }),
  );
});
