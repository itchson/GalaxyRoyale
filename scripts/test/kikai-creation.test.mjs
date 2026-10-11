import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

// Execute the real creator and portrait adapter; DOM fixture does not prove layout.
const compile = (name) =>
  ts.transpileModule(
    readFileSync(
      new URL(`../../apps/game/src/${name}.ts`, import.meta.url),
      "utf8",
    ),
    {
      compilerOptions: {
        target: ts.ScriptTarget.ES2022,
        module: ts.ModuleKind.CommonJS,
      },
    },
  ).outputText;
const creatorSource = compile("kikai"),
  portraitSource = compile("kikai-portrait");
function fixture(create = async () => {}) {
  const keys = new Map();
  let document;
  class Element {
    children = [];
    dataset = {};
    attributes = {};
    hidden = false;
    disabled = false;
    textContent = "";
    append(...children) {
      this.children.push(...children);
    }
    setAttribute(name, value) {
      this.attributes[name] = value;
    }
    getAttribute(name) {
      return this.attributes[name] ?? null;
    }
    addEventListener() {}
    focus() {
      document.activeElement = this;
    }
  }
  document = {
    body: new Element(),
    activeElement: null,
    createElement: () => new Element(),
  };
  const exports = {},
    portraits = {};
  vm.runInNewContext(portraitSource, { exports: portraits });
  vm.runInNewContext(creatorSource, {
    exports,
    document,
    Error,
    require: (name) => (name === "./kikai-portrait" ? portraits : {}),
    crypto: {
      getRandomValues: (bytes) => {
        bytes.set([0x00, 0x01, 0xab, 0xcd, 0xff, 0x22, 0x33, 0x44]);
        return bytes;
      },
    },
    window: { addEventListener: (name, callback) => keys.set(name, callback) },
  });
  let opens = 0,
    closes = 0;
  const creator = new exports.DroidCreation({
    create,
    onOpen: () => opens++,
    onClose: () => closes++,
  });
  const flatten = (n) => [n, ...n.children.flatMap(flatten)];
  const nodes = () => flatten(document.body);
  const choice = (field, value) =>
    nodes().find((n) => n.dataset[field] === value);
  return {
    creator,
    nodes,
    choice,
    opens: () => opens,
    closes: () => closes,
    portrait: () => nodes().find((n) => n.dataset.pixelKind === "portrait"),
    issue: () => nodes().find((n) => n.textContent === "ISSUE LICENSE"),
    close: () =>
      nodes().find(
        (n) => n.attributes["aria-label"] === "Cancel droid creation",
      ),
    escape: () =>
      keys.get("keydown")({
        key: "Escape",
        preventDefault() {},
        stopImmediatePropagation() {},
      }),
  };
}

test("creator exposes DORA/DORO/TALO and updates the actual license preview across 27 combinations", () => {
  const f = fixture();
  f.creator.open();
  const kinds = ["dora", "doro", "talo"],
    corps = ["pan", "renko", "hugo"];
  assert.deepEqual(
    f
      .nodes()
      .filter((n) => n.dataset.droidKind)
      .map((n) => n.textContent),
    ["DORA", "DORO", "TALO"],
  );
  for (const [k, kind] of kinds.entries())
    for (const [m, main] of corps.entries())
      for (const [s, sub] of corps.entries()) {
        f.choice("droidKind", kind).onclick();
        f.choice("corporation", main).onclick();
        f.choice("subCorporation", sub).onclick();
        const portrait = f.portrait();
        assert.equal(portrait.dataset.pixelValue, "9");
        assert.equal(portrait.dataset.portraitKind, String(k));
        assert.equal(portrait.dataset.portraitMain, String(m));
        assert.equal(portrait.dataset.portraitSub, String(s));
        assert.equal(
          f.nodes().filter((n) => n.attributes["aria-pressed"] === "true")
            .length,
          3,
        );
        assert.ok(
          f
            .nodes()
            .some(
              (n) =>
                n.textContent ===
                `${kind.toUpperCase()}-${main.toUpperCase()}-${sub.toUpperCase()}`,
            ),
        );
      }
  assert.ok(
    f.nodes().every((n) => !/female|male|neutral/i.test(n.textContent)),
  );
  assert.equal(f.opens(), 1);
});

test("confirmation issues the previewed identity once and blocks edits/cancellation during the save", async () => {
  let finish;
  const pending = new Promise((resolve) => (finish = resolve)),
    issued = [];
  const f = fixture((identity) => {
    issued.push(identity);
    return pending;
  });
  f.creator.open();
  f.choice("droidKind", "doro").onclick();
  f.choice("corporation", "renko").onclick();
  f.choice("subCorporation", "hugo").onclick();
  f.issue().onclick();
  f.issue().onclick();
  f.escape();
  f.close().onclick();
  assert.equal(f.creator.opened, true);
  assert.equal(issued.length, 1);
  assert.deepEqual(JSON.parse(JSON.stringify(issued[0])), {
    kind: "doro",
    corporation: "renko",
    sub_corporation: "hugo",
    hex_id: "0001abcdff223344",
  });
  assert.ok(
    f
      .nodes()
      .filter((n) => n.type === "button")
      .every((n) => n.disabled),
  );
  finish();
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(f.creator.opened, false);
  assert.equal(f.closes(), 1);
});

test("failed persistence locks the issued portrait and retries the same identity despite attempted choice changes", async () => {
  const issued = [];
  const f = fixture(async (identity) => {
    issued.push(identity);
    throw new Error("Save unavailable");
  });
  f.creator.open();
  f.choice("droidKind", "talo").onclick();
  f.choice("subCorporation", "hugo").onclick();
  f.issue().onclick();
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(f.creator.opened, true);
  assert.equal(f.portrait().dataset.portraitKind, "2");
  assert.equal(f.portrait().dataset.portraitSub, "2");
  assert.ok(f.nodes().some((n) => n.textContent === "Save unavailable"));
  assert.equal(f.issue().disabled, false);
  f.choice("droidKind", "dora").onclick();
  f.choice("corporation", "renko").onclick();
  f.choice("subCorporation", "pan").onclick();
  assert.equal(f.portrait().dataset.portraitKind, "2");
  assert.equal(f.portrait().dataset.portraitMain, "0");
  assert.equal(f.portrait().dataset.portraitSub, "2");
  assert.ok(
    f
      .nodes()
      .filter(
        (n) =>
          n.dataset.droidKind ||
          n.dataset.corporation ||
          n.dataset.subCorporation,
      )
      .every((n) => n.disabled),
  );
  f.issue().onclick();
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(issued.length, 2);
  assert.deepEqual(issued[1], issued[0]);
  f.escape();
  assert.equal(f.creator.opened, false);
  assert.equal(f.closes(), 1);
});
