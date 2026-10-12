import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
const root = "C:/Users/mitch/Documents/ChatGPT/galaxy-royale";
const out = import.meta.dirname;
const sourcePath = resolve(out, "inputs/apps/game/src/loadout.ts");
mkdirSync(resolve(sourcePath, ".."), { recursive: true });
writeFileSync(sourcePath, readFileSync(resolve(root, "apps/game/src/loadout.ts")));
const original = readFileSync(resolve(root, "scripts/test/loadout.test.mjs"), "utf8");
const end = original.indexOf('test("first-upgrade recipe');
if (end < 0) throw new Error("Actual fixture boundary moved");
const prefix = original.slice(0, end).replace('"../../apps/game/src/loadout.ts"', '"./inputs/apps/game/src/loadout.ts"');
const probe = String.raw`
// Actual painter/Root scroll change; fake DOM with declared scroll clamping.
// This is not browser BCR, layout, rendering or native input evidence.
function boundedScroll(f) {
  let offset = 0, max = 200;
  Object.defineProperty(f.target, "scrollTop", {
    configurable: true,
    get: () => offset,
    set: value => { offset = Math.max(0, Math.min(max, value)); },
  });
  const replace = f.target.replaceChildren.bind(f.target);
  f.target.replaceChildren = (...nodes) => { replace(...nodes); offset = 0; };
  return value => { max = value; offset = Math.min(offset, max); };
}
const results = [];
{
  const f = fixture(), h = hud(); boundedScroll(f);
  const before = copy(h);
  f.exports.paintForge(f.target, h, f.actions);
  f.find("browse-abilities").events.click();
  f.target.scrollTop = 63;
  f.find("ability-filter-blueprint").events.click();
  assert.equal(f.target.scrollTop, 63);
  assert.equal(f.document.activeElement, f.find("ability-filter-blueprint"));
  assert.deepEqual(h, before);
  results.push("ability filter retains prior scroll/focus");
}
{
  const f = fixture(), h = hud(); boundedScroll(f);
  f.exports.paintForge(f.target, h, f.actions);
  f.find("manage-0").events.click();
  f.target.scrollTop = 71;
  f.find("ability-category-2").events.click();
  assert.equal(f.target.scrollTop, 71);
  assert.equal(f.document.activeElement, f.find("ability-category-2"));
  f.find("back").events.click();
  assert.equal(f.target.scrollTop, 0);
  assert.equal(f.document.activeElement, f.find("manage-2"));
  results.push("same collection category retains scroll; Back resets overview");
}
{
  const f = fixture(), h = hud(); const max = boundedScroll(f);
  f.exports.paintShipHangar(f.target, h, { craft() {}, select() {} });
  f.target.scrollTop = 89;
  f.find("hull-filter-owned").events.click();
  assert.equal(f.target.scrollTop, 89);
  assert.equal(f.document.activeElement, f.find("hull-filter-owned"));
  max(20);
  f.find("hull-filter-blueprint").events.click();
  assert.equal(f.target.scrollTop, 20);
  assert.equal(f.document.activeElement, f.find("hull-filter-blueprint"));
  results.push("hull filters retain legal scroll and accept clamped shorter content");
}
console.log(JSON.stringify({ passed: results.length, results, scope: "Actual Root loadout.ts with declared fake DOM scroll bounds, no browser/render proof" }, null, 2));
`;
writeFileSync(resolve(out, "scroll-probe.mjs"), prefix + probe);
